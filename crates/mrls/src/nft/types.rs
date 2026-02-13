//! Types for the NFT API

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

/// Deserialize a value that could be either a float, integer, or a string containing a number.
/// Moralis API returns NFT floor prices as strings like "0.0189" or "6.36499".
fn string_or_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct StringOrF64Visitor;

    impl<'de> Visitor<'de> for StringOrF64Visitor {
        type Value = Option<f64>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a float, integer, or string containing a number")
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v as f64))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v as f64))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            if v.is_empty() {
                return Ok(None);
            }
            v.parse().map(Some).map_err(de::Error::custom)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }
    }

    deserializer.deserialize_any(StringOrF64Visitor)
}

/// Deserialize a value that could be either an integer or a nested object
/// containing a "total" field. Moralis API sometimes returns stats as nested objects.
fn int_or_nested_total<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntOrNestedVisitor;

    impl<'de> Visitor<'de> for IntOrNestedVisitor {
        type Value = Option<i64>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("an integer, a string containing an integer, or a map with a 'total' field")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> {
            Ok(Some(v))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> {
            Ok(Some(v as i64))
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse().map(Some).map_err(de::Error::custom)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut total: Option<i64> = None;
            while let Some(key) = map.next_key::<String>()? {
                if key == "total" {
                    total = Some(map.next_value::<i64>()?);
                } else {
                    let _: serde_json::Value = map.next_value()?;
                }
            }
            Ok(total)
        }
    }

    deserializer.deserialize_any(IntOrNestedVisitor)
}

/// NFT metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Nft {
    /// Token address
    pub token_address: Option<String>,
    /// Token ID
    pub token_id: Option<String>,
    /// Owner address
    pub owner_of: Option<String>,
    /// Token hash
    pub token_hash: Option<String>,
    /// Block number minted
    pub block_number_minted: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Amount (for ERC1155)
    pub amount: Option<String>,
    /// Contract type (ERC721/ERC1155)
    pub contract_type: Option<String>,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token URI
    pub token_uri: Option<String>,
    /// Metadata JSON string
    pub metadata: Option<String>,
    /// Last token URI sync
    pub last_token_uri_sync: Option<String>,
    /// Last metadata sync
    pub last_metadata_sync: Option<String>,
    /// Minter address
    pub minter_address: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
    /// Verified collection
    pub verified_collection: Option<bool>,
    /// Floor price (Moralis returns as string like "0.0189")
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price: Option<f64>,
    /// Floor price USD (Moralis returns as string like "6.36499")
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price_usd: Option<f64>,
    /// Floor price currency
    pub floor_price_currency: Option<String>,
}

/// NFT collection metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftCollection {
    /// Token address
    pub token_address: Option<String>,
    /// Contract type
    pub contract_type: Option<String>,
    /// Collection name
    pub name: Option<String>,
    /// Collection symbol
    pub symbol: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
    /// Verified collection
    pub verified_collection: Option<bool>,
}

/// NFT transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTransfer {
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Token address
    pub token_address: Option<String>,
    /// Token ID
    pub token_id: Option<String>,
    /// From address
    pub from_address: Option<String>,
    /// To address
    pub to_address: Option<String>,
    /// Value (price in wei)
    pub value: Option<String>,
    /// Amount
    pub amount: Option<String>,
    /// Contract type
    pub contract_type: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Log index
    pub log_index: Option<i32>,
    /// Operator
    pub operator: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
    /// Verified collection
    pub verified_collection: Option<bool>,
}

/// NFT owner
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftOwner {
    /// Token address
    pub token_address: Option<String>,
    /// Token ID
    pub token_id: Option<String>,
    /// Owner address
    pub owner_of: Option<String>,
    /// Amount
    pub amount: Option<String>,
    /// Token hash
    pub token_hash: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block number minted
    pub block_number_minted: Option<String>,
    /// Contract type
    pub contract_type: Option<String>,
    /// Token URI
    pub token_uri: Option<String>,
    /// Metadata
    pub metadata: Option<String>,
    /// Name
    pub name: Option<String>,
    /// Symbol
    pub symbol: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
    /// Verified collection
    pub verified_collection: Option<bool>,
}

/// NFT trade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTrade {
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// Token address
    pub token_address: Option<String>,
    /// Token IDs
    pub token_ids: Option<Vec<String>>,
    /// Seller address
    pub seller_address: Option<String>,
    /// Buyer address
    pub buyer_address: Option<String>,
    /// Marketplace address
    pub marketplace_address: Option<String>,
    /// Price
    pub price: Option<String>,
    /// Price formatted
    pub price_formatted: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
}

/// NFT floor price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftFloorPrice {
    /// Floor price (Moralis returns as string like "6.36499")
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price: Option<f64>,
    /// Floor price USD (Moralis returns as string like "6.36499")
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price_usd: Option<f64>,
    /// Floor price currency
    pub floor_price_currency: Option<String>,
    /// Marketplace
    pub marketplace: Option<String>,
    /// Marketplace address
    pub marketplace_address: Option<String>,
    /// Retrieved at
    pub retrieved_at: Option<String>,
}

/// NFT collection stats
///
/// Moralis API returns some fields as nested objects (e.g., `{"owners": {"total": 5}}`)
/// and some numeric values as strings. This struct handles both representations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftCollectionStats {
    /// Total tokens
    pub total_tokens: Option<String>,
    /// Unique owners (may come as nested `{"total": N}` object)
    #[serde(default, deserialize_with = "int_or_nested_total")]
    pub owners: Option<i64>,
    /// Floor price (Moralis returns as string)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price: Option<f64>,
    /// Floor price USD (Moralis returns as string)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price_usd: Option<f64>,
    /// Market cap USD
    #[serde(default, deserialize_with = "string_or_f64")]
    pub market_cap_usd: Option<f64>,
    /// Volume 24h
    #[serde(default, deserialize_with = "string_or_f64")]
    pub volume_24h: Option<f64>,
    /// Volume 24h USD
    #[serde(default, deserialize_with = "string_or_f64")]
    pub volume_24h_usd: Option<f64>,
    /// Average price 24h
    #[serde(default, deserialize_with = "string_or_f64")]
    pub average_price_24h: Option<f64>,
    /// Average price 24h USD
    #[serde(default, deserialize_with = "string_or_f64")]
    pub average_price_24h_usd: Option<f64>,
    /// Sales 24h (may come as nested `{"total": N}` object)
    #[serde(default, deserialize_with = "int_or_nested_total")]
    pub sales_24h: Option<i64>,
}

/// NFT trait
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTrait {
    /// Trait type
    pub trait_type: Option<String>,
    /// Value
    pub value: Option<serde_json::Value>,
    /// Count
    pub count: Option<i64>,
    /// Percentage
    pub percentage: Option<f64>,
}

/// Paginated NFT response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftResponse<T> {
    /// Status
    pub status: Option<String>,
    /// Page
    pub page: Option<i32>,
    /// Page size
    pub page_size: Option<i32>,
    /// Cursor
    pub cursor: Option<String>,
    /// Results
    pub result: Vec<T>,
}

/// Request for fetching multiple NFTs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMultipleNftsRequest {
    /// List of tokens to fetch
    pub tokens: Vec<NftTokenInput>,
    /// Whether to normalize metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalise_metadata: Option<bool>,
    /// Whether to include media items
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_items: Option<bool>,
}

/// Input for a single NFT token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTokenInput {
    /// Token address
    pub token_address: String,
    /// Token ID
    pub token_id: String,
}

/// Request for fetching NFTs by traits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftsByTraitsRequest {
    /// Traits to filter by
    pub traits: Vec<TraitFilter>,
    /// Cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

/// Trait filter for NFT queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitFilter {
    /// Trait type
    pub trait_type: String,
    /// Trait value
    pub value: serde_json::Value,
}

/// Historical floor price data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalFloorPrice {
    /// Timestamp
    pub timestamp: Option<String>,
    /// Floor price (Moralis may return as string)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price: Option<f64>,
    /// Floor price USD (Moralis may return as string)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price_usd: Option<f64>,
    /// Floor price currency
    pub floor_price_currency: Option<String>,
}

/// NFT sync status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSyncStatus {
    /// Status
    pub status: Option<String>,
}

/// Request for fetching metadata for multiple NFT contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetMultipleCollectionsRequest {
    /// List of contract addresses
    pub addresses: Vec<String>,
}

/// NFT sale price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSalePrice {
    /// Token address
    pub token_address: Option<String>,
    /// Token ID
    pub token_id: Option<String>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Price
    pub price: Option<String>,
    /// Price formatted
    pub price_formatted: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// Payment token
    pub payment_token: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Marketplace
    pub marketplace: Option<String>,
    /// Buyer address
    pub buyer_address: Option<String>,
    /// Seller address
    pub seller_address: Option<String>,
}

/// Trait resync status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitResyncStatus {
    /// Status
    pub status: Option<String>,
    /// Message
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nft_floor_price_as_string() {
        // Moralis returns floor_price as string like "0.0189"
        let json = r#"{
            "floor_price": "0.0189",
            "floor_price_usd": "6.36499",
            "floor_price_currency": "ETH",
            "marketplace": "opensea"
        }"#;
        let floor: NftFloorPrice = serde_json::from_str(json).unwrap();
        assert!((floor.floor_price.unwrap() - 0.0189).abs() < 0.0001);
        assert!((floor.floor_price_usd.unwrap() - 6.36499).abs() < 0.001);
    }

    #[test]
    fn test_nft_floor_price_as_number() {
        // Should also handle numeric floor_price
        let json = r#"{
            "floor_price": 0.0189,
            "floor_price_usd": 6.36499
        }"#;
        let floor: NftFloorPrice = serde_json::from_str(json).unwrap();
        assert!((floor.floor_price.unwrap() - 0.0189).abs() < 0.0001);
    }

    #[test]
    fn test_nft_metadata_floor_price_string() {
        let json = r#"{
            "token_address": "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d",
            "token_id": "1",
            "name": "BoredApeYachtClub",
            "floor_price": "6.36499",
            "floor_price_usd": "19050.5"
        }"#;
        let nft: Nft = serde_json::from_str(json).unwrap();
        assert!((nft.floor_price.unwrap() - 6.36499).abs() < 0.001);
        assert!((nft.floor_price_usd.unwrap() - 19050.5).abs() < 0.1);
    }

    #[test]
    fn test_collection_stats_nested_owners() {
        // Moralis returns owners as {"total": N} nested object
        let json = r#"{
            "total_tokens": "10000",
            "owners": {"total": 6500},
            "floor_price": "6.36499",
            "floor_price_usd": "19050.5",
            "market_cap_usd": "190505000",
            "volume_24h": "100.5",
            "sales_24h": {"total": 42}
        }"#;
        let stats: NftCollectionStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.owners, Some(6500));
        assert_eq!(stats.sales_24h, Some(42));
        assert!((stats.floor_price.unwrap() - 6.36499).abs() < 0.001);
    }

    #[test]
    fn test_collection_stats_flat_integers() {
        // Should also handle flat integers
        let json = r#"{
            "total_tokens": "10000",
            "owners": 6500,
            "floor_price": 6.36499,
            "sales_24h": 42
        }"#;
        let stats: NftCollectionStats = serde_json::from_str(json).unwrap();
        assert_eq!(stats.owners, Some(6500));
        assert_eq!(stats.sales_24h, Some(42));
    }

    #[test]
    fn test_collection_stats_string_market_cap() {
        // market_cap_usd can come as string
        let json = r#"{
            "total_tokens": "10000",
            "market_cap_usd": "190505000.50"
        }"#;
        let stats: NftCollectionStats = serde_json::from_str(json).unwrap();
        assert!((stats.market_cap_usd.unwrap() - 190_505_000.50).abs() < 0.1);
    }
}
