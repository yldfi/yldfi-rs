//! HTTP client for the `KyberSwap` Aggregator API

use crate::error::{self, Error, Result};
use crate::types::{
    BuildRouteData, BuildRouteRequest, BuildRouteResponse, Chain, RouteData, RouteRequest,
    RouteSummary, RoutesResponse,
};
use crate::{default_config, Config};
use yldfi_common::api::BaseClient;

/// Client for the `KyberSwap` Aggregator API
#[derive(Debug, Clone)]
pub struct Client {
    base: BaseClient,
    client_id: Option<String>,
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(default_config())
    }

    /// Create a new client with custom configuration
    pub fn with_config(config: Config) -> Result<Self> {
        let base = BaseClient::new(config)?;
        Ok(Self {
            base,
            client_id: None,
        })
    }

    /// Identify the integration with an `X-Client-Id` header
    ///
    /// `KyberSwap` uses the client id for attribution and rate limiting.
    #[must_use]
    pub fn with_client_id(mut self, client_id: impl Into<String>) -> Self {
        self.client_id = Some(client_id.into());
        self
    }

    /// Get the configured `X-Client-Id`, if any
    #[must_use]
    pub fn client_id(&self) -> Option<&str> {
        self.client_id.as_deref()
    }

    /// Request headers: `X-Api-Key` (not `Authorization`) when an API key is
    /// configured, plus `X-Client-Id` when set
    fn build_headers(&self) -> reqwest::header::HeaderMap {
        let mut headers = reqwest::header::HeaderMap::new();
        if let Some(key) = self.config().get_api_key() {
            if let Ok(value) = reqwest::header::HeaderValue::from_str(key) {
                headers.insert("X-Api-Key", value);
            }
        }
        if let Some(ref id) = self.client_id {
            if let Ok(value) = reqwest::header::HeaderValue::from_str(id) {
                headers.insert(
                    reqwest::header::HeaderName::from_static("x-client-id"),
                    value,
                );
            }
        }
        headers
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

    /// Get swap routes (V1 API)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kybr::{Client, Chain, RouteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), kybr::Error> {
    ///     let client = Client::new()?;
    ///
    ///     let request = RouteRequest::new(
    ///         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///     );
    ///
    ///     let routes = client.get_routes(Chain::Ethereum, &request).await?;
    ///     println!("Output: {}", routes.amount_out);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn get_routes(&self, chain: Chain, request: &RouteRequest) -> Result<RouteSummary> {
        let data = self.get_route_data(chain, request).await?;
        data.summary()
            .map_err(|e| error::invalid_param(format!("unexpected routeSummary shape: {e}")))
    }

    /// Get full route data including router address
    pub async fn get_route_data(&self, chain: Chain, request: &RouteRequest) -> Result<RouteData> {
        let query_params = request.to_query_params();
        let path = format!("/{}/api/v1/routes", chain.as_str());

        let response: RoutesResponse = self
            .base
            .get_with_headers(&path, &query_params, self.build_headers())
            .await?;

        if response.code != 0 {
            return Err(Error::api(response.code as u16, response.message));
        }

        response.data.ok_or_else(error::no_route_found)
    }

    /// Build a route for execution (get transaction data)
    ///
    /// After getting routes, call this to get the transaction data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kybr::{Client, Chain, RouteRequest, BuildRouteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), kybr::Error> {
    ///     let client = Client::new()?;
    ///
    ///     // First get routes
    ///     let request = RouteRequest::new(
    ///         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///     );
    ///     let route_data = client.get_route_data(Chain::Ethereum, &request).await?;
    ///
    ///     // Then build the route to get tx data (summary passed through unmodified)
    ///     let build_request = BuildRouteRequest {
    ///         route_summary: route_data.route_summary,
    ///         sender: "0xYourAddress".to_string(),
    ///         recipient: "0xYourAddress".to_string(),
    ///         slippage_tolerance_bps: Some(50), // 0.5%
    ///         deadline: None,
    ///         enable_permit: None,
    ///     };
    ///
    ///     let tx_data = client.build_route(Chain::Ethereum, &build_request).await?;
    ///     println!("Router: {}", tx_data.router_address);
    ///     println!("Data: {}", tx_data.data);
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn build_route(
        &self,
        chain: Chain,
        request: &BuildRouteRequest,
    ) -> Result<BuildRouteData> {
        let path = format!("/{}/api/v1/route/build", chain.as_str());
        let response: BuildRouteResponse = self
            .base
            .post_json_with_headers(&path, request, self.build_headers())
            .await?;

        if response.code != 0 {
            return Err(Error::api(response.code as u16, response.message));
        }

        response.data.ok_or_else(error::no_route_found)
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
    fn test_chain_parsing() {
        assert_eq!(Chain::try_from_str("ethereum"), Some(Chain::Ethereum));
        assert_eq!(Chain::try_from_str("arbitrum"), Some(Chain::Arbitrum));
        assert_eq!(Chain::try_from_str("unknown"), None);
    }

    #[test]
    fn test_route_request_builder() {
        let request = RouteRequest::new("0xTokenIn", "0xTokenOut", "1000000000000000000")
            .with_recipient("0xRecipient");

        assert_eq!(request.to, Some("0xRecipient".to_string()));
    }

    #[test]
    fn test_route_request_query_params() {
        let request = RouteRequest::new("0xTokenIn", "0xTokenOut", "1000")
            .with_included_sources("uniswap,curve")
            .with_excluded_sources("balancer")
            .with_gas_include(true)
            .with_save_gas(true);
        let params = request.to_query_params();
        let get = |k: &str| {
            params
                .iter()
                .find(|(key, _)| *key == k)
                .map(|(_, v)| v.as_str())
        };
        assert_eq!(get("tokenIn"), Some("0xTokenIn"));
        assert_eq!(get("includedSources"), Some("uniswap,curve"));
        assert_eq!(get("excludedSources"), Some("balancer"));
        assert_eq!(get("gasInclude"), Some("true"));
        assert_eq!(get("saveGas"), Some("true"));
        // Slippage is a build/swap parameter, not a routes parameter
        assert_eq!(get("slippageTolerance"), None);
        assert_eq!(get("includeDexs"), None);
    }

    #[test]
    fn test_route_summary_price_impact_from_usd() {
        let summary: crate::types::RouteSummary = serde_json::from_value(serde_json::json!({
            "tokenIn": "0x1111111111111111111111111111111111111111",
            "tokenOut": "0x2222222222222222222222222222222222222222",
            "amountIn": "1000",
            "amountInUsd": "100",
            "amountOut": "990",
            "amountOutUsd": "99",
            "gas": "100000",
            "gasPrice": "1",
            "gasUsd": "0.1",
            "l1FeeUsd": "0",
            "checksum": "1",
            "timestamp": 1,
            "route": []
        }))
        .unwrap();
        let impact = summary.price_impact_percent().unwrap();
        assert!((impact - 1.0).abs() < 1e-9);
        let json = serde_json::to_value(&summary).unwrap();
        assert!(json.get("priceImpact").is_none());
        assert!(json.get("amountOutMin").is_none());
    }

    #[test]
    fn test_client_id_header() {
        let client = Client::new().unwrap();
        assert!(client.build_headers().get("x-client-id").is_none());

        let client = client.with_client_id("example-app");
        assert_eq!(client.client_id(), Some("example-app"));
        assert_eq!(
            client.build_headers().get("x-client-id").unwrap(),
            "example-app"
        );
    }

    #[test]
    fn test_default_config() {
        let config = crate::default_config();
        assert_eq!(config.base_url, crate::DEFAULT_BASE_URL);

        let client = Client::with_config(crate::gateway_config("k")).unwrap();
        assert_eq!(client.config().base_url, crate::GATEWAY_BASE_URL);
        assert_eq!(client.build_headers().get("X-Api-Key").unwrap(), "k");
        assert!(client.build_headers().get("authorization").is_none());
    }
}
