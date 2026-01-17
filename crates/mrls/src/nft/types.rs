//! Types for the NFT API

use serde::{Deserialize, Serialize};

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
    /// Floor price
    pub floor_price: Option<f64>,
    /// Floor price USD
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
    /// Floor price
    pub floor_price: Option<f64>,
    /// Floor price USD
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftCollectionStats {
    /// Total tokens
    pub total_tokens: Option<String>,
    /// Unique owners
    pub owners: Option<i64>,
    /// Floor price
    pub floor_price: Option<f64>,
    /// Floor price USD
    pub floor_price_usd: Option<f64>,
    /// Market cap USD
    pub market_cap_usd: Option<f64>,
    /// Volume 24h
    pub volume_24h: Option<f64>,
    /// Volume 24h USD
    pub volume_24h_usd: Option<f64>,
    /// Average price 24h
    pub average_price_24h: Option<f64>,
    /// Average price 24h USD
    pub average_price_24h_usd: Option<f64>,
    /// Sales 24h
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
    /// Floor price
    pub floor_price: Option<f64>,
    /// Floor price USD
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
