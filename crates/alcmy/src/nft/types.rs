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

/// Result of [`NftApi::is_holder_of_contract`](super::NftApi::is_holder_of_contract)
///
/// Serializes with the same shape as the retired `isHolderOfContract` endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsHolderResponse {
    pub is_holder_of_contract: bool,
}

impl IsHolderResponse {
    /// Derive holder status from a contract-filtered `getNFTsForOwner` response
    #[must_use]
    pub fn from_owned_nfts(response: &OwnedNftsResponse) -> Self {
        Self {
            is_holder_of_contract: response.total_count > 0 || !response.owned_nfts.is_empty(),
        }
    }
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

/// Response for isSpamContract
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IsSpamResponse {
    pub is_spam_contract: bool,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn owned(json: serde_json::Value) -> OwnedNftsResponse {
        serde_json::from_value(json).expect("valid OwnedNftsResponse")
    }

    #[test]
    fn is_holder_from_empty_response_is_false() {
        let response = owned(serde_json::json!({ "ownedNfts": [], "totalCount": 0 }));
        assert!(!IsHolderResponse::from_owned_nfts(&response).is_holder_of_contract);
    }

    #[test]
    fn is_holder_from_non_zero_count_is_true() {
        let response = owned(serde_json::json!({ "ownedNfts": [], "totalCount": 3 }));
        assert!(IsHolderResponse::from_owned_nfts(&response).is_holder_of_contract);
    }

    #[test]
    fn is_holder_serializes_like_retired_endpoint() {
        let value = serde_json::to_value(IsHolderResponse {
            is_holder_of_contract: true,
        })
        .unwrap();
        assert_eq!(value, serde_json::json!({ "isHolderOfContract": true }));
    }

    #[test]
    fn get_nfts_for_owner_options_emit_contract_filter() {
        let options = GetNftsForOwnerOptions {
            contract_addresses: Some(vec!["0xabc".to_string()]),
            page_size: Some(1),
            with_metadata: Some(false),
            ..Default::default()
        };
        let params = options.to_query_params();
        assert!(params.contains(&("contractAddresses[]", "0xabc".to_string())));
        assert!(params.contains(&("pageSize", "1".to_string())));
        assert!(params.contains(&("withMetadata", "false".to_string())));
    }
}
