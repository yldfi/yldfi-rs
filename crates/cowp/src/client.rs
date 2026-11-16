//! HTTP client for the `CoW` Protocol API
//!
//! This client uses the shared `BaseClient` infrastructure from `yldfi_common`
//! while handling `CoW` Protocol's chain-specific base URLs.

use crate::error::{self, Error, Result};
use crate::types::{
    ApiError, AppDataObject, Chain, CompetitionOrderStatus, Order, OrderCancellations,
    OrderCreation, OrderLookup, OrderResponse, QuoteRequest, QuoteResponse, TotalSurplus, Trade,
    TradesQuery,
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
    /// Base URL used for every chain instead of the chain's URL
    base_url_override: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            // Placeholder URL - actual URL is determined by chain at request time
            base_url: Chain::Mainnet.api_url().to_string(),
            timeout: Duration::from_secs(30),
            proxy: None,
            chain: Chain::Mainnet,
            base_url_override: None,
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

    /// Send every request to `url` (e.g. a self-hosted orderbook or a
    /// staging API, including the network path segment) instead of the
    /// chain's `api.cow.fi` URL. The `chain` argument of each call is then
    /// ignored.
    #[must_use]
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        let url = url.into();
        self.base_url.clone_from(&url);
        self.base_url_override = Some(url);
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
    base_url_override: Option<String>,
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
            base_url_override: config.base_url_override,
        })
    }

    /// Get the base URL for a chain
    fn base_url(&self, chain: Option<Chain>) -> &str {
        match &self.base_url_override {
            Some(url) => url.trim_end_matches('/'),
            None => chain.unwrap_or(self.default_chain).api_url(),
        }
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

    /// Get orders by owner address (first page, API default page size)
    pub async fn get_orders_by_owner(
        &self,
        chain: Option<Chain>,
        owner: &str,
    ) -> Result<Vec<Order>> {
        self.get_orders_by_owner_paginated(chain, owner, None, None)
            .await
    }

    /// Get orders by owner address with pagination
    ///
    /// `offset` defaults to 0 and `limit` to 10 on the API side; `limit`
    /// must be between 1 and 1000.
    pub async fn get_orders_by_owner_paginated(
        &self,
        chain: Option<Chain>,
        owner: &str,
        offset: Option<u64>,
        limit: Option<u32>,
    ) -> Result<Vec<Order>> {
        let path = orders_by_owner_path(owner, offset, limit);
        self.get(chain, &path).await
    }

    /// Get several orders by UID (`POST /api/v1/orders/by_uids`)
    ///
    /// Takes up to 128 UIDs. Each element of the result is either the order
    /// or an error for an order the API could not convert. Orders that do
    /// not exist are left out, and the result is not guaranteed to be in
    /// the order of `uids`.
    pub async fn get_orders_by_uids(
        &self,
        chain: Option<Chain>,
        uids: &[String],
    ) -> Result<Vec<OrderLookup>> {
        if uids.len() > MAX_ORDERS_BY_UIDS {
            return Err(error::invalid_param(format!(
                "at most {MAX_ORDERS_BY_UIDS} order UIDs per request, got {}",
                uids.len()
            )));
        }
        self.post(chain, "/api/v1/orders/by_uids", &uids).await
    }

    /// Get the competition status of an order
    /// (`GET /api/v1/orders/{uid}/status`)
    ///
    /// Besides the status, lists the solvers that took part in the latest
    /// competition for the order, with the amounts each proposed to execute.
    pub async fn get_order_status(
        &self,
        chain: Option<Chain>,
        uid: &str,
    ) -> Result<CompetitionOrderStatus> {
        let path = format!("/api/v1/orders/{uid}/status");
        self.get(chain, &path).await
    }

    /// Get the orders settled in a transaction
    /// (`GET /api/v1/transactions/{tx_hash}/orders`)
    pub async fn get_orders_by_tx_hash(
        &self,
        chain: Option<Chain>,
        tx_hash: &str,
    ) -> Result<Vec<Order>> {
        let path = format!("/api/v1/transactions/{tx_hash}/orders");
        self.get(chain, &path).await
    }

    /// Get the full `appData` document registered for an `appData` hash
    /// (`GET /api/v1/app_data/{app_data_hash}`)
    ///
    /// Returns a 404 error when no document is stored for the hash.
    pub async fn get_app_data(
        &self,
        chain: Option<Chain>,
        app_data_hash: &str,
    ) -> Result<AppDataObject> {
        let path = format!("/api/v1/app_data/{app_data_hash}");
        self.get(chain, &path).await
    }

    /// Get the total surplus a user has earned
    /// (`GET /api/v1/users/{address}/total_surplus`)
    ///
    /// CoW marks this endpoint as unstable.
    pub async fn get_total_surplus(
        &self,
        chain: Option<Chain>,
        address: &str,
    ) -> Result<TotalSurplus> {
        let path = format!("/api/v1/users/{address}/total_surplus");
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

    /// Fetch every page of a trades query (`GET /api/v2/trades`, 1000 per
    /// page), starting at the query's offset.
    async fn get_all_trades(
        &self,
        chain: Option<Chain>,
        mut query: TradesQuery,
    ) -> Result<Vec<Trade>> {
        const PAGE: u32 = 1000;
        let mut all = Vec::new();
        let mut offset = query.offset.unwrap_or(0);
        loop {
            query = query.offset(offset).limit(PAGE);
            let page = self.get_trades(chain, &query).await?;
            let full = page.len() >= PAGE as usize;
            offset += page.len() as u64;
            all.extend(page);
            if !full {
                return Ok(all);
            }
        }
    }

    /// Get all trades by owner
    ///
    /// CoW removed the unpaginated `GET /api/v1/trades`; this now pages
    /// through `GET /api/v2/trades` and returns every trade.
    #[deprecated(
        since = "0.1.5",
        note = "use `get_trades(chain, &TradesQuery::by_owner(owner))` and page through the results"
    )]
    pub async fn get_trades_by_owner(
        &self,
        chain: Option<Chain>,
        owner: &str,
    ) -> Result<Vec<Trade>> {
        self.get_all_trades(chain, TradesQuery::by_owner(owner))
            .await
    }

    /// Get all trades for an order
    ///
    /// CoW removed the unpaginated `GET /api/v1/trades`; this now pages
    /// through `GET /api/v2/trades` and returns every trade.
    #[deprecated(
        since = "0.1.5",
        note = "use `get_trades(chain, &TradesQuery::by_order(uid))` and page through the results"
    )]
    pub async fn get_trades_by_order(
        &self,
        chain: Option<Chain>,
        order_uid: &str,
    ) -> Result<Vec<Trade>> {
        self.get_all_trades(chain, TradesQuery::by_order(order_uid))
            .await
    }

    /// Get the current auction
    ///
    /// Note: `GET /api/v1/auction` is permissioned upstream (the public API
    /// returns 403); access must be requested from the `CoW` team.
    pub async fn get_auction(&self, chain: Option<Chain>) -> Result<serde_json::Value> {
        match self.get(chain, "/api/v1/auction").await {
            Err(e) if e.status_code() == Some(403) => Err(error::invalid_param(
                "the CoW Protocol /api/v1/auction endpoint is permissioned (HTTP 403); \
                 request access from the CoW team, or use `competition` for recent auction data",
            )),
            other => other,
        }
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

    /// Get a token's price in the chain's native token
    ///
    /// Returns `{"price": <f64>}`: the value of one atom of `token` in atoms
    /// (wei) of the native token. It is not a USD price.
    pub async fn get_native_price(
        &self,
        chain: Option<Chain>,
        token: &str,
    ) -> Result<serde_json::Value> {
        let path = format!("/api/v1/token/{token}/native_price");
        self.get(chain, &path).await
    }
}

/// Most order UIDs `POST /api/v1/orders/by_uids` accepts per request
const MAX_ORDERS_BY_UIDS: usize = 128;

/// Build the `GET /api/v1/account/{owner}/orders` path with pagination
fn orders_by_owner_path(owner: &str, offset: Option<u64>, limit: Option<u32>) -> String {
    let mut params = Vec::new();
    if let Some(offset) = offset {
        params.push(format!("offset={offset}"));
    }
    if let Some(limit) = limit {
        params.push(format!("limit={limit}"));
    }
    let mut path = format!("/api/v1/account/{owner}/orders");
    if !params.is_empty() {
        path.push('?');
        path.push_str(&params.join("&"));
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::OrderKind;

    #[test]
    fn test_quote_request_sends_onchain_order() {
        let mut request = crate::types::QuoteRequest::sell(
            "0x1111111111111111111111111111111111111111",
            "0x2222222222222222222222222222222222222222",
            "1000",
            "0x3333333333333333333333333333333333333333",
        );
        request.on_chain_order = Some(true);
        request.signing_scheme = Some(crate::types::SigningScheme::EthSign);
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["onchainOrder"], true);
        assert!(json.get("onChainOrder").is_none());
        assert_eq!(json["signingScheme"], "ethsign");
    }

    #[test]
    fn test_quote_response_without_from() {
        let response: crate::types::QuoteResponse = serde_json::from_value(serde_json::json!({
            "quote": {
                "sellToken": "0x1111111111111111111111111111111111111111",
                "buyToken": "0x2222222222222222222222222222222222222222",
                "sellAmount": "1000",
                "buyAmount": "990",
                "validTo": 1700000000,
                "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
                "feeAmount": "0",
                "kind": "sell",
                "partiallyFillable": false,
                "signingScheme": "ethsign"
            },
            "expiration": "2024-01-01T00:00:00Z",
            "verified": false
        }))
        .unwrap();
        assert!(response.from.is_none());
        assert_eq!(
            response.quote.signing_scheme,
            Some(crate::types::SigningScheme::EthSign)
        );
    }

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
    fn test_orders_by_owner_path() {
        assert_eq!(
            orders_by_owner_path("0xabc", None, None),
            "/api/v1/account/0xabc/orders"
        );
        assert_eq!(
            orders_by_owner_path("0xabc", Some(20), Some(50)),
            "/api/v1/account/0xabc/orders?offset=20&limit=50"
        );
        assert_eq!(
            orders_by_owner_path("0xabc", None, Some(5)),
            "/api/v1/account/0xabc/orders?limit=5"
        );
    }

    #[test]
    fn test_order_response_bare_string() {
        let resp: OrderResponse = serde_json::from_str("\"0x1234abcd\"").unwrap();
        assert_eq!(resp.uid, "0x1234abcd");
    }

    #[test]
    fn test_order_response_object_fallback() {
        let resp: OrderResponse = serde_json::from_str(r#"{"UID":"0x1234abcd"}"#).unwrap();
        assert_eq!(resp.uid, "0x1234abcd");
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

    #[test]
    fn test_chain_mappings_round_trip() {
        for chain in Chain::all() {
            assert_eq!(chain.as_str().parse::<Chain>().unwrap(), *chain);
            assert_eq!(
                chain.chain_id().to_string().parse::<Chain>().unwrap(),
                *chain
            );
            let common = yldfi_common::Chain::from(*chain);
            assert!(
                !matches!(common, yldfi_common::Chain::Other(_)),
                "{chain:?}"
            );
            assert_eq!(Chain::try_from(common), Ok(*chain));
        }
        assert_eq!(Chain::Base.api_url(), "https://api.cow.fi/base");
        assert_eq!(Chain::Bnb.api_url(), "https://api.cow.fi/bnb");
        assert_eq!(Chain::try_from_str("bsc"), Some(Chain::Bnb));
        assert_eq!(Chain::try_from_str("8453"), Some(Chain::Base));
        assert_eq!(Chain::try_from_str("10"), None);
    }

    #[test]
    fn test_order_parses_live_shape() {
        let json = r#"{"uid":"0xcfeb","sellToken":"0xa","buyToken":"0xb","sellAmount":"1",
            "buyAmount":"2","kind":"sell","status":"presignaturePending",
            "creationDate":"2026-09-25T10:00:00.000000Z","executedSellAmount":"0",
            "executedBuyAmount":"0","executedFeeAmount":"0","owner":"0xo","receiver":"0xo"}"#;
        let order: crate::types::Order = serde_json::from_str(json).unwrap();
        assert_eq!(order.created_date, "2026-09-25T10:00:00.000000Z");
        assert_eq!(order.status, crate::types::OrderStatus::PresignaturePending);
        let out = serde_json::to_value(&order).unwrap();
        assert_eq!(out["creationDate"], "2026-09-25T10:00:00.000000Z");
        assert_eq!(out["status"], "presignaturePending");
    }

    fn mock_client(server: &wiremock::MockServer) -> Client {
        Client::with_config(Config::new().with_base_url(server.uri())).unwrap()
    }

    fn order_json(uid: &str) -> serde_json::Value {
        serde_json::json!({
            "uid": uid, "sellToken": "0xa", "buyToken": "0xb", "sellAmount": "1",
            "buyAmount": "2", "kind": "sell", "status": "fulfilled",
            "creationDate": "2026-09-25T10:00:00.000000Z", "owner": "0xo"
        })
    }

    #[tokio::test]
    async fn test_base_url_override_ignores_chain() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/orders/0x01"))
            .respond_with(ResponseTemplate::new(200).set_body_json(order_json("0x01")))
            .expect(1)
            .mount(&server)
            .await;
        let client = mock_client(&server);
        let order = client.get_order(Some(Chain::Gnosis), "0x01").await.unwrap();
        assert_eq!(order.uid, "0x01");
    }

    #[tokio::test]
    async fn test_get_orders_by_uids() {
        use wiremock::matchers::{body_json, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/orders/by_uids"))
            .and(body_json(serde_json::json!(["0x01", "0x02"])))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([
                {"order": order_json("0x01")},
                {"error": {"uid": "0x02", "description": "bad order"}}
            ])))
            .mount(&server)
            .await;
        let client = mock_client(&server);
        let uids = vec!["0x01".to_string(), "0x02".to_string()];
        let result = client.get_orders_by_uids(None, &uids).await.unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].order().unwrap().uid, "0x01");
        assert_eq!(result[1].uid(), "0x02");
        match &result[1] {
            crate::types::OrderLookup::Error(e) => assert_eq!(e.description, "bad order"),
            crate::types::OrderLookup::Order(_) => panic!("expected an error entry"),
        }
    }

    #[tokio::test]
    async fn test_get_orders_by_uids_rejects_too_many() {
        let client =
            Client::with_config(Config::new().with_base_url("http://127.0.0.1:9")).unwrap();
        let uids = vec!["0x01".to_string(); MAX_ORDERS_BY_UIDS + 1];
        let err = client.get_orders_by_uids(None, &uids).await.unwrap_err();
        assert!(err.to_string().contains("at most 128"), "{err}");
    }

    #[tokio::test]
    async fn test_get_order_status() {
        use crate::types::CompetitionStatus;
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/orders/0x01/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "type": "traded",
                "value": [
                    {"solver": "loser"},
                    {"solver": "winner", "executedAmounts": {"sell": "10", "buy": "20"}}
                ]
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/orders/0x02/status"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"type": "open"})),
            )
            .mount(&server)
            .await;
        let client = mock_client(&server);

        let status = client.get_order_status(None, "0x01").await.unwrap();
        assert_eq!(status.status, CompetitionStatus::Traded);
        assert_eq!(status.value.len(), 2);
        assert!(status.value[0].executed_amounts.is_none());
        let winner = status.value[1].executed_amounts.as_ref().unwrap();
        assert_eq!((winner.sell.as_str(), winner.buy.as_str()), ("10", "20"));

        let status = client.get_order_status(None, "0x02").await.unwrap();
        assert_eq!(status.status, CompetitionStatus::Open);
        assert!(status.value.is_empty());
    }

    #[test]
    fn test_competition_status_values() {
        use crate::types::{CompetitionOrderStatus, CompetitionStatus};
        let s: CompetitionOrderStatus =
            serde_json::from_str(r#"{"type":"someFutureStatus","value":null}"#).unwrap();
        assert_eq!(s.status, CompetitionStatus::Unknown);
        assert!(s.value.is_empty());
        let s: CompetitionOrderStatus = serde_json::from_str(r#"{"type":"executing"}"#).unwrap();
        assert_eq!(s.status, CompetitionStatus::Executing);
    }

    #[tokio::test]
    async fn test_get_orders_by_tx_hash() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/transactions/0xabc/orders"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!([order_json("0x01"), order_json("0x02")])),
            )
            .mount(&server)
            .await;
        let client = mock_client(&server);
        let orders = client.get_orders_by_tx_hash(None, "0xabc").await.unwrap();
        let uids: Vec<_> = orders.iter().map(|o| o.uid.as_str()).collect();
        assert_eq!(uids, ["0x01", "0x02"]);
    }

    #[tokio::test]
    async fn test_get_app_data() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/app_data/0xhash"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "fullAppData": r#"{"version":"0.9.0","metadata":{}}"#
            })))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/api/v1/app_data/0xmissing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;
        let client = mock_client(&server);
        let app_data = client.get_app_data(None, "0xhash").await.unwrap();
        assert_eq!(
            app_data.full_app_data,
            r#"{"version":"0.9.0","metadata":{}}"#
        );
        let err = client.get_app_data(None, "0xmissing").await.unwrap_err();
        assert_eq!(err.status_code(), Some(404));
    }

    #[tokio::test]
    async fn test_get_total_surplus() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/users/0xuser/total_surplus"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"totalSurplus": "123456789"})),
            )
            .mount(&server)
            .await;
        let client = mock_client(&server);
        let surplus = client.get_total_surplus(None, "0xuser").await.unwrap();
        assert_eq!(surplus.total_surplus.as_deref(), Some("123456789"));
    }

    #[test]
    fn test_order_status_values() {
        use crate::types::OrderStatus;
        for (s, v) in [
            ("\"open\"", OrderStatus::Open),
            ("\"fulfilled\"", OrderStatus::Fulfilled),
            ("\"cancelled\"", OrderStatus::Cancelled),
            ("\"expired\"", OrderStatus::Expired),
            ("\"someFutureStatus\"", OrderStatus::Unknown),
        ] {
            assert_eq!(serde_json::from_str::<OrderStatus>(s).unwrap(), v);
        }
    }
}
