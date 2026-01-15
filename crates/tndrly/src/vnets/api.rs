//! Virtual TestNets API operations

use super::admin_rpc::AdminRpc;
use super::types::*;
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Virtual TestNets API client
pub struct VNetsApi<'a> {
    client: &'a Client,
}

impl<'a> VNetsApi<'a> {
    /// Create a new VNets API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new Virtual TestNet
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = CreateVNetRequest::new("my-testnet", "My TestNet", 1)
    ///     .block_number(18000000)
    ///     .sync_state(true);
    ///
    /// let vnet = client.vnets().create(&request).await?;
    /// println!("Created VNet: {}", vnet.id);
    /// ```
    pub async fn create(&self, request: &CreateVNetRequest) -> Result<VNet> {
        self.client.post("/vnets", request).await
    }

    /// List Virtual TestNets
    ///
    /// Returns a vector of VNets directly (API returns a raw array).
    ///
    /// # Example
    ///
    /// ```ignore
    /// // List all VNets
    /// let vnets = client.vnets().list(None).await?;
    ///
    /// // List with filters
    /// let query = ListVNetsQuery::new()
    ///     .slug("pr-123")
    ///     .per_page(50);
    /// let vnets = client.vnets().list(Some(query)).await?;
    /// ```
    pub async fn list(&self, query: Option<ListVNetsQuery>) -> Result<Vec<VNet>> {
        match query {
            Some(q) => self.client.get_with_query("/vnets", &q).await,
            None => self.client.get("/vnets").await,
        }
    }

    /// Get a Virtual TestNet by ID
    pub async fn get(&self, id: &str) -> Result<VNet> {
        self.client
            .get(&format!("/vnets/{}", encode_path_segment(id)))
            .await
    }

    /// Delete a Virtual TestNet
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/vnets/{}", encode_path_segment(id)))
            .await
    }

    /// Delete multiple Virtual TestNets
    ///
    /// Useful for CI/CD cleanup after test runs.
    pub async fn delete_many(&self, ids: Vec<String>) -> Result<()> {
        let request = DeleteVNetsRequest::multiple(ids);
        self.client.delete_with_body("/vnets", &request).await
    }

    /// Fork a Virtual TestNet
    ///
    /// Creates a new VNet based on the state of an existing one.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = ForkVNetRequest::new("source-vnet-id", "forked-slug", "Forked VNet")
    ///     .block_number(12345678);
    ///
    /// let forked_vnet = client.vnets().fork(&request).await?;
    /// ```
    pub async fn fork(&self, request: &ForkVNetRequest) -> Result<VNet> {
        self.client.post("/vnets/fork", request).await
    }

    /// List transactions on a Virtual TestNet
    ///
    /// Returns transactions as a raw array (API returns JSON array directly).
    ///
    /// # Example
    ///
    /// ```ignore
    /// let txs = client.vnets().transactions("vnet-123", None).await?;
    /// for tx in txs {
    ///     println!("Tx: {} - {}", tx.tx_hash, tx.status.unwrap_or_default());
    /// }
    /// ```
    pub async fn transactions(
        &self,
        vnet_id: &str,
        query: Option<ListVNetTransactionsQuery>,
    ) -> Result<Vec<VNetTransaction>> {
        let path = format!("/vnets/{}/transactions", encode_path_segment(vnet_id));
        match query {
            Some(q) => self.client.get_with_query(&path, &q).await,
            None => self.client.get(&path).await,
        }
    }

    /// Simulate a transaction on a Virtual TestNet
    ///
    /// Unlike the main Simulation API, this simulates against the VNet's state.
    pub async fn simulate(
        &self,
        vnet_id: &str,
        request: &VNetSimulationRequest,
    ) -> Result<serde_json::Value> {
        self.client
            .post(
                &format!(
                    "/vnets/{}/transactions/simulate",
                    encode_path_segment(vnet_id)
                ),
                request,
            )
            .await
    }

    /// Get the RPC URLs for a Virtual TestNet
    pub async fn rpc_urls(&self, vnet_id: &str) -> Result<VNetRpcs> {
        let vnet = self.get(vnet_id).await?;
        vnet.rpcs
            .ok_or_else(|| crate::error::Error::not_found("RPC URLs not available for this VNet"))
    }

    /// Get an Admin RPC client for a Virtual TestNet
    ///
    /// The Admin RPC client provides methods for manipulating the VNet state,
    /// including time warping, balance setting, storage manipulation, and snapshots.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let admin = client.vnets().admin_rpc("vnet-123").await?;
    ///
    /// // Set balance for an address
    /// admin.set_balance("0x1234...", "1000000000000000000").await?;
    ///
    /// // Advance time by 1 hour
    /// admin.increase_time(3600).await?;
    ///
    /// // Create a snapshot
    /// let snapshot_id = admin.snapshot().await?;
    ///
    /// // Do some operations...
    ///
    /// // Revert to snapshot
    /// admin.revert(&snapshot_id).await?;
    /// ```
    pub async fn admin_rpc(&self, vnet_id: &str) -> Result<AdminRpc> {
        let rpcs = self.rpc_urls(vnet_id).await?;
        let admin_url = rpcs.admin().ok_or_else(|| {
            crate::error::Error::not_found("Admin RPC URL not available for this VNet")
        })?;
        AdminRpc::new(admin_url)
    }

    /// Get an Admin RPC client from an existing VNet object
    ///
    /// Use this when you already have the VNet object to avoid an extra API call.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let vnet = client.vnets().create(&request).await?;
    /// let admin = client.vnets().admin_rpc_from_vnet(&vnet)?;
    /// admin.set_balance("0x1234...", "1000000000000000000").await?;
    /// ```
    pub fn admin_rpc_from_vnet(&self, vnet: &VNet) -> Result<AdminRpc> {
        let rpcs = vnet.rpcs.as_ref().ok_or_else(|| {
            crate::error::Error::not_found("RPC URLs not available for this VNet")
        })?;
        let admin_url = rpcs.admin().ok_or_else(|| {
            crate::error::Error::not_found("Admin RPC URL not available for this VNet")
        })?;
        AdminRpc::new(admin_url)
    }

    /// Update a Virtual TestNet
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = UpdateVNetRequest::new()
    ///     .display_name("Updated TestNet Name");
    /// client.vnets().update("vnet-123", &request).await?;
    /// ```
    pub async fn update(&self, id: &str, request: &UpdateVNetRequest) -> Result<VNet> {
        self.client
            .patch(&format!("/vnets/{}", encode_path_segment(id)), request)
            .await
    }

    /// Send a transaction to be executed on a Virtual TestNet
    ///
    /// Unlike `simulate`, this actually modifies the VNet's state.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = SendVNetTransactionRequest::new(
    ///     "0x1234...",  // from
    ///     "0x5678...",  // to
    ///     "0xa9059cbb..." // data (transfer)
    /// ).value("1000000000000000000");  // 1 ETH
    ///
    /// let tx = client.vnets().send_transaction("vnet-123", &request).await?;
    /// ```
    pub async fn send_transaction(
        &self,
        vnet_id: &str,
        request: &SendVNetTransactionRequest,
    ) -> Result<VNetTransaction> {
        self.client
            .post(
                &format!("/vnets/{}/transactions", encode_path_segment(vnet_id)),
                request,
            )
            .await
    }

    /// Get a specific transaction from a Virtual TestNet
    ///
    /// # Example
    ///
    /// ```ignore
    /// let tx = client.vnets().get_transaction(
    ///     "vnet-123",
    ///     "0xabc123..."
    /// ).await?;
    /// ```
    pub async fn get_transaction(&self, vnet_id: &str, tx_hash: &str) -> Result<VNetTransaction> {
        self.client
            .get(&format!(
                "/vnets/{}/transactions/{}",
                encode_path_segment(vnet_id),
                encode_path_segment(tx_hash)
            ))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vnet_request_builder() {
        let request = CreateVNetRequest::new("test-vnet", "Test VNet", 1)
            .block_number(18_000_000)
            .chain_id(31_337)
            .sync_state(true);

        assert_eq!(request.slug, "test-vnet");
        assert_eq!(request.display_name, "Test VNet");
        assert_eq!(request.fork_config.network_id, 1);
        assert_eq!(request.fork_config.block_number, Some(18_000_000));
        assert_eq!(request.virtual_network_config.chain_config.chain_id, 31_337);
        assert!(request.sync_state_config.is_some());
    }

    #[test]
    fn test_list_query_builder() {
        let query = ListVNetsQuery::new().slug("pr-").page(2).per_page(50);

        assert_eq!(query.slug, Some("pr-".to_string()));
        assert_eq!(query.page, Some(2));
        assert_eq!(query.per_page, Some(50));
    }

    #[test]
    fn test_create_vnet_request_serialization() {
        // This test ensures the JSON structure matches what the Tenderly API expects
        let request = CreateVNetRequest::new("test-vnet", "Test VNet", 1).chain_id(31_337);

        let json = serde_json::to_value(&request).unwrap();

        // chain_id must be nested inside chain_config, not at virtual_network_config level
        assert!(
            json["virtual_network_config"]["chain_config"]["chain_id"].is_number(),
            "chain_id must be nested in chain_config: {}",
            serde_json::to_string_pretty(&json).unwrap()
        );
        assert_eq!(
            json["virtual_network_config"]["chain_config"]["chain_id"],
            31_337
        );

        // Ensure chain_id is NOT at the wrong level
        assert!(
            json["virtual_network_config"]["chain_id"].is_null(),
            "chain_id should not be directly in virtual_network_config"
        );
    }
}
