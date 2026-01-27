//! Solana DAS API implementation

use super::types::{Asset, DisplayOptions, AssetProof, GetAssetsResponse, PaginationOptions, SearchAssetsRequest, GetTokenAccountsResponse, GetNftEditionsResponse, GetAssetSignaturesResponse};
use crate::client::Client;
use crate::error::Result;

/// Solana DAS API for Digital Asset Standard queries
pub struct SolanaApi<'a> {
    client: &'a Client,
}

impl<'a> SolanaApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a single asset by ID
    pub async fn get_asset(&self, id: &str) -> Result<Asset> {
        let params = serde_json::json!({ "id": id });
        self.client.rpc("getAsset", vec![params]).await
    }

    /// Get a single asset with display options
    pub async fn get_asset_with_options(
        &self,
        id: &str,
        options: &DisplayOptions,
    ) -> Result<Asset> {
        let params = serde_json::json!({
            "id": id,
            "displayOptions": options
        });
        self.client.rpc("getAsset", vec![params]).await
    }

    /// Get multiple assets by IDs
    pub async fn get_assets(&self, ids: &[&str]) -> Result<Vec<Asset>> {
        let params = serde_json::json!({ "ids": ids });
        self.client.rpc("getAssets", vec![params]).await
    }

    /// Get proof for a compressed asset
    pub async fn get_asset_proof(&self, id: &str) -> Result<AssetProof> {
        let params = serde_json::json!({ "id": id });
        self.client.rpc("getAssetProof", vec![params]).await
    }

    /// Get proofs for multiple compressed assets
    pub async fn get_asset_proofs(
        &self,
        ids: &[&str],
    ) -> Result<std::collections::HashMap<String, AssetProof>> {
        let params = serde_json::json!({ "ids": ids });
        self.client.rpc("getAssetProofs", vec![params]).await
    }

    /// Get assets by owner
    pub async fn get_assets_by_owner(&self, owner: &str) -> Result<GetAssetsResponse> {
        self.get_assets_by_owner_with_options(
            owner,
            &PaginationOptions::default(),
            &DisplayOptions::default(),
        )
        .await
    }

    /// Get assets by owner with options
    pub async fn get_assets_by_owner_with_options(
        &self,
        owner: &str,
        pagination: &PaginationOptions,
        display: &DisplayOptions,
    ) -> Result<GetAssetsResponse> {
        let params = serde_json::json!({
            "ownerAddress": owner,
            "limit": pagination.limit,
            "page": pagination.page,
            "cursor": pagination.cursor,
            "displayOptions": display
        });
        self.client.rpc("getAssetsByOwner", vec![params]).await
    }

    /// Get assets by creator
    pub async fn get_assets_by_creator(&self, creator: &str) -> Result<GetAssetsResponse> {
        let params = serde_json::json!({
            "creatorAddress": creator,
            "onlyVerified": true
        });
        self.client.rpc("getAssetsByCreator", vec![params]).await
    }

    /// Get assets by authority
    pub async fn get_assets_by_authority(&self, authority: &str) -> Result<GetAssetsResponse> {
        let params = serde_json::json!({ "authorityAddress": authority });
        self.client.rpc("getAssetsByAuthority", vec![params]).await
    }

    /// Get assets by group (e.g., collection)
    pub async fn get_assets_by_group(
        &self,
        group_key: &str,
        group_value: &str,
    ) -> Result<GetAssetsResponse> {
        let params = serde_json::json!({
            "groupKey": group_key,
            "groupValue": group_value
        });
        self.client.rpc("getAssetsByGroup", vec![params]).await
    }

    /// Search assets with filters
    pub async fn search_assets(&self, request: &SearchAssetsRequest) -> Result<GetAssetsResponse> {
        self.client.rpc("searchAssets", vec![request]).await
    }

    /// Get token accounts
    pub async fn get_token_accounts(
        &self,
        owner: Option<&str>,
        mint: Option<&str>,
    ) -> Result<GetTokenAccountsResponse> {
        let mut params = serde_json::Map::new();
        if let Some(o) = owner {
            params.insert("owner".to_string(), serde_json::json!(o));
        }
        if let Some(m) = mint {
            params.insert("mint".to_string(), serde_json::json!(m));
        }
        self.client
            .rpc("getTokenAccounts", vec![serde_json::Value::Object(params)])
            .await
    }

    /// Get NFT editions
    pub async fn get_nft_editions(&self, mint: &str) -> Result<GetNftEditionsResponse> {
        let params = serde_json::json!({ "mint": mint });
        self.client.rpc("getNftEditions", vec![params]).await
    }

    /// Get asset signatures (transactions)
    pub async fn get_asset_signatures(&self, id: &str) -> Result<GetAssetSignaturesResponse> {
        let params = serde_json::json!({ "id": id });
        self.client.rpc("getAssetSignatures", vec![params]).await
    }
}
