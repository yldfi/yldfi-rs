//! Types for ETF data (Pro)

use serde::{Deserialize, Serialize};

/// ETF overview data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtfOverview {
    /// List of individual ETF data
    #[serde(default)]
    pub data: Vec<EtfData>,
    /// Total AUM across all ETFs
    pub total_aum: Option<f64>,
    /// Total daily flow
    pub total_flow: Option<f64>,
}

/// Individual ETF data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtfData {
    /// ETF ticker
    pub ticker: Option<String>,
    /// Issuer name
    pub issuer: Option<String>,
    /// Full ETF name
    pub name: Option<String>,
    /// Custodian
    pub custodian: Option<String>,
    /// Management fee percentage
    pub fee: Option<f64>,
    /// Assets under management
    pub aum: Option<f64>,
    /// Daily flow
    pub flow: Option<f64>,
    /// Trading volume
    pub volume: Option<f64>,
}

/// Historical ETF data point
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtfHistoryPoint {
    /// Date string
    pub date: Option<String>,
    /// Timestamp
    pub timestamp: Option<u64>,
    /// Total AUM
    pub total_aum: Option<f64>,
    /// Total flow
    pub total_flow: Option<f64>,
    /// Individual ETF data for this date
    #[serde(default)]
    pub etfs: Vec<EtfHistoryData>,
}

/// ETF data in history response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtfHistoryData {
    /// ETF ticker
    pub ticker: Option<String>,
    /// AUM for this date
    pub aum: Option<f64>,
    /// Flow for this date
    pub flow: Option<f64>,
}

/// FDV performance data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FdvPerformance {
    /// Category name
    pub name: Option<String>,
    /// Performance value (as decimal, e.g., 0.05 = 5%)
    pub performance: Option<f64>,
    /// Date/timestamp
    pub date: Option<String>,
}

/// ETF daily flows data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EtfFlow {
    /// CoinGecko ID (e.g., "bitcoin", "ethereum")
    pub gecko_id: Option<String>,
    /// Date (e.g., "2024-01-15")
    pub day: Option<String>,
    /// Sum of all USD flows per asset for the day
    pub total_flow_usd: Option<f64>,
}

/// ETF snapshot data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EtfSnapshot {
    /// ETF ticker symbol
    pub ticker: Option<String>,
    /// Current timestamp
    pub timestamp: Option<u64>,
    /// Asset name (e.g., "BTC", "ETH")
    pub asset: Option<String>,
    /// ETF issuer name
    pub issuer: Option<String>,
    /// Full ETF name
    pub etf_name: Option<String>,
    /// Custodian name
    pub custodian: Option<String>,
    /// Percentage fee
    pub pct_fee: Option<f64>,
    /// ETF URL
    pub url: Option<String>,
    /// Net flows
    pub flows: Option<f64>,
    /// Assets under management
    pub aum: Option<f64>,
    /// Trading volume
    pub volume: Option<f64>,
}
