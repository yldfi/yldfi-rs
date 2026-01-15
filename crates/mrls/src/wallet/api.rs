//! Wallet API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for wallet endpoints
#[derive(Debug, Default, Serialize)]
pub struct WalletQuery {
    /// Chain to query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    /// Pagination cursor
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl WalletQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Set chain
    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chain = Some(chain.into());
        self
    }

    /// Set cursor
    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    /// Set limit
    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// API for wallet operations
pub struct WalletApi<'a> {
    client: &'a Client,
}

impl<'a> WalletApi<'a> {
    /// Create a new wallet API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get native balance (ETH, MATIC, etc.) for an address
    pub async fn get_native_balance(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<NativeBalance> {
        let path = format!("/{}/balance", address);
        if let Some(chain) = chain {
            let query = WalletQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get all token balances for an address
    pub async fn get_token_balances(
        &self,
        address: &str,
        query: Option<&WalletQuery>,
    ) -> Result<Vec<TokenBalance>> {
        let path = format!("/{}/erc20", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get transactions for an address
    pub async fn get_transactions(
        &self,
        address: &str,
        query: Option<&WalletQuery>,
    ) -> Result<PaginatedResponse<WalletTransaction>> {
        let path = format!("/{}", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get net worth for an address across all chains
    pub async fn get_net_worth(&self, address: &str) -> Result<NetWorth> {
        let path = format!("/wallets/{}/net-worth", address);
        self.client.get(&path).await
    }

    /// Get active chains for an address
    pub async fn get_active_chains(&self, address: &str) -> Result<ActiveChains> {
        let path = format!("/wallets/{}/chains", address);
        self.client.get(&path).await
    }
}
