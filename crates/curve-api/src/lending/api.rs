//! Lending API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// API for Curve lending vaults
pub struct LendingApi<'a> {
    client: &'a Client,
}

impl<'a> LendingApi<'a> {
    /// Create a new lending API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get lending vaults from a specific registry on a chain
    pub async fn get(&self, chain: &str, registry: &str) -> Result<LendingVaultsResponse> {
        let path = format!("/getLendingVaults/{}/{}", chain, registry);
        self.client.get(&path).await
    }

    /// Get all lending vaults on a specific chain
    pub async fn get_all_on_chain(&self, chain: &str) -> Result<LendingVaultsResponse> {
        let path = format!("/getLendingVaults/all/{}", chain);
        self.client.get(&path).await
    }

    /// Get all lending vaults across all chains
    pub async fn get_all(&self) -> Result<LendingVaultsResponse> {
        self.client.get("/getLendingVaults/all").await
    }
}
