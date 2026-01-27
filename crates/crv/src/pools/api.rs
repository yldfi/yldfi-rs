//! Pools API client

use super::types::{HiddenPoolsResponse, PoolListResponse, PoolsResponse};
use crate::client::Client;
use crate::error::Result;

/// API for Curve pools
pub struct PoolsApi<'a> {
    client: &'a Client,
}

impl<'a> PoolsApi<'a> {
    /// Create a new pools API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get pools from a specific registry on a chain
    ///
    /// # Arguments
    /// * `chain` - Chain ID (e.g., "ethereum", "polygon", "arbitrum")
    /// * `registry` - Registry ID (e.g., "main", "factory", "factory-crypto")
    pub async fn get(&self, chain: &str, registry: &str) -> Result<PoolsResponse> {
        let path = format!("/getPools/{chain}/{registry}");
        self.client.get(&path).await
    }

    /// Get all pools on a specific chain
    pub async fn get_all_on_chain(&self, chain: &str) -> Result<PoolsResponse> {
        let path = format!("/getPools/all/{chain}");
        self.client.get(&path).await
    }

    /// Get all pools across all chains
    pub async fn get_all(&self) -> Result<PoolsResponse> {
        self.client.get("/getPools/all").await
    }

    /// Get pools with TVL >= $10k on a specific chain
    pub async fn get_big(&self, chain: &str) -> Result<PoolsResponse> {
        let path = format!("/getPools/big/{chain}");
        self.client.get(&path).await
    }

    /// Get pools with TVL >= $10k across all chains
    pub async fn get_all_big(&self) -> Result<PoolsResponse> {
        self.client.get("/getPools/big").await
    }

    /// Get pools with TVL < $10k on a specific chain
    pub async fn get_small(&self, chain: &str) -> Result<PoolsResponse> {
        let path = format!("/getPools/small/{chain}");
        self.client.get(&path).await
    }

    /// Get pools with TVL < $10k across all chains
    pub async fn get_all_small(&self) -> Result<PoolsResponse> {
        self.client.get("/getPools/small").await
    }

    /// Get pools with $0 TVL on a specific chain
    pub async fn get_empty(&self, chain: &str) -> Result<PoolsResponse> {
        let path = format!("/getPools/empty/{chain}");
        self.client.get(&path).await
    }

    /// Get pools with $0 TVL across all chains
    pub async fn get_all_empty(&self) -> Result<PoolsResponse> {
        self.client.get("/getPools/empty").await
    }

    /// Get list of pool addresses on a chain
    pub async fn list(&self, chain: &str) -> Result<PoolListResponse> {
        let path = format!("/getPoolList/{chain}");
        self.client.get(&path).await
    }

    /// Get hidden/dysfunctional pools
    pub async fn get_hidden(&self) -> Result<HiddenPoolsResponse> {
        self.client.get("/getHiddenPools").await
    }
}
