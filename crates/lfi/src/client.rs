//! HTTP client for the LI.FI API

use crate::error::{self, Error, Result};
use crate::types::{
    Chain, ChainId, ChainsResponse, Connection, ConnectionsRequest, ConnectionsResponse,
    ContractCallsRequest, GasStatusResponse, Quote, QuoteRequest, Route, RoutesRequest,
    RoutesResponse, StatusRequest, StatusResponse, ToAmountQuoteRequest, Token, TokensRequest,
    TokensResponse, Tool, ToolsResponse, TransfersRequest, TransfersResponse,
};
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use yldfi_common::api::SecretApiKey;
use yldfi_common::http::HttpClientConfig;

const DEFAULT_BASE_URL: &str = "https://li.quest/v1";

/// Configuration for the LI.FI API client
#[derive(Debug, Clone)]
pub struct Config {
    /// Base URL for the API
    pub base_url: String,
    /// HTTP client configuration (timeout, proxy, user-agent)
    pub http: HttpClientConfig,
    /// Integrator identifier (recommended for production)
    pub integrator: Option<String>,
    /// Default referrer address
    pub referrer: Option<String>,
    /// Default integrator fee as a decimal fraction (`0.003` = 0.3%; LI.FI
    /// requires a value below 1)
    pub fee: Option<f64>,
    /// Optional API key, sent as `x-lifi-api-key`. Without one, LI.FI rate
    /// limits by IP. Keep it server-side.
    pub api_key: Option<SecretApiKey>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            http: HttpClientConfig::default(),
            integrator: None,
            referrer: None,
            fee: None,
            api_key: None,
        }
    }
}

impl Config {
    /// Create a new config with default settings
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the integrator identifier
    #[must_use]
    pub fn with_integrator(mut self, integrator: impl Into<String>) -> Self {
        self.integrator = Some(integrator.into());
        self
    }

    /// Set the API key (sent as the `x-lifi-api-key` header)
    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(SecretApiKey::new(api_key));
        self
    }

    /// Set the API key if one is given
    #[must_use]
    pub fn with_optional_api_key(mut self, api_key: Option<String>) -> Self {
        self.api_key = api_key.map(SecretApiKey::new);
        self
    }

    /// Set the referrer address
    #[must_use]
    pub fn with_referrer(mut self, referrer: impl Into<String>) -> Self {
        self.referrer = Some(referrer.into());
        self
    }

    /// Set the integrator fee as a decimal fraction (`0.003` = 0.3%)
    ///
    /// Sent to LI.FI unchanged; `0.3` would be a **30%** fee.
    #[must_use]
    pub fn with_fee(mut self, fee: f64) -> Self {
        self.fee = Some(fee);
        self
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

/// Client for the LI.FI cross-chain aggregator API
///
/// LI.FI provides optimal routes for cross-chain swaps and bridges, aggregating
/// multiple bridges (Stargate, Hop, Connext, etc.) and DEXs.
#[derive(Debug, Clone)]
pub struct Client {
    http: HttpClient,
    base_url: String,
    integrator: Option<String>,
    referrer: Option<String>,
    fee: Option<f64>,
    api_key: Option<SecretApiKey>,
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(Config::default())
    }

    /// Create a new client with an integrator identifier
    ///
    /// The integrator string helps LI.FI track API usage and is recommended
    /// for production applications.
    pub fn with_integrator(integrator: impl Into<String>) -> Result<Self> {
        Self::with_config(Config::default().with_integrator(integrator))
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let http = yldfi_common::build_client(&config.http)?;

        Ok(Self {
            http,
            base_url: config.base_url,
            integrator: config.integrator,
            referrer: config.referrer,
            fee: config.fee,
            api_key: config.api_key,
        })
    }

    /// Attach the API key header, if configured
    fn authed(&self, req: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.api_key {
            Some(key) => req.header("x-lifi-api-key", key.expose()),
            None => req,
        }
    }

    /// Get the integrator identifier
    #[must_use]
    pub fn integrator(&self) -> Option<&str> {
        self.integrator.as_deref()
    }

    /// Make a GET request to the API
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.authed(self.http.get(&url)).send().await?;

        self.handle_response(response).await
    }

    /// Make a GET request with query parameters
    async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.authed(self.http.get(&url).query(query)).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the API
    async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.authed(self.http.post(&url).json(body)).send().await?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if !status.is_success() {
            let status_code = status.as_u16();
            let retry_after = yldfi_common::api::extract_retry_after(response.headers());
            let body = response.text().await.unwrap_or_default();
            return Err(error_from_body(status_code, &body, retry_after));
        }

        let data = response.json().await?;
        Ok(data)
    }

    // ========================================================================
    // Quote API
    // ========================================================================

    /// Get a quote for a cross-chain swap
    ///
    /// This returns the single best route based on the provided parameters.
    /// For more options, use [`get_routes`](Self::get_routes).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, QuoteRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::with_integrator("my-app")?;
    ///
    ///     // Get quote for swapping 1 ETH on Ethereum to USDC on Arbitrum
    ///     let request = QuoteRequest::new(
    ///         chains::ETHEREUM,
    ///         chains::ARBITRUM,
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
    ///         "0xaf88d065e77c8cC2239327C5EDb3A432268e5831", // USDC on Arbitrum
    ///         "1000000000000000000", // 1 ETH
    ///         "0xYourAddress",
    ///     ).with_slippage(0.005); // 0.5% (fraction)
    ///
    ///     let quote = client.get_quote(&request).await?;
    ///     println!("Estimated output: {}", quote.estimate.to_amount);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_quote(&self, request: &QuoteRequest) -> Result<Quote> {
        // Build query with integrator if set
        let mut query_request = request.clone();
        if query_request.integrator.is_none() {
            query_request.integrator = self.integrator.clone();
        }
        if query_request.referrer.is_none() {
            query_request.referrer = self.referrer.clone();
        }
        if query_request.fee.is_none() {
            query_request.fee = self.fee;
        }

        self.get_with_query("/quote", &query_request).await
    }

    /// Get a quote for receiving an exact amount (`GET /quote/toAmount`)
    ///
    /// LI.FI works out how much of the source token to send; read it from
    /// `quote.action.from_amount`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, ToAmountQuoteRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::with_integrator("my-app")?;
    ///
    ///     // How much USDC on Ethereum buys exactly 10 USDC on Arbitrum?
    ///     let request = ToAmountQuoteRequest::new(
    ///         chains::ETHEREUM,
    ///         chains::ARBITRUM,
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    ///         "10000000", // 10 USDC
    ///         "0xYourAddress",
    ///     );
    ///
    ///     let quote = client.get_quote_to_amount(&request).await?;
    ///     println!("Send: {}", quote.action.from_amount);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_quote_to_amount(&self, request: &ToAmountQuoteRequest) -> Result<Quote> {
        let mut request = request.clone();
        if request.integrator.is_none() {
            request.integrator = self.integrator.clone();
        }
        if request.referrer.is_none() {
            request.referrer = self.referrer.clone();
        }
        if request.fee.is_none() {
            request.fee = self.fee;
        }

        self.get_with_query("/quote/toAmount", &request).await
    }

    /// Get a quote that bridges or swaps tokens and then runs contract calls
    /// on the destination chain (`POST /quote/contractCalls`, beta)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, ContractCall, ContractCallsRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::with_integrator("my-app")?;
    ///
    ///     // Stake 0.01 ETH with Lido on Ethereum, paying with USDC on Arbitrum
    ///     let stake = ContractCall::new(
    ///         "10000000000000000",
    ///         "0x0000000000000000000000000000000000000000",
    ///         "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84",
    ///         "0x",
    ///         "110000",
    ///     )
    ///     .with_to_token_address("0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84");
    ///
    ///     let request = ContractCallsRequest::new(
    ///         chains::ARBITRUM,
    ///         "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    ///         "0xYourAddress",
    ///         chains::ETHEREUM,
    ///         "ETH",
    ///         "10000000000000000",
    ///         vec![stake],
    ///     );
    ///
    ///     let quote = client.get_contract_calls_quote(&request).await?;
    ///     println!("Send: {}", quote.action.from_amount);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_contract_calls_quote(&self, request: &ContractCallsRequest) -> Result<Quote> {
        let mut request = request.clone();
        if request.integrator.is_none() {
            request.integrator = self.integrator.clone();
        }
        if request.referrer.is_none() {
            request.referrer = self.referrer.clone();
        }
        if request.fee.is_none() {
            request.fee = self.fee;
        }

        self.post("/quote/contractCalls", &request).await
    }

    // ========================================================================
    // Routes API (Advanced)
    // ========================================================================

    /// Get multiple routes for a cross-chain swap
    ///
    /// This returns all available routes, sorted by the specified order preference.
    /// Use this when you want to show users multiple options.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, RoutesRequest, RoutesOptions, RouteOrder, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::with_integrator("my-app")?;
    ///
    ///     let options = RoutesOptions::new()
    ///         .with_slippage(0.005) // 0.5% (fraction)
    ///         .with_order(RouteOrder::Cheapest);
    ///
    ///     let request = RoutesRequest::new(
    ///         chains::ETHEREUM,
    ///         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
    ///         "1000000000000000000",
    ///         "0xYourAddress",
    ///         chains::ARBITRUM,
    ///         "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
    ///     ).with_options(options);
    ///
    ///     let response = client.get_routes(&request).await?;
    ///     println!("Found {} routes", response.routes.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_routes(&self, request: &RoutesRequest) -> Result<RoutesResponse> {
        // Add integrator to options if not set
        let mut request = request.clone();
        if let Some(ref mut options) = request.options {
            if options.integrator.is_none() {
                options.integrator = self.integrator.clone();
            }
            if options.referrer.is_none() {
                options.referrer = self.referrer.clone();
            }
            if options.fee.is_none() {
                options.fee = self.fee;
            }
        } else if self.integrator.is_some() || self.referrer.is_some() || self.fee.is_some() {
            let mut options = crate::types::RoutesOptions::new();
            options.integrator = self.integrator.clone();
            options.referrer = self.referrer.clone();
            options.fee = self.fee;
            request.options = Some(options);
        }

        self.post("/advanced/routes", &request).await
    }

    /// Get a specific step's transaction data
    ///
    /// After selecting a route, use this to get updated transaction data
    /// for a specific step.
    ///
    /// Note: re-serializing a typed [`Step`](crate::types::Step) drops any
    /// fields not modelled by this crate. Prefer
    /// [`get_step_transaction_raw`](Self::get_step_transaction_raw) when the
    /// step comes straight from an API response.
    pub async fn get_step_transaction(
        &self,
        step: &crate::types::Step,
    ) -> Result<crate::types::Step> {
        self.post("/advanced/stepTransaction", step).await
    }

    /// Get a step's transaction data, passing the step JSON through untouched.
    ///
    /// LI.FI validates the step it receives (e.g. `action.fromToken.priceUSD`
    /// is required), so the step returned by `/quote` or `/advanced/routes`
    /// should be sent back unmodified.
    pub async fn get_step_transaction_raw(
        &self,
        step: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        self.post("/advanced/stepTransaction", step).await
    }

    // ========================================================================
    // Status API
    // ========================================================================

    /// Get the status of a cross-chain transaction
    ///
    /// Use this to track the progress of a cross-chain swap after submitting
    /// the transaction.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, StatusRequest, TransactionStatus};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let request = StatusRequest::new("0xYourTxHash")
    ///         .with_bridge("stargate");
    ///
    ///     let status = client.get_status(&request).await?;
    ///
    ///     match status.status {
    ///         TransactionStatus::Done => println!("Transaction complete!"),
    ///         TransactionStatus::Pending => println!("Still processing..."),
    ///         TransactionStatus::Failed => println!("Transaction failed"),
    ///         _ => {}
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_status(&self, request: &StatusRequest) -> Result<StatusResponse> {
        self.get_with_query("/status", request).await
    }

    // ========================================================================
    // Chains API
    // ========================================================================

    /// Get list of supported chains
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///     let chains = client.get_chains().await?;
    ///
    ///     for chain in &chains {
    ///         println!("{}: {} (ID: {})", chain.key, chain.name, chain.id);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_chains(&self) -> Result<Vec<Chain>> {
        let response: ChainsResponse = self.get("/chains").await?;
        Ok(response.chains)
    }

    /// Get a specific chain by ID
    pub async fn get_chain(&self, chain_id: ChainId) -> Result<Option<Chain>> {
        let chains = self.get_chains().await?;
        Ok(chains.into_iter().find(|c| c.id == chain_id))
    }

    // ========================================================================
    // Tokens API
    // ========================================================================

    /// Get list of supported tokens
    ///
    /// Returns tokens organized by chain ID. Use `TokensRequest` to filter
    /// by specific chains.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, TokensRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///
    ///     // Get tokens for Ethereum and Arbitrum
    ///     let request = TokensRequest::new()
    ///         .with_chains(vec![chains::ETHEREUM, chains::ARBITRUM]);
    ///
    ///     let response = client.get_tokens(&request).await?;
    ///
    ///     for (chain_id, tokens) in &response.tokens {
    ///         println!("Chain {}: {} tokens", chain_id, tokens.len());
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_tokens(&self, request: &TokensRequest) -> Result<TokensResponse> {
        self.get_with_query("/tokens", request).await
    }

    /// Get all tokens (no filtering)
    pub async fn get_all_tokens(&self) -> Result<TokensResponse> {
        self.get("/tokens").await
    }

    /// Get tokens for a specific chain
    pub async fn get_chain_tokens(&self, chain_id: ChainId) -> Result<Vec<Token>> {
        let request = TokensRequest::new().with_chains(vec![chain_id]);
        let response = self.get_tokens(&request).await?;
        Ok(response
            .tokens
            .get(&chain_id.to_string())
            .cloned()
            .unwrap_or_default())
    }

    /// Get one token by address or symbol (`GET /token`)
    ///
    /// An unknown token is reported by the API as an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///     let usdc = client.get_token(chains::ETHEREUM, "USDC").await?;
    ///     println!("{} {} {:?}", usdc.symbol, usdc.address, usdc.price_usd);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_token(&self, chain_id: ChainId, token: &str) -> Result<Token> {
        self.get_with_query(
            "/token",
            &[("chain", chain_id.to_string().as_str()), ("token", token)],
        )
        .await
    }

    // ========================================================================
    // Connections API
    // ========================================================================

    /// Get available connections (bridge/exchange pairs)
    ///
    /// This returns information about which tokens can be bridged/swapped
    /// between chains.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, ConnectionsRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///
    ///     // Get connections from Ethereum to Arbitrum
    ///     let request = ConnectionsRequest::between(chains::ETHEREUM, chains::ARBITRUM);
    ///
    ///     let connections = client.get_connections(&request).await?;
    ///     println!("Found {} connections", connections.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns a `MissingParameter` error without sending a request when
    /// `from_chain` or `to_chain` is not set (LI.FI requires both).
    pub async fn get_connections(&self, request: &ConnectionsRequest) -> Result<Vec<Connection>> {
        check_connections_request(request)?;
        let response: ConnectionsResponse = self.get_with_query("/connections", request).await?;
        Ok(response.connections)
    }

    /// Get all connections (no filtering)
    ///
    /// LI.FI now requires `fromChain` and `toChain`, so this always returns a
    /// `MissingParameter` error.
    #[deprecated(
        note = "LI.FI requires fromChain and toChain; use get_connections with \
                ConnectionsRequest::between"
    )]
    pub async fn get_all_connections(&self) -> Result<Vec<Connection>> {
        self.get_connections(&ConnectionsRequest::default()).await
    }

    // ========================================================================
    // Tools API
    // ========================================================================

    /// Get available tools (bridges and exchanges)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///     let tools = client.get_tools().await?;
    ///
    ///     println!("Bridges:");
    ///     for bridge in &tools.bridges {
    ///         println!("  - {}: {}", bridge.key, bridge.name);
    ///     }
    ///
    ///     println!("Exchanges:");
    ///     for exchange in &tools.exchanges {
    ///         println!("  - {}: {}", exchange.key, exchange.name);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_tools(&self) -> Result<ToolsResponse> {
        self.get("/tools").await
    }

    /// Get available bridges
    pub async fn get_bridges(&self) -> Result<Vec<Tool>> {
        let tools = self.get_tools().await?;
        Ok(tools.bridges)
    }

    /// Get available exchanges
    pub async fn get_exchanges(&self) -> Result<Vec<Tool>> {
        let tools = self.get_tools().await?;
        Ok(tools.exchanges)
    }

    // ========================================================================
    // Gas API
    // ========================================================================

    /// Get gas prices for all supported chains
    ///
    /// Returns a map of chain ID strings to gas prices.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///     let gas_prices = client.get_all_gas_prices().await?;
    ///
    ///     if let Some(eth_gas) = gas_prices.get("1") {
    ///         println!("Ethereum fast gas: {:?} wei", eth_gas.fast);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_all_gas_prices(&self) -> Result<crate::types::GasPricesResponse> {
        self.get("/gas/prices").await
    }

    /// Get gas prices for a specific chain (`GET /gas/prices/{chainId}`)
    ///
    /// An unsupported chain is reported by the API as an error.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///     let gas = client.get_gas_prices(chains::ETHEREUM).await?;
    ///     println!("Fast gas price: {:?} wei", gas.fast);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_gas_prices(&self, chain_id: ChainId) -> Result<crate::types::GasPrice> {
        self.get(&gas_prices_path(chain_id)).await
    }

    /// Get the status of a LI.Fuel gas refuel (`GET /gas/status`)
    ///
    /// `tx_hash` is the transaction that started the refuel; an unknown hash
    /// returns status `NOT_FOUND`.
    pub async fn get_gas_status(&self, tx_hash: &str) -> Result<GasStatusResponse> {
        self.get_with_query("/gas/status", &[("txHash", tx_hash)])
            .await
    }

    // ========================================================================
    // Analytics API
    // ========================================================================

    /// Get a page of transfers (`GET /v2/analytics/transfers`)
    ///
    /// Each transfer has the same shape as a [`get_status`](Self::get_status)
    /// response. The request goes to the `/v2` sibling of the configured
    /// base URL (`.../v1` becomes `.../v2`).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lfi::{Client, TransferStatusFilter, TransfersRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lfi::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let mut request = TransfersRequest::for_wallet("0xYourAddress")
    ///         .with_status(TransferStatusFilter::All)
    ///         .with_limit(50);
    ///     loop {
    ///         let page = client.get_transfers(&request).await?;
    ///         for transfer in &page.data {
    ///             println!("{:?} {:?}", transfer.transaction_id, transfer.status);
    ///         }
    ///         match page.next {
    ///             Some(next) if page.has_next => request = request.with_next(next),
    ///             _ => break,
    ///         }
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_transfers(&self, request: &TransfersRequest) -> Result<TransfersResponse> {
        let url = format!("{}/analytics/transfers", v2_base_url(&self.base_url));
        let response = self
            .authed(self.http.get(&url).query(request))
            .send()
            .await?;

        self.handle_response(response).await
    }

    // ========================================================================
    // Convenience Methods
    // ========================================================================

    /// Get a quote and return just the transaction request
    ///
    /// This is a convenience method for getting transaction data ready to execute.
    pub async fn get_transaction(
        &self,
        request: &QuoteRequest,
    ) -> Result<crate::types::TransactionRequest> {
        let quote = self.get_quote(request).await?;
        quote.transaction_request.ok_or_else(error::no_transaction)
    }

    /// Get the best route from multiple routes
    pub async fn get_best_route(&self, request: &RoutesRequest) -> Result<Route> {
        let response = self.get_routes(request).await?;
        response
            .routes
            .into_iter()
            .next()
            .ok_or_else(error::no_route_found)
    }
}

/// Path for a single chain's gas prices
fn gas_prices_path(chain_id: ChainId) -> String {
    format!("/gas/prices/{chain_id}")
}

/// The `/v2` base URL next to a `/v1` one
///
/// A base URL without a trailing `/v1` gets `/v2` appended.
fn v2_base_url(base_url: &str) -> String {
    let base = base_url.trim_end_matches('/');
    match base.strip_suffix("/v1") {
        Some(root) => format!("{root}/v2"),
        None => format!("{base}/v2"),
    }
}

/// Check the parameters `/connections` requires
fn check_connections_request(request: &ConnectionsRequest) -> Result<()> {
    if request.from_chain.is_none() {
        return Err(error::missing_parameter("fromChain"));
    }
    if request.to_chain.is_none() {
        return Err(error::missing_parameter("toChain"));
    }
    Ok(())
}

/// Build an error from a non-success response body
///
/// LI.FI error bodies are JSON (`{"message": ..., "code": ...}`); the message
/// is extracted, but the status still decides the variant so 429 and 5xx stay
/// retryable.
fn error_from_body(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    match serde_json::from_str::<crate::types::ApiError>(body) {
        Ok(error) => Error::from_response(status, &error.message, retry_after),
        Err(_) => Error::from_response(status, body, retry_after),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{chains, ContractCall, RouteOrder, RoutesOptions};

    #[test]
    fn test_connections_require_both_chains() {
        assert!(check_connections_request(&ConnectionsRequest::new()).is_err());
        assert!(check_connections_request(&ConnectionsRequest::new().with_from_chain(1)).is_err());
        let request = ConnectionsRequest::between(chains::ETHEREUM, chains::ARBITRUM);
        assert!(check_connections_request(&request).is_ok());
        let built = reqwest::Client::new()
            .get("https://lifi.invalid/v1/connections")
            .query(&request)
            .build()
            .unwrap();
        assert_eq!(built.url().query(), Some("fromChain=1&toChain=42161"));
    }

    #[test]
    fn test_fee_is_sent_as_fraction() {
        let request = QuoteRequest::new(1, 1, "0xa", "0xb", "1000", "0xc").with_fee(0.003);
        let built = reqwest::Client::new()
            .get("https://lifi.invalid/v1/quote")
            .query(&request)
            .build()
            .unwrap();
        let query = built.url().query().unwrap();
        assert!(query.contains("fee=0.003"), "{query}");
        assert!(!query.contains("allowSwitchChain"), "{query}");
    }

    #[test]
    fn test_status_response_documented_fields() {
        let status: StatusResponse = serde_json::from_value(serde_json::json!({
            "transactionId": "0x01",
            "sending": {
                "txHash": "0xaa",
                "chainId": 42161,
                "gasAmount": "21000",
                "gasToken": {"address": "0x0000000000000000000000000000000000000000",
                    "chainId": 42161, "symbol": "ETH", "decimals": 18, "name": "ETH"},
                "amountUSD": "1.00",
                "value": "0",
                "includedSteps": [{"tool": "example"}]
            },
            "fromAddress": "0x1111111111111111111111111111111111111111",
            "toAddress": "0x2222222222222222222222222222222222222222",
            "tool": "example",
            "status": "DONE",
            "substatus": "COMPLETED",
            "feeCosts": [{"name": "Integrator fee", "amount": "1"}]
        }))
        .unwrap();
        assert_eq!(status.tool.as_deref(), Some("example"));
        assert_eq!(
            status.from_address.as_deref(),
            Some("0x1111111111111111111111111111111111111111")
        );
        assert_eq!(status.fee_costs.as_ref().map(Vec::len), Some(1));
        let sending = status.sending.unwrap();
        assert_eq!(sending.gas_amount.as_deref(), Some("21000"));
        assert_eq!(sending.gas_token.unwrap().symbol, "ETH");
        assert_eq!(sending.amount_usd.as_deref(), Some("1.00"));
        assert_eq!(sending.included_steps.map(|s| s.len()), Some(1));
    }

    #[test]
    fn test_json_error_bodies_keep_retryable_status() {
        let body = r#"{"message":"Too many requests","code":1005}"#;
        let err = error_from_body(429, body, Some(5));
        assert!(err.is_retryable());
        assert_eq!(err.retry_after(), Some(Duration::from_secs(5)));

        let err = error_from_body(503, r#"{"message":"Service unavailable"}"#, None);
        assert!(err.is_retryable());
        assert!(err.to_string().contains("Service unavailable"));

        let err = error_from_body(400, r#"{"message":"Invalid fromToken","code":1011}"#, None);
        assert!(!err.is_retryable());
        assert_eq!(err.status_code(), Some(400));
        assert!(err.to_string().contains("Invalid fromToken"));
    }

    #[test]
    fn test_gas_prices_uses_per_chain_path() {
        assert_eq!(gas_prices_path(chains::ETHEREUM), "/gas/prices/1");
        let gas: crate::types::GasPrice = serde_json::from_str(
            r#"{"standard":1000000000,"fast":2000000000,"fastest":3000000000,"lastUpdated":1700000000}"#,
        )
        .unwrap();
        assert_eq!(gas.fast, Some(2_000_000_000));
        assert_eq!(gas.last_updated, Some(1_700_000_000));
    }

    #[test]
    fn test_client_creation() {
        let client = Client::new();
        assert!(client.is_ok());
    }

    #[test]
    fn test_client_with_integrator() {
        let client = Client::with_integrator("test-app").unwrap();
        assert_eq!(client.integrator(), Some("test-app"));
    }

    #[test]
    fn test_config_builder() {
        let config = Config::new()
            .with_integrator("my-app")
            .with_referrer("0x1234")
            .with_fee(0.003)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.integrator, Some("my-app".to_string()));
        assert_eq!(config.referrer, Some("0x1234".to_string()));
        assert_eq!(config.fee, Some(0.003));
        assert_eq!(config.http.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_quote_request_builder() {
        let request = QuoteRequest::new(
            chains::ETHEREUM,
            chains::ARBITRUM,
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
            "1000000000000000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        )
        .with_slippage(0.005)
        .with_integrator("test-app")
        .with_order(RouteOrder::Cheapest);

        assert_eq!(request.from_chain, chains::ETHEREUM);
        assert_eq!(request.to_chain, chains::ARBITRUM);
        assert_eq!(request.slippage, Some(0.005));
        assert_eq!(request.integrator, Some("test-app".to_string()));
        assert_eq!(request.order, Some(RouteOrder::Cheapest));
    }

    #[test]
    fn test_routes_request_builder() {
        let options = RoutesOptions::new()
            .with_slippage(0.01)
            .with_order(RouteOrder::Fastest);

        let request = RoutesRequest::new(
            chains::ETHEREUM,
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "1000000000000000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            chains::BASE,
            "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",
        )
        .with_options(options);

        assert_eq!(request.from_chain_id, chains::ETHEREUM);
        assert_eq!(request.to_chain_id, chains::BASE);
        assert!(request.options.is_some());
    }

    #[test]
    fn test_status_request_builder() {
        let request = StatusRequest::new("0x1234567890abcdef")
            .with_bridge("stargate")
            .with_from_chain(chains::ETHEREUM)
            .with_to_chain(chains::ARBITRUM);

        assert_eq!(request.tx_hash, "0x1234567890abcdef");
        assert_eq!(request.bridge, Some("stargate".to_string()));
        assert_eq!(request.from_chain, Some(chains::ETHEREUM));
        assert_eq!(request.to_chain, Some(chains::ARBITRUM));
    }

    #[test]
    fn test_connections_request_builder() {
        let request = ConnectionsRequest::new()
            .with_from_chain(chains::ETHEREUM)
            .with_to_chain(chains::POLYGON)
            .with_from_token("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

        assert_eq!(request.from_chain, Some(chains::ETHEREUM));
        assert_eq!(request.to_chain, Some(chains::POLYGON));
        assert!(request.from_token.is_some());
    }

    #[test]
    fn test_tokens_request_builder() {
        let request = TokensRequest::new().with_chains(vec![chains::ETHEREUM, chains::ARBITRUM]);

        assert!(request.chains.is_some());
        assert_eq!(request.chains.as_ref().unwrap().len(), 2);
    }

    #[tokio::test]
    #[ignore]
    async fn test_get_quote_real() {
        let client = Client::new().unwrap();

        let request = QuoteRequest::new(
            chains::ETHEREUM,
            chains::ETHEREUM,
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        );

        match client.get_quote(&request).await {
            Ok(quote) => {
                println!("Quote received!");
                println!("To amount: {}", quote.estimate.to_amount);
            }
            Err(e) => {
                println!("Error: {:?}", e);
                panic!("Quote request failed");
            }
        }
    }

    /// A one-shot HTTP server on localhost that answers `status` with `body`.
    /// Returns a `/v1` base URL and a handle that yields the raw request.
    fn serve_once(status: u16, body: &'static str) -> (String, std::thread::JoinHandle<String>) {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = Vec::new();
            let mut buf = [0u8; 4096];
            // Read the headers, then as much body as Content-Length says
            loop {
                let n = stream.read(&mut buf).unwrap();
                request.extend_from_slice(&buf[..n]);
                let text = String::from_utf8_lossy(&request);
                if let Some(end) = text.find("\r\n\r\n") {
                    let length = text[..end]
                        .lines()
                        .find_map(|l| {
                            let (name, value) = l.split_once(':')?;
                            name.eq_ignore_ascii_case("content-length")
                                .then(|| value.trim().parse::<usize>().ok())?
                        })
                        .unwrap_or(0);
                    if request.len() >= end + 4 + length {
                        break;
                    }
                }
                if n == 0 {
                    break;
                }
            }
            write!(
                stream,
                "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            )
            .unwrap();
            String::from_utf8(request).unwrap()
        });
        (format!("http://{addr}/v1"), handle)
    }

    fn mock_client(base_url: &str) -> Client {
        Client::with_config(
            Config::new()
                .with_base_url(base_url)
                .with_integrator("test-app"),
        )
        .unwrap()
    }

    /// The request line, e.g. `GET /v1/token?chain=1&token=USDC`
    fn request_line(request: &str) -> &str {
        request
            .lines()
            .next()
            .unwrap()
            .trim_end_matches(" HTTP/1.1")
    }

    const TOKEN_JSON: &str = r#"{"address":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","chainId":1,
        "symbol":"USDC","decimals":6,"name":"USD Coin","coinKey":"USDC","priceUSD":"1.0000908634",
        "marketCapUSD":69064275759.0928,"tags":["stablecoin"],"verificationStatus":"verified"}"#;

    /// Trimmed from a live `/quote/toAmount` response
    const STEP_JSON: &str = r#"{"type":"lifi","id":"54d860da-05b9-445c-8d04-bdcf009e69d9:0",
        "tool":"across","toolDetails":{"key":"across","name":"AcrossV4"},
        "action":{"fromToken":{"address":"0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48","chainId":1,
            "symbol":"USDC","decimals":6,"name":"USD Coin"},"fromAmount":"10052647",
            "toToken":{"address":"0xaf88d065e77c8cC2239327C5EDb3A432268e5831","chainId":42161,
            "symbol":"USDC","decimals":6,"name":"USD Coin"},"fromChainId":1,"toChainId":42161,
            "slippage":0.005,"fromAddress":"0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"},
        "estimate":{"tool":"across","fromAmount":"10052647","toAmount":"10020020",
            "toAmountMin":"10000000","approvalAddress":"0x1231DEB6f5749EF6cE6943a275A1D3E7486F4EaE"},
        "includedSteps":[],"recommendedSlippage":0.005}"#;

    #[tokio::test]
    async fn test_get_token_sends_chain_and_token() {
        let (base, server) = serve_once(200, TOKEN_JSON);
        let token = mock_client(&base)
            .get_token(chains::ETHEREUM, "USDC")
            .await
            .unwrap();
        assert_eq!(token.symbol, "USDC");
        assert_eq!(token.decimals, 6);
        assert_eq!(token.price_usd.as_deref(), Some("1.0000908634"));
        let request = server.join().unwrap();
        assert_eq!(request_line(&request), "GET /v1/token?chain=1&token=USDC");
    }

    #[tokio::test]
    async fn test_get_token_reports_unknown_token() {
        let (base, server) = serve_once(
            400,
            r#"{"message":"/token Unknown token symbol","code":1011}"#,
        );
        let err = mock_client(&base)
            .get_token(chains::ETHEREUM, "NOPE")
            .await
            .unwrap_err();
        assert_eq!(err.status_code(), Some(400));
        assert!(err.to_string().contains("Unknown token symbol"), "{err}");
        server.join().unwrap();
    }

    #[tokio::test]
    async fn test_get_quote_to_amount_sends_to_amount_and_defaults() {
        let (base, server) = serve_once(200, STEP_JSON);
        let request = ToAmountQuoteRequest::new(
            chains::ETHEREUM,
            chains::ARBITRUM,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
            "10000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        )
        .with_slippage(0.005)
        .with_denied_bridges(vec!["hop".into(), "relay".into()]);
        let quote = mock_client(&base)
            .get_quote_to_amount(&request)
            .await
            .unwrap();
        assert_eq!(quote.action.from_amount, "10052647");
        assert_eq!(quote.estimate.to_amount_min, "10000000");
        let request = server.join().unwrap();
        let line = request_line(&request);
        assert!(line.starts_with("GET /v1/quote/toAmount?"), "{line}");
        for part in [
            "fromChain=1",
            "toChain=42161",
            "toAmount=10000000",
            "slippage=0.005",
            "denyBridges=hop%2Crelay",
            "integrator=test-app",
        ] {
            assert!(line.contains(part), "{part} missing from {line}");
        }
        assert!(!line.contains("fromAmount"), "{line}");
    }

    #[tokio::test]
    async fn test_get_contract_calls_quote_posts_body() {
        let (base, server) = serve_once(200, STEP_JSON);
        let call = ContractCall::new(
            "10000000000000000",
            "0x0000000000000000000000000000000000000000",
            "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84",
            "0x",
            "110000",
        )
        .with_to_token_address("0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84");
        let request = ContractCallsRequest::new(
            chains::ARBITRUM,
            "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            chains::ETHEREUM,
            "ETH",
            "10000000000000000",
            vec![call],
        )
        .with_slippage(0.005);
        let quote = mock_client(&base)
            .get_contract_calls_quote(&request)
            .await
            .unwrap();
        assert_eq!(quote.tool, "across");
        let request = server.join().unwrap();
        assert_eq!(request_line(&request), "POST /v1/quote/contractCalls");
        let body: serde_json::Value =
            serde_json::from_str(request.split_once("\r\n\r\n").unwrap().1).unwrap();
        assert_eq!(body["fromChain"], 42161);
        assert_eq!(body["toChain"], 1);
        assert_eq!(body["toAmount"], "10000000000000000");
        assert_eq!(body["integrator"], "test-app");
        assert_eq!(body["slippage"], 0.005);
        let call = &body["contractCalls"][0];
        assert_eq!(call["toContractCallData"], "0x");
        assert_eq!(call["toContractGasLimit"], "110000");
        assert_eq!(
            call["toTokenAddress"],
            "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84"
        );
        assert!(call.get("toApprovalAddress").is_none());
        assert!(body.get("toFallbackAddress").is_none());
    }

    #[tokio::test]
    async fn test_get_gas_status() {
        let (base, server) = serve_once(200, r#"{"status":"NOT_FOUND"}"#);
        let status = mock_client(&base).get_gas_status("0xabc").await.unwrap();
        assert_eq!(status.status, crate::types::TransactionStatus::NotFound);
        assert!(status.sending.is_none());
        let request = server.join().unwrap();
        assert_eq!(request_line(&request), "GET /v1/gas/status?txHash=0xabc");

        let status: GasStatusResponse = serde_json::from_str(
            r#"{"status":"DONE","sending":{"txHash":"0x01","chainId":137,"amount":"10000","block":39397739},
                "receiving":{"txHash":"0x02","chainId":100}}"#,
        )
        .unwrap();
        assert_eq!(status.status, crate::types::TransactionStatus::Done);
        assert_eq!(status.sending.unwrap().amount.as_deref(), Some("10000"));
        assert_eq!(status.receiving.unwrap().chain_id, Some(100));
    }

    #[tokio::test]
    async fn test_get_transfers_uses_v2_and_pages() {
        // Shape of a live /v2/analytics/transfers page, trimmed
        let (base, server) = serve_once(
            200,
            r#"{"data":[{"transactionId":"0xb9f1","sending":{"txHash":"0x24c9","chainId":1,
                "amount":"1","timestamp":1760036399},"receiving":{"txHash":"0x5d1e","chainId":10,
                "timestamp":"2025-10-09T18:59:59.000Z"},
                "lifiExplorerLink":"https://scan.li.fi/tx/0x24c9","fromAddress":"0xd8da",
                "toAddress":"0xd8da","tool":"across","status":"DONE","substatus":"COMPLETED",
                "metadata":{"integrator":"jumper.exchange"},"feeCosts":[]}],
                "hasPrevious":true,"hasNext":true,"next":"W3siJGRhdGUi","previous":"W3siJGRhdGUj"}"#,
        );
        let request = TransfersRequest::for_wallet("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
            .with_status(crate::types::TransferStatusFilter::All)
            .with_time_range(Some(1_600_000_000), None)
            .with_from_chain(chains::ETHEREUM)
            .with_limit(1)
            .with_next("W3siJGRhdGUh");
        let page = mock_client(&base).get_transfers(&request).await.unwrap();
        assert_eq!(page.data.len(), 1);
        assert_eq!(page.data[0].tool.as_deref(), Some("across"));
        let receiving = page.data[0].receiving.as_ref().unwrap();
        assert_eq!(receiving.timestamp, Some(1_760_036_399));
        assert_eq!(page.data[0].status, crate::types::TransactionStatus::Done);
        assert!(page.has_next && page.has_previous);
        assert_eq!(page.next.as_deref(), Some("W3siJGRhdGUi"));
        let request = server.join().unwrap();
        assert_eq!(
            request_line(&request),
            "GET /v2/analytics/transfers?wallet=0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045\
             &status=ALL&fromTimestamp=1600000000&fromChain=1&limit=1&next=W3siJGRhdGUh"
        );

        // The last page has no cursors
        let page: TransfersResponse =
            serde_json::from_str(r#"{"data":[],"hasPrevious":false,"hasNext":false}"#).unwrap();
        assert!(page.data.is_empty() && !page.has_next && page.next.is_none());
    }

    #[test]
    fn test_v2_base_url() {
        assert_eq!(v2_base_url("https://li.quest/v1"), "https://li.quest/v2");
        assert_eq!(v2_base_url("https://li.quest/v1/"), "https://li.quest/v2");
        assert_eq!(
            v2_base_url("https://proxy.example/lifi"),
            "https://proxy.example/lifi/v2"
        );
    }

    #[tokio::test]
    #[ignore]
    async fn test_new_endpoints_real() {
        let client = Client::new().unwrap();
        let usdc = client.get_token(chains::ETHEREUM, "USDC").await.unwrap();
        assert_eq!(usdc.address, "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");

        let quote = client
            .get_quote_to_amount(&ToAmountQuoteRequest::new(
                chains::ETHEREUM,
                chains::ARBITRUM,
                "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
                "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
                "10000000",
                "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            ))
            .await
            .unwrap();
        println!("toAmount quote: send {}", quote.action.from_amount);

        let page = client
            .get_transfers(
                &TransfersRequest::for_wallet("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
                    .with_status(crate::types::TransferStatusFilter::All)
                    .with_time_range(Some(1_600_000_000), None)
                    .with_limit(3),
            )
            .await
            .unwrap();
        println!(
            "transfers: {} (has_next {})",
            page.data.len(),
            page.has_next
        );
        for status in [
            crate::types::TransferStatusFilter::All,
            crate::types::TransferStatusFilter::Pending,
        ] {
            let page = client
                .get_transfers(&TransfersRequest::new().with_status(status).with_limit(100))
                .await
                .unwrap();
            println!("{status:?} transfers: {}", page.data.len());
        }

        let status = client
            .get_gas_status("0x74546ce8aac58d33c212474293dcfeeadecef115847da75131a2ff6692e03b96")
            .await
            .unwrap();
        println!("gas status: {:?}", status.status);

        let quote = client
            .get_contract_calls_quote(&ContractCallsRequest::new(
                chains::ARBITRUM,
                "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
                "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
                chains::ETHEREUM,
                "ETH",
                "10000000000000000",
                vec![ContractCall::new(
                    "10000000000000000",
                    "0x0000000000000000000000000000000000000000",
                    "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84",
                    "0x",
                    "110000",
                )
                .with_to_token_address("0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84")],
            ))
            .await
            .unwrap();
        println!("contractCalls quote: send {}", quote.action.from_amount);
    }
}
