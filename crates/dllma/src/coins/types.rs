//! Types for coin/price data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token identifier in the format `chain:address`
///
/// Examples:
/// - `ethereum:0xdF574c24545E5FfEcb9a659c229253D4111d87e1` (ERC-20 token)
/// - `coingecko:ethereum` (by `CoinGecko` ID)
/// - `ethereum:0x0000000000000000000000000000000000000000` (native ETH)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    /// Chain identifier (e.g., "ethereum", "bsc", "polygon", "coingecko")
    pub chain: String,
    /// Token address or identifier
    pub address: String,
}

impl Token {
    /// Create a new token identifier
    pub fn new(chain: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            chain: chain.into(),
            address: address.into(),
        }
    }

    /// Create an Ethereum mainnet token
    pub fn ethereum(address: impl Into<String>) -> Self {
        Self::new("ethereum", address)
    }

    /// Create a BSC token
    pub fn bsc(address: impl Into<String>) -> Self {
        Self::new("bsc", address)
    }

    /// Create a Polygon token
    pub fn polygon(address: impl Into<String>) -> Self {
        Self::new("polygon", address)
    }

    /// Create an Arbitrum token
    pub fn arbitrum(address: impl Into<String>) -> Self {
        Self::new("arbitrum", address)
    }

    /// Create a token by `CoinGecko` ID
    pub fn coingecko(id: impl Into<String>) -> Self {
        Self::new("coingecko", id)
    }

    /// Format as `chain:address` string
    #[must_use]
    pub fn format(&self) -> String {
        format!("{}:{}", self.chain, self.address)
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.chain, self.address)
    }
}

/// Price data for a single coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinPrice {
    /// Price in USD
    pub price: f64,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Unix timestamp of the price
    pub timestamp: Option<u64>,
    /// Confidence score (0-1)
    pub confidence: Option<f64>,
}

/// Response from current prices endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PricesResponse {
    /// Map of token ID to price data
    pub coins: HashMap<String, CoinPrice>,
}

/// Historical price query item
#[derive(Debug, Clone, Serialize)]
pub struct HistoricalPriceQuery {
    /// Token identifier (chain:address)
    pub coins: Vec<String>,
    /// Unix timestamp
    pub timestamp: u64,
}

/// Batch historical price request body
#[derive(Debug, Clone, Serialize)]
pub struct BatchHistoricalRequest {
    /// List of coin/timestamp pairs to query
    pub coins: HashMap<String, Vec<u64>>,
}

/// Price chart data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChartDataPoint {
    /// Unix timestamp
    pub timestamp: u64,
    /// Price in USD
    pub price: f64,
}

/// Response from price chart endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChartResponse {
    /// Map of token ID to price history
    pub coins: HashMap<String, ChartData>,
}

/// Price chart data for a single coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChartData {
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Price data points
    pub prices: Vec<ChartDataPoint>,
    /// Confidence score
    pub confidence: Option<f64>,
}

/// Price percentage change data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PricePercentage {
    /// Token symbol
    pub symbol: Option<String>,
    /// Price change percentage
    pub price: f64,
}

/// Response from percentage change endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PercentageResponse {
    /// Map of token ID to percentage data
    pub coins: HashMap<String, PricePercentage>,
}

/// First price record for a coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FirstPrice {
    /// First recorded price
    pub price: f64,
    /// Token symbol
    pub symbol: Option<String>,
    /// Unix timestamp of first price
    pub timestamp: u64,
}

/// Response from first price endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FirstPriceResponse {
    /// Map of token ID to first price data
    pub coins: HashMap<String, FirstPrice>,
}

/// Block number at a timestamp
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BlockResponse {
    /// Block height
    pub height: u64,
    /// Unix timestamp
    pub timestamp: u64,
}
