//! Volume API client

use super::types::{CategoryVolume, ChainVolume, VolumeTimeseries};
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for volume endpoints
#[derive(Debug, Default, Serialize)]
pub struct VolumeQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeframe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_date: Option<String>,
}

impl VolumeQuery {
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

/// API for volume analytics
pub struct VolumeApi<'a> {
    client: &'a Client,
}

impl<'a> VolumeApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get volume by chain
    pub async fn get_chains_volume(&self) -> Result<Vec<ChainVolume>> {
        self.client.get("/volume/chains").await
    }

    /// Get volume by category
    pub async fn get_categories_volume(&self) -> Result<Vec<CategoryVolume>> {
        self.client.get("/volume/categories").await
    }

    /// Get overall volume timeseries
    pub async fn get_timeseries(&self, query: Option<&VolumeQuery>) -> Result<VolumeTimeseries> {
        if let Some(q) = query {
            self.client.get_with_query("/volume/timeseries", q).await
        } else {
            self.client.get("/volume/timeseries").await
        }
    }

    /// Get volume timeseries for a category
    pub async fn get_category_timeseries(
        &self,
        category_id: &str,
        query: Option<&VolumeQuery>,
    ) -> Result<VolumeTimeseries> {
        let path = format!("/volume/timeseries/{category_id}");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }
}
