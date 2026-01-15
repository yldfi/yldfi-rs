//! DeFi API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for DeFi endpoints
#[derive(Debug, Default, Serialize)]
pub struct DefiQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchange: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_block: Option<String>,
}

impl DefiQuery {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chain = Some(chain.into());
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn exchange(mut self, exchange: impl Into<String>) -> Self {
        self.exchange = Some(exchange.into());
        self
    }
}

/// API for DeFi operations
pub struct DefiApi<'a> {
    client: &'a Client,
}

impl<'a> DefiApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get price between two tokens in a pair
    pub async fn get_pair_price(
        &self,
        token0: &str,
        token1: &str,
        query: Option<&DefiQuery>,
    ) -> Result<PairPrice> {
        let path = format!("/{}/{}/price", token0, token1);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get reserves for a pair
    pub async fn get_pair_reserves(
        &self,
        pair_address: &str,
        chain: Option<&str>,
    ) -> Result<PairReserves> {
        let path = format!("/{}/reserves", pair_address);
        if let Some(chain) = chain {
            let query = DefiQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get pair address for two tokens
    pub async fn get_pair_address(
        &self,
        token0: &str,
        token1: &str,
        query: Option<&DefiQuery>,
    ) -> Result<PairAddress> {
        let path = format!("/{}/{}/pairAddress", token0, token1);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get DeFi summary for a wallet
    pub async fn get_wallet_defi_summary(&self, address: &str) -> Result<DefiSummary> {
        let path = format!("/wallets/{}/defi/summary", address);
        self.client.get(&path).await
    }

    /// Get all DeFi positions for a wallet
    pub async fn get_wallet_defi_positions(
        &self,
        address: &str,
        query: Option<&DefiQuery>,
    ) -> Result<DefiPositionsResponse> {
        let path = format!("/wallets/{}/defi/positions", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get DeFi positions for a specific protocol
    pub async fn get_wallet_protocol_positions(
        &self,
        address: &str,
        protocol: &str,
        query: Option<&DefiQuery>,
    ) -> Result<DefiPositionsResponse> {
        let path = format!("/wallets/{}/defi/{}/positions", address, protocol);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }
}
