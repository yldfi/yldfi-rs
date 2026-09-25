//! NFT API implementation

use super::types::{
    ContractMetadataBatchResponse, ContractMetadataResponse, ContractsForOwnerResponse,
    FloorPriceResponse, GetNftsForOwnerOptions, IsHolderResponse, IsSpamResponse, Nft,
    NftsForContractResponse, OwnedNftsResponse, OwnersForContractResponse, OwnersForNftResponse,
    RefreshMetadataResponse,
};
use crate::client::Client;
use crate::error::Result;

/// NFT API for ownership, metadata, floor prices, and spam detection
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
    ///
    /// Alchemy retired the dedicated `isHolderOfContract` endpoint on 2026-09-30,
    /// so this is implemented on top of `getNFTsForOwner` filtered to a single
    /// contract (one result page, no metadata).
    pub async fn is_holder_of_contract(
        &self,
        wallet: &str,
        contract_address: &str,
    ) -> Result<IsHolderResponse> {
        let options = GetNftsForOwnerOptions {
            contract_addresses: Some(vec![contract_address.to_string()]),
            page_size: Some(1),
            with_metadata: Some(false),
            ..Default::default()
        };
        let response = self
            .get_nfts_for_owner_with_options(wallet, &options)
            .await?;
        Ok(IsHolderResponse::from_owned_nfts(&response))
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

    // ========== Pricing Methods ==========

    /// Get floor price for a contract
    pub async fn get_floor_price(&self, contract_address: &str) -> Result<FloorPriceResponse> {
        let query = [("contractAddress", contract_address)];
        self.client.nft_get("getFloorPrice", &query).await
    }

    // ========== Spam Methods ==========

    /// Check if a contract is spam
    pub async fn is_spam_contract(&self, contract_address: &str) -> Result<IsSpamResponse> {
        let query = [("contractAddress", contract_address)];
        self.client.nft_get("isSpamContract", &query).await
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
}
