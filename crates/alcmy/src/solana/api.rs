//! Solana DAS API implementation
//!
//! DAS methods take their parameters by name, so every call sends a JSON
//! object as `params` (not a one-element array).

use serde_json::{Map, Value};

use super::types::{
    Asset, AssetProof, DisplayOptions, GetAssetSignaturesResponse, GetAssetsResponse,
    GetNftEditionsResponse, GetTokenAccountsResponse, PaginationOptions, SearchAssetsRequest,
};
use crate::client::Client;
use crate::error::Result;

/// Solana DAS API for Digital Asset Standard queries
pub struct SolanaApi<'a> {
    client: &'a Client,
}

/// Build a by-name params object from required fields plus pagination and
/// (optionally) display options under `display_key`.
fn params_object(
    fields: Vec<(&str, Value)>,
    pagination: Option<&PaginationOptions>,
    display: Option<(&str, &DisplayOptions)>,
) -> Value {
    let mut map = Map::new();
    for (k, v) in fields {
        map.insert(k.to_string(), v);
    }
    if let Some(p) = pagination {
        if let Ok(Value::Object(p)) = serde_json::to_value(p) {
            map.extend(p);
        }
    }
    if let Some((key, d)) = display {
        if !d.is_empty() {
            map.insert(
                key.to_string(),
                serde_json::to_value(d).unwrap_or(Value::Null),
            );
        }
    }
    Value::Object(map)
}

impl<'a> SolanaApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a single asset by ID
    pub async fn get_asset(&self, id: &str) -> Result<Asset> {
        self.get_asset_with_options(id, &DisplayOptions::default())
            .await
    }

    /// Get a single asset with display options
    pub async fn get_asset_with_options(
        &self,
        id: &str,
        options: &DisplayOptions,
    ) -> Result<Asset> {
        let params = params_object(
            vec![("id", Value::from(id))],
            None,
            Some(("displayOptions", options)),
        );
        self.client.rpc("getAsset", params).await
    }

    /// Get multiple assets by IDs
    pub async fn get_assets(&self, ids: &[&str]) -> Result<Vec<Asset>> {
        let params = serde_json::json!({ "ids": ids });
        self.client.rpc("getAssets", params).await
    }

    /// Get proof for a compressed asset
    pub async fn get_asset_proof(&self, id: &str) -> Result<AssetProof> {
        let params = serde_json::json!({ "id": id });
        self.client.rpc("getAssetProof", params).await
    }

    /// Get proofs for multiple compressed assets
    pub async fn get_asset_proofs(
        &self,
        ids: &[&str],
    ) -> Result<std::collections::HashMap<String, AssetProof>> {
        let params = serde_json::json!({ "ids": ids });
        self.client.rpc("getAssetProofs", params).await
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
        let params = params_object(
            vec![("ownerAddress", Value::from(owner))],
            Some(pagination),
            Some(("options", display)),
        );
        self.client.rpc("getAssetsByOwner", params).await
    }

    /// Get assets by creator (verified creators only)
    pub async fn get_assets_by_creator(&self, creator: &str) -> Result<GetAssetsResponse> {
        let params = serde_json::json!({
            "creatorAddress": creator,
            "onlyVerified": true
        });
        self.client.rpc("getAssetsByCreator", params).await
    }

    /// Get assets by authority
    pub async fn get_assets_by_authority(&self, authority: &str) -> Result<GetAssetsResponse> {
        let params = serde_json::json!({ "authorityAddress": authority });
        self.client.rpc("getAssetsByAuthority", params).await
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
        self.client.rpc("getAssetsByGroup", params).await
    }

    /// Search assets with filters
    pub async fn search_assets(&self, request: &SearchAssetsRequest) -> Result<GetAssetsResponse> {
        self.client.rpc("searchAssets", request).await
    }

    /// Get token accounts by owner and/or mint
    pub async fn get_token_accounts(
        &self,
        owner: Option<&str>,
        mint: Option<&str>,
    ) -> Result<GetTokenAccountsResponse> {
        self.client
            .rpc("getTokenAccounts", token_accounts_params(owner, mint))
            .await
    }

    /// Get NFT editions of a master edition mint
    pub async fn get_nft_editions(&self, mint: &str) -> Result<GetNftEditionsResponse> {
        let params = serde_json::json!({ "mintAddress": mint });
        self.client.rpc("getNftEditions", params).await
    }

    /// Get asset signatures (transactions)
    pub async fn get_asset_signatures(&self, id: &str) -> Result<GetAssetSignaturesResponse> {
        let params = serde_json::json!({ "id": id });
        self.client.rpc("getAssetSignatures", params).await
    }
}

/// `getTokenAccounts` params (`ownerAddress` / `mintAddress`)
fn token_accounts_params(owner: Option<&str>, mint: Option<&str>) -> Value {
    let mut fields = Vec::new();
    if let Some(o) = owner {
        fields.push(("ownerAddress", Value::from(o)));
    }
    if let Some(m) = mint {
        fields.push(("mintAddress", Value::from(m)));
    }
    params_object(fields, None, None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_assets_by_owner_params_are_by_name() {
        let pagination = PaginationOptions {
            page: Some(2),
            limit: Some(50),
            ..Default::default()
        };
        let display = DisplayOptions {
            show_fungible: Some(true),
            ..Default::default()
        };
        let params = params_object(
            vec![("ownerAddress", Value::from("Owner111"))],
            Some(&pagination),
            Some(("options", &display)),
        );
        assert_eq!(
            params,
            serde_json::json!({
                "ownerAddress": "Owner111",
                "page": 2,
                "limit": 50,
                "options": {"showFungible": true}
            })
        );
    }

    #[test]
    fn get_asset_params_omit_empty_display_options() {
        let params = params_object(
            vec![("id", Value::from("Asset111"))],
            None,
            Some(("displayOptions", &DisplayOptions::default())),
        );
        assert_eq!(params, serde_json::json!({"id": "Asset111"}));
    }

    #[test]
    fn token_accounts_params_use_address_names() {
        assert_eq!(
            token_accounts_params(Some("Owner111"), Some("Mint111")),
            serde_json::json!({"ownerAddress": "Owner111", "mintAddress": "Mint111"})
        );
    }
}
