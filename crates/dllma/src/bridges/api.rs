//! Bridges API endpoints (Pro)

use crate::client::Client;
use crate::error::Result;

use super::types::{
    BridgeDetail, BridgeTransactionsResponse, BridgesResponse, ChainBridgeVolume, DailyBridgeStats,
    ListBridgesOptions, TransactionsOptions,
};

/// Bridges API client (Pro only)
pub struct BridgesApi<'a> {
    client: &'a Client,
}

impl<'a> BridgesApi<'a> {
    /// Create a new Bridges API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all bridges
    ///
    /// **Requires Pro API key**
    pub async fn list(&self) -> Result<BridgesResponse> {
        self.client.get_pro("/bridges/bridges").await
    }

    /// List all bridges with options
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `options` - Query options (include chains)
    pub async fn list_with_options(&self, options: &ListBridgesOptions) -> Result<BridgesResponse> {
        let path = format!("/bridges/bridges{}", options.to_query_string());
        self.client.get_pro(&path).await
    }

    /// Get detailed bridge data
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `id` - Bridge ID
    pub async fn get(&self, id: u64) -> Result<BridgeDetail> {
        let path = format!("/bridges/bridge/{id}");
        self.client.get_pro(&path).await
    }

    /// Get bridge volume for a specific chain
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "Arbitrum")
    pub async fn chain_volume(&self, chain: &str) -> Result<ChainBridgeVolume> {
        let path = format!("/bridges/bridgevolume/{chain}");
        self.client.get_pro(&path).await
    }

    /// Get daily bridge stats for a chain
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Unix timestamp for the day
    /// * `chain` - Chain name
    pub async fn daily_stats(&self, timestamp: u64, chain: &str) -> Result<DailyBridgeStats> {
        let path = format!("/bridges/bridgedaystats/{timestamp}/{chain}");
        self.client.get_pro(&path).await
    }

    /// Get bridge transactions
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `id` - Bridge ID
    pub async fn transactions(&self, id: u64) -> Result<BridgeTransactionsResponse> {
        let path = format!("/bridges/transactions/{id}");
        self.client.get_pro(&path).await
    }

    /// Get bridge transactions with options
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `id` - Bridge ID
    /// * `options` - Query options (limit, time range, filters)
    pub async fn transactions_with_options(
        &self,
        id: u64,
        options: &TransactionsOptions,
    ) -> Result<BridgeTransactionsResponse> {
        let path = format!("/bridges/transactions/{}{}", id, options.to_query_string());
        self.client.get_pro(&path).await
    }
}
