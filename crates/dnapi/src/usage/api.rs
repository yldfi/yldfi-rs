//! Usage API implementation

use super::types::UsageResponse;
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

    /// Get account usage statistics
    pub async fn get(&self) -> Result<UsageResponse> {
        let url = format!("{}/v1/usage", self.client.base_url());
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }
}
