//! HTTP client for the `CoW` Protocol API
//!
//! This client uses the shared `BaseClient` infrastructure from `yldfi_common`
//! while handling `CoW` Protocol's chain-specific base URLs.

use crate::error::{self, Error, Result};
use crate::types::{
    ApiError, Chain, Order, OrderCancellations, OrderCreation, OrderResponse, QuoteRequest,
    QuoteResponse, Trade, TradesQuery,
};
use serde::de::DeserializeOwned;
use std::time::Duration;

use yldfi_common::api::{ApiConfig, BaseClient};

/// Configuration for the `CoW` Protocol API client
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL (placeholder - actual URL is determined by chain)
    base_url: String,
    /// Request timeout
    pub timeout: Duration,
    /// Optional proxy URL
    pub proxy: Option<String>,
    /// Default chain
    pub chain: Chain,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Placeholder URL - actual URL is determined by chain at request time
            base_url: Chain::Mainnet.api_url().to_string(),
            timeout: Duration::from_secs(30),
            proxy: None,
            chain: Chain::Mainnet,
        }
    }
}

impl Config {
    /// Create a new config with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set default chain
    #[must_use]
    pub fn with_chain(mut self, chain: Chain) -> Self {
        self.chain = chain;
        self.base_url = chain.api_url().to_string();
        self
    }

    /// Set a custom timeout
    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a proxy URL
    #[must_use]
    pub fn with_proxy(mut self, proxy: impl Into<String>) -> Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Set optional proxy URL
    #[must_use]
    pub fn with_optional_proxy(mut self, proxy: Option<String>) -> Self {
        self.proxy = proxy;
        self
    }

    /// Set optional proxy URL (alias for consistency with other crates)
    #[must_use]
    pub fn optional_proxy(self, proxy: Option<String>) -> Self {
        self.with_optional_proxy(proxy)
    }

    /// Convert to `ApiConfig` for `BaseClient`
    fn to_api_config(&self) -> ApiConfig {
        ApiConfig::new(&self.base_url)
            .timeout(self.timeout)
            .optional_proxy(self.proxy.clone())
    }
}

/// Client for the `CoW` Protocol (`CowSwap`) API
///
/// `CoW` Protocol provides MEV-protected swaps through batch auctions.
/// Quotes are free and don't require authentication.
///
/// Note: This client uses `BaseClient` for HTTP infrastructure but handles
/// chain-specific base URLs (mainnet, gnosis, arbitrum) separately since
/// each chain has a different API endpoint.
#[derive(Debug, Clone)]
pub struct Client {
    base: BaseClient,
    default_chain: Chain,
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let default_chain = config.chain;
        let base = BaseClient::new(config.to_api_config())?;

        Ok(Self {
            base,
            default_chain,
        })
    }

    /// Get the base URL for a chain
    fn base_url(&self, chain: Option<Chain>) -> &'static str {
        chain.unwrap_or(self.default_chain).api_url()
    }

    /// Make a GET request to the API
    async fn get<T: DeserializeOwned>(&self, chain: Option<Chain>, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url(chain), path);
        let response = self.base.http().get(&url).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the API
    async fn post<T: DeserializeOwned, B: serde::Serialize>(
        &self,
        chain: Option<Chain>,
        path: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url(chain), path);
        let response = self.base.http().post(&url).json(body).send().await?;

        self.handle_response(response).await
    }

    /// Handle API response with `CoW` Protocol-specific error parsing
    ///
    /// `CoW` Protocol returns structured errors for conditions like `NoLiquidity`
    /// and `UnsupportedToken` that we map to domain-specific error types.
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status().as_u16();

        if !response.status().is_success() {
            let body = response.text().await.unwrap_or_default();

            // Try to parse as CoW Protocol API error for domain-specific handling
            if let Ok(api_error) = serde_json::from_str::<ApiError>(&body) {
                return match api_error.error_type.as_str() {
                    "NoLiquidity" => Err(error::insufficient_liquidity()),
                    "UnsupportedToken" => Err(error::invalid_param(format!(
                        "Unsupported token: {}",
                        api_error.description
                    ))),
                    _ => Err(Error::api(status, api_error.description)),
                };
            }

            return Err(Error::from_response(status, &body, None));
        }

        let data = response.json().await?;
        Ok(data)
    }

    /// Get a swap quote
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cowp::{Client, Chain, QuoteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cowp::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let request = QuoteRequest::sell(
    ///         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    ///         "1000000000000000000", // 1 WETH
    ///         "0xYourAddress",
    ///     );
    ///
    ///     let quote = client.get_quote(None, &request).await?;
    ///     println!("Buy amount: {}", quote.quote.buy_amount);
    ///     println!("Fee: {}", quote.quote.fee_amount);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_quote(
        &self,
        chain: Option<Chain>,
        request: &QuoteRequest,
    ) -> Result<QuoteResponse> {
        self.post(chain, "/api/v1/quote", request).await
    }

    /// Submit an order
    ///
    /// Note: The order must be signed before submission.
    pub async fn create_order(
        &self,
        chain: Option<Chain>,
        order: &OrderCreation,
    ) -> Result<String> {
        let response: OrderResponse = self.post(chain, "/api/v1/orders", order).await?;
        Ok(response.uid)
    }

    /// Get order by UID
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cowp::{Client, Chain};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cowp::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let order = client.get_order(None, "0x...order_uid...").await?;
    ///     println!("Status: {:?}", order.status);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_order(&self, chain: Option<Chain>, uid: &str) -> Result<Order> {
        let path = format!("/api/v1/orders/{uid}");
        self.get(chain, &path).await
    }

    /// Get orders by owner address
    pub async fn get_orders_by_owner(
        &self,
        chain: Option<Chain>,
        owner: &str,
    ) -> Result<Vec<Order>> {
        let path = format!("/api/v1/account/{owner}/orders");
        self.get(chain, &path).await
    }

    /// Cancel order by UID
    ///
    /// Note: This requires an EIP-712 signature of `OrderCancellation(bytes orderUid)`.
    #[deprecated(
        since = "0.1.5",
        note = "CoW Protocol deprecated `DELETE /api/v1/orders/{uid}`; use `cancel_orders` with a signed `OrderCancellations` payload"
    )]
    pub async fn cancel_order(
        &self,
        chain: Option<Chain>,
        uid: &str,
        signature: &str,
    ) -> Result<()> {
        let path = format!("/api/v1/orders/{uid}");
        let body = serde_json::json!({
            "signature": signature,
            "signingScheme": "eip712"
        });
        self.delete(chain, &path, &body).await
    }

    /// Cancel one or more orders (`DELETE /api/v1/orders`)
    ///
    /// The signature must be an EIP-712 (or `eth_sign`) signature of
    /// `OrderCancellations(bytes[] orderUids)` from the orders' owner. Up to
    /// 128 orders can be cancelled per request. This is a best-effort
    /// cancellation: solvers may still settle an order that is already part
    /// of an in-flight settlement.
    pub async fn cancel_orders(
        &self,
        chain: Option<Chain>,
        cancellations: &OrderCancellations,
    ) -> Result<()> {
        self.delete(chain, "/api/v1/orders", cancellations).await
    }

    /// Send a DELETE request with a JSON body, expecting no response body
    async fn delete<B: serde::Serialize + ?Sized>(
        &self,
        chain: Option<Chain>,
        path: &str,
        body: &B,
    ) -> Result<()> {
        let url = format!("{}{}", self.base_url(chain), path);
        let response = self.base.http().delete(&url).json(body).send().await?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            return Err(Error::from_response(status, &message, None));
        }

        Ok(())
    }

    /// Get trades (paginated, `GET /api/v2/trades`)
    ///
    /// Results are sorted newest first. When `limit` is unset the API
    /// returns at most 10 trades; page through results by increasing
    /// `offset` until a page shorter than `limit` is returned.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use cowp::{Client, TradesQuery};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), cowp::Error> {
    ///     let client = Client::new()?;
    ///     let query = TradesQuery::by_owner("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
    ///         .limit(100);
    ///     let trades = client.get_trades(None, &query).await?;
    ///     println!("{} trades", trades.len());
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_trades(
        &self,
        chain: Option<Chain>,
        query: &TradesQuery,
    ) -> Result<Vec<Trade>> {
        let path = format!("/api/v2/trades?{}", query.to_query_string());
        self.get(chain, &path).await
    }

    /// Get trades by owner
    #[deprecated(
        since = "0.1.5",
        note = "CoW Protocol deprecated unpaginated `GET /api/v1/trades`; use `get_trades(chain, &TradesQuery::by_owner(owner))`"
    )]
    pub async fn get_trades_by_owner(
        &self,
        chain: Option<Chain>,
        owner: &str,
    ) -> Result<Vec<Trade>> {
        let path = format!("/api/v1/trades?owner={owner}");
        self.get(chain, &path).await
    }

    /// Get trades for an order
    #[deprecated(
        since = "0.1.5",
        note = "CoW Protocol deprecated unpaginated `GET /api/v1/trades`; use `get_trades(chain, &TradesQuery::by_order(uid))`"
    )]
    pub async fn get_trades_by_order(
        &self,
        chain: Option<Chain>,
        order_uid: &str,
    ) -> Result<Vec<Trade>> {
        let path = format!("/api/v1/trades?orderUid={order_uid}");
        self.get(chain, &path).await
    }

    /// Get the current auction
    pub async fn get_auction(&self, chain: Option<Chain>) -> Result<serde_json::Value> {
        self.get(chain, "/api/v1/auction").await
    }

    /// Get solver competition data for a specific auction
    /// (`GET /api/v2/solver_competition/{auction_id}`)
    pub async fn get_solver_competition(
        &self,
        chain: Option<Chain>,
        auction_id: u64,
    ) -> Result<serde_json::Value> {
        let path = format!("/api/v2/solver_competition/{auction_id}");
        self.get(chain, &path).await
    }

    /// Get solver competition data for the most recent auction
    /// (`GET /api/v2/solver_competition/latest`)
    pub async fn get_latest_solver_competition(
        &self,
        chain: Option<Chain>,
    ) -> Result<serde_json::Value> {
        self.get(chain, "/api/v2/solver_competition/latest").await
    }

    /// Get solver competition data by settlement transaction hash
    /// (`GET /api/v2/solver_competition/by_tx_hash/{tx_hash}`)
    pub async fn get_solver_competition_by_tx_hash(
        &self,
        chain: Option<Chain>,
        tx_hash: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/api/v2/solver_competition/by_tx_hash/{tx_hash}");
        self.get(chain, &path).await
    }

    /// Get the native token price in USD
    pub async fn get_native_price(
        &self,
        chain: Option<Chain>,
        token: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/api/v1/token/{token}/native_price");
        self.get(chain, &path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OrderKind;

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_chain_urls() {
        assert_eq!(Chain::Mainnet.api_url(), "https://api.cow.fi/mainnet");
        assert_eq!(Chain::Gnosis.api_url(), "https://api.cow.fi/xdai");
        assert_eq!(Chain::Arbitrum.api_url(), "https://api.cow.fi/arbitrum_one");
    }

    #[test]
    fn test_quote_request_sell() {
        let request = QuoteRequest::sell(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        );

        assert_eq!(request.kind, OrderKind::Sell);
        assert!(request.sell_amount_before_fee.is_some());
        assert!(request.buy_amount_after_fee.is_none());
    }

    #[test]
    fn test_quote_request_buy() {
        let request = QuoteRequest::buy(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        );

        assert_eq!(request.kind, OrderKind::Buy);
        assert!(request.sell_amount_before_fee.is_none());
        assert!(request.buy_amount_after_fee.is_some());
    }

    #[test]
    fn test_trades_query_string() {
        let q = TradesQuery::by_owner("0xabc");
        assert_eq!(q.to_query_string(), "owner=0xabc");
        let q = TradesQuery::by_order("0xdef").offset(20).limit(10);
        assert_eq!(q.to_query_string(), "orderUid=0xdef&offset=20&limit=10");
    }

    #[test]
    fn test_order_cancellations_serialization() {
        let c = OrderCancellations::new(vec!["0x01".into()], "0xsig");
        assert_eq!(
            serde_json::to_value(&c).unwrap(),
            serde_json::json!({
                "orderUids": ["0x01"],
                "signature": "0xsig",
                "signingScheme": "eip712"
            })
        );
        let c = c.with_signing_scheme(crate::types::EcdsaSigningScheme::EthSign);
        assert_eq!(
            serde_json::to_value(&c).unwrap()["signingScheme"],
            "ethsign"
        );
    }

    #[test]
    fn test_chain_parsing() {
        assert_eq!(Chain::try_from_str("mainnet"), Some(Chain::Mainnet));
        assert_eq!(Chain::try_from_str("ethereum"), Some(Chain::Mainnet));
        assert_eq!(Chain::try_from_str("gnosis"), Some(Chain::Gnosis));
        assert_eq!(Chain::try_from_str("arbitrum"), Some(Chain::Arbitrum));
        assert_eq!(Chain::try_from_str("unknown"), None);
    }
}
