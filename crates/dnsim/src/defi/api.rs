//! `DeFi` positions API endpoints

use super::types::{DefiPositionsResponse, DefiPositionsOptions};
use crate::client::Client;
use crate::error::Result;

/// `DeFi` API
pub struct DefiApi<'a> {
    client: &'a Client,
}

impl<'a> DefiApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get `DeFi` positions for a wallet (Beta)
    ///
    /// Note: This endpoint is temporarily unavailable during rearchitecting.
    ///
    /// # Arguments
    /// * `address` - Wallet address
    pub async fn positions(&self, address: &str) -> Result<DefiPositionsResponse> {
        let path = format!("/beta/evm/defi/positions/{address}");
        self.client.get(&path).await
    }

    /// Get `DeFi` positions with options (Beta)
    ///
    /// Note: This endpoint is temporarily unavailable during rearchitecting.
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `options` - Query options
    pub async fn positions_with_options(
        &self,
        address: &str,
        options: &DefiPositionsOptions,
    ) -> Result<DefiPositionsResponse> {
        let path = format!(
            "/beta/evm/defi/positions/{}{}",
            address,
            options.to_query_string()
        );
        self.client.get(&path).await
    }
}
