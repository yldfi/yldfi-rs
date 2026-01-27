//! Rust client for the Enso Finance `DeFi` aggregator API
//!
//! Enso Finance is a `DeFi` infrastructure platform that provides routing and
//! execution for complex `DeFi` operations. It supports bundling multiple `DeFi`
//! actions into single transactions and provides smart routing across protocols.
//!
//! # Features
//!
//! - Multi-action bundling (swap + deposit + stake in one tx)
//! - Cross-protocol routing
//! - Position management (enter/exit strategies)
//! - Support for lending, DEXs, yield farming
//! - Gas-efficient batched transactions
//!
//! # Quick Start
//!
//! ```no_run
//! use ensof::{Client, Chain, RouteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), ensof::Error> {
//!     let client = Client::with_api_key("your-api-key")?;
//!
//!     // Get route for swapping tokens
//!     let request = RouteRequest::new(
//!         Chain::Ethereum.chain_id(),
//!         "0xYourAddress",
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 ETH
//!         100, // 1% slippage
//!     );
//!
//!     let response = client.get_route(&request).await?;
//!     println!("Output: {}", response.amount_out);
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::{
    ApiErrorResponse, BundleAction, BundleRequest, BundleResponse, Chain, RouteRequest,
    RouteResponse, RouteStep, RoutingStrategy, TokenBalance, TokenPrice, TransactionData,
};

// Re-export common utilities
pub use yldfi_common::api::{ApiConfig, BaseClient};
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the Enso Finance API
pub const DEFAULT_BASE_URL: &str = "https://api.enso.finance";

/// Native token address (used for ETH and other native tokens)
pub const NATIVE_TOKEN_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// Configuration for the Enso Finance API client
///
/// This is a type alias for `ApiConfig` with Enso-specific defaults.
pub type Config = ApiConfig;

/// Create a default Enso config
#[must_use]
pub fn default_config() -> Config {
    ApiConfig::new(DEFAULT_BASE_URL)
}

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    ApiConfig::with_api_key(DEFAULT_BASE_URL, api_key)
}

/// Client for interacting with the Enso Finance API
#[derive(Debug, Clone)]
pub struct Client {
    base: BaseClient,
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(default_config())
    }

    /// Create a new client with an API key
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(config_with_api_key(api_key))
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let base = BaseClient::new(config)?;
        Ok(Self { base })
    }

    /// Get the underlying base client
    #[must_use]
    pub fn base(&self) -> &BaseClient {
        &self.base
    }

    /// Get the configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        self.base.config()
    }

    /// Get optimal route for token swap
    ///
    /// Returns the best route for swapping tokens with transaction data.
    ///
    /// # Arguments
    ///
    /// * `request` - Route request parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ensof::{Client, Chain, RouteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), ensof::Error> {
    ///     let client = Client::with_api_key("your-key")?;
    ///
    ///     let request = RouteRequest::new(
    ///         1, // Ethereum
    ///         "0xYourAddress",
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///         100,
    ///     );
    ///
    ///     let route = client.get_route(&request).await?;
    ///     println!("Output: {}", route.amount_out);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_route(&self, request: &RouteRequest) -> Result<RouteResponse> {
        self.base
            .post_json("/api/v1/shortcuts/route", request)
            .await
    }

    /// Bundle multiple `DeFi` actions into one transaction
    ///
    /// Allows combining multiple actions (swap, deposit, stake) into a single transaction.
    ///
    /// # Arguments
    ///
    /// * `request` - Bundle request with actions
    ///
    /// # Example
    ///
    /// ```no_run
    /// use ensof::{Client, BundleRequest, BundleAction};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), ensof::Error> {
    ///     let client = Client::with_api_key("your-key")?;
    ///
    ///     let actions = vec![
    ///         BundleAction::swap("0xTokenA", "0xTokenB", "1000000"),
    ///     ];
    ///
    ///     let request = BundleRequest::new(1, "0xYourAddress", actions);
    ///     let bundle = client.bundle(&request).await?;
    ///
    ///     println!("Send to: {}", bundle.tx.to);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn bundle(&self, request: &BundleRequest) -> Result<BundleResponse> {
        self.base
            .post_json("/api/v1/shortcuts/bundle", request)
            .await
    }

    /// Get token price
    ///
    /// # Arguments
    ///
    /// * `chain_id` - Chain ID
    /// * `token` - Token address
    pub async fn get_token_price(&self, chain_id: u64, token: &str) -> Result<TokenPrice> {
        let path = format!("/api/v1/prices/{chain_id}/{token}");
        self.base
            .get::<TokenPrice, _>(&path, &[] as &[(&str, &str)])
            .await
    }

    /// Get token balances for address
    ///
    /// # Arguments
    ///
    /// * `chain_id` - Chain ID
    /// * `address` - Wallet address
    pub async fn get_balances(&self, chain_id: u64, address: &str) -> Result<Vec<TokenBalance>> {
        let path = format!("/api/v1/balances/{chain_id}/{address}");
        self.base
            .get::<Vec<TokenBalance>, _>(&path, &[] as &[(&str, &str)])
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_api_key() {
        let client = Client::with_api_key("test-key");
        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.config().get_api_key(), Some("test-key"));
    }

    #[test]
    fn test_default_config() {
        let config = default_config();
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_config_with_api_key() {
        let config = config_with_api_key("my-key");
        assert_eq!(config.get_api_key(), Some("my-key"));
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
    }
}
