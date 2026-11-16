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
    Action, ActionInput, ActionVariant, ApiErrorResponse, ApproveTransaction, BundleAction,
    BundleApproval, BundleRequest, BundleResponse, Chain, ChainRef, Network, PaginationMeta,
    PreTransaction, Project, Protocol, QuoteRequest, QuoteResponse, RouteRequest, RouteResponse,
    RouteStep, RoutingStrategy, Standard, StandardAction, StandardProtocol, TokenBalance,
    TokenInfo, TokenPrice, TokensPage, TokensRequest, TransactionData, UnderlyingToken, Wallet,
};

// Re-export common utilities
pub use yldfi_common::api::{ApiConfig, BaseClient};
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Percent-encode a query value (RFC 3986 unreserved characters kept)
fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for b in value.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

/// Default base URL for the Enso Finance API
pub const DEFAULT_BASE_URL: &str = "https://api.enso.build";

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
        let query = request
            .query_params()
            .iter()
            .map(|(k, v)| format!("{k}={}", urlencode(v)))
            .collect::<Vec<_>>()
            .join("&");
        let path = format!("/api/v1/shortcuts/bundle?{query}");
        self.base.post_json(&path, request.body()).await
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
    /// * `chain_id` - Chain ID (required by the API; always sent)
    /// * `address` - Wallet address (EOA address)
    pub async fn get_balances(&self, chain_id: u64, address: &str) -> Result<Vec<TokenBalance>> {
        let params = balances_params(chain_id, address);
        let params_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.base
            .get::<Vec<TokenBalance>, _>("/api/v1/wallet/balances", &params_refs)
            .await
    }

    /// GET `path` with query parameters (repeated keys for lists)
    async fn get_query<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        params: &[(&'static str, String)],
    ) -> Result<T> {
        self.base.get::<T, _>(path, params).await
    }

    /// Get prices of several tokens on one chain (max 100 addresses)
    ///
    /// The list follows `addresses`; a token Enso has no price for is `None`.
    pub async fn get_token_prices<S: AsRef<str>>(
        &self,
        chain_id: u64,
        addresses: &[S],
    ) -> Result<Vec<Option<TokenPrice>>> {
        let params: Vec<(&'static str, String)> = addresses
            .iter()
            .map(|a| ("addresses", a.as_ref().to_string()))
            .collect();
        self.get_query(&format!("/api/v1/prices/{chain_id}"), &params)
            .await
    }

    /// Search tokens (base and `DeFi`) Enso knows, one page at a time
    pub async fn get_tokens(&self, request: &TokensRequest) -> Result<TokensPage> {
        self.get_query("/api/v1/tokens", &request.query_params())
            .await
    }

    /// Get a quote (output amount and route, no transaction)
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<QuoteResponse> {
        self.get_query("/api/v1/shortcuts/quote", &request.query_params())
            .await
    }

    /// List the networks Enso supports
    pub async fn get_networks(&self) -> Result<Vec<Network>> {
        self.get_query("/api/v1/networks", &[]).await
    }

    /// List protocols, optionally only those on `chain_id` or with `slug`
    pub async fn get_protocols(
        &self,
        chain_id: Option<u64>,
        slug: Option<&str>,
    ) -> Result<Vec<Protocol>> {
        let mut params = Vec::new();
        if let Some(chain_id) = chain_id {
            params.push(("chainId", chain_id.to_string()));
        }
        if let Some(slug) = slug {
            params.push(("slug", slug.to_string()));
        }
        self.get_query("/api/v1/protocols", &params).await
    }

    /// Get one protocol by slug (e.g. `aave-v2`)
    pub async fn get_protocol(&self, slug: &str) -> Result<Protocol> {
        let path = format!("/api/v1/standards/{}", urlencode(slug));
        self.get_query(&path, &[]).await
    }

    /// List standards: each protocol integration, its forks and its actions
    pub async fn get_standards(&self) -> Result<Vec<Standard>> {
        self.get_query("/api/v1/standards", &[]).await
    }

    /// List projects and their protocol slugs
    pub async fn get_projects(&self) -> Result<Vec<Project>> {
        self.get_query("/api/v1/projects", &[]).await
    }

    /// List the protocols of a project (e.g. `aave`)
    pub async fn get_project_protocols(&self, project: &str) -> Result<Vec<Protocol>> {
        let path = format!("/api/v1/projects/{}/protocols", urlencode(project));
        self.get_query(&path, &[]).await
    }

    /// List the swap aggregators Enso routes through
    pub async fn get_aggregators(&self) -> Result<Vec<String>> {
        self.get_query("/api/v1/aggregators", &[]).await
    }

    /// List the bundle actions and their inputs
    pub async fn get_actions(&self) -> Result<Vec<Action>> {
        self.get_query("/api/v1/actions", &[]).await
    }

    /// List the bundle actions a protocol supports (by protocol slug)
    pub async fn get_protocol_actions(&self, slug: &str) -> Result<Vec<Action>> {
        let path = format!("/api/v1/actions/{}", urlencode(slug));
        self.get_query(&path, &[]).await
    }

    /// Get the Enso smart wallet of `from_address`
    pub async fn get_wallet(
        &self,
        chain_id: u64,
        from_address: &str,
        routing_strategy: Option<RoutingStrategy>,
    ) -> Result<Wallet> {
        let params = wallet_params(chain_id, from_address, routing_strategy);
        self.get_query("/api/v1/wallet", &params).await
    }

    /// Build a transaction approving `amount` (smallest units) of `token`
    /// for Enso's router (or the smart wallet, with that routing strategy)
    pub async fn get_approve_transaction(
        &self,
        chain_id: u64,
        from_address: &str,
        token: &str,
        amount: &str,
        routing_strategy: Option<RoutingStrategy>,
    ) -> Result<ApproveTransaction> {
        let mut params = wallet_params(chain_id, from_address, routing_strategy);
        params.push(("tokenAddress", token.to_string()));
        params.push(("amount", amount.to_string()));
        self.get_query("/api/v1/wallet/approve", &params).await
    }

    /// Get the account ID of the API key
    pub async fn get_account_id(&self) -> Result<String> {
        self.get_query("/api/v1/account/accountId", &[]).await
    }
}

/// Query parameters shared by the wallet endpoints
fn wallet_params(
    chain_id: u64,
    from_address: &str,
    routing_strategy: Option<RoutingStrategy>,
) -> Vec<(&'static str, String)> {
    let mut params = vec![
        ("chainId", chain_id.to_string()),
        ("fromAddress", from_address.to_string()),
    ];
    if let Some(strategy) = routing_strategy {
        params.push(("routingStrategy", strategy.as_str().to_string()));
    }
    params
}

/// Query parameters for `GET /api/v1/wallet/balances`
fn balances_params(chain_id: u64, address: &str) -> Vec<(&'static str, String)> {
    vec![
        ("chainId", chain_id.to_string()),
        ("eoaAddress", address.to_string()),
        ("useEoa", "true".to_string()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balances_always_send_chain_id() {
        let params = balances_params(0, "0x2222222222222222222222222222222222222222");
        assert!(params.contains(&("chainId", "0".to_string())));
        let params = balances_params(8453, "0x2222222222222222222222222222222222222222");
        assert_eq!(params[0], ("chainId", "8453".to_string()));
    }

    /// A one-shot HTTP server on localhost answering 200 with `body`; the
    /// handle yields the request line (method and path with query)
    fn serve_once(body: &'static str) -> (Client, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buf = [0u8; 4096];
            while !request.windows(4).any(|w| w == b"\r\n\r\n") {
                let n = stream.read(&mut buf).unwrap();
                if n == 0 {
                    break;
                }
                request.extend_from_slice(&buf[..n]);
            }
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .unwrap();
            let request = String::from_utf8_lossy(&request).into_owned();
            request.lines().next().unwrap_or_default().to_string()
        });
        let client = Client::with_config(ApiConfig::new(format!("http://{addr}"))).unwrap();
        (client, handle)
    }

    #[tokio::test]
    async fn get_token_prices_repeats_addresses_and_keeps_nulls() {
        let (client, server) = serve_once(
            r#"[{"decimals":18,"price":3000.5,"address":"0xa","symbol":"WETH","name":"Wrapped Ether","timestamp":1,"chainId":1},null]"#,
        );
        let prices = client.get_token_prices(1, &["0xa", "0xb"]).await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/prices/1?addresses=0xa&addresses=0xb HTTP/1.1"
        );
        assert_eq!(prices.len(), 2);
        assert_eq!(prices[0].as_ref().unwrap().price, 3000.5);
        assert!(prices[1].is_none());
    }

    #[tokio::test]
    async fn get_tokens_sends_filters() {
        let (client, server) = serve_once(
            r#"{"meta":{"total":1,"lastPage":1,"currentPage":1,"perPage":100,"prev":null,"next":null,"cursor":null},
            "data":[{"address":"0xa","chainId":1,"type":"defi","decimals":18,"symbol":"aWETH","name":null,
            "logosUri":null,"underlyingTokens":[{"address":"0xb","chainId":1,"type":"base","decimals":18,
            "symbol":"WETH","name":"Wrapped Ether","logosUri":["https://example.invalid/w.png"]}],
            "project":"aave","protocol":"aave-v2","depositType":0,"redeemType":0,"apy":"3.5","tvl":1000.25,
            "primaryAddress":"0xc"}]}"#,
        );
        let request = TokensRequest::new()
            .with_chain_id(1)
            .with_project("aave")
            .with_symbols(vec!["aWETH".into()])
            .with_page(1, 100);
        let page = client.get_tokens(&request).await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/tokens?chainId=1&project=aave&page=1&pageSize=100&symbol=aWETH HTTP/1.1"
        );
        assert_eq!(page.meta.per_page, 100);
        let token = &page.data[0];
        assert_eq!(token.kind.as_deref(), Some("defi"));
        assert_eq!(token.apy, Some(3.5));
        assert_eq!(token.tvl, Some(1000.25));
        let underlying = token.underlying_tokens.as_ref().unwrap();
        assert_eq!(underlying[0].symbol.as_deref(), Some("WETH"));
    }

    #[tokio::test]
    async fn get_quote_sends_lists_as_repeated_keys() {
        let (client, server) = serve_once(
            r#"{"amountOut":"2999000000","priceImpact":-12,"route":[{"tokenIn":["0xa"],"tokenOut":["0xb"],
            "protocol":"uniswap-v3","action":"swap","args":{"fee":"500"}}],"feeAmount":["0"],"ensoFeeAmount":["0"]}"#,
        );
        let request = QuoteRequest::new(1, "0xa", "0xb", "1000")
            .with_from_address("0xf")
            .with_ignore_aggregators(vec!["0x".into(), "1inch".into()]);
        let quote = client.get_quote(&request).await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/shortcuts/quote?chainId=1&fromAddress=0xf&tokenIn=0xa&tokenOut=0xb&amountIn=1000&ignoreAggregators=0x&ignoreAggregators=1inch HTTP/1.1"
        );
        assert_eq!(quote.amount_out, "2999000000");
        assert_eq!(quote.price_impact, Some(-12.0));
        assert_eq!(quote.route[0].protocol, "uniswap-v3");
    }

    #[tokio::test]
    async fn get_networks_lists_networks() {
        let (client, server) = serve_once(r#"[{"id":1,"name":"mainnet","isConnected":true}]"#);
        let networks = client.get_networks().await.unwrap();
        assert_eq!(server.join().unwrap(), "GET /api/v1/networks HTTP/1.1");
        assert_eq!(networks[0].id, 1);
        assert!(networks[0].is_connected);
    }

    #[tokio::test]
    async fn get_protocols_filters_by_chain() {
        let (client, server) = serve_once(
            r#"[{"project":"uniswap","slug":"uniswap-v2","name":"Uniswap V2","description":null,
            "url":"https://app.uniswap.org/swap","logosUri":"https://example.invalid/u.png",
            "chains":[{"id":1,"name":"mainnet"}]}]"#,
        );
        let protocols = client.get_protocols(Some(1), None).await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/protocols?chainId=1 HTTP/1.1"
        );
        assert_eq!(protocols[0].slug, "uniswap-v2");
        assert_eq!(
            protocols[0].logos_uri.as_deref(),
            Some(&["https://example.invalid/u.png".to_string()][..])
        );
        assert_eq!(protocols[0].chains.as_ref().unwrap()[0].id, 1);
    }

    #[tokio::test]
    async fn get_protocol_reads_one_protocol() {
        let (client, server) = serve_once(
            r#"{"project":"aave","slug":"aave-v2","name":null,"description":null,"url":null,"logosUri":null,"chains":null}"#,
        );
        let protocol = client.get_protocol("aave-v2").await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/standards/aave-v2 HTTP/1.1"
        );
        assert_eq!(protocol.project, "aave");
        assert!(protocol.chains.is_none());
    }

    #[tokio::test]
    async fn get_standards_lists_standards() {
        let (client, server) = serve_once(
            r#"[{"protocol":{"slug":"aave-v3","url":"https://aave.com"},"forks":[{"slug":"spark","url":"https://spark.fi"}],
            "actions":[{"action":"deposit","name":"Deposit","functionNames":["supply"],
            "supportedChains":[{"id":1,"name":"mainnet"}],"inputs":["tokenIn","amountIn"]}]}]"#,
        );
        let standards = client.get_standards().await.unwrap();
        assert_eq!(server.join().unwrap(), "GET /api/v1/standards HTTP/1.1");
        assert_eq!(standards[0].protocol.slug, "aave-v3");
        assert_eq!(standards[0].forks[0].slug, "spark");
        assert_eq!(standards[0].actions[0].function_names, vec!["supply"]);
    }

    #[tokio::test]
    async fn get_projects_lists_projects() {
        let (client, server) =
            serve_once(r#"[{"id":"aave","chains":[1,137],"protocols":["aave-v2","aave-v3"]}]"#);
        let projects = client.get_projects().await.unwrap();
        assert_eq!(server.join().unwrap(), "GET /api/v1/projects HTTP/1.1");
        assert_eq!(projects[0].chains, vec![1, 137]);
        assert_eq!(projects[0].protocols.len(), 2);
    }

    #[tokio::test]
    async fn get_project_protocols_encodes_project() {
        let (client, server) = serve_once(r#"[{"project":"aave","slug":"aave-v3"}]"#);
        let protocols = client.get_project_protocols("aave").await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/projects/aave/protocols HTTP/1.1"
        );
        assert_eq!(protocols[0].slug, "aave-v3");
    }

    #[tokio::test]
    async fn get_aggregators_lists_names() {
        let (client, server) = serve_once(r#"["0x","1inch","paraswap"]"#);
        let aggregators = client.get_aggregators().await.unwrap();
        assert_eq!(server.join().unwrap(), "GET /api/v1/aggregators HTTP/1.1");
        assert_eq!(aggregators, vec!["0x", "1inch", "paraswap"]);
    }

    #[tokio::test]
    async fn get_actions_reads_inputs_and_variants() {
        let (client, server) = serve_once(
            r#"[{"action":"swap","inputs":{"tokenIn":{"type":"address","description":"Token in"},
            "receiver":{"type":"address","description":"Receiver","optional":true}},
            "variants":[{"action":"swap","inputs":{},"bundleName":"swap"}]}]"#,
        );
        let actions = client.get_actions().await.unwrap();
        assert_eq!(server.join().unwrap(), "GET /api/v1/actions HTTP/1.1");
        assert_eq!(actions[0].inputs["tokenIn"].kind, "address");
        assert_eq!(actions[0].inputs["receiver"].optional, Some(true));
        assert_eq!(actions[0].variants[0].bundle_name.as_deref(), Some("swap"));
    }

    #[tokio::test]
    async fn get_protocol_actions_uses_slug() {
        let (client, server) = serve_once(r#"[{"action":"deposit","inputs":{}}]"#);
        let actions = client.get_protocol_actions("aave-v3").await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/actions/aave-v3 HTTP/1.1"
        );
        assert_eq!(actions[0].action, "deposit");
        assert!(actions[0].variants.is_empty());
    }

    #[tokio::test]
    async fn get_wallet_sends_routing_strategy() {
        let (client, server) = serve_once(r#"{"address":"0xw","isDeployed":false}"#);
        let wallet = client
            .get_wallet(1, "0xf", Some(RoutingStrategy::Delegate))
            .await
            .unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/wallet?chainId=1&fromAddress=0xf&routingStrategy=delegate HTTP/1.1"
        );
        assert_eq!(wallet.address, "0xw");
        assert!(!wallet.is_deployed);
    }

    #[tokio::test]
    async fn get_approve_transaction_reads_tx_without_value() {
        let (client, server) = serve_once(
            r#"{"tx":{"data":"0x095ea7b3","to":"0xt","from":"0xf"},"gas":"50000","token":"0xt",
            "amount":"1000","spender":"0xs"}"#,
        );
        let approve = client
            .get_approve_transaction(1, "0xf", "0xt", "1000", None)
            .await
            .unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/wallet/approve?chainId=1&fromAddress=0xf&tokenAddress=0xt&amount=1000 HTTP/1.1"
        );
        assert_eq!(approve.tx.data, "0x095ea7b3");
        assert_eq!(approve.tx.value, "0");
        assert_eq!(approve.gas.as_deref(), Some("50000"));
        assert_eq!(approve.spender.as_deref(), Some("0xs"));
    }

    #[tokio::test]
    async fn get_account_id_reads_string() {
        let (client, server) = serve_once(r#""acct-123""#);
        let id = client.get_account_id().await.unwrap();
        assert_eq!(
            server.join().unwrap(),
            "GET /api/v1/account/accountId HTTP/1.1"
        );
        assert_eq!(id, "acct-123");
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
