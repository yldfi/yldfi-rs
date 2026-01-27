//! Webhooks API endpoints (Beta)

use super::types::{
    AddressesListOptions, AddressesListResponse, CreateWebhookRequest, ReplaceAddressesRequest,
    UpdateAddressesRequest, UpdateWebhookRequest, Webhook, WebhooksListOptions,
    WebhooksListResponse,
};
use crate::client::Client;
use crate::error::Result;

/// Webhooks API
pub struct WebhooksApi<'a> {
    client: &'a Client,
}

impl<'a> WebhooksApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all webhooks
    pub async fn list(&self) -> Result<WebhooksListResponse> {
        self.client.get("/beta/evm/subscriptions/webhooks").await
    }

    /// List webhooks with options
    pub async fn list_with_options(
        &self,
        options: &WebhooksListOptions,
    ) -> Result<WebhooksListResponse> {
        let path = format!(
            "/beta/evm/subscriptions/webhooks{}",
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Create a webhook
    pub async fn create(&self, request: &CreateWebhookRequest) -> Result<Webhook> {
        self.client
            .post("/beta/evm/subscriptions/webhooks", request)
            .await
    }

    /// Get a specific webhook
    ///
    /// # Arguments
    /// * `webhook_id` - Webhook UUID
    pub async fn get(&self, webhook_id: &str) -> Result<Webhook> {
        let path = format!("/beta/evm/subscriptions/webhooks/{webhook_id}");
        self.client.get(&path).await
    }

    /// Update a webhook
    ///
    /// # Arguments
    /// * `webhook_id` - Webhook UUID
    /// * `request` - Update request
    pub async fn update(
        &self,
        webhook_id: &str,
        request: &UpdateWebhookRequest,
    ) -> Result<Webhook> {
        let path = format!("/beta/evm/subscriptions/webhooks/{webhook_id}");
        self.client.patch(&path, request).await
    }

    /// Delete a webhook
    ///
    /// # Arguments
    /// * `webhook_id` - Webhook UUID
    pub async fn delete(&self, webhook_id: &str) -> Result<()> {
        let path = format!("/beta/evm/subscriptions/webhooks/{webhook_id}");
        self.client.delete_no_content(&path).await
    }

    /// Get webhook addresses
    ///
    /// # Arguments
    /// * `webhook_id` - Webhook UUID
    pub async fn get_addresses(&self, webhook_id: &str) -> Result<AddressesListResponse> {
        let path = format!("/beta/evm/subscriptions/webhooks/{webhook_id}/addresses");
        self.client.get(&path).await
    }

    /// Get webhook addresses with options
    pub async fn get_addresses_with_options(
        &self,
        webhook_id: &str,
        options: &AddressesListOptions,
    ) -> Result<AddressesListResponse> {
        let path = format!(
            "/beta/evm/subscriptions/webhooks/{}/addresses{}",
            webhook_id,
            options.to_query_string()
        );
        self.client.get(&path).await
    }

    /// Replace webhook addresses (replaces entire list)
    ///
    /// # Arguments
    /// * `webhook_id` - Webhook UUID
    /// * `addresses` - New addresses list
    pub async fn replace_addresses(&self, webhook_id: &str, addresses: Vec<String>) -> Result<()> {
        let path = format!("/beta/evm/subscriptions/webhooks/{webhook_id}/addresses");
        let request = ReplaceAddressesRequest { addresses };
        self.client.put_no_content(&path, &request).await
    }

    /// Update webhook addresses (add/remove)
    ///
    /// # Arguments
    /// * `webhook_id` - Webhook UUID
    /// * `request` - Update request with add/remove addresses
    pub async fn update_addresses(
        &self,
        webhook_id: &str,
        request: &UpdateAddressesRequest,
    ) -> Result<()> {
        let path = format!("/beta/evm/subscriptions/webhooks/{webhook_id}/addresses");
        self.client.patch_no_content(&path, request).await
    }
}
