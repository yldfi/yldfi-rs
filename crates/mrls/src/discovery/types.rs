//! Types for the Discovery API

use serde::{Deserialize, Serialize};

/// Discovered token
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredToken {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Token decimals
    pub token_decimals: Option<u8>,
    /// Chain
    pub chain: Option<String>,
    /// Price USD
    pub price_usd: Option<f64>,
    /// Price change 24h
    pub price_change_24h: Option<f64>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
    /// Market cap USD
    pub market_cap_usd: Option<f64>,
    /// Liquidity USD
    pub liquidity_usd: Option<f64>,
    /// Fully diluted valuation
    pub fully_diluted_valuation: Option<f64>,
    /// Total supply
    pub total_supply: Option<String>,
    /// Max supply
    pub max_supply: Option<String>,
    /// Holders count
    pub holders_count: Option<i64>,
    /// Created at
    pub created_at: Option<String>,
    /// Security score
    pub security_score: Option<i32>,
    /// On chain strength index
    pub on_chain_strength_index: Option<f64>,
}

/// Token discovery filter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveryFilter {
    /// Minimum market cap
    pub min_market_cap: Option<f64>,
    /// Maximum market cap
    pub max_market_cap: Option<f64>,
    /// Minimum liquidity
    pub min_liquidity: Option<f64>,
    /// Maximum liquidity
    pub max_liquidity: Option<f64>,
    /// Minimum volume 24h
    pub min_volume_24h: Option<f64>,
    /// Maximum volume 24h
    pub max_volume_24h: Option<f64>,
    /// Minimum holders
    pub min_holders: Option<i64>,
    /// Minimum security score
    pub min_security_score: Option<i32>,
    /// Chains to filter
    pub chains: Option<Vec<String>>,
}

/// Token analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAnalytics {
    /// Token address
    pub token_address: Option<String>,
    /// Buyers count
    pub total_buyers: Option<i64>,
    /// Sellers count
    pub total_sellers: Option<i64>,
    /// Net buyers (buyers - sellers)
    pub net_buyers: Option<i64>,
    /// Buy volume USD
    pub buy_volume_usd: Option<f64>,
    /// Sell volume USD
    pub sell_volume_usd: Option<f64>,
    /// Experienced buyers count
    pub experienced_buyers: Option<i64>,
}

/// Token score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenScore {
    /// Token address
    pub token_address: Option<String>,
    /// Security score (0-100)
    pub security_score: Option<i32>,
    /// On chain strength index
    pub on_chain_strength_index: Option<f64>,
    /// Is verified
    pub is_verified: Option<bool>,
    /// Is possible spam
    pub is_possible_spam: Option<bool>,
    /// Risk flags
    pub risk_flags: Option<Vec<String>>,
}

/// Discovery response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    /// Cursor
    pub cursor: Option<String>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    pub result: Vec<DiscoveredToken>,
}
