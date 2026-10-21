//! Types for ETF data (Pro)

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

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

/// FDV performance for one day, across token categories
///
/// Each response item is `{"date": <unix>, "<Category>": <value>, ...}`.
#[derive(Debug, Clone, Default, Serialize)]
pub struct FdvPerformance {
    /// Day (Unix timestamp)
    pub date: Option<u64>,
    /// Performance keyed by category name (e.g. "Meme", "Oracle")
    pub categories: HashMap<String, f64>,
}

// Hand-written rather than `#[serde(flatten)]`: flattened numbers fail to
// deserialize with serde_json's `arbitrary_precision` (see `crate::de`).
impl<'de> Deserialize<'de> for FdvPerformance {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let date = map
            .remove("date")
            .as_ref()
            .and_then(crate::de::value_to_u64);
        let categories = map
            .into_iter()
            .filter_map(|(k, v)| crate::de::value_to_f64(&v).map(|n| (k, n)))
            .collect();
        Ok(Self { date, categories })
    }
}

/// ETF daily flows data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct EtfFlow {
    /// `CoinGecko` ID (e.g., "bitcoin", "ethereum")
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fdv_performance_splits_date_and_categories() {
        let json = r#"[{"date":1700000000,"Meme":1.25,"Oracle":-0.5,"Bitcoin":null}]"#;
        let p: Vec<FdvPerformance> = serde_json::from_str(json).unwrap();
        assert_eq!(p[0].date, Some(1_700_000_000));
        assert_eq!(p[0].categories.len(), 2);
        assert_eq!(p[0].categories["Meme"], 1.25);
        assert_eq!(p[0].categories["Oracle"], -0.5);
    }
}
