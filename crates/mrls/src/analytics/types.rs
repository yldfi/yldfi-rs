//! Types for the Token Analytics API

use serde::{Deserialize, Serialize};

/// Token analytics timeseries data point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDataPoint {
    /// Timestamp
    pub timestamp: Option<String>,
    /// Total buyers
    pub total_buyers: Option<i64>,
    /// Total sellers
    pub total_sellers: Option<i64>,
    /// Net buyers (buyers - sellers)
    pub net_buyers: Option<i64>,
    /// Buy volume USD
    pub buy_volume_usd: Option<f64>,
    /// Sell volume USD
    pub sell_volume_usd: Option<f64>,
    /// Total volume USD
    pub total_volume_usd: Option<f64>,
}

/// Token analytics timeseries response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTimeseries {
    /// Token address
    pub token_address: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// Data points
    pub data: Vec<AnalyticsDataPoint>,
}

/// Request for batch token analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyticsRequest {
    /// Token addresses to fetch analytics for
    pub tokens: Vec<TokenAnalyticsInput>,
}

/// Token analytics input for batch requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenAnalyticsInput {
    /// Token address
    pub token_address: String,
    /// Chain (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
}

/// Batch token analytics result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchAnalyticsResult {
    /// Token address
    pub token_address: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// Total buyers
    pub total_buyers: Option<i64>,
    /// Total sellers
    pub total_sellers: Option<i64>,
    /// Net buyers
    pub net_buyers: Option<i64>,
    /// Buy volume USD
    pub buy_volume_usd: Option<f64>,
    /// Sell volume USD
    pub sell_volume_usd: Option<f64>,
    /// Experienced buyers count
    pub experienced_buyers: Option<i64>,
}

/// Request for analytics timeseries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTimeseriesRequest {
    /// Token addresses to fetch timeseries for
    pub tokens: Vec<TokenAnalyticsInput>,
    /// Timeframe (e.g., "1h", "4h", "1d")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
    /// From date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,
    /// To date
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
}
