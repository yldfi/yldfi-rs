//! Activity API endpoints

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Activity API
pub struct ActivityApi<'a> {
    client: &'a Client,
}

impl<'a> ActivityApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get wallet activity
    ///
    /// # Arguments
    /// * `address` - Wallet address
    pub async fn get(&self, address: &str) -> Result<ActivityResponse> {
        let path = format!("/v1/evm/activity/{}", address);
        self.client.get(&path).await
    }

    /// Get wallet activity with options
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `options` - Query options (chain_ids, offset, limit)
    pub async fn get_with_options(
        &self,
        address: &str,
        options: &ActivityOptions,
    ) -> Result<ActivityResponse> {
        let path = format!("/v1/evm/activity/{}{}", address, options.to_query_string());
        self.client.get(&path).await
    }
}
