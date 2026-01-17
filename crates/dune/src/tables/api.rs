//! Tables API implementation

use super::types::*;
use crate::client::Client;
use crate::error::{self, Error, Result};

/// Tables (uploads) API
pub struct TablesApi<'a> {
    client: &'a Client,
}

impl<'a> TablesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new table
    pub async fn create(&self, request: &CreateTableRequest) -> Result<CreateTableResponse> {
        let url = format!("{}/v1/datasets", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Upload CSV data to create or update a table
    pub async fn upload_csv(&self, request: &UploadCsvRequest) -> Result<UploadCsvResponse> {
        let url = format!("{}/v1/uploads/csv", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// List all tables
    pub async fn list(&self) -> Result<ListTablesResponse> {
        self.list_with_options(&ListTablesOptions::default()).await
    }

    /// List tables with options
    pub async fn list_with_options(
        &self,
        options: &ListTablesOptions,
    ) -> Result<ListTablesResponse> {
        let url = format!(
            "{}/v1/uploads{}",
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

    /// Get a specific table
    pub async fn get(&self, namespace: &str, table_name: &str) -> Result<Table> {
        let url = format!(
            "{}/v1/datasets/{}/{}",
            self.client.base_url(),
            namespace,
            table_name
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!(
                "Table {}/{}",
                namespace, table_name
            )))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Insert rows into a table
    ///
    /// Data should be a JSON array of objects, where each object represents a row.
    pub async fn insert(
        &self,
        namespace: &str,
        table_name: &str,
        data: &serde_json::Value,
    ) -> Result<InsertResponse> {
        let url = format!(
            "{}/v1/uploads/{}/{}/insert",
            self.client.base_url(),
            namespace,
            table_name
        );
        let response = self.client.http().post(&url).json(data).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!(
                "Table {}/{}",
                namespace, table_name
            )))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Clear all data from a table
    pub async fn clear(&self, namespace: &str, table_name: &str) -> Result<ClearTableResponse> {
        let url = format!(
            "{}/v1/uploads/{}/{}/clear",
            self.client.base_url(),
            namespace,
            table_name
        );
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!(
                "Table {}/{}",
                namespace, table_name
            )))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Delete a table
    pub async fn delete(&self, namespace: &str, table_name: &str) -> Result<DeleteTableResponse> {
        let url = format!(
            "{}/v1/uploads/{}/{}",
            self.client.base_url(),
            namespace,
            table_name
        );
        let response = self.client.http().delete(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!(
                "Table {}/{}",
                namespace, table_name
            )))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }
}
