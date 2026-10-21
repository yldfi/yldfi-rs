//! Tables API implementation

use super::types::{
    ClearTableResponse, CreateTableRequest, CreateTableResponse, DeleteTableResponse, InsertFormat,
    InsertResponse, ListTablesOptions, ListTablesResponse, Table, UploadCsvRequest,
    UploadCsvResponse,
};
use crate::client::Client;
use crate::error::{self, Result};
use reqwest::header::CONTENT_TYPE;
use serde::Serialize;

/// Tables (uploads) API
pub struct TablesApi<'a> {
    client: &'a Client,
}

impl<'a> TablesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new, empty uploaded table with a defined schema
    ///
    /// Calls `POST /v1/uploads`.
    pub async fn create(&self, request: &CreateTableRequest) -> Result<CreateTableResponse> {
        let url = format!("{}/v1/uploads", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Upload CSV data to create or update a table
    pub async fn upload_csv(&self, request: &UploadCsvRequest) -> Result<UploadCsvResponse> {
        let url = format!("{}/v1/uploads/csv", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(error::handle_error_response(response).await)
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
            Err(error::handle_error_response(response).await)
        }
    }

    /// Get a specific table
    ///
    /// Calls `GET /v1/datasets/{slug}` where the slug is `{namespace}.{table_name}`.
    pub async fn get(&self, namespace: &str, table_name: &str) -> Result<Table> {
        let url = format!(
            "{}/v1/datasets/{}.{}",
            self.client.base_url(),
            namespace,
            table_name
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Table {namespace}/{table_name}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Insert rows into a table from a raw CSV or NDJSON body
    ///
    /// Calls `POST /v1/uploads/{namespace}/{table_name}/insert` with the
    /// `Content-Type` for `format`. CSV bodies need a header row; NDJSON bodies
    /// hold one JSON object per line.
    pub async fn insert(
        &self,
        namespace: &str,
        table_name: &str,
        format: InsertFormat,
        body: impl Into<Vec<u8>>,
    ) -> Result<InsertResponse> {
        let response = self
            .insert_request(namespace, table_name, format, body.into())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Table {namespace}/{table_name}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Insert rows into a table, serializing each row as one NDJSON line
    pub async fn insert_rows<T: Serialize>(
        &self,
        namespace: &str,
        table_name: &str,
        rows: &[T],
    ) -> Result<InsertResponse> {
        let body = rows_to_ndjson(rows)?;
        self.insert(namespace, table_name, InsertFormat::Ndjson, body)
            .await
    }

    fn insert_request(
        &self,
        namespace: &str,
        table_name: &str,
        format: InsertFormat,
        body: Vec<u8>,
    ) -> reqwest::RequestBuilder {
        let url = format!(
            "{}/v1/uploads/{}/{}/insert",
            self.client.base_url(),
            namespace,
            table_name
        );
        self.client
            .http()
            .post(&url)
            .header(CONTENT_TYPE, format.content_type())
            .body(body)
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
            Err(error::not_found(format!("Table {namespace}/{table_name}")))
        } else {
            Err(error::handle_error_response(response).await)
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
            Err(error::not_found(format!("Table {namespace}/{table_name}")))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }
}

/// Serialize rows as newline-delimited JSON (one object per line)
fn rows_to_ndjson<T: Serialize>(rows: &[T]) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    for row in rows {
        serde_json::to_writer(&mut body, row)?;
        body.push(b'\n');
    }
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client() -> Client {
        Client::with_base_url("test-key", "https://dune.example.test/api").unwrap()
    }

    #[test]
    fn insert_request_sets_content_type_and_raw_body() {
        let client = test_client();
        let csv = "id,name\n1,alpha\n";
        let request = client
            .tables()
            .insert_request("test_ns", "test_table", InsertFormat::Csv, csv.into())
            .build()
            .unwrap();

        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(
            request.url().as_str(),
            "https://dune.example.test/api/v1/uploads/test_ns/test_table/insert"
        );
        assert_eq!(request.headers()[CONTENT_TYPE], "text/csv");
        assert_eq!(
            request.body().and_then(|b| b.as_bytes()),
            Some(csv.as_bytes())
        );
    }

    #[test]
    fn insert_rows_body_is_ndjson() {
        let rows = vec![
            serde_json::json!({"id": 1, "name": "alpha"}),
            serde_json::json!({"id": 2, "name": "beta"}),
        ];
        let body = rows_to_ndjson(&rows).unwrap();
        assert_eq!(
            String::from_utf8(body.clone()).unwrap(),
            "{\"id\":1,\"name\":\"alpha\"}\n{\"id\":2,\"name\":\"beta\"}\n"
        );

        let client = test_client();
        let request = client
            .tables()
            .insert_request("test_ns", "test_table", InsertFormat::Ndjson, body)
            .build()
            .unwrap();
        assert_eq!(request.headers()[CONTENT_TYPE], "application/x-ndjson");
    }
}
