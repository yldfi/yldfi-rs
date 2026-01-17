//! Types for token emissions/unlocks data (Pro)

use serde::{Deserialize, Serialize};

/// Token emissions summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionsSummary {
    /// Protocol name
    pub name: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token gecko ID
    pub gecko_id: Option<String>,
    /// Next unlock date
    pub next_unlock_date: Option<String>,
    /// Next unlock amount in USD
    pub next_unlock_usd: Option<f64>,
    /// Next unlock percent of supply
    pub next_unlock_percent: Option<f64>,
    /// Total locked
    pub total_locked: Option<f64>,
    /// Total locked in USD
    pub total_locked_usd: Option<f64>,
    /// Max supply
    pub max_supply: Option<f64>,
    /// Circulating supply
    pub circulating_supply: Option<f64>,
}

/// Detailed emission schedule
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmissionDetail {
    /// Protocol name
    pub name: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token gecko ID
    pub gecko_id: Option<String>,
    /// Max supply
    pub max_supply: Option<f64>,
    /// Circulating supply
    pub circulating_supply: Option<f64>,
    /// Unlock events
    #[serde(default)]
    pub events: Vec<UnlockEvent>,
    /// Token allocation
    #[serde(default)]
    pub allocations: Vec<TokenAllocation>,
}

/// Token unlock event
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockEvent {
    /// Event date
    pub date: String,
    /// Unlock type (cliff, linear, etc.)
    pub unlock_type: Option<String>,
    /// Amount unlocked
    pub amount: Option<f64>,
    /// Percent of total supply
    pub percent: Option<f64>,
    /// Category (team, investors, etc.)
    pub category: Option<String>,
    /// Description
    pub description: Option<String>,
}

/// Token allocation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAllocation {
    /// Allocation category
    pub category: String,
    /// Allocation amount
    pub amount: Option<f64>,
    /// Percent of total
    pub percent: Option<f64>,
    /// Vesting schedule
    pub vesting: Option<String>,
}
