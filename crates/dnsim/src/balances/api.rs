//! Balances API endpoints

use super::types::{BalancesResponse, BalancesOptions, SingleBalanceOptions, SingleBalanceResponse};
use crate::client::Client;
use crate::error::Result;

/// Balances API
pub struct BalancesApi<'a> {
    client: &'a Client,
}

impl<'a> BalancesApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token balances for a wallet
    ///
    /// # Arguments
    /// * `address` - Wallet address
    pub async fn get(&self, address: &str) -> Result<BalancesResponse> {
        let path = format!("/v1/evm/balances/{address}");
        self.client.get(&path).await
    }

    /// Get token balances with options
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `options` - Query options
    pub async fn get_with_options(
        &self,
        address: &str,
        options: &BalancesOptions,
    ) -> Result<BalancesResponse> {
        let path = format!("/v1/evm/balances/{}{}", address, options.to_query_string());
        self.client.get(&path).await
    }

    /// Get single token balance
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `token_address` - Token contract address or "native"
    /// * `options` - Query options (`chain_ids` is required)
    pub async fn get_token(
        &self,
        address: &str,
        token_address: &str,
        options: &SingleBalanceOptions,
    ) -> Result<SingleBalanceResponse> {
        let path = format!(
            "/v1/evm/balances/{}/token/{}{}",
            address,
            token_address,
            options.to_query_string()
        );
        self.client.get(&path).await
    }
}
