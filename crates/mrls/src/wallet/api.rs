//! Wallet API client

use super::types::{
    ActiveChains, NativeBalance, NetWorth, PaginatedResponse, TokenApproval, TokenBalance,
    TokenProfitability, WalletBalances, WalletHistoryEntry, WalletInsight, WalletProfitability,
    WalletStats, WalletTransaction,
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

/// Query parameters for [`WalletApi::get_active_chains_with_query`]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct ActiveChainsQuery {
    /// Chains to check (e.g. `"eth"`, `"0x89"`); empty means the Moralis
    /// default set
    pub chains: Vec<String>,
}

impl ActiveChainsQuery {
    /// Create a new query (Moralis default chains)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chain to check
    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chains.push(chain.into());
        self
    }

    /// Add several chains to check
    #[must_use]
    pub fn chains<I, S>(mut self, chains: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.chains.extend(chains.into_iter().map(Into::into));
        self
    }

    /// Query pairs, with `chains` in the indexed form Moralis expects
    /// (`chains[0]=eth&chains[1]=polygon`)
    fn to_query(&self) -> Vec<(String, String)> {
        indexed_chains(&self.chains)
    }
}

fn indexed_chains(chains: &[String]) -> Vec<(String, String)> {
    chains
        .iter()
        .enumerate()
        .map(|(i, chain)| (format!("chains[{i}]"), chain.clone()))
        .collect()
}

/// Query parameters for [`WalletApi::get_insight`]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct WalletInsightQuery {
    /// Chains to aggregate (e.g. `"eth"`, `"0x89"`); empty means every
    /// supported chain
    pub chains: Vec<String>,
    /// Include the per-chain breakdown (`includeChainBreakdown`)
    pub include_chain_breakdown: Option<bool>,
}

impl WalletInsightQuery {
    /// Create a new query (all chains, no breakdown)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a chain to aggregate
    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chains.push(chain.into());
        self
    }

    /// Include the per-chain breakdown
    #[must_use]
    pub fn include_chain_breakdown(mut self, include: bool) -> Self {
        self.include_chain_breakdown = Some(include);
        self
    }

    /// Query pairs, with `chains` in the indexed form Moralis expects
    /// (`chains[0]=eth&chains[1]=polygon`)
    fn to_query(&self) -> Vec<(String, String)> {
        let mut query = indexed_chains(&self.chains);
        if let Some(include) = self.include_chain_breakdown {
            query.push(("includeChainBreakdown".to_string(), include.to_string()));
        }
        query
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

    /// Get active chains for an address (Moralis default chain set)
    pub async fn get_active_chains(&self, address: &str) -> Result<ActiveChains> {
        self.get_active_chains_with_query(address, &ActiveChainsQuery::default())
            .await
    }

    /// Get active chains for an address, limited to the chains in `query`
    ///
    /// Calls `GET /wallets/{address}/chains`; an empty query uses the
    /// Moralis default chain set.
    pub async fn get_active_chains_with_query(
        &self,
        address: &str,
        query: &ActiveChainsQuery,
    ) -> Result<ActiveChains> {
        let path = format!("/wallets/{address}/chains");
        let pairs = query.to_query();
        if pairs.is_empty() {
            self.client.get(&path).await
        } else {
            self.client.get_with_query(&path, &pairs).await
        }
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

    /// Get wallet insight metrics: activity age, transfer counts,
    /// counterparties, gas spent and swap volume
    ///
    /// Calls `GET /wallets/{address}/insight`. Without a query (or with no
    /// chains) the metrics aggregate every supported chain.
    pub async fn get_insight(
        &self,
        address: &str,
        query: Option<&WalletInsightQuery>,
    ) -> Result<WalletInsight> {
        let path = format!("/wallets/{address}/insight");
        let query = query.map(WalletInsightQuery::to_query).unwrap_or_default();
        self.client.get_with_query(&path, &query).await
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

    #[test]
    fn insight_query_uses_indexed_chains() {
        let q = WalletInsightQuery::new()
            .chain("eth")
            .chain("0x89")
            .include_chain_breakdown(true)
            .to_query();
        let mut url = reqwest::Url::parse("https://example.com/").unwrap();
        url.query_pairs_mut().extend_pairs(&q);
        assert_eq!(
            url.query().unwrap(),
            "chains%5B0%5D=eth&chains%5B1%5D=0x89&includeChainBreakdown=true"
        );
        assert!(WalletInsightQuery::new().to_query().is_empty());
    }

    #[test]
    fn active_chains_query_uses_indexed_chains() {
        let q = ActiveChainsQuery::new()
            .chain("eth")
            .chains(["base", "0x89"])
            .to_query();
        let mut url = reqwest::Url::parse("https://example.com/").unwrap();
        url.query_pairs_mut().extend_pairs(&q);
        assert_eq!(
            url.query().unwrap(),
            "chains%5B0%5D=eth&chains%5B1%5D=base&chains%5B2%5D=0x89"
        );
        assert!(ActiveChainsQuery::new().to_query().is_empty());
    }

    #[tokio::test]
    async fn get_insight_parses_metrics() {
        use wiremock::matchers::{method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/wallets/0xwallet/insight"))
            .and(query_param("chains[0]", "eth"))
            .and(query_param("includeChainBreakdown", "true"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "address": "0xwallet",
                "addressType": "eoa",
                "walletAgeDays": 1200,
                "firstActivityAt": {
                    "chain": "0x1", "blockNumber": "12345678",
                    "blockTimestamp": "2023-01-01T00:00:00.000Z",
                    "transactionHash": "0xhash", "type": "native_transfer",
                    "direction": "received"
                },
                "lastActivityAt": null,
                "activeDays": 300,
                "activeChains": 1,
                "mostActiveChain": "0x1",
                "transactionsInitiated": 500,
                "transactionsInvolved": "800",
                "nativeTransfers": {"sent": 10, "received": 20, "total": 30},
                "erc20Transfers": {"sent": "1", "received": "2", "total": "3"},
                "nftTransfers": {"sent": 0, "received": 0, "total": 0},
                "uniqueCounterparties": {"sentTo": 40, "receivedFrom": 50},
                "swapVolumeUsd": 12345.67,
                "totalGasSpentUsd": "321.5",
                "nativeNetFlowUsd": "-10.25",
                "uniqueTokensInteracted": 25,
                "contractsCreated": 0,
                "chainBreakdown": [{
                    "chain": "0x1",
                    "walletAgeDays": 1200,
                    "totalGasSpentNative": "0.123456789012345678",
                    "totalGasSpentUsd": "321.5",
                    "largestNativeTransferIn": 2.5
                }]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let config = crate::client::Config::new("key").base_url(server.uri());
        let client = crate::client::Client::with_config(config).unwrap();
        let query = WalletInsightQuery::new()
            .chain("eth")
            .include_chain_breakdown(true);
        let insight = client
            .wallet()
            .get_insight("0xwallet", Some(&query))
            .await
            .unwrap();
        assert_eq!(insight.wallet_age_days, Some(1200));
        let first = insight.first_activity_at.unwrap();
        assert_eq!(first.block_number.as_deref(), Some("12345678"));
        assert_eq!(first.activity_type.as_deref(), Some("native_transfer"));
        assert!(insight.last_activity_at.is_none());
        assert_eq!(insight.transactions_involved, Some(800));
        assert_eq!(insight.erc20_transfers.unwrap().total, Some(3));
        assert_eq!(
            insight.unique_counterparties.unwrap().received_from,
            Some(50)
        );
        assert_eq!(insight.swap_volume_usd, Some(12345.67));
        assert_eq!(insight.total_gas_spent_usd, Some(321.5));
        assert_eq!(insight.native_net_flow_usd, Some(-10.25));
        let chain = &insight.chain_breakdown.unwrap()[0];
        assert_eq!(
            chain.total_gas_spent_native.as_deref(),
            Some("0.123456789012345678")
        );
        assert_eq!(chain.largest_native_transfer_in.as_deref(), Some("2.5"));
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
