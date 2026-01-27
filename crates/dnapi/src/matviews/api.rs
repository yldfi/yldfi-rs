//! Materialized Views API implementation

use super::types::{
    DeleteMatviewResponse, ListMatviewsOptions, ListMatviewsResponse, Matview,
    RefreshMatviewRequest, RefreshMatviewResponse, UpsertMatviewRequest, UpsertMatviewResponse,
};
use crate::client::Client;
use crate::error::{self, Error, Result};

/// Materialized Views API
pub struct MatviewsApi<'a> {
    client: &'a Client,
}

impl<'a> MatviewsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create or update a materialized view
    pub async fn upsert(&self, request: &UpsertMatviewRequest) -> Result<UpsertMatviewResponse> {
        let url = format!("{}/v1/materialized-views", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get a materialized view by name
    pub async fn get(&self, name: &str) -> Result<Matview> {
        let url = format!("{}/v1/materialized-views/{}", self.client.base_url(), name);
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Matview {name}")))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// List all materialized views
    pub async fn list(&self) -> Result<ListMatviewsResponse> {
        self.list_with_options(&ListMatviewsOptions::default())
            .await
    }

    /// List materialized views with options
    pub async fn list_with_options(
        &self,
        options: &ListMatviewsOptions,
    ) -> Result<ListMatviewsResponse> {
        let url = format!(
            "{}/v1/materialized-views{}",
            self.client.base_url(),
            options.to_query_string()
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Refresh a materialized view
    pub async fn refresh(&self, name: &str) -> Result<RefreshMatviewResponse> {
        self.refresh_with_options(name, &RefreshMatviewRequest::default())
            .await
    }

    /// Refresh a materialized view with options
    pub async fn refresh_with_options(
        &self,
        name: &str,
        request: &RefreshMatviewRequest,
    ) -> Result<RefreshMatviewResponse> {
        let url = format!(
            "{}/v1/materialized-views/{}/refresh",
            self.client.base_url(),
            name
        );
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Matview {name}")))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Delete a materialized view
    pub async fn delete(&self, name: &str) -> Result<DeleteMatviewResponse> {
        let url = format!("{}/v1/materialized-views/{}", self.client.base_url(), name);
        let response = self.client.http().delete(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Matview {name}")))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }
}
