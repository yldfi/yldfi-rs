//! Token holders API endpoints

use super::types::{TokenHoldersOptions, TokenHoldersResponse};
use crate::client::Client;
use crate::error::Result;

/// Holders API
pub struct HoldersApi<'a> {
    client: &'a Client,
}

impl<'a> HoldersApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token holders
    ///
    /// # Arguments
    /// * `chain_id` - Chain ID
    /// * `address` - Token contract address
    pub async fn get(&self, chain_id: i64, address: &str) -> Result<TokenHoldersResponse> {
        let path = format!("/v1/evm/token-holders/{chain_id}/{address}");
        self.client.get(&path).await
    }

    /// Get token holders with options
    ///
    /// # Arguments
    /// * `chain_id` - Chain ID
    /// * `address` - Token contract address
    /// * `options` - Query options
    pub async fn get_with_options(
        &self,
        chain_id: i64,
        address: &str,
        options: &TokenHoldersOptions,
    ) -> Result<TokenHoldersResponse> {
        let path = format!(
            "/v1/evm/token-holders/{}/{}{}",
            chain_id,
            address,
            options.to_query_string()
        );
        self.client.get(&path).await
    }
}
