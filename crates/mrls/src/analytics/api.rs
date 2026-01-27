//! Token Analytics API client

use super::types::{AnalyticsTimeseriesRequest, AnalyticsTimeseries, BatchAnalyticsRequest, BatchAnalyticsResult};
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
    pub async fn get_timeseries(
        &self,
        request: &AnalyticsTimeseriesRequest,
    ) -> Result<Vec<AnalyticsTimeseries>> {
        self.client
            .post("/tokens/analytics/timeseries", request)
            .await
    }

    /// Get batch token analytics
    pub async fn get_batch(
        &self,
        request: &BatchAnalyticsRequest,
    ) -> Result<Vec<BatchAnalyticsResult>> {
        self.client.post("/tokens/analytics", request).await
    }
}
