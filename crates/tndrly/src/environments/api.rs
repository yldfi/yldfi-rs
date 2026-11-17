//! Environments API operations

use super::types::{
    CreateEnvironmentRequest, Environment, EnvironmentSimulation, ListEnvironmentTransactionsQuery,
    ListEnvironmentsQuery,
};
use crate::client::{encode_path_segment, Client};
use crate::error::Result;
use crate::vnets::{
    SendVNetTransactionRequest, VNetCallEnvelope, VNetSimulationRequest, VNetTransaction,
};

/// Environments API client
pub struct EnvironmentsApi<'a> {
    client: &'a Client,
}

impl<'a> EnvironmentsApi<'a> {
    /// Create a new Environments API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Full URL of `/public/v1/account/{account}/project/{project}/environments{path}`
    fn url(&self, path: &str) -> String {
        self.client.root_url(&format!(
            "/public/v1{}/environments{path}",
            self.client.project_segment()
        ))
    }

    /// `/{environment}/instance/{instance}` path segment
    fn instance_path(environment_id: &str, instance_id: &str) -> String {
        format!(
            "/{}/instance/{}",
            encode_path_segment(environment_id),
            encode_path_segment(instance_id)
        )
    }

    /// List the project's environments
    ///
    /// `GET .../environments`
    pub async fn list(&self, query: Option<ListEnvironmentsQuery>) -> Result<Vec<Environment>> {
        let url = self.url("");
        match query {
            Some(q) => self.client.get_url_with_query(&url, &q).await,
            None => self.client.get_url(&url).await,
        }
    }

    /// Get an environment by ID
    ///
    /// `GET .../environments/{environmentId}`
    pub async fn get(&self, environment_id: &str) -> Result<Environment> {
        let url = self.url(&format!("/{}", encode_path_segment(environment_id)));
        self.client.get_url(&url).await
    }

    /// Create a multi-chain environment
    ///
    /// `POST .../environments`. Provisions one Virtual `TestNet` per
    /// configured network (forked at the given block, or the latest) and
    /// returns the environment with its active instance.
    ///
    /// # Errors
    /// An invalid-parameter error, without sending anything, if no network
    /// is configured.
    pub async fn create(&self, request: &CreateEnvironmentRequest) -> Result<Environment> {
        request.validate().map_err(crate::error::invalid_param)?;
        self.client.post_url(&self.url(""), request).await
    }

    /// List transactions across the `VNets` of an environment instance
    ///
    /// `GET .../environments/{environmentId}/instance/{instanceId}/transactions`
    pub async fn transactions(
        &self,
        environment_id: &str,
        instance_id: &str,
        query: Option<ListEnvironmentTransactionsQuery>,
    ) -> Result<Vec<VNetTransaction>> {
        let url = self.url(&format!(
            "{}/transactions",
            Self::instance_path(environment_id, instance_id)
        ));
        match query {
            Some(q) => self.client.get_url_with_query(&url, &q.to_pairs()).await,
            None => self.client.get_url(&url).await,
        }
    }

    /// Get one transaction of an environment instance by hash
    ///
    /// `GET .../environments/{environmentId}/instance/{instanceId}/transactions/{txHash}`
    pub async fn get_transaction(
        &self,
        environment_id: &str,
        instance_id: &str,
        tx_hash: &str,
    ) -> Result<VNetTransaction> {
        let url = self.url(&format!(
            "{}/transactions/{}",
            Self::instance_path(environment_id, instance_id),
            encode_path_segment(tx_hash)
        ));
        self.client.get_url(&url).await
    }

    /// Send a transaction on one `VNet` of an environment instance
    ///
    /// `POST .../instance/{instanceId}/vnet/{vnetId}/transactions`. The
    /// transaction is executed and changes the `VNet`'s state. The body is
    /// `{"callArgs": {...}}`, as for
    /// [`VNetsApi::send_transaction`](crate::vnets::VNetsApi::send_transaction).
    pub async fn send_transaction(
        &self,
        environment_id: &str,
        instance_id: &str,
        vnet_id: &str,
        request: &SendVNetTransactionRequest,
    ) -> Result<VNetTransaction> {
        let url = self.url(&format!(
            "{}/vnet/{}/transactions",
            Self::instance_path(environment_id, instance_id),
            encode_path_segment(vnet_id)
        ));
        let envelope = VNetCallEnvelope {
            call_args: request,
            block_number: None,
        };
        self.client.post_url(&url, &envelope).await
    }

    /// Simulate a transaction on one `VNet` of an environment instance
    ///
    /// `POST .../instance/{instanceId}/vnet/{vnetId}/transactions/simulate`.
    /// Runs against the latest block without changing state. The body is
    /// `{"callArgs": {...}, "blockNumber": "latest"}`, as for
    /// [`VNetsApi::simulate`](crate::vnets::VNetsApi::simulate).
    pub async fn simulate_transaction(
        &self,
        environment_id: &str,
        instance_id: &str,
        vnet_id: &str,
        request: &VNetSimulationRequest,
    ) -> Result<EnvironmentSimulation> {
        let url = self.url(&format!(
            "{}/vnet/{}/transactions/simulate",
            Self::instance_path(environment_id, instance_id),
            encode_path_segment(vnet_id)
        ));
        let envelope = VNetCallEnvelope {
            call_args: request,
            block_number: Some("latest"),
        };
        self.client.post_url(&url, &envelope).await
    }
}

#[cfg(test)]
mod tests {
    use crate::environments::{
        BridgeConfig, CreateEnvironmentRequest, EnvironmentNetworkConfig,
        ListEnvironmentTransactionsQuery, ListEnvironmentsQuery,
    };
    use crate::test_util::mock_client;
    use crate::vnets::{SendVNetTransactionRequest, VNetSimulationRequest};
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, ResponseTemplate};

    const BASE: &str = "/api/public/v1/account/acct/project/proj/environments";

    fn environment_json() -> serde_json::Value {
        json!({
            "id": "env1",
            "display_name": "My Multi-Chain Environment",
            "slug": "my-multi-chain-env",
            "created_at": "2026-01-01T00:00:00Z",
            "networks": [{
                "id": "net-1",
                "network_id": "1",
                "network_type": "vnet",
                "block_number": "0x12c50f0",
                "explorer_config": { "enabled": true, "verification_visibility": "abi" },
                "accounts": [{ "address": "0xf39f" }]
            }],
            "bridge_config": { "enabled": true, "relay_mode": "autoRelay", "delay_seconds": 0 },
            "active_instance": {
                "id": "inst1",
                "vnets": [{
                    "id": "vnet1",
                    "status": "running",
                    "rpcs": [
                        { "name": "Admin RPC", "url": "https://admin.example" },
                        { "name": "Public RPC", "url": "https://public.example" }
                    ],
                    "fork_config": { "network_id": 1, "block_number": "0x12c50f0" },
                    "virtual_network_config": { "chain_config": { "chain_id": 1 } },
                    "sync_state_config": { "enabled": false }
                }]
            }
        })
    }

    fn transaction_json() -> serde_json::Value {
        json!({
            "id": "tx-1",
            "vnet_id": "vnet1",
            "status": "success",
            "error_reason": null,
            "reverted": false,
            "tx_hash": "0xabc",
            "block_number": "0x10",
            "from": "0x01",
            "to": "0x02"
        })
    }

    #[tokio::test]
    async fn test_list_environments() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path(BASE))
            .and(query_param("page", "2"))
            .and(query_param("per_page", "5"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([environment_json()])))
            .expect(1)
            .mount(&server)
            .await;

        let query = ListEnvironmentsQuery::new().page(2).per_page(5);
        let envs = client.environments().list(Some(query)).await.unwrap();
        assert_eq!(envs.len(), 1);
        assert_eq!(envs[0].id, "env1");
        assert_eq!(envs[0].networks[0].network_id.as_deref(), Some("1"));
        let vnet = &envs[0].active_instance.as_ref().unwrap().vnets[0];
        assert_eq!(vnet.admin_rpc_url(), Some("https://admin.example"));
        assert_eq!(vnet.public_rpc_url(), Some("https://public.example"));
        assert_eq!(vnet.fork_config.as_ref().unwrap().network_id, 1);
    }

    #[tokio::test]
    async fn test_get_environment() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path(format!("{BASE}/env1")))
            .respond_with(ResponseTemplate::new(200).set_body_json(environment_json()))
            .expect(1)
            .mount(&server)
            .await;

        let env = client.environments().get("env1").await.unwrap();
        assert_eq!(env.slug.as_deref(), Some("my-multi-chain-env"));
        let bridge = env.bridge_config.unwrap();
        assert_eq!(bridge.relay_mode.as_deref(), Some("autoRelay"));
        assert_eq!(bridge.delay_seconds, Some(0));
    }

    #[tokio::test]
    async fn test_create_environment() {
        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path(BASE))
            .and(body_json(json!({
                "display_name": "My Multi-Chain Environment",
                "slug": "my-multi-chain-env",
                "network_configs": [
                    {
                        "network_id": "1",
                        "block_number": "0x12c50f0",
                        "explorer_config": { "enabled": true, "verification_visibility": "abi" }
                    },
                    {
                        "network_id": "8453",
                        "accounts": [{ "address": "0xf39f", "balance": "0xde0b6b3a7640000" }]
                    }
                ],
                "bridge_config": { "enabled": true, "relay_mode": "autoRelay" }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(environment_json()))
            .expect(1)
            .mount(&server)
            .await;

        let request = CreateEnvironmentRequest::new("My Multi-Chain Environment")
            .slug("my-multi-chain-env")
            .network(
                EnvironmentNetworkConfig::new("1")
                    .block_number("19681520")
                    .explorer("abi"),
            )
            .network(EnvironmentNetworkConfig::new("8453").account("0xf39f", "1000000000000000000"))
            .bridge_config(BridgeConfig::auto_relay());
        let env = client.environments().create(&request).await.unwrap();
        assert_eq!(env.id, "env1");
    }

    #[tokio::test]
    async fn test_create_environment_requires_a_network() {
        let (server, client) = mock_client().await;
        let err = client
            .environments()
            .create(&CreateEnvironmentRequest::new("empty"))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("at least one network"), "{err}");
        assert!(server.received_requests().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_environment_transactions_repeat_list_filters() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path(format!("{BASE}/env1/instance/inst1/transactions")))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!([transaction_json()])))
            .expect(1)
            .mount(&server)
            .await;

        let query = ListEnvironmentTransactionsQuery::new()
            .status("success")
            .status("failed")
            .vnet_id("vnet1")
            .per_page(50);
        let txs = client
            .environments()
            .transactions("env1", "inst1", Some(query))
            .await
            .unwrap();
        assert_eq!(txs.len(), 1);
        assert!(txs[0].is_success());

        let requests = server.received_requests().await.unwrap();
        assert_eq!(
            requests[0].url.query(),
            Some("vnet_ids=vnet1&status=success&status=failed&per_page=50")
        );
    }

    #[tokio::test]
    async fn test_get_environment_transaction() {
        let (server, client) = mock_client().await;
        Mock::given(method("GET"))
            .and(path(format!(
                "{BASE}/env1/instance/inst1/transactions/0xabc"
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(transaction_json()))
            .expect(1)
            .mount(&server)
            .await;

        let tx = client
            .environments()
            .get_transaction("env1", "inst1", "0xabc")
            .await
            .unwrap();
        assert_eq!(tx.tx_hash.as_deref(), Some("0xabc"));
        assert_eq!(tx.block_number_as_u64(), Some(16));
    }

    #[tokio::test]
    async fn test_send_environment_transaction() {
        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path(format!(
                "{BASE}/env1/instance/inst1/vnet/vnet1/transactions"
            )))
            .and(body_json(json!({
                "callArgs": { "from": "0x01", "to": "0x02", "input": "0x", "value": "0x64" }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(transaction_json()))
            .expect(1)
            .mount(&server)
            .await;

        let request = SendVNetTransactionRequest::new("0x01", "0x02", "0x").value("100");
        let tx = client
            .environments()
            .send_transaction("env1", "inst1", "vnet1", &request)
            .await
            .unwrap();
        assert_eq!(tx.id.as_deref(), Some("tx-1"));
    }

    #[tokio::test]
    async fn test_simulate_environment_transaction() {
        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path(format!(
                "{BASE}/env1/instance/inst1/vnet/vnet1/transactions/simulate"
            )))
            .and(body_json(json!({
                "callArgs": { "from": "0x01", "to": "0x02", "input": "0x" },
                "blockNumber": "latest"
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "blockNumber": "0x10",
                "gasUsed": "0x5208",
                "status": true,
                "type": "0x2",
                "logs": [],
                "assetChanges": [{ "type": "Transfer", "amount": "1" }],
                "interopInfo": { "chainId": "8453" }
            })))
            .expect(1)
            .mount(&server)
            .await;

        let request = VNetSimulationRequest::new("0x01", "0x").to("0x02");
        let sim = client
            .environments()
            .simulate_transaction("env1", "inst1", "vnet1", &request)
            .await
            .unwrap();
        assert_eq!(sim.status, Some(true));
        assert_eq!(sim.gas_used.as_deref(), Some("0x5208"));
        assert_eq!(sim.tx_type.as_deref(), Some("0x2"));
        assert_eq!(sim.asset_changes.unwrap().len(), 1);
    }
}
