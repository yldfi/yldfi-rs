//! HTTP client for the 1inch Swap API v6.0
//!
//! This module provides the client implementation for interacting with
//! the 1inch DEX aggregator API.

use crate::error::{self, Result};
use crate::types::{
    AllowanceResponse, ApiErrorResponse, ApprovalTransaction, Chain, LiquiditySource,
    LiquiditySourcesResponse, QuoteRequest, QuoteResponse, SpenderResponse, SwapRequest,
    SwapResponse, TokenInfo, TokenListResponse,
};
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::time::Duration;
use yldfi_common::http::HttpClientConfig;

/// Default base URL for the 1inch API
pub const DEFAULT_BASE_URL: &str = "https://api.1inch.dev";

/// API version for the swap endpoint
const SWAP_API_VERSION: &str = "v6.0";

/// Configuration for the 1inch API client
#[derive(Debug, Clone)]
pub struct Config {
    /// API key for authentication (required for production use)
    pub api_key: String,
    /// Base URL for the API
    pub base_url: String,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
}

impl Config {
    /// Create a new config with an API key (required)
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your 1inch API key (get one at <https://portal.1inch.dev>)
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: DEFAULT_BASE_URL.to_string(),
            http: HttpClientConfig::default(),
        }
    }

    /// Set a custom base URL
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set a custom timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.http.timeout = timeout;
        self
    }

    /// Set a proxy URL
    #[must_use]
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.http.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn with_optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.http.proxy = proxy;
        self
    }

    /// Set optional proxy URL (alias for consistency with other crates)
    #[must_use]
    pub fn optional_proxy(self, proxy: Option<String>) -> Self {
        self.with_optional_proxy(proxy)
    }
}

/// Client for the 1inch DEX Aggregator Swap API v6.0
///
/// The client requires an API key for authentication. Get one at
/// <https://portal.1inch.dev>.
///
/// # Rate Limits
///
/// - Free tier: 1 request per second, 100,000 calls per month
/// - Higher tiers available for production use
///
/// # Example
///
/// ```no_run
/// use oneinch::{Client, Chain, QuoteRequest};
///
/// #[tokio::main]
/// async fn main() -> Result<(), oneinch::Error> {
///     let client = Client::new("your-api-key")?;
///
///     let request = QuoteRequest::new(
///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
///         "1000000000000000000", // 1 ETH in wei
///     );
///
///     let quote = client.get_quote(Chain::Ethereum, &request).await?;
///     println!("You will receive: {} USDC (in minimal units)", quote.to_amount);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    config: Config,
}

impl Client {
    /// Create a new client with an API key
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your 1inch API key
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    pub fn new(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::new(api_key))
    }

    /// Create a new client with custom configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the HTTP client cannot be initialized.
    pub fn with_config(config: Config) -> Result<Self> {
        let http = yldfi_common::build_client(&config.http)?;
        Ok(Self { http, config })
    }

    /// Get the HTTP client
    #[must_use]
    pub fn http(&self) -> &HttpClient {
        &self.http
    }

    /// Get the configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Build the full URL for a swap API endpoint
    fn swap_url(&self, chain: Chain, endpoint: &str) -> String {
        format!(
            "{}/swap/{}/{}/{}",
            self.config.base_url,
            SWAP_API_VERSION,
            chain.chain_id(),
            endpoint
        )
    }

    /// Make a GET request to the API with query parameters
    async fn get_with_params<T: DeserializeOwned>(
        &self,
        url: &str,
        params: &[(&str, String)],
    ) -> Result<T> {
        let response = self
            .http
            .get(url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Accept", "application/json")
            .query(params)
            .send()
            .await?;

        let status = response.status().as_u16();

        if !response.status().is_success() {
            // Try to parse error response
            let body = response.text().await.unwrap_or_default();

            // Try to parse as API error response
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&body) {
                let message = error_response
                    .description
                    .or(error_response.error)
                    .unwrap_or_else(|| body.clone());

                return Err(error::from_response(status, &message, None));
            }

            return Err(error::from_response(status, &body, None));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Make a GET request to the API without query parameters
    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        self.get_with_params(url, &[]).await
    }

    /// Get a swap quote without transaction data
    ///
    /// This endpoint returns the expected output amount for a swap without
    /// generating the actual transaction data. Use this for price checks
    /// and displaying quotes to users.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to query
    /// * `request` - Quote request parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oneinch::{Client, Chain, QuoteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), oneinch::Error> {
    ///     let client = Client::new("your-api-key")?;
    ///
    ///     let request = QuoteRequest::new(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "1000000000000000000", // 1 ETH
    ///     )
    ///     .with_tokens_info()
    ///     .with_protocols_info();
    ///
    ///     let quote = client.get_quote(Chain::Ethereum, &request).await?;
    ///     println!("Output: {}", quote.to_amount);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_quote(&self, chain: Chain, request: &QuoteRequest) -> Result<QuoteResponse> {
        let url = self.swap_url(chain, "quote");
        let params = request.to_query_params();
        self.get_with_params(&url, &params).await
    }

    /// Get swap transaction data ready for execution
    ///
    /// This endpoint returns complete transaction data that can be signed
    /// and submitted to execute the swap on-chain.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to execute the swap on
    /// * `request` - Swap request parameters including slippage tolerance
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oneinch::{Client, Chain, SwapRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), oneinch::Error> {
    ///     let client = Client::new("your-api-key")?;
    ///
    ///     let request = SwapRequest::new(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "1000000000000000000", // 1 ETH
    ///         "0xYourWalletAddress",
    ///         1.0, // 1% slippage
    ///     );
    ///
    ///     let swap = client.get_swap(Chain::Ethereum, &request).await?;
    ///
    ///     // Use with ethers/alloy to send the transaction
    ///     println!("To: {}", swap.tx.to);
    ///     println!("Data: {}", swap.tx.data);
    ///     println!("Value: {}", swap.tx.value);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_swap(&self, chain: Chain, request: &SwapRequest) -> Result<SwapResponse> {
        let url = self.swap_url(chain, "swap");
        let params = request.to_query_params();
        self.get_with_params(&url, &params).await
    }

    /// Get list of supported tokens on a chain
    ///
    /// Returns a map of token addresses to token information for all
    /// tokens supported by 1inch on the specified chain.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oneinch::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), oneinch::Error> {
    ///     let client = Client::new("your-api-key")?;
    ///     let tokens = client.get_tokens(Chain::Ethereum).await?;
    ///
    ///     println!("Found {} tokens", tokens.len());
    ///     for (address, token) in tokens.iter().take(5) {
    ///         println!("{}: {} ({})", address, token.name, token.symbol);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_tokens(&self, chain: Chain) -> Result<HashMap<String, TokenInfo>> {
        let url = self.swap_url(chain, "tokens");
        let response: TokenListResponse = self.get(&url).await?;
        Ok(response.tokens)
    }

    /// Get list of available liquidity sources (DEXs/protocols) on a chain
    ///
    /// Returns all liquidity sources that 1inch can route through on the
    /// specified chain. Use the protocol IDs in the `protocols` parameter
    /// of quote/swap requests to filter which DEXs to use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oneinch::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), oneinch::Error> {
    ///     let client = Client::new("your-api-key")?;
    ///     let sources = client.get_liquidity_sources(Chain::Ethereum).await?;
    ///
    ///     for source in &sources {
    ///         println!("{}: {}", source.id, source.title);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_liquidity_sources(&self, chain: Chain) -> Result<Vec<LiquiditySource>> {
        let url = self.swap_url(chain, "liquidity-sources");
        let response: LiquiditySourcesResponse = self.get(&url).await?;
        Ok(response.protocols)
    }

    /// Get the 1inch router address that needs token approval
    ///
    /// Before swapping ERC20 tokens, you need to approve the 1inch router
    /// to spend your tokens. This endpoint returns the router address.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oneinch::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), oneinch::Error> {
    ///     let client = Client::new("your-api-key")?;
    ///     let spender = client.get_approve_spender(Chain::Ethereum).await?;
    ///
    ///     println!("Approve tokens to: {}", spender);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_approve_spender(&self, chain: Chain) -> Result<String> {
        let url = self.swap_url(chain, "approve/spender");
        let response: SpenderResponse = self.get(&url).await?;
        Ok(response.address)
    }

    /// Get current token allowance for the 1inch router
    ///
    /// Check how many tokens the 1inch router is currently approved to spend
    /// from a given wallet address.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to query
    /// * `token_address` - The ERC20 token contract address
    /// * `wallet_address` - The wallet address to check allowance for
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oneinch::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), oneinch::Error> {
    ///     let client = Client::new("your-api-key")?;
    ///
    ///     let allowance = client.get_approve_allowance(
    ///         Chain::Ethereum,
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "0xYourWalletAddress",
    ///     ).await?;
    ///
    ///     println!("Current allowance: {}", allowance);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_approve_allowance(
        &self,
        chain: Chain,
        token_address: &str,
        wallet_address: &str,
    ) -> Result<String> {
        let url = self.swap_url(chain, "approve/allowance");
        let params = vec![
            ("tokenAddress", token_address.to_string()),
            ("walletAddress", wallet_address.to_string()),
        ];
        let response: AllowanceResponse = self.get_with_params(&url, &params).await?;
        Ok(response.allowance)
    }

    /// Get transaction data for approving token spending
    ///
    /// Returns transaction data to approve the 1inch router to spend
    /// a specified amount of tokens. If no amount is specified, it
    /// approves the maximum amount (unlimited).
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain
    /// * `token_address` - The ERC20 token contract address
    /// * `amount` - Optional amount to approve (None = unlimited)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oneinch::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), oneinch::Error> {
    ///     let client = Client::new("your-api-key")?;
    ///
    ///     // Get unlimited approval transaction
    ///     let approval_tx = client.get_approve_transaction(
    ///         Chain::Ethereum,
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         None, // unlimited
    ///     ).await?;
    ///
    ///     println!("Send approval to: {}", approval_tx.to);
    ///     println!("Data: {}", approval_tx.data);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_approve_transaction(
        &self,
        chain: Chain,
        token_address: &str,
        amount: Option<&str>,
    ) -> Result<ApprovalTransaction> {
        let url = self.swap_url(chain, "approve/transaction");
        let mut params = vec![("tokenAddress", token_address.to_string())];

        if let Some(amt) = amount {
            params.push(("amount", amt.to_string()));
        }

        self.get_with_params(&url, &params).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = Config::new("test-api-key")
            .with_base_url("https://custom.api.com")
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.api_key, "test-api-key");
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.http.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_client_creation() {
        let client = Client::new("test-api-key");
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.config().api_key, "test-api-key");
        assert_eq!(client.config().base_url, DEFAULT_BASE_URL);
    }

    #[test]
    fn test_swap_url_generation() {
        let client = Client::new("test-api-key").unwrap();

        let url = client.swap_url(Chain::Ethereum, "quote");
        assert_eq!(url, "https://api.1inch.dev/swap/v6.0/1/quote");

        let url = client.swap_url(Chain::Polygon, "swap");
        assert_eq!(url, "https://api.1inch.dev/swap/v6.0/137/swap");

        let url = client.swap_url(Chain::Arbitrum, "tokens");
        assert_eq!(url, "https://api.1inch.dev/swap/v6.0/42161/tokens");
    }

    #[test]
    fn test_custom_base_url() {
        let config = Config::new("test-api-key").with_base_url("https://custom.1inch.io");
        let client = Client::with_config(config).unwrap();

        let url = client.swap_url(Chain::Ethereum, "quote");
        assert_eq!(url, "https://custom.1inch.io/swap/v6.0/1/quote");
    }
}
