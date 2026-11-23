//! On-chain Uniswap LP position valuation
//!
//! - v3: `uniswap-v3-sdk` extensions (`get_position`, `get_all_positions_by_owner`,
//!   `get_collectable_token_amounts`), cross-checked with an `eth_call` of
//!   `NonfungiblePositionManager.collect` from the NFT owner.
//! - v4: `uniswap-v4-sdk` extensions (`get_position`, `PoolManagerLens`) with
//!   uncollected fees from the PoolManager's fee growth.
//! - v2 and v2 forks: one multicall on the pair; fees are compounded into the
//!   reserves.
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
use uniswap_v4_sdk::prelude::sdk_core::addresses::{ChainAddresses, CHAIN_TO_ADDRESSES_MAP};
use uniswap_v4_sdk::prelude::sdk_core::prelude::{BaseCurrency, BaseCurrencyCore, FractionBase};

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
        function positions(uint256 tokenId) external view returns (uint96 nonce, address operator, address token0, address token1, uint24 fee, int24 tickLower, int24 tickUpper, uint128 liquidity, uint256 feeGrowthInside0LastX128, uint256 feeGrowthInside1LastX128, uint128 tokensOwed0, uint128 tokensOwed1);
        function collect(CollectParams calldata params) external payable returns (uint256 amount0, uint256 amount1);
    }

    #[sol(rpc)]
    interface IV4PositionManagerLite {
        function ownerOf(uint256 tokenId) external view returns (address);
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

/// Concurrent position valuations when listing an owner's positions
const LIST_CONCURRENCY: usize = 4;

/// Uniswap version of an NFT position
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum NftVersion {
    /// Uniswap v3 NonfungiblePositionManager
    #[default]
    V3,
    /// Uniswap v4 PositionManager
    V4,
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
}

impl Ctx {
    fn block_id(&self) -> BlockId {
        BlockId::number(self.block)
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
    let block = match block {
        Some(b) => b,
        None => provider.get_block_number().await.map_err(rpc_err)?,
    };
    Ok(Ctx {
        provider,
        chain,
        block,
    })
}

/// Errors may echo the RPC URL (with a key in it); strip URLs
fn rpc_err(e: impl std::fmt::Display) -> anyhow::Error {
    anyhow::anyhow!(crate::utils::url::redact_urls_in_text(&e.to_string()))
}

async fn raw_call(ctx: &Ctx, to: Address, selector: [u8; 4]) -> Option<alloy::primitives::Bytes> {
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(to)
        .input(alloy::primitives::Bytes::copy_from_slice(&selector).into());
    ctx.provider.call(tx).block(ctx.block_id()).await.ok()
}

async fn erc20_symbol(ctx: &Ctx, token: Address) -> String {
    match raw_call(ctx, token, [0x95, 0xd8, 0x9b, 0x41]).await {
        Some(b) => decode_symbol(&b).unwrap_or_else(|| short_addr(token)),
        None => short_addr(token),
    }
}

async fn erc20_decimals(ctx: &Ctx, token: Address) -> anyhow::Result<u8> {
    let b = raw_call(ctx, token, [0x31, 0x3c, 0xe5, 0x67])
        .await
        .ok_or_else(|| anyhow::anyhow!("decimals() failed for {token:#x}"))?;
    let v = U256::try_from_be_slice(b.get(..32).unwrap_or(&b))
        .ok_or_else(|| anyhow::anyhow!("bad decimals() for {token:#x}"))?;
    u8::try_from(v).map_err(|_| anyhow::anyhow!("bad decimals() for {token:#x}"))
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

async fn value_v3(ctx: &Ctx, token_id: U256) -> anyhow::Result<LpPosition> {
    use uniswap_v3_sdk::extensions::{get_collectable_token_amounts, get_position};

    let addrs = ctx.addresses()?;
    let npm = addrs.nonfungible_position_manager;
    let chain_id = ctx.chain.chain_id();
    let bid = Some(ctx.block_id());

    let npm_c = INonfungiblePositionManagerLite::new(npm, ctx.provider.clone());
    let owner = match npm_c.ownerOf(token_id).block(ctx.block_id()).call().await {
        Ok(o) => o,
        Err(e) if is_revert(&e) => return Err(not_found(token_id)),
        Err(e) => return Err(rpc_err(e)),
    };
    let raw_pos = npm_c
        .positions(token_id)
        .block(ctx.block_id())
        .call()
        .await
        .map_err(rpc_err)?;
    let owed_raw = (
        U256::from(raw_pos.tokensOwed0),
        U256::from(raw_pos.tokensOwed1),
    );

    let pos = get_position(chain_id, npm, token_id, ctx.provider.clone(), bid)
        .await
        .map_err(|e| rpc_err(format!("v3 position {token_id}: {e}")))?;

    let t0 = pos.pool.token0.address();
    let t1 = pos.pool.token1.address();
    let (s0, s1) = tokio::join!(erc20_symbol(ctx, t0), erc20_symbol(ctx, t1));
    let tokens = vec![
        erc20_info(t0, s0, pos.pool.token0.decimals),
        erc20_info(t1, s1, pos.pool.token1.decimals),
    ];

    let a0 = bigint_to_u256(pos.amount0().map_err(rpc_err)?.quotient())?;
    let a1 = bigint_to_u256(pos.amount1().map_err(rpc_err)?.quotient())?;

    // Fees, two ways
    let growth = get_collectable_token_amounts(npm, token_id, ctx.provider.clone(), bid)
        .await
        .map_err(|e| rpc_err(format!("v3 fee growth {token_id}: {e}")))?;
    let collect = npm_c
        .collect(INonfungiblePositionManagerLite::CollectParams {
            tokenId: token_id,
            recipient: owner,
            amount0Max: u128::MAX,
            amount1Max: u128::MAX,
        })
        .from(owner)
        .block(ctx.block_id())
        .call()
        .await
        .map(|r| (r.amount0, r.amount1))
        .map_err(|e| crate::utils::url::redact_urls_in_text(&e.to_string()));
    let (collectable, cross) = v3_reconcile_fees(growth, collect, &tokens);
    let (fees_raw, owed) = split_collectable(collectable, owed_raw);

    let tick = pos.pool.tick_current;
    let fee_u24: alloy::primitives::aliases::U24 = pos.pool.fee.into();
    let fee: u32 = fee_u24.to::<u32>();
    let pool_addr = pos.pool.address(None, None);

    let method = if cross.authoritative == "collect_eth_call" {
        "amounts: uniswap-v3-sdk get_position (liquidity at the current sqrt price); collectable total: eth_call of NonfungiblePositionManager.collect(tokenId, owner, max, max) from the owner, cross-checked against tokensOwed + pool fee growth (get_collectable_token_amounts); fees = collectable - tokensOwed, tokens_owed = tokensOwed, each counted once".to_string()
    } else {
        "amounts: uniswap-v3-sdk get_position (liquidity at the current sqrt price); fees: pool fee growth since the last liquidity change (get_collectable_token_amounts - tokensOwed); tokens_owed = tokensOwed; the collect eth_call failed".to_string()
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
        value_usd: UsdValue::default(),
        fee_tier: Some(fee),
        fee_pct: Some(fee_pct(fee)),
        tick_lower: Some(pos.tick_lower),
        tick_upper: Some(pos.tick_upper),
        tick_current: Some(tick),
        in_range: Some(in_range(tick, pos.tick_lower, pos.tick_upper)),
        liquidity: pos.liquidity.to_string(),
        lp_total_supply: None,
        pool_share_pct: None,
        tokens,
        method,
        notes: Vec::new(),
    })
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

fn is_revert(e: &impl std::fmt::Display) -> bool {
    e.to_string().to_lowercase().contains("revert")
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

async fn value_v4(ctx: &Ctx, token_id: U256) -> anyhow::Result<LpPosition> {
    use uniswap_v4_sdk::extensions::{get_position, PoolManagerLens};
    use uniswap_v4_sdk::prelude::calculate_position_key;

    let addrs = ctx.addresses()?;
    let (pm, manager) = match (addrs.v4_position_manager, addrs.v4_pool_manager) {
        (Some(pm), Some(m)) => (pm, m),
        _ => anyhow::bail!("No Uniswap v4 deployment known for {}", ctx.chain.name()),
    };
    let chain_id = ctx.chain.chain_id();
    let bid = Some(ctx.block_id());

    let owner = match IV4PositionManagerLite::new(pm, ctx.provider.clone())
        .ownerOf(token_id)
        .block(ctx.block_id())
        .call()
        .await
    {
        Ok(o) => o,
        Err(e) if is_revert(&e) => return Err(not_found(token_id)),
        Err(e) => return Err(rpc_err(e)),
    };
    let pos = get_position(chain_id, pm, token_id, ctx.provider.clone(), bid)
        .await
        .map_err(|e| rpc_err(format!("v4 position {token_id}: {e}")))?;

    let mut tokens = Vec::with_capacity(2);
    for c in [&pos.pool.currency0, &pos.pool.currency1] {
        if c.is_native() {
            tokens.push(native_info(ctx.chain));
        } else {
            let a = c.address();
            tokens.push(erc20_info(a, erc20_symbol(ctx, a).await, c.decimals()));
        }
    }

    let a0 = bigint_to_u256(pos.amount0().map_err(rpc_err)?.quotient())?;
    let a1 = bigint_to_u256(pos.amount1().map_err(rpc_err)?.quotient())?;

    // Uncollected fees from the PoolManager position (owner = PositionManager,
    // salt = tokenId)
    let lens = PoolManagerLens::new(manager, ctx.provider.clone());
    let pool_id = pos.pool.pool_id;
    let (tl, tu) = (pos.tick_lower, pos.tick_upper);
    let key = calculate_position_key(
        pm,
        I24::try_from(tl).map_err(|e| anyhow::anyhow!("tick {tl}: {e}"))?,
        I24::try_from(tu).map_err(|e| anyhow::anyhow!("tick {tu}: {e}"))?,
        B256::from(token_id),
    );
    let (liq, last0, last1) = lens
        .get_position_info(pool_id, key, bid)
        .await
        .map_err(rpc_err)?;
    let (in0, in1) = lens
        .get_fee_growth_inside(pool_id, tl, tu, bid)
        .await
        .map_err(rpc_err)?;
    let (f0, f1) = uniswap_v3_sdk::prelude::get_tokens_owed(last0, last1, liq, in0, in1);

    let mut notes = Vec::new();
    if liq != pos.liquidity {
        notes.push(format!(
            "PoolManager position liquidity {liq} differs from PositionManager liquidity {}; fees may be wrong",
            pos.liquidity
        ));
    }

    let declared_fee = pos.pool.fee.to::<u32>();
    let fee = if declared_fee == DYNAMIC_FEE_FLAG {
        let (_, _, _, lp_fee) = lens.get_slot0(pool_id, bid).await.map_err(rpc_err)?;
        notes.push("dynamic-fee pool: fee_tier is the current LP fee from slot0".into());
        lp_fee.to::<u32>()
    } else {
        declared_fee
    };

    let hooks = pos.pool.hooks;
    let mut method = "amounts: uniswap-v4-sdk get_position (liquidity at the current sqrt price); fees: PoolManager fee growth inside the range minus the position's feeGrowthInsideLast (PoolManagerLens get_fee_growth_inside / get_position_info), times liquidity".to_string();
    if hooks != Address::ZERO {
        method.push_str("; hooked pool: hooks can take or add value outside the PoolManager fee growth, which this does not include");
    }

    let tick = pos.pool.tick_current;
    Ok(LpPosition {
        protocol: "uniswap-v4".into(),
        chain: ctx.chain.name().to_string(),
        block: ctx.block,
        token_id: Some(token_id.to_string()),
        owner: format!("{owner:#x}"),
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
    let p = IUniswapV2PairLite::new(pair, ctx.provider.clone());
    let (balance, total_supply, reserves, t0, t1) = ctx
        .provider
        .multicall()
        .add(p.balanceOf(owner))
        .add(p.totalSupply())
        .add(p.getReserves())
        .add(p.token0())
        .add(p.token1())
        .block(ctx.block_id())
        .aggregate()
        .await
        .map_err(|e| {
            rpc_err(format!(
                "{pair:#x} does not answer as a Uniswap v2-style pair: {e}"
            ))
        })?;
    let r0 = U256::from(reserves.reserve0);
    let r1 = U256::from(reserves.reserve1);
    let (a0, a1) = v2_share(balance, total_supply, r0, r1)?;

    let (s0, s1, d0, d1, pair_symbol) = tokio::join!(
        erc20_symbol(ctx, t0),
        erc20_symbol(ctx, t1),
        erc20_decimals(ctx, t0),
        erc20_decimals(ctx, t1),
        erc20_symbol(ctx, pair),
    );
    let tokens = vec![erc20_info(t0, s0, d0?), erc20_info(t1, s1, d1?)];
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
            "v2-style pair ({pair_symbol}): one multicall of balanceOf(owner), totalSupply(), getReserves(), token0(), token1(); amounts = reserves x balance / totalSupply. Swap fees are compounded into the reserves, so there are no separate uncollected fees"
        ),
        notes: vec![
            "Only LP tokens held directly by the owner are counted (not LP tokens staked in a farm or gauge)".into(),
            "If the factory's protocol fee is on, a small unminted protocol share is not deducted".into(),
        ],
    })
}

/// Fetch USD prices for every distinct token across `positions` and fill
/// `prices` and `value_usd`.
async fn price_positions(positions: &mut [LpPosition], chain: &str) {
    let mut keys: Vec<String> = positions
        .iter()
        .flat_map(|p| p.tokens.iter().map(|t| t.price_key.clone()))
        .collect();
    keys.sort();
    keys.dedup();

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
    let map: HashMap<String, (Option<f64>, Option<String>)> = fetched.into_iter().collect();

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
        apply_usd(p);
    }
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

fn emit(positions: &[LpPosition], single: bool, format: OutputFormat) -> anyhow::Result<()> {
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
            if !single && positions.len() > 1 {
                let totals: Vec<Option<f64>> =
                    positions.iter().map(|p| p.value_usd.total).collect();
                if totals.iter().all(|t| t.is_some()) && !positions[0].prices.is_empty() {
                    let sum: f64 = totals.iter().flatten().sum();
                    println!(
                        "\nTotal value: ${sum:.2} across {} positions",
                        positions.len()
                    );
                }
            }
        }
        OutputFormat::Ndjson => {
            for p in positions {
                println!("{}", serde_json::to_string(p)?);
            }
        }
        OutputFormat::Json => {
            if single {
                println!("{}", serde_json::to_string_pretty(&positions[0])?);
            } else {
                let total: Option<f64> = positions
                    .iter()
                    .map(|p| p.value_usd.total)
                    .sum::<Option<f64>>();
                let out = serde_json::json!({
                    "count": positions.len(),
                    "total_value_usd": if positions.iter().all(|p| !p.prices.is_empty()) { total } else { None },
                    "positions": positions,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            }
        }
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
        price_positions(&mut list, ctx.chain.name()).await;
        pos = list[0].clone();
    }
    emit(std::slice::from_ref(&pos), true, args.format)
}

/// `uniswap lp-positions`
pub async fn handle_lp_positions(
    args: &LpPositionsArgs,
    chain: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    use uniswap_v3_sdk::extensions::get_all_positions_by_owner;

    let ctx = connect(args.rpc_url.as_deref(), chain, args.block).await?;
    let owner = parse_addr(&args.owner, "owner")?;
    let npm = ctx.addresses()?.nonfungible_position_manager;
    if !quiet {
        eprintln!(
            "Listing v3 positions of {owner:#x} on {} at block {}...",
            ctx.chain, ctx.block
        );
    }
    let states = get_all_positions_by_owner(npm, owner, ctx.provider.clone(), Some(ctx.block_id()))
        .await
        .map_err(|e| rpc_err(format!("listing v3 positions: {e}")))?;
    let total = states.len();
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
            "{total} v3 position NFT(s); valuing {} {}",
            ids.len(),
            if args.all {
                ""
            } else {
                "open (use --all for closed ones)"
            }
        );
    }

    let results: Vec<anyhow::Result<LpPosition>> = futures::stream::iter(ids.iter().copied())
        .map(|id| {
            let ctx = &ctx;
            async move { value_v3(ctx, id).await }
        })
        .buffered(LIST_CONCURRENCY)
        .collect()
        .await;
    let mut positions = Vec::with_capacity(results.len());
    for (id, r) in ids.iter().zip(results) {
        match r {
            Ok(p) => positions.push(p),
            Err(e) => eprintln!("Warning: position {id}: {e}"),
        }
    }
    if !args.no_price {
        price_positions(&mut positions, ctx.chain.name()).await;
    }
    if !quiet {
        eprintln!("Note: v4 positions are not listed (the v4 PositionManager is not enumerable on-chain); value one with `uniswap position <id> --version v4`");
    }
    emit(&positions, false, args.format)
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
}
