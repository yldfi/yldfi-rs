//! Data normalization types for aggregating responses from different API sources
//!
//! Each API service returns data in different formats. This module provides
//! normalized types that can be constructed from any source.

use serde::{Deserialize, Serialize};

/// Normalized price with consistent formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedPrice {
    /// Price in USD (always present)
    pub usd: f64,
    /// Price in native token (e.g., ETH)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub native: Option<f64>,
    /// Token decimals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    /// Confidence score (0.0 - 1.0, some sources provide this)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    /// 24h change percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub change_24h_pct: Option<f64>,
    /// Market cap in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub market_cap_usd: Option<f64>,
    /// 24h volume in USD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume_24h_usd: Option<f64>,
}

impl NormalizedPrice {
    /// Create a new normalized price with just USD value
    pub fn new(usd: f64) -> Self {
        Self {
            usd,
            native: None,
            decimals: None,
            confidence: None,
            change_24h_pct: None,
            market_cap_usd: None,
            volume_24h_usd: None,
        }
    }

    /// Builder: set native price
    #[must_use]
    pub fn with_native(mut self, native: f64) -> Self {
        self.native = Some(native);
        self
    }

    /// Builder: set decimals
    #[must_use]
    pub fn with_decimals(mut self, decimals: u8) -> Self {
        self.decimals = Some(decimals);
        self
    }

    /// Builder: set confidence
    #[must_use]
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = Some(confidence);
        self
    }

    /// Builder: set 24h change
    #[must_use]
    pub fn with_change_24h(mut self, change: f64) -> Self {
        self.change_24h_pct = Some(change);
        self
    }

    /// Builder: set market cap
    #[must_use]
    pub fn with_market_cap(mut self, market_cap: f64) -> Self {
        self.market_cap_usd = Some(market_cap);
        self
    }

    /// Builder: set volume
    #[must_use]
    pub fn with_volume(mut self, volume: f64) -> Self {
        self.volume_24h_usd = Some(volume);
        self
    }
}

/// Aggregated price statistics from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAggregation {
    /// Median USD price across all sources
    pub median_usd: f64,
    /// Mean USD price across all sources
    pub mean_usd: f64,
    /// Minimum USD price
    pub min_usd: f64,
    /// Maximum USD price
    pub max_usd: f64,
    /// Spread percentage: (max-min)/median * 100
    pub spread_pct: f64,
    /// True if spread < 1% (sources agree)
    pub sources_agreed: bool,
    /// Best source (lowest latency among agreed sources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_source: Option<String>,
}

impl PriceAggregation {
    /// Calculate aggregation from a list of prices
    ///
    /// Filters out NaN and infinite values before calculating statistics.
    /// Returns None if no valid prices remain after filtering.
    pub fn from_prices(prices: &[f64]) -> Option<Self> {
        // Filter out NaN and infinite values
        let mut sorted: Vec<f64> = prices.iter().copied().filter(|p| p.is_finite()).collect();

        if sorted.is_empty() {
            return None;
        }

        // Sort is now safe since we filtered out NaN values
        sorted.sort_by(|a, b| a.partial_cmp(b).expect("filtered values are finite"));

        let len = sorted.len();
        let median_usd = if len.is_multiple_of(2) {
            (sorted[len / 2 - 1] + sorted[len / 2]) / 2.0
        } else {
            sorted[len / 2]
        };

        let mean_usd = sorted.iter().sum::<f64>() / len as f64;
        // Use safer access pattern - unwrap_or provides fallback even though
        // we know the vec is non-empty, this protects against future refactoring
        let min_usd = sorted.first().copied().unwrap_or(0.0);
        let max_usd = sorted.last().copied().unwrap_or(0.0);

        let spread_pct = if median_usd > 0.0 {
            ((max_usd - min_usd) / median_usd) * 100.0
        } else {
            0.0
        };

        let sources_agreed = spread_pct < 1.0;

        Some(Self {
            median_usd,
            mean_usd,
            min_usd,
            max_usd,
            spread_pct,
            sources_agreed,
            best_source: None,
        })
    }

    /// Set the best source
    #[must_use]
    pub fn with_best_source(mut self, source: impl Into<String>) -> Self {
        self.best_source = Some(source.into());
        self
    }
}

/// Normalized token balance with consistent formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedBalance {
    /// Token contract address (checksummed)
    pub token_address: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Raw balance as string (full precision)
    pub balance_raw: String,
    /// Formatted balance (human readable)
    pub balance_formatted: f64,
    /// Token decimals
    pub decimals: u8,
    /// USD value (if available)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usd_value: Option<f64>,
    /// Token logo URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

impl NormalizedBalance {
    /// Create a new normalized balance
    pub fn new(
        token_address: impl Into<String>,
        symbol: impl Into<String>,
        balance_raw: impl Into<String>,
        decimals: u8,
    ) -> Self {
        let raw = balance_raw.into();
        let balance_formatted = parse_balance_formatted(&raw, decimals);
        Self {
            token_address: token_address.into(),
            symbol: symbol.into(),
            name: None,
            balance_raw: raw,
            balance_formatted,
            decimals,
            usd_value: None,
            logo_url: None,
        }
    }

    /// Builder: set name
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Builder: set USD value
    #[must_use]
    pub fn with_usd_value(mut self, usd_value: f64) -> Self {
        self.usd_value = Some(usd_value);
        self
    }

    /// Builder: set logo URL
    #[must_use]
    pub fn with_logo(mut self, logo_url: impl Into<String>) -> Self {
        self.logo_url = Some(logo_url.into());
        self
    }
}

/// Parse a raw balance string (decimal or hex) into a formatted f64
///
/// Returns 0.0 for invalid inputs (empty strings, malformed hex/decimal).
/// Note: This intentionally conflates invalid input with zero balance for simplicity.
fn parse_balance_formatted(raw: &str, decimals: u8) -> f64 {
    let raw = raw.trim();

    // Handle empty or "0x"/"0X" only strings
    if raw.is_empty() {
        return 0.0;
    }

    // Parse as integer
    let raw_int: u128 =
        if let Some(hex_part) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
            if hex_part.is_empty() {
                return 0.0;
            }
            u128::from_str_radix(hex_part, 16).unwrap_or(0)
        } else {
            raw.parse().unwrap_or(0)
        };

    // Prevent overflow: 10^39 exceeds u128::MAX, so cap decimals at 38
    // For decimals > 38, the divisor would overflow, so return 0.0
    if decimals > 38 {
        return 0.0;
    }

    // Convert to f64 with decimals
    let divisor = 10u128.pow(decimals as u32) as f64;
    raw_int as f64 / divisor
}

/// Aggregated portfolio statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioAggregation {
    /// Total USD value of all tokens
    pub total_usd_value: f64,
    /// Merged and deduplicated token balances
    pub tokens: Vec<MergedBalance>,
    /// NFTs (if included)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nfts: Option<Vec<NormalizedNft>>,
    /// Chains covered
    pub chains_covered: Vec<String>,
}

/// Token balance merged from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedBalance {
    /// Token contract address (checksummed)
    pub address: String,
    /// Token symbol
    pub symbol: String,
    /// Balance (average if multiple sources)
    pub balance: f64,
    /// USD value (average if multiple sources)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usd_value: Option<f64>,
    /// Sources that reported this token
    pub found_in: Vec<String>,
}

/// Normalized NFT with consistent formatting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedNft {
    /// NFT contract address (checksummed)
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// NFT name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Collection name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
    /// Image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Metadata URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_url: Option<String>,
    /// Token standard (ERC721, ERC1155)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    /// Floor price in native token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub floor_price: Option<f64>,
}

impl NormalizedNft {
    /// Create a new normalized NFT
    pub fn new(contract: impl Into<String>, token_id: impl Into<String>) -> Self {
        Self {
            contract: contract.into(),
            token_id: token_id.into(),
            name: None,
            collection_name: None,
            image_url: None,
            metadata_url: None,
            token_type: None,
            floor_price: None,
        }
    }

    /// Builder: set name
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Builder: set collection name
    #[must_use]
    pub fn with_collection(mut self, collection: impl Into<String>) -> Self {
        self.collection_name = Some(collection.into());
        self
    }

    /// Builder: set image URL
    #[must_use]
    pub fn with_image(mut self, image_url: impl Into<String>) -> Self {
        self.image_url = Some(image_url.into());
        self
    }

    /// Builder: set metadata URL
    #[must_use]
    pub fn with_metadata(mut self, metadata_url: impl Into<String>) -> Self {
        self.metadata_url = Some(metadata_url.into());
        self
    }

    /// Builder: set token type
    #[must_use]
    pub fn with_token_type(mut self, token_type: impl Into<String>) -> Self {
        self.token_type = Some(token_type.into());
        self
    }

    /// Builder: set floor price
    #[must_use]
    pub fn with_floor_price(mut self, floor_price: f64) -> Self {
        self.floor_price = Some(floor_price);
        self
    }
}

/// NFT aggregation from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAggregation {
    /// Total NFTs found
    pub total_count: usize,
    /// Merged and deduplicated NFTs
    pub nfts: Vec<MergedNft>,
    /// Collections covered
    pub collections: Vec<String>,
}

/// NFT merged from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedNft {
    /// NFT contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Best available name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Collection name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,
    /// Best available image URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    /// Sources that reported this NFT
    pub found_in: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_aggregation() {
        // Use prices with spread < 1% so sources_agreed is true
        let prices = vec![100.0, 100.4, 99.6, 100.2];
        let agg = PriceAggregation::from_prices(&prices).unwrap();

        assert!((agg.median_usd - 100.1).abs() < 0.01);
        assert_eq!(agg.min_usd, 99.6);
        assert_eq!(agg.max_usd, 100.4);
        assert!(agg.sources_agreed); // spread < 1% (0.8%)
    }

    #[test]
    fn test_parse_balance_formatted() {
        // Decimal string
        assert!((parse_balance_formatted("1000000000000000000", 18) - 1.0).abs() < 0.0001);

        // Hex string
        assert!((parse_balance_formatted("0xde0b6b3a7640000", 18) - 1.0).abs() < 0.0001);

        // USDC with 6 decimals
        assert!((parse_balance_formatted("1000000", 6) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_normalized_price_builder() {
        let price = NormalizedPrice::new(100.0)
            .with_native(0.05)
            .with_decimals(18)
            .with_change_24h(5.5);

        assert_eq!(price.usd, 100.0);
        assert_eq!(price.native, Some(0.05));
        assert_eq!(price.decimals, Some(18));
        assert_eq!(price.change_24h_pct, Some(5.5));
    }
}
