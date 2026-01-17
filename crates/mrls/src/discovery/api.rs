//! Discovery API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for discovery endpoints
#[derive(Debug, Default, Serialize)]
pub struct DiscoveryQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl DiscoveryQuery {
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
}

/// API for token discovery
pub struct DiscoveryApi<'a> {
    client: &'a Client,
}

impl<'a> DiscoveryApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get tokens with rising liquidity
    pub async fn get_rising_liquidity(
        &self,
        query: Option<&DiscoveryQuery>,
    ) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/rising-liquidity";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get tokens with buying pressure
    pub async fn get_buying_pressure(
        &self,
        query: Option<&DiscoveryQuery>,
    ) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/buying-pressure";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get solid performers
    pub async fn get_solid_performers(
        &self,
        query: Option<&DiscoveryQuery>,
    ) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/solid-performers";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get tokens with experienced buyers
    pub async fn get_experienced_buyers(
        &self,
        query: Option<&DiscoveryQuery>,
    ) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/experienced-buyers";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get risky bet tokens
    pub async fn get_risky_bets(
        &self,
        query: Option<&DiscoveryQuery>,
    ) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/risky-bets";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get blue chip tokens
    pub async fn get_blue_chip(&self, query: Option<&DiscoveryQuery>) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/blue-chip";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get top gainers
    pub async fn get_top_gainers(
        &self,
        query: Option<&DiscoveryQuery>,
    ) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/top-gainers";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get top losers
    pub async fn get_top_losers(
        &self,
        query: Option<&DiscoveryQuery>,
    ) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/top-losers";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get trending tokens
    pub async fn get_trending(&self, query: Option<&DiscoveryQuery>) -> Result<DiscoveryResponse> {
        let path = "/discovery/tokens/trending";
        if let Some(q) = query {
            self.client.get_with_query(path, q).await
        } else {
            self.client.get(path).await
        }
    }

    /// Get token analytics
    pub async fn get_token_analytics(
        &self,
        token_address: &str,
        chain: Option<&str>,
    ) -> Result<TokenAnalytics> {
        let path = format!("/tokens/{}/analytics", token_address);
        if let Some(chain) = chain {
            let query = DiscoveryQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get token score
    pub async fn get_token_score(
        &self,
        token_address: &str,
        chain: Option<&str>,
    ) -> Result<TokenScore> {
        let path = format!("/tokens/{}/score", token_address);
        if let Some(chain) = chain {
            let query = DiscoveryQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Filter tokens with custom criteria
    pub async fn filter_tokens(&self, filter: &DiscoveryFilter) -> Result<DiscoveryResponse> {
        self.client.post("/discovery/tokens", filter).await
    }

    /// Get single token details from discovery
    pub async fn get_token(&self, address: &str, chain: Option<&str>) -> Result<DiscoveredToken> {
        #[derive(Serialize)]
        struct TokenQuery {
            address: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            chain: Option<String>,
        }
        let query = TokenQuery {
            address: address.to_string(),
            chain: chain.map(|s| s.to_string()),
        };
        self.client.get_with_query("/discovery/token", &query).await
    }
}
