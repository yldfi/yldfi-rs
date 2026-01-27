//! Types for the NFT API

use serde::{Deserialize, Serialize};

/// NFT token type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NftTokenType {
    Erc721,
    Erc1155,
    NoSupportedNftStandard,
    NotAContract,
    Unknown,
}

/// `OpenSea` safety level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenSeaSafetyLevel {
    Safe,
    Approved,
    Verified,
    NotRequested,
}

/// NFT spam classification
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpamInfo {
    pub is_spam: bool,
    #[serde(default)]
    pub classifications: Vec<String>,
}

/// Contract metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractMetadata {
    pub address: String,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub total_supply: Option<String>,
    pub token_type: Option<NftTokenType>,
    pub contract_deployer: Option<String>,
    pub deployed_block_number: Option<u64>,
    pub opensea_metadata: Option<OpenSeaMetadata>,
}

/// `OpenSea` collection metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenSeaMetadata {
    pub floor_price: Option<f64>,
    pub collection_name: Option<String>,
    pub collection_slug: Option<String>,
    pub safety_level: Option<OpenSeaSafetyLevel>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub external_url: Option<String>,
    pub twitter_username: Option<String>,
    pub discord_url: Option<String>,
    pub banner_image_url: Option<String>,
    pub last_ingested_at: Option<String>,
}

/// NFT image
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftImage {
    pub cached_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub png_url: Option<String>,
    pub content_type: Option<String>,
    pub size: Option<u64>,
    pub original_url: Option<String>,
}

/// NFT raw metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftRawMetadata {
    pub token_uri: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub error: Option<String>,
}

/// NFT attribute
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftAttribute {
    pub trait_type: Option<String>,
    pub value: Option<serde_json::Value>,
    pub display_type: Option<String>,
}

/// NFT metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Nft {
    pub contract: ContractMetadata,
    pub token_id: String,
    pub token_type: Option<NftTokenType>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub token_uri: Option<String>,
    pub image: Option<NftImage>,
    pub raw: Option<NftRawMetadata>,
    #[serde(default)]
    pub attributes: Vec<NftAttribute>,
    pub balance: Option<String>,
    pub acquired_at: Option<AcquiredAt>,
    pub collection: Option<CollectionInfo>,
    pub mint: Option<MintInfo>,
    pub time_last_updated: Option<String>,
}

/// Collection info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionInfo {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub external_url: Option<String>,
    pub banner_image_url: Option<String>,
}

/// Mint info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MintInfo {
    pub mint_address: Option<String>,
    pub block_number: Option<u64>,
    pub timestamp: Option<String>,
    pub transaction_hash: Option<String>,
}

/// NFT acquisition info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcquiredAt {
    pub block_timestamp: Option<String>,
    pub block_number: Option<u64>,
}

/// Response for getNFTsForOwner
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedNftsResponse {
    pub owned_nfts: Vec<Nft>,
    pub total_count: u64,
    pub valid_at: Option<BlockInfo>,
    pub page_key: Option<String>,
}

/// Block info for response validation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockInfo {
    pub block_number: Option<u64>,
    pub block_hash: Option<String>,
    pub block_timestamp: Option<String>,
}

/// Response for getOwnersForNFT
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnersForNftResponse {
    pub owners: Vec<String>,
    pub page_key: Option<String>,
}

/// Response for getOwnersForContract
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnersForContractResponse {
    pub owners: Vec<OwnerInfo>,
    pub page_key: Option<String>,
}

/// Owner info with token balances
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerInfo {
    pub owner_address: String,
    pub token_balances: Vec<TokenBalance>,
}

/// Token balance for owner
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    pub token_id: String,
    pub balance: String,
}

/// Response for getContractsForOwner
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractsForOwnerResponse {
    pub contracts: Vec<ContractWithMetadata>,
    pub total_count: u64,
    pub page_key: Option<String>,
}

/// Contract with additional metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractWithMetadata {
    #[serde(flatten)]
    pub contract: ContractMetadata,
    pub total_balance: Option<String>,
    pub num_distinct_tokens_owned: Option<String>,
    pub is_spam: Option<bool>,
    pub display_nft: Option<Nft>,
}

/// Response for isHolderOfContract
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsHolderResponse {
    pub is_holder_of_contract: bool,
}

/// Response for getNFTsForContract
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftsForContractResponse {
    pub nfts: Vec<Nft>,
    pub page_key: Option<String>,
}

/// Response for getContractMetadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractMetadataResponse {
    #[serde(flatten)]
    pub contract: ContractMetadata,
}

/// Response for getContractMetadataBatch
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractMetadataBatchResponse {
    pub contracts: Vec<ContractMetadata>,
}

/// Collection metadata response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionMetadata {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub banner_image_url: Option<String>,
    pub external_url: Option<String>,
    pub twitter_username: Option<String>,
    pub discord_url: Option<String>,
}

/// NFT sale
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftSale {
    pub marketplace: Option<String>,
    pub marketplace_address: Option<String>,
    pub contract_address: String,
    pub token_id: String,
    pub quantity: String,
    pub buyer_address: String,
    pub seller_address: String,
    pub taker: Option<String>,
    pub seller_fee: Option<Fee>,
    pub protocol_fee: Option<Fee>,
    pub royalty_fee: Option<Fee>,
    pub block_number: u64,
    pub log_index: u64,
    pub bundle_index: u64,
    pub transaction_hash: String,
}

/// Fee info for sales
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fee {
    pub amount: Option<String>,
    pub token_address: Option<String>,
    pub symbol: Option<String>,
    pub decimals: Option<u8>,
}

/// Response for getNFTSales
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftSalesResponse {
    pub nft_sales: Vec<NftSale>,
    pub valid_at: Option<BlockInfo>,
    pub page_key: Option<String>,
}

/// Floor price response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FloorPriceResponse {
    pub opensea: Option<FloorPriceMarketplace>,
    pub looksrare: Option<FloorPriceMarketplace>,
}

/// Floor price for a marketplace
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FloorPriceMarketplace {
    pub floor_price: Option<f64>,
    pub price_currency: Option<String>,
    pub collection_url: Option<String>,
    pub retrieved_at: Option<String>,
    pub error: Option<String>,
}

/// Response for getSpamContracts
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpamContractsResponse {
    pub contract_addresses: Vec<String>,
}

/// Response for isSpamContract
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsSpamResponse {
    pub is_spam_contract: bool,
}

/// Response for isAirdropNFT
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsAirdropResponse {
    pub is_airdrop: bool,
}

/// NFT rarity response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftRarityResponse {
    pub rarities: Vec<AttributeRarity>,
}

/// Attribute rarity info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeRarity {
    pub trait_type: String,
    pub value: serde_json::Value,
    pub prevalence: f64,
}

/// NFT attribute summary response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeSummaryResponse {
    pub contract_address: String,
    pub total_supply: String,
    pub summary: serde_json::Value,
}

/// Refresh metadata response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshMetadataResponse {
    pub contract_address: String,
    pub token_id: String,
    pub refresh_state: String,
}

/// Options for getNFTsForOwner
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNftsForOwnerOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_addresses: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_filters: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_filters: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_metadata: Option<bool>,
}

impl GetNftsForOwnerOptions {
    #[must_use] 
    pub fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(ref key) = self.page_key {
            params.push(("pageKey", key.clone()));
        }
        if let Some(size) = self.page_size {
            params.push(("pageSize", size.to_string()));
        }
        if let Some(ref addrs) = self.contract_addresses {
            for addr in addrs {
                params.push(("contractAddresses[]", addr.clone()));
            }
        }
        if let Some(ref filters) = self.exclude_filters {
            for f in filters {
                params.push(("excludeFilters[]", f.clone()));
            }
        }
        if let Some(ref filters) = self.include_filters {
            for f in filters {
                params.push(("includeFilters[]", f.clone()));
            }
        }
        if let Some(ref order) = self.order_by {
            params.push(("orderBy", order.clone()));
        }
        if let Some(with_meta) = self.with_metadata {
            params.push(("withMetadata", with_meta.to_string()));
        }
        params
    }
}

/// Response for getCollectionsForOwner
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CollectionsForOwnerResponse {
    pub collections: Vec<OwnedCollection>,
    pub total_count: u64,
    pub page_key: Option<String>,
}

/// Collection owned by an address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnedCollection {
    pub name: Option<String>,
    pub slug: Option<String>,
    pub image_url: Option<String>,
    pub banner_image_url: Option<String>,
    pub external_url: Option<String>,
    pub num_distinct_tokens_owned: Option<u64>,
    pub total_balance: Option<String>,
    pub is_spam: Option<bool>,
    pub contract: Option<ContractMetadata>,
}

/// Response for invalidateContract
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvalidateContractResponse {
    pub contract_address: String,
    pub progress: String,
}
