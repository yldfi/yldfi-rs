//! Types for the NFT API

use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};

/// Deserialize a value that could be either a float, integer, or a string containing a number.
/// Moralis API returns NFT floor prices as strings like "0.0189" or "6.36499".
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

/// Deserialize a value that could be either an integer or a nested object
/// containing a "total" field. Moralis API sometimes returns stats as nested objects.
/// Deserialize a value that could be a string or an integer into a string.
fn string_or_int<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::String(value) => Ok(Some(value)),
        serde_json::Value::Number(number) => Ok(Some(number.to_string())),
        value => Err(de::Error::custom(format!(
            "expected a string or integer, got {value}"
        ))),
    }
}

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

/// Marketplace reference: Moralis returns `{ name, logo }`, older responses a bare name
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum NftMarketplace {
    /// Marketplace object
    Info {
        /// Marketplace name
        name: Option<String>,
        /// Marketplace logo URL
        logo: Option<String>,
    },
    /// Marketplace name only
    Name(String),
}

impl NftMarketplace {
    /// Marketplace name, whichever form was returned
    #[must_use]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Info { name, .. } => name.as_deref(),
            Self::Name(name) => Some(name),
        }
    }
}

/// NFT floor price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftFloorPrice {
    /// Collection address
    pub address: Option<String>,
    /// Token ID (token-level floor price only)
    pub token_id: Option<String>,
    /// Floor price (Moralis returns as string like "6.36499")
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price: Option<f64>,
    /// Floor price USD (Moralis returns as string like "6.36499")
    #[serde(default, deserialize_with = "string_or_f64")]
    pub floor_price_usd: Option<f64>,
    /// Floor price currency (Moralis returns this as `currency`)
    #[serde(alias = "floor_price_currency")]
    pub currency: Option<String>,
    /// Marketplace
    pub marketplace: Option<NftMarketplace>,
    /// Last updated
    pub last_updated: Option<String>,
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

/// NFT trait value with its frequency (item of `/nft/{address}/traits/paginate`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTrait {
    /// Trait type
    pub trait_type: Option<String>,
    /// Trait value
    #[serde(alias = "value")]
    pub trait_value: Option<serde_json::Value>,
    /// Number of NFTs with this value
    pub count: Option<i64>,
    /// Share of the collection with this value (percent)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub percentage: Option<f64>,
    /// Rarity label
    pub rarity_label: Option<String>,
}

/// One value of a trait type in `/nft/{address}/traits`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTraitValue {
    /// Trait value
    pub trait_value: Option<serde_json::Value>,
    /// Number of NFTs with this value
    pub count: Option<i64>,
    /// Share of the collection with this value (percent)
    #[serde(default, deserialize_with = "string_or_f64")]
    pub percentage: Option<f64>,
    /// Rarity label
    pub rarity_label: Option<String>,
}

/// A trait type and its values in `/nft/{address}/traits`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTraitGroup {
    /// Trait type
    pub trait_type: Option<String>,
    /// Values of this trait type
    #[serde(default)]
    pub values: Vec<NftTraitValue>,
}

/// Response of `/nft/{address}/traits`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftCollectionTraits {
    /// Trait types
    #[serde(default)]
    pub traits: Vec<NftTraitGroup>,
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
    /// Floor price currency (Moralis returns this as `currency`)
    #[serde(alias = "floor_price_currency")]
    pub currency: Option<String>,
    /// Marketplace the floor price was observed on
    pub marketplace: Option<serde_json::Value>,
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

/// Payment token used in an NFT sale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftPaymentToken {
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Token decimals
    #[serde(default, deserialize_with = "string_or_int")]
    pub token_decimals: Option<String>,
    /// Token address
    pub token_address: Option<String>,
}

/// A single NFT sale (last / lowest / highest)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSale {
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Buyer address
    pub buyer_address: Option<String>,
    /// Seller address
    pub seller_address: Option<String>,
    /// Price in the payment token's base units
    #[serde(default, deserialize_with = "string_or_int")]
    pub price: Option<String>,
    /// Price formatted
    #[serde(default, deserialize_with = "string_or_int")]
    pub price_formatted: Option<String>,
    /// USD price at the time of sale
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_price_at_sale: Option<f64>,
    /// Current USD value of the sale amount
    #[serde(default, deserialize_with = "string_or_f64")]
    pub current_usd_value: Option<f64>,
    /// Token ID
    #[serde(default, deserialize_with = "string_or_int")]
    pub token_id: Option<String>,
    /// Payment token
    pub payment_token: Option<NftPaymentToken>,
}

/// Average NFT sale price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftAverageSale {
    /// Average price in the payment token's base units
    #[serde(default, deserialize_with = "string_or_int")]
    pub price: Option<String>,
    /// Average price formatted
    #[serde(default, deserialize_with = "string_or_int")]
    pub price_formatted: Option<String>,
    /// Current USD value
    #[serde(default, deserialize_with = "string_or_f64")]
    pub current_usd_value: Option<f64>,
}

/// NFT sale price summary (collection or token) over the requested period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftSalePrices {
    /// Most recent sale
    pub last_sale: Option<NftSale>,
    /// Lowest sale
    pub lowest_sale: Option<NftSale>,
    /// Highest sale
    pub highest_sale: Option<NftSale>,
    /// Average sale
    pub average_sale: Option<NftAverageSale>,
    /// Number of trades
    pub total_trades: Option<i64>,
    /// Message (set when no data is available)
    pub message: Option<String>,
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
    fn collection_traits_groups_values_by_type() {
        let json = r#"{"traits": [{
            "trait_type": "Background",
            "values": [
                {"trait_value": "Blue", "count": 12, "percentage": 1.2, "rarity_label": "Top 2%"},
                {"trait_value": 7, "count": 3, "percentage": "0.3", "rarity_label": "Top 1%"}
            ]
        }]}"#;
        let resp: NftCollectionTraits = serde_json::from_str(json).unwrap();
        let group = &resp.traits[0];
        assert_eq!(group.trait_type.as_deref(), Some("Background"));
        assert_eq!(group.values.len(), 2);
        assert_eq!(group.values[0].count, Some(12));
        assert_eq!(group.values[1].percentage, Some(0.3));
    }

    #[test]
    fn paginated_trait_uses_trait_value() {
        let json = r#"{"trait_type": "Hat", "trait_value": "Cap", "count": 5,
            "percentage": 0.5, "rarity_label": "Top 1%"}"#;
        let t: NftTrait = serde_json::from_str(json).unwrap();
        assert_eq!(t.trait_value, Some(serde_json::json!("Cap")));
        assert_eq!(t.rarity_label.as_deref(), Some("Top 1%"));
    }

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

    #[test]
    fn test_historical_floor_price_envelope() {
        let json = r#"{
            "cursor": "next",
            "page": 0,
            "page_size": 100,
            "result": [{
                "timestamp": "2024-01-01T00:00:00.000Z",
                "floor_price": "0.5",
                "floor_price_usd": "1500.25",
                "currency": "eth",
                "marketplace": "opensea"
            }]
        }"#;
        let resp: NftResponse<HistoricalFloorPrice> = serde_json::from_str(json).unwrap();
        assert_eq!(resp.cursor.as_deref(), Some("next"));
        let p = &resp.result[0];
        assert_eq!(p.floor_price, Some(0.5));
        assert_eq!(p.floor_price_usd, Some(1500.25));
        assert_eq!(p.currency.as_deref(), Some("eth"));
        assert_eq!(p.marketplace, Some(serde_json::json!("opensea")));
    }

    #[test]
    fn test_nft_floor_price_spec_shape() {
        let json = r#"{
            "address": "0xnft",
            "floor_price": "0.5",
            "floor_price_usd": "1500",
            "currency": "eth",
            "marketplace": {"name": "opensea", "logo": "https://example.com/logo.png"},
            "last_updated": "2024-01-01T00:00:00.000Z"
        }"#;
        let floor: NftFloorPrice = serde_json::from_str(json).unwrap();
        assert_eq!(floor.address.as_deref(), Some("0xnft"));
        assert_eq!(floor.currency.as_deref(), Some("eth"));
        let marketplace = floor.marketplace.unwrap();
        assert_eq!(marketplace.name(), Some("opensea"));
        assert_eq!(
            marketplace,
            NftMarketplace::Info {
                name: Some("opensea".to_string()),
                logo: Some("https://example.com/logo.png".to_string()),
            }
        );

        // Legacy shape: `floor_price_currency` and a bare marketplace name
        let legacy = r#"{"floor_price_currency": "ETH", "marketplace": "opensea"}"#;
        let floor: NftFloorPrice = serde_json::from_str(legacy).unwrap();
        assert_eq!(floor.currency.as_deref(), Some("ETH"));
        assert_eq!(
            floor.marketplace,
            Some(NftMarketplace::Name("opensea".to_string()))
        );
    }

    #[test]
    fn test_nft_sale_prices_object() {
        let json = r#"{
            "last_sale": {
                "transaction_hash": "0xtx",
                "block_timestamp": "2024-01-01T00:00:00.000Z",
                "buyer_address": "0xbuyer",
                "seller_address": "0xseller",
                "price": "1000000000000000000",
                "price_formatted": "1",
                "usd_price_at_sale": "2500.5",
                "current_usd_value": "3000",
                "token_id": "42",
                "payment_token": {
                    "token_name": "Ether",
                    "token_symbol": "ETH",
                    "token_logo": "https://example.com/eth.png",
                    "token_decimals": "18",
                    "token_address": "0xeee"
                }
            },
            "lowest_sale": null,
            "highest_sale": null,
            "average_sale": {"price": "500000000000000000", "price_formatted": "0.5", "current_usd_value": "1500"},
            "total_trades": 3
        }"#;
        let prices: NftSalePrices = serde_json::from_str(json).unwrap();
        let last = prices.last_sale.unwrap();
        assert_eq!(last.token_id.as_deref(), Some("42"));
        assert_eq!(last.usd_price_at_sale, Some(2500.5));
        assert_eq!(
            last.payment_token.unwrap().token_decimals.as_deref(),
            Some("18")
        );
        assert!(prices.lowest_sale.is_none());
        assert_eq!(prices.average_sale.unwrap().current_usd_value, Some(1500.0));
        assert_eq!(prices.total_trades, Some(3));
        assert!(prices.message.is_none());
    }
}
