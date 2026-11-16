//! Types for the v2 yields API (`/yields/v2/*`, Pro)
//!
//! All APY figures are percentages. Nested objects that the API may leave
//! out or send as `null` read as their default (every field `None`), and
//! the structs are `#[non_exhaustive]` so new upstream fields can be added
//! without a breaking change.

use serde::{Deserialize, Serialize};

use crate::de::{null_default, opt_string_or_number};

/// v2 yields response wrapper: `{"data": ...}` (no `status` field, unlike
/// [`super::YieldsResponse`])
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct YieldsV2Response<T> {
    /// Response data
    pub data: T,
}

/// A page of earn pools from `/yields/v2/earn/query`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct EarnPoolsPage {
    /// Pools on this page, TVL descending
    #[serde(default, deserialize_with = "null_default")]
    pub data: Vec<EarnPool>,
    /// Paging information
    #[serde(default, deserialize_with = "null_default")]
    pub pagination: Pagination,
}

/// Paging information of a filtered query
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Pagination {
    /// Rows matching the filters, across all pages
    pub total: Option<u64>,
    /// 1-based page number
    pub page: Option<u64>,
    /// Page size
    pub limit: Option<u64>,
}

/// A token in a pool or market
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct YieldToken {
    /// Token address
    #[serde(default)]
    pub address: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u32>,
}

/// Receipt / vault token representing a position
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PoolToken {
    /// Token address
    #[serde(default)]
    pub address: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token name
    pub name: Option<String>,
}

/// Display strings for listings
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct PoolDisplay {
    /// Main label (for borrow markets: the collateral symbol, or
    /// `"COLLATERAL / DEBT"` for isolated pairs)
    pub label: Option<String>,
    /// Detail, e.g. a market variant
    pub detail: Option<String>,
}

/// APY in percent, split into organic and incentive parts
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApyComponents {
    /// Organic APY, excluding incentive rewards
    pub base: Option<f64>,
    /// APY from incentive token emissions; `None` when there are none
    pub reward: Option<f64>,
    /// `base + reward` for supply APYs; `base - reward` for borrow costs,
    /// where a negative total means the borrower is net paid
    pub total: Option<f64>,
}

/// APY over several windows
///
/// Latest-data endpoints fill `current`, `avg_7d` and `avg_30d`; daily
/// history endpoints fill `avg_1d` (that day), `avg_7d` and `avg_30d`
/// (trailing averages ending that day).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApyWindows {
    /// Latest observed values
    #[serde(default, deserialize_with = "null_default")]
    pub current: ApyComponents,
    /// Average over the day (history rows)
    #[serde(default, deserialize_with = "null_default")]
    pub avg_1d: ApyComponents,
    /// Trailing 7-day average
    #[serde(default, deserialize_with = "null_default")]
    pub avg_7d: ApyComponents,
    /// Trailing 30-day average
    #[serde(default, deserialize_with = "null_default")]
    pub avg_30d: ApyComponents,
}

/// APY figure (or score) keyed by which components it includes
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ApyByComposition {
    /// Base APY only
    pub base: Option<f64>,
    /// Base plus reward
    pub base_reward: Option<f64>,
    /// Base plus intrinsic
    pub base_intrinsic: Option<f64>,
    /// Base plus reward plus intrinsic
    pub base_reward_intrinsic: Option<f64>,
}

/// Net APY of a borrow route over several windows
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct NetApyWindows {
    /// Latest observed values
    #[serde(default, deserialize_with = "null_default")]
    pub current: ApyByComposition,
    /// Trailing 7-day average
    #[serde(default, deserialize_with = "null_default")]
    pub avg_7d: ApyByComposition,
    /// Trailing 30-day average
    #[serde(default, deserialize_with = "null_default")]
    pub avg_30d: ApyByComposition,
}

/// Intrinsic APY in percent over several windows (see [`ApyWindows`] for
/// which windows each endpoint fills)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntrinsicApy {
    /// Latest observed value
    #[serde(default, deserialize_with = "null_default")]
    pub current: BaseApy,
    /// Average over the day (history rows)
    #[serde(default, deserialize_with = "null_default")]
    pub avg_1d: BaseApy,
    /// Trailing 7-day average
    #[serde(default, deserialize_with = "null_default")]
    pub avg_7d: BaseApy,
    /// Trailing 30-day average
    #[serde(default, deserialize_with = "null_default")]
    pub avg_30d: BaseApy,
}

/// A base-only APY value
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BaseApy {
    /// APY in percent
    pub base: Option<f64>,
}

/// Yield accruing inside a yield-bearing token (e.g. stETH staking yield
/// inside a wstETH pool), reported separately and NOT included in the
/// base/reward/total APY
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntrinsicYield {
    /// Intrinsic APY
    #[serde(default, deserialize_with = "null_default")]
    pub apy: IntrinsicApy,
    /// Earn pool the intrinsic yield is sourced from (latest-data rows)
    pub source_pool: Option<IntrinsicSource>,
}

/// Earn pool an intrinsic yield is sourced from
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct IntrinsicSource {
    /// Earn pool id
    #[serde(default)]
    pub id: String,
    /// Address of the yield-bearing token
    pub pool_token: Option<String>,
    /// Symbol of the yield-bearing token
    pub symbol: Option<String>,
}

/// Boolean pool attributes, also filters of `/yields/v2/earn/query`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct EarnAttributes {
    /// All pool assets are stablecoins
    pub stablecoin_only: Option<bool>,
    /// The position is exposed to a single asset
    pub single_asset_exposure: Option<bool>,
    /// The position can suffer impermanent loss
    pub impermanent_loss_risk: Option<bool>,
}

/// Pool token holder statistics; `None` where holders are not tracked
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct HolderStats {
    /// Number of holders
    pub count: Option<f64>,
    /// Average position size in USD
    pub avg_position_usd: Option<f64>,
    /// Share of supply held by the top 10 holders, in percent
    pub top_10_share_pct: Option<f64>,
}

/// An earn pool (supply-side yield) from `/yields/v2/earn`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct EarnPool {
    /// Pool id (UUID), the same id space as the v1 `pool` field
    pub id: String,
    /// Stable pool key
    #[serde(default)]
    pub pool_key: String,
    /// Receipt / vault token; `None` when the pool has none
    pub pool_token: Option<PoolToken>,
    /// Display strings
    #[serde(default, deserialize_with = "null_default")]
    pub display: PoolDisplay,
    /// The asset provided to enter the pool (e.g. ETH, USDC)
    pub reference_asset_symbol: Option<String>,
    /// Pool page on the protocol app
    pub url: Option<String>,
    /// Protocol display name
    #[serde(default)]
    pub protocol: String,
    /// Chain display name
    #[serde(default)]
    pub chain: String,
    /// Tokens deposited into the pool
    #[serde(default, deserialize_with = "null_default")]
    pub underlying_tokens: Vec<YieldToken>,
    /// Incentive tokens currently emitted to the pool
    #[serde(default, deserialize_with = "null_default")]
    pub reward_tokens: Vec<YieldToken>,
    /// Total value locked in USD
    pub tvl_usd: Option<f64>,
    /// Supply APY
    #[serde(default, deserialize_with = "null_default")]
    pub apy: ApyWindows,
    /// Intrinsic yield of a yield-bearing underlying token
    pub intrinsic: Option<IntrinsicYield>,
    /// Stability score (0 to 1, higher is steadier) of the trailing 30-day
    /// APY series, by APY composition
    #[serde(default, deserialize_with = "null_default")]
    pub stability_30d: ApyByComposition,
    /// Pool token price per share for vault-style pools
    pub price_per_share: Option<f64>,
    /// Trailing 24h trading volume in USD for DEX pools
    pub volume_1d_usd: Option<f64>,
    /// Boolean attributes
    #[serde(default, deserialize_with = "null_default")]
    pub attributes: EarnAttributes,
    /// Holder statistics
    #[serde(default, deserialize_with = "null_default")]
    pub holders: HolderStats,
    /// Remaining deposit capacity in USD for supply-capped lending pools
    pub available_supply_usd: Option<f64>,
}

/// One day of an earn pool's history
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct EarnHistoryPoint {
    /// Day (UTC)
    pub date: String,
    /// Total value locked in USD that day
    pub tvl_usd: Option<f64>,
    /// Supply APY (`avg_1d`, `avg_7d`, `avg_30d`)
    #[serde(default, deserialize_with = "null_default")]
    pub apy: ApyWindows,
    /// Intrinsic yield of a yield-bearing underlying token
    pub intrinsic: Option<IntrinsicYield>,
    /// Pool token price per share for vault-style pools
    pub price_per_share: Option<f64>,
}

/// Measured base APY in percent; can be negative
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VerifiedApy {
    /// 1-day average (that day, for history rows)
    pub avg_1d: Option<f64>,
    /// Trailing 7-day average
    pub avg_7d: Option<f64>,
    /// Trailing 30-day average
    pub avg_30d: Option<f64>,
}

/// Exit health from the most recent simulated withdrawal
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ExitHealth {
    /// `instant`, `queued`, `partial`, `depleted`, `unavailable` or
    /// `blocked`
    pub status: Option<String>,
    /// Wait in seconds before a queued withdrawal can be claimed
    pub eta_seconds: Option<f64>,
    /// Exit fee in basis points
    pub fee_bps: Option<f64>,
    /// Timestamp of the observation
    pub as_of: Option<String>,
    /// Flat exit fee in raw base units of the denomination, as a decimal
    /// string
    #[serde(default, deserialize_with = "opt_string_or_number")]
    pub fee_flat: Option<String>,
}

/// Funded exit capacity
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ExitLiquidity {
    /// Raw base units of the denomination, as a decimal string
    #[serde(default, deserialize_with = "opt_string_or_number")]
    pub amount: Option<String>,
    /// Capacity in USD
    pub usd: Option<f64>,
    /// Price impact of the simulated exit, in basis points
    pub slippage_bps: Option<f64>,
    /// Day (UTC) the capacity was measured
    pub as_of: Option<String>,
}

/// An amount in raw base units of the denomination, valued in USD
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VerifiedAmount {
    /// Raw base units of the denomination, as a decimal string
    #[serde(default, deserialize_with = "opt_string_or_number")]
    pub amount: Option<String>,
    /// Value in USD
    pub usd: Option<f64>,
    /// Day (UTC) it was measured
    pub as_of: Option<String>,
}

/// Measured yield for an earn pool, from `/yields/v2/earn/verified`
///
/// Each figure is replayed from a real holder's withdrawal, so it shows
/// what an exit would have paid rather than what the protocol reports.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VerifiedEarnPool {
    /// Pool id (UUID); joins to [`EarnPool::id`]
    pub id: String,
    /// Measured base APY
    #[serde(default, deserialize_with = "null_default")]
    pub apy: VerifiedApy,
    /// Timestamp the APY windows end at
    pub apy_as_of: Option<String>,
    /// Why the APY is absent or qualified
    pub apy_note: Option<String>,
    /// Agreement between holders: `high`, `medium`, `low` or `conflict`
    pub apy_confidence: Option<String>,
    /// Exit health
    #[serde(default, deserialize_with = "null_default")]
    pub exit: ExitHealth,
    /// Funded exit capacity; `None` when none is exposed on-chain
    pub exit_liquidity: Option<ExitLiquidity>,
    /// Receipt supply priced at the rate an exit actually paid
    #[serde(default, deserialize_with = "null_default")]
    pub tvl: VerifiedAmount,
    /// Underlying assets per position token, in human units
    pub price_per_share: Option<f64>,
    /// Asset an exit pays out (`symbol` is not sent); `None` when it could
    /// not be pinned
    pub denomination: Option<YieldToken>,
    /// On-chain calls the simulation executes, in order (only from
    /// `/yields/v2/earn/verified/{id}`)
    #[serde(default, deserialize_with = "null_default")]
    pub methods: Vec<String>,
}

/// One measured day of a verified earn pool's history
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VerifiedHistoryPoint {
    /// Day (UTC)
    pub date: String,
    /// Raw measurement outcome, e.g. `ok_full`, `delayed` or
    /// `reverted_liquidity`
    pub outcome: Option<String>,
    /// Wait in seconds before that day's queued withdrawal could be claimed
    pub eta_seconds: Option<f64>,
    /// Holder-consensus confidence: `high`, `medium`, `low` or `conflict`
    pub confidence: Option<String>,
    /// Measured base APY
    #[serde(default, deserialize_with = "null_default")]
    pub apy: VerifiedApy,
    /// Underlying assets per position token, in human units
    pub price_per_share: Option<f64>,
    /// The pool's reported price, on days the exit could not price one
    pub price_per_share_quoted: Option<f64>,
    /// Why `price_per_share` is `None` on a measured day
    pub reason: Option<String>,
    /// Funded exit capacity
    pub exit_liquidity: Option<ExitLiquidity>,
    /// Receipt supply priced at the rate an exit actually paid
    pub tvl: Option<VerifiedAmount>,
    /// Supply valued at the reported rate, for days `tvl` is missing
    pub tvl_quoted: Option<VerifiedAmount>,
}

/// Intrinsic yield on either side of a borrow market
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct MarketIntrinsic {
    /// Intrinsic yield of the supply-side token
    pub supply: Option<IntrinsicYield>,
    /// Intrinsic yield of the borrowed token, which adds to the effective
    /// borrow cost
    pub borrow: Option<IntrinsicYield>,
}

/// A borrow market from `/yields/v2/borrow/markets`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BorrowMarket {
    /// Market id (UUID)
    pub id: String,
    /// Stable market key
    #[serde(default)]
    pub pool_key: String,
    /// Receipt / vault token; `None` when the market has none
    pub pool_token: Option<PoolToken>,
    /// Display strings
    #[serde(default, deserialize_with = "null_default")]
    pub display: PoolDisplay,
    /// Market page on the protocol app
    pub url: Option<String>,
    /// Protocol display name
    #[serde(default)]
    pub protocol: String,
    /// Chain display name
    #[serde(default)]
    pub chain: String,
    /// Token supplied as collateral
    pub collateral_token: Option<YieldToken>,
    /// Token borrowed from the market
    pub borrow_token: Option<YieldToken>,
    /// Incentive tokens currently emitted to the market
    #[serde(default, deserialize_with = "null_default")]
    pub reward_tokens: Vec<YieldToken>,
    /// Maximum loan-to-value as a fraction (0.805 = 80.5%)
    pub loan_to_value: Option<f64>,
    /// Total supplied, in USD
    pub total_supply_usd: Option<f64>,
    /// Total borrowed, in USD
    pub total_borrow_usd: Option<f64>,
    /// Liquidity still available to borrow, in USD
    pub available_borrow_usd: Option<f64>,
    /// APY earned by suppliers
    #[serde(default, deserialize_with = "null_default")]
    pub supply_apy: ApyWindows,
    /// APY paid by borrowers (`total` = base - reward, can be negative)
    #[serde(default, deserialize_with = "null_default")]
    pub borrow_cost_apy: ApyWindows,
    /// Intrinsic yield on either side
    #[serde(default, deserialize_with = "null_default")]
    pub intrinsic: MarketIntrinsic,
}

/// One day of a borrow market's history
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BorrowMarketHistoryPoint {
    /// Day (UTC)
    pub date: String,
    /// Total borrowed, in USD
    pub total_borrow_usd: Option<f64>,
    /// Liquidity still available to borrow, in USD
    pub available_borrow_usd: Option<f64>,
    /// Borrow cost APY (`avg_1d`, `avg_7d`, `avg_30d`)
    #[serde(default, deserialize_with = "null_default")]
    pub borrow_cost_apy: ApyWindows,
    /// Intrinsic yield of the borrowed token
    pub intrinsic: Option<IntrinsicYield>,
}

/// Collateral side of a borrow route
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RouteCollateral {
    /// Collateral pool id (UUID)
    #[serde(default)]
    pub id: String,
    /// Stable pool key of the collateral pool
    #[serde(default)]
    pub pool_key: String,
    /// Pool page on the protocol app
    pub url: Option<String>,
    /// Token supplied as collateral
    pub token: Option<YieldToken>,
    /// Maximum loan-to-value as a fraction
    pub loan_to_value: Option<f64>,
    /// Total supplied to the collateral pool, in USD
    pub total_supply_usd: Option<f64>,
    /// Supply APY earned on the collateral
    #[serde(default, deserialize_with = "null_default")]
    pub apy: ApyWindows,
    /// Intrinsic yield of the collateral token
    pub intrinsic: Option<IntrinsicYield>,
    /// Incentive tokens emitted to the collateral side
    #[serde(default, deserialize_with = "null_default")]
    pub reward_tokens: Vec<YieldToken>,
}

/// Debt side of a borrow route
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct RouteBorrow {
    /// Debt market id (UUID)
    #[serde(default)]
    pub id: String,
    /// Stable pool key of the debt market
    #[serde(default)]
    pub pool_key: String,
    /// Market page on the protocol app
    pub url: Option<String>,
    /// Token borrowed
    pub token: Option<YieldToken>,
    /// Total borrowed, in USD
    pub total_borrow_usd: Option<f64>,
    /// Liquidity still available to borrow, in USD
    pub available_borrow_usd: Option<f64>,
    /// Borrow cost APY (`total` = base - reward, can be negative)
    #[serde(default, deserialize_with = "null_default")]
    pub cost_apy: ApyWindows,
    /// Intrinsic yield of the borrowed token
    pub intrinsic: Option<IntrinsicYield>,
    /// Incentive tokens emitted to the borrow side
    #[serde(default, deserialize_with = "null_default")]
    pub reward_tokens: Vec<YieldToken>,
}

/// A borrowing route from `/yields/v2/borrow/routes`: a collateral position
/// paired with a debt market on the same protocol and chain
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct BorrowRoute {
    /// Display strings
    #[serde(default, deserialize_with = "null_default")]
    pub display: PoolDisplay,
    /// Protocol display name
    #[serde(default)]
    pub protocol: String,
    /// Chain display name
    #[serde(default)]
    pub chain: String,
    /// The supplied collateral position
    #[serde(default, deserialize_with = "null_default")]
    pub collateral: RouteCollateral,
    /// The borrowed debt position
    #[serde(default, deserialize_with = "null_default")]
    pub borrow: RouteBorrow,
    /// Collateral supply APY minus borrow cost; negative means the position
    /// costs more than the collateral earns
    #[serde(default, deserialize_with = "null_default")]
    pub net_apy: NetApyWindows,
}

/// APY of a looped position
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct LoopApy {
    /// Number of borrow/re-supply iterations
    pub loop_count: Option<f64>,
    /// Net APY in percent at `loop_count` iterations
    pub total: Option<f64>,
}

/// Accumulated leverage of a looped position
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Leverage {
    /// Total supplied relative to initial capital
    pub supply_multiplier: Option<f64>,
    /// Total borrowed relative to initial capital
    pub borrow_multiplier: Option<f64>,
}

/// Incentive tokens per market side
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SideRewardTokens {
    /// Emitted to the supply side
    #[serde(default, deserialize_with = "null_default")]
    pub supply: Vec<YieldToken>,
    /// Emitted to the borrow side
    #[serde(default, deserialize_with = "null_default")]
    pub borrow: Vec<YieldToken>,
}

/// A leveraged looping strategy from `/yields/v2/loops`: supply a token,
/// borrow it against itself and re-supply, at the maximum loan-to-value
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct LoopStrategy {
    /// Collateral pool id (UUID)
    pub id: String,
    /// Display strings
    #[serde(default, deserialize_with = "null_default")]
    pub display: PoolDisplay,
    /// Protocol display name
    #[serde(default)]
    pub protocol: String,
    /// Chain display name
    #[serde(default)]
    pub chain: String,
    /// Token being looped (supplied and borrowed)
    pub token: Option<YieldToken>,
    /// Market page on the protocol app
    pub url: Option<String>,
    /// Maximum loan-to-value as a fraction
    pub loan_to_value: Option<f64>,
    /// Total supplied to the market, in USD
    pub total_supply_usd: Option<f64>,
    /// Total borrowed from the market, in USD
    pub total_borrow_usd: Option<f64>,
    /// Liquidity still available to borrow, in USD
    pub available_borrow_usd: Option<f64>,
    /// Unlevered supply APY
    #[serde(default, deserialize_with = "null_default")]
    pub supply_apy: ApyWindows,
    /// Borrow cost APY (`total` = base - reward, can be negative)
    #[serde(default, deserialize_with = "null_default")]
    pub borrow_cost_apy: ApyWindows,
    /// APY of the looped position
    #[serde(default, deserialize_with = "null_default")]
    pub loop_apy: LoopApy,
    /// Accumulated leverage
    #[serde(default, deserialize_with = "null_default")]
    pub leverage: Leverage,
    /// Intrinsic yield of the looped token
    pub intrinsic: Option<IntrinsicYield>,
    /// Incentive tokens per side
    #[serde(default, deserialize_with = "null_default")]
    pub reward_tokens: SideRewardTokens,
}

/// History window for the v2 `.../history` endpoints
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub enum HistoryRange {
    /// Last 30 days
    Days30,
    /// Last 90 days
    Days90,
    /// The full series (the API default)
    #[default]
    Max,
}

impl HistoryRange {
    /// The `range` query value
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Days30 => "30d",
            Self::Days90 => "90d",
            Self::Max => "max",
        }
    }
}

/// Filters and paging for `/yields/v2/earn/query`
///
/// All filters combine with AND; several chains (or protocols) match any of
/// them. Without filters the API returns the top 100 pools by TVL.
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct EarnQuery {
    /// Chain slugs, lowercase (at most 20)
    pub chains: Vec<String>,
    /// Protocol slugs, lowercase, e.g. `aave-v3` (at most 20)
    pub protocols: Vec<String>,
    /// `true` keeps only all-stablecoin pools, `false` only the rest
    pub stablecoin: Option<bool>,
    /// `true` keeps only single-asset pools, `false` only the rest
    pub single_asset_exposure: Option<bool>,
    /// `true` keeps only pools with impermanent loss risk, `false` only the
    /// rest
    pub impermanent_loss_risk: Option<bool>,
    /// Minimum TVL in USD (inclusive)
    pub min_tvl: Option<f64>,
    /// 1-based page number (API default 1)
    pub page: Option<u32>,
    /// Page size, 1 to 300 (API default 100); larger values are rejected
    pub limit: Option<u32>,
}

impl EarnQuery {
    /// Create an empty query
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chain slug
    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chains.push(chain.into());
        self
    }

    /// Add a protocol slug
    #[must_use]
    pub fn protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocols.push(protocol.into());
        self
    }

    /// Filter on all-stablecoin pools
    #[must_use]
    pub fn stablecoin(mut self, stablecoin: bool) -> Self {
        self.stablecoin = Some(stablecoin);
        self
    }

    /// Filter on single-asset exposure
    #[must_use]
    pub fn single_asset_exposure(mut self, single: bool) -> Self {
        self.single_asset_exposure = Some(single);
        self
    }

    /// Filter on impermanent loss risk
    #[must_use]
    pub fn impermanent_loss_risk(mut self, risk: bool) -> Self {
        self.impermanent_loss_risk = Some(risk);
        self
    }

    /// Set the minimum TVL in USD
    #[must_use]
    pub fn min_tvl(mut self, min_tvl: f64) -> Self {
        self.min_tvl = Some(min_tvl);
        self
    }

    /// Set the page number (1-based)
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the page size (1 to 300)
    #[must_use]
    pub fn limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Build the query string, including the leading `?` (empty when
    /// nothing is set)
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut q = url::form_urlencoded::Serializer::new(String::new());
        for chain in &self.chains {
            q.append_pair("chain", chain);
        }
        for protocol in &self.protocols {
            q.append_pair("protocol", protocol);
        }
        let flags = [
            ("stablecoin", self.stablecoin),
            ("single_asset_exposure", self.single_asset_exposure),
            ("impermanent_loss_risk", self.impermanent_loss_risk),
        ];
        for (name, value) in flags {
            if let Some(v) = value {
                q.append_pair(name, if v { "true" } else { "false" });
            }
        }
        if let Some(v) = self.min_tvl {
            q.append_pair("min_tvl", &v.to_string());
        }
        if let Some(v) = self.page {
            q.append_pair("page", &v.to_string());
        }
        if let Some(v) = self.limit {
            q.append_pair("limit", &v.to_string());
        }
        let q = q.finish();
        if q.is_empty() {
            q
        } else {
            format!("?{q}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn earn_query_string() {
        assert_eq!(EarnQuery::new().to_query_string(), "");
        let q = EarnQuery::new()
            .chain("ethereum")
            .chain("arbitrum")
            .protocol("aave-v3")
            .stablecoin(true)
            .impermanent_loss_risk(false)
            .min_tvl(1_000_000.0)
            .page(2)
            .limit(50);
        assert_eq!(
            q.to_query_string(),
            "?chain=ethereum&chain=arbitrum&protocol=aave-v3&stablecoin=true\
             &impermanent_loss_risk=false&min_tvl=1000000&page=2&limit=50"
        );
    }

    #[test]
    fn null_nested_objects_read_as_default() {
        let json = r#"{"data":[{"id":"p1","display":null,"apy":{"current":null,
            "avg_7d":{"base":1.5,"reward":null,"total":1.5}},"holders":null,
            "underlying_tokens":null,"pool_token":null,"intrinsic":null}]}"#;
        let r: YieldsV2Response<Vec<EarnPool>> = serde_json::from_str(json).unwrap();
        let p = &r.data[0];
        assert!(p.display.label.is_none());
        assert!(p.apy.current.total.is_none());
        assert_eq!(p.apy.avg_7d.base, Some(1.5));
        assert!(p.underlying_tokens.is_empty());
        assert!(p.holders.count.is_none());
    }

    #[test]
    fn verified_amounts_accept_numbers_or_strings() {
        let json = r#"{"data":{"id":"p1","apy":{"avg_1d":-0.5,"avg_7d":3.1,"avg_30d":null},
            "exit":{"status":"queued","eta_seconds":86400,"fee_flat":"1000"},
            "exit_liquidity":null,
            "tvl":{"amount":"123456789012345678901234567890","usd":10.5,"as_of":"2026-11-01"},
            "denomination":{"address":"0x01","decimals":6},"methods":["redeem(uint256)"]}}"#;
        let r: YieldsV2Response<VerifiedEarnPool> = serde_json::from_str(json).unwrap();
        let p = r.data;
        assert_eq!(p.apy.avg_1d, Some(-0.5));
        assert_eq!(p.exit.status.as_deref(), Some("queued"));
        assert_eq!(p.exit.fee_flat.as_deref(), Some("1000"));
        assert!(p.exit_liquidity.is_none());
        assert_eq!(
            p.tvl.amount.as_deref(),
            Some("123456789012345678901234567890")
        );
        assert_eq!(p.denomination.unwrap().decimals, Some(6));
        assert_eq!(p.methods, vec!["redeem(uint256)"]);

        let json = r#"{"data":{"id":"p1","tvl":{"amount":42}}}"#;
        let r: YieldsV2Response<VerifiedEarnPool> = serde_json::from_str(json).unwrap();
        assert_eq!(r.data.tvl.amount.as_deref(), Some("42"));
    }
}
