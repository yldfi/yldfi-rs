//! Market Data API client

use super::types::{GlobalMarketCap, GlobalVolume, TopNftCollection, TopToken};
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for market data endpoints
#[derive(Debug, Default, Serialize)]
pub struct MarketQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl MarketQuery {
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
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// API for market data operations
pub struct MarketApi<'a> {
    client: &'a Client,
}

impl<'a> MarketApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get top ERC20 tokens by market cap
    pub async fn get_top_tokens(&self, query: Option<&MarketQuery>) -> Result<Vec<TopToken>> {
        let path = "/market-data/erc20s/top-tokens";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get top gainers and losers
    pub async fn get_top_movers(&self, query: Option<&MarketQuery>) -> Result<serde_json::Value> {
        let path = "/market-data/erc20s/top-movers";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get top NFT collections
    pub async fn get_top_nft_collections(
        &self,
        query: Option<&MarketQuery>,
    ) -> Result<Vec<TopNftCollection>> {
        let path = "/market-data/nfts/top-collections";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get hottest NFT collections
    pub async fn get_hottest_nft_collections(
        &self,
        query: Option<&MarketQuery>,
    ) -> Result<Vec<TopNftCollection>> {
        let path = "/market-data/nfts/hottest-collections";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get global market cap
    pub async fn get_global_market_cap(&self) -> Result<GlobalMarketCap> {
        self.client.get("/market-data/global/market-cap").await
    }

    /// Get global volume
    pub async fn get_global_volume(&self) -> Result<GlobalVolume> {
        self.client.get("/market-data/global/volume").await
    }
}
