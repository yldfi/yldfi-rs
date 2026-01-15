//! Simulation API operations

use super::types::*;
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Simulation API client
pub struct SimulationApi<'a> {
    client: &'a Client,
}

impl<'a> SimulationApi<'a> {
    /// Create a new simulation API client
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
    /// * `page` - Page number (0-indexed)
    /// * `per_page` - Number of results per page (max 100)
    pub async fn list(&self, page: u32, per_page: u32) -> Result<SimulationListResponse> {
        let query = SimulationListQuery { page, per_page };
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

    /// Trace an existing transaction
    pub async fn trace(&self, hash: &str) -> Result<serde_json::Value> {
        self.client
            .get(&format!("/trace/{}", encode_path_segment(hash)))
            .await
    }
}

#[derive(serde::Serialize)]
struct SimulationListQuery {
    page: u32,
    #[serde(rename = "perPage")]
    per_page: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
