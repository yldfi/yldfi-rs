//! Wallet API operations
//!
//! Note: The Tenderly Wallets API has limited endpoints. Wallets can be added
//! (one at a time or in bulk) and queried on specific networks, but there is
//! no list, update, or delete endpoint in the current API.

use super::types::{
    AddWalletRequest, AddWalletResponse, BulkAddWalletsRequest, BulkAddWalletsResponse,
    WalletOnNetwork,
};
use crate::client::{encode_path_segment, Client};
use crate::contracts::{AccountType, ListContractsQuery};
use crate::error::Result;

/// Wallet API client
pub struct WalletsApi<'a> {
    client: &'a Client,
}

impl<'a> WalletsApi<'a> {
    /// Create a new Wallet API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all wallets in the project
    ///
    /// Wallets are stored alongside contracts: this calls
    /// `GET /contracts?accountType=wallet`.
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
        let query = ListContractsQuery::new(AccountType::Wallet);
        self.client.get_with_query("/contracts", &query).await
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
        request.validate().map_err(crate::error::invalid_param)?;

        self.client.post("/wallet", request).await
    }

    /// Add many wallets to the project in one request
    ///
    /// `POST /v2/accounts/{account}/projects/{project}/wallets`. Adds every
    /// address on each of its networks (at most 100 wallet-network pairs).
    /// Addresses the project already has, as wallets or contracts, are left
    /// unchanged and returned under `existing`, so resending the same list is
    /// safe. If any network is unsupported, or the account's address limit
    /// would be exceeded, nothing is added.
    ///
    /// # Errors
    ///
    /// An invalid-parameter error, without sending anything, if
    /// [`BulkAddWalletsRequest::validate`] fails.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tndrly::wallets::{BulkAddWalletsRequest, BulkWallet};
    ///
    /// let request = BulkAddWalletsRequest::new()
    ///     .wallet(BulkWallet::new("0xe2e2...").network("1").network("8453").tag("treasury"))
    ///     .wallet(BulkWallet::new("0xb0b0...").network("1"));
    /// let result = client.wallets().add_many(&request).await?;
    /// println!("added {}, already present {}", result.added.len(), result.existing.len());
    /// ```
    pub async fn add_many(
        &self,
        request: &BulkAddWalletsRequest,
    ) -> Result<BulkAddWalletsResponse> {
        request.validate().map_err(crate::error::invalid_param)?;
        let url = self.client.root_url(&format!(
            "/v2/accounts/{}/projects/{}/wallets",
            encode_path_segment(self.client.account()),
            encode_path_segment(self.client.project())
        ));
        self.client.post_url(&url, request).await
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

    #[tokio::test]
    async fn test_add_many_posts_to_v2_route() {
        use crate::test_util::mock_client;
        use crate::wallets::{BulkAddWalletsRequest, BulkWallet};
        use serde_json::json;
        use wiremock::matchers::{body_json, method, path};
        use wiremock::{Mock, ResponseTemplate};

        let (server, client) = mock_client().await;
        Mock::given(method("POST"))
            .and(path("/api/v2/accounts/acct/projects/proj/wallets"))
            .and(body_json(json!({
                "wallets": [
                    {
                        "address": "0xe2",
                        "network_ids": ["1", "8453"],
                        "display_name": "Treasury",
                        "tags": ["treasury"]
                    },
                    { "address": "0xb0", "network_ids": ["1"] }
                ]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "added": [{
                    "id": "eth:1:0xe2",
                    "account": { "address": "0xe2", "network_id": "1", "type": "wallet" },
                    "display_name": "Treasury",
                    "tags": [{ "tag": "treasury" }]
                }],
                "existing": null
            })))
            .expect(1)
            .mount(&server)
            .await;

        let request = BulkAddWalletsRequest::new()
            .wallet(
                BulkWallet::new("0xe2")
                    .network("1")
                    .network("8453")
                    .display_name("Treasury")
                    .tag("treasury"),
            )
            .wallet(BulkWallet::new("0xb0").network("1"));
        let result = client.wallets().add_many(&request).await.unwrap();
        assert_eq!(result.added.len(), 1);
        assert_eq!(result.added[0].address(), Some("0xe2"));
        assert!(result.added[0].is_wallet());
        assert!(result.existing.is_empty());
    }

    #[test]
    fn test_bulk_add_validation() {
        use crate::wallets::{BulkAddWalletsRequest, BulkWallet};

        assert!(BulkAddWalletsRequest::new().validate().is_err());
        let no_network = BulkAddWalletsRequest::new().wallet(BulkWallet::new("0x1"));
        assert!(no_network.validate().is_err());

        // 101 distinct pairs is over the limit; repeating a pair counts once
        let mut over = BulkAddWalletsRequest::new();
        for i in 0..101 {
            over = over.wallet(BulkWallet::new(format!("0x{i:x}")).network("1"));
        }
        assert!(over.validate().is_err());
        let mut repeated = BulkAddWalletsRequest::new();
        for _ in 0..150 {
            repeated = repeated.wallet(BulkWallet::new("0xAB").network("1"));
        }
        assert!(repeated.validate().is_ok());
    }

    #[test]
    fn test_add_wallet_response_is_bare_array() {
        // POST /wallet returns one project-account entry per network
        let json = serde_json::json!([
            {
                "id": "eth:1:0x0000000000000000000000000000000000000001",
                "account": {
                    "id": "eth:1:0x0000000000000000000000000000000000000001",
                    "address": "0x0000000000000000000000000000000000000001",
                    "network_id": "1",
                    "balance": "1000",
                    "type": "wallet"
                },
                "project_id": "00000000-0000-0000-0000-000000000000",
                "display_name": "Test wallet",
                "account_type": "wallet",
                "added_at": "2024-01-01T00:00:00Z"
            }
        ]);
        let resp: AddWalletResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.len(), 1);
        assert_eq!(resp[0].network_id(), Some("1"));
        assert!(resp[0].is_wallet());
    }
}
