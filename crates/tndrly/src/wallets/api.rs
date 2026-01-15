//! Wallet API operations
//!
//! Note: The Tenderly Wallets API has limited endpoints. Wallets can be added
//! and queried on specific networks, but there is no list, update, or delete
//! endpoint in the current API.

use super::types::*;
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Wallet API client
pub struct WalletsApi<'a> {
    client: &'a Client,
}

impl<'a> WalletsApi<'a> {
    /// Create a new Wallet API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all wallets in the project
    ///
    /// Note: Wallets are stored alongside contracts and filtered by account_type.
    /// This method fetches from the contracts endpoint and filters for wallets.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let wallets = client.wallets().list().await?;
    /// for wallet in wallets {
    ///     println!("Wallet: {} on network {}",
    ///         wallet.address().unwrap_or("unknown"),
    ///         wallet.network_id().unwrap_or("unknown"));
    /// }
    /// ```
    pub async fn list(&self) -> Result<Vec<WalletOnNetwork>> {
        // Wallets are returned via /contracts with account_type = "wallet"
        let all: Vec<WalletOnNetwork> = self.client.get("/contracts").await?;
        Ok(all
            .into_iter()
            .filter(|w| w.account_type.as_deref() == Some("wallet"))
            .collect())
    }

    /// Add a wallet to the project for monitoring
    ///
    /// Adds a wallet address to your Tenderly project for monitoring across
    /// the specified networks.
    ///
    /// # Errors
    ///
    /// Returns an error if no networks have been specified. Use `.network()` to add
    /// at least one network to monitor the wallet on.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = AddWalletRequest::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
    ///     .display_name("vitalik.eth")
    ///     .network("1")      // Ethereum mainnet
    ///     .network("137");   // Polygon
    ///
    /// let response = client.wallets().add(&request).await?;
    /// ```
    pub async fn add(&self, request: &AddWalletRequest) -> Result<AddWalletResponse> {
        // Validate that at least one network is specified
        request
            .validate()
            .map_err(crate::error::Error::invalid_param)?;

        self.client.post("/wallet", request).await
    }

    /// Get wallet details on a specific network
    ///
    /// Returns detailed information about a wallet's activity on a specific network,
    /// including balance, transaction count, and other network-specific data.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let wallet = client.wallets()
    ///     .get("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "1")
    ///     .await?;
    /// println!("Balance: {:?}", wallet.balance);
    /// println!("Is contract: {}", wallet.is_contract);
    /// ```
    pub async fn get(&self, address: &str, network_id: &str) -> Result<WalletOnNetwork> {
        self.client
            .get(&format!(
                "/wallet/{}/network/{}",
                encode_path_segment(address),
                encode_path_segment(network_id)
            ))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_wallet_request() {
        let request = AddWalletRequest::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            .display_name("vitalik.eth")
            .network("1")
            .network("137");

        assert_eq!(
            request.address,
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        );
        assert_eq!(request.display_name, Some("vitalik.eth".to_string()));
        assert_eq!(
            request.network_ids,
            vec!["1".to_string(), "137".to_string()]
        );
    }

    #[test]
    fn test_add_wallet_request_serialization() {
        // Verify JSON structure matches Tenderly API expectations
        let request = AddWalletRequest::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            .display_name("vitalik.eth")
            .network("1")
            .network("137");

        let json = serde_json::to_value(&request).unwrap();

        // Verify field names serialize correctly
        assert_eq!(
            json["address"],
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        );
        assert_eq!(json["display_name"], "vitalik.eth");
        assert!(json["network_ids"].is_array());
        assert_eq!(json["network_ids"][0], "1");
        assert_eq!(json["network_ids"][1], "137");
    }
}
