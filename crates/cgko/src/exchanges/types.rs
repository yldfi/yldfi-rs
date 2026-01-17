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

/// Volume chart data point [timestamp, volume]
pub type VolumeChart = Vec<[f64; 2]>;
