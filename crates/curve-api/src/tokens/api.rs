//! Tokens API client

use crate::client::Client;
use crate::error::Result;
use super::types::*;

/// API for Curve tokens
pub struct TokensApi<'a> {
    client: &'a Client,
}

impl<'a> TokensApi<'a> {
    /// Create a new tokens API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get all tokens in pools with $10k+ TVL on a chain
    pub async fn get_all(&self, chain: &str) -> Result<TokensResponse> {
        let path = format!("/getTokens/all/{}", chain);
        self.client.get(&path).await
    }
}
