//! Types for the Discovery API

use serde::{Deserialize, Serialize};

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
