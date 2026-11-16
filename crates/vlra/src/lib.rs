//! Rust client for the Velora (`ParaSwap`) DEX aggregator API
//!
//! Velora (formerly `ParaSwap`) is a leading DEX aggregator that provides optimal
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
//! use vlra::{Client, Chain, PriceRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), vlra::Error> {
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
    SwapExchange, SwapRequest, SwapResponse, Token, TokenListResponse, TransactionRequest,
    TransactionResponse, DEFAULT_PRICE_VERSION,
};

// Re-export common utilities
pub use yldfi_common::api::{ApiConfig, BaseClient};
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the Velora API
/// (formerly `https://api.paraswap.io`, which still serves the same API)
pub const DEFAULT_BASE_URL: &str = "https://api.velora.xyz";

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

/// Client for interacting with the Velora (`ParaSwap`) API
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
    /// use vlra::{Client, Chain, PriceRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), vlra::Error> {
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
    /// use vlra::{Client, Chain, PriceRequest, TransactionRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), vlra::Error> {
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
    ///         &price,
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
        let path = transaction_path(chain, request);
        self.base
            .post_json_with_headers(&path, request, self.build_headers())
            .await
    }

    /// Get a market price route and its ready-to-send transaction in one
    /// call (`GET /swap`)
    ///
    /// Combines [`get_price`](Self::get_price) and
    /// [`build_transaction`](Self::build_transaction). Velora gives this
    /// endpoint lower rate limits and excludes RFQ liquidity; use the
    /// two-step flow when that matters.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use vlra::{Client, Chain, PriceRequest, SwapRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), vlra::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let price = PriceRequest::sell(
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///     )
    ///     .with_src_decimals(18)
    ///     .with_dest_decimals(6);
    ///     let request = SwapRequest::new(price, "0xYourWalletAddress").with_slippage(100);
    ///
    ///     let swap = client.get_swap(Chain::Ethereum, &request).await?;
    ///     println!("Output: {}", swap.price_route.dest_amount);
    ///     println!("Send to: {}", swap.tx_params.to);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_swap(&self, chain: Chain, request: &SwapRequest) -> Result<SwapResponse> {
        let params = request.to_query_params(chain.chain_id());
        let params_refs: Vec<(&str, &str)> = params
            .iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
            .collect();

        self.base
            .get_with_headers("/swap", &params_refs, self.build_headers())
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
    /// use vlra::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), vlra::Error> {
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

/// Path (with query string) for `POST /transactions/{chainId}`.
fn transaction_path(chain: Chain, request: &TransactionRequest) -> String {
    let path = format!("/transactions/{}", chain.chain_id());
    let params = request.to_query_params();
    if params.is_empty() {
        return path;
    }
    let query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(params)
        .finish();
    format!("{path}?{query}")
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
    fn test_price_request_version_param() {
        let find = |params: &[(String, String)]| {
            params
                .iter()
                .find(|(k, _)| k == "version")
                .map(|(_, v)| v.clone())
        };
        let req = PriceRequest::sell("0xa", "0xb", "1");
        assert_eq!(
            find(&req.to_query_params(1)).as_deref(),
            Some(DEFAULT_PRICE_VERSION)
        );
        let req = req.with_version("5");
        assert_eq!(find(&req.to_query_params(1)).as_deref(), Some("5"));
    }

    #[test]
    fn test_transaction_path_carries_ignore_flags() {
        let route = serde_json::json!({
            "blockNumber": 1, "network": 1,
            "srcToken": "0xa", "srcDecimals": 18, "srcAmount": "1",
            "destToken": "0xb", "destDecimals": 6, "destAmount": "2",
            "bestRoute": [], "tokenTransferProxy": "0xc", "contractAddress": "0xc",
            "contractMethod": "swapExactAmountIn", "side": "SELL"
        });
        let req = TransactionRequest::from_raw_price_route(route, "0xd", 100).unwrap();
        assert_eq!(transaction_path(Chain::Ethereum, &req), "/transactions/1");
        let req = req.with_ignore_checks(true).with_ignore_gas(false);
        let path = transaction_path(Chain::Ethereum, &req);
        let url = url::Url::parse(&format!("https://example.invalid{path}")).unwrap();
        assert_eq!(url.path(), "/transactions/1");
        let pairs: Vec<(String, String)> = url.query_pairs().into_owned().collect();
        assert_eq!(
            pairs,
            vec![
                ("ignoreGasEstimate".to_string(), "false".to_string()),
                ("ignoreChecks".to_string(), "true".to_string()),
            ]
        );
    }

    /// A one-shot HTTP server on localhost that answers 200 with `body` and
    /// hands back the request head it received
    fn serve_once(body: String) -> (String, std::sync::mpsc::Receiver<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            if let Ok((mut stream, _)) = listener.accept() {
                let mut buf = [0u8; 8192];
                let n = stream.read(&mut buf).unwrap_or(0);
                let _ = tx.send(String::from_utf8_lossy(&buf[..n]).into_owned());
                let _ = write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
            }
        });
        (format!("http://{addr}"), rx)
    }

    #[tokio::test]
    async fn test_get_swap() {
        let body = serde_json::json!({
            "priceRoute": {
                "blockNumber": 21_000_000, "network": 8453,
                "srcToken": "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", "srcDecimals": 18,
                "srcAmount": "1000000000000000000",
                "destToken": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913", "destDecimals": 6,
                "destAmount": "2500000000",
                "bestRoute": [{"percent": 100, "swaps": [{
                    "srcToken": "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee", "srcDecimals": 18,
                    "destToken": "0x833589fcd6edb6e08f4c7c32d4f71b54bda02913", "destDecimals": 6,
                    "swapExchanges": [{"exchange": "UniswapV3", "srcAmount": "1000000000000000000",
                        "destAmount": "2500000000", "percent": 100, "poolAddresses": ["0xp"]}]
                }]}],
                "gasCostUSD": "0.01", "gasCost": "150000", "side": "SELL", "version": "6.2",
                "contractAddress": "0x6a000f20005980200259b80c5102003040001068",
                "tokenTransferProxy": "0x6a000f20005980200259b80c5102003040001068",
                "contractMethod": "swapExactAmountIn", "partnerFee": 0,
                "srcUSD": "2500", "destUSD": "2499", "partner": "anon",
                "maxImpactReached": false, "hmac": "abc"
            },
            "txParams": {
                "from": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
                "to": "0x6a000f20005980200259b80c5102003040001068",
                "value": "1000000000000000000", "data": "0xe3ead59e",
                "maxFeePerGas": "1000", "maxPriorityFeePerGas": "1",
                "gas": "200000", "chainId": 8453
            }
        });
        let (url, request_head) = serve_once(body.to_string());
        let client = Client::with_config(ApiConfig::new(url)).unwrap();
        let price = PriceRequest::sell(NATIVE_TOKEN_ADDRESS, "0xb", "1000000000000000000");
        let request =
            SwapRequest::new(price, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").with_slippage(50);
        let swap = client.get_swap(Chain::Base, &request).await.unwrap();
        assert_eq!(swap.price_route.dest_amount, "2500000000");
        assert_eq!(swap.price_route.best_route[0].swaps.len(), 1);
        assert_eq!(swap.tx_params.chain_id, 8453);
        assert_eq!(swap.tx_params.data, "0xe3ead59e");
        assert_eq!(swap.tx_params.gas.as_deref(), Some("200000"));

        let head = request_head.recv().unwrap();
        let request_line = head.lines().next().unwrap();
        let target = request_line.split(' ').nth(1).unwrap();
        let url = url::Url::parse(&format!("https://example.invalid{target}")).unwrap();
        assert_eq!(url.path(), "/swap");
        let pairs: std::collections::HashMap<String, String> =
            url.query_pairs().into_owned().collect();
        assert_eq!(pairs["network"], "8453");
        assert_eq!(pairs["side"], "SELL");
        assert_eq!(pairs["version"], DEFAULT_PRICE_VERSION);
        assert_eq!(
            pairs["userAddress"],
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        );
        assert_eq!(pairs["slippage"], "50");
        assert!(!pairs.contains_key("receiver"));
    }

    #[test]
    fn test_swap_request_query_params() {
        let request = SwapRequest::new(PriceRequest::buy("0xa", "0xb", "5"), "0xu")
            .with_receiver("0xr")
            .with_partner_fee("0xp", 20)
            .with_take_surplus(true);
        let params = request.to_query_params(10);
        let get = |k: &str| {
            params
                .iter()
                .find(|(key, _)| key == k)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(get("side"), Some("BUY"));
        assert_eq!(get("network"), Some("10"));
        assert_eq!(get("receiver"), Some("0xr"));
        assert_eq!(get("partnerAddress"), Some("0xp"));
        assert_eq!(get("partnerFeeBps"), Some("20"));
        assert_eq!(get("takeSurplus"), Some("true"));
        assert_eq!(get("slippage"), None);
    }

    #[test]
    fn test_config_with_api_key() {
        let config = config_with_api_key("my-key");
        assert_eq!(config.get_api_key(), Some("my-key"));
        assert_eq!(config.base_url, DEFAULT_BASE_URL);
    }
}
