//! Types for coins endpoints

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Coin list item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinListItem {
    pub id: String,
    pub symbol: String,
    pub name: String,
    #[serde(default)]
    pub platforms: HashMap<String, Option<String>>,
}

/// Coin market data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinMarket {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub image: Option<String>,
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub market_cap_rank: Option<u32>,
    pub fully_diluted_valuation: Option<f64>,
    pub total_volume: Option<f64>,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub market_cap_change_percentage_24h: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
    pub ath: Option<f64>,
    pub ath_change_percentage: Option<f64>,
    pub ath_date: Option<String>,
    pub atl: Option<f64>,
    pub atl_change_percentage: Option<f64>,
    pub atl_date: Option<String>,
    pub last_updated: Option<String>,
    #[serde(default)]
    pub sparkline_in_7d: Option<SparklineData>,
}

/// Sparkline data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SparklineData {
    pub price: Vec<f64>,
}

/// Full coin data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinData {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub web_slug: Option<String>,
    pub categories: Option<Vec<String>>,
    pub description: Option<HashMap<String, String>>,
    pub links: Option<serde_json::Value>,
    pub image: Option<CoinImage>,
    pub genesis_date: Option<String>,
    pub market_cap_rank: Option<u32>,
    pub market_data: Option<MarketData>,
    pub last_updated: Option<String>,
}

/// Coin images
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinImage {
    pub thumb: Option<String>,
    pub small: Option<String>,
    pub large: Option<String>,
}

/// Market data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketData {
    pub current_price: Option<HashMap<String, f64>>,
    pub market_cap: Option<HashMap<String, f64>>,
    pub total_volume: Option<HashMap<String, f64>>,
    pub fully_diluted_valuation: Option<HashMap<String, f64>>,
    pub high_24h: Option<HashMap<String, f64>>,
    pub low_24h: Option<HashMap<String, f64>>,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub price_change_percentage_7d: Option<f64>,
    pub price_change_percentage_30d: Option<f64>,
    pub circulating_supply: Option<f64>,
    pub total_supply: Option<f64>,
    pub max_supply: Option<f64>,
}

/// Market chart data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketChart {
    pub prices: Vec<(f64, f64)>,
    pub market_caps: Vec<(f64, f64)>,
    pub total_volumes: Vec<(f64, f64)>,
}

/// OHLC data point [timestamp, open, high, low, close]
pub type OhlcData = Vec<[f64; 5]>;

/// Coin historical data (for a specific date)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinHistory {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub image: Option<CoinImage>,
    pub market_data: Option<HistoricalMarketData>,
    pub community_data: Option<serde_json::Value>,
    pub developer_data: Option<serde_json::Value>,
    pub public_interest_stats: Option<serde_json::Value>,
}

/// Historical market data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalMarketData {
    pub current_price: Option<HashMap<String, f64>>,
    pub market_cap: Option<HashMap<String, f64>>,
    pub total_volume: Option<HashMap<String, f64>>,
}

/// Top gainers/losers item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopMoverItem {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub image: Option<String>,
    pub market_cap_rank: Option<u32>,
    pub usd: Option<f64>,
    pub usd_24h_vol: Option<f64>,
    pub usd_24h_change: Option<f64>,
}

/// Top gainers/losers response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TopMoversResponse {
    pub top_gainers: Vec<TopMoverItem>,
    pub top_losers: Vec<TopMoverItem>,
}

/// Recently added coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RecentlyAddedCoin {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub activated_at: Option<u64>,
}

/// Coin tickers response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinTickers {
    pub name: String,
    pub tickers: Vec<Ticker>,
}

/// Ticker data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ticker {
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

/// Ticker market info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TickerMarket {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub has_trading_incentive: Option<bool>,
}

/// Coin contract data (coin data by contract address)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinContractData {
    pub id: String,
    pub symbol: String,
    pub name: String,
    pub asset_platform_id: Option<String>,
    pub contract_address: Option<String>,
    pub image: Option<CoinImage>,
    pub market_data: Option<MarketData>,
    pub last_updated: Option<String>,
}

/// Options for markets query
#[derive(Debug, Clone, Default)]
pub struct MarketsOptions {
    pub order: Option<String>,
    pub per_page: Option<u32>,
    pub page: Option<u32>,
    pub sparkline: bool,
    pub price_change_percentage: Option<String>,
    pub category: Option<String>,
}

impl MarketsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref o) = self.order {
            params.push(format!("order={o}"));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={pp}"));
        }
        if let Some(p) = self.page {
            params.push(format!("page={p}"));
        }
        if self.sparkline {
            params.push("sparkline=true".to_string());
        }
        if let Some(ref pcp) = self.price_change_percentage {
            params.push(format!("price_change_percentage={pcp}"));
        }
        if let Some(ref c) = self.category {
            params.push(format!("category={c}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("&{}", params.join("&"))
        }
    }
}

/// Supply chart data (circulating or total supply over time)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupplyChart {
    /// Vec of [timestamp, supply]
    pub circulating_supply: Option<Vec<(f64, f64)>>,
    pub total_supply: Option<Vec<(f64, f64)>>,
}
