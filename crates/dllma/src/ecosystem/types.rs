//! Types for ecosystem data (Pro)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Category TVL data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    /// Category name
    pub name: String,
    /// Total TVL in USD
    pub tvl: f64,
    /// Number of protocols
    pub protocols: u64,
}

/// Fork relationship data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fork {
    /// Original protocol name
    pub name: String,
    /// Forked protocols
    #[serde(default)]
    pub forks: Vec<String>,
    /// Fork count
    pub fork_count: Option<u64>,
}

/// Oracle protocol data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Oracle {
    /// Oracle name
    pub name: String,
    /// Protocols using this oracle
    #[serde(default)]
    pub protocols: Vec<String>,
    /// Protocol count
    pub protocol_count: Option<u64>,
    /// Total TVL secured
    pub tvl: Option<f64>,
}

/// Entity/company information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    /// Entity name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Protocols owned/affiliated
    #[serde(default)]
    pub protocols: Vec<String>,
    /// Total TVL
    pub tvl: Option<f64>,
}

/// Protocol treasury data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Treasury {
    /// Protocol name
    pub name: String,
    /// Token holdings
    #[serde(default)]
    pub tokens: HashMap<String, TreasuryToken>,
    /// Total value in USD
    pub total_value_usd: Option<f64>,
    /// Own token value
    pub own_token_value: Option<f64>,
    /// Other token value
    pub other_token_value: Option<f64>,
    /// Stablecoin value
    pub stablecoin_value: Option<f64>,
}

/// Treasury token holding
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreasuryToken {
    /// Token symbol
    pub symbol: Option<String>,
    /// Amount held
    pub amount: Option<f64>,
    /// Value in USD
    pub value_usd: Option<f64>,
}

/// Hack/exploit data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hack {
    /// Protocol name
    pub name: String,
    /// Project name
    pub project: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// Date of hack
    pub date: String,
    /// Amount lost in USD
    pub amount: Option<f64>,
    /// Hack technique
    pub technique: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Link to more info
    pub link: Option<String>,
    /// Amount returned
    pub returned_amount: Option<f64>,
}

/// Funding raise data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Raise {
    /// Protocol name
    pub name: String,
    /// Round type (Seed, Series A, etc.)
    pub round: Option<String>,
    /// Amount raised in USD
    pub amount: Option<f64>,
    /// Date of raise
    pub date: Option<String>,
    /// Lead investors
    #[serde(default)]
    pub lead_investors: Vec<String>,
    /// All investors
    #[serde(default)]
    pub investors: Vec<String>,
    /// Valuation
    pub valuation: Option<f64>,
    /// Chains
    #[serde(default)]
    pub chains: Vec<String>,
    /// Category
    pub category: Option<String>,
    /// Source link
    pub source: Option<String>,
}

/// Historical liquidity data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityData {
    /// Token symbol
    pub symbol: String,
    /// Historical data points
    #[serde(default)]
    pub data: Vec<LiquidityPoint>,
}

/// Liquidity data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityPoint {
    /// Unix timestamp
    pub date: u64,
    /// Liquidity in USD
    pub liquidity: f64,
}

/// Token protocols response (shows which protocols hold a token)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenProtocols {
    /// Token symbol
    pub symbol: String,
    /// Protocols holding this token
    #[serde(default)]
    pub protocols: Vec<TokenProtocolEntry>,
}

/// Protocol entry for token holders
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenProtocolEntry {
    /// Protocol name
    pub name: String,
    /// Amount held
    pub amount: Option<f64>,
    /// Value in USD
    pub value_usd: Option<f64>,
}

/// Protocol inflows data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolInflows {
    /// Protocol name
    pub name: String,
    /// Date
    pub date: String,
    /// Inflow in USD
    pub inflow_usd: Option<f64>,
    /// Outflow in USD
    pub outflow_usd: Option<f64>,
    /// Net flow
    pub net_flow_usd: Option<f64>,
}

/// Chain assets breakdown
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainAssets {
    /// Chain name
    pub chain: String,
    /// Asset breakdown
    #[serde(default)]
    pub assets: HashMap<String, f64>,
    /// Total value
    pub total: Option<f64>,
}
