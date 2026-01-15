//! NFT API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for NFT endpoints
#[derive(Debug, Default, Serialize)]
pub struct NftQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalise_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_items: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_spam: Option<bool>,
}

impl NftQuery {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chain = Some(chain.into());
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn exclude_spam(mut self, exclude: bool) -> Self {
        self.exclude_spam = Some(exclude);
        self
    }
}

/// API for NFT operations
pub struct NftApi<'a> {
    client: &'a Client,
}

impl<'a> NftApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get NFTs owned by an address
    pub async fn get_wallet_nfts(
        &self,
        address: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<Nft>> {
        let path = format!("/{}/nft", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get NFT transfers by wallet
    pub async fn get_wallet_nft_transfers(
        &self,
        address: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftTransfer>> {
        let path = format!("/{}/nft/transfers", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get NFT collections owned by an address
    pub async fn get_wallet_collections(
        &self,
        address: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftCollection>> {
        let path = format!("/{}/nft/collections", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get NFT transfers by contract
    pub async fn get_contract_transfers(
        &self,
        contract: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftTransfer>> {
        let path = format!("/nft/{}/transfers", contract);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get NFT transfers for a specific token
    pub async fn get_token_transfers(
        &self,
        contract: &str,
        token_id: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftTransfer>> {
        let path = format!("/nft/{}/{}/transfers", contract, token_id);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get collection metadata
    pub async fn get_collection_metadata(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<NftCollection> {
        let path = format!("/nft/{}/metadata", contract);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get owners of NFTs in a collection
    pub async fn get_collection_owners(
        &self,
        contract: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftOwner>> {
        let path = format!("/nft/{}/owners", contract);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get owners of a specific NFT token
    pub async fn get_token_owners(
        &self,
        contract: &str,
        token_id: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftOwner>> {
        let path = format!("/nft/{}/{}/owners", contract, token_id);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get NFT metadata for a specific token
    pub async fn get_nft_metadata(
        &self,
        contract: &str,
        token_id: &str,
        chain: Option<&str>,
    ) -> Result<Nft> {
        let path = format!("/nft/{}/{}", contract, token_id);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get collection stats
    pub async fn get_collection_stats(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<NftCollectionStats> {
        let path = format!("/nft/{}/stats", contract);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get floor price for a collection
    pub async fn get_floor_price(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<NftFloorPrice> {
        let path = format!("/nft/{}/floor-price", contract);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get floor price for a specific token
    pub async fn get_token_floor_price(
        &self,
        contract: &str,
        token_id: &str,
        chain: Option<&str>,
    ) -> Result<NftFloorPrice> {
        let path = format!("/nft/{}/{}/floor-price", contract, token_id);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get trades for a collection
    pub async fn get_collection_trades(
        &self,
        contract: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftTrade>> {
        let path = format!("/nft/{}/trades", contract);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get trades for a specific token
    pub async fn get_token_trades(
        &self,
        contract: &str,
        token_id: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftTrade>> {
        let path = format!("/nft/{}/{}/trades", contract, token_id);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get wallet NFT trades
    pub async fn get_wallet_trades(
        &self,
        address: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftTrade>> {
        let path = format!("/wallets/{}/nfts/trades", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get collection traits
    pub async fn get_collection_traits(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<Vec<NftTrait>> {
        let path = format!("/nft/{}/traits", contract);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get unique owners count
    pub async fn get_unique_owners(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = format!("/nft/{}/unique-owners", contract);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Resync NFT metadata
    pub async fn resync_metadata(
        &self,
        contract: &str,
        token_id: &str,
        chain: Option<&str>,
    ) -> Result<serde_json::Value> {
        let path = format!("/nft/{}/{}/metadata/resync", contract, token_id);
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }
}
