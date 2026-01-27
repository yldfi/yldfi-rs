//! Curve Router API
//!
//! Find optimal swap routes across Curve pools and generate calldata for
//! the Curve Router-NG contract.
//!
//! # Example
//!
//! ```no_run
//! use crv::{Client, router::{RouterApi, router_address}};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), crv::Error> {
//!     let client = Client::new()?;
//!
//!     // Build the route graph from pool data
//!     let pools = client.pools().get_all_on_chain("ethereum").await?;
//!     let router = RouterApi::new("ethereum", &pools.data.pool_data);
//!
//!     // Find routes from USDC to DAI
//!     let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
//!     let dai = "0x6B175474E89094C44Da98b954EescdeCB5BE3Af";
//!
//!     let routes = router.find_routes(usdc, dai);
//!     println!("Found {} routes", routes.len());
//!
//!     if let Some(best) = routes.first() {
//!         println!("Best route has {} hops", best.steps.len());
//!
//!         // Get calldata for the router contract
//!         let calldata = router.encode_swap(best, "1000000", "990000")?;
//!         println!("Calldata length: {} bytes", calldata.len());
//!     }
//!
//!     Ok(())
//! }
//! ```

mod finder;
mod graph;
pub mod types;

pub use finder::{find_best_route, find_routes};
pub use graph::build_graph;
pub use types::{
    eth_wrapper_pairs, router_address, GraphEdge, GraphNode, PoolType, QuotedRoute, Route,
    RouteGraph, RouteStep, SwapParams, SwapType, WrapperPair, GRAPH_MAX_EDGES,
    MAX_ROUTES_PER_CRITERION, MAX_ROUTE_STEPS, MAX_SEARCH_DEPTH, MIN_TVL_USD,
};

use crate::pools::Pool;
use crate::Result;

/// High-level API for Curve routing
#[derive(Debug, Clone)]
pub struct RouterApi {
    /// The route graph
    graph: RouteGraph,
    /// Chain identifier
    chain: String,
}

impl RouterApi {
    /// Create a new router API from pool data
    pub fn new(chain: impl Into<String>, pools: &[Pool]) -> Self {
        let chain = chain.into();
        let graph = build_graph(&chain, pools);
        Self { graph, chain }
    }

    /// Create from an existing graph
    #[must_use] 
    pub fn from_graph(graph: RouteGraph) -> Self {
        let chain = graph.chain_id.clone();
        Self { graph, chain }
    }

    /// Get the underlying route graph
    #[must_use] 
    pub fn graph(&self) -> &RouteGraph {
        &self.graph
    }

    /// Get the chain identifier
    #[must_use] 
    pub fn chain(&self) -> &str {
        &self.chain
    }

    /// Get the router contract address for this chain
    #[must_use] 
    pub fn router_address(&self) -> Option<&'static str> {
        router_address(&self.chain)
    }

    /// Find all routes between two tokens
    ///
    /// Returns routes sorted by: shortest first, then by highest TVL.
    #[must_use] 
    pub fn find_routes(&self, from: &str, to: &str) -> Vec<Route> {
        find_routes(&self.graph, from, to)
    }

    /// Find the single best route between two tokens
    ///
    /// The "best" route is typically the shortest path with good liquidity.
    #[must_use] 
    pub fn find_best_route(&self, from: &str, to: &str) -> Option<Route> {
        find_best_route(&self.graph, from, to)
    }

    /// Check if a token exists in the routing graph
    #[must_use] 
    pub fn has_token(&self, address: &str) -> bool {
        self.graph.has_token(address)
    }

    /// Get statistics about the graph
    #[must_use] 
    pub fn stats(&self) -> RouterStats {
        RouterStats {
            chain: self.chain.clone(),
            token_count: self.graph.token_count(),
            edge_count: self.graph.edge_count(),
        }
    }

    /// Encode a swap transaction for the router contract
    ///
    /// # Arguments
    /// * `route` - The route to execute
    /// * `amount_in` - Input amount as decimal string (will be parsed)
    /// * `min_amount_out` - Minimum output amount as decimal string
    ///
    /// # Returns
    /// ABI-encoded calldata for the `exchange` function
    pub fn encode_swap(
        &self,
        route: &Route,
        amount_in: &str,
        min_amount_out: &str,
    ) -> Result<Vec<u8>> {
        encode_exchange_calldata(route, amount_in, min_amount_out)
    }

    /// Get calldata for quoting (`get_dy` call)
    ///
    /// # Arguments
    /// * `route` - The route to quote
    /// * `amount_in` - Input amount as decimal string
    ///
    /// # Returns
    /// ABI-encoded calldata for the `get_dy` function
    pub fn encode_quote(&self, route: &Route, amount_in: &str) -> Result<Vec<u8>> {
        encode_get_dy_calldata(route, amount_in)
    }
}

/// Statistics about the router graph
#[derive(Debug, Clone)]
pub struct RouterStats {
    /// Chain identifier
    pub chain: String,
    /// Number of tokens in the graph
    pub token_count: usize,
    /// Number of edges (possible swaps)
    pub edge_count: usize,
}

/// ABI encoding helpers
///
/// The router contract has two main functions:
/// - `exchange(address[11] _route, uint256[5][5] _swap_params, uint256 _amount, uint256 _expected)`
/// - `get_dy(address[11] _route, uint256[5][5] _swap_params, uint256 _amount)`
///
/// Encode calldata for the `exchange` function
fn encode_exchange_calldata(
    route: &Route,
    amount_in: &str,
    min_amount_out: &str,
) -> Result<Vec<u8>> {
    // Function selector: exchange(address[11],uint256[5][5],uint256,uint256)
    // keccak256("exchange(address[11],uint256[5][5],uint256,uint256)")[0:4]
    let selector: [u8; 4] = [0x1a, 0x4c, 0x1c, 0xa3];

    let (route_addrs, swap_params) = route.to_contract_format();

    // Parse amounts
    let amount_in: u128 = amount_in
        .parse()
        .map_err(|_| crate::error::invalid_param("Invalid amount_in".to_string()))?;
    let min_out: u128 = min_amount_out
        .parse()
        .map_err(|_| crate::error::invalid_param("Invalid min_amount_out".to_string()))?;

    // Build calldata
    let mut calldata = Vec::with_capacity(4 + 32 * 11 + 32 * 25 + 32 + 32);
    calldata.extend_from_slice(&selector);

    // Encode address[11] - each address is padded to 32 bytes
    for addr in &route_addrs {
        let addr_bytes = parse_address(addr)?;
        // Left-pad to 32 bytes
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(&addr_bytes);
    }

    // Encode uint256[5][5] - flattened, each uint256 is 32 bytes
    for row in &swap_params {
        for &val in row {
            let mut padded = [0u8; 32];
            padded[31] = val;
            calldata.extend_from_slice(&padded);
        }
    }

    // Encode _amount (uint256)
    let mut amount_bytes = [0u8; 32];
    amount_bytes[16..32].copy_from_slice(&amount_in.to_be_bytes());
    calldata.extend_from_slice(&amount_bytes);

    // Encode _expected (uint256)
    let mut expected_bytes = [0u8; 32];
    expected_bytes[16..32].copy_from_slice(&min_out.to_be_bytes());
    calldata.extend_from_slice(&expected_bytes);

    Ok(calldata)
}

/// Encode calldata for the `get_dy` function (quote)
fn encode_get_dy_calldata(route: &Route, amount_in: &str) -> Result<Vec<u8>> {
    // Function selector: get_dy(address[11],uint256[5][5],uint256)
    // keccak256("get_dy(address[11],uint256[5][5],uint256)")[0:4]
    let selector: [u8; 4] = [0x55, 0x6d, 0x6e, 0x9f];

    let (route_addrs, swap_params) = route.to_contract_format();

    // Parse amount
    let amount_in: u128 = amount_in
        .parse()
        .map_err(|_| crate::error::invalid_param("Invalid amount_in".to_string()))?;

    // Build calldata
    let mut calldata = Vec::with_capacity(4 + 32 * 11 + 32 * 25 + 32);
    calldata.extend_from_slice(&selector);

    // Encode address[11]
    for addr in &route_addrs {
        let addr_bytes = parse_address(addr)?;
        calldata.extend_from_slice(&[0u8; 12]);
        calldata.extend_from_slice(&addr_bytes);
    }

    // Encode uint256[5][5]
    for row in &swap_params {
        for &val in row {
            let mut padded = [0u8; 32];
            padded[31] = val;
            calldata.extend_from_slice(&padded);
        }
    }

    // Encode _amount (uint256)
    let mut amount_bytes = [0u8; 32];
    amount_bytes[16..32].copy_from_slice(&amount_in.to_be_bytes());
    calldata.extend_from_slice(&amount_bytes);

    Ok(calldata)
}

/// Parse an address string to bytes
fn parse_address(addr: &str) -> Result<[u8; 20]> {
    let addr = addr.strip_prefix("0x").unwrap_or(addr);
    if addr.is_empty() {
        return Ok([0u8; 20]);
    }
    if addr.len() != 40 {
        return Err(crate::error::invalid_param(format!(
            "Invalid address length: {}",
            addr.len()
        )));
    }

    let mut bytes = [0u8; 20];
    for (i, chunk) in addr.as_bytes().chunks(2).enumerate() {
        let hex_str = std::str::from_utf8(chunk)
            .map_err(|_| crate::error::invalid_param("Invalid hex".to_string()))?;
        bytes[i] = u8::from_str_radix(hex_str, 16)
            .map_err(|_| crate::error::invalid_param("Invalid hex digit".to_string()))?;
    }

    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_pool(id: &str, address: &str, coins: Vec<(&str, &str)>, tvl: f64) -> Pool {
        let coins_json: Vec<_> = coins
            .iter()
            .map(|(addr, symbol)| {
                json!({
                    "address": addr,
                    "symbol": symbol
                })
            })
            .collect();

        Pool(json!({
            "id": id,
            "address": address,
            "lpTokenAddress": format!("0xlp_{}", id),
            "coins": coins_json,
            "usdTotal": tvl,
            "isCrypto": false,
            "isFactory": false,
            "isLending": false,
            "isMetaPool": false
        }))
    }

    #[test]
    fn test_router_api_creation() {
        let pools = vec![make_pool(
            "3pool",
            "0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7",
            vec![
                ("0x6B175474E89094C44Da98b954EedeAC495271d0F", "DAI"),
                ("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "USDC"),
                ("0xdAC17F958D2ee523a2206206994597C13D831ec7", "USDT"),
            ],
            100_000_000.0,
        )];

        let router = RouterApi::new("ethereum", &pools);

        let stats = router.stats();
        assert_eq!(stats.chain, "ethereum");
        assert!(stats.token_count > 0);
        assert!(stats.edge_count > 0);

        assert!(router.router_address().is_some());
    }

    #[test]
    fn test_find_route_in_pool() {
        let pools = vec![make_pool(
            "3pool",
            "0x3pool",
            vec![("0xdai", "DAI"), ("0xusdc", "USDC"), ("0xusdt", "USDT")],
            100_000_000.0,
        )];

        let router = RouterApi::new("ethereum", &pools);

        // Should find direct route DAI -> USDC
        let routes = router.find_routes("0xdai", "0xusdc");
        assert!(!routes.is_empty());
        assert_eq!(routes[0].steps.len(), 1);
    }

    #[test]
    fn test_encode_swap_calldata() {
        let pools = vec![make_pool(
            "3pool",
            "0xbEbc44782C7dB0a1A60Cb6fe97d0b483032FF1C7",
            vec![
                ("0x6B175474E89094C44Da98b954EedeAC495271d0F", "DAI"),
                ("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "USDC"),
            ],
            100_000_000.0,
        )];

        let router = RouterApi::new("ethereum", &pools);
        let routes = router.find_routes(
            "0x6B175474E89094C44Da98b954EedeAC495271d0F",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        );

        assert!(!routes.is_empty());

        let calldata = router
            .encode_swap(&routes[0], "1000000000000000000", "990000")
            .unwrap();

        // Should start with function selector
        assert_eq!(&calldata[0..4], &[0x1a, 0x4c, 0x1c, 0xa3]);

        // Calldata should be: 4 (selector) + 11*32 (addresses) + 25*32 (params) + 32 (amount) + 32 (expected)
        // = 4 + 352 + 800 + 32 + 32 = 1220 bytes
        assert_eq!(calldata.len(), 1220);
    }

    #[test]
    fn test_parse_address() {
        let addr = parse_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
        assert_eq!(addr[0], 0xA0);
        assert_eq!(addr[19], 0x48);

        // Without 0x prefix
        let addr2 = parse_address("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").unwrap();
        assert_eq!(addr, addr2);

        // Empty address
        let empty = parse_address("").unwrap();
        assert_eq!(empty, [0u8; 20]);
    }
}
