//! Derivatives API endpoints

use super::types::{DerivativeTicker, DerivativesOptions, DerivativeExchange, DerivativeExchangesOptions, DerivativeExchangeDetail, DerivativeExchangeListItem};
use crate::client::Client;
use crate::error::Result;

/// Derivatives API
pub struct DerivativesApi<'a> {
    client: &'a Client,
}

impl<'a> DerivativesApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all derivative tickers
    pub async fn list(&self) -> Result<Vec<DerivativeTicker>> {
        self.client.get("/derivatives").await
    }

    /// List derivative tickers with options
    pub async fn list_with_options(
        &self,
        options: &DerivativesOptions,
    ) -> Result<Vec<DerivativeTicker>> {
        let path = format!("/derivatives{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// List derivatives exchanges
    pub async fn exchanges(&self) -> Result<Vec<DerivativeExchange>> {
        self.client.get("/derivatives/exchanges").await
    }

    /// List derivatives exchanges with options
    pub async fn exchanges_with_options(
        &self,
        options: &DerivativeExchangesOptions,
    ) -> Result<Vec<DerivativeExchange>> {
        let path = format!("/derivatives/exchanges{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get derivatives exchange by ID
    pub async fn exchange(&self, id: &str) -> Result<DerivativeExchangeDetail> {
        let path = format!("/derivatives/exchanges/{id}");
        self.client.get(&path).await
    }

    /// Get derivatives exchange by ID with tickers
    pub async fn exchange_with_tickers(
        &self,
        id: &str,
        include_tickers: &str,
    ) -> Result<DerivativeExchangeDetail> {
        let path = format!(
            "/derivatives/exchanges/{id}?include_tickers={include_tickers}"
        );
        self.client.get(&path).await
    }

    /// List derivatives exchanges (id and name only)
    pub async fn exchanges_list(&self) -> Result<Vec<DerivativeExchangeListItem>> {
        self.client.get("/derivatives/exchanges/list").await
    }
}
