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

use super::types::{TokenAnalytics, TokenScore, TokenScoreHistory};
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

    /// Get a token's score history
    ///
    /// Calls `GET /tokens/{address}/score/historical`. `timeframe` is `1d`,
    /// `7d` or `30d`.
    pub async fn get_token_score_history(
        &self,
        token_address: &str,
        timeframe: &str,
        chain: Option<&str>,
    ) -> Result<TokenScoreHistory> {
        let path = format!("/tokens/{token_address}/score/historical");
        let mut query = vec![("timeframe", timeframe)];
        if let Some(chain) = chain {
            query.push(("chain", chain));
        }
        self.client.get_with_query(&path, &query).await
    }
}

#[cfg(test)]
mod tests {
    use crate::{Client, Config};
    use wiremock::matchers::{header, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn token_score_history_sends_timeframe_and_chain() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/tokens/0xtoken/score/historical"))
            .and(query_param("timeframe", "7d"))
            .and(query_param("chain", "base"))
            .and(header("X-API-Key", "key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "chainId": "0x2105",
                "tokenAddress": "0xtoken",
                "timeseries": [
                    {"timestamp": "2026-11-01T00:00:00.000Z", "score": 81},
                    {"timestamp": "2026-11-02T00:00:00.000Z", "score": "82.5"}
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::with_config(Config::new("key").base_url(server.uri())).unwrap();
        let history = client
            .discovery()
            .get_token_score_history("0xtoken", "7d", Some("base"))
            .await
            .unwrap();
        assert_eq!(history.chain_id.as_deref(), Some("0x2105"));
        let scores: Vec<_> = history.timeseries.iter().map(|p| p.score).collect();
        assert_eq!(scores, vec![Some(81.0), Some(82.5)]);
    }
}
