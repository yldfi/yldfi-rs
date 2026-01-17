//! HTTP client for the KyberSwap Aggregator API

use crate::error::{self, Error, Result};
use crate::types::{
    BuildRouteData, BuildRouteRequest, BuildRouteResponse, Chain, RouteData, RouteRequest,
    RouteSummary, RoutesResponse,
};
use crate::{default_config, Config};
use yldfi_common::api::BaseClient;

/// Client for the KyberSwap Aggregator API
#[derive(Debug, Clone)]
pub struct Client {
    base: BaseClient,
}

impl Client {
    /// Create a new client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(default_config())
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

    /// Get swap routes (V1 API)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use kyberswap::{Client, Chain, RouteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), kyberswap::Error> {
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
        let mut query_params: Vec<(&str, String)> = vec![
            ("tokenIn", request.token_in.clone()),
            ("tokenOut", request.token_out.clone()),
            ("amountIn", request.amount_in.clone()),
        ];

        if let Some(ref to) = request.to {
            query_params.push(("to", to.clone()));
        }
        if let Some(save_gas) = request.save_gas {
            query_params.push(("saveGas", save_gas.to_string()));
        }
        if let Some(bps) = request.slippage_tolerance_bps {
            query_params.push(("slippageTolerance", bps.to_string()));
        }

        let path = format!("/{}/api/v1/routes", chain.as_str());
        let query_refs: Vec<(&str, &str)> =
            query_params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: RoutesResponse = self.base.get(&path, &query_refs).await?;

        if response.code != 0 {
            return Err(Error::api(response.code as u16, response.message));
        }

        response
            .data
            .map(|d| d.route_summary)
            .ok_or_else(error::no_route_found)
    }

    /// Get full route data including router address
    pub async fn get_route_data(&self, chain: Chain, request: &RouteRequest) -> Result<RouteData> {
        let mut query_params: Vec<(&str, String)> = vec![
            ("tokenIn", request.token_in.clone()),
            ("tokenOut", request.token_out.clone()),
            ("amountIn", request.amount_in.clone()),
        ];

        if let Some(ref to) = request.to {
            query_params.push(("to", to.clone()));
        }
        if let Some(bps) = request.slippage_tolerance_bps {
            query_params.push(("slippageTolerance", bps.to_string()));
        }

        let path = format!("/{}/api/v1/routes", chain.as_str());
        let query_refs: Vec<(&str, &str)> =
            query_params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let response: RoutesResponse = self.base.get(&path, &query_refs).await?;

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
    /// use kyberswap::{Client, Chain, RouteRequest, BuildRouteRequest};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), kyberswap::Error> {
    ///     let client = Client::new()?;
    ///
    ///     // First get routes
    ///     let request = RouteRequest::new(
    ///         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
    ///         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
    ///         "1000000000000000000",
    ///     );
    ///     let route_summary = client.get_routes(Chain::Ethereum, &request).await?;
    ///
    ///     // Then build the route to get tx data
    ///     let build_request = BuildRouteRequest {
    ///         route_summary,
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
        let response: BuildRouteResponse = self.base.post_json(&path, request).await?;

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
            .with_slippage_bps(50)
            .with_recipient("0xRecipient");

        assert_eq!(request.slippage_tolerance_bps, Some(50));
        assert_eq!(request.to, Some("0xRecipient".to_string()));
    }

    #[test]
    fn test_default_config() {
        let config = crate::default_config();
        assert_eq!(config.base_url, crate::DEFAULT_BASE_URL);
    }
}
