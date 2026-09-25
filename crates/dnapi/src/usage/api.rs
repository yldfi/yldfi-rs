//! Usage API implementation

use super::types::{UsageRequest, UsageResponse};
use crate::client::Client;
use crate::error::{Error, Result};

/// Usage API
pub struct UsageApi<'a> {
    client: &'a Client,
}

impl<'a> UsageApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get account usage statistics for the current billing period
    ///
    /// Calls `POST /v1/usage` with an empty request body.
    pub async fn get(&self) -> Result<UsageResponse> {
        self.get_with_request(&UsageRequest::default()).await
    }

    /// Get account usage statistics, optionally restricted to a date range
    ///
    /// Calls `POST /v1/usage`. Dates use the `YYYY-MM-DD` format.
    pub async fn get_with_request(&self, request: &UsageRequest) -> Result<UsageResponse> {
        let url = format!("{}/v1/usage", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }
}
