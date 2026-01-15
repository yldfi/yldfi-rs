//! Types for the Market Data API

use serde::{Deserialize, Serialize};

/// Top token data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopToken {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Token decimals
    pub token_decimals: Option<u8>,
    /// Price USD
    pub price_usd: Option<f64>,
    /// Price 24h change percentage
    pub price_24h_percent_change: Option<f64>,
    /// Price 7d change percentage
    pub price_7d_percent_change: Option<f64>,
    /// Market cap USD
    pub market_cap_usd: Option<f64>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
    /// Volume change 24h percentage
    pub volume_change_24h: Option<f64>,
}

/// Top mover (gainer/loser)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopMover {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Price USD
    pub price_usd: Option<f64>,
    /// Price change percentage
    pub price_percent_change: Option<f64>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
}

/// Top NFT collection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopNftCollection {
    /// Collection address
    pub collection_address: Option<String>,
    /// Collection title
    pub collection_title: Option<String>,
    /// Collection image
    pub collection_image: Option<String>,
    /// Floor price USD
    pub floor_price_usd: Option<f64>,
    /// Floor price 24h change percentage
    pub floor_price_24hr_percent_change: Option<f64>,
    /// Volume 24h USD
    pub volume_usd: Option<f64>,
    /// Volume 24h change percentage
    pub volume_24hr_percent_change: Option<f64>,
    /// Average price USD
    pub average_price_usd: Option<f64>,
}

/// Global market cap data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalMarketCap {
    /// Total market cap USD
    pub total_market_cap_usd: Option<f64>,
    /// Market cap change 24h percentage
    pub market_cap_change_24h: Option<f64>,
}

/// Global volume data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalVolume {
    /// Total volume 24h USD
    pub total_volume_24h_usd: Option<f64>,
    /// Volume change 24h percentage
    pub volume_change_24h: Option<f64>,
}

/// Market data response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataResponse<T> {
    /// Page
    pub page: Option<i32>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    pub result: Vec<T>,
}
