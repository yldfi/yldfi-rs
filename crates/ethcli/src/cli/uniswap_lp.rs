//! On-chain Uniswap LP position valuation
//!
//! - v3: positions and pools read from the NonfungiblePositionManager and the
//!   chain's factory, amounts from a `uniswap-v3-sdk` `Position`, fees from
//!   an `eth_call` of `NonfungiblePositionManager.collect` from the NFT owner,
//!   cross-checked against tokensOwed + pool fee growth.
//! - v4: pool key and ticks from the PositionManager, pool, tick and position
//!   state from the PoolManager (`extsload`), amounts from a `uniswap-v4-sdk`
//!   `Position`, uncollected fees from the PoolManager's fee growth.
//! - v2 and v2 forks: the pair's reserves and supply; fees are compounded
//!   into the reserves.
//!
//! Reads are batched through Multicall3 in as few stages as their
//! dependencies allow (see `uniswap_lp_reads`), with individual calls at
//! blocks before Multicall3 existed.
//!
//! Every read is pinned to one block so the figures are consistent.

use crate::cli::OutputFormat;
use alloy::eips::BlockId;
use alloy::primitives::{aliases::I24, Address, B256, U256, U512};
use alloy::providers::{DynProvider, Provider, ProviderBuilder};
use alloy::sol;
use clap::{Args, ValueEnum};
use futures::StreamExt;
use serde::Serialize;
use std::collections::HashMap;

use super::uniswap_lp_reads::{dedup, Call, ChainReader, Outcome, Reader};
pub use super::uniswap_lp_v4_scan::V4Discovery;
use uniswap_v4_sdk::prelude::sdk_core::addresses::{ChainAddresses, CHAIN_TO_ADDRESSES_MAP};
use uniswap_v4_sdk::prelude::sdk_core::prelude::FractionBase;

sol! {
    #[sol(rpc)]
    interface INonfungiblePositionManagerLite {
        struct CollectParams {
            uint256 tokenId;
            address recipient;
            uint128 amount0Max;
            uint128 amount1Max;
        }
        function ownerOf(uint256 tokenId) external view returns (address);
        function factory() external view returns (address);
        function positions(uint256 tokenId) external view returns (uint96 nonce, address operator, address token0, address token1, uint24 fee, int24 tickLower, int24 tickUpper, uint128 liquidity, uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128, uint128 tokensOwed0, uint128 tokensOwed1);
        function collect(CollectParams calldata params) external payable returns (uint256 amount0, uint256 amount1);
    }

    #[sol(rpc)]
    interface IUniswapV3FactoryLite {
        function getPool(address tokenA, address tokenB, uint24 fee) external view returns (address);
    }

    #[sol(rpc)]
    interface IUniswapV3PoolLite {
        function slot0() external view returns (uint160 sqrtPriceX96, int24 tick, uint16 observationIndex, uint16 observationCardinality, uint16 observationCardinalityNext, uint8 feeProtocol, bool unlocked);
        function liquidity() external view returns (uint128);
        function feeGrowthGlobal0X128() external view returns (uint256);
        function feeGrowthGlobal1X128() external view returns (uint256);
        function ticks(int24 tick) external view returns (uint128 liquidityGross, int128 liquidityNet, uint256 feeGrowthOutside0X128, uint256 feeGrowthOutside1X128, int56 tickCumulativeOutside, uint160 secondsPerLiquidityOutsideX128, uint32 secondsOutside, bool initialized);
    }

    #[sol(rpc)]
    interface IV4PositionManagerLite {
        struct PoolKey {
            address currency0;
            address currency1;
            uint24 fee;
            int24 tickSpacing;
            address hooks;
        }
        function ownerOf(uint256 tokenId) external view returns (address);
        function getPoolAndPositionInfo(uint256 tokenId) external view returns (PoolKey memory poolKey, uint256 info);
        function getPositionLiquidity(uint256 tokenId) external view returns (uint128 liquidity);
    }

    interface IErc20MetaLite {
        function decimals() external view returns (uint256);
        function symbol() external view returns (string);
    }

    interface IPoolManagerExtsloadLite {
        function extsload(bytes32[] calldata slots) external view returns (bytes32[] memory values);
    }

    #[sol(rpc)]
    interface IUniswapV2PairLite {
        function balanceOf(address owner) external view returns (uint256);
        function totalSupply() external view returns (uint256);
        function getReserves() external view returns (uint112 reserve0, uint112 reserve1, uint32 blockTimestampLast);
        function token0() external view returns (address);
        function token1() external view returns (address);
    }
}

/// v4 pools with this fee value read their LP fee from slot0 (dynamic fee)
const DYNAMIC_FEE_FLAG: u32 = 0x80_0000;

/// Multicall3, at the same address on every chain that has it
const MULTICALL3: Address =
    alloy::primitives::address!("0xcA11bde05977b3631167028862bE2a173976CA11");

/// Block ranges of the v4 Transfer log scan requested at once
const LOG_SCAN_SEGMENTS: u64 = 4;

/// DefiLlama search window for historical prices
const HISTORICAL_SEARCH_WIDTH: &str = "4h";

/// collect eth_calls in flight at once (each is sent from its owner, so they
/// cannot be batched)
const COLLECT_CONCURRENCY: usize = 4;

/// Uniswap version of an NFT position
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum NftVersion {
    /// Uniswap v3 NonfungiblePositionManager
    #[default]
    V3,
    /// Uniswap v4 PositionManager
    V4,
}

/// Versions `lp-positions` lists
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum ListVersion {
    V3,
    V4,
    #[default]
    All,
}

/// Where `lp-positions` finds v4 token ids
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum V4Source {
    /// PositionManager `Transfer` logs to the owner (eth_getLogs)
    #[default]
    Logs,
    /// The block explorer's ERC-721 transfer list (Etherscan API key)
    Explorer,
}

/// Arguments for `uniswap position`
#[derive(Args, Debug)]
pub struct PositionArgs {
    /// NFT token id of a v3 or v4 position (omit when using --v2)
    #[arg(required_unless_present = "v2", conflicts_with = "v2")]
    pub token_id: Option<String>,

    /// Position manager version for TOKEN_ID
    #[arg(long, value_enum, default_value = "v3")]
    pub version: NftVersion,

    /// Uniswap v2-style pair (also Sushi and other v2 forks); needs --owner
    #[arg(long, value_name = "PAIR", requires = "owner")]
    pub v2: Option<String>,

    /// LP token holder for --v2 (address or address-book label)
    #[arg(long)]
    pub owner: Option<String>,

    /// Block number to value at (default: latest)
    #[arg(long)]
    pub block: Option<u64>,

    /// Skip USD pricing
    #[arg(long)]
    pub no_price: bool,

    /// RPC URL (default: the configured endpoint for --chain)
    #[arg(long, env = "ETH_RPC_URL", hide_env_values = true)]
    pub rpc_url: Option<String>,

    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
    pub format: OutputFormat,
}

/// Arguments for `uniswap lp-positions`
#[derive(Args, Debug)]
pub struct LpPositionsArgs {
    /// Owner address (or address-book label)
    #[arg(long)]
    pub owner: String,

    /// Include closed positions (no liquidity and nothing to collect)
    #[arg(long)]
    pub all: bool,

    /// Which positions to list: v3 (enumerable on-chain), v4 (found from the
    /// PositionManager's Transfer logs) or both
    #[arg(long, value_enum, default_value = "all")]
    pub version: ListVersion,

    /// First block of the v4 Transfer log scan (default: the PositionManager's
    /// deployment block; required on chains where that is not known)
    #[arg(long)]
    pub from_block: Option<u64>,

    /// Where v4 token ids come from: PositionManager Transfer logs over RPC,
    /// or the block explorer's ERC-721 transfer list (needs an Etherscan key).
    /// Ownership is always confirmed on-chain
    #[arg(long, value_enum, default_value = "logs")]
    pub v4_source: V4Source,

    /// Cap on eth_getLogs requests for the v4 scan (explorer pages: at most
    /// 100); when reached, v4 is reported as incomplete
    #[arg(long, default_value_t = super::uniswap_lp_v4_scan::DEFAULT_MAX_REQUESTS)]
    pub max_log_requests: u64,

    /// Block number to value at (default: latest)
    #[arg(long)]
    pub block: Option<u64>,

    /// Skip USD pricing
    #[arg(long)]
    pub no_price: bool,

    /// RPC URL (default: the configured endpoint for --chain)
    #[arg(long, env = "ETH_RPC_URL", hide_env_values = true)]
    pub rpc_url: Option<String>,

    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
    pub format: OutputFormat,
}

/// A token in a position
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TokenInfo {
    /// Token address (zero address for the native currency in v4)
    pub address: String,
    pub symbol: String,
    pub decimals: u8,
    /// True for the chain's native currency (v4 only)
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub native: bool,
    /// What the price aggregator is asked for
    #[serde(skip)]
    pub price_key: String,
}

/// An amount of one token
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TokenAmount {
    pub symbol: String,
    /// Base units
    pub raw: String,
    /// Human units (raw / 10^decimals)
    pub amount: f64,
}

/// USD price of one token
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TokenPrice {
    pub symbol: String,
    pub usd: Option<f64>,
    /// Which price source(s) the figure came from
    pub source: Option<String>,
}

/// USD values (null when a needed price is missing)
#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct UsdValue {
    pub amounts: Option<f64>,
    pub fees: Option<f64>,
    /// v3 tokensOwed (see `tokens_owed`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owed: Option<f64>,
    pub total: Option<f64>,
}

/// v3 uncollected-fee cross-check
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FeeCrossCheck {
    /// tokensOwed + fees from pool fee growth (uniswap-v3-sdk
    /// `get_collectable_token_amounts`): what collect should pay
    pub fee_growth: Vec<TokenAmount>,
    /// From an eth_call of NonfungiblePositionManager.collect from the owner
    pub collect_eth_call: Option<Vec<TokenAmount>>,
    /// Whether the two agree exactly (null when the eth_call failed)
    pub agree: Option<bool>,
    /// Which collectable total the fees / tokens_owed split is based on
    pub authoritative: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// One valued LP position
#[derive(Debug, Clone, Serialize)]
pub struct LpPosition {
    /// uniswap-v2 (any v2-style pair), uniswap-v3 or uniswap-v4
    pub protocol: String,
    pub chain: String,
    pub block: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
    pub owner: String,
    /// NonfungiblePositionManager, v4 PositionManager, or the v2 pair
    pub contract: String,
    /// v3 pool address, v4 pool id, or the v2 pair
    pub pool: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<String>,
    pub tokens: Vec<TokenInfo>,
    /// Fee in hundredths of a basis point (3000 = 0.30%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_tier: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_pct: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_lower: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_upper: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tick_current: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub in_range: Option<bool>,
    /// Position liquidity (v3/v4) or LP token balance (v2)
    pub liquidity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lp_total_supply: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pool_share_pct: Option<f64>,
    /// Tokens the liquidity is worth now (excluding uncollected fees)
    pub amounts: Vec<TokenAmount>,
    /// Fees earned since the position's last liquidity change, from fee
    /// growth x liquidity (null for v2: fees are compounded into `amounts`)
    pub uncollected_fees: Option<Vec<TokenAmount>>,
    /// v3 only: tokens already credited to the position's tokensOwed and not
    /// yet collected. Principal from a liquidity decrease ends up here, and
    /// so do fees credited at the last liquidity change; the two cannot be
    /// told apart on-chain. Counted once, in `value_usd.owed`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens_owed: Option<Vec<TokenAmount>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_cross_check: Option<FeeCrossCheck>,
    pub prices: Vec<TokenPrice>,
    /// "latest" (current prices) or the block's UTC time (historical prices
    /// from DefiLlama when --block is given); null with --no-price
    pub prices_at: Option<String>,
    pub value_usd: UsdValue,
    /// How the amounts and fees were computed
    pub method: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// raw / 10^decimals as f64
pub fn to_human(raw: U256, decimals: u8) -> f64 {
    alloy::primitives::utils::format_units(raw, decimals)
        .ok()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(f64::NAN)
}

fn token_amount(raw: U256, token: &TokenInfo) -> TokenAmount {
    TokenAmount {
        symbol: token.symbol.clone(),
        raw: raw.to_string(),
        amount: to_human(raw, token.decimals),
    }
}

/// Owner's share of a v2 pair's reserves: reserve * balance / totalSupply,
/// computed in 512 bits so it cannot overflow.
pub fn v2_share(
    balance: U256,
    total_supply: U256,
    reserve0: U256,
    reserve1: U256,
) -> anyhow::Result<(U256, U256)> {
    if total_supply.is_zero() {
        anyhow::bail!("pair has zero total supply");
    }
    if balance > total_supply {
        anyhow::bail!("LP balance exceeds total supply");
    }
    let share = |reserve: U256| -> U256 {
        let v = U512::from(reserve) * U512::from(balance) / U512::from(total_supply);
        // v <= reserve, so it fits
        U256::from(v)
    };
    Ok((share(reserve0), share(reserve1)))
}

/// A position earns fees while tick_lower <= tick < tick_upper
pub fn in_range(tick: i32, tick_lower: i32, tick_upper: i32) -> bool {
    tick_lower <= tick && tick < tick_upper
}

/// Sum of amount * price. A missing price only matters for a non-zero amount.
pub fn usd_sum(amounts: &[TokenAmount], prices: &[Option<f64>]) -> Option<f64> {
    let mut total = 0.0;
    for (i, a) in amounts.iter().enumerate() {
        if a.amount == 0.0 {
            continue;
        }
        total += a.amount * prices.get(i).copied().flatten()?;
    }
    Some(total)
}

/// Fill `value_usd` from `prices`
pub fn apply_usd(pos: &mut LpPosition) {
    let prices: Vec<Option<f64>> = pos.prices.iter().map(|p| p.usd).collect();
    let amounts = usd_sum(&pos.amounts, &prices);
    let fees = match &pos.uncollected_fees {
        Some(f) => usd_sum(f, &prices),
        None => Some(0.0),
    };
    let owed = pos.tokens_owed.as_ref().map(|o| usd_sum(o, &prices));
    pos.value_usd = UsdValue {
        amounts,
        fees: if pos.uncollected_fees.is_some() {
            fees
        } else {
            None
        },
        owed: owed.flatten(),
        total: match (amounts, fees, owed) {
            (Some(a), Some(f), None) => Some(a + f),
            (Some(a), Some(f), Some(Some(o))) => Some(a + f + o),
            _ => None,
        },
    };
}

/// Decode an ERC20 symbol() return: string, or bytes32 for old tokens
fn decode_symbol(data: &[u8]) -> Option<String> {
    let s = crate::utils::format::sanitize_symbol(&decode_symbol_raw(data)?);
    (!s.trim().is_empty()).then(|| s.trim().to_string())
}

fn decode_symbol_raw(data: &[u8]) -> Option<String> {
    use alloy::sol_types::SolValue;
    if let Ok(s) = String::abi_decode(data) {
        let s = s.trim_matches(char::from(0)).trim().to_string();
        if !s.is_empty() {
            return Some(s);
        }
    }
    if data.len() == 32 {
        let s: String = data
            .iter()
            .take_while(|b| **b != 0)
            .map(|b| *b as char)
            .collect();
        if !s.is_empty() && s.chars().all(|c| c.is_ascii_graphic()) {
            return Some(s);
        }
    }
    None
}

fn short_addr(a: Address) -> String {
    let s = format!("{a:#x}");
    format!("{}..{}", &s[..6], &s[s.len() - 4..])
}

fn fee_pct(fee: u32) -> f64 {
    fee as f64 / 10_000.0
}

// ---------------------------------------------------------------------------
// Chain plumbing
// ---------------------------------------------------------------------------

struct Ctx {
    provider: DynProvider,
    chain: crate::config::Chain,
    block: u64,
    /// The block was given with --block (so prices should be historical)
    historical: bool,
    /// Contract reads at `block` (Multicall3 batches where it exists)
    reader: ChainReader,
}

impl Ctx {
    fn block_id(&self) -> BlockId {
        BlockId::number(self.block)
    }

    async fn block_timestamp(&self) -> anyhow::Result<u64> {
        let b = self
            .provider
            .get_block_by_number(self.block.into())
            .await
            .map_err(rpc_err)?
            .ok_or_else(|| anyhow::anyhow!("block {} not found", self.block))?;
        Ok(b.header.timestamp)
    }

    fn addresses(&self) -> anyhow::Result<&'static ChainAddresses> {
        CHAIN_TO_ADDRESSES_MAP
            .get(&self.chain.chain_id())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "No official Uniswap v3/v4 deployment known for chain {} ({})",
                    self.chain.name(),
                    self.chain.chain_id()
                )
            })
    }
}

async fn connect(rpc_arg: Option<&str>, chain: &str, block: Option<u64>) -> anyhow::Result<Ctx> {
    let (url, chain) = super::uniswap::resolve_rpc_target(rpc_arg, chain)?;
    let provider = ProviderBuilder::new()
        .connect_http(
            url.parse()
                .map_err(|_| anyhow::anyhow!("Invalid RPC URL for {chain}"))?,
        )
        .erased();
    let historical = block.is_some();
    let block = match block {
        Some(b) => b,
        None => provider.get_block_number().await.map_err(rpc_err)?,
    };
    // Whether Multicall3 has code at this block; before its deployment every
    // read is sent on its own
    let block_id = BlockId::number(block);
    let multicall = !provider
        .get_code_at(MULTICALL3)
        .block_id(block_id)
        .await
        .map_err(rpc_err)?
        .is_empty();
    Ok(Ctx {
        reader: ChainReader::new(provider.clone(), block_id, multicall),
        provider,
        chain,
        block,
        historical,
    })
}

/// Errors may echo the RPC URL (with a key in it); strip URLs
fn rpc_err(e: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!(crate::utils::url::redact_urls_in_text(&e.to_string()))
}

fn erc20_info(address: Address, symbol: String, decimals: u8) -> TokenInfo {
    TokenInfo {
        address: format!("{address:#x}"),
        symbol,
        decimals,
        native: false,
        price_key: format!("{address:#x}"),
    }
}

fn native_info(chain: crate::config::Chain) -> TokenInfo {
    TokenInfo {
        address: format!("{:#x}", Address::ZERO),
        symbol: chain.native_symbol().to_string(),
        decimals: 18,
        native: true,
        price_key: chain.native_symbol().to_string(),
    }
}

fn parse_token_id(s: &str) -> anyhow::Result<U256> {
    s.parse::<U256>()
        .map_err(|e| anyhow::anyhow!("Invalid token id '{s}': {e}"))
}

fn bigint_to_u256(v: impl ToString) -> anyhow::Result<U256> {
    let s = v.to_string();
    s.parse::<U256>()
        .map_err(|e| anyhow::anyhow!("amount '{s}' out of range: {e}"))
}

// ---------------------------------------------------------------------------
// Valuation
// ---------------------------------------------------------------------------

/// Token metadata read in one stage: decimals and symbol of each distinct
/// token (bytes32 symbols decoded; a failed symbol read gives the short
/// address, a failed decimals read fails the positions that need it)
async fn read_token_meta(
    r: &dyn Reader,
    tokens: &[Address],
    extra: Vec<Call>,
) -> (HashMap<Address, Result<(u8, String), String>>, Vec<Outcome>) {
    let mut calls = Vec::with_capacity(tokens.len() * 2 + extra.len());
    for t in tokens {
        calls.push(Call::new(*t, IErc20MetaLite::decimalsCall {}));
        calls.push(Call::new(*t, IErc20MetaLite::symbolCall {}));
    }
    let n_extra = extra.len();
    calls.extend(extra);
    let mut out = r.read(calls).await;
    let rest = out.split_off(out.len() - n_extra);
    let meta = tokens
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let d = match &out[2 * i] {
                Outcome::Ok(b) => decode_decimals(b, *t),
                _ => Err(format!("decimals() failed for {t:#x}")),
            };
            let s = match &out[2 * i + 1] {
                Outcome::Ok(b) => decode_symbol(b).unwrap_or_else(|| short_addr(*t)),
                _ => short_addr(*t),
            };
            (*t, d.map(|d| (d, s)))
        })
        .collect();
    (meta, rest)
}

/// decimals() return data as u8 (the first word)
fn decode_decimals(b: &[u8], token: Address) -> Result<u8, String> {
    let v = U256::try_from_be_slice(b.get(..32).unwrap_or(b))
        .ok_or_else(|| format!("bad decimals() for {token:#x}"))?;
    u8::try_from(v).map_err(|_| format!("bad decimals() for {token:#x}"))
}

fn token_meta(
    meta: &HashMap<Address, Result<(u8, String), String>>,
    t: Address,
) -> anyhow::Result<(u8, String)> {
    meta.get(&t)
        .cloned()
        .unwrap_or_else(|| Err(format!("no metadata read for {t:#x}")))
        .map_err(|e| anyhow::anyhow!(e))
}

/// Everything read for one v3 position
struct V3Reads {
    token_id: U256,
    npm: Address,
    owner: Address,
    raw_pos: INonfungiblePositionManagerLite::positionsReturn,
    pool_addr: Address,
    meta0: (u8, String),
    meta1: (u8, String),
    slot0: IUniswapV3PoolLite::slot0Return,
    pool_liq: u128,
    growth_global: (U256, U256),
    lower: IUniswapV3PoolLite::ticksReturn,
    upper: IUniswapV3PoolLite::ticksReturn,
    collect: Result<(U256, U256), String>,
}

/// Value v3 positions with batched reads at one block:
///
/// 1. `factory()`, and `ownerOf` + `positions` of every token id
/// 2. `getPool` of each distinct (token0, token1, fee), with `decimals` and
///    `symbol` of each distinct token
/// 3. `slot0`, `liquidity`, `feeGrowthGlobal0/1X128` of each distinct pool
///    and `ticks` of each distinct (pool, tick)
/// 4. `collect` from each owner, one eth_call per position
///
/// Results are in the order of `ids`; a position whose reads failed is an
/// `Err` with the reason, without affecting the others.
async fn value_v3_many(ctx: &Ctx, r: &dyn Reader, ids: &[U256]) -> Vec<anyhow::Result<LpPosition>> {
    use INonfungiblePositionManagerLite as Npm;
    if ids.is_empty() {
        return Vec::new();
    }
    let fail_all = |e: &anyhow::Error| ids.iter().map(|_| Err(anyhow::anyhow!("{e}"))).collect();
    let npm = match ctx.addresses() {
        Ok(a) => a.nonfungible_position_manager,
        Err(e) => return fail_all(&e),
    };

    // Stage 1
    let mut calls = vec![Call::new(npm, Npm::factoryCall {})];
    for id in ids {
        calls.push(Call::new(npm, Npm::ownerOfCall { tokenId: *id }));
        calls.push(Call::new(npm, Npm::positionsCall { tokenId: *id }));
    }
    let out = r.read(calls).await;
    let factory = out[0].decode::<Npm::factoryCall>("factory()");
    let mut out = out.into_iter().skip(1);
    let mut stage1: Vec<anyhow::Result<(Address, Npm::positionsReturn, Address)>> = Vec::new();
    for id in ids {
        let (o, p) = (out.next().expect("ownerOf"), out.next().expect("positions"));
        stage1.push((|| {
            if o.is_revert() {
                return Err(not_found(*id));
            }
            let owner = o.decode::<Npm::ownerOfCall>("ownerOf()").map_err(rpc_err)?;
            let pos = p
                .decode::<Npm::positionsCall>("positions()")
                .map_err(rpc_err)?;
            let f = factory.clone().map_err(rpc_err)?;
            Ok((owner, pos, f))
        })());
    }

    // Stage 2
    let ok1: Vec<&(Address, Npm::positionsReturn, Address)> =
        stage1.iter().filter_map(|s| s.as_ref().ok()).collect();
    let (pool_keys, _) = dedup(
        &ok1.iter()
            .map(|(_, p, f)| (*f, p.token0, p.token1, p.fee))
            .collect::<Vec<_>>(),
    );
    let (tokens, _) = dedup(
        &ok1.iter()
            .flat_map(|(_, p, _)| [p.token0, p.token1])
            .collect::<Vec<_>>(),
    );
    let get_pools = pool_keys
        .iter()
        .map(|(f, a, b, fee)| {
            Call::new(
                *f,
                IUniswapV3FactoryLite::getPoolCall {
                    tokenA: *a,
                    tokenB: *b,
                    fee: *fee,
                },
            )
        })
        .collect();
    let (meta, pool_out) = if ok1.is_empty() {
        (HashMap::new(), Vec::new())
    } else {
        read_token_meta(r, &tokens, get_pools).await
    };
    let pools: HashMap<_, Result<Address, String>> = pool_keys
        .iter()
        .zip(&pool_out)
        .map(|(k, o)| {
            (
                *k,
                o.decode::<IUniswapV3FactoryLite::getPoolCall>("getPool()"),
            )
        })
        .collect();

    // Stage 3
    let pool_of = |p: &Npm::positionsReturn, f: Address| -> Option<Address> {
        pools
            .get(&(f, p.token0, p.token1, p.fee))
            .and_then(|r| r.as_ref().ok())
            .copied()
            .filter(|a| *a != Address::ZERO)
    };
    let (pool_addrs, _) = dedup(
        &ok1.iter()
            .filter_map(|(_, p, f)| pool_of(p, *f))
            .collect::<Vec<_>>(),
    );
    let (pool_ticks, _) = dedup(
        &ok1.iter()
            .filter_map(|(_, p, f)| pool_of(p, *f).map(|a| (a, p.tickLower, p.tickUpper)))
            .flat_map(|(a, l, u)| [(a, l), (a, u)])
            .collect::<Vec<_>>(),
    );
    let mut calls = Vec::with_capacity(pool_addrs.len() * 4 + pool_ticks.len());
    for a in &pool_addrs {
        calls.push(Call::new(*a, IUniswapV3PoolLite::slot0Call {}));
        calls.push(Call::new(*a, IUniswapV3PoolLite::liquidityCall {}));
        calls.push(Call::new(
            *a,
            IUniswapV3PoolLite::feeGrowthGlobal0X128Call {},
        ));
        calls.push(Call::new(
            *a,
            IUniswapV3PoolLite::feeGrowthGlobal1X128Call {},
        ));
    }
    for (a, t) in &pool_ticks {
        calls.push(Call::new(*a, IUniswapV3PoolLite::ticksCall { tick: *t }));
    }
    let out = if calls.is_empty() {
        Vec::new()
    } else {
        r.read(calls).await
    };
    type PoolState = (IUniswapV3PoolLite::slot0Return, u128, (U256, U256));
    let pool_state: HashMap<Address, Result<PoolState, String>> = pool_addrs
        .iter()
        .enumerate()
        .map(|(i, a)| {
            let o = &out[4 * i..4 * i + 4];
            let s = (|| {
                Ok((
                    o[0].decode::<IUniswapV3PoolLite::slot0Call>("slot0()")?,
                    o[1].decode::<IUniswapV3PoolLite::liquidityCall>("liquidity()")?,
                    (
                        o[2].decode::<IUniswapV3PoolLite::feeGrowthGlobal0X128Call>(
                            "feeGrowthGlobal0X128()",
                        )?,
                        o[3].decode::<IUniswapV3PoolLite::feeGrowthGlobal1X128Call>(
                            "feeGrowthGlobal1X128()",
                        )?,
                    ),
                ))
            })();
            (*a, s)
        })
        .collect();
    let base = 4 * pool_addrs.len();
    let tick_state: HashMap<_, Result<IUniswapV3PoolLite::ticksReturn, String>> = pool_ticks
        .iter()
        .enumerate()
        .map(|(i, k)| {
            (
                *k,
                out[base + i].decode::<IUniswapV3PoolLite::ticksCall>("ticks()"),
            )
        })
        .collect();

    // Stage 4: collect from each owner (must be sent from the owner, so not
    // batched)
    let collects: Vec<Option<Result<(U256, U256), String>>> =
        futures::stream::iter(ids.iter().zip(&stage1))
            .map(|(id, s)| async move {
                let (owner, _, _) = s.as_ref().ok()?;
                let call = Call::new(
                    npm,
                    Npm::collectCall {
                        params: Npm::CollectParams {
                            tokenId: *id,
                            recipient: *owner,
                            amount0Max: u128::MAX,
                            amount1Max: u128::MAX,
                        },
                    },
                );
                Some(
                    r.call_from(*owner, call)
                        .await
                        .decode::<Npm::collectCall>("collect()")
                        .map(|c| (c.amount0, c.amount1)),
                )
            })
            .buffered(COLLECT_CONCURRENCY)
            .collect()
            .await;

    ids.iter()
        .zip(stage1)
        .zip(collects)
        .map(|((id, s1), collect)| {
            let (owner, raw_pos, factory) = s1?;
            let key = (factory, raw_pos.token0, raw_pos.token1, raw_pos.fee);
            let pool_addr = pools
                .get(&key)
                .cloned()
                .unwrap_or_else(|| Err("getPool() not read".into()))
                .map_err(rpc_err)?;
            if pool_addr == Address::ZERO {
                anyhow::bail!("v3 position {id}: factory has no pool for its tokens and fee");
            }
            let (slot0, pool_liq, growth_global) = pool_state
                .get(&pool_addr)
                .cloned()
                .unwrap_or_else(|| Err("pool state not read".into()))
                .map_err(rpc_err)?;
            let tick = |t| {
                tick_state
                    .get(&(pool_addr, t))
                    .cloned()
                    .unwrap_or_else(|| Err("ticks() not read".into()))
                    .map_err(rpc_err)
            };
            let (lower, upper) = (tick(raw_pos.tickLower)?, tick(raw_pos.tickUpper)?);
            let meta0 = token_meta(&meta, raw_pos.token0)?;
            let meta1 = token_meta(&meta, raw_pos.token1)?;
            assemble_v3(
                ctx,
                V3Reads {
                    token_id: *id,
                    npm,
                    owner,
                    raw_pos,
                    pool_addr,
                    meta0,
                    meta1,
                    slot0,
                    pool_liq,
                    growth_global,
                    lower,
                    upper,
                    collect: collect.unwrap_or_else(|| Err("collect() not sent".into())),
                },
            )
        })
        .collect()
}

async fn value_v3(ctx: &Ctx, token_id: U256) -> anyhow::Result<LpPosition> {
    value_v3_many(ctx, &ctx.reader, &[token_id])
        .await
        .pop()
        .expect("one result per id")
}

fn assemble_v3(ctx: &Ctx, rd: V3Reads) -> anyhow::Result<LpPosition> {
    use uniswap_v3_sdk::prelude::{get_tokens_owed, FeeAmount, Pool, Position};
    use uniswap_v4_sdk::prelude::sdk_core::prelude::Token;

    let V3Reads {
        token_id,
        npm,
        owner,
        raw_pos,
        pool_addr,
        meta0: (d0, s0),
        meta1: (d1, s1),
        slot0,
        pool_liq,
        growth_global: (g0, g1),
        lower,
        upper,
        collect,
    } = rd;
    let chain_id = ctx.chain.chain_id();
    let owed_raw = (
        U256::from(raw_pos.tokensOwed0),
        U256::from(raw_pos.tokensOwed1),
    );
    let (t0, t1) = (raw_pos.token0, raw_pos.token1);
    let fee: u32 = raw_pos.fee.to::<u32>();
    let (tl, tu) = (raw_pos.tickLower.as_i32(), raw_pos.tickUpper.as_i32());
    let tokens = vec![erc20_info(t0, s0, d0), erc20_info(t1, s1, d1)];

    // Amounts: uniswap-v3-sdk Position on a Pool built from these reads
    let pool = Pool::new(
        Token::new(chain_id, t0, d0, None, None, 0, 0),
        Token::new(chain_id, t1, d1, None, None, 0, 0),
        FeeAmount::from(fee),
        slot0.sqrtPriceX96,
        pool_liq,
    )
    .map_err(|e| anyhow::anyhow!("v3 position {token_id}: {e}"))?;
    let pos = Position::new(pool, raw_pos.liquidity, tl, tu);
    let a0 = bigint_to_u256(pos.amount0().map_err(rpc_err)?.quotient())?;
    let a1 = bigint_to_u256(pos.amount1().map_err(rpc_err)?.quotient())?;
    let tick = slot0.tick.as_i32();

    // Fees, two ways: pool fee growth (the same maths as the SDK's
    // get_collectable_token_amounts) and the collect eth_call
    let (in0, in1) = fee_growth_inside(
        tick,
        tl,
        tu,
        (g0, g1),
        (lower.feeGrowthOutside0X128, lower.feeGrowthOutside1X128),
        (upper.feeGrowthOutside0X128, upper.feeGrowthOutside1X128),
    );
    let (n0, n1) = get_tokens_owed(
        raw_pos.feeGrowthInside0LastX128,
        raw_pos.feeGrowthInside1LastX128,
        raw_pos.liquidity,
        in0,
        in1,
    );
    let growth = (owed_raw.0 + n0, owed_raw.1 + n1);
    let (collectable, cross) = v3_reconcile_fees(growth, collect, &tokens);
    let (fees_raw, owed) = split_collectable(collectable, owed_raw);

    let method = if cross.authoritative == "collect_eth_call" {
        "amounts: uniswap-v3-sdk Position on the pool read from the chain's factory (liquidity at the current sqrt price); collectable total: eth_call of NonfungiblePositionManager.collect(tokenId, owner, max, max) from the owner, cross-checked against tokensOwed + pool fee growth inside the range x liquidity; fees = collectable - tokensOwed, tokens_owed = tokensOwed, each counted once".to_string()
    } else {
        "amounts: uniswap-v3-sdk Position on the pool read from the chain's factory (liquidity at the current sqrt price); fees: pool fee growth inside the range since the last liquidity change x liquidity; tokens_owed = tokensOwed; the collect eth_call failed".to_string()
    };

    Ok(LpPosition {
        protocol: "uniswap-v3".into(),
        chain: ctx.chain.name().to_string(),
        block: ctx.block,
        token_id: Some(token_id.to_string()),
        owner: format!("{owner:#x}"),
        contract: format!("{npm:#x}"),
        pool: format!("{pool_addr:#x}"),
        hooks: None,
        amounts: vec![token_amount(a0, &tokens[0]), token_amount(a1, &tokens[1])],
        uncollected_fees: Some(vec![
            token_amount(fees_raw.0, &tokens[0]),
            token_amount(fees_raw.1, &tokens[1]),
        ]),
        tokens_owed: Some(vec![
            token_amount(owed.0, &tokens[0]),
            token_amount(owed.1, &tokens[1]),
        ]),
        fee_cross_check: Some(cross),
        prices: Vec::new(),
        prices_at: None,
        value_usd: UsdValue::default(),
        fee_tier: Some(fee),
        fee_pct: Some(fee_pct(fee)),
        tick_lower: Some(tl),
        tick_upper: Some(tu),
        tick_current: Some(tick),
        in_range: Some(in_range(tick, tl, tu)),
        liquidity: raw_pos.liquidity.to_string(),
        lp_total_supply: None,
        pool_share_pct: None,
        tokens,
        method,
        notes: Vec::new(),
    })
}

/// Ticks from a v4 PositionInfo word: tickLower at bits 8..32, tickUpper at
/// bits 32..56 (v4-periphery PositionInfoLibrary)
pub fn decode_v4_position_info(info: U256) -> (i32, i32) {
    let int24 = |shift: usize| -> i32 {
        let raw = ((info >> shift) & U256::from(0xFF_FFFFu32)).to::<u32>();
        ((raw << 8) as i32) >> 8
    };
    (int24(8), int24(32))
}

/// Fee growth inside [tick_lower, tick_upper), as Uniswap's Tick library
/// computes it (wrapping subtraction is intended)
pub fn fee_growth_inside(
    tick: i32,
    tick_lower: i32,
    tick_upper: i32,
    global: (U256, U256),
    lower_outside: (U256, U256),
    upper_outside: (U256, U256),
) -> (U256, U256) {
    let one = |g: U256, lo: U256, up: U256| -> U256 {
        let below = if tick >= tick_lower {
            lo
        } else {
            g.wrapping_sub(lo)
        };
        let above = if tick < tick_upper {
            up
        } else {
            g.wrapping_sub(up)
        };
        g.wrapping_sub(below).wrapping_sub(above)
    };
    (
        one(global.0, lower_outside.0, upper_outside.0),
        one(global.1, lower_outside.1, upper_outside.1),
    )
}

/// Split what `collect` would pay into fees earned since the last liquidity
/// change and the position's existing tokensOwed, so nothing is counted
/// twice: owed = min(tokensOwed, collectable), fees = collectable - owed.
pub fn split_collectable(
    collectable: (U256, U256),
    tokens_owed: (U256, U256),
) -> ((U256, U256), (U256, U256)) {
    let one = |c: U256, o: U256| -> (U256, U256) {
        let owed = o.min(c);
        (c - owed, owed)
    };
    let (f0, o0) = one(collectable.0, tokens_owed.0);
    let (f1, o1) = one(collectable.1, tokens_owed.1);
    ((f0, f1), (o0, o1))
}

/// Error for a token id with no position
pub fn not_found(token_id: U256) -> anyhow::Error {
    anyhow::anyhow!("not_found: position {token_id} does not exist (burned or never minted)")
}

/// Pick the authoritative v3 fee figure: the collect eth_call when it worked,
/// otherwise the fee-growth computation.
pub fn v3_reconcile_fees(
    growth: (U256, U256),
    collect: Result<(U256, U256), String>,
    tokens: &[TokenInfo],
) -> ((U256, U256), FeeCrossCheck) {
    let as_amounts =
        |v: (U256, U256)| vec![token_amount(v.0, &tokens[0]), token_amount(v.1, &tokens[1])];
    match collect {
        Ok(c) => (
            c,
            FeeCrossCheck {
                fee_growth: as_amounts(growth),
                collect_eth_call: Some(as_amounts(c)),
                agree: Some(c == growth),
                authoritative: "collect_eth_call".into(),
                error: None,
            },
        ),
        Err(e) => (
            growth,
            FeeCrossCheck {
                fee_growth: as_amounts(growth),
                collect_eth_call: None,
                agree: None,
                authoritative: "fee_growth".into(),
                error: Some(e),
            },
        ),
    }
}

/// v4 pool id: keccak256(abi.encode(poolKey))
pub fn v4_pool_id(key: &IV4PositionManagerLite::PoolKey) -> B256 {
    use alloy::sol_types::SolValue;
    alloy::primitives::keccak256(key.abi_encode())
}

/// PoolManager storage slots read for one v4 position, in this order:
/// slot0, liquidity, feeGrowthGlobal0/1, lower tick feeGrowthOutside0/1,
/// upper tick feeGrowthOutside0/1, position liquidity and
/// feeGrowthInside0/1LastX128 (the layout PoolManagerLens and StateView use)
pub fn v4_state_slots(
    pool_id: B256,
    tick_lower: I24,
    tick_upper: I24,
    position_key: B256,
) -> Vec<B256> {
    use alloy::sol_types::SolValue;
    let k = |v: Vec<u8>| U256::from_be_bytes(alloy::primitives::keccak256(v).0);
    let state = k((pool_id, U256::from(6u8)).abi_encode());
    let tick = |t: I24| k((t, state + U256::from(4u8)).abi_encode());
    let (lo, up) = (tick(tick_lower), tick(tick_upper));
    let pos = k((position_key, state + U256::from(6u8)).abi_encode());
    let one = U256::from(1u8);
    let two = U256::from(2u8);
    [
        state,
        state + U256::from(3u8),
        state + one,
        state + two,
        lo + one,
        lo + two,
        up + one,
        up + two,
        pos,
        pos + one,
        pos + two,
    ]
    .into_iter()
    .map(B256::from)
    .collect()
}

/// The values behind [`v4_state_slots`]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct V4State {
    pub sqrt_price_x96: alloy::primitives::U160,
    pub tick: i32,
    pub lp_fee: u32,
    pub liquidity: u128,
    pub growth_global: (U256, U256),
    pub lower_outside: (U256, U256),
    pub upper_outside: (U256, U256),
    pub position_liquidity: u128,
    pub position_last: (U256, U256),
}

/// Decode the words read at [`v4_state_slots`]
pub fn decode_v4_state(w: &[B256]) -> Option<V4State> {
    if w.len() != 11 {
        return None;
    }
    let u = |i: usize| U256::from_be_bytes(w[i].0);
    let low128 = |i: usize| u128::from_be_bytes(w[i].0[16..32].try_into().expect("16 bytes"));
    let int24 = |b: &[u8]| -> i32 {
        ((i32::from(b[0]) << 24) | (i32::from(b[1]) << 16) | (i32::from(b[2]) << 8)) >> 8
    };
    let uint24 =
        |b: &[u8]| -> u32 { (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]) };
    let s0 = &w[0].0;
    Some(V4State {
        sqrt_price_x96: alloy::primitives::U160::from_be_slice(&s0[12..32]),
        tick: int24(&s0[9..12]),
        lp_fee: uint24(&s0[3..6]),
        liquidity: low128(1),
        growth_global: (u(2), u(3)),
        lower_outside: (u(4), u(5)),
        upper_outside: (u(6), u(7)),
        position_liquidity: low128(8),
        position_last: (u(9), u(10)),
    })
}

async fn value_v4(ctx: &Ctx, token_id: U256) -> anyhow::Result<LpPosition> {
    value_v4_many(ctx, &ctx.reader, &[token_id], true)
        .await
        .pop()
        .expect("one result per id")
        .map(|p| p.expect("closed positions are kept with include_closed"))
}

/// What stage 1 read for one v4 position
struct V4Stage1 {
    token_id: U256,
    owner: Address,
    key: IV4PositionManagerLite::PoolKey,
    tl: i32,
    tu: i32,
    tl24: I24,
    tu24: I24,
    pool_id: B256,
    liquidity: u128,
}

/// The v4 PositionManager and PoolManager of the chain
fn v4_contracts(ctx: &Ctx) -> anyhow::Result<(Address, Address)> {
    let addrs = ctx.addresses()?;
    match (addrs.v4_position_manager, addrs.v4_pool_manager) {
        (Some(pm), Some(m)) => Ok((pm, m)),
        _ => anyhow::bail!("No Uniswap v4 deployment known for {}", ctx.chain.name()),
    }
}

/// Value v4 positions with batched reads at one block:
///
/// 1. PositionManager `ownerOf`, `getPoolAndPositionInfo` and
///    `getPositionLiquidity` of every id
/// 2. `decimals` and `symbol` of each distinct ERC-20 currency, and one
///    PoolManager `extsload(bytes32[])` per position of its pool, tick and
///    position slots
///
/// Results are in the order of `ids`. `Ok(None)` is a closed position (no
/// liquidity) skipped because `include_closed` is false: v4 credits fees to
/// the owner whenever liquidity changes, so a position with no liquidity has
/// nothing left to collect. A position whose reads failed is an `Err` with
/// the reason, without affecting the others.
async fn value_v4_many(
    ctx: &Ctx,
    r: &dyn Reader,
    ids: &[U256],
    include_closed: bool,
) -> Vec<anyhow::Result<Option<LpPosition>>> {
    use uniswap_v4_sdk::prelude::calculate_position_key;
    use IV4PositionManagerLite as Pm;
    if ids.is_empty() {
        return Vec::new();
    }
    let (pm, manager) = match v4_contracts(ctx) {
        Ok(c) => c,
        Err(e) => {
            let msg = e.to_string();
            return ids
                .iter()
                .map(|_| Err(anyhow::anyhow!(msg.clone())))
                .collect();
        }
    };

    // Stage 1
    let mut calls = Vec::with_capacity(ids.len() * 3);
    for id in ids {
        calls.push(Call::new(pm, Pm::ownerOfCall { tokenId: *id }));
        calls.push(Call::new(
            pm,
            Pm::getPoolAndPositionInfoCall { tokenId: *id },
        ));
        calls.push(Call::new(pm, Pm::getPositionLiquidityCall { tokenId: *id }));
    }
    let out = r.read(calls).await;
    let stage1: Vec<anyhow::Result<Option<V4Stage1>>> = ids
        .iter()
        .enumerate()
        .map(|(i, id)| {
            let o = &out[3 * i..3 * i + 3];
            if o[0].is_revert() {
                return Err(not_found(*id));
            }
            let owner = o[0]
                .decode::<Pm::ownerOfCall>("ownerOf()")
                .map_err(rpc_err)?;
            let info = o[1]
                .decode::<Pm::getPoolAndPositionInfoCall>("getPoolAndPositionInfo()")
                .map_err(rpc_err)?;
            let liquidity = o[2]
                .decode::<Pm::getPositionLiquidityCall>("getPositionLiquidity()")
                .map_err(rpc_err)?;
            if liquidity == 0 && !include_closed {
                return Ok(None);
            }
            let (tl, tu) = decode_v4_position_info(info.info);
            let key = info.poolKey;
            Ok(Some(V4Stage1 {
                token_id: *id,
                owner,
                pool_id: v4_pool_id(&key),
                key,
                tl,
                tu,
                tl24: I24::try_from(tl).map_err(|e| anyhow::anyhow!("tick {tl}: {e}"))?,
                tu24: I24::try_from(tu).map_err(|e| anyhow::anyhow!("tick {tu}: {e}"))?,
                liquidity,
            }))
        })
        .collect();

    // Stage 2: currency metadata and the PoolManager state together
    let open: Vec<&V4Stage1> = stage1
        .iter()
        .filter_map(|s| s.as_ref().ok().and_then(|s| s.as_ref()))
        .collect();
    let erc20s: Vec<Address> = open
        .iter()
        .flat_map(|s| [s.key.currency0, s.key.currency1])
        .filter(|a| *a != Address::ZERO)
        .collect();
    let (erc20s, _) = dedup(&erc20s);
    let loads = open
        .iter()
        .map(|s| {
            // PoolManager position: owner = PositionManager, salt = tokenId
            let pkey = calculate_position_key(pm, s.tl24, s.tu24, B256::from(s.token_id));
            Call::new(
                manager,
                IPoolManagerExtsloadLite::extsloadCall {
                    slots: v4_state_slots(s.pool_id, s.tl24, s.tu24, pkey),
                },
            )
        })
        .collect();
    let (meta, rest) = read_token_meta(r, &erc20s, loads).await;

    let mut rest = rest.into_iter();
    stage1
        .into_iter()
        .map(|s| match s {
            Ok(Some(s)) => {
                let load = rest.next().expect("one extsload per open position");
                assemble_v4(ctx, pm, &s, &meta, &load)
                    .map(Some)
                    .map_err(|e| anyhow::anyhow!("v4 position {}: {e}", s.token_id))
            }
            other => other.map(|_| None),
        })
        .collect()
}

/// Build one v4 position from its reads
fn assemble_v4(
    ctx: &Ctx,
    pm: Address,
    s: &V4Stage1,
    meta: &HashMap<Address, Result<(u8, String), String>>,
    load: &Outcome,
) -> anyhow::Result<LpPosition> {
    use uniswap_v4_sdk::prelude::sdk_core::prelude::{Currency, Ether, Token};
    use uniswap_v4_sdk::prelude::{Pool, Position};

    let chain_id = ctx.chain.chain_id();
    let (key, tl, tu, pool_id) = (&s.key, s.tl, s.tu, s.pool_id);
    let words = load
        .decode::<IPoolManagerExtsloadLite::extsloadCall>("extsload()")
        .map_err(rpc_err)?;
    let st = decode_v4_state(&words)
        .ok_or_else(|| anyhow::anyhow!("extsload() returned {} words", words.len()))?;

    // Currencies (decimals and symbols read here; the SDK's own token
    // metadata read cannot decode bytes32 symbols)
    let mut tokens = Vec::with_capacity(2);
    let mut currencies = Vec::with_capacity(2);
    for a in [key.currency0, key.currency1] {
        if a == Address::ZERO {
            tokens.push(native_info(ctx.chain));
            currencies.push(Currency::NativeCurrency(Ether::on_chain(chain_id)));
        } else {
            let (d, sym) = token_meta(meta, a)?;
            tokens.push(erc20_info(a, sym, d));
            currencies.push(Currency::Token(Token::new(
                chain_id, a, d, None, None, 0, 0,
            )));
        }
    }
    let sdk_pool_id = Pool::get_pool_id(
        &currencies[0],
        &currencies[1],
        key.fee,
        key.tickSpacing.as_i32(),
        key.hooks,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    if sdk_pool_id != pool_id {
        anyhow::bail!("pool id {pool_id:#x} differs from the SDK's {sdk_pool_id:#x}");
    }
    let mut c = currencies.into_iter();
    let pool = Pool::new(
        c.next().expect("two currencies"),
        c.next().expect("two currencies"),
        key.fee,
        key.tickSpacing.as_i32(),
        key.hooks,
        st.sqrt_price_x96,
        st.liquidity,
    )
    .map_err(|e| anyhow::anyhow!("{e}"))?;
    let pos = Position::new(pool, s.liquidity, tl, tu);

    let a0 = bigint_to_u256(pos.amount0().map_err(rpc_err)?.quotient())?;
    let a1 = bigint_to_u256(pos.amount1().map_err(rpc_err)?.quotient())?;

    // Uncollected fees: PoolManager fee growth inside the range minus the
    // position's last value, times its liquidity
    let (in0, in1) = fee_growth_inside(
        st.tick,
        tl,
        tu,
        st.growth_global,
        st.lower_outside,
        st.upper_outside,
    );
    let liq = st.position_liquidity;
    let (last0, last1) = st.position_last;
    let (f0, f1) = uniswap_v3_sdk::prelude::get_tokens_owed(last0, last1, liq, in0, in1);

    let mut notes = Vec::new();
    if liq != pos.liquidity {
        notes.push(format!(
            "PoolManager position liquidity {liq} differs from PositionManager liquidity {}; fees may be wrong",
            pos.liquidity
        ));
    }

    let declared_fee = key.fee.to::<u32>();
    let fee = if declared_fee == DYNAMIC_FEE_FLAG {
        notes.push("dynamic-fee pool: fee_tier is the current LP fee from slot0".into());
        st.lp_fee
    } else {
        declared_fee
    };

    let hooks = key.hooks;
    let mut method = "amounts: uniswap-v4-sdk Position on the pool read from the PositionManager and PoolManager (liquidity at the current sqrt price); fees: computed, not read: PoolManager fee growth inside the range minus the position's feeGrowthInsideLast (pool, tick and position slots read with PoolManager.extsload, as PoolManagerLens does), times liquidity. v4 pays accrued fees to the owner on every liquidity change, so these are the fees since the last one".to_string();
    if hooks != Address::ZERO {
        method.push_str("; hooked pool: hooks can take or add value outside the PoolManager fee growth, which this does not include");
    }

    let tick = st.tick;
    Ok(LpPosition {
        protocol: "uniswap-v4".into(),
        chain: ctx.chain.name().to_string(),
        block: ctx.block,
        token_id: Some(s.token_id.to_string()),
        owner: format!("{:#x}", s.owner),
        contract: format!("{pm:#x}"),
        pool: format!("{pool_id:#x}"),
        hooks: Some(format!("{hooks:#x}")),
        amounts: vec![token_amount(a0, &tokens[0]), token_amount(a1, &tokens[1])],
        uncollected_fees: Some(vec![
            token_amount(f0, &tokens[0]),
            token_amount(f1, &tokens[1]),
        ]),
        tokens_owed: None,
        fee_cross_check: None,
        prices: Vec::new(),
        prices_at: None,
        value_usd: UsdValue::default(),
        fee_tier: Some(fee),
        fee_pct: Some(fee_pct(fee)),
        tick_lower: Some(tl),
        tick_upper: Some(tu),
        tick_current: Some(tick),
        in_range: Some(in_range(tick, tl, tu)),
        liquidity: pos.liquidity.to_string(),
        lp_total_supply: None,
        pool_share_pct: None,
        tokens,
        method,
        notes,
    })
}

async fn value_v2(ctx: &Ctx, pair: Address, owner: Address) -> anyhow::Result<LpPosition> {
    let not_pair = |e: String| {
        rpc_err(format!(
            "{pair:#x} does not answer as a Uniswap v2-style pair: {e}"
        ))
    };
    use IUniswapV2PairLite as P;
    let r = &ctx.reader;
    let via_multicall = r.via_multicall();
    let out = r
        .read(vec![
            Call::new(pair, P::balanceOfCall { owner }),
            Call::new(pair, P::totalSupplyCall {}),
            Call::new(pair, P::getReservesCall {}),
            Call::new(pair, P::token0Call {}),
            Call::new(pair, P::token1Call {}),
        ])
        .await;
    let balance = out[0]
        .decode::<P::balanceOfCall>("balanceOf()")
        .map_err(not_pair)?;
    let total_supply = out[1]
        .decode::<P::totalSupplyCall>("totalSupply()")
        .map_err(not_pair)?;
    let reserves = out[2]
        .decode::<P::getReservesCall>("getReserves()")
        .map_err(not_pair)?;
    let t0 = out[3]
        .decode::<P::token0Call>("token0()")
        .map_err(not_pair)?;
    let t1 = out[4]
        .decode::<P::token1Call>("token1()")
        .map_err(not_pair)?;
    let r0 = U256::from(reserves.reserve0);
    let r1 = U256::from(reserves.reserve1);
    let (a0, a1) = v2_share(balance, total_supply, r0, r1)?;

    let (uniq, _) = dedup(&[t0, t1]);
    let (meta, rest) = read_token_meta(
        r,
        &uniq,
        vec![Call::new(pair, IErc20MetaLite::symbolCall {})],
    )
    .await;
    let pair_symbol = match &rest[0] {
        Outcome::Ok(b) => decode_symbol(b).unwrap_or_else(|| short_addr(pair)),
        _ => short_addr(pair),
    };
    let (d0, s0) = token_meta(&meta, t0)?;
    let (d1, s1) = token_meta(&meta, t1)?;
    let tokens = vec![erc20_info(t0, s0, d0), erc20_info(t1, s1, d1)];
    let share_pct = to_human(balance, 18) / to_human(total_supply, 18) * 100.0;

    Ok(LpPosition {
        protocol: "uniswap-v2".into(),
        chain: ctx.chain.name().to_string(),
        block: ctx.block,
        token_id: None,
        owner: format!("{owner:#x}"),
        contract: format!("{pair:#x}"),
        pool: format!("{pair:#x}"),
        hooks: None,
        amounts: vec![token_amount(a0, &tokens[0]), token_amount(a1, &tokens[1])],
        uncollected_fees: None,
        tokens_owed: None,
        fee_cross_check: None,
        prices: Vec::new(),
        prices_at: None,
        value_usd: UsdValue::default(),
        fee_tier: None,
        fee_pct: None,
        tick_lower: None,
        tick_upper: None,
        tick_current: None,
        in_range: None,
        liquidity: balance.to_string(),
        lp_total_supply: Some(total_supply.to_string()),
        pool_share_pct: Some(share_pct),
        tokens,
        method: format!(
            "v2-style pair ({pair_symbol}): {} of balanceOf(owner), totalSupply(), getReserves(), token0(), token1(); amounts = reserves x balance / totalSupply. Swap fees are compounded into the reserves, so there are no separate uncollected fees",
            if via_multicall {
                "one multicall (token metadata in a second)"
            } else {
                "individual calls (Multicall3 did not exist at this block)"
            }
        ),
        notes: vec![
            "Only LP tokens held directly by the owner are counted (not LP tokens staked in a farm or gauge)".into(),
            "If the factory's protocol fee is on, a small unminted protocol share is not deducted".into(),
        ],
    })
}

/// Fetch USD prices for every distinct token across `positions` and fill
/// `prices` and `value_usd`.
async fn price_positions(positions: &mut [LpPosition], ctx: &Ctx) {
    let chain = ctx.chain.name();
    let mut keys: Vec<String> = positions
        .iter()
        .flat_map(|p| p.tokens.iter().map(|t| t.price_key.clone()))
        .collect();
    keys.sort();
    keys.dedup();

    let (map, prices_at) = if ctx.historical {
        match ctx.block_timestamp().await {
            Ok(ts) => (
                historical_prices(&keys, chain, ts).await,
                crate::rpc::block_time::format_timestamp(ts),
            ),
            Err(e) => {
                eprintln!(
                    "Warning: no block time for historical prices ({e}); USD values left out"
                );
                (HashMap::new(), "unavailable".to_string())
            }
        }
    } else {
        (current_prices(&keys, chain).await, "latest".to_string())
    };

    for p in positions.iter_mut() {
        p.prices = p
            .tokens
            .iter()
            .map(|t| {
                let (usd, source) = map.get(&t.price_key).cloned().unwrap_or((None, None));
                TokenPrice {
                    symbol: t.symbol.clone(),
                    usd,
                    source,
                }
            })
            .collect();
        p.prices_at = Some(prices_at.clone());
        apply_usd(p);
    }
}

type PriceMap = HashMap<String, (Option<f64>, Option<String>)>;

/// Prices at a past time from DefiLlama, as `price --at` does
async fn historical_prices(keys: &[String], chain: &str, ts: u64) -> PriceMap {
    let llama_chain = crate::aggregator::chain_map::normalize_chain_for_source("llama", chain);
    let tokens: Vec<(String, dllma::coins::Token)> = keys
        .iter()
        .map(|k| {
            let t = if k.starts_with("0x") {
                dllma::coins::Token::new(llama_chain.clone(), k)
            } else {
                dllma::coins::Token::coingecko(crate::aggregator::price::symbol_to_coingecko_id(k))
            };
            (k.clone(), t)
        })
        .collect();
    let mut map = PriceMap::new();
    let client = match crate::aggregator::price::get_llama_client() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: DefiLlama client error ({e}); USD values left out");
            return map;
        }
    };
    let list: Vec<dllma::coins::Token> = tokens.iter().map(|(_, t)| t.clone()).collect();
    match client
        .coins()
        .historical_with_search_width(ts, &list, HISTORICAL_SEARCH_WIDTH)
        .await
    {
        Ok(resp) => {
            for (k, t) in &tokens {
                let entry = resp.coins.get(&t.format()).map(|c| {
                    let at = c.timestamp.unwrap_or(ts);
                    (
                        Some(c.price),
                        Some(format!(
                            "defillama at {} ({:+} s from the block)",
                            crate::rpc::block_time::format_timestamp(at),
                            at as i64 - ts as i64
                        )),
                    )
                });
                map.insert(k.clone(), entry.unwrap_or((None, None)));
            }
        }
        Err(e) => {
            eprintln!("Warning: DefiLlama historical prices failed ({e}); USD values left out")
        }
    }
    map
}

async fn current_prices(keys: &[String], chain: &str) -> PriceMap {
    let fetched = futures::future::join_all(keys.iter().map(|k| async move {
        let r = crate::aggregator::price::fetch_prices_all(k, chain).await;
        let agg = &r.aggregated;
        let price = if agg.median_usd > 0.0 && r.sources_succeeded > 0 {
            let src = format!(
                "median of {} source(s){}",
                r.sources_succeeded,
                agg.best_source
                    .as_deref()
                    .map(|b| format!("; closest to median: {b}"))
                    .unwrap_or_default()
            );
            (Some(agg.median_usd), Some(src))
        } else {
            (None, None)
        };
        (k.clone(), price)
    }))
    .await;
    fetched.into_iter().collect()
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

fn fmt_amt(a: &TokenAmount) -> String {
    format!("{} {}", trim_num(a.amount), a.symbol)
}

fn trim_num(v: f64) -> String {
    if v == 0.0 {
        "0".into()
    } else if v.abs() >= 1000.0 {
        format!("{v:.2}")
    } else {
        format!("{v:.6}")
    }
}

fn fmt_usd(v: Option<f64>) -> String {
    v.map(|v| format!("${v:.2}"))
        .unwrap_or_else(|| "n/a".into())
}

fn print_table(p: &LpPosition) {
    let pair = p
        .tokens
        .iter()
        .map(|t| t.symbol.as_str())
        .collect::<Vec<_>>()
        .join("/");
    let id = p
        .token_id
        .as_deref()
        .map(|i| format!(" #{i}"))
        .unwrap_or_default();
    println!(
        "{}{} {} on {} (block {})",
        p.protocol, id, pair, p.chain, p.block
    );
    println!("  owner        {}", p.owner);
    println!("  pool         {}", p.pool);
    if let Some(h) = p
        .hooks
        .as_deref()
        .filter(|h| *h != format!("{:#x}", Address::ZERO))
    {
        println!("  hooks        {h}");
    }
    if let (Some(f), Some(pct)) = (p.fee_tier, p.fee_pct) {
        println!("  fee tier     {f} ({pct}%)");
    }
    if let (Some(lo), Some(hi), Some(cur)) = (p.tick_lower, p.tick_upper, p.tick_current) {
        let state = if p.in_range == Some(true) {
            "in range"
        } else {
            "OUT OF RANGE"
        };
        println!("  ticks        [{lo}, {hi}) current {cur} ({state})");
    }
    println!("  liquidity    {}", p.liquidity);
    if let Some(s) = p.pool_share_pct {
        println!("  pool share   {s:.6}%");
    }
    let amounts: Vec<String> = p.amounts.iter().map(fmt_amt).collect();
    println!("  amounts      {}", amounts.join(" + "));
    match &p.uncollected_fees {
        Some(f) => {
            let f: Vec<String> = f.iter().map(fmt_amt).collect();
            println!("  fees         {}", f.join(" + "));
        }
        None => println!("  fees         compounded into amounts (v2)"),
    }
    if let Some(o) = p
        .tokens_owed
        .as_ref()
        .filter(|o| o.iter().any(|a| a.amount != 0.0))
    {
        let o: Vec<String> = o.iter().map(fmt_amt).collect();
        println!(
            "  owed         {} (tokensOwed: withdrawn principal and/or fees credited earlier)",
            o.join(" + ")
        );
    }
    if let Some(c) = &p.fee_cross_check {
        match (&c.collect_eth_call, c.agree) {
            (Some(_), Some(true)) => {
                println!("  cross-check  collect eth_call == tokensOwed + fee growth (exact)")
            }
            (Some(_), _) => {
                let g: Vec<String> = c.fee_growth.iter().map(fmt_amt).collect();
                println!(
                    "  cross-check  DISAGREE: fee growth gives {}; reporting collect eth_call",
                    g.join(" + ")
                );
            }
            (None, _) => println!(
                "  cross-check  collect eth_call failed ({}); fees from fee growth",
                c.error.as_deref().unwrap_or("unknown error")
            ),
        }
    }
    if let Some(at) = p.prices_at.as_deref().filter(|a| *a != "latest") {
        println!("  prices at    {at}");
    }
    for pr in &p.prices {
        if let Some(usd) = pr.usd {
            println!(
                "  price        {} ${} ({})",
                pr.symbol,
                trim_num(usd),
                pr.source.as_deref().unwrap_or("")
            );
        }
    }
    if !p.prices.is_empty() {
        if p.uncollected_fees.is_some() {
            let owed = p
                .value_usd
                .owed
                .filter(|o| *o != 0.0)
                .map(|o| format!(" + owed {}", fmt_usd(Some(o))))
                .unwrap_or_default();
            println!(
                "  value        {} (amounts {} + fees {}{owed})",
                fmt_usd(p.value_usd.total),
                fmt_usd(p.value_usd.amounts),
                fmt_usd(p.value_usd.fees)
            );
        } else {
            println!("  value        {}", fmt_usd(p.value_usd.total));
        }
    }
    println!("  method       {}", p.method);
    for n in &p.notes {
        println!("  note         {n}");
    }
}

/// A position `lp-positions` could not value
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct FailedPosition {
    pub token_id: String,
    pub error: String,
}

/// Total of `value_usd.total` across positions; None when any position has
/// no total, nothing was priced, any position failed, or the list is known
/// to be incomplete
pub fn list_total(
    positions: &[LpPosition],
    failed: &[FailedPosition],
    incomplete: bool,
) -> Option<f64> {
    if incomplete || !failed.is_empty() || positions.iter().any(|p| p.prices.is_empty()) {
        return None;
    }
    positions.iter().map(|p| p.value_usd.total).sum()
}

/// What `lp-positions` reports besides the positions
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListInfo {
    /// Position NFTs found (v3 owned + v4 still owned), open or closed
    pub nft_count: usize,
    /// v3 position NFTs the owner holds (null when v3 was not listed)
    pub v3_nft_count: Option<usize>,
    /// How v4 ids were found (null when v4 was not listed)
    pub v4_discovery: Option<V4Discovery>,
    /// Why part of the list may be missing (empty when complete)
    pub incomplete: Vec<String>,
    /// Other remarks (a chain without v4, ...)
    pub notes: Vec<String>,
}

impl ListInfo {
    fn complete(&self, failed: &[FailedPosition]) -> bool {
        failed.is_empty() && self.incomplete.is_empty()
    }
}

fn print_discovery(d: &V4Discovery) {
    println!(
        "v4 discovery: {} ({}), blocks {}-{} ({} from {}), {} requests, {} candidates, {} still owned ({} transferred out, {} burned, {} unknown){}",
        d.source,
        d.position_manager,
        d.from_block,
        d.to_block,
        if d.complete { "complete" } else { "INCOMPLETE" },
        d.from_block_source,
        d.requests,
        d.candidates,
        d.still_owned,
        d.transferred_out,
        d.burned,
        d.owner_unknown,
        d.error
            .as_deref()
            .map(|e| format!("; stopped: {e}"))
            .unwrap_or_default()
    );
}

/// `list` is Some for `lp-positions`, None for one position
fn emit(
    positions: &[LpPosition],
    failed: &[FailedPosition],
    list: Option<&ListInfo>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    let incomplete = list.is_some_and(|l| !l.incomplete.is_empty());
    match format {
        OutputFormat::Table => {
            if positions.is_empty() {
                println!("No positions found");
            }
            for (i, p) in positions.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                print_table(p);
            }
            if let Some(l) = list {
                for f in failed {
                    println!("\nFAILED position {}: {}", f.token_id, f.error);
                }
                println!();
                if let Some(d) = &l.v4_discovery {
                    print_discovery(d);
                }
                for n in &l.notes {
                    println!("note: {n}");
                }
                for r in &l.incomplete {
                    println!("INCOMPLETE: {r}");
                }
                match list_total(positions, failed, incomplete) {
                    Some(sum) if !positions.is_empty() => println!(
                        "Total value: ${sum:.2} across {} positions",
                        positions.len()
                    ),
                    _ if !failed.is_empty() => println!(
                        "INCOMPLETE: {} of {} positions could not be valued; no total",
                        failed.len(),
                        positions.len() + failed.len()
                    ),
                    _ if incomplete => println!("No total: the list may be incomplete"),
                    _ => {}
                }
            }
        }
        OutputFormat::Ndjson => {
            for p in positions {
                println!("{}", serde_json::to_string(p)?);
            }
            if let Some(l) = list {
                println!(
                    "{}",
                    serde_json::json!({
                        "summary": true,
                        "complete": l.complete(failed),
                        "incomplete": l.incomplete,
                        "failed": failed,
                        "nft_count": l.nft_count,
                        "v3_nft_count": l.v3_nft_count,
                        "v4_discovery": l.v4_discovery,
                        "notes": l.notes,
                        "total_value_usd": list_total(positions, failed, incomplete),
                    })
                );
            }
        }
        OutputFormat::Json => match list {
            None => println!("{}", serde_json::to_string_pretty(&positions[0])?),
            Some(l) => {
                let out = serde_json::json!({
                    "complete": l.complete(failed),
                    "incomplete": l.incomplete,
                    "nft_count": l.nft_count,
                    "v3_nft_count": l.v3_nft_count,
                    "v4_discovery": l.v4_discovery,
                    "notes": l.notes,
                    "count": positions.len(),
                    "failed": failed,
                    "total_value_usd": list_total(positions, failed, incomplete),
                    "positions": positions,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            }
        },
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

fn parse_addr(s: &str, what: &str) -> anyhow::Result<Address> {
    let resolved = crate::utils::address::resolve_label(s);
    resolved
        .parse::<Address>()
        .map_err(|e| anyhow::anyhow!("Invalid {what} '{s}': {e}"))
}

/// `uniswap position`
pub async fn handle_position(args: &PositionArgs, chain: &str, quiet: bool) -> anyhow::Result<()> {
    let ctx = connect(args.rpc_url.as_deref(), chain, args.block).await?;
    if !quiet {
        eprintln!(
            "Valuing position on {} at block {}...",
            ctx.chain, ctx.block
        );
    }
    let mut pos = if let Some(pair) = &args.v2 {
        let pair = parse_addr(pair, "pair")?;
        let owner = parse_addr(args.owner.as_deref().unwrap_or_default(), "owner")?;
        value_v2(&ctx, pair, owner).await?
    } else {
        let id = parse_token_id(args.token_id.as_deref().unwrap_or_default())?;
        match args.version {
            NftVersion::V3 => value_v3(&ctx, id).await?,
            NftVersion::V4 => value_v4(&ctx, id).await?,
        }
    };
    let mut list = [pos.clone()];
    if !args.no_price {
        price_positions(&mut list, &ctx).await;
        pos = list[0].clone();
    }
    emit(std::slice::from_ref(&pos), &[], None, args.format)
}

/// Candidate v4 ids from the explorer's ERC-721 transfer list of the
/// PositionManager for `owner` (transfers to the owner), with the number of
/// pages read and why the walk stopped early, if it did
async fn explorer_v4_candidates(
    ctx: &Ctx,
    pm: Address,
    owner: Address,
    from: u64,
    max_pages: u64,
    etherscan_key: Option<String>,
) -> anyhow::Result<(Vec<U256>, u64, Option<String>)> {
    use foundry_block_explorers::account::{Sort, TokenQueryOption, TxListParams};
    let key = etherscan_key
        .ok_or_else(|| anyhow::anyhow!("--v4-source explorer needs an Etherscan API key"))?;
    let client = crate::etherscan::Client::new(ctx.chain, Some(key))?;
    client.ensure_api_supported()?;
    let client = &client;
    let fetched = super::account::fetch_all_pages(
        from,
        ctx.block,
        true,
        max_pages,
        super::account::PAGE_DELAY,
        move |page, start, end| async move {
            Ok(client
                .get_erc721_token_transfer_events(
                    TokenQueryOption::ByAddressAndContract(owner, pm),
                    Some(TxListParams {
                        start_block: start,
                        end_block: end,
                        page,
                        offset: super::account::ALL_PAGE_SIZE,
                        sort: Sort::Asc,
                    }),
                )
                .await?)
        },
        |t| {
            t.block_number
                .as_number()
                .map(|n| n.to::<u64>())
                .unwrap_or(0)
        },
    )
    .await?;
    let mut ids = std::collections::BTreeSet::new();
    for t in &fetched.rows {
        if t.contract_address == pm && t.to == Some(owner) {
            ids.insert(parse_token_id(&t.token_id)?);
        }
    }
    let stopped = if fetched.complete {
        None
    } else {
        Some(
            fetched
                .stopped
                .clone()
                .unwrap_or_else(|| "explorer walk stopped early".into()),
        )
    };
    Ok((ids.into_iter().collect(), fetched.requests, stopped))
}

/// Find the v4 ids `owner` holds at the valuation block: candidates from
/// Transfer logs (or the explorer), confirmed with `ownerOf`. Returns the
/// discovery report, the owned ids and the candidates whose owner could not
/// be read.
async fn discover_v4(
    ctx: &Ctx,
    owner: Address,
    args: &LpPositionsArgs,
    pm: Address,
    etherscan_key: Option<String>,
) -> anyhow::Result<(V4Discovery, Vec<U256>, Vec<FailedPosition>)> {
    use super::uniswap_lp_v4_scan::{
        classify_owner, deployment_block, scan_transfers, Ownership, ProviderLogs, ScanOpts,
    };
    let chain_id = ctx.chain.chain_id();
    let (from, from_source) = match (args.from_block, deployment_block(chain_id, pm)) {
        (Some(b), _) => (b, "--from-block"),
        (None, Some(b)) => (b, "deployment"),
        (None, None) => anyhow::bail!(
            "no verified deployment block for the v4 PositionManager on {}; pass --from-block (the block it was deployed in, or any earlier block)",
            ctx.chain.name()
        ),
    };
    if from > ctx.block {
        anyhow::bail!(
            "--from-block {from} is after the valuation block {}",
            ctx.block
        );
    }

    let (source, ids, requests, scanned_to, error) = match args.v4_source {
        V4Source::Logs => {
            let src = ProviderLogs {
                provider: ctx.provider.clone(),
                pm,
                owner,
            };
            let opts = ScanOpts {
                max_requests: args.max_log_requests,
                retry_delay: std::time::Duration::from_millis(500),
                segments: LOG_SCAN_SEGMENTS,
            };
            let o = scan_transfers(&src, pm, owner, from, ctx.block, &opts).await;
            ("logs", o.ids, o.requests, o.scanned_to, o.error)
        }
        V4Source::Explorer => {
            let (ids, requests, stopped) = explorer_v4_candidates(
                ctx,
                pm,
                owner,
                from,
                args.max_log_requests.min(100),
                etherscan_key,
            )
            .await?;
            let to = stopped.is_none().then_some(ctx.block);
            ("explorer", ids, requests, to, stopped)
        }
    };

    // Confirm ownership at the valuation block
    use IV4PositionManagerLite as Pm;
    let out = ctx
        .reader
        .read(
            ids.iter()
                .map(|id| Call::new(pm, Pm::ownerOfCall { tokenId: *id }))
                .collect(),
        )
        .await;
    let (mut owned, mut unknown) = (Vec::new(), Vec::new());
    let (mut moved, mut burned) = (0, 0);
    for (id, o) in ids.iter().zip(&out) {
        match classify_owner(o, owner) {
            Ownership::Owned => owned.push(*id),
            Ownership::TransferredOut => moved += 1,
            Ownership::Burned => burned += 1,
            Ownership::Unknown(e) => unknown.push(FailedPosition {
                token_id: id.to_string(),
                error: format!("v4 ownership unknown: {e}"),
            }),
        }
    }
    let d = V4Discovery {
        source: source.into(),
        position_manager: format!("{pm:#x}"),
        from_block: from,
        from_block_source: from_source.into(),
        to_block: ctx.block,
        scanned_to,
        complete: error.is_none(),
        requests,
        candidates: ids.len(),
        still_owned: owned.len(),
        transferred_out: moved,
        burned,
        owner_unknown: unknown.len(),
        error,
    };
    Ok((d, owned, unknown))
}

/// `uniswap lp-positions`
pub async fn handle_lp_positions(
    args: &LpPositionsArgs,
    chain: &str,
    etherscan_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    use uniswap_v3_sdk::extensions::get_all_positions_by_owner;

    let ctx = connect(args.rpc_url.as_deref(), chain, args.block).await?;
    let owner = parse_addr(&args.owner, "owner")?;
    let addrs = ctx.addresses()?;
    let mut info = ListInfo::default();
    let mut positions = Vec::new();
    let mut failed = Vec::new();
    let closed_hint = if args.all {
        ""
    } else {
        " open (use --all for closed ones)"
    };
    fn record(
        positions: &mut Vec<LpPosition>,
        failed: &mut Vec<FailedPosition>,
        id: &U256,
        r: anyhow::Result<Option<LpPosition>>,
    ) {
        match r {
            Ok(Some(p)) => positions.push(p),
            Ok(None) => {}
            Err(e) => {
                eprintln!("Warning: position {id}: {e}");
                failed.push(FailedPosition {
                    token_id: id.to_string(),
                    error: e.to_string(),
                });
            }
        }
    }

    // v3: the NonfungiblePositionManager is enumerable
    if args.version != ListVersion::V4 {
        let npm = addrs.nonfungible_position_manager;
        if !quiet {
            eprintln!(
                "Listing v3 positions of {owner:#x} on {} at block {}...",
                ctx.chain, ctx.block
            );
        }
        match get_all_positions_by_owner(npm, owner, ctx.provider.clone(), Some(ctx.block_id()))
            .await
        {
            Ok(states) => {
                info.v3_nft_count = Some(states.len());
                let ids: Vec<U256> = states
                    .iter()
                    .filter(|s| {
                        args.all
                            || s.position.liquidity > 0
                            || s.position.tokensOwed0 > 0
                            || s.position.tokensOwed1 > 0
                    })
                    .map(|s| s.tokenId)
                    .collect();
                if !quiet {
                    eprintln!(
                        "{} v3 position NFT(s); valuing {}{closed_hint}",
                        states.len(),
                        ids.len()
                    );
                }
                let results = value_v3_many(&ctx, &ctx.reader, &ids).await;
                for (id, r) in ids.iter().zip(results) {
                    record(&mut positions, &mut failed, id, r.map(Some));
                }
            }
            Err(e) => {
                let e = crate::utils::url::redact_urls_in_text(&e.to_string());
                eprintln!("Warning: v3 positions not listed: {e}");
                info.incomplete
                    .push(format!("v3 positions not listed: {e}"));
            }
        }
    }

    // v4: the PositionManager is not enumerable; find ids from Transfer logs
    if args.version != ListVersion::V3 {
        match addrs.v4_position_manager {
            None if args.version == ListVersion::All => info.notes.push(format!(
                "no Uniswap v4 PositionManager on {}",
                ctx.chain.name()
            )),
            None => anyhow::bail!("No Uniswap v4 deployment known for {}", ctx.chain.name()),
            Some(pm) => {
                if !quiet {
                    eprintln!(
                        "Finding v4 positions of {owner:#x} from PositionManager Transfer {}...",
                        match args.v4_source {
                            V4Source::Logs => "logs",
                            V4Source::Explorer => "events on the explorer",
                        }
                    );
                }
                match discover_v4(&ctx, owner, args, pm, etherscan_key).await {
                    Ok((d, ids, unknown)) => {
                        if !quiet {
                            eprintln!(
                                "v4: {} candidate(s) from {} request(s), {} still owned; valuing{closed_hint}",
                                d.candidates, d.requests, d.still_owned
                            );
                        }
                        if let Some(e) = &d.error {
                            eprintln!("Warning: v4 discovery incomplete: {e}");
                            info.incomplete.push(format!(
                                "v4 discovery covered blocks {}-{} of {}-{}: {e}",
                                d.from_block,
                                d.scanned_to.map_or("none".to_string(), |b| b.to_string()),
                                d.from_block,
                                d.to_block
                            ));
                        }
                        failed.extend(unknown);
                        let results = value_v4_many(&ctx, &ctx.reader, &ids, args.all).await;
                        for (id, r) in ids.iter().zip(results) {
                            record(&mut positions, &mut failed, id, r);
                        }
                        info.v4_discovery = Some(d);
                    }
                    Err(e) => {
                        let e = crate::utils::url::redact_urls_in_text(&e.to_string());
                        eprintln!("Warning: v4 positions not listed: {e}");
                        info.incomplete
                            .push(format!("v4 positions not listed: {e}"));
                    }
                }
            }
        }
    }

    info.nft_count =
        info.v3_nft_count.unwrap_or(0) + info.v4_discovery.as_ref().map_or(0, |d| d.still_owned);
    if !args.no_price {
        price_positions(&mut positions, &ctx).await;
    }
    emit(&positions, &failed, Some(&info), args.format)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tok(sym: &str, dec: u8) -> TokenInfo {
        TokenInfo {
            address: format!("{:#x}", Address::ZERO),
            symbol: sym.into(),
            decimals: dec,
            native: false,
            price_key: sym.into(),
        }
    }

    fn u(v: u128) -> U256 {
        U256::from(v)
    }

    #[test]
    fn v2_share_is_proportional() {
        let (a0, a1) = v2_share(u(25), u(100), u(1_000), u(4_000)).unwrap();
        assert_eq!((a0, a1), (u(250), u(1_000)));
    }

    #[test]
    fn v2_share_rounds_down() {
        let (a0, _) = v2_share(u(1), u(3), u(10), u(0)).unwrap();
        assert_eq!(a0, u(3));
    }

    #[test]
    fn v2_share_full_balance_returns_reserves() {
        let r = U256::from(u128::MAX >> 16); // uint112-sized reserve
        let ts = U256::MAX / u(2);
        let (a0, a1) = v2_share(ts, ts, r, r).unwrap();
        assert_eq!((a0, a1), (r, r));
    }

    #[test]
    fn v2_share_no_overflow_with_huge_values() {
        // reserve * balance overflows 256 bits; 512-bit maths must not
        let r = U256::from(1u8) << 111;
        let b = U256::from(1u8) << 200;
        let ts = U256::from(1u8) << 201;
        let (a0, _) = v2_share(b, ts, r, r).unwrap();
        assert_eq!(a0, U256::from(1u8) << 110);
    }

    #[test]
    fn v2_share_rejects_bad_supply() {
        assert!(v2_share(u(1), u(0), u(1), u(1)).is_err());
        assert!(v2_share(u(2), u(1), u(1), u(1)).is_err());
    }

    #[test]
    fn in_range_is_half_open() {
        assert!(in_range(0, 0, 10));
        assert!(in_range(9, 0, 10));
        assert!(!in_range(10, 0, 10));
        assert!(!in_range(-1, 0, 10));
    }

    #[test]
    fn to_human_scales_by_decimals() {
        assert_eq!(to_human(u(1_500_000), 6), 1.5);
        assert_eq!(to_human(u(10u128.pow(18)), 18), 1.0);
    }

    #[test]
    fn usd_sum_ignores_missing_price_for_zero_amount() {
        let t = [tok("A", 0), tok("B", 0)];
        let amounts = vec![token_amount(u(2), &t[0]), token_amount(u(0), &t[1])];
        assert_eq!(usd_sum(&amounts, &[Some(3.0), None]), Some(6.0));
        let amounts = vec![token_amount(u(2), &t[0]), token_amount(u(1), &t[1])];
        assert_eq!(usd_sum(&amounts, &[Some(3.0), None]), None);
    }

    #[test]
    fn v3_reconcile_prefers_collect_and_flags_disagreement() {
        let t = [tok("A", 0), tok("B", 0)];
        let (fees, c) = v3_reconcile_fees((u(5), u(7)), Ok((u(5), u(8))), &t);
        assert_eq!(fees, (u(5), u(8)));
        assert_eq!(c.agree, Some(false));
        assert_eq!(c.authoritative, "collect_eth_call");
        assert_eq!(c.fee_growth[1].raw, "7");

        let (_, c) = v3_reconcile_fees((u(5), u(7)), Ok((u(5), u(7))), &t);
        assert_eq!(c.agree, Some(true));
    }

    #[test]
    fn v3_reconcile_falls_back_to_fee_growth() {
        let t = [tok("A", 0), tok("B", 0)];
        let (fees, c) = v3_reconcile_fees((u(5), u(7)), Err("reverted".into()), &t);
        assert_eq!(fees, (u(5), u(7)));
        assert_eq!(c.agree, None);
        assert_eq!(c.authoritative, "fee_growth");
        assert_eq!(c.error.as_deref(), Some("reverted"));
    }

    fn sample(fees: Option<Vec<TokenAmount>>) -> LpPosition {
        let t = vec![tok("USDC", 6), tok("WETH", 18)];
        LpPosition {
            protocol: "uniswap-v3".into(),
            chain: "ethereum".into(),
            block: 1,
            token_id: Some("1".into()),
            owner: "0x0".into(),
            contract: "0x0".into(),
            pool: "0x0".into(),
            hooks: None,
            amounts: vec![
                token_amount(u(2_000_000), &t[0]),
                token_amount(u(10u128.pow(18)), &t[1]),
            ],
            uncollected_fees: fees,
            tokens_owed: None,
            fee_cross_check: None,
            prices: vec![
                TokenPrice {
                    symbol: "USDC".into(),
                    usd: Some(1.0),
                    source: Some("x".into()),
                },
                TokenPrice {
                    symbol: "WETH".into(),
                    usd: Some(3000.0),
                    source: Some("y".into()),
                },
            ],
            prices_at: Some("latest".into()),
            value_usd: UsdValue::default(),
            fee_tier: Some(500),
            fee_pct: Some(fee_pct(500)),
            tick_lower: Some(-10),
            tick_upper: Some(10),
            tick_current: Some(0),
            in_range: Some(true),
            liquidity: "1".into(),
            lp_total_supply: None,
            pool_share_pct: None,
            tokens: t,
            method: "m".into(),
            notes: vec![],
        }
    }

    #[test]
    fn apply_usd_adds_amounts_and_fees() {
        let t = [tok("USDC", 6), tok("WETH", 18)];
        let mut p = sample(Some(vec![
            token_amount(u(1_000_000), &t[0]),
            token_amount(u(10u128.pow(17)), &t[1]),
        ]));
        apply_usd(&mut p);
        assert_eq!(p.value_usd.amounts, Some(3002.0));
        assert_eq!(p.value_usd.fees, Some(301.0));
        assert_eq!(p.value_usd.total, Some(3303.0));
    }

    #[test]
    fn split_collectable_separates_owed_from_new_fees() {
        // collect pays 1_005 of token0: 1_000 already owed (e.g. principal
        // from an uncollected decrease) and 5 of new fees
        let (fees, owed) = split_collectable((u(1_005), u(7)), (u(1_000), u(0)));
        assert_eq!(fees, (u(5), u(7)));
        assert_eq!(owed, (u(1_000), u(0)));
        // owed never exceeds what is collectable
        let (fees, owed) = split_collectable((u(3), u(0)), (u(10), u(0)));
        assert_eq!((fees.0, owed.0), (u(0), u(3)));
    }

    #[test]
    fn apply_usd_counts_owed_once() {
        let t = [tok("USDC", 6), tok("WETH", 18)];
        let mut p = sample(Some(vec![
            token_amount(u(1_000_000), &t[0]),
            token_amount(u(0), &t[1]),
        ]));
        p.tokens_owed = Some(vec![
            token_amount(u(500_000_000), &t[0]),
            token_amount(u(0), &t[1]),
        ]);
        apply_usd(&mut p);
        assert_eq!(p.value_usd.amounts, Some(3002.0));
        assert_eq!(p.value_usd.fees, Some(1.0));
        assert_eq!(p.value_usd.owed, Some(500.0));
        assert_eq!(p.value_usd.total, Some(3503.0));
    }

    #[test]
    fn fee_growth_inside_matches_tick_library() {
        let g = (u(1_000), u(2_000));
        let lo = (u(100), u(200));
        let up = (u(300), u(400));
        // in range: global - below(lower outside) - above(upper outside)
        assert_eq!(fee_growth_inside(5, 0, 10, g, lo, up), (u(600), u(1_400)));
        // below the range: lower_outside - upper_outside
        assert_eq!(
            fee_growth_inside(-5, 0, 10, g, lo, up).0,
            U256::ZERO.wrapping_sub(u(200))
        );
        // above the range: upper_outside - lower_outside
        assert_eq!(fee_growth_inside(20, 0, 10, g, lo, up), (u(200), u(200)));
    }

    #[test]
    fn decode_v4_position_info_ticks() {
        // tickLower = -887220 at bits 8..32, tickUpper = 887220 at 32..56
        let lo = (-887_220i32 as u32) & 0xFF_FFFF;
        let hi = 887_220u32;
        let info = (U256::from(hi) << 32) | (U256::from(lo) << 8) | U256::from(1u8);
        assert_eq!(decode_v4_position_info(info), (-887_220, 887_220));
    }

    #[test]
    fn list_total_is_null_when_any_position_failed() {
        let mut p = sample(Some(vec![]));
        apply_usd(&mut p);
        assert_eq!(
            list_total(std::slice::from_ref(&p), &[], false),
            Some(3002.0)
        );
        let f = FailedPosition {
            token_id: "1".into(),
            error: "x".into(),
        };
        assert_eq!(list_total(&[p], &[f], false), None);
    }

    #[test]
    fn not_found_message() {
        let e = not_found(u(42)).to_string();
        assert!(e.starts_with("not_found: position 42 does not exist"));
    }

    #[test]
    fn decode_symbol_sanitizes() {
        use alloy::sol_types::SolValue;
        let enc = "US\x1b[31mDC\u{202e}X".to_string().abi_encode();
        assert_eq!(decode_symbol(&enc).as_deref(), Some("US[31mDCX"));
        let enc = "\u{2066}\u{202a}".to_string().abi_encode();
        assert_eq!(decode_symbol(&enc), None);
    }

    #[test]
    fn apply_usd_v2_has_no_fee_value() {
        let mut p = sample(None);
        apply_usd(&mut p);
        assert_eq!(p.value_usd.fees, None);
        assert_eq!(p.value_usd.total, Some(3002.0));
    }

    #[test]
    fn json_shape() {
        let mut p = sample(Some(vec![]));
        apply_usd(&mut p);
        let v = serde_json::to_value(&p).unwrap();
        assert_eq!(v["protocol"], "uniswap-v3");
        assert_eq!(v["fee_pct"], 0.05);
        assert_eq!(v["in_range"], true);
        assert_eq!(v["amounts"][0]["symbol"], "USDC");
        assert_eq!(v["amounts"][0]["raw"], "2000000");
        assert_eq!(v["amounts"][0]["amount"], 2.0);
        assert!(v.get("hooks").is_none());
        assert!(v["tokens"][0].get("price_key").is_none());
        assert!(v["tokens"][0].get("native").is_none());
        assert_eq!(v["value_usd"]["total"], 3002.0);
    }

    #[test]
    fn decode_symbol_handles_string_and_bytes32() {
        use alloy::sol_types::SolValue;
        let enc = "UNI".to_string().abi_encode();
        assert_eq!(decode_symbol(&enc).as_deref(), Some("UNI"));
        let mut b = [0u8; 32];
        b[..3].copy_from_slice(b"MKR");
        assert_eq!(decode_symbol(&b).as_deref(), Some("MKR"));
        assert_eq!(decode_symbol(&[]), None);
    }

    #[test]
    fn official_addresses_present() {
        let m = &CHAIN_TO_ADDRESSES_MAP[&1];
        assert_eq!(
            m.nonfungible_position_manager,
            "0xC36442b4a4522E871399CD717aBDD847Ab11FE88"
                .parse::<Address>()
                .unwrap()
        );
        assert_eq!(
            m.v4_position_manager,
            Some(
                "0xbD216513d74C8cf14cf4747E6AaA6420FF64ee9e"
                    .parse::<Address>()
                    .unwrap()
            )
        );
        assert_eq!(
            m.v4_pool_manager,
            Some(
                "0x000000000004444c5dc75cB358380D2e3dE08A90"
                    .parse::<Address>()
                    .unwrap()
            )
        );
    }

    // --- batched reads -----------------------------------------------------

    use super::super::uniswap_lp_reads::mock::MockReader;
    use alloy::primitives::{address, aliases::U24, U160};
    use alloy::sol_types::SolCall;

    const NPM: Address = address!("0xc36442b4a4522e871399cd717abdd847ab11fe88");
    const FACTORY: Address = address!("0x1f98431c8ad98523631ae4a59f267346ea31f984");
    const A: Address = address!("0x00000000000000000000000000000000000000aa");
    const B: Address = address!("0x00000000000000000000000000000000000000bb");
    const C: Address = address!("0x00000000000000000000000000000000000000cc");
    const POOL_AB: Address = address!("0x000000000000000000000000000000000000a0b0");
    const POOL_AC: Address = address!("0x000000000000000000000000000000000000a0c0");
    const OWNER: Address = address!("0x0000000000000000000000000000000000000001");

    fn test_ctx() -> Ctx {
        let provider = ProviderBuilder::new()
            .connect_http("http://127.0.0.1:9".parse().unwrap())
            .erased();
        Ctx {
            reader: ChainReader::new(provider.clone(), BlockId::number(1), true),
            provider,
            chain: crate::config::Chain::Ethereum,
            block: 1,
            historical: true,
        }
    }

    fn ok<C: SolCall>(ret: &C::Return) -> Outcome {
        Outcome::Ok(C::abi_encode_returns(ret).into())
    }

    fn word(v: U256) -> Outcome {
        Outcome::Ok(v.to_be_bytes_vec().into())
    }

    fn str_ret(s: &str) -> Outcome {
        use alloy::sol_types::SolValue;
        Outcome::Ok(s.to_string().abi_encode().into())
    }

    fn i24(v: i32) -> I24 {
        I24::try_from(v).unwrap()
    }

    fn set_position(m: &mut MockReader, id: u64, t0: Address, t1: Address, owed: u128) {
        use INonfungiblePositionManagerLite as Npm;
        let id = U256::from(id);
        m.set(
            NPM,
            Npm::ownerOfCall { tokenId: id },
            ok::<Npm::ownerOfCall>(&OWNER),
        );
        let pos = Npm::positionsReturn {
            nonce: Default::default(),
            operator: Address::ZERO,
            token0: t0,
            token1: t1,
            fee: U24::from(3000u32),
            tickLower: i24(-60),
            tickUpper: i24(60),
            liquidity: 1_000_000_000_000_000_000,
            feeGrowthInside0LastX128: U256::ZERO,
            feeGrowthInside1LastX128: U256::ZERO,
            tokensOwed0: owed,
            tokensOwed1: 0,
        };
        m.set(
            NPM,
            Npm::positionsCall { tokenId: id },
            ok::<Npm::positionsCall>(&pos),
        );
        let collect = Npm::collectCall {
            params: Npm::CollectParams {
                tokenId: id,
                recipient: OWNER,
                amount0Max: u128::MAX,
                amount1Max: u128::MAX,
            },
        };
        let ret = Npm::collectReturn {
            amount0: U256::from(owed),
            amount1: U256::ZERO,
        };
        m.set_from(OWNER, NPM, collect, ok::<Npm::collectCall>(&ret));
    }

    fn set_pool(m: &mut MockReader, pool: Address, t0: Address, t1: Address) {
        use IUniswapV3PoolLite as P;
        m.set(
            FACTORY,
            IUniswapV3FactoryLite::getPoolCall {
                tokenA: t0,
                tokenB: t1,
                fee: U24::from(3000u32),
            },
            ok::<IUniswapV3FactoryLite::getPoolCall>(&pool),
        );
        let slot0 = P::slot0Return {
            sqrtPriceX96: U160::from(1u8) << 96,
            tick: i24(0),
            observationIndex: 0,
            observationCardinality: 1,
            observationCardinalityNext: 1,
            feeProtocol: 0,
            unlocked: true,
        };
        m.set(pool, P::slot0Call {}, ok::<P::slot0Call>(&slot0));
        m.set(pool, P::liquidityCall {}, word(U256::from(10u128.pow(18))));
        m.set(pool, P::feeGrowthGlobal0X128Call {}, word(U256::ZERO));
        m.set(pool, P::feeGrowthGlobal1X128Call {}, word(U256::ZERO));
        let tick = P::ticksReturn {
            liquidityGross: 0,
            liquidityNet: 0,
            feeGrowthOutside0X128: U256::ZERO,
            feeGrowthOutside1X128: U256::ZERO,
            tickCumulativeOutside: Default::default(),
            secondsPerLiquidityOutsideX128: Default::default(),
            secondsOutside: 0,
            initialized: true,
        };
        for t in [-60, 60] {
            m.set(
                pool,
                P::ticksCall { tick: i24(t) },
                ok::<P::ticksCall>(&tick),
            );
        }
    }

    fn set_token(m: &mut MockReader, t: Address, sym: &str) {
        m.set(t, IErc20MetaLite::decimalsCall {}, word(U256::from(18u8)));
        m.set(t, IErc20MetaLite::symbolCall {}, str_ret(sym));
    }

    fn base_mock() -> MockReader {
        let mut m = MockReader::default();
        m.set(
            NPM,
            INonfungiblePositionManagerLite::factoryCall {},
            ok::<INonfungiblePositionManagerLite::factoryCall>(&FACTORY),
        );
        set_token(&mut m, A, "AAA");
        set_token(&mut m, B, "BBB");
        set_token(&mut m, C, "CCC");
        set_pool(&mut m, POOL_AB, A, B);
        set_pool(&mut m, POOL_AC, A, C);
        m
    }

    #[tokio::test]
    async fn v3_batches_in_three_stages_and_dedups() {
        let mut m = base_mock();
        // Positions 1 and 2 share pool A/B; 3 is in A/C; 4 is burned
        set_position(&mut m, 1, A, B, 5);
        set_position(&mut m, 2, A, B, 0);
        set_position(&mut m, 3, A, C, 7);
        let ids: Vec<U256> = [1u64, 2, 3, 4].into_iter().map(U256::from).collect();
        let res = value_v3_many(&test_ctx(), &m, &ids).await;

        // 1: factory + (ownerOf, positions) x 4
        // 2: decimals + symbol of A, B, C, and getPool of A/B and A/C
        // 3: 4 reads per pool (2 pools) + ticks -60/60 per pool
        assert_eq!(m.batch_sizes(), vec![9, 3 * 2 + 2, 2 * 4 + 2 * 2]);
        // collect from the owner, once per existing position
        assert_eq!(*m.single_calls.lock().unwrap(), 3);

        assert_eq!(res.len(), 4);
        let p1 = res[0].as_ref().unwrap();
        assert_eq!(p1.pool, format!("{POOL_AB:#x}"));
        assert_eq!(p1.tokens[0].symbol, "AAA");
        assert_eq!(p1.tokens_owed.as_ref().unwrap()[0].raw, "5");
        assert_eq!(p1.fee_cross_check.as_ref().unwrap().agree, Some(true));
        assert_eq!(
            p1.amounts[0].raw, p1.amounts[1].raw,
            "price 1, symmetric range"
        );
        assert_eq!(res[2].as_ref().unwrap().pool, format!("{POOL_AC:#x}"));
        assert_eq!(res[2].as_ref().unwrap().tokens[1].symbol, "CCC");
        let e = res[3].as_ref().unwrap_err().to_string();
        assert!(e.starts_with("not_found:"), "{e}");
    }

    #[tokio::test]
    async fn v3_failed_read_fails_only_that_position() {
        let mut m = base_mock();
        set_position(&mut m, 1, A, B, 0);
        set_position(&mut m, 3, A, C, 0);
        m.set(
            POOL_AC,
            IUniswapV3PoolLite::slot0Call {},
            Outcome::Failed("boom".into()),
        );
        let ids = [U256::from(1u8), U256::from(3u8)];
        let res = value_v3_many(&test_ctx(), &m, &ids).await;
        assert!(res[0].is_ok());
        let e = res[1].as_ref().unwrap_err().to_string();
        assert!(e.contains("slot0() failed: boom"), "{e}");
    }

    #[tokio::test]
    async fn v3_collect_failure_falls_back_to_fee_growth() {
        let mut m = base_mock();
        set_position(&mut m, 1, A, B, 9);
        m.from_answers.clear();
        let res = value_v3_many(&test_ctx(), &m, &[U256::from(1u8)]).await;
        let p = res[0].as_ref().unwrap();
        let cc = p.fee_cross_check.as_ref().unwrap();
        assert_eq!(cc.authoritative, "fee_growth");
        assert_eq!(cc.agree, None);
        assert_eq!(p.tokens_owed.as_ref().unwrap()[0].raw, "9");
    }

    #[tokio::test]
    async fn v3_bytes32_symbol_and_missing_symbol() {
        let mut m = base_mock();
        let mut mkr = [0u8; 32];
        mkr[..3].copy_from_slice(b"MKR");
        m.set(
            A,
            IErc20MetaLite::symbolCall {},
            Outcome::Ok(mkr.to_vec().into()),
        );
        m.set(
            B,
            IErc20MetaLite::symbolCall {},
            Outcome::Reverted(String::new()),
        );
        set_position(&mut m, 1, A, B, 0);
        let res = value_v3_many(&test_ctx(), &m, &[U256::from(1u8)]).await;
        let p = res[0].as_ref().unwrap();
        assert_eq!(p.tokens[0].symbol, "MKR");
        assert_eq!(p.tokens[1].symbol, short_addr(B));
    }

    #[tokio::test]
    async fn v3_no_ids_sends_nothing() {
        let m = base_mock();
        assert!(value_v3_many(&test_ctx(), &m, &[]).await.is_empty());
        assert!(m.batch_sizes().is_empty());
    }

    #[test]
    fn v4_pool_id_matches_sdk() {
        use uniswap_v4_sdk::prelude::sdk_core::prelude::{Currency, Ether, Token};
        use uniswap_v4_sdk::prelude::Pool;
        let usdc = address!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let key = IV4PositionManagerLite::PoolKey {
            currency0: Address::ZERO,
            currency1: usdc,
            fee: U24::from(500u32),
            tickSpacing: i24(10),
            hooks: Address::ZERO,
        };
        let sdk = Pool::get_pool_id(
            &Currency::NativeCurrency(Ether::on_chain(1)),
            &Currency::Token(Token::new(1, usdc, 6, None, None, 0, 0)),
            key.fee,
            10,
            Address::ZERO,
        )
        .unwrap();
        assert_eq!(v4_pool_id(&key), sdk);
    }

    /// Answer the v4 reads of one position in pool A/B (fee 3000, spacing
    /// 60, no hooks, price 1) with range -60..60 and `liq` liquidity
    fn set_v4_position(m: &mut MockReader, id: u64, liq: u128) {
        use uniswap_v4_sdk::prelude::calculate_position_key;
        use IV4PositionManagerLite as Pm;
        let addrs = CHAIN_TO_ADDRESSES_MAP.get(&1).unwrap();
        let (pm, manager) = (
            addrs.v4_position_manager.unwrap(),
            addrs.v4_pool_manager.unwrap(),
        );
        let id = U256::from(id);
        m.set(
            pm,
            Pm::ownerOfCall { tokenId: id },
            ok::<Pm::ownerOfCall>(&OWNER),
        );
        let key = Pm::PoolKey {
            currency0: A,
            currency1: B,
            fee: U24::from(3000u32),
            tickSpacing: i24(60),
            hooks: Address::ZERO,
        };
        let lo = (-60i32 as u32) & 0xFF_FFFF;
        let info = (U256::from(60u32) << 32) | (U256::from(lo) << 8);
        m.set(
            pm,
            Pm::getPoolAndPositionInfoCall { tokenId: id },
            ok::<Pm::getPoolAndPositionInfoCall>(&Pm::getPoolAndPositionInfoReturn {
                poolKey: key.clone(),
                info,
            }),
        );
        m.set(
            pm,
            Pm::getPositionLiquidityCall { tokenId: id },
            ok::<Pm::getPositionLiquidityCall>(&liq),
        );
        let pkey = calculate_position_key(pm, i24(-60), i24(60), B256::from(id));
        let mut s0 = [0u8; 32];
        s0[3..6].copy_from_slice(&[0x00, 0x0b, 0xb8]); // lp fee 3000
        s0[12..32].copy_from_slice(&U160::from(1u8).wrapping_shl(96).to_be_bytes::<20>());
        let mut words = vec![B256::from(s0), B256::from(U256::from(10_000u32))];
        words.extend((0..6).map(|_| B256::ZERO));
        words.push(B256::from(U256::from(liq)));
        words.extend((0..2).map(|_| B256::ZERO));
        m.set(
            manager,
            IPoolManagerExtsloadLite::extsloadCall {
                slots: v4_state_slots(v4_pool_id(&key), i24(-60), i24(60), pkey),
            },
            ok::<IPoolManagerExtsloadLite::extsloadCall>(&words),
        );
    }

    #[tokio::test]
    async fn v4_batches_in_two_stages_and_skips_closed() {
        let mut m = MockReader::default();
        set_token(&mut m, A, "AAA");
        set_token(&mut m, B, "BBB");
        set_v4_position(&mut m, 1, 5_000);
        set_v4_position(&mut m, 2, 0); // closed
        set_v4_position(&mut m, 4, 7_000);
        // 3 was never answered: ownerOf reverts (burned)
        let ids: Vec<U256> = [1u64, 2, 3, 4].into_iter().map(U256::from).collect();
        let res = value_v4_many(&test_ctx(), &m, &ids, false).await;

        // 1: (ownerOf, getPoolAndPositionInfo, getPositionLiquidity) x 4
        // 2: decimals + symbol of A and B, one extsload per open position
        assert_eq!(m.batch_sizes(), vec![12, 2 * 2 + 2]);
        assert_eq!(res.len(), 4);
        let p1 = res[0].as_ref().unwrap().as_ref().unwrap();
        assert_eq!(p1.protocol, "uniswap-v4");
        assert_eq!(p1.liquidity, "5000");
        assert_eq!(p1.tokens[1].symbol, "BBB");
        assert_eq!(p1.in_range, Some(true));
        assert!(p1.notes.is_empty(), "{:?}", p1.notes);
        assert!(p1.method.contains("computed, not read"));
        assert!(res[1].as_ref().unwrap().is_none(), "closed is skipped");
        let e = res[2].as_ref().unwrap_err().to_string();
        assert!(e.starts_with("not_found:"), "{e}");
        assert_eq!(res[3].as_ref().unwrap().as_ref().unwrap().liquidity, "7000");

        // --all keeps the closed one
        let res = value_v4_many(&test_ctx(), &m, &ids[..2], true).await;
        let p2 = res[1].as_ref().unwrap().as_ref().unwrap();
        assert_eq!(p2.liquidity, "0");
        assert_eq!(p2.amounts[0].raw, "0");
    }

    #[tokio::test]
    async fn v4_failed_load_fails_only_that_position() {
        let mut m = MockReader::default();
        set_token(&mut m, A, "AAA");
        set_token(&mut m, B, "BBB");
        set_v4_position(&mut m, 1, 5_000);
        set_v4_position(&mut m, 2, 6_000);
        // Break position 2's PoolManager read
        let mut m2 = MockReader::default();
        set_v4_position(&mut m2, 2, 6_000);
        for call in m2.answers.keys() {
            if call.target
                == CHAIN_TO_ADDRESSES_MAP
                    .get(&1)
                    .unwrap()
                    .v4_pool_manager
                    .unwrap()
            {
                m.answers
                    .insert(call.clone(), Outcome::Failed("boom".into()));
            }
        }
        let ids = [U256::from(1u8), U256::from(2u8)];
        let res = value_v4_many(&test_ctx(), &m, &ids, false).await;
        assert!(res[0].as_ref().unwrap().is_some());
        let e = res[1].as_ref().unwrap_err().to_string();
        assert!(e.contains("extsload() failed: boom"), "{e}");
    }

    #[test]
    fn list_total_is_null_when_list_incomplete() {
        assert_eq!(list_total(&[], &[], true), None);
        assert_eq!(list_total(&[], &[], false), Some(0.0));
    }

    #[test]
    fn v4_state_decodes_packed_words() {
        let mut s0 = [0u8; 32];
        s0[3..6].copy_from_slice(&[0x00, 0x0b, 0xb8]); // lp fee 3000
        s0[9..12].copy_from_slice(&[0xff, 0xff, 0xc4]); // tick -60
        s0[31] = 7; // sqrt price
        let mut liq = [0u8; 32];
        liq[0] = 0xff; // upper half is not liquidity
        liq[31] = 42;
        let mut words = vec![B256::from(s0), B256::from(liq)];
        words.extend((2u8..8).map(|i| B256::from(U256::from(i))));
        words.push(B256::from(liq));
        words.extend((9u8..11).map(|i| B256::from(U256::from(i))));
        let st = decode_v4_state(&words).unwrap();
        assert_eq!(st.lp_fee, 3000);
        assert_eq!(st.tick, -60);
        assert_eq!(st.sqrt_price_x96, U160::from(7u8));
        assert_eq!(st.liquidity, 42);
        assert_eq!(st.growth_global, (U256::from(2u8), U256::from(3u8)));
        assert_eq!(st.lower_outside, (U256::from(4u8), U256::from(5u8)));
        assert_eq!(st.upper_outside, (U256::from(6u8), U256::from(7u8)));
        assert_eq!(st.position_liquidity, 42);
        assert_eq!(st.position_last, (U256::from(9u8), U256::from(10u8)));
        assert!(decode_v4_state(&words[..10]).is_none());
    }

    #[test]
    fn v4_slots_follow_pool_layout() {
        let id = B256::repeat_byte(1);
        let s = v4_state_slots(id, i24(-60), i24(60), B256::repeat_byte(2));
        assert_eq!(s.len(), 11);
        let state = U256::from_be_bytes(s[0].0);
        assert_eq!(U256::from_be_bytes(s[1].0), state + U256::from(3u8));
        assert_eq!(U256::from_be_bytes(s[2].0), state + U256::from(1u8));
        assert_eq!(U256::from_be_bytes(s[3].0), state + U256::from(2u8));
        assert_eq!(
            U256::from_be_bytes(s[5].0),
            U256::from_be_bytes(s[4].0) + U256::from(1u8)
        );
        assert_ne!(s[4], s[6], "lower and upper tick slots differ");
    }
}
