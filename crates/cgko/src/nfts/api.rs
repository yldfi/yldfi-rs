//! NFT API endpoints

use super::types::{NftListItem, NftListOptions, NftCollection, NftMarketItem, NftTickersResponse, NftMarketChart};
use crate::client::Client;
use crate::error::Result;

/// NFT API
pub struct NftsApi<'a> {
    client: &'a Client,
}

impl<'a> NftsApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all NFT collections
    pub async fn list(&self) -> Result<Vec<NftListItem>> {
        self.client.get("/nfts/list").await
    }

    /// List NFT collections with options
    pub async fn list_with_options(&self, options: &NftListOptions) -> Result<Vec<NftListItem>> {
        let path = format!("/nfts/list{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get NFT collection by ID
    pub async fn get(&self, id: &str) -> Result<NftCollection> {
        let path = format!("/nfts/{id}");
        self.client.get(&path).await
    }

    /// Get NFT collection by contract address
    pub async fn by_contract(
        &self,
        asset_platform_id: &str,
        contract_address: &str,
    ) -> Result<NftCollection> {
        let path = format!("/nfts/{asset_platform_id}/contract/{contract_address}");
        self.client.get(&path).await
    }

    /// Get NFT market data
    pub async fn markets(&self) -> Result<Vec<NftMarketItem>> {
        self.client.get("/nfts/markets").await
    }

    /// Get NFT market data with options
    pub async fn markets_with_options(
        &self,
        options: &NftListOptions,
    ) -> Result<Vec<NftMarketItem>> {
        let path = format!("/nfts/markets{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get NFT collection tickers
    pub async fn tickers(&self, id: &str) -> Result<NftTickersResponse> {
        let path = format!("/nfts/{id}/tickers");
        self.client.get(&path).await
    }

    /// Get NFT collection market chart by ID (Pro API only)
    pub async fn market_chart(&self, id: &str, days: &str) -> Result<NftMarketChart> {
        let path = format!("/nfts/{id}/market_chart?days={days}");
        self.client.get(&path).await
    }

    /// Get NFT collection market chart by contract address (Pro API only)
    pub async fn contract_market_chart(
        &self,
        asset_platform_id: &str,
        contract_address: &str,
        days: &str,
    ) -> Result<NftMarketChart> {
        let path = format!(
            "/nfts/{asset_platform_id}/contract/{contract_address}/market_chart?days={days}"
        );
        self.client.get(&path).await
    }
}
