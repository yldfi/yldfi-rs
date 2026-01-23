//! Types for Pyth Hermes API responses

use serde::{Deserialize, Serialize};

/// Price data from Pyth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceData {
    /// Price value as a string (to preserve precision)
    pub price: String,
    /// Confidence interval
    pub conf: String,
    /// Exponent (price = price * 10^expo)
    pub expo: i32,
    /// Publish time (unix timestamp)
    pub publish_time: i64,
}

/// EMA (Exponential Moving Average) price data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmaPrice {
    /// EMA price value
    pub price: String,
    /// EMA confidence
    pub conf: String,
    /// Exponent
    pub expo: i32,
    /// Publish time
    pub publish_time: i64,
}

/// Parsed price feed with current and EMA prices
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPriceFeed {
    /// Feed ID
    pub id: String,
    /// Current price
    pub price: PriceData,
    /// EMA price
    #[serde(default)]
    pub ema_price: Option<EmaPrice>,
    /// Metadata from the price update response
    #[serde(default)]
    pub metadata: Option<PriceUpdateMetadata>,
}

/// Metadata from price update response (different from feed attributes)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceUpdateMetadata {
    /// Slot number
    #[serde(default)]
    pub slot: Option<u64>,
    /// Proof available time
    #[serde(default)]
    pub proof_available_time: Option<i64>,
    /// Previous publish time
    #[serde(default)]
    pub prev_publish_time: Option<i64>,
}

/// Response from /v2/updates/price/latest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestPriceResponse {
    /// Binary price update data (for on-chain use)
    pub binary: BinaryData,
    /// Parsed price feeds
    pub parsed: Vec<ParsedPriceFeed>,
}

/// Binary data for on-chain updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryData {
    /// Encoding type
    pub encoding: String,
    /// Encoded data
    pub data: Vec<String>,
}

/// Price feed ID entry from the feed list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceFeedId {
    /// Feed ID (hex string)
    pub id: String,
    /// Attributes
    #[serde(default)]
    pub attributes: PriceFeedAttributes,
}

/// Attributes for a price feed
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PriceFeedAttributes {
    /// Asset type
    #[serde(default)]
    pub asset_type: Option<String>,
    /// Base asset
    #[serde(default)]
    pub base: Option<String>,
    /// Quote asset
    #[serde(default)]
    pub quote: Option<String>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Generic symbol
    #[serde(default)]
    pub generic_symbol: Option<String>,
    /// Symbol
    #[serde(default)]
    pub symbol: Option<String>,
}

impl ParsedPriceFeed {
    /// Convert price to f64 using the exponent
    pub fn price_f64(&self) -> Option<f64> {
        let price: f64 = self.price.price.parse().ok()?;
        Some(price * 10f64.powi(self.price.expo))
    }

    /// Convert EMA price to f64 using the exponent
    pub fn ema_price_f64(&self) -> Option<f64> {
        let ema = self.ema_price.as_ref()?;
        let price: f64 = ema.price.parse().ok()?;
        Some(price * 10f64.powi(ema.expo))
    }

    /// Get confidence interval as f64
    pub fn confidence_f64(&self) -> Option<f64> {
        let conf: f64 = self.price.conf.parse().ok()?;
        Some(conf * 10f64.powi(self.price.expo))
    }

    /// Get EMA confidence interval as f64
    pub fn ema_confidence_f64(&self) -> Option<f64> {
        let ema = self.ema_price.as_ref()?;
        let conf: f64 = ema.conf.parse().ok()?;
        Some(conf * 10f64.powi(ema.expo))
    }

    /// Check if price is stale (older than max_age seconds)
    pub fn is_stale(&self, max_age_secs: i64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        now - self.price.publish_time > max_age_secs
    }
}
