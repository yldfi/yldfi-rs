//! Types for the Discovery API

use crate::de::string_or_f64;
use serde::{Deserialize, Serialize};

/// Values for the 5m / 1h / 6h / 24h windows
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WindowValues {
    /// Last 5 minutes
    #[serde(rename = "5m", default, deserialize_with = "string_or_f64")]
    pub m5: Option<f64>,
    /// Last hour
    #[serde(rename = "1h", default, deserialize_with = "string_or_f64")]
    pub h1: Option<f64>,
    /// Last 6 hours
    #[serde(rename = "6h", default, deserialize_with = "string_or_f64")]
    pub h6: Option<f64>,
    /// Last 24 hours
    #[serde(rename = "24h", default, deserialize_with = "string_or_f64")]
    pub h24: Option<f64>,
}

/// Token analytics (`GET /tokens/{address}/analytics`, also the item type of
/// the batch `POST /tokens/analytics`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAnalytics {
    /// Token address (not in the spec; kept if Moralis returns it)
    pub token_address: Option<String>,
    /// Chain ID (e.g. "0x1")
    pub chain_id: Option<String>,
    /// Category ID
    pub category_id: Option<String>,
    /// Buy volume (USD)
    pub total_buy_volume: Option<WindowValues>,
    /// Sell volume (USD)
    pub total_sell_volume: Option<WindowValues>,
    /// Buyers count
    pub total_buyers: Option<WindowValues>,
    /// Sellers count
    pub total_sellers: Option<WindowValues>,
    /// Buy transactions count
    pub total_buys: Option<WindowValues>,
    /// Sell transactions count
    pub total_sells: Option<WindowValues>,
    /// Unique wallets
    pub unique_wallets: Option<WindowValues>,
    /// Price change (percent)
    pub price_percent_change: Option<WindowValues>,
    /// USD price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_price: Option<f64>,
    /// Total liquidity (USD)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub total_liquidity: Option<f64>,
    /// Fully diluted valuation (USD)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub total_fully_diluted_valuation: Option<f64>,
}

/// Values for the score endpoint's 10m ... 30d windows
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScoreWindowValues {
    /// Last 10 minutes
    #[serde(rename = "10m", default, deserialize_with = "string_or_f64")]
    pub m10: Option<f64>,
    /// Last 30 minutes
    #[serde(rename = "30m", default, deserialize_with = "string_or_f64")]
    pub m30: Option<f64>,
    /// Last hour
    #[serde(rename = "1h", default, deserialize_with = "string_or_f64")]
    pub h1: Option<f64>,
    /// Last 4 hours
    #[serde(rename = "4h", default, deserialize_with = "string_or_f64")]
    pub h4: Option<f64>,
    /// Last 12 hours
    #[serde(rename = "12h", default, deserialize_with = "string_or_f64")]
    pub h12: Option<f64>,
    /// Last day
    #[serde(rename = "1d", default, deserialize_with = "string_or_f64")]
    pub d1: Option<f64>,
    /// Last 7 days
    #[serde(rename = "7d", default, deserialize_with = "string_or_f64")]
    pub d7: Option<f64>,
    /// Last 30 days
    #[serde(rename = "30d", default, deserialize_with = "string_or_f64")]
    pub d30: Option<f64>,
}

/// Supply metrics in a token score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenScoreSupply {
    /// Total supply (string or number)
    pub total: Option<serde_json::Value>,
    /// Share of supply held by the top 10 holders (percent)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub top10_percent: Option<f64>,
}

/// Metrics behind a token score
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenScoreMetrics {
    /// USD price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_price: Option<f64>,
    /// Liquidity (USD)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub liquidity_usd: Option<f64>,
    /// Volume (USD) per window
    pub volume_usd: Option<ScoreWindowValues>,
    /// Transaction count per window
    pub transactions: Option<ScoreWindowValues>,
    /// Supply metrics
    pub supply: Option<TokenScoreSupply>,
}

/// Token score (`GET /tokens/{address}/score`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenScore {
    /// Token address
    pub token_address: Option<String>,
    /// Chain ID (e.g. "0x1")
    pub chain_id: Option<String>,
    /// Score
    #[serde(default, deserialize_with = "string_or_f64")]
    pub score: Option<f64>,
    /// Last update timestamp
    pub updated_at: Option<String>,
    /// Metrics behind the score
    pub metrics: Option<TokenScoreMetrics>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_analytics_parses_window_objects() {
        let json = r#"{
            "chainId": "0x1",
            "categoryId": "test-category",
            "totalBuyVolume": {"5m": 1.5, "1h": 10, "6h": 60, "24h": 240},
            "totalBuyers": {"24h": 12},
            "pricePercentChange": {"1h": -0.5},
            "usdPrice": "1.25",
            "totalLiquidity": "1000",
            "totalFullyDilutedValuation": 5000
        }"#;
        let a: TokenAnalytics = serde_json::from_str(json).unwrap();
        assert_eq!(a.chain_id.as_deref(), Some("0x1"));
        assert_eq!(a.total_buy_volume.as_ref().unwrap().m5, Some(1.5));
        assert_eq!(a.total_buyers.as_ref().unwrap().h24, Some(12.0));
        assert_eq!(a.price_percent_change.as_ref().unwrap().h1, Some(-0.5));
        assert_eq!(a.usd_price, Some(1.25));
        assert_eq!(a.total_fully_diluted_valuation, Some(5000.0));
    }

    #[test]
    fn token_score_parses_metrics() {
        let json = r#"{
            "tokenAddress": "0x0000000000000000000000000000000000000001",
            "chainId": "0x1",
            "score": 87,
            "updatedAt": "2026-01-01T00:00:00Z",
            "metrics": {
                "usdPrice": 1.25,
                "liquidityUsd": 1000,
                "volumeUsd": {"1h": 10, "1d": 240, "30d": 7200},
                "transactions": {"10m": 3},
                "supply": {"total": "1000000", "top10Percent": 42.5}
            }
        }"#;
        let s: TokenScore = serde_json::from_str(json).unwrap();
        assert_eq!(s.score, Some(87.0));
        let m = s.metrics.unwrap();
        assert_eq!(m.volume_usd.as_ref().unwrap().d30, Some(7200.0));
        assert_eq!(m.transactions.as_ref().unwrap().m10, Some(3.0));
        assert_eq!(m.supply.unwrap().top10_percent, Some(42.5));
    }
}
