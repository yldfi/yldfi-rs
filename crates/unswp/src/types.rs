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
///
/// Deserializes V3/V4 swaps directly and V2 swaps (`pair`, `to`,
/// `amount{0,1}{In,Out}`) via `RawSwap`, normalizing V2 amounts to signed
/// pool deltas (`amount = in - out`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", from = "RawSwap")]
pub struct Swap {
    /// Swap ID
    pub id: String,
    /// Transaction info (nested object)
    pub transaction: SwapTransaction,
    /// Block timestamp (as string from subgraph)
    pub timestamp: String,
    /// Pool info (nested object; the pair for V2)
    pub pool: SwapPool,
    /// Sender address
    pub sender: String,
    /// Recipient address (V2 `to`, V3 `recipient`; not indexed by V4)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recipient: Option<String>,
    /// EOA that initiated the transaction (V3/V4 `origin`, V2 `from`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    /// Amount of token0 (signed pool delta)
    pub amount0: String,
    /// Amount of token1 (signed pool delta)
    pub amount1: String,
    /// USD value of the swap
    #[serde(rename = "amountUSD")]
    pub amount_usd: Option<String>,
}

/// Wire format covering V2, V3 and V4 swap entities
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawSwap {
    id: String,
    transaction: SwapTransaction,
    timestamp: String,
    #[serde(default)]
    pool: Option<SwapPool>,
    #[serde(default)]
    pair: Option<SwapPool>,
    sender: String,
    #[serde(default)]
    recipient: Option<String>,
    #[serde(default)]
    to: Option<String>,
    #[serde(default)]
    origin: Option<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    amount0: Option<String>,
    #[serde(default)]
    amount1: Option<String>,
    #[serde(default)]
    amount0_in: Option<String>,
    #[serde(default)]
    amount1_in: Option<String>,
    #[serde(default)]
    amount0_out: Option<String>,
    #[serde(default)]
    amount1_out: Option<String>,
    #[serde(default, rename = "amountUSD")]
    amount_usd: Option<String>,
}

/// Parse a non-negative decimal string into `(digits, scale)`
fn parse_decimal(s: &str) -> Option<(U256, usize)> {
    let s = s.trim();
    if s.is_empty() {
        return Some((U256::ZERO, 0));
    }
    let (int, frac) = s.split_once('.').unwrap_or((s, ""));
    if !int.chars().chain(frac.chars()).all(|c| c.is_ascii_digit()) {
        return None;
    }
    let digits = format!("{int}{frac}");
    let value = if digits.is_empty() {
        U256::ZERO
    } else {
        U256::from_str_radix(&digits, 10).ok()?
    };
    Some((value, frac.len()))
}

/// Format `value / 10^scale` as a decimal string without trailing zeros
fn format_decimal(value: U256, scale: usize) -> String {
    let digits = value.to_string();
    if scale == 0 {
        return digits;
    }
    let padded = format!("{digits:0>width$}", width = scale + 1);
    let (int, frac) = padded.split_at(padded.len() - scale);
    let frac = frac.trim_end_matches('0');
    if frac.is_empty() {
        int.to_string()
    } else {
        format!("{int}.{frac}")
    }
}

/// `in - out` as an exact decimal string (subgraph `BigDecimal` values)
///
/// Falls back to `f64` only if a value is not a plain non-negative decimal.
fn signed_delta(amount_in: Option<&str>, amount_out: Option<&str>) -> String {
    let exact = parse_decimal(amount_in.unwrap_or("0"))
        .zip(parse_decimal(amount_out.unwrap_or("0")))
        .and_then(|((a, sa), (b, sb))| {
            let scale = sa.max(sb);
            let ten = U256::from(10u8);
            let a = a.checked_mul(ten.checked_pow(U256::from(scale - sa))?)?;
            let b = b.checked_mul(ten.checked_pow(U256::from(scale - sb))?)?;
            Some(if a >= b {
                format_decimal(a - b, scale)
            } else {
                format!("-{}", format_decimal(b - a, scale))
            })
        });
    exact.unwrap_or_else(|| {
        let parse = |v: Option<&str>| v.and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
        (parse(amount_in) - parse(amount_out)).to_string()
    })
}

impl From<RawSwap> for Swap {
    fn from(r: RawSwap) -> Self {
        let amount0 = r
            .amount0
            .unwrap_or_else(|| signed_delta(r.amount0_in.as_deref(), r.amount0_out.as_deref()));
        let amount1 = r
            .amount1
            .unwrap_or_else(|| signed_delta(r.amount1_in.as_deref(), r.amount1_out.as_deref()));
        Self {
            id: r.id,
            transaction: r.transaction,
            timestamp: r.timestamp,
            pool: r.pool.or(r.pair).unwrap_or(SwapPool { id: String::new() }),
            sender: r.sender,
            recipient: r.recipient.or(r.to),
            origin: r.origin.or(r.from),
            amount0,
            amount1,
            amount_usd: r.amount_usd,
        }
    }
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
    /// All-time cumulative volume in USD (not 24h; use day data for
    /// recent volume)
    #[serde(rename = "volumeUSD")]
    pub volume_usd: String,
    /// All-time cumulative fees collected in USD
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
    /// Fees in USD (V2: derived as 0.3% of `volume_usd`)
    #[serde(rename = "feesUSD")]
    pub fees_usd: String,
    /// Open price (token1/token0); `None` for V2 pairs
    #[serde(default)]
    pub open: Option<String>,
    /// High price; `None` for V2 pairs
    #[serde(default)]
    pub high: Option<String>,
    /// Low price; `None` for V2 pairs
    #[serde(default)]
    pub low: Option<String>,
    /// Close price; `None` for V2 pairs
    #[serde(default)]
    pub close: Option<String>,
}

/// V2 `pairDayDatas` row (mapped to [`PoolDayData`])
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PairDayDataV2 {
    date: u64,
    pair_address: String,
    #[serde(rename = "dailyVolumeUSD")]
    daily_volume_usd: String,
    #[serde(rename = "reserveUSD")]
    reserve_usd: String,
}

impl From<PairDayDataV2> for PoolDayData {
    fn from(r: PairDayDataV2) -> Self {
        Self {
            date: r.date,
            pool: SwapPool { id: r.pair_address },
            fees_usd: v2_fees_from_volume(&r.daily_volume_usd),
            volume_usd: r.daily_volume_usd,
            tvl_usd: r.reserve_usd,
            open: None,
            high: None,
            low: None,
            close: None,
        }
    }
}

/// V2 fees (0.3% of volume) as an exact decimal string
fn v2_fees_from_volume(volume: &str) -> String {
    match parse_decimal(volume) {
        Some((value, scale)) => value
            .checked_mul(U256::from(3u8))
            .map_or_else(String::new, |v| format_decimal(v, scale + 3)),
        None => String::new(),
    }
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

/// V4 position NFT as indexed by the V4 subgraph
///
/// The V4 subgraph only tracks ownership; pool key, ticks and liquidity are
/// stored on-chain in the `PositionManager` (see Uniswap/v4-subgraph
/// `schema.graphql`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PositionV4 {
    /// Position ID (NFT token ID)
    pub id: String,
    /// NFT token ID
    pub token_id: String,
    /// Current owner address
    pub owner: String,
    /// EOA that minted the position
    #[serde(default)]
    pub origin: Option<String>,
    /// Creation timestamp (unix seconds, as string)
    #[serde(default)]
    pub created_at_timestamp: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_delta_is_exact() {
        assert_eq!(signed_delta(Some("0"), Some("5000")), "-5000");
        assert_eq!(signed_delta(Some("2"), Some("0")), "2");
        assert_eq!(signed_delta(Some("0.1"), Some("0.3")), "-0.2");
        assert_eq!(
            signed_delta(Some("123456789012345678.123456789012345678"), Some("0")),
            "123456789012345678.123456789012345678"
        );
        assert_eq!(
            signed_delta(Some("1.000000000000000001"), Some("0.000000000000000001")),
            "1"
        );
        assert_eq!(signed_delta(None, Some("0.5")), "-0.5");
    }

    #[test]
    fn v2_pair_day_maps_to_pool_day() {
        let raw: PairDayDataV2 = serde_json::from_value(serde_json::json!({
            "date": 1700006400,
            "pairAddress": "0xpair",
            "dailyVolumeUSD": "1000.5",
            "reserveUSD": "250000"
        }))
        .unwrap();
        let day = PoolDayData::from(raw);
        assert_eq!(day.pool.id, "0xpair");
        assert_eq!(day.volume_usd, "1000.5");
        assert_eq!(day.tvl_usd, "250000");
        assert_eq!(day.fees_usd, "3.0015");
        assert!(day.open.is_none());
    }
}
