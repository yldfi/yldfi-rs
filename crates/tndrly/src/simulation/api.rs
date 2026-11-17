//! Simulation API operations

use super::drafts::{
    CreateSimulationDraftRequest, SimulationDraft, SimulationDraftCreated, SimulationInfoResponse,
};
use super::types::{
    BundleSimulationRequest, BundleSimulationResponse, SimulationListResponse, SimulationRequest,
    SimulationResponse,
};
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Simulation API client
pub struct SimulationApi<'a> {
    client: &'a Client,
}

impl<'a> SimulationApi<'a> {
    /// Create a new simulation API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Simulate a single transaction
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = SimulationRequest::new(
    ///     "0x0000000000000000000000000000000000000000",
    ///     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///     "0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045"
    /// );
    /// let result = client.simulation().simulate(&request).await?;
    /// ```
    pub async fn simulate(&self, request: &SimulationRequest) -> Result<SimulationResponse> {
        self.client.post("/simulate", request).await
    }

    /// Simulate a bundle of transactions in sequence
    ///
    /// Each transaction is simulated on top of the state changes from previous ones.
    pub async fn simulate_bundle(
        &self,
        request: &BundleSimulationRequest,
    ) -> Result<BundleSimulationResponse> {
        self.client.post("/simulate-bundle", request).await
    }

    /// List saved simulations
    ///
    /// # Arguments
    ///
    /// * `page` - Page number (1-indexed; values below 1 are clamped to 1)
    /// * `per_page` - Number of results per page (clamped to 1..=100)
    pub async fn list(&self, page: u32, per_page: u32) -> Result<SimulationListResponse> {
        let query = SimulationListQuery::new(page, per_page);
        self.client.get_with_query("/simulations", &query).await
    }

    /// Get a saved simulation by ID (basic details)
    ///
    /// Returns basic simulation data. For full details including
    /// contracts, transaction traces, and generated access list,
    /// use [`get_full`](Self::get_full) instead.
    pub async fn get(&self, id: &str) -> Result<SimulationResponse> {
        self.client
            .get(&format!("/simulations/{}", encode_path_segment(id)))
            .await
    }

    /// Get full simulation details by ID
    ///
    /// Returns complete simulation data including:
    /// - Full simulation details
    /// - Transaction traces and call data
    /// - Contracts involved
    /// - Generated access list
    ///
    /// This uses POST as per the Tenderly API specification and returns
    /// significantly more data than [`get`](Self::get).
    pub async fn get_full(&self, id: &str) -> Result<SimulationResponse> {
        let empty: serde_json::Value = serde_json::json!({});
        self.client
            .post(&format!("/simulations/{}", encode_path_segment(id)), &empty)
            .await
    }

    /// Get simulation info/metadata by ID
    pub async fn info(&self, id: &str) -> Result<serde_json::Value> {
        self.client
            .get(&format!("/simulations/{}/info", encode_path_segment(id)))
            .await
    }

    /// Get the quick details of a saved simulation, typed
    ///
    /// `GET /simulations/{id}/info`, the same call as [`info`](Self::info),
    /// read into [`SimulationInfoResponse`]: the transaction, its block
    /// header and the state overrides.
    pub async fn get_info(&self, id: &str) -> Result<SimulationInfoResponse> {
        self.client
            .get(&format!("/simulations/{}/info", encode_path_segment(id)))
            .await
    }

    /// Store a Simulator setup as a draft and get its ID
    ///
    /// `POST /v2/account/{account}/project/{project}/simulation-drafts`.
    /// Nothing is simulated: the draft only prefills the dashboard's
    /// Simulator form for whoever opens
    /// [`SimulationDraftCreated::dashboard_url`].
    ///
    /// # Errors
    /// An invalid-parameter error, without sending anything, if the payload
    /// has no rows.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tndrly::simulation::{CreateSimulationDraftRequest, SimulationDraftPayload, SimulationDraftRow};
    ///
    /// let row = SimulationDraftRow::new("0xdac17f958d2ee523a2206206994597c13d831ec7")
    ///     .from("0xab5801a7d398351b8be11c439e05c5b3259aec9b")
    ///     .raw_input("0xa9059cbb...");
    /// let request = CreateSimulationDraftRequest::new(SimulationDraftPayload::new("1").row(row));
    /// let draft = client.simulation().create_draft(&request).await?;
    /// println!("{}", draft.dashboard_url());
    /// ```
    pub async fn create_draft(
        &self,
        request: &CreateSimulationDraftRequest,
    ) -> Result<SimulationDraftCreated> {
        request.validate().map_err(crate::error::invalid_param)?;
        self.client.post_url(&self.drafts_url(""), request).await
    }

    /// Get a stored simulation draft
    ///
    /// `GET /v2/account/{account}/project/{project}/simulation-drafts/{id}`
    pub async fn get_draft(&self, id: &str) -> Result<SimulationDraft> {
        let url = self.drafts_url(&format!("/{}", encode_path_segment(id)));
        self.client.get_url(&url).await
    }

    fn drafts_url(&self, path: &str) -> String {
        self.client.root_url(&format!(
            "/v2{}/simulation-drafts{path}",
            self.client.project_segment()
        ))
    }

    /// Share a simulation publicly
    ///
    /// Returns the public URL for the shared simulation.
    pub async fn share(&self, id: &str) -> Result<String> {
        let empty: serde_json::Value = serde_json::json!({});
        self.client
            .post_no_response(
                &format!("/simulations/{}/share", encode_path_segment(id)),
                &empty,
            )
            .await?;

        Ok(format!(
            "https://dashboard.tenderly.co/shared/simulation/{}",
            encode_path_segment(id)
        ))
    }

    /// Unshare a simulation (make it private)
    pub async fn unshare(&self, id: &str) -> Result<()> {
        let empty: serde_json::Value = serde_json::json!({});
        self.client
            .post_no_response(
                &format!("/simulations/{}/unshare", encode_path_segment(id)),
                &empty,
            )
            .await
    }
}

#[derive(serde::Serialize)]
struct SimulationListQuery {
    page: u32,
    #[serde(rename = "perPage")]
    per_page: u32,
}

impl SimulationListQuery {
    /// Maximum page size accepted by the API
    const MAX_PER_PAGE: u32 = 100;

    /// Build a query that satisfies the API constraints
    /// (`page >= 1`, `1 <= perPage <= 100`).
    fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, Self::MAX_PER_PAGE),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::simulation::{SimulationDraftPayload, SimulationDraftRow};
    use crate::test_util::mock_client;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::{Mock, ResponseTemplate};

    const DRAFTS: &str = "/api/v2/account/acct/project/proj/simulation-drafts";

    #[tokio::test]
    async fn test_create_draft() {
        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path(DRAFTS))
            .and(body_json(json!({
                "payload": {
                    "v": 2,
                    "network": { "id": "1" },
                    "rows": [{
                        "contractAddress": "0xdac1",
                        "inputDataType": "raw",
                        "rawFunctionInput": "0xa9059cbb",
                        "from": "0xab58",
                        "gas": 100000
                    }]
                },
                "shared": true
            })))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(json!({ "resource_id": "draft 1" })),
            )
            .expect(1)
            .mount(&server)
            .await;

        let row = SimulationDraftRow::new("0xdac1")
            .from("0xab58")
            .raw_input("0xa9059cbb")
            .gas(100_000);
        let request = CreateSimulationDraftRequest::new(SimulationDraftPayload::new("1").row(row))
            .shared(true);
        let created = client.simulation().create_draft(&request).await.unwrap();
        assert_eq!(created.resource_id, "draft 1");
        assert_eq!(
            created.dashboard_url(),
            "https://dashboard.tenderly.co/simulator/new?draftId=draft%201"
        );
    }

    #[tokio::test]
    async fn test_create_draft_requires_a_row() {
        let (server, client) = mock_client().await;
        let request = CreateSimulationDraftRequest::new(SimulationDraftPayload::new("1"));
        assert!(client.simulation().create_draft(&request).await.is_err());
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_draft() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path(format!("{DRAFTS}/3f2a")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "payload": {
                    "v": 2,
                    "network": { "id": "1" },
                    "rows": [{
                        "contractAddress": "0xdac1",
                        "inputDataType": "decoded",
                        "functionInputs": ["0xab58", "1000000"],
                        "gasPrice": 12_345_678_901_234_567_890_u64,
                        "blockIndex": null
                    }]
                },
                "account_id": "fa4a",
                "project_id": "aaf3"
            })))
            .expect(1)
            .mount(&server)
            .await;

        let draft = client.simulation().get_draft("3f2a").await.unwrap();
        assert_eq!(draft.payload.v, 2);
        assert_eq!(draft.payload.network.as_ref().unwrap().id, "1");
        let row = &draft.payload.rows[0];
        assert_eq!(row.contract_address, "0xdac1");
        assert_eq!(row.input_data_type.as_deref(), Some("decoded"));
        assert_eq!(row.function_inputs.as_ref().unwrap()[1], "1000000");
        assert_eq!(
            row.gas_price.as_ref().unwrap().to_string(),
            "12345678901234567890"
        );
        assert_eq!(draft.project_id.as_deref(), Some("aaf3"));
    }

    #[tokio::test]
    async fn test_get_info() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path(
                "/api/v1/account/acct/project/proj/simulations/sim1/info",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "transaction_info": {
                    "id": "sim1",
                    "network_id": "1",
                    "block_number": "17884583",
                    "gas_used": "337375",
                    "status": true,
                    "decoded_info": { "method_name": "multiSwap" },
                    "addresses": ["0x3f41"]
                },
                "block_header": { "number": "0x110ace7" },
                "state_overrides": {},
                "created_at": 1_702_050_051_710_u64
            })))
            .expect(1)
            .mount(&server)
            .await;

        let info = client.simulation().get_info("sim1").await.unwrap();
        let tx = info.transaction_info.unwrap();
        assert_eq!(tx.gas_used.as_deref(), Some("337375"));
        assert_eq!(tx.status, Some(true));
        assert_eq!(tx.addresses.unwrap(), vec!["0x3f41".to_string()]);
        assert_eq!(info.block_header.unwrap()["number"], "0x110ace7");
    }

    #[test]
    fn test_simulation_list_query_clamps() {
        let q = SimulationListQuery::new(0, 0);
        assert_eq!((q.page, q.per_page), (1, 1));
        let q = SimulationListQuery::new(3, 500);
        assert_eq!((q.page, q.per_page), (3, 100));
        let q = SimulationListQuery::new(2, 20);
        assert_eq!(
            serde_json::to_value(&q).unwrap(),
            serde_json::json!({"page": 2, "perPage": 20})
        );
    }
    use crate::simulation::AccessListEntry;

    #[test]
    fn test_simulation_request_builder() {
        let request = SimulationRequest::new("0x1234", "0x5678", "0xabcd")
            .network_id("137")
            .value_wei(1_000_000_000_000_000_000u128)
            .gas(100_000)
            .block_number(12_345_678)
            .save(true);

        assert_eq!(request.network_id, "137");
        assert_eq!(request.from, "0x1234");
        assert_eq!(request.to, "0x5678");
        assert_eq!(request.input, "0xabcd");
        assert_eq!(request.value, Some("0xde0b6b3a7640000".to_string()));
        assert_eq!(request.gas, Some(100_000));
        assert_eq!(request.block_number, Some(12_345_678));
        assert!(request.save);
    }

    #[test]
    fn test_simulation_request_state_overrides() {
        let request = SimulationRequest::new("0x1234", "0x5678", "0xabcd")
            .override_balance("0xaaaa", "1000000000000000000")
            .override_storage("0xbbbb", "0x0", "0x1")
            .override_code("0xcccc", "0x6080");

        let overrides = request.state_objects.unwrap();
        assert!(overrides.contains_key("0xaaaa"));
        assert!(overrides.contains_key("0xbbbb"));
        assert!(overrides.contains_key("0xcccc"));
    }

    #[test]
    fn test_simulation_request_gas_estimation() {
        let request = SimulationRequest::new("0x1234", "0x5678", "0xabcd")
            .estimate_gas(true)
            .generate_access_list(true)
            .transaction_index(5);

        assert_eq!(request.estimate_gas, Some(true));
        assert_eq!(request.generate_access_list, Some(true));
        assert_eq!(request.transaction_index, Some(5));
    }

    #[test]
    fn test_simulation_request_access_list() {
        let entry = AccessListEntry::new("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
            .storage_key("0x0000000000000000000000000000000000000000000000000000000000000001")
            .storage_key("0x0000000000000000000000000000000000000000000000000000000000000002");

        let request = SimulationRequest::new("0x1234", "0x5678", "0xabcd").access_list(vec![entry]);

        assert!(request.access_list.is_some());
        let list = request.access_list.unwrap();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].storage_keys.len(), 2);
        // Setting access_list should set transaction type to 1 (EIP-2930)
        assert_eq!(request.transaction_type, Some(1));
    }

    #[test]
    fn test_simulation_request_add_access_list_entry() {
        let entry1 = AccessListEntry::new("0xaaaa");
        let entry2 = AccessListEntry::new("0xbbbb")
            .storage_keys(vec!["0x01".to_string(), "0x02".to_string()]);

        let request = SimulationRequest::new("0x1234", "0x5678", "0xabcd")
            .add_access_list_entry(entry1)
            .add_access_list_entry(entry2);

        let list = request.access_list.unwrap();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].address, "0xaaaa");
        assert_eq!(list[1].address, "0xbbbb");
        assert_eq!(list[1].storage_keys.len(), 2);
    }

    #[test]
    fn test_simulation_request_l2_params() {
        let request = SimulationRequest::new("0x1234", "0x5678", "0xabcd")
            .network_id("10") // Optimism
            .l1_block_number(12_345_678)
            .l1_timestamp(1_700_000_000)
            .l1_message_sender("0xdeadbeef")
            .deposit_tx(true)
            .system_tx(false)
            .mint(1_000_000)
            .amount_to_mint("1000000000000000000");

        assert_eq!(request.network_id, "10");
        assert_eq!(request.l1_block_number, Some(12_345_678));
        assert_eq!(request.l1_timestamp, Some(1_700_000_000));
        assert_eq!(request.l1_message_sender, Some("0xdeadbeef".to_string()));
        assert_eq!(request.deposit_tx, Some(true));
        assert_eq!(request.system_tx, Some(false));
        assert_eq!(request.mint, Some(1_000_000));
        assert_eq!(
            request.amount_to_mint,
            Some("1000000000000000000".to_string())
        );
    }

    #[test]
    fn test_access_list_entry_builder() {
        let entry = AccessListEntry::new("0xcontract")
            .storage_key("0xslot1")
            .storage_key("0xslot2");

        assert_eq!(entry.address, "0xcontract");
        assert_eq!(entry.storage_keys.len(), 2);
        assert_eq!(entry.storage_keys[0], "0xslot1");
        assert_eq!(entry.storage_keys[1], "0xslot2");
    }

    #[test]
    fn test_simulation_request_serialization() {
        let request = SimulationRequest::new("0x1234", "0x5678", "0xabcd")
            .estimate_gas(true)
            .generate_access_list(true);

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"estimate_gas\":true"));
        assert!(json.contains("\"generate_access_list\":true"));
    }
    #[test]
    fn test_simulation_response_reads_nested_transaction_info() {
        // call_trace and logs live under transaction.transaction_info
        let json = serde_json::json!({
            "simulation": {
                "id": "00000000-0000-0000-0000-000000000001",
                "network_id": "1",
                "block_number": 1,
                "from": "0x0000000000000000000000000000000000000001",
                "to": "0x0000000000000000000000000000000000000002",
                "input": "0x",
                "gas": 21000,
                "value": "0",
                "status": true
            },
            "transaction": {
                "hash": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "status": true,
                "transaction_info": {
                    "call_trace": { "calls": [] },
                    "logs": [{ "name": "Transfer" }],
                    "state_diff": [],
                    "asset_changes": null,
                    "console_logs": null
                }
            }
        });
        let resp: crate::simulation::SimulationResponse = serde_json::from_value(json).unwrap();
        let info = resp.transaction.unwrap().transaction_info.unwrap();
        assert!(info.call_trace.is_some());
        assert_eq!(info.logs.len(), 1);
        assert_eq!(info.logs[0]["name"], "Transfer");
        assert!(info.extra.contains_key("console_logs"));

        let null_logs: crate::simulation::TransactionExecutionInfo =
            serde_json::from_value(serde_json::json!({ "logs": null })).unwrap();
        assert!(null_logs.logs.is_empty());
    }
}
