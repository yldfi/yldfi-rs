//! Types for the Token Analytics API

use serde::{Deserialize, Serialize};

/// Token analytics timeseries data point
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsDataPoint {
    /// Timestamp
    pub timestamp: Option<String>,
    /// Buy volume (USD)
    pub buy_volume: Option<f64>,
    /// Sell volume (USD)
    pub sell_volume: Option<f64>,
    /// Liquidity (USD)
    pub liquidity_usd: Option<f64>,
    /// Fully diluted valuation (USD)
    pub fully_diluted_valuation: Option<f64>,
}

/// Token analytics timeseries for one token
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyticsTimeseries {
    /// Token address
    pub token_address: Option<String>,
    /// Chain ID (e.g. "0x1")
    pub chain_id: Option<String>,
    /// Data points
    #[serde(default)]
    pub timeseries: Vec<AnalyticsDataPoint>,
}

/// Token analytics timeseries response envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTimeseriesResponse {
    /// One entry per requested token
    #[serde(default)]
    pub result: Vec<AnalyticsTimeseries>,
}

/// Request for batch token analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchAnalyticsRequest {
    /// Token addresses to fetch analytics for
    pub tokens: Vec<TokenAnalyticsInput>,
}

/// Token analytics input for batch requests
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAnalyticsInput {
    /// Token address
    pub token_address: String,
    /// Chain (e.g. "eth", "0x1")
    pub chain: String,
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

/// Request body for analytics timeseries
///
/// The timeframe is a query parameter; pass it to
/// [`AnalyticsApi::get_timeseries`](crate::AnalyticsApi::get_timeseries).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsTimeseriesRequest {
    /// Token addresses to fetch timeseries for
    pub tokens: Vec<TokenAnalyticsInput>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timeseries_request_body_is_camel_case() {
        let req = AnalyticsTimeseriesRequest {
            tokens: vec![TokenAnalyticsInput {
                token_address: "0xtoken".to_string(),
                chain: "eth".to_string(),
            }],
        };
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({"tokens": [{"tokenAddress": "0xtoken", "chain": "eth"}]})
        );
    }

    #[test]
    fn test_timeseries_response_envelope() {
        let json = r#"{
            "result": [{
                "chainId": "0x1",
                "tokenAddress": "0xtoken",
                "timeseries": [{
                    "timestamp": "2024-01-01T00:00:00Z",
                    "buyVolume": 10.5,
                    "sellVolume": 4.0,
                    "liquidityUsd": 1000.0,
                    "fullyDilutedValuation": 50000.0
                }]
            }]
        }"#;
        let resp: AnalyticsTimeseriesResponse = serde_json::from_str(json).unwrap();
        let t = &resp.result[0];
        assert_eq!(t.chain_id.as_deref(), Some("0x1"));
        assert_eq!(t.token_address.as_deref(), Some("0xtoken"));
        assert_eq!(t.timeseries[0].buy_volume, Some(10.5));
        assert_eq!(t.timeseries[0].fully_diluted_valuation, Some(50000.0));
    }
}
