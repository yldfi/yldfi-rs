//! Collectibles API endpoints

use super::types::{CollectiblesResponse, CollectiblesOptions};
use crate::client::Client;
use crate::error::Result;

/// Collectibles API
pub struct CollectiblesApi<'a> {
    client: &'a Client,
}

impl<'a> CollectiblesApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get collectibles (NFTs) for a wallet
    ///
    /// # Arguments
    /// * `address` - Wallet address
    pub async fn get(&self, address: &str) -> Result<CollectiblesResponse> {
        let path = format!("/v1/evm/collectibles/{address}");
        self.client.get(&path).await
    }

    /// Get collectibles with options
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `options` - Query options
    pub async fn get_with_options(
        &self,
        address: &str,
        options: &CollectiblesOptions,
    ) -> Result<CollectiblesResponse> {
        let path = format!(
            "/v1/evm/collectibles/{}{}",
            address,
            options.to_query_string()
        );
        self.client.get(&path).await
    }
}
