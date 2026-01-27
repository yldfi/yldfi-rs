//! NFT API implementation

use super::types::{
    AttributeSummaryResponse, CollectionMetadata, CollectionsForOwnerResponse, ContractMetadata,
    ContractMetadataBatchResponse, ContractMetadataResponse, ContractsForOwnerResponse,
    FloorPriceResponse, GetNftsForOwnerOptions, InvalidateContractResponse, IsAirdropResponse,
    IsHolderResponse, IsSpamResponse, Nft, NftRarityResponse, NftSalesResponse,
    NftsForContractResponse, OwnedNftsResponse, OwnersForContractResponse, OwnersForNftResponse,
    RefreshMetadataResponse, SpamContractsResponse,
};
use crate::client::Client;
use crate::error::Result;

/// NFT API for ownership, metadata, sales, and spam detection
pub struct NftApi<'a> {
    client: &'a Client,
}

impl<'a> NftApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    // ========== Ownership Methods ==========

    /// Get all NFTs owned by an address
    pub async fn get_nfts_for_owner(&self, owner: &str) -> Result<OwnedNftsResponse> {
        self.get_nfts_for_owner_with_options(owner, &GetNftsForOwnerOptions::default())
            .await
    }

    /// Get all NFTs owned by an address with options
    pub async fn get_nfts_for_owner_with_options(
        &self,
        owner: &str,
        options: &GetNftsForOwnerOptions,
    ) -> Result<OwnedNftsResponse> {
        let mut query: Vec<(&str, String)> = vec![("owner", owner.to_string())];
        query.extend(options.to_query_params());

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.nft_get("getNFTsForOwner", &query_refs).await
    }

    /// Get owners of a specific NFT
    pub async fn get_owners_for_nft(
        &self,
        contract_address: &str,
        token_id: &str,
    ) -> Result<OwnersForNftResponse> {
        let query = [("contractAddress", contract_address), ("tokenId", token_id)];
        self.client.nft_get("getOwnersForNFT", &query).await
    }

    /// Get all owners of an NFT contract
    pub async fn get_owners_for_contract(
        &self,
        contract_address: &str,
    ) -> Result<OwnersForContractResponse> {
        let query = [("contractAddress", contract_address)];
        self.client.nft_get("getOwnersForContract", &query).await
    }

    /// Get NFT contracts owned by an address
    pub async fn get_contracts_for_owner(&self, owner: &str) -> Result<ContractsForOwnerResponse> {
        let query = [("owner", owner)];
        self.client.nft_get("getContractsForOwner", &query).await
    }

    /// Check if an address owns any NFT from a contract
    pub async fn is_holder_of_contract(
        &self,
        wallet: &str,
        contract_address: &str,
    ) -> Result<IsHolderResponse> {
        let query = [("wallet", wallet), ("contractAddress", contract_address)];
        self.client.nft_get("isHolderOfContract", &query).await
    }

    // ========== Metadata Methods ==========

    /// Get metadata for a specific NFT
    pub async fn get_nft_metadata(&self, contract_address: &str, token_id: &str) -> Result<Nft> {
        let query = [("contractAddress", contract_address), ("tokenId", token_id)];
        self.client.nft_get("getNFTMetadata", &query).await
    }

    /// Get metadata for multiple NFTs in batch
    pub async fn get_nft_metadata_batch(
        &self,
        tokens: Vec<(String, String)>, // (contract_address, token_id)
    ) -> Result<Vec<Nft>> {
        let body = serde_json::json!({
            "tokens": tokens.into_iter().map(|(addr, id)| {
                serde_json::json!({
                    "contractAddress": addr,
                    "tokenId": id
                })
            }).collect::<Vec<_>>()
        });
        self.client.nft_post("getNFTMetadataBatch", &body).await
    }

    /// Get all NFTs for a contract
    pub async fn get_nfts_for_contract(
        &self,
        contract_address: &str,
    ) -> Result<NftsForContractResponse> {
        self.get_nfts_for_contract_with_options(contract_address, None, None)
            .await
    }

    /// Get all NFTs for a contract with pagination
    pub async fn get_nfts_for_contract_with_options(
        &self,
        contract_address: &str,
        start_token: Option<&str>,
        limit: Option<u32>,
    ) -> Result<NftsForContractResponse> {
        let mut query = vec![("contractAddress", contract_address.to_string())];
        if let Some(start) = start_token {
            query.push(("startToken", start.to_string()));
        }
        if let Some(l) = limit {
            query.push(("limit", l.to_string()));
        }
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.nft_get("getNFTsForContract", &query_refs).await
    }

    /// Get contract metadata
    pub async fn get_contract_metadata(
        &self,
        contract_address: &str,
    ) -> Result<ContractMetadataResponse> {
        let query = [("contractAddress", contract_address)];
        self.client.nft_get("getContractMetadata", &query).await
    }

    /// Get metadata for multiple contracts in batch
    pub async fn get_contract_metadata_batch(
        &self,
        contract_addresses: Vec<String>,
    ) -> Result<ContractMetadataBatchResponse> {
        let body = serde_json::json!({
            "contractAddresses": contract_addresses
        });
        self.client
            .nft_post("getContractMetadataBatch", &body)
            .await
    }

    /// Get collection metadata by `OpenSea` slug
    pub async fn get_collection_metadata(&self, slug: &str) -> Result<CollectionMetadata> {
        let query = [("collectionSlug", slug)];
        self.client.nft_get("getCollectionMetadata", &query).await
    }

    /// Search contract metadata by keyword
    pub async fn search_contract_metadata(
        &self,
        query_text: &str,
    ) -> Result<Vec<ContractMetadata>> {
        let query = [("query", query_text)];
        self.client.nft_get("searchContractMetadata", &query).await
    }

    /// Compute rarity for an NFT
    pub async fn compute_rarity(
        &self,
        contract_address: &str,
        token_id: &str,
    ) -> Result<NftRarityResponse> {
        let query = [("contractAddress", contract_address), ("tokenId", token_id)];
        self.client.nft_get("computeRarity", &query).await
    }

    /// Summarize NFT attributes for a contract
    pub async fn summarize_nft_attributes(
        &self,
        contract_address: &str,
    ) -> Result<AttributeSummaryResponse> {
        let query = [("contractAddress", contract_address)];
        self.client.nft_get("summarizeNFTAttributes", &query).await
    }

    /// Refresh metadata for an NFT
    pub async fn refresh_nft_metadata(
        &self,
        contract_address: &str,
        token_id: &str,
    ) -> Result<RefreshMetadataResponse> {
        let body = serde_json::json!({
            "contractAddress": contract_address,
            "tokenId": token_id
        });
        self.client.nft_post("refreshNftMetadata", &body).await
    }

    // ========== Sales Methods ==========

    /// Get NFT sales for a contract
    pub async fn get_nft_sales(&self, contract_address: &str) -> Result<NftSalesResponse> {
        self.get_nft_sales_with_options(contract_address, None, None, None)
            .await
    }

    /// Get NFT sales with options
    pub async fn get_nft_sales_with_options(
        &self,
        contract_address: &str,
        token_id: Option<&str>,
        from_block: Option<u64>,
        to_block: Option<u64>,
    ) -> Result<NftSalesResponse> {
        let mut query = vec![("contractAddress", contract_address.to_string())];
        if let Some(id) = token_id {
            query.push(("tokenId", id.to_string()));
        }
        if let Some(from) = from_block {
            query.push(("fromBlock", from.to_string()));
        }
        if let Some(to) = to_block {
            query.push(("toBlock", to.to_string()));
        }
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client.nft_get("getNFTSales", &query_refs).await
    }

    /// Get floor price for a contract
    pub async fn get_floor_price(&self, contract_address: &str) -> Result<FloorPriceResponse> {
        let query = [("contractAddress", contract_address)];
        self.client.nft_get("getFloorPrice", &query).await
    }

    // ========== Spam Methods ==========

    /// Get list of spam contracts
    pub async fn get_spam_contracts(&self) -> Result<SpamContractsResponse> {
        self.client.nft_get("getSpamContracts", &[]).await
    }

    /// Check if a contract is spam
    pub async fn is_spam_contract(&self, contract_address: &str) -> Result<IsSpamResponse> {
        let query = [("contractAddress", contract_address)];
        self.client.nft_get("isSpamContract", &query).await
    }

    /// Check if an NFT is an airdrop
    pub async fn is_airdrop_nft(
        &self,
        contract_address: &str,
        token_id: &str,
    ) -> Result<IsAirdropResponse> {
        let query = [("contractAddress", contract_address), ("tokenId", token_id)];
        self.client.nft_get("isAirdropNFT", &query).await
    }

    /// Report a contract as spam
    pub async fn report_spam(&self, contract_address: &str) -> Result<()> {
        let query = [("address", contract_address)];
        let _: serde_json::Value = self.client.nft_get("reportSpam", &query).await?;
        Ok(())
    }

    // ========== Collection Methods ==========

    /// Get all NFTs for a collection by slug
    pub async fn get_nfts_for_collection(
        &self,
        collection_slug: &str,
    ) -> Result<NftsForContractResponse> {
        self.get_nfts_for_collection_with_options(collection_slug, None, None)
            .await
    }

    /// Get all NFTs for a collection with pagination
    pub async fn get_nfts_for_collection_with_options(
        &self,
        collection_slug: &str,
        start_token: Option<&str>,
        limit: Option<u32>,
    ) -> Result<NftsForContractResponse> {
        let mut query = vec![("collectionSlug", collection_slug.to_string())];
        if let Some(start) = start_token {
            query.push(("startToken", start.to_string()));
        }
        if let Some(l) = limit {
            query.push(("limit", l.to_string()));
        }
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .nft_get("getNFTsForCollection", &query_refs)
            .await
    }

    /// Get collections owned by an address
    pub async fn get_collections_for_owner(
        &self,
        owner: &str,
    ) -> Result<CollectionsForOwnerResponse> {
        self.get_collections_for_owner_with_options(owner, None, None)
            .await
    }

    /// Get collections owned by an address with pagination
    pub async fn get_collections_for_owner_with_options(
        &self,
        owner: &str,
        page_key: Option<&str>,
        page_size: Option<u32>,
    ) -> Result<CollectionsForOwnerResponse> {
        let mut query = vec![("owner", owner.to_string())];
        if let Some(key) = page_key {
            query.push(("pageKey", key.to_string()));
        }
        if let Some(size) = page_size {
            query.push(("pageSize", size.to_string()));
        }
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.client
            .nft_get("getCollectionsForOwner", &query_refs)
            .await
    }

    // ========== Cache Invalidation ==========

    /// Invalidate cached metadata for a contract
    pub async fn invalidate_contract(
        &self,
        contract_address: &str,
    ) -> Result<InvalidateContractResponse> {
        let body = serde_json::json!({
            "contractAddress": contract_address
        });
        self.client.nft_post("invalidateContract", &body).await
    }
}
