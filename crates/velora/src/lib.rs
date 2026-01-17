//! Rust client for the Velora (ParaSwap) DEX aggregator API
//!
//! Velora (formerly ParaSwap) is a leading DEX aggregator that provides optimal
//! swap routes across multiple DEXs. It offers advanced features like MEV protection,
//! gas optimization, and multi-path routing.
//!
//! # Features
//!
//! - Multi-chain support (Ethereum, BSC, Polygon, Arbitrum, Optimism, etc.)
//! - MEV protection through private transactions
//! - Advanced routing with multi-path swaps
//! - Gas optimization
//! - Delta pricing algorithm
//!
//! # Quick Start
//!
//! ```no_run
//! use velora::{Client, Chain, PriceRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), velora::Error> {
//!     let client = Client::new()?;
//!
//!     // Get price for swapping 1 ETH to USDC
//!     let request = PriceRequest::sell(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 ETH in wei
//!     );
//!
//!     let response = client.get_price(Chain::Ethereum, &request).await?;
//!     println!("You would receive: {} USDC (in minimal units)", response.price_route.dest_amount);
//!
//!     Ok(())
//! }
//! ```

pub mod error;
pub mod types;

pub use error::{Error, Result};
pub use types::{
    ApiErrorResponse, Chain, PriceRequest, PriceResponse, PriceRoute, Route, Side, Swap,
    SwapExchange, Token, TokenListResponse, TransactionRequest, TransactionResponse,
};

// Re-export common utilities
pub use yldfi_common::api::{ApiConfig, BaseClient};
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the Velora API
pub const DEFAULT_BASE_URL: &str = "https://api.paraswap.io";

/// Native token address (used for ETH and other native tokens)
pub const NATIVE_TOKEN_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// Configuration for the Velora API client
///
/// This is a type alias for `ApiConfig` with Velora-specific defaults.
pub type Config = ApiConfig;

/// Create a default Velora config
#[must_use]
pub fn default_config() -> Config {
    ApiConfig::new(DEFAULT_BASE_URL)
}

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    ApiConfig::with_api_key(DEFAULT_BASE_URL, api_key)
}

/// Client for interacting with the Velora (ParaSwap) API
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

    /// Build custom headers for Velora API
    ///
    /// Velora uses `x-partner-apikey` header instead of Bearer token
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        if let Some(api_key) = self.config().get_api_key() {
            if let Ok(value) = reqwest::header::HeaderValue::from_str(api_key) {
                headers.insert(
                    reqwest::header::HeaderName::from_static("x-partner-apikey"),
                    value,
                );
            }
        }

        headers
    }

    /// Get optimal swap price and routing
    ///
    /// Returns the best price and route for swapping tokens.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to get the price for
    /// * `request` - Price request parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// use velora::{Client, Chain, PriceRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), velora::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let request = PriceRequest::sell(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///     );
    ///
    ///     let price = client.get_price(Chain::Ethereum, &request).await?;
    ///     println!("Output: {}", price.price_route.dest_amount);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_price(&self, chain: Chain, request: &PriceRequest) -> Result<PriceResponse> {
        let params = request.to_query_params(chain.chain_id());
        let params_refs: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.base
            .get_with_headers("/prices", &params_refs, self.build_headers())
            .await
    }

    /// Build a transaction from a price route
    ///
    /// Returns transaction data ready for signing and sending.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain for the transaction
    /// * `request` - Transaction request parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// use velora::{Client, Chain, PriceRequest, TransactionRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), velora::Error> {
    ///     let client = Client::new()?;
    ///
    ///     // First get a price
    ///     let price_request = PriceRequest::sell(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///     );
    ///     let price = client.get_price(Chain::Ethereum, &price_request).await?;
    ///
    ///     // Build the transaction
    ///     let tx_request = TransactionRequest::new(
    ///         &price.price_route,
    ///         "0xYourWalletAddress",
    ///         100, // 1% slippage in basis points
    ///     );
    ///
    ///     let tx = client.build_transaction(Chain::Ethereum, &tx_request).await?;
    ///     println!("Send to: {}", tx.to);
    ///     println!("Data: {}", tx.data);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn build_transaction(
        &self,
        chain: Chain,
        request: &TransactionRequest,
    ) -> Result<TransactionResponse> {
        let path = format!("/transactions/{}", chain.chain_id());
        self.base
            .post_json_with_headers(&path, request, self.build_headers())
            .await
    }

    /// Get available tokens for a network
    ///
    /// Returns the list of tokens supported on the specified chain.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to get tokens for
    ///
    /// # Example
    ///
    /// ```no_run
    /// use velora::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), velora::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let tokens = client.get_tokens(Chain::Ethereum).await?;
    ///     for token in &tokens.tokens[..5] {
    ///         println!("{}: {}", token.symbol, token.address);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_tokens(&self, chain: Chain) -> Result<TokenListResponse> {
        let path = format!("/tokens/{}", chain.chain_id());
        self.base
            .get_with_headers::<TokenListResponse, _>(
                &path,
                &[] as &[(&str, &str)],
                self.build_headers(),
            )
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
