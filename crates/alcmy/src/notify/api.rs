//! Notify/Webhooks API implementation

use super::types::{
    CreateWebhookRequest, GraphqlVariable, ListAddressesResponse, ListNftFiltersResponse,
    ListWebhooksResponse, PatchGraphqlVariableRequest, ReplaceWebhookAddressesRequest,
    UpdateNftFiltersRequest, UpdateWebhookAddressesRequest, UpdateWebhookRequest, Webhook,
};
use crate::client::Client;
use crate::error::{Error, Result};

const NOTIFY_BASE_URL: &str = "https://dashboard.alchemy.com/api";

/// Notify API for webhook management
pub struct NotifyApi<'a> {
    client: &'a Client,
}

impl<'a> NotifyApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    fn auth_token(&self) -> String {
        self.client.api_key().to_string()
    }

    async fn get<R>(&self, path: &str) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{NOTIFY_BASE_URL}{path}");
        let response = self
            .client
            .http()
            .get(&url)
            .header("X-Alchemy-Token", self.auth_token())
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    async fn post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{NOTIFY_BASE_URL}{path}");
        let response = self
            .client
            .http()
            .post(&url)
            .header("X-Alchemy-Token", self.auth_token())
            .json(body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    async fn put<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{NOTIFY_BASE_URL}{path}");
        let response = self
            .client
            .http()
            .put(&url)
            .header("X-Alchemy-Token", self.auth_token())
            .json(body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    async fn patch<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{NOTIFY_BASE_URL}{path}");
        let response = self
            .client
            .http()
            .patch(&url)
            .header("X-Alchemy-Token", self.auth_token())
            .json(body)
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let url = format!("{NOTIFY_BASE_URL}{path}");
        let response = self
            .client
            .http()
            .delete(&url)
            .header("X-Alchemy-Token", self.auth_token())
            .send()
            .await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    // ========== Webhook Methods ==========

    /// List all webhooks for the team
    pub async fn list_webhooks(&self) -> Result<ListWebhooksResponse> {
        self.get("/team-webhooks").await
    }

    /// Create a new webhook
    pub async fn create_webhook(&self, request: &CreateWebhookRequest) -> Result<Webhook> {
        self.post("/create-webhook", request).await
    }

    /// Update a webhook
    pub async fn update_webhook(&self, request: &UpdateWebhookRequest) -> Result<Webhook> {
        self.put("/update-webhook", request).await
    }

    /// Delete a webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        let body = serde_json::json!({ "webhook_id": webhook_id });
        let url = format!("{NOTIFY_BASE_URL}/delete-webhook");
        let response = self
            .client
            .http()
            .delete(&url)
            .header("X-Alchemy-Token", self.auth_token())
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    // ========== Address Methods ==========

    /// List addresses tracked by a webhook
    pub async fn list_webhook_addresses(&self, webhook_id: &str) -> Result<ListAddressesResponse> {
        self.get(&format!("/webhook-addresses?webhook_id={webhook_id}"))
            .await
    }

    /// Add or remove addresses from a webhook (PATCH - idempotent)
    pub async fn update_webhook_addresses(
        &self,
        request: &UpdateWebhookAddressesRequest,
    ) -> Result<()> {
        let _: serde_json::Value = self.patch("/update-webhook-addresses", request).await?;
        Ok(())
    }

    /// Replace all addresses for a webhook (PUT)
    pub async fn replace_webhook_addresses(
        &self,
        request: &ReplaceWebhookAddressesRequest,
    ) -> Result<()> {
        let _: serde_json::Value = self.put("/update-webhook-addresses", request).await?;
        Ok(())
    }

    // ========== NFT Filter Methods ==========

    /// List NFT filters for a webhook
    pub async fn list_nft_filters(&self, webhook_id: &str) -> Result<ListNftFiltersResponse> {
        self.get(&format!("/webhook-nft-filters?webhook_id={webhook_id}"))
            .await
    }

    /// Update NFT filters for a webhook
    pub async fn update_nft_filters(&self, request: &UpdateNftFiltersRequest) -> Result<()> {
        let _: serde_json::Value = self.patch("/update-webhook-nft-filters", request).await?;
        Ok(())
    }

    // ========== GraphQL Variable Methods ==========

    /// Get a GraphQL variable
    pub async fn get_graphql_variable(&self, variable: &str) -> Result<GraphqlVariable> {
        self.get(&format!("/graphql/variables/{variable}")).await
    }

    /// Create a GraphQL variable
    pub async fn create_graphql_variable(
        &self,
        variable: &str,
        values: Vec<String>,
    ) -> Result<GraphqlVariable> {
        let body = serde_json::json!({ "values": values });
        self.post(&format!("/graphql/variables/{variable}"), &body)
            .await
    }

    /// Update a GraphQL variable (add/remove values)
    pub async fn patch_graphql_variable(
        &self,
        variable: &str,
        request: &PatchGraphqlVariableRequest,
    ) -> Result<GraphqlVariable> {
        self.patch(&format!("/graphql/variables/{variable}"), request)
            .await
    }

    /// Delete a GraphQL variable
    pub async fn delete_graphql_variable(&self, variable: &str) -> Result<()> {
        self.delete(&format!("/graphql/variables/{variable}")).await
    }
}
