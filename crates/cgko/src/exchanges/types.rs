//! Types for exchanges endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Exchange list item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeListItem {
    pub id: String,
    pub name: String,
}

/// Exchange data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Exchange {
    pub id: Option<String>,
    pub name: Option<String>,
    pub year_established: Option<u32>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub image: Option<String>,
    pub has_trading_incentive: Option<bool>,
    pub trust_score: Option<u32>,
    pub trust_score_rank: Option<u32>,
    pub trade_volume_24h_btc: Option<f64>,
    pub trade_volume_24h_btc_normalized: Option<f64>,
}

/// Exchange tickers response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeTickers {
    pub name: String,
    pub tickers: Vec<ExchangeTicker>,
}

/// Exchange ticker
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeTicker {
    pub base: String,
    pub target: String,
    pub market: Option<TickerMarket>,
    pub last: Option<f64>,
    pub volume: Option<f64>,
    pub converted_last: Option<HashMap<String, f64>>,
    pub converted_volume: Option<HashMap<String, f64>>,
    pub trust_score: Option<String>,
    pub bid_ask_spread_percentage: Option<f64>,
    pub timestamp: Option<String>,
    pub last_traded_at: Option<String>,
    pub last_fetch_at: Option<String>,
    pub is_anomaly: Option<bool>,
    pub is_stale: Option<bool>,
    pub trade_url: Option<String>,
    pub coin_id: Option<String>,
    pub target_coin_id: Option<String>,
}

/// Ticker market
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickerMarket {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub has_trading_incentive: Option<bool>,
}

/// Volume chart data point `[timestamp_ms, volume_in_btc]`
///
/// The API sends the volume as a decimal string (and sometimes the timestamp
/// as a float); both string and number volumes are accepted.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct VolumeChartPoint(pub f64, pub f64);

impl<'de> Deserialize<'de> for VolumeChartPoint {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        // Through `Value`, not an untagged enum: untagged numbers break when
        // serde_json's `arbitrary_precision` feature is enabled in the build.
        let (timestamp, volume): (f64, serde_json::Value) = Deserialize::deserialize(deserializer)?;
        let volume = match &volume {
            serde_json::Value::Number(n) => n.as_f64(),
            serde_json::Value::String(s) => s.parse().ok(),
            _ => None,
        }
        .ok_or_else(|| serde::de::Error::custom(format!("invalid volume: {volume}")))?;
        Ok(Self(timestamp, volume))
    }
}

/// Volume chart data
pub type VolumeChart = Vec<VolumeChartPoint>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn volume_chart_accepts_string_volumes() {
        let json = r#"[[1779719400000, "76327.68894"], [1767282600000.0, "100719.03"], [1, 2.5]]"#;
        let chart: VolumeChart = serde_json::from_str(json).unwrap();
        assert_eq!(chart[0], VolumeChartPoint(1_779_719_400_000.0, 76327.68894));
        assert_eq!(chart[1].1, 100_719.03);
        assert_eq!(chart[2], VolumeChartPoint(1.0, 2.5));
    }
}
