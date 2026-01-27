//! Types for TVL and protocol data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Protocol summary with TVL data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Protocol {
    /// Protocol ID
    pub id: String,
    /// Protocol name
    pub name: String,
    /// URL-friendly slug
    pub slug: String,
    /// Protocol symbol/ticker
    pub symbol: Option<String>,
    /// Chain the protocol is deployed on (or "Multi-Chain")
    pub chain: Option<String>,
    /// All chains the protocol is deployed on
    #[serde(default)]
    pub chains: Vec<String>,
    /// Current total TVL in USD
    pub tvl: Option<f64>,
    /// TVL change in last 1 day (percentage)
    pub change_1d: Option<f64>,
    /// TVL change in last 7 days (percentage)
    pub change_7d: Option<f64>,
    /// TVL change in last 1 hour (percentage)
    pub change_1h: Option<f64>,
    /// Market cap in USD
    pub mcap: Option<f64>,
    /// Staking TVL
    pub staking: Option<f64>,
    /// Protocol category
    pub category: Option<String>,
    /// Protocol logo URL
    pub logo: Option<String>,
    /// Protocol URL
    pub url: Option<String>,
    /// Protocol description
    pub description: Option<String>,
    /// Twitter handle
    pub twitter: Option<String>,
    /// GitHub organization
    pub github: Option<Vec<String>>,
    /// Audit links
    pub audit_links: Option<Vec<String>>,
    /// Whether the protocol is listed on `DefiLlama`
    pub listed_at: Option<u64>,
    /// Parent protocol (for forks/variants)
    pub parent_protocol: Option<String>,
    /// Forked from
    pub forked_from: Option<Vec<String>>,
    /// Oracle used
    pub oracles: Option<Vec<String>>,
    /// Governance token address
    pub governance_id: Option<Vec<String>>,
    /// Treasury address
    pub treasury: Option<String>,
    /// Additional chain-specific TVL breakdown
    #[serde(default, rename = "chainTvls")]
    pub chain_tvls: HashMap<String, f64>,
}

/// Detailed protocol data with historical TVL
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolDetail {
    /// Protocol ID
    pub id: String,
    /// Protocol name
    pub name: String,
    /// URL-friendly slug
    pub slug: String,
    /// Protocol symbol/ticker
    pub symbol: Option<String>,
    /// Chain the protocol is deployed on
    pub chain: Option<String>,
    /// All chains the protocol is deployed on
    #[serde(default)]
    pub chains: Vec<String>,
    /// Current total TVL in USD
    pub tvl: Option<f64>,
    /// Protocol category
    pub category: Option<String>,
    /// Protocol logo URL
    pub logo: Option<String>,
    /// Protocol URL
    pub url: Option<String>,
    /// Protocol description
    pub description: Option<String>,
    /// Twitter handle
    pub twitter: Option<String>,
    /// GitHub organization
    #[serde(default)]
    pub github: Vec<String>,
    /// Audit links
    #[serde(default)]
    pub audit_links: Vec<String>,
    /// Historical TVL data points
    #[serde(default)]
    pub tvl_list: Vec<TvlDataPoint>,
    /// Chain-specific TVL breakdown
    #[serde(default, rename = "chainTvls")]
    pub chain_tvls: HashMap<String, ChainTvl>,
    /// Current chain TVLs (simple)
    #[serde(default, rename = "currentChainTvls")]
    pub current_chain_tvls: HashMap<String, f64>,
    /// Token breakdown
    #[serde(default)]
    pub tokens: Vec<TokenBreakdown>,
    /// Methodology description
    pub methodology: Option<String>,
    /// Module path in `DefiLlama` adapters
    pub module: Option<String>,
    /// Treasury info
    pub treasury: Option<String>,
    /// Market cap
    pub mcap: Option<f64>,
}

/// Historical TVL data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TvlDataPoint {
    /// Unix timestamp
    pub date: u64,
    /// Total TVL in USD
    #[serde(rename = "totalLiquidityUSD")]
    pub total_liquidity_usd: f64,
}

/// Chain-specific TVL data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainTvl {
    /// Historical TVL data for this chain
    #[serde(default)]
    pub tvl: Vec<TvlDataPoint>,
    /// Token breakdown for this chain
    #[serde(default)]
    pub tokens: Vec<TokenBreakdown>,
    /// Tokens in USD
    #[serde(default)]
    pub tokens_in_usd: Vec<TokenBreakdown>,
}

/// Token breakdown in TVL
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenBreakdown {
    /// Unix timestamp
    pub date: u64,
    /// Token amounts
    pub tokens: HashMap<String, f64>,
}

/// Chain TVL summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    /// Chain name
    pub name: Option<String>,
    /// Alternative: `gecko_id` as name in some responses
    pub gecko_id: Option<String>,
    /// Chain CMC ID
    pub cmc_id: Option<String>,
    /// Total TVL
    pub tvl: f64,
    /// Token symbol
    pub token_symbol: Option<String>,
}

/// Historical chain TVL data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalChainTvl {
    /// Unix timestamp
    pub date: u64,
    /// Total TVL in USD
    pub tvl: f64,
}

/// Response containing all chains TVL
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainsResponse(pub Vec<Chain>);

/// Historical TVL for a specific chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainHistoricalTvl(pub Vec<HistoricalChainTvl>);

/// Token protocol usage (Pro)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenProtocol {
    /// Protocol name
    pub name: String,
    /// Protocol category
    pub category: Option<String>,
    /// Token amounts by chain/identifier in USD
    #[serde(default)]
    pub amount_usd: HashMap<String, f64>,
}

/// Protocol capital flows (Pro)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolInflows {
    /// Net outflows in USD (negative means outflow)
    pub outflows: Option<f64>,
    /// Token state at old timestamp
    pub old_tokens: Option<TokenSnapshot>,
    /// Current token state
    pub current_tokens: Option<TokenSnapshot>,
}

/// Token snapshot at a point in time
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenSnapshot {
    /// Unix timestamp
    pub date: u64,
    /// Token amounts
    pub tvl: HashMap<String, f64>,
}

/// Chain assets breakdown (Pro)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainAssets(pub HashMap<String, ChainAssetBreakdown>);

/// Asset breakdown for a chain
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChainAssetBreakdown {
    /// Canonical (native) assets
    pub canonical: Option<AssetCategory>,
    /// Native assets
    pub native: Option<AssetCategory>,
    /// Bridged/third-party assets
    #[serde(rename = "thirdParty")]
    pub third_party: Option<AssetCategory>,
    /// Own tokens
    #[serde(rename = "ownTokens")]
    pub own_tokens: Option<AssetCategory>,
}

/// Asset category with total and breakdown
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetCategory {
    /// Total value as string (to preserve precision)
    pub total: String,
    /// Token breakdown
    #[serde(default)]
    pub breakdown: HashMap<String, String>,
}
