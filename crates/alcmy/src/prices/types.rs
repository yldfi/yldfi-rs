//! Types for the Prices API

use serde::{Deserialize, Serialize};

/// Token price with currency information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPrice {
    /// Token symbol (for symbol-based queries)
    pub symbol: Option<String>,
    /// Network name (for address-based queries)
    pub network: Option<String>,
    /// Token contract address (for address-based queries)
    pub address: Option<String>,
    /// Price in various currencies
    pub prices: Vec<PriceEntry>,
    /// Error if price couldn't be fetched
    pub error: Option<String>,
}

/// Price in a specific currency
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceEntry {
    /// Currency code (e.g., "usd", "eth")
    pub currency: String,
    /// Price value
    pub value: String,
    /// Last updated timestamp
    pub last_updated_at: Option<String>,
}

/// Response for token prices by symbol
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPricesBySymbolResponse {
    pub data: Vec<TokenPrice>,
}

/// Response for token prices by address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPricesByAddressResponse {
    pub data: Vec<TokenPrice>,
}

/// Token address for price lookup
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAddress {
    /// Network name (e.g., "eth-mainnet")
    pub network: String,
    /// Token contract address
    pub address: String,
}

/// Request body for token prices by address
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPricesByAddressRequest {
    pub addresses: Vec<TokenAddress>,
}

/// Historical price data point
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalPricePoint {
    /// Timestamp for this data point
    pub timestamp: String,
    /// Price value
    pub value: String,
}

/// Historical price response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalPriceResponse {
    /// Token symbol (if queried by symbol)
    pub symbol: Option<String>,
    /// Network name (if queried by address)
    pub network: Option<String>,
    /// Token address (if queried by address)
    pub address: Option<String>,
    /// Currency of the prices
    pub currency: String,
    /// Historical price data points
    pub data: Vec<HistoricalPricePoint>,
    /// Error if prices couldn't be fetched
    pub error: Option<String>,
}

/// Time interval for historical prices
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HistoricalInterval {
    FiveMinutes,
    OneHour,
    OneDay,
}

impl HistoricalInterval {
    #[must_use] 
    pub fn as_str(&self) -> &'static str {
        match self {
            HistoricalInterval::FiveMinutes => "5m",
            HistoricalInterval::OneHour => "1h",
            HistoricalInterval::OneDay => "1d",
        }
    }
}

/// Request for historical token prices
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoricalPriceRequest {
    /// Token symbol (mutually exclusive with network+address)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Network name (required with address)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Token contract address (required with network)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    /// Start timestamp (ISO 8601 or Unix)
    pub start_time: String,
    /// End timestamp (ISO 8601 or Unix)
    pub end_time: String,
    /// Time interval between data points
    pub interval: String,
}
