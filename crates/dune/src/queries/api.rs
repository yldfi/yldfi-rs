//! Queries API implementation

use super::types::*;
use crate::client::Client;
use crate::error::{self, Error, Result};

/// Queries API
pub struct QueriesApi<'a> {
    client: &'a Client,
}

impl<'a> QueriesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new query
    pub async fn create(&self, request: &CreateQueryRequest) -> Result<CreateQueryResponse> {
        let url = format!("{}/v1/query", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get a query by ID
    pub async fn get(&self, query_id: i64) -> Result<Query> {
        let url = format!("{}/v1/query/{}", self.client.base_url(), query_id);
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Update a query
    pub async fn update(
        &self,
        query_id: i64,
        request: &UpdateQueryRequest,
    ) -> Result<UpdateQueryResponse> {
        let url = format!("{}/v1/query/{}", self.client.base_url(), query_id);
        let response = self.client.http().patch(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// List queries
    pub async fn list(&self) -> Result<ListQueriesResponse> {
        self.list_with_options(&ListQueriesOptions::default()).await
    }

    /// List queries with options
    pub async fn list_with_options(
        &self,
        options: &ListQueriesOptions,
    ) -> Result<ListQueriesResponse> {
        let url = format!(
            "{}/v1/queries{}",
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

    /// Archive a query
    pub async fn archive(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/archive", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Unarchive a query
    pub async fn unarchive(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/unarchive", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Make a query private
    pub async fn make_private(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/private", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Make a query public (unprivate)
    pub async fn make_public(&self, query_id: i64) -> Result<()> {
        let url = format!("{}/v1/query/{}/unprivate", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(())
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }
}
