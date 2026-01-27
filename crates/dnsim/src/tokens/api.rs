//! Token info API endpoints

use super::types::{TokenInfoOptions, TokensResponse};
use crate::client::Client;
use crate::error::Result;

/// Tokens API
pub struct TokensApi<'a> {
    client: &'a Client,
}

impl<'a> TokensApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token info
    ///
    /// # Arguments
    /// * `address` - Token contract address or "native"
    /// * `options` - Query options (`chain_ids` is required)
    pub async fn get(&self, address: &str, options: &TokenInfoOptions) -> Result<TokensResponse> {
        let path = format!(
            "/v1/evm/token-info/{}{}",
            address,
            options.to_query_string()
        );
        self.client.get(&path).await
    }
}
