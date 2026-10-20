//! Token Analytics API client

use super::types::{
    AnalyticsTimeseries, AnalyticsTimeseriesRequest, AnalyticsTimeseriesResponse,
    BatchAnalyticsRequest, BatchAnalyticsResult,
};
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for analytics endpoints
#[derive(Debug, Default, Serialize)]
pub struct AnalyticsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
}

impl AnalyticsQuery {
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
    pub fn timeframe(mut self, timeframe: impl Into<String>) -> Self {
        self.timeframe = Some(timeframe.into());
        self
    }

    #[must_use]
    pub fn from_date(mut self, date: impl Into<String>) -> Self {
        self.from_date = Some(date.into());
        self
    }

    #[must_use]
    pub fn to_date(mut self, date: impl Into<String>) -> Self {
        self.to_date = Some(date.into());
        self
    }
}

/// API for token analytics operations
pub struct AnalyticsApi<'a> {
    client: &'a Client,
}

impl<'a> AnalyticsApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get token analytics timeseries (batch)
    ///
    /// `timeframe` is required by Moralis and sent as a query parameter:
    /// `1d`, `7d` or `30d`.
    pub async fn get_timeseries(
        &self,
        request: &AnalyticsTimeseriesRequest,
        timeframe: &str,
    ) -> Result<Vec<AnalyticsTimeseries>> {
        let query = AnalyticsQuery::new().timeframe(timeframe);
        let response: AnalyticsTimeseriesResponse = self
            .client
            .post_with_query("/tokens/analytics/timeseries", request, &query)
            .await?;
        Ok(response.result)
    }

    /// Get batch token analytics
    pub async fn get_batch(
        &self,
        request: &BatchAnalyticsRequest,
    ) -> Result<Vec<BatchAnalyticsResult>> {
        self.client.post("/tokens/analytics", request).await
    }
}

#[cfg(test)]
mod tests {
    use crate::analytics::TokenAnalyticsInput;
    use crate::{Client, Config};
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_timeseries_sends_timeframe_in_query_and_unwraps_result() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/tokens/analytics/timeseries"))
            .and(query_param("timeframe", "7d"))
            .and(body_json(serde_json::json!({
                "tokens": [{"tokenAddress": "0xtoken", "chain": "eth"}]
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "result": [{"chainId": "0x1", "tokenAddress": "0xtoken", "timeseries": []}]
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = Client::with_config(Config::new("key").base_url(server.uri())).unwrap();
        let request = super::AnalyticsTimeseriesRequest {
            tokens: vec![TokenAnalyticsInput {
                token_address: "0xtoken".to_string(),
                chain: "eth".to_string(),
            }],
        };
        let result = client
            .analytics()
            .get_timeseries(&request, "7d")
            .await
            .unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].chain_id.as_deref(), Some("0x1"));
    }
}
