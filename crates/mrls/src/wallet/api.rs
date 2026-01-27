//! Wallet API client

use super::types::{NativeBalance, TokenBalance, PaginatedResponse, WalletTransaction, NetWorth, ActiveChains, TokenApproval, WalletHistoryEntry, WalletStats, WalletProfitability, TokenProfitability, WalletBalances};
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
    #[must_use] 
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
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get native balance (ETH, MATIC, etc.) for an address
    pub async fn get_native_balance(
        &self,
        address: &str,
        chain: Option<&str>,
    ) -> Result<NativeBalance> {
        let path = format!("/{address}/balance");
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
        let path = format!("/{address}/erc20");
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
        let path = format!("/{address}");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get net worth for an address across all chains
    pub async fn get_net_worth(&self, address: &str) -> Result<NetWorth> {
        let path = format!("/wallets/{address}/net-worth");
        self.client.get(&path).await
    }

    /// Get active chains for an address
    pub async fn get_active_chains(&self, address: &str) -> Result<ActiveChains> {
        let path = format!("/wallets/{address}/chains");
        self.client.get(&path).await
    }

    /// Get token approvals for an address
    pub async fn get_approvals(
        &self,
        address: &str,
        query: Option<&WalletQuery>,
    ) -> Result<PaginatedResponse<TokenApproval>> {
        let path = format!("/wallets/{address}/approvals");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get wallet history (decoded transactions)
    pub async fn get_history(
        &self,
        address: &str,
        query: Option<&WalletQuery>,
    ) -> Result<PaginatedResponse<WalletHistoryEntry>> {
        let path = format!("/wallets/{address}/history");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get wallet tokens with prices
    pub async fn get_tokens(
        &self,
        address: &str,
        query: Option<&WalletQuery>,
    ) -> Result<PaginatedResponse<TokenBalance>> {
        let path = format!("/wallets/{address}/tokens");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get wallet stats
    pub async fn get_stats(&self, address: &str) -> Result<WalletStats> {
        let path = format!("/wallets/{address}/stats");
        self.client.get(&path).await
    }

    /// Get wallet profitability summary
    pub async fn get_profitability_summary(&self, address: &str) -> Result<WalletProfitability> {
        let path = format!("/wallets/{address}/profitability/summary");
        self.client.get(&path).await
    }

    /// Get wallet profitability by token
    pub async fn get_profitability(
        &self,
        address: &str,
        query: Option<&WalletQuery>,
    ) -> Result<PaginatedResponse<TokenProfitability>> {
        let path = format!("/wallets/{address}/profitability");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get balances for multiple wallets (batch)
    pub async fn get_multiple_balances(
        &self,
        wallet_addresses: &[&str],
        chain: Option<&str>,
    ) -> Result<Vec<WalletBalances>> {
        #[derive(Serialize)]
        struct BalancesQuery {
            wallet_addresses: Vec<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            chain: Option<String>,
        }

        let query = BalancesQuery {
            wallet_addresses: wallet_addresses.iter().map(std::string::ToString::to_string).collect(),
            chain: chain.map(std::string::ToString::to_string),
        };

        self.client
            .get_with_query("/wallets/balances", &query)
            .await
    }
}
