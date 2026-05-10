//! Types for the Market Data API

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

/// Deserialize a value that could be either a float, integer, or a string containing a number.
fn string_or_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(number) => number
            .as_f64()
            .map(Some)
            .ok_or_else(|| de::Error::custom("number cannot be represented as f64")),
        serde_json::Value::String(value) => {
            if value.is_empty() || value == "null" {
                Ok(None)
            } else {
                value.parse().map(Some).map_err(de::Error::custom)
            }
        }
        value => Err(de::Error::custom(format!(
            "expected a float, integer, or string containing a number, got {value}"
        ))),
    }
}

/// Top token data
///
/// Moralis API returns snake_case fields with `contract_address` for the address.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopToken {
    /// Token address (API returns as `contract_address`)
    #[serde(alias = "token_address", alias = "tokenAddress")]
    pub contract_address: Option<String>,
    /// Token name
    #[serde(alias = "tokenName")]
    pub token_name: Option<String>,
    /// Token symbol
    #[serde(alias = "tokenSymbol")]
    pub token_symbol: Option<String>,
    /// Token logo
    #[serde(alias = "tokenLogo")]
    pub token_logo: Option<String>,
    /// Token decimals (API returns as string)
    #[serde(default, deserialize_with = "string_or_f64", alias = "tokenDecimals")]
    pub token_decimals: Option<f64>,
    /// Price USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "priceUsd")]
    pub price_usd: Option<f64>,
    /// Price 24h change percentage
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "price24hPercentChange"
    )]
    pub price_24h_percent_change: Option<f64>,
    /// Price 7d change percentage
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "price7dPercentChange"
    )]
    pub price_7d_percent_change: Option<f64>,
    /// Market cap USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "marketCapUsd")]
    pub market_cap_usd: Option<f64>,
    /// Volume 24h USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "volume24hUsd")]
    pub volume_24h_usd: Option<f64>,
    /// Volume change 24h percentage
    #[serde(default, deserialize_with = "string_or_f64", alias = "volumeChange24h")]
    pub volume_change_24h: Option<f64>,
}

/// Top mover (gainer/loser)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopMover {
    /// Token address
    #[serde(alias = "token_address")]
    pub token_address: Option<String>,
    /// Token name
    #[serde(alias = "token_name")]
    pub token_name: Option<String>,
    /// Token symbol
    #[serde(alias = "token_symbol")]
    pub token_symbol: Option<String>,
    /// Token logo
    #[serde(alias = "token_logo")]
    pub token_logo: Option<String>,
    /// Price USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "price_usd")]
    pub price_usd: Option<f64>,
    /// Price change percentage
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "price_percent_change"
    )]
    pub price_percent_change: Option<f64>,
    /// Volume 24h USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "volume_24h_usd")]
    pub volume_24h_usd: Option<f64>,
}

/// Top NFT collection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopNftCollection {
    /// Collection address
    #[serde(alias = "collection_address")]
    pub collection_address: Option<String>,
    /// Collection title
    #[serde(alias = "collection_title")]
    pub collection_title: Option<String>,
    /// Collection image
    #[serde(alias = "collection_image")]
    pub collection_image: Option<String>,
    /// Floor price USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "floor_price_usd")]
    pub floor_price_usd: Option<f64>,
    /// Floor price 24h change percentage
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "floor_price_24hr_percent_change"
    )]
    pub floor_price_24hr_percent_change: Option<f64>,
    /// Volume 24h USD
    #[serde(default, deserialize_with = "string_or_f64", alias = "volume_usd")]
    pub volume_usd: Option<f64>,
    /// Volume 24h change percentage
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "volume_24hr_percent_change"
    )]
    pub volume_24hr_percent_change: Option<f64>,
    /// Average price USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "average_price_usd"
    )]
    pub average_price_usd: Option<f64>,
}

/// Global market cap data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalMarketCap {
    /// Total market cap USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "total_market_cap_usd"
    )]
    pub total_market_cap_usd: Option<f64>,
    /// Market cap change 24h percentage
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "market_cap_change_24h"
    )]
    pub market_cap_change_24h: Option<f64>,
}

/// Global volume data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalVolume {
    /// Total volume 24h USD
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "total_volume_24h_usd"
    )]
    pub total_volume_24h_usd: Option<f64>,
    /// Volume change 24h percentage
    #[serde(
        default,
        deserialize_with = "string_or_f64",
        alias = "volume_change_24h"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_token_camel_case() {
        let json = r#"{
            "tokenAddress": "0xtoken",
            "tokenName": "Bitcoin",
            "tokenSymbol": "BTC",
            "tokenLogo": "https://logo.png",
            "tokenDecimals": 8,
            "priceUsd": 50000.0,
            "price24hPercentChange": 5.0,
            "marketCapUsd": 1000000000.0,
            "volume24hUsd": 50000000.0
        }"#;
        let token: TopToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.contract_address, Some("0xtoken".to_string()));
        assert_eq!(token.token_name, Some("Bitcoin".to_string()));
        assert_eq!(token.token_symbol, Some("BTC".to_string()));
        assert_eq!(token.price_usd, Some(50000.0));
        assert_eq!(token.market_cap_usd, Some(1000000000.0));
    }

    #[test]
    fn test_top_token_snake_case() {
        // Actual Moralis API response format
        let json = r#"{
            "contract_address": "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2",
            "token_name": "Wrapped Ether",
            "token_symbol": "WETH",
            "token_logo": "https://logo.png",
            "token_decimals": "18",
            "price_usd": "2059.63",
            "price_24h_percent_change": "6.55",
            "price_7d_percent_change": "-0.35",
            "market_cap_usd": "248582348238"
        }"#;
        let token: TopToken = serde_json::from_str(json).unwrap();
        assert_eq!(
            token.contract_address,
            Some("0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2".to_string())
        );
        assert_eq!(token.token_name, Some("Wrapped Ether".to_string()));
        assert_eq!(token.token_decimals, Some(18.0));
        assert_eq!(token.price_usd, Some(2059.63));
    }

    #[test]
    fn test_top_token_string_numbers() {
        let json = r#"{
            "contract_address": "0xtoken",
            "price_usd": "50000.50",
            "market_cap_usd": "1000000000"
        }"#;
        let token: TopToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.price_usd, Some(50000.50));
        assert_eq!(token.market_cap_usd, Some(1000000000.0));
    }

    #[test]
    fn test_top_nft_collection_camel_case() {
        let json = r#"{
            "collectionAddress": "0xbayc",
            "collectionTitle": "Bored Ape Yacht Club",
            "collectionImage": "https://image.png",
            "floorPriceUsd": 19000.0,
            "floorPrice24hrPercentChange": -2.5,
            "volumeUsd": 500000.0,
            "averagePriceUsd": 20000.0
        }"#;
        let nft: TopNftCollection = serde_json::from_str(json).unwrap();
        assert_eq!(nft.collection_address, Some("0xbayc".to_string()));
        assert_eq!(
            nft.collection_title,
            Some("Bored Ape Yacht Club".to_string())
        );
        assert_eq!(nft.floor_price_usd, Some(19000.0));
    }

    #[test]
    fn test_top_nft_collection_string_numbers() {
        let json = r#"{
            "collectionAddress": "0xbayc",
            "floorPriceUsd": "19000.50",
            "volumeUsd": "500000"
        }"#;
        let nft: TopNftCollection = serde_json::from_str(json).unwrap();
        assert_eq!(nft.floor_price_usd, Some(19000.50));
        assert_eq!(nft.volume_usd, Some(500000.0));
    }

    #[test]
    fn test_top_token_null_string_values() {
        let json = r#"{
            "contract_address": "0xtoken",
            "price_usd": "50000.0",
            "price_24h_percent_change": "null",
            "price_7d_percent_change": "null",
            "market_cap_usd": "null"
        }"#;
        let token: TopToken = serde_json::from_str(json).unwrap();
        assert_eq!(token.price_usd, Some(50000.0));
        assert_eq!(token.price_24h_percent_change, None);
        assert_eq!(token.price_7d_percent_change, None);
        assert_eq!(token.market_cap_usd, None);
    }
}
