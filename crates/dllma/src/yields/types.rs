//! Types for yield/farming data (Pro)

use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a Vec that may be null or missing as an empty Vec
fn null_to_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(deserializer).map(std::option::Option::unwrap_or_default)
}

/// Deserialize a Vec<String> that may contain null elements, filtering them out
fn strings_filtering_nulls<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<Vec<Option<String>>>::deserialize(deserializer)
        .map(|opt| opt.unwrap_or_default().into_iter().flatten().collect())
}

/// Yields API response wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct YieldsResponse<T> {
    /// Response status
    pub status: String,
    /// Response data
    pub data: T,
}

/// Yield pool data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldPool {
    /// Pool ID
    pub pool: String,
    /// Pool chain
    pub chain: String,
    /// Pool project/protocol
    pub project: String,
    /// Pool symbol
    pub symbol: String,
    /// Pool TVL in USD
    pub tvl_usd: Option<f64>,
    /// Base APY (from trading fees, etc.)
    pub apy_base: Option<f64>,
    /// Reward APY (from token rewards)
    pub apy_reward: Option<f64>,
    /// Total APY (base + reward)
    pub apy: Option<f64>,
    /// APY 7-day mean
    pub apy_mean30d: Option<f64>,
    /// Reward tokens
    #[serde(default, deserialize_with = "strings_filtering_nulls")]
    pub reward_tokens: Vec<String>,
    /// Underlying tokens
    #[serde(default, deserialize_with = "strings_filtering_nulls")]
    pub underlying_tokens: Vec<String>,
    /// IL risk (impermanent loss)
    pub il_risk: Option<String>,
    /// Pool exposure
    pub exposure: Option<String>,
    /// Predictions
    pub predictions: Option<YieldPredictions>,
    /// Pool meta
    pub pool_meta: Option<String>,
    /// Pool URL
    pub url: Option<String>,
    /// Stablecoin pool
    pub stablecoin: Option<bool>,
}

/// Yield predictions
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldPredictions {
    /// Predicted class
    pub predicted_class: Option<String>,
    /// Predicted probability
    pub predicted_probability: Option<f64>,
    /// Binnned confidence
    pub binned_confidence: Option<i32>,
}

/// Legacy pool data with contract addresses
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyPool {
    /// Pool ID
    pub pool: String,
    /// Chain
    pub chain: String,
    /// Project
    pub project: String,
    /// Symbol
    pub symbol: String,
    /// TVL
    pub tvl_usd: Option<f64>,
    /// APY
    pub apy: Option<f64>,
    /// Pool address
    pub pool_address: Option<String>,
    /// Underlying tokens with addresses
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub underlying_tokens: Vec<TokenInfo>,
}

/// Token info with address
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    /// Token address
    pub address: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
}

/// Yield chart data point
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YieldChartPoint {
    /// Unix timestamp
    pub timestamp: String,
    /// TVL in USD
    pub tvl_usd: Option<f64>,
    /// APY
    pub apy: Option<f64>,
    /// Base APY
    pub apy_base: Option<f64>,
    /// Reward APY
    pub apy_reward: Option<f64>,
    /// IL 7-day
    pub il7d: Option<f64>,
    /// APY 7-day base
    pub apy_base7d: Option<f64>,
}

/// Borrow pool data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BorrowPool {
    /// Pool ID
    pub pool: String,
    /// Chain
    pub chain: String,
    /// Project
    pub project: String,
    /// Symbol
    pub symbol: String,
    /// TVL
    pub tvl_usd: Option<f64>,
    /// Lend APY (supply APY)
    pub apy: Option<f64>,
    /// Borrow APY
    pub apy_borrow: Option<f64>,
    /// Total supply in USD
    pub total_supply_usd: Option<f64>,
    /// Total borrow in USD
    pub total_borrow_usd: Option<f64>,
    /// LTV (loan-to-value ratio)
    pub ltv: Option<f64>,
    /// Utilization rate
    pub utilization: Option<f64>,
    /// Reward tokens
    #[serde(default, deserialize_with = "strings_filtering_nulls")]
    pub reward_tokens: Vec<String>,
}

/// Lend/Borrow chart data point
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LendBorrowChartPoint {
    /// Timestamp
    pub timestamp: String,
    /// Supply APY
    pub apy: Option<f64>,
    /// Borrow APY
    pub apy_borrow: Option<f64>,
    /// Total supply
    pub total_supply_usd: Option<f64>,
    /// Total borrow
    pub total_borrow_usd: Option<f64>,
}

/// Perpetual futures funding rate
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PerpRate {
    /// Market/pair
    pub market: String,
    /// Base asset
    pub base_asset: String,
    /// Exchange/protocol
    pub marketplace: String,
    /// Funding rate (annualized)
    pub funding_rate: Option<f64>,
    /// Open interest in USD
    pub open_interest: Option<f64>,
    /// Funding rate 7-day average
    pub funding_rate7d_average: Option<f64>,
}

/// Liquid staking derivative rate
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LsdRate {
    /// Protocol name
    pub name: String,
    /// Symbol
    pub symbol: String,
    /// Chain
    pub chain: String,
    /// Current APY
    pub apy: Option<f64>,
    /// TVL
    pub tvl: Option<f64>,
    /// ETH staked
    pub eth_staked: Option<f64>,
    /// Market share
    pub market_share: Option<f64>,
}
