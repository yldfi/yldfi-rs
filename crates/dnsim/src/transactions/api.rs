//! Transactions API endpoints

use super::types::{TransactionsOptions, TransactionsResponse};
use crate::client::Client;
use crate::error::Result;

/// Transactions API
pub struct TransactionsApi<'a> {
    client: &'a Client,
}

impl<'a> TransactionsApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get transactions for a wallet
    ///
    /// # Arguments
    /// * `address` - Wallet address
    pub async fn get(&self, address: &str) -> Result<TransactionsResponse> {
        let path = format!("/v1/evm/transactions/{address}");
        self.client.get(&path).await
    }

    /// Get transactions with options
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `options` - Query options
    pub async fn get_with_options(
        &self,
        address: &str,
        options: &TransactionsOptions,
    ) -> Result<TransactionsResponse> {
        let path = format!(
            "/v1/evm/transactions/{}{}",
            address,
            options.to_query_string()
        );
        self.client.get(&path).await
    }
}
