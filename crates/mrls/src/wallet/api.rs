//! Wallet API client

use super::types::{
    ActiveChains, NativeBalance, NetWorth, PaginatedResponse, TokenApproval, TokenBalance,
    TokenProfitability, WalletBalances, WalletHistoryEntry, WalletProfitability, WalletStats,
    WalletTransaction,
};
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

    /// Get native balances for multiple wallets (batch, max 25 addresses)
    ///
    /// # Errors
    /// Returns a configuration error if `wallet_addresses` is empty or has
    /// more than [`MAX_MULTIPLE_BALANCES_ADDRESSES`] entries.
    pub async fn get_multiple_balances(
        &self,
        wallet_addresses: &[&str],
        chain: Option<&str>,
    ) -> Result<Vec<WalletBalances>> {
        let query = multiple_balances_query(wallet_addresses, chain)?;
        self.client
            .get_with_query("/wallets/balances", &query)
            .await
    }
}

/// Maximum number of addresses accepted by `GET /wallets/balances`
pub const MAX_MULTIPLE_BALANCES_ADDRESSES: usize = 25;

/// Build the query for `GET /wallets/balances`.
///
/// `serde_urlencoded` (used by reqwest's `.query()`) cannot serialize a
/// `Vec` field, so the `wallet_addresses` array is encoded manually using
/// the indexed form Moralis expects: `wallet_addresses[0]=..&wallet_addresses[1]=..`.
fn multiple_balances_query(
    wallet_addresses: &[&str],
    chain: Option<&str>,
) -> Result<Vec<(String, String)>> {
    if wallet_addresses.is_empty() || wallet_addresses.len() > MAX_MULTIPLE_BALANCES_ADDRESSES {
        return Err(crate::error::config(format!(
            "wallet_addresses must contain between 1 and {MAX_MULTIPLE_BALANCES_ADDRESSES} addresses (got {})",
            wallet_addresses.len()
        )));
    }

    let mut query: Vec<(String, String)> = wallet_addresses
        .iter()
        .enumerate()
        .map(|(i, addr)| (format!("wallet_addresses[{i}]"), (*addr).to_string()))
        .collect();
    if let Some(chain) = chain {
        query.push(("chain".to_string(), chain.to_string()));
    }
    Ok(query)
}

#[cfg(test)]
mod multiple_balances_tests {
    use super::*;

    #[test]
    fn query_uses_indexed_array_and_is_url_encodable() {
        let q = multiple_balances_query(&["0xaa", "0xbb"], Some("eth")).unwrap();
        let mut url = reqwest::Url::parse("https://example.com/").unwrap();
        url.query_pairs_mut().extend_pairs(&q);
        let encoded = url.query().unwrap().to_string();
        assert_eq!(
            encoded,
            "wallet_addresses%5B0%5D=0xaa&wallet_addresses%5B1%5D=0xbb&chain=eth"
        );
    }

    #[test]
    fn query_validates_address_count() {
        assert!(multiple_balances_query(&[], None).is_err());
        let many = vec!["0xaa"; MAX_MULTIPLE_BALANCES_ADDRESSES + 1];
        assert!(multiple_balances_query(&many, None).is_err());
        let max = vec!["0xaa"; MAX_MULTIPLE_BALANCES_ADDRESSES];
        assert_eq!(multiple_balances_query(&max, None).unwrap().len(), 25);
    }

    #[tokio::test]
    async fn get_multiple_balances_builds_request() {
        // Previously failed before sending with "builder error: unsupported value".
        // Use an unroutable local URL: we only care that the error is not a
        // request-builder error.
        let config = crate::client::Config::new("k").base_url("http://127.0.0.1:9");
        let client = crate::client::Client::with_config(config).unwrap();
        let err = client
            .wallet()
            .get_multiple_balances(&["0xaa", "0xbb"], None)
            .await
            .unwrap_err();
        assert!(!err.to_string().contains("builder error"), "{err}");
    }
}
