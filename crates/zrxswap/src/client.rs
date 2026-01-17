//! HTTP client for the 0x Swap API v2
//!
//! This module provides the main client for interacting with the 0x API,
//! including support for quotes, prices, and liquidity sources.

use crate::error::{Error, Result};
use crate::types::{
    ApiError, Chain, PriceRequest, PriceResponse, QuoteRequest, QuoteResponse, Source,
    SourcesResponse,
};
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use std::time::Duration;
use yldfi_common::http::HttpClientConfig;

/// Default base URL for the 0x API
pub const DEFAULT_BASE_URL: &str = "https://api.0x.org";

/// API version header value
const API_VERSION: &str = "v2";

/// Configuration for the 0x API client
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL for the API
    pub base_url: String,
    /// API key for authentication (required for production use)
    pub api_key: Option<String>,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            api_key: None,
            http: HttpClientConfig::default(),
        }
    }
}

impl Config {
    /// Create a new config with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new config with an API key
    #[must_use]
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Set a custom base URL
    #[must_use]
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = url.into();
        self
    }

    /// Set a custom timeout
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http.timeout = timeout;
        self
    }

    /// Set a custom timeout (alias for `timeout` for consistency)
    #[must_use]
    pub fn with_timeout(self, timeout: Duration) -> Self {
        self.timeout(timeout)
    }

    /// Set the API key
    #[must_use]
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set a proxy URL
    #[must_use]
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.http.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.http.proxy = proxy;
        self
    }
}

/// Client for the 0x Swap API v2
///
/// The 0x API provides professional-grade DEX aggregation across multiple
/// chains. It uses Permit2 for efficient token approvals and supports
/// gasless trading.
///
/// # API Key
///
/// An API key is required for production use. You can get one at
/// <https://0x.org/docs/introduction/getting-started>.
///
/// # Example
///
/// ```no_run
/// use zrxswap::{Client, Chain, QuoteRequest};
///
/// #[tokio::main]
/// async fn main() -> Result<(), zrxswap::Error> {
///     let client = Client::with_api_key("your-api-key")?;
///
///     // Get an indicative price
///     let request = QuoteRequest::sell(
///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
///         "1000000000000000000", // 1 ETH
///     );
///
///     let price = client.get_price(Chain::Ethereum, &request).await?;
///     println!("You would receive: {} USDC", price.buy_amount);
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
    /// Create a new client with default configuration
    ///
    /// Note: An API key is required for production use. Use `with_api_key()`
    /// or `with_config()` to provide one.
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }

    /// Create a new client with an API key
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zrxswap::Client;
    ///
    /// let client = Client::with_api_key("your-api-key")?;
    /// # Ok::<(), zrxswap::Error>(())
    /// ```
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::with_api_key(api_key))
    }

    /// Create a new client with custom configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zrxswap::{Client, Config};
    /// use std::time::Duration;
    ///
    /// let config = Config::new()
    ///     .api_key("your-api-key")
    ///     .timeout(Duration::from_secs(60));
    ///
    /// let client = Client::with_config(config)?;
    /// # Ok::<(), zrxswap::Error>(())
    /// ```
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

    /// Build the base URL (no chain in path for v2 API)
    fn api_base_url(&self) -> &str {
        &self.config.base_url
    }

    /// Add chainId to query params
    fn add_chain_param(&self, chain: Chain, params: &mut Vec<(String, String)>) {
        params.insert(0, ("chainId".to_string(), chain.chain_id().to_string()));
    }

    /// Build request headers
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();

        // Add version header (required for v2)
        headers.insert(
            reqwest::header::HeaderName::from_static("0x-version"),
            reqwest::header::HeaderValue::from_static(API_VERSION),
        );

        // Add API key if provided
        if let Some(ref api_key) = self.config.api_key {
            if let Ok(value) = reqwest::header::HeaderValue::from_str(api_key) {
                headers.insert(
                    reqwest::header::HeaderName::from_static("0x-api-key"),
                    value,
                );
            }
        }

        headers
    }

    /// Make a GET request to the API
    async fn get<T: DeserializeOwned>(&self, url: &str, params: &[(String, String)]) -> Result<T> {
        let response = self
            .http
            .get(url)
            .headers(self.build_headers())
            .query(params)
            .send()
            .await?;

        let status = response.status();

        if !status.is_success() {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse().ok());

            let body = response.text().await.unwrap_or_default();

            // Try to parse as API error for better error messages
            if let Ok(api_error) = serde_json::from_str::<ApiError>(&body) {
                let message = api_error
                    .message
                    .or(api_error.reason)
                    .unwrap_or_else(|| body.clone());
                return Err(Error::from_response(status.as_u16(), &message, retry_after));
            }

            return Err(Error::from_response(status.as_u16(), &body, retry_after));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Get a swap quote with transaction data
    ///
    /// This endpoint returns a full quote including transaction data that can
    /// be used to execute the swap. Requires a taker address for executable quotes.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to get the quote for
    /// * `request` - Quote request parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zrxswap::{Client, Chain, QuoteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), zrxswap::Error> {
    ///     let client = Client::with_api_key("your-api-key")?;
    ///
    ///     let request = QuoteRequest::sell(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "1000000000000000000", // 1 ETH
    ///     )
    ///     .with_taker("0xYourWalletAddress")
    ///     .with_slippage_bps(100); // 1% slippage
    ///
    ///     let quote = client.get_quote(Chain::Ethereum, &request).await?;
    ///
    ///     if let Some(tx) = quote.transaction {
    ///         println!("Send to: {}", tx.to);
    ///         println!("Data: {}", tx.data);
    ///         println!("Value: {}", tx.value);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_quote(&self, chain: Chain, request: &QuoteRequest) -> Result<QuoteResponse> {
        let url = format!("{}/swap/permit2/quote", self.api_base_url());
        let mut params = request.to_query_params();
        self.add_chain_param(chain, &mut params);
        self.get(&url, &params).await
    }

    /// Get an indicative price (no transaction data)
    ///
    /// This endpoint returns pricing information without generating transaction
    /// data. It's faster and cheaper (in terms of rate limits) than `get_quote()`.
    /// Use this for displaying prices to users before they decide to swap.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to get the price for
    /// * `request` - Price request parameters (same as QuoteRequest)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zrxswap::{Client, Chain, QuoteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), zrxswap::Error> {
    ///     let client = Client::with_api_key("your-api-key")?;
    ///
    ///     let request = QuoteRequest::sell(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///     );
    ///
    ///     let price = client.get_price(Chain::Ethereum, &request).await?;
    ///     println!("Expected output: {} USDC", price.buy_amount);
    ///
    ///     if let Some(sources) = &price.liquidity_sources {
    ///         for source in sources {
    ///             println!("  {} ({}%)", source.name, source.proportion_percent().unwrap_or(0.0));
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_price(&self, chain: Chain, request: &PriceRequest) -> Result<PriceResponse> {
        let url = format!("{}/swap/permit2/price", self.api_base_url());
        let mut params = request.to_query_params();
        self.add_chain_param(chain, &mut params);
        self.get(&url, &params).await
    }

    /// Get available liquidity sources for a chain
    ///
    /// Returns the list of DEXs and liquidity sources that 0x can route through
    /// on the specified chain. Use the source names to exclude specific sources
    /// from quotes using `QuoteRequest::with_excluded_sources()`.
    ///
    /// # Arguments
    ///
    /// * `chain` - The blockchain to get sources for
    ///
    /// # Example
    ///
    /// ```no_run
    /// use zrxswap::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), zrxswap::Error> {
    ///     let client = Client::with_api_key("your-api-key")?;
    ///
    ///     let sources = client.get_sources(Chain::Ethereum).await?;
    ///
    ///     println!("Available sources on Ethereum:");
    ///     for source in &sources {
    ///         println!("  - {}", source.name);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_sources(&self, chain: Chain) -> Result<Vec<Source>> {
        let url = format!("{}/sources", self.api_base_url());
        let mut params = vec![];
        self.add_chain_param(chain, &mut params);
        let response: SourcesResponse = self.get(&url, &params).await?;
        Ok(response.sources)
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
        assert_eq!(client.config().api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .api_key("my-key")
            .base_url("https://custom.api.com")
            .timeout(Duration::from_secs(60));

        assert_eq!(config.api_key, Some("my-key".to_string()));
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.http.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_api_base_url() {
        let client = Client::new().unwrap();
        assert_eq!(client.api_base_url(), "https://api.0x.org");
    }

    #[test]
    fn test_chain_param() {
        let client = Client::new().unwrap();
        let mut params = vec![];
        client.add_chain_param(Chain::Ethereum, &mut params);
        assert_eq!(params, vec![("chainId".to_string(), "1".to_string())]);

        let mut params = vec![];
        client.add_chain_param(Chain::Polygon, &mut params);
        assert_eq!(params, vec![("chainId".to_string(), "137".to_string())]);
    }

    #[test]
    fn test_build_headers() {
        let client = Client::with_api_key("test-api-key").unwrap();
        let headers = client.build_headers();

        assert!(headers.contains_key("0x-version"));
        assert!(headers.contains_key("0x-api-key"));
        assert_eq!(headers.get("0x-version").unwrap(), "v2");
        assert_eq!(headers.get("0x-api-key").unwrap(), "test-api-key");
    }

    #[test]
    fn test_build_headers_no_api_key() {
        let client = Client::new().unwrap();
        let headers = client.build_headers();

        assert!(headers.contains_key("0x-version"));
        assert!(!headers.contains_key("0x-api-key"));
    }
}
