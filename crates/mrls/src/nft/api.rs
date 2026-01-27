//! NFT API client

use super::types::{
    GetMultipleCollectionsRequest, GetMultipleNftsRequest, HistoricalFloorPrice, Nft,
    NftCollection, NftCollectionStats, NftFloorPrice, NftOwner, NftResponse, NftSalePrice,
    NftSyncStatus, NftTrade, NftTrait, NftTransfer, NftsByTraitsRequest, TraitResyncStatus,
};
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
    #[must_use]
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
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get NFTs owned by an address
    pub async fn get_wallet_nfts(
        &self,
        address: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<Nft>> {
        let path = format!("/{address}/nft");
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
        let path = format!("/{address}/nft/transfers");
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
        let path = format!("/{address}/nft/collections");
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
        let path = format!("/nft/{contract}/transfers");
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
        let path = format!("/nft/{contract}/{token_id}/transfers");
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
        let path = format!("/nft/{contract}/metadata");
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
        let path = format!("/nft/{contract}/owners");
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
        let path = format!("/nft/{contract}/{token_id}/owners");
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
        let path = format!("/nft/{contract}/{token_id}");
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
        let path = format!("/nft/{contract}/stats");
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
        let path = format!("/nft/{contract}/floor-price");
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
        let path = format!("/nft/{contract}/{token_id}/floor-price");
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
        let path = format!("/nft/{contract}/trades");
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
        let path = format!("/nft/{contract}/{token_id}/trades");
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
        let path = format!("/wallets/{address}/nfts/trades");
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
        let path = format!("/nft/{contract}/traits");
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get collection traits with pagination
    pub async fn get_collection_traits_paginated(
        &self,
        contract: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftTrait>> {
        let path = format!("/nft/{contract}/traits/paginate");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
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
        let path = format!("/nft/{contract}/unique-owners");
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
        let path = format!("/nft/{contract}/{token_id}/metadata/resync");
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get multiple NFTs by token addresses and IDs (batch)
    pub async fn get_multiple_nfts(
        &self,
        request: &GetMultipleNftsRequest,
        chain: Option<&str>,
    ) -> Result<Vec<Nft>> {
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client
                .post_with_query("/nft/getMultipleNFTs", request, &query)
                .await
        } else {
            self.client.post("/nft/getMultipleNFTs", request).await
        }
    }

    /// Get NFTs by traits
    pub async fn get_nfts_by_traits(
        &self,
        contract: &str,
        request: &NftsByTraitsRequest,
        chain: Option<&str>,
    ) -> Result<NftResponse<Nft>> {
        let path = format!("/nft/{contract}/nfts-by-traits");
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.post_with_query(&path, request, &query).await
        } else {
            self.client.post(&path, request).await
        }
    }

    /// Get historical floor price for a collection
    pub async fn get_floor_price_historical(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<Vec<HistoricalFloorPrice>> {
        let path = format!("/nft/{contract}/floor-price/historical");
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Sync NFT collection metadata
    pub async fn sync_collection(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<NftSyncStatus> {
        let body = serde_json::json!({});
        let path = if let Some(c) = chain {
            format!("/nft/{contract}/sync?chain={c}")
        } else {
            format!("/nft/{contract}/sync")
        };
        self.client.put(&path, &body).await
    }

    /// Get NFTs by contract address
    pub async fn get_contract_nfts(
        &self,
        contract: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<Nft>> {
        let path = format!("/nft/{contract}");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get metadata for multiple NFT contracts (batch)
    pub async fn get_multiple_collections(
        &self,
        request: &GetMultipleCollectionsRequest,
        chain: Option<&str>,
    ) -> Result<Vec<NftCollection>> {
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client
                .post_with_query("/nft/metadata", request, &query)
                .await
        } else {
            self.client.post("/nft/metadata", request).await
        }
    }

    /// Resync NFT collection traits
    pub async fn resync_traits(
        &self,
        contract: &str,
        chain: Option<&str>,
    ) -> Result<TraitResyncStatus> {
        let path = format!("/nft/{contract}/traits/resync");
        if let Some(chain) = chain {
            let query = NftQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get NFT sale prices by collection
    pub async fn get_collection_prices(
        &self,
        contract: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftSalePrice>> {
        let path = format!("/nft/{contract}/price");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get NFT sale prices by token
    pub async fn get_token_prices(
        &self,
        contract: &str,
        token_id: &str,
        query: Option<&NftQuery>,
    ) -> Result<NftResponse<NftSalePrice>> {
        let path = format!("/nft/{contract}/{token_id}/price");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }
}
