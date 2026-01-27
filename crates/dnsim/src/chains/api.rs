//! Supported chains API endpoints

use super::types::ChainsResponse;
use crate::client::Client;
use crate::error::Result;

/// Chains API
pub struct ChainsApi<'a> {
    client: &'a Client,
}

impl<'a> ChainsApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get list of supported chains
    pub async fn list(&self) -> Result<ChainsResponse> {
        self.client.get("/v1/evm/supported-chains").await
    }
}
