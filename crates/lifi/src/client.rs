//! HTTP client for the LI.FI API

use crate::error::{self, Error, Result};
use crate::types::{
    Chain, ChainId, ChainsResponse, Connection, ConnectionsRequest, ConnectionsResponse, Quote,
    QuoteRequest, Route, RoutesRequest, RoutesResponse, StatusRequest, StatusResponse, Token,
    TokensRequest, TokensResponse, Tool, ToolsResponse,
};
use reqwest::Client as HttpClient;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
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
    /// Default fee percentage (0-3%)
    pub fee: Option<f64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            http: HttpClientConfig::default(),
            integrator: None,
            referrer: None,
            fee: None,
        }
    }
}

impl Config {
    /// Create a new config with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the integrator identifier
    #[must_use]
    pub fn with_integrator(mut self, integrator: impl Into<String>) -> Self {
        self.integrator = Some(integrator.into());
        self
    }

    /// Set the referrer address
    #[must_use]
    pub fn with_referrer(mut self, referrer: impl Into<String>) -> Self {
        self.referrer = Some(referrer.into());
        self
    }

    /// Set the integrator fee (0-3%)
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
        })
    }

    /// Get the integrator identifier
    pub fn integrator(&self) -> Option<&str> {
        self.integrator.as_deref()
    }

    /// Make a GET request to the API
    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(&url).send().await?;

        self.handle_response(response).await
    }

    /// Make a GET request with query parameters
    async fn get_with_query<T: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(&url).query(query).send().await?;

        self.handle_response(response).await
    }

    /// Make a POST request to the API
    async fn post<T: DeserializeOwned, B: Serialize>(&self, path: &str, body: &B) -> Result<T> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.post(&url).json(body).send().await?;

        self.handle_response(response).await
    }

    /// Handle API response
    async fn handle_response<T: DeserializeOwned>(&self, response: reqwest::Response) -> Result<T> {
        let status = response.status();

        if !status.is_success() {
            let status_code = status.as_u16();
            let body = response.text().await.unwrap_or_default();

            // Try to parse error message from response
            if let Ok(error) = serde_json::from_str::<crate::types::ApiError>(&body) {
                return Err(Error::api(status_code, error.message));
            }

            return Err(Error::from_response(status_code, &body, None));
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
    /// use lifi::{Client, QuoteRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
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
    ///     ).with_slippage(0.5);
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
    /// use lifi::{Client, RoutesRequest, RoutesOptions, RouteOrder, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
    ///     let client = Client::with_integrator("my-app")?;
    ///
    ///     let options = RoutesOptions::new()
    ///         .with_slippage(0.5)
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
    pub async fn get_step_transaction(
        &self,
        step: &crate::types::Step,
    ) -> Result<crate::types::Step> {
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
    /// use lifi::{Client, StatusRequest, TransactionStatus};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
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
    /// use lifi::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
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
    /// use lifi::{Client, TokensRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
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
    /// use lifi::{Client, ConnectionsRequest, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
    ///     let client = Client::new()?;
    ///
    ///     // Get connections from Ethereum to Arbitrum
    ///     let request = ConnectionsRequest::new()
    ///         .with_from_chain(chains::ETHEREUM)
    ///         .with_to_chain(chains::ARBITRUM);
    ///
    ///     let connections = client.get_connections(&request).await?;
    ///     println!("Found {} connections", connections.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_connections(&self, request: &ConnectionsRequest) -> Result<Vec<Connection>> {
        let response: ConnectionsResponse = self.get_with_query("/connections", request).await?;
        Ok(response.connections)
    }

    /// Get all connections (no filtering)
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
    /// use lifi::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
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

    /// Get gas prices for a chain
    ///
    /// # Example
    ///
    /// ```no_run
    /// use lifi::{Client, chains};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), lifi::Error> {
    ///     let client = Client::new()?;
    ///     let gas = client.get_gas_prices(chains::ETHEREUM).await?;
    ///
    ///     if let Some(fast) = &gas.fast {
    ///         println!("Fast gas price: {} wei", fast);
    ///     }
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_gas_prices(&self, chain_id: ChainId) -> Result<crate::types::GasPrice> {
        self.get(&format!("/gas?chainId={}", chain_id)).await
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{chains, RouteOrder, RoutesOptions};

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
            .with_fee(0.3)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.integrator, Some("my-app".to_string()));
        assert_eq!(config.referrer, Some("0x1234".to_string()));
        assert_eq!(config.fee, Some(0.3));
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
        .with_slippage(0.5)
        .with_integrator("test-app")
        .with_order(RouteOrder::Cheapest);

        assert_eq!(request.from_chain, chains::ETHEREUM);
        assert_eq!(request.to_chain, chains::ARBITRUM);
        assert_eq!(request.slippage, Some(0.5));
        assert_eq!(request.integrator, Some("test-app".to_string()));
        assert_eq!(request.order, Some(RouteOrder::Cheapest));
    }

    #[test]
    fn test_routes_request_builder() {
        let options = RoutesOptions::new()
            .with_slippage(1.0)
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
}
