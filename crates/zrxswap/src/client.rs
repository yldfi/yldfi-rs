//! HTTP client for the 0x Swap API v2
//!
//! This module provides the main client for interacting with the 0x API,
//! including support for quotes, prices, and liquidity sources.

use crate::error::{Error, Result};
use crate::types::{
    ApiError, Chain, ChainsResponse, GaslessPriceResponse, GaslessQuoteResponse,
    GaslessStatusResponse, PriceRequest, PriceResponse, QuoteRequest, QuoteResponse,
    SourcesResponse, SupportedChain,
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
///
/// `Debug` output redacts the API key.
#[derive(Clone)]
pub struct Config {
    /// Base URL for the API
    pub base_url: String,
    /// API key for authentication (required for production use)
    pub api_key: Option<String>,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Config")
            .field("base_url", &self.base_url)
            .field("api_key", &self.api_key.as_ref().map(|_| "[REDACTED]"))
            .field("http", &self.http)
            .finish()
    }
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
            let retry_after = yldfi_common::api::extract_retry_after(response.headers());

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

    /// GET a `/swap/permit2/{price,quote}` endpoint.
    ///
    /// When no route exists the API answers 200 with only
    /// `{"liquidityAvailable": false, "zid": ...}`. Report that as
    /// [`DomainError::NoRouteFound`](crate::error::DomainError::NoRouteFound)
    /// instead of failing to decode the missing quote fields (which surfaced
    /// as a retryable HTTP error).
    async fn get_swap<T: DeserializeOwned>(
        &self,
        url: &str,
        params: &[(String, String)],
    ) -> Result<T> {
        let value: serde_json::Value = self.get(url, params).await?;
        decode_swap_response(value)
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
        self.get_swap(&url, &params).await
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
    /// * `request` - Price request parameters (same as `QuoteRequest`)
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
        self.get_swap(&url, &params).await
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
    ///         println!("  - {source}");
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_sources(&self, chain: Chain) -> Result<Vec<String>> {
        let url = format!("{}/sources", self.api_base_url());
        let mut params = vec![];
        self.add_chain_param(chain, &mut params);
        let response: SourcesResponse = self.get(&url, &params).await?;
        Ok(response.sources)
    }

    /// Get an indicative price using AllowanceHolder for approvals
    ///
    /// Same as [`get_price`](Self::get_price), but for the AllowanceHolder
    /// flow (`/swap/allowance-holder/price`): the taker sets a standard
    /// ERC-20 allowance on `allowance_target` instead of signing a Permit2
    /// message. 0x recommends this flow for most integrators.
    pub async fn get_allowance_holder_price(
        &self,
        chain: Chain,
        request: &PriceRequest,
    ) -> Result<PriceResponse> {
        let url = format!("{}/swap/allowance-holder/price", self.api_base_url());
        let mut params = request.to_query_params();
        self.add_chain_param(chain, &mut params);
        self.get_swap(&url, &params).await
    }

    /// Get a firm quote using AllowanceHolder for approvals
    ///
    /// Same as [`get_quote`](Self::get_quote), but for the AllowanceHolder
    /// flow (`/swap/allowance-holder/quote`). Requires a taker. The response
    /// has no `permit2` data: approve `allowance_target` for the sell token,
    /// then send `transaction` as is.
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
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
    ///         "1000000000", // 1000 USDC
    ///     )
    ///     .with_taker("0xYourWalletAddress");
    ///
    ///     let quote = client
    ///         .get_allowance_holder_quote(Chain::Ethereum, &request)
    ///         .await?;
    ///     println!("Approve: {:?}", quote.allowance_target);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_allowance_holder_quote(
        &self,
        chain: Chain,
        request: &QuoteRequest,
    ) -> Result<QuoteResponse> {
        let url = format!("{}/swap/allowance-holder/quote", self.api_base_url());
        let mut params = request.to_query_params();
        self.add_chain_param(chain, &mut params);
        self.get_swap(&url, &params).await
    }

    /// Get the chains the Swap API supports (`/swap/chains`)
    ///
    /// Includes chains this crate's [`Chain`] enum does not list; see
    /// [`SupportedChain::chain`].
    pub async fn get_swap_chains(&self) -> Result<Vec<SupportedChain>> {
        let url = format!("{}/swap/chains", self.api_base_url());
        let response: ChainsResponse = self.get(&url, &[]).await?;
        Ok(response.chains)
    }

    /// Get the chains the Gasless API supports (`/gasless/chains`)
    pub async fn get_gasless_chains(&self) -> Result<Vec<SupportedChain>> {
        let url = format!("{}/gasless/chains", self.api_base_url());
        let response: ChainsResponse = self.get(&url, &[]).await?;
        Ok(response.chains)
    }

    /// Get an indicative price for a gasless swap (`/gasless/price`)
    ///
    /// Gasless swaps sell an ERC-20 token (not the native token) by
    /// `sellAmount`; the gas is paid from the sell token.
    pub async fn get_gasless_price(
        &self,
        chain: Chain,
        request: &PriceRequest,
    ) -> Result<GaslessPriceResponse> {
        let url = format!("{}/gasless/price", self.api_base_url());
        let mut params = request.to_query_params();
        self.add_chain_param(chain, &mut params);
        self.get_swap(&url, &params).await
    }

    /// Get a firm quote for a gasless swap (`/gasless/quote`)
    ///
    /// Requires a taker. Returns the EIP-712 trade (and, when needed,
    /// approval) to sign; submitting the signatures is not supported by
    /// this crate.
    pub async fn get_gasless_quote(
        &self,
        chain: Chain,
        request: &QuoteRequest,
    ) -> Result<GaslessQuoteResponse> {
        let url = format!("{}/gasless/quote", self.api_base_url());
        let mut params = request.to_query_params();
        self.add_chain_param(chain, &mut params);
        self.get_swap(&url, &params).await
    }

    /// Get the status of a submitted gasless trade
    /// (`/gasless/status/{tradeHash}`)
    ///
    /// # Arguments
    ///
    /// * `chain` - The chain the trade was submitted on
    /// * `trade_hash` - The trade's EIP-712 hash (`trade.hash` of the quote)
    pub async fn get_gasless_status(
        &self,
        chain: Chain,
        trade_hash: &str,
    ) -> Result<GaslessStatusResponse> {
        let url = format!(
            "{}/gasless/status/{}",
            self.api_base_url(),
            encode_path_segment(trade_hash)
        );
        let mut params = vec![];
        self.add_chain_param(chain, &mut params);
        self.get(&url, &params).await
    }
}

/// Percent-encode a value for use as one path segment
fn encode_path_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for b in segment.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Decode a swap price/quote body, mapping `liquidityAvailable: false` to
/// a non-retryable "no route" error.
fn decode_swap_response<T: DeserializeOwned>(value: serde_json::Value) -> Result<T> {
    if value.get("liquidityAvailable") == Some(&serde_json::Value::Bool(false)) {
        return Err(crate::error::no_route_found());
    }
    Ok(serde_json::from_value(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_no_liquidity_is_non_retryable_no_route() {
        use crate::error::DomainError;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/swap/permit2/price"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"liquidityAvailable":false,"zid":"0x1"}"#),
            )
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        let request = PriceRequest::sell("0xa", "0xb", "1");
        let err = client
            .get_price(Chain::Ethereum, &request)
            .await
            .unwrap_err();
        assert!(!err.is_retryable(), "{err:?}");
        assert!(
            matches!(
                err,
                crate::error::ApiError::Domain(DomainError::NoRouteFound)
            ),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn test_allowance_holder_quote() {
        use wiremock::matchers::{header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/swap/allowance-holder/quote"))
            .and(query_param("chainId", "8453"))
            .and(query_param("taker", "0xc"))
            .and(header("0x-version", "v2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "allowanceTarget": "0x0000000000001ff3684f28c67538d4d072c22734",
                "blockNumber": "1",
                "buyAmount": "2", "buyToken": "0xb",
                "sellAmount": "1", "sellToken": "0xa",
                "liquidityAvailable": true,
                "minBuyAmount": "1",
                "transaction": {"to": "0xd", "data": "0x", "gas": null, "gasPrice": "1", "value": "0"},
                "zid": "0x1"
            })))
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        let request = QuoteRequest::sell("0xa", "0xb", "1").with_taker("0xc");
        let quote = client
            .get_allowance_holder_quote(Chain::Base, &request)
            .await
            .unwrap();
        assert_eq!(quote.buy_amount, "2");
        assert!(quote.permit2.is_none());
        assert_eq!(quote.transaction.unwrap().to, "0xd");
    }

    #[tokio::test]
    async fn test_allowance_holder_price_no_route() {
        use crate::error::DomainError;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/swap/allowance-holder/price"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(r#"{"liquidityAvailable":false,"zid":"0x1"}"#),
            )
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        let err = client
            .get_allowance_holder_price(Chain::Ethereum, &PriceRequest::sell("0xa", "0xb", "1"))
            .await
            .unwrap_err();
        assert!(matches!(
            err,
            crate::error::ApiError::Domain(DomainError::NoRouteFound)
        ));
    }

    #[tokio::test]
    async fn test_chains() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        for route in ["/swap/chains", "/gasless/chains"] {
            Mock::given(method("GET"))
                .and(path(route))
                .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "chains": [
                        {"chainId": 1, "chainName": "Ethereum"},
                        {"chainId": 999999, "chainName": "Somechain"}
                    ],
                    "zid": "0x1"
                })))
                .mount(&server)
                .await;
        }
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        for chains in [
            client.get_swap_chains().await.unwrap(),
            client.get_gasless_chains().await.unwrap(),
        ] {
            assert_eq!(chains.len(), 2);
            assert_eq!(chains[0].chain_name, "Ethereum");
            assert_eq!(chains[0].chain(), Some(Chain::Ethereum));
            assert_eq!(chains[1].chain(), None);
        }
    }

    #[tokio::test]
    async fn test_gasless_quote() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gasless/quote"))
            .and(query_param("chainId", "137"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "allowanceTarget": null,
                "approval": null,
                "blockNumber": "1",
                "buyAmount": "2", "buyToken": "0xb",
                "sellAmount": "1", "sellToken": "0xa",
                "fees": {
                    "integratorFee": null,
                    "integratorFees": null,
                    "zeroExFee": {"amount": "3", "token": "0xb", "type": "volume"},
                    "gasFee": {"amount": "4", "token": "0xb", "type": "gas"}
                },
                "liquidityAvailable": true,
                "target": "0xe",
                "trade": {
                    "type": "settler_metatransaction",
                    "hash": "0xfeed",
                    "eip712": {"types": {}, "domain": {"chainId": 137}, "message": {}, "primaryType": "x"}
                },
                "zid": "0x1"
            })))
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        let request = QuoteRequest::sell("0xa", "0xb", "1").with_taker("0xc");
        let quote = client
            .get_gasless_quote(Chain::Polygon, &request)
            .await
            .unwrap();
        assert!(quote.approval.is_none());
        let trade = quote.trade.unwrap();
        assert_eq!(trade.kind, "settler_metatransaction");
        assert_eq!(trade.hash, "0xfeed");
        let fees = quote.fees.unwrap();
        assert_eq!(fees.gas_fee.unwrap().amount, "4");
        assert_eq!(fees.zero_ex_fee.unwrap().fee_type, "volume");
    }

    #[tokio::test]
    async fn test_gasless_price() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gasless/price"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "buyAmount": "2", "buyToken": "0xb",
                "sellAmount": "1", "sellToken": "0xa",
                "liquidityAvailable": true,
                "zid": "0x1"
            })))
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        let price = client
            .get_gasless_price(Chain::Ethereum, &PriceRequest::sell("0xa", "0xb", "1"))
            .await
            .unwrap();
        assert_eq!(price.buy_amount, "2");
        assert!(price.fees.is_none());
    }

    #[tokio::test]
    async fn test_gasless_status() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/gasless/status/0xabc"))
            .and(query_param("chainId", "1"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "status": "failed",
                "reason": "order_expired",
                "transactions": [{"hash": "0xt", "timestamp": 1_700_000_000}],
                "zid": "0x1"
            })))
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        let status = client
            .get_gasless_status(Chain::Ethereum, "0xabc")
            .await
            .unwrap();
        assert_eq!(status.status, "failed");
        assert_eq!(status.reason.as_deref(), Some("order_expired"));
        assert_eq!(status.transactions[0].timestamp, 1_700_000_000);
        assert!(status.approval_transactions.is_empty());
    }

    #[test]
    fn test_encode_path_segment() {
        assert_eq!(encode_path_segment("0xAbC-_.~"), "0xAbC-_.~");
        assert_eq!(encode_path_segment("a/b?c"), "a%2Fb%3Fc");
    }

    #[test]
    fn test_decode_swap_response() {
        let ok: PriceResponse = decode_swap_response(serde_json::json!({
            "liquidityAvailable": true,
            "sellAmount": "1", "buyAmount": "2", "sellToken": "0xa", "buyToken": "0xb"
        }))
        .unwrap();
        assert_eq!(ok.buy_amount, "2");

        // A malformed body is a non-retryable JSON error, not a transport error
        let err =
            decode_swap_response::<PriceResponse>(serde_json::json!({"zid": "0x1"})).unwrap_err();
        assert!(matches!(err, crate::error::ApiError::Json(_)), "{err:?}");
        assert!(!err.is_retryable());
    }

    #[tokio::test]
    async fn test_retry_after_is_capped() {
        use wiremock::matchers::method;
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(429).insert_header("retry-after", "999999"))
            .mount(&server)
            .await;
        let client = Client::with_config(Config::new().base_url(server.uri())).unwrap();
        let err = client
            .get::<serde_json::Value>(&server.uri(), &[])
            .await
            .unwrap_err();
        assert!(err.is_retryable());
        assert_eq!(err.retry_after(), Some(Duration::from_secs(3600)));
    }

    #[test]
    fn test_debug_redacts_api_key() {
        let config = Config::with_api_key("zrx-secret-456");
        let client = Client::with_config(config.clone()).unwrap();
        for debug in [format!("{config:?}"), format!("{client:?}")] {
            assert!(!debug.contains("zrx-secret-456"), "{debug}");
            assert!(debug.contains("[REDACTED]"), "{debug}");
        }
        assert!(format!("{:?}", Config::new()).contains("api_key: None"));
    }

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
