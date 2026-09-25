//! Token analytics and score client
//!
//! Moralis removed every `/discovery/*` endpoint on 2026-06-04. Only the
//! per-token analytics (`GET /tokens/{address}/analytics`) and score
//! (`GET /tokens/{address}/score`) endpoints remain; they are kept here for
//! backward compatibility. For token discovery use
//! [`TokenApi::search`](crate::token::TokenApi::search) (`GET /tokens/search`),
//! [`TokenApi::get_trending`](crate::token::TokenApi::get_trending)
//! (`GET /tokens/trending`) or the batch
//! [`AnalyticsApi`](crate::analytics::AnalyticsApi) (`POST /tokens/analytics`).

use super::types::{TokenAnalytics, TokenScore};
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
    #[must_use]
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

/// API for per-token analytics and scores
pub struct DiscoveryApi<'a> {
    client: &'a Client,
}

impl<'a> DiscoveryApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token analytics
    pub async fn get_token_analytics(
        &self,
        token_address: &str,
        chain: Option<&str>,
    ) -> Result<TokenAnalytics> {
        let path = format!("/tokens/{token_address}/analytics");
        if let Some(chain) = chain {
            let query = DiscoveryQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Get token score
    ///
    /// Calls `GET /tokens/{address}/score`. Since 2026-07-31 Moralis only
    /// serves token scores for EVM chains.
    pub async fn get_token_score(
        &self,
        token_address: &str,
        chain: Option<&str>,
    ) -> Result<TokenScore> {
        let path = format!("/tokens/{token_address}/score");
        if let Some(chain) = chain {
            let query = DiscoveryQuery::new().chain(chain);
            self.client.get_with_query(&path, &query).await
        } else {
            self.client.get(&path).await
        }
    }
}
