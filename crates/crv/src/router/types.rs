//! Types for the Curve Router
//!
//! These types model the Curve Router-NG contract interface for building
//! swap routes across multiple pools.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum number of hops in a route (matching curve-router-ng)
pub const MAX_ROUTE_STEPS: usize = 5;

/// Maximum number of edges per token pair in the graph
pub const GRAPH_MAX_EDGES: usize = 3;

/// Maximum search depth for route finding
pub const MAX_SEARCH_DEPTH: usize = 4;

/// Maximum routes to track per sorting criterion
pub const MAX_ROUTES_PER_CRITERION: usize = 5;

/// Minimum TVL in USD for a pool to be included in routing
pub const MIN_TVL_USD: f64 = 1000.0;

/// Swap type definitions matching curve-router-ng Router.vy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u8)]
pub enum SwapType {
    /// Standard exchange on pools
    Exchange = 1,
    /// Exchange underlying tokens
    ExchangeUnderlying = 2,
    /// Underlying exchange via zap (factory stable metapools with lending base pool or crypto-meta pools)
    ExchangeUnderlyingZap = 3,
    /// Coin to LP token (`add_liquidity`)
    AddLiquidity = 4,
    /// Lending pool underlying coin to LP token (`add_liquidity`)
    AddLiquidityUnderlying = 5,
    /// LP token to coin (`remove_liquidity_one_coin`)
    RemoveLiquidity = 6,
    /// LP token to lending/fake pool underlying coin (`remove_liquidity_one_coin`)
    RemoveLiquidityUnderlying = 7,
    /// Special wrapping: ETH↔WETH, ETH→stETH, ETH→frxETH, stETH↔wstETH, ETH→wBETH
    Wrap = 8,
    /// ERC4626 asset ↔ share conversions
    Erc4626 = 9,
}

impl SwapType {
    /// Convert from u8
    #[must_use] 
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Exchange),
            2 => Some(Self::ExchangeUnderlying),
            3 => Some(Self::ExchangeUnderlyingZap),
            4 => Some(Self::AddLiquidity),
            5 => Some(Self::AddLiquidityUnderlying),
            6 => Some(Self::RemoveLiquidity),
            7 => Some(Self::RemoveLiquidityUnderlying),
            8 => Some(Self::Wrap),
            9 => Some(Self::Erc4626),
            _ => None,
        }
    }

    /// Convert to u8
    #[must_use] 
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Pool type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[repr(u8)]
pub enum PoolType {
    /// Main/stable pools
    #[default]
    Main = 1,
    /// Crypto/volatile pools
    Crypto = 2,
    /// Factory pools
    Factory = 3,
    /// Factory crypto pools
    FactoryCrypto = 4,
    /// Lending pools
    Lending = 5,
}

impl PoolType {
    /// Convert from u8
    #[must_use] 
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            1 => Some(Self::Main),
            2 => Some(Self::Crypto),
            3 => Some(Self::Factory),
            4 => Some(Self::FactoryCrypto),
            5 => Some(Self::Lending),
            _ => None,
        }
    }

    /// Convert to u8
    #[must_use] 
    pub fn as_u8(self) -> u8 {
        self as u8
    }
}

/// Swap parameters for a single step [i, j, `swap_type`, `pool_type`, `n_coins`]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SwapParams {
    /// Input coin index in pool
    pub i: u8,
    /// Output coin index in pool
    pub j: u8,
    /// Type of swap to perform
    pub swap_type: SwapType,
    /// Type of pool
    pub pool_type: PoolType,
    /// Number of coins in pool
    pub n_coins: u8,
}

impl SwapParams {
    /// Create new swap parameters
    #[must_use] 
    pub fn new(i: u8, j: u8, swap_type: SwapType, pool_type: PoolType, n_coins: u8) -> Self {
        Self {
            i,
            j,
            swap_type,
            pool_type,
            n_coins,
        }
    }

    /// Convert to array format [i, j, `swap_type`, `pool_type`, `n_coins`]
    #[must_use] 
    pub fn to_array(&self) -> [u8; 5] {
        [
            self.i,
            self.j,
            self.swap_type.as_u8(),
            self.pool_type.as_u8(),
            self.n_coins,
        ]
    }

    /// Create from array format
    #[must_use] 
    pub fn from_array(arr: [u8; 5]) -> Option<Self> {
        Some(Self {
            i: arr[0],
            j: arr[1],
            swap_type: SwapType::from_u8(arr[2])?,
            pool_type: PoolType::from_u8(arr[3])?,
            n_coins: arr[4],
        })
    }
}

/// A single step in a swap route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteStep {
    /// Pool ID (for reference)
    pub pool_id: String,
    /// Pool/swap address to use
    pub pool_address: String,
    /// Input token address
    pub input_coin: String,
    /// Output token address
    pub output_coin: String,
    /// Swap parameters
    pub swap_params: SwapParams,
    /// TVL of the pool in USD (for sorting)
    pub tvl_usd: f64,
}

impl RouteStep {
    /// Create a new route step
    pub fn new(
        pool_id: impl Into<String>,
        pool_address: impl Into<String>,
        input_coin: impl Into<String>,
        output_coin: impl Into<String>,
        swap_params: SwapParams,
        tvl_usd: f64,
    ) -> Self {
        Self {
            pool_id: pool_id.into(),
            pool_address: pool_address.into(),
            input_coin: input_coin.into(),
            output_coin: output_coin.into(),
            swap_params,
            tvl_usd,
        }
    }
}

/// A complete route from input to output token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Steps in the route
    pub steps: Vec<RouteStep>,
    /// Input token address
    pub input_token: String,
    /// Output token address
    pub output_token: String,
    /// Minimum TVL across all steps
    pub min_tvl: f64,
    /// Total TVL across all steps
    pub total_tvl: f64,
}

impl Route {
    /// Create a new empty route
    pub fn new(input_token: impl Into<String>, output_token: impl Into<String>) -> Self {
        Self {
            steps: Vec::new(),
            input_token: input_token.into(),
            output_token: output_token.into(),
            min_tvl: f64::MAX,
            total_tvl: 0.0,
        }
    }

    /// Add a step to the route
    pub fn push(&mut self, step: RouteStep) {
        self.min_tvl = self.min_tvl.min(step.tvl_usd);
        self.total_tvl += step.tvl_usd;
        self.steps.push(step);
    }

    /// Get number of steps (hops)
    #[must_use] 
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if route is empty
    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Convert to router contract format
    /// Returns (route[11], `swap_params`[5][5])
    #[must_use] 
    pub fn to_contract_format(&self) -> ([String; 11], [[u8; 5]; 5]) {
        // Route format: [input, pool, output, pool, output, ...]
        // Alternating: token, pool/zap, token, pool/zap, token...
        let mut route: [String; 11] = Default::default();
        let mut params: [[u8; 5]; 5] = [[0; 5]; 5];

        if self.steps.is_empty() {
            return (route, params);
        }

        // First token
        route[0] = self.steps[0].input_coin.clone();

        for (i, step) in self.steps.iter().enumerate() {
            if i >= MAX_ROUTE_STEPS {
                break;
            }
            // Pool address at odd indices (1, 3, 5, 7, 9)
            route[i * 2 + 1] = step.pool_address.clone();
            // Output token at even indices (2, 4, 6, 8, 10)
            route[i * 2 + 2] = step.output_coin.clone();
            // Swap params
            params[i] = step.swap_params.to_array();
        }

        (route, params)
    }
}

/// A route with quote information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotedRoute {
    /// The route
    pub route: Route,
    /// Expected output amount (raw, needs decimal adjustment)
    pub expected_output: String,
    /// Price impact as percentage (0.01 = 1%)
    pub price_impact: Option<f64>,
    /// Estimated gas cost
    pub estimated_gas: Option<u64>,
}

/// Edge in the routing graph connecting two tokens
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// The route step for this edge
    pub step: RouteStep,
}

/// Node in the routing graph (a token)
#[derive(Debug, Clone, Default)]
pub struct GraphNode {
    /// Token address (lowercase)
    pub address: String,
    /// Outgoing edges to other tokens
    pub edges: Vec<GraphEdge>,
}

/// Route graph for finding swap paths
#[derive(Debug, Clone, Default)]
pub struct RouteGraph {
    /// Nodes indexed by token address (lowercase)
    pub nodes: HashMap<String, GraphNode>,
    /// Chain ID this graph is for
    pub chain_id: String,
}

impl RouteGraph {
    /// Create a new empty graph
    pub fn new(chain_id: impl Into<String>) -> Self {
        Self {
            nodes: HashMap::new(),
            chain_id: chain_id.into(),
        }
    }

    /// Get or create a node for a token
    pub fn get_or_create_node(&mut self, address: &str) -> &mut GraphNode {
        let key = address.to_lowercase();
        self.nodes.entry(key.clone()).or_insert_with(|| GraphNode {
            address: key,
            edges: Vec::new(),
        })
    }

    /// Add an edge between two tokens
    pub fn add_edge(&mut self, from: &str, to: &str, step: RouteStep) {
        let from_key = from.to_lowercase();
        let to_key = to.to_lowercase();

        // Get the from node
        let node = self.get_or_create_node(&from_key);

        // Check if we already have too many edges to this destination
        let existing_to_dest = node
            .edges
            .iter()
            .filter(|e| e.step.output_coin.to_lowercase() == to_key)
            .count();

        if existing_to_dest < GRAPH_MAX_EDGES {
            node.edges.push(GraphEdge { step });
        } else {
            // Replace lowest TVL edge if new one is better
            if let Some((idx, min_edge)) = node
                .edges
                .iter()
                .enumerate()
                .filter(|(_, e)| e.step.output_coin.to_lowercase() == to_key)
                .min_by(|a, b| a.1.step.tvl_usd.total_cmp(&b.1.step.tvl_usd))
            {
                if step.tvl_usd > min_edge.step.tvl_usd {
                    node.edges[idx] = GraphEdge { step };
                }
            }
        }

        // Ensure destination node exists
        self.get_or_create_node(&to_key);
    }

    /// Get edges from a token
    #[must_use] 
    pub fn get_edges(&self, from: &str) -> Option<&[GraphEdge]> {
        self.nodes
            .get(&from.to_lowercase())
            .map(|n| n.edges.as_slice())
    }

    /// Check if a token exists in the graph
    #[must_use] 
    pub fn has_token(&self, address: &str) -> bool {
        self.nodes.contains_key(&address.to_lowercase())
    }

    /// Get number of tokens in the graph
    #[must_use] 
    pub fn token_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get total number of edges
    #[must_use] 
    pub fn edge_count(&self) -> usize {
        self.nodes.values().map(|n| n.edges.len()).sum()
    }
}

/// Known wrapper token pairs for special swap type 8
#[derive(Debug, Clone)]
pub struct WrapperPair {
    /// Token A address
    pub token_a: String,
    /// Token B address
    pub token_b: String,
    /// Symbol for token A
    pub symbol_a: String,
    /// Symbol for token B
    pub symbol_b: String,
}

/// Ethereum mainnet wrapper pairs
#[must_use] 
pub fn eth_wrapper_pairs() -> Vec<WrapperPair> {
    vec![
        WrapperPair {
            token_a: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_lowercase(),
            token_b: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_lowercase(),
            symbol_a: "ETH".to_string(),
            symbol_b: "WETH".to_string(),
        },
        WrapperPair {
            token_a: "0xae7ab96520DE3A18E5e111B5EaAb095312D7fE84".to_lowercase(),
            token_b: "0x7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0".to_lowercase(),
            symbol_a: "stETH".to_string(),
            symbol_b: "wstETH".to_string(),
        },
        WrapperPair {
            token_a: "0x5E8422345238F34275888049021821E8E08CAa1f".to_lowercase(),
            token_b: "0xac3E018457B222d93114458476f3E3416Abbe38F".to_lowercase(),
            symbol_a: "frxETH".to_string(),
            symbol_b: "sfrxETH".to_string(),
        },
    ]
}

/// Router contract addresses by chain
#[must_use] 
pub fn router_address(chain: &str) -> Option<&'static str> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" => Some("0x16C6521Dff6baB339122a0FE25a9116693265353"),
        "optimism" | "op" => Some("0x0DCDED3545D565bA3B19E683431381007245d983"),
        "polygon" | "matic" => Some("0x0DCDED3545D565bA3B19E683431381007245d983"),
        "arbitrum" | "arb" => Some("0x2191718CD32d02B8E60BAdFFeA33E4B5DD9A0A0D"),
        "avalanche" | "avax" => Some("0x0DCDED3545D565bA3B19E683431381007245d983"),
        "fantom" | "ftm" => Some("0x0DCDED3545D565bA3B19E683431381007245d983"),
        "base" => Some("0x4f37A9d177470499A2dD084621020b023fcffc1F"),
        "bsc" | "bnb" => Some("0xA72C85C258A81761433B4e8da60505Fe3Dd551CC"),
        "gnosis" | "xdai" => Some("0x0DCDED3545D565bA3B19E683431381007245d983"),
        "kava" => Some("0x0DCDED3545D565bA3B19E683431381007245d983"),
        "fraxtal" => Some("0x9f2Fa7709B30c75047980a0d70A106728f0Ef2db"),
        "mantle" => Some("0x4f37A9d177470499A2dD084621020b023fcffc1F"),
        "zksync" => Some("0x7C915390e109CA66934f1eB285854375D1B127FA"),
        "xlayer" | "x-layer" => Some("0xBFab8ebc836E1c4D81837798FC076D219C9a1855"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_swap_params_array_roundtrip() {
        let params = SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2);
        let arr = params.to_array();
        let restored = SwapParams::from_array(arr).unwrap();
        assert_eq!(params, restored);
    }

    #[test]
    fn test_route_to_contract_format() {
        let mut route = Route::new("0xtoken_a", "0xtoken_c");

        route.push(RouteStep::new(
            "pool1",
            "0xpool1",
            "0xtoken_a",
            "0xtoken_b",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            1000.0,
        ));

        route.push(RouteStep::new(
            "pool2",
            "0xpool2",
            "0xtoken_b",
            "0xtoken_c",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Crypto, 2),
            2000.0,
        ));

        let (route_addrs, params) = route.to_contract_format();

        assert_eq!(route_addrs[0], "0xtoken_a");
        assert_eq!(route_addrs[1], "0xpool1");
        assert_eq!(route_addrs[2], "0xtoken_b");
        assert_eq!(route_addrs[3], "0xpool2");
        assert_eq!(route_addrs[4], "0xtoken_c");

        assert_eq!(params[0], [0, 1, 1, 1, 2]);
        assert_eq!(params[1], [0, 1, 1, 2, 2]);
    }

    #[test]
    fn test_graph_add_edge() {
        let mut graph = RouteGraph::new("ethereum");

        let step = RouteStep::new(
            "pool1",
            "0xpool1",
            "0xtoken_a",
            "0xtoken_b",
            SwapParams::new(0, 1, SwapType::Exchange, PoolType::Main, 2),
            1000.0,
        );

        graph.add_edge("0xtoken_a", "0xtoken_b", step);

        assert!(graph.has_token("0xtoken_a"));
        assert!(graph.has_token("0xtoken_b"));
        assert_eq!(graph.get_edges("0xtoken_a").unwrap().len(), 1);
    }

    #[test]
    fn test_router_addresses() {
        assert!(router_address("ethereum").is_some());
        assert!(router_address("polygon").is_some());
        assert!(router_address("unknown_chain").is_none());
    }
}
