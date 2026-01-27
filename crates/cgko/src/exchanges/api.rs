//! Exchanges API endpoints

use super::types::{Exchange, ExchangeListItem, ExchangeTickers, VolumeChart};
use crate::client::Client;
use crate::error::Result;

/// Exchanges API
pub struct ExchangesApi<'a> {
    client: &'a Client,
}

impl<'a> ExchangesApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all exchanges
    pub async fn list(&self) -> Result<Vec<Exchange>> {
        self.client.get("/exchanges").await
    }

    /// List exchanges with pagination
    pub async fn list_paginated(&self, per_page: u32, page: u32) -> Result<Vec<Exchange>> {
        let path = format!("/exchanges?per_page={per_page}&page={page}");
        self.client.get(&path).await
    }

    /// Get exchange ID list (for mapping)
    pub async fn id_list(&self) -> Result<Vec<ExchangeListItem>> {
        self.client.get("/exchanges/list").await
    }

    /// Get exchange data by ID
    pub async fn get(&self, id: &str) -> Result<Exchange> {
        let path = format!("/exchanges/{id}");
        self.client.get(&path).await
    }

    /// Get exchange tickers
    pub async fn tickers(&self, id: &str) -> Result<ExchangeTickers> {
        let path = format!("/exchanges/{id}/tickers");
        self.client.get(&path).await
    }

    /// Get exchange volume chart
    ///
    /// # Arguments
    /// * `id` - Exchange ID
    /// * `days` - Data range (1, 7, 14, 30, 90, 180, 365)
    pub async fn volume_chart(&self, id: &str, days: u32) -> Result<VolumeChart> {
        let path = format!("/exchanges/{id}/volume_chart?days={days}");
        self.client.get(&path).await
    }

    /// Get exchange volume chart by date range (Pro API only)
    ///
    /// # Arguments
    /// * `id` - Exchange ID
    /// * `from` - Unix timestamp start
    /// * `to` - Unix timestamp end
    pub async fn volume_chart_range(&self, id: &str, from: u64, to: u64) -> Result<VolumeChart> {
        let path = format!("/exchanges/{id}/volume_chart/range?from={from}&to={to}");
        self.client.get(&path).await
    }
}
