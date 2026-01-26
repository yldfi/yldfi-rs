//! Types for Uniswap data

use alloy::primitives::{Address, U256};
use serde::{Deserialize, Serialize};

/// Uniswap pool information (V3)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pool {
    /// Pool contract address
    pub address: Address,
    /// Token0 address
    pub token0: Address,
    /// Token1 address
    pub token1: Address,
    /// Fee tier in basis points (e.g., 500 = 0.05%, 3000 = 0.3%, 10000 = 1%)
    pub fee: u32,
    /// Current tick
    pub tick: i32,
    /// Current sqrt price as Q64.96
    pub sqrt_price_x96: U256,
    /// Total liquidity
    pub liquidity: u128,
}

/// Pool state from on-chain query
#[derive(Debug, Clone)]
pub struct PoolState {
    /// Current sqrt price as Q64.96
    pub sqrt_price_x96: U256,
    /// Current tick
    pub tick: i32,
    /// Observation index
    pub observation_index: u16,
    /// Observation cardinality
    pub observation_cardinality: u16,
    /// Observation cardinality next
    pub observation_cardinality_next: u16,
    /// Fee protocol
    pub fee_protocol: u8,
    /// Whether the pool is unlocked
    pub unlocked: bool,
}

/// Quote result for a swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    /// Input token address
    pub token_in: Address,
    /// Output token address
    pub token_out: Address,
    /// Input amount
    pub amount_in: U256,
    /// Expected output amount
    pub amount_out: U256,
    /// Price impact as percentage (0.01 = 1%)
    pub price_impact: f64,
    /// Route taken (pool addresses)
    pub route: Vec<Address>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// Token contract address
    pub address: Address,
    /// Token symbol (e.g., "WETH")
    pub symbol: String,
    /// Token name (e.g., "Wrapped Ether")
    pub name: String,
    /// Token decimals
    pub decimals: u8,
}

// ============================================================================
// Subgraph types (historical data)
// ============================================================================

/// Historical swap event from subgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Swap {
    /// Swap ID
    pub id: String,
    /// Transaction info (nested object)
    pub transaction: SwapTransaction,
    /// Block timestamp (as string from subgraph)
    pub timestamp: String,
    /// Pool info (nested object)
    pub pool: SwapPool,
    /// Sender address
    pub sender: String,
    /// Recipient address
    pub recipient: String,
    /// Amount of token0
    pub amount0: String,
    /// Amount of token1
    pub amount1: String,
    /// USD value of the swap
    #[serde(rename = "amountUSD")]
    pub amount_usd: Option<String>,
}

/// Transaction reference in swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapTransaction {
    /// Transaction hash
    pub id: String,
}

/// Pool reference in swap
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapPool {
    /// Pool address
    pub id: String,
}

/// Pool data from subgraph (includes historical metrics)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolData {
    /// Pool ID (address)
    pub id: String,
    /// Token0 info
    pub token0: SubgraphToken,
    /// Token1 info
    pub token1: SubgraphToken,
    /// Fee tier
    #[serde(rename = "feeTier")]
    pub fee_tier: String,
    /// Total value locked in USD
    #[serde(rename = "totalValueLockedUSD")]
    pub total_value_locked_usd: String,
    /// 24h volume in USD
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
    /// Total fees collected in USD
    #[serde(rename = "feesUSD")]
    pub fees_usd: String,
    /// Number of transactions
    #[serde(rename = "txCount")]
    pub tx_count: String,
}

/// Token data from subgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubgraphToken {
    /// Token address
    pub id: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Token decimals
    pub decimals: String,
}

/// Daily pool statistics from subgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolDayData {
    /// Date (unix timestamp at start of day)
    pub date: u64,
    /// Pool info (nested object)
    pub pool: SwapPool,
    /// Volume in USD
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
    /// TVL in USD
    #[serde(rename = "tvlUSD")]
    pub tvl_usd: String,
    /// Fees in USD
    #[serde(rename = "feesUSD")]
    pub fees_usd: String,
    /// Open price (token1/token0)
    pub open: String,
    /// High price
    pub high: String,
    /// Low price
    pub low: String,
    /// Close price
    pub close: String,
}

/// ETH price bundle from subgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bundle {
    /// Bundle ID (always "1")
    pub id: String,
    /// ETH price in USD
    #[serde(rename = "ethPriceUSD")]
    pub eth_price_usd: String,
}

/// Liquidity position from subgraph (V3 LP NFT position)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    /// Position ID (NFT token ID)
    pub id: String,
    /// Owner address
    pub owner: String,
    /// Pool this position is in
    pub pool: PositionPool,
    /// Liquidity amount
    pub liquidity: String,
    /// Amount of token0 deposited
    pub deposited_token0: String,
    /// Amount of token1 deposited
    pub deposited_token1: String,
    /// Amount of token0 withdrawn
    pub withdrawn_token0: String,
    /// Amount of token1 withdrawn
    pub withdrawn_token1: String,
    /// Fees collected in token0
    pub collected_fees_token0: String,
    /// Fees collected in token1
    pub collected_fees_token1: String,
    /// Lower tick of the position
    pub tick_lower: PositionTick,
    /// Upper tick of the position
    pub tick_upper: PositionTick,
}

/// Pool reference in position
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionPool {
    /// Pool address
    pub id: String,
    /// Token0 info
    pub token0: SubgraphToken,
    /// Token1 info
    pub token1: SubgraphToken,
    /// Fee tier
    pub fee_tier: String,
    /// Current tick
    pub tick: Option<String>,
    /// Current sqrt price
    pub sqrt_price: Option<String>,
    /// Token0 price in USD
    pub token0_price: Option<String>,
    /// Token1 price in USD
    pub token1_price: Option<String>,
}

/// Tick reference in position
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionTick {
    /// Tick index
    pub tick_idx: String,
}

/// V2 Liquidity position (ERC-20 LP tokens)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquidityPositionV2 {
    /// Position ID
    pub id: String,
    /// User address
    pub user: UserRef,
    /// Pair info
    pub pair: PairV2,
    /// LP token balance
    pub liquidity_token_balance: String,
}

/// User reference in V2 position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRef {
    /// User address
    pub id: String,
}

/// V2 Pair data (for positions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairV2 {
    /// Pair address
    pub id: String,
    /// Token0
    pub token0: SubgraphToken,
    /// Token1
    pub token1: SubgraphToken,
    /// Reserve of token0
    pub reserve0: String,
    /// Reserve of token1
    pub reserve1: String,
    /// Total supply of LP tokens
    pub total_supply: String,
    /// Reserve in USD
    pub reserve_usd: Option<String>,
}

/// V2 Pair data with volume/fees (for yield calculation)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairDataV2 {
    /// Pair address
    pub id: String,
    /// Token0
    pub token0: SubgraphToken,
    /// Token1
    pub token1: SubgraphToken,
    /// Reserve of token0
    pub reserve0: String,
    /// Reserve of token1
    pub reserve1: String,
    /// Total supply of LP tokens
    pub total_supply: String,
    /// Reserve in USD (TVL)
    #[serde(rename = "reserveUSD")]
    pub reserve_usd: String,
    /// Total volume in USD
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
    /// Number of transactions
    #[serde(rename = "txCount")]
    pub tx_count: String,
}

/// V4 Position (similar to V3 but with pool key instead of pool address)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionV4 {
    /// Position ID (NFT token ID)
    pub id: String,
    /// Owner address
    pub owner: String,
    /// Pool this position is in
    pub pool: PoolV4,
    /// Liquidity amount
    pub liquidity: String,
    /// Lower tick
    pub tick_lower: i32,
    /// Upper tick
    pub tick_upper: i32,
}

/// V4 Pool data (for positions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolV4 {
    /// Pool ID
    pub id: String,
    /// Token0
    pub token0: SubgraphToken,
    /// Token1
    pub token1: SubgraphToken,
    /// Fee tier
    pub fee: String,
    /// Hook address (V4 feature)
    pub hooks: Option<String>,
    /// Total value locked in USD
    pub total_value_locked_usd: Option<String>,
}

/// V4 Pool data with volume/fees (for yield calculation)
/// Schema matches official Uniswap V4 subgraph
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolDataV4 {
    /// Pool ID
    pub id: String,
    /// Token0
    pub token0: SubgraphToken,
    /// Token1
    pub token1: SubgraphToken,
    /// Fee tier (in hundredths of a bip, e.g., 3000 = 0.3%)
    pub fee_tier: String,
    /// Hook address (V4 feature)
    pub hooks: Option<String>,
    /// Total value locked in USD
    #[serde(rename = "totalValueLockedUSD")]
    pub total_value_locked_usd: String,
    /// Total volume in USD
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
    /// Total fees in USD
    #[serde(rename = "feesUSD")]
    pub fees_usd: String,
    /// Number of transactions
    #[serde(rename = "txCount")]
    pub tx_count: String,
}

// ============================================================================
// GraphQL response wrappers
// ============================================================================

/// Generic GraphQL response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLResponse<T> {
    /// Response data
    pub data: Option<T>,
    /// Errors if any
    pub errors: Option<Vec<GraphQLError>>,
}

/// GraphQL error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLError {
    /// Error message
    pub message: String,
}

impl<T> GraphQLResponse<T> {
    /// Check if the response has errors
    pub fn has_errors(&self) -> bool {
        self.errors.as_ref().is_some_and(|e| !e.is_empty())
    }

    /// Get the first error message if any
    pub fn first_error(&self) -> Option<&str> {
        self.errors
            .as_ref()
            .and_then(|e| e.first())
            .map(|e| e.message.as_str())
    }
}
