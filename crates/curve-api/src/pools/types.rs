//! Types for the Pools API

use serde::{Deserialize, Serialize};

/// Response wrapper for pool data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolsResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Pool data
    pub data: PoolsData,
}

/// Pool data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolsData {
    /// List of pools
    pub pool_data: Vec<Pool>,
    /// Total TVL in USD
    pub tvl: Option<f64>,
    /// Total TVL across all pools
    pub tvl_all: Option<f64>,
}

/// A Curve pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pool {
    /// Pool ID (e.g., "factory-v2-0")
    pub id: String,
    /// Pool name
    pub name: String,
    /// Pool address
    pub address: String,
    /// LP token address
    pub lp_token_address: Option<String>,
    /// Gauge address
    pub gauge_address: Option<String>,
    /// Pool implementation
    pub implementation: Option<String>,
    /// Pool type (e.g., "stable", "crypto")
    #[serde(rename = "assetTypeName")]
    pub asset_type_name: Option<String>,
    /// Coins in the pool
    pub coins: Vec<Coin>,
    /// Underlying coins (for lending pools)
    pub underlying_coins: Option<Vec<Coin>>,
    /// USD total in the pool
    pub usd_total: Option<f64>,
    /// Total supply of LP token
    pub total_supply: Option<String>,
    /// Amplification coefficient
    pub amplification_coefficient: Option<String>,
    /// Virtual price
    pub virtual_price: Option<String>,
    /// Whether the pool is a metapool
    pub is_meta: Option<bool>,
    /// Whether the pool uses a crypto swap
    pub is_crypto: Option<bool>,
    /// Pool parameters
    pub parameters: Option<PoolParameters>,
    /// Gauge rewards
    pub gauge_rewards: Option<Vec<GaugeReward>>,
}

/// A coin in a pool
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coin {
    /// Coin address
    pub address: String,
    /// USD price
    pub usd_price: Option<f64>,
    /// Decimals
    pub decimals: Option<String>,
    /// Symbol
    pub symbol: Option<String>,
    /// Pool balance
    pub pool_balance: Option<String>,
    /// Is base pool LP token
    pub is_base_pool_lp_token: Option<bool>,
}

/// Pool parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolParameters {
    /// A parameter (for crypto pools)
    #[serde(rename = "A")]
    pub a: Option<String>,
    /// Fee (as percentage string)
    pub fee: Option<String>,
    /// Admin fee
    pub admin_fee: Option<String>,
    /// Gamma (for crypto pools)
    pub gamma: Option<String>,
}

/// Gauge reward info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GaugeReward {
    /// Reward token address
    pub token_address: String,
    /// Reward token symbol
    pub symbol: Option<String>,
    /// APY from this reward
    pub apy: Option<f64>,
}

/// Response for pool list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolListResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Pool addresses
    pub data: PoolListData,
}

/// Pool list data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PoolListData {
    /// Pool addresses by registry
    pub pool_list: Vec<String>,
}

/// Hidden pools response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HiddenPoolsResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Hidden pools by chain
    pub data: serde_json::Value,
}
