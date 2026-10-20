//! Notify/Webhooks API implementation

use super::types::{
    CreateWebhookRequest, DataEnvelope, GraphqlVariable, GraphqlVariableRequest,
    ListAddressesResponse, ListNftFiltersResponse, ListWebhooksResponse, NotifyPageOptions,
    PatchGraphqlVariableRequest, ReplaceWebhookAddressesRequest, UpdateNftFiltersRequest,
    UpdateWebhookAddressesRequest, UpdateWebhookRequest, Webhook,
};
use crate::client::Client;
use crate::error::{self, Result};
use reqwest::{Method, RequestBuilder};

const NOTIFY_BASE_URL: &str = "https://dashboard.alchemy.com/api";

/// Notify API for webhook management
pub struct NotifyApi<'a> {
    client: &'a Client,
}

impl<'a> NotifyApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Build an authenticated request.
    ///
    /// The Notify API authenticates with the dashboard Auth Token
    /// (`X-Alchemy-Token`), not the app API key.
    fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = format!("{NOTIFY_BASE_URL}{path}");
        Ok(self
            .client
            .http()
            .request(method, url)
            .header("X-Alchemy-Token", self.client.notify_token()?))
    }

    /// Send a request and decode the JSON response
    async fn send_json<R>(&self, request: RequestBuilder) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let response = request.send().await.map_err(error::http)?;
        if response.status().is_success() {
            response.json().await.map_err(error::http)
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Send a request whose success body carries nothing useful
    async fn send_unit(&self, request: RequestBuilder) -> Result<()> {
        let response = request.send().await.map_err(error::http)?;
        if response.status().is_success() {
            Ok(())
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    // ========== Webhook Methods ==========

    /// List all webhooks for the team
    pub async fn list_webhooks(&self) -> Result<ListWebhooksResponse> {
        self.send_json(self.request(Method::GET, "/team-webhooks")?)
            .await
    }

    /// Create a new webhook
    pub async fn create_webhook(&self, request: &CreateWebhookRequest) -> Result<Webhook> {
        let req = self.request(Method::POST, "/create-webhook")?.json(request);
        let envelope: DataEnvelope<Webhook> = self.send_json(req).await?;
        Ok(envelope.data)
    }

    /// Update a webhook
    pub async fn update_webhook(&self, request: &UpdateWebhookRequest) -> Result<Webhook> {
        let req = self.request(Method::PUT, "/update-webhook")?.json(request);
        let envelope: DataEnvelope<Webhook> = self.send_json(req).await?;
        Ok(envelope.data)
    }

    fn delete_webhook_request(&self, webhook_id: &str) -> Result<RequestBuilder> {
        Ok(self
            .request(Method::DELETE, "/delete-webhook")?
            .query(&[("webhook_id", webhook_id)]))
    }

    /// Delete a webhook
    pub async fn delete_webhook(&self, webhook_id: &str) -> Result<()> {
        self.send_unit(self.delete_webhook_request(webhook_id)?)
            .await
    }

    // ========== Address Methods ==========

    fn list_webhook_addresses_request(
        &self,
        webhook_id: &str,
        page: &NotifyPageOptions,
    ) -> Result<RequestBuilder> {
        Ok(self
            .request(Method::GET, "/webhook-addresses")?
            .query(&[("webhook_id", webhook_id)])
            .query(&page.to_query_params()))
    }

    /// List addresses tracked by a webhook (first page)
    pub async fn list_webhook_addresses(&self, webhook_id: &str) -> Result<ListAddressesResponse> {
        self.list_webhook_addresses_with_options(webhook_id, &NotifyPageOptions::default())
            .await
    }

    /// List addresses tracked by a webhook, one page at a time
    ///
    /// Pass `pagination.cursors.after` from the previous response as `after`.
    pub async fn list_webhook_addresses_with_options(
        &self,
        webhook_id: &str,
        page: &NotifyPageOptions,
    ) -> Result<ListAddressesResponse> {
        self.send_json(self.list_webhook_addresses_request(webhook_id, page)?)
            .await
    }

    /// Add or remove addresses from a webhook (PATCH - idempotent)
    pub async fn update_webhook_addresses(
        &self,
        request: &UpdateWebhookAddressesRequest,
    ) -> Result<()> {
        let req = self
            .request(Method::PATCH, "/update-webhook-addresses")?
            .json(request);
        self.send_unit(req).await
    }

    /// Replace all addresses for a webhook (PUT)
    pub async fn replace_webhook_addresses(
        &self,
        request: &ReplaceWebhookAddressesRequest,
    ) -> Result<()> {
        let req = self
            .request(Method::PUT, "/update-webhook-addresses")?
            .json(request);
        self.send_unit(req).await
    }

    // ========== NFT Filter Methods ==========

    /// List NFT filters for a webhook (first page)
    pub async fn list_nft_filters(&self, webhook_id: &str) -> Result<ListNftFiltersResponse> {
        self.list_nft_filters_with_options(webhook_id, &NotifyPageOptions::default())
            .await
    }

    /// List NFT filters for a webhook, one page at a time
    pub async fn list_nft_filters_with_options(
        &self,
        webhook_id: &str,
        page: &NotifyPageOptions,
    ) -> Result<ListNftFiltersResponse> {
        let req = self
            .request(Method::GET, "/webhook-nft-filters")?
            .query(&[("webhook_id", webhook_id)])
            .query(&page.to_query_params());
        self.send_json(req).await
    }

    /// Update NFT filters for a webhook
    pub async fn update_nft_filters(&self, request: &UpdateNftFiltersRequest) -> Result<()> {
        let req = self
            .request(Method::PATCH, "/update-webhook-nft-filters")?
            .json(request);
        self.send_unit(req).await
    }

    // ========== GraphQL Variable Methods ==========

    /// Get the elements of a GraphQL variable (first page)
    pub async fn get_graphql_variable(&self, variable: &str) -> Result<GraphqlVariable> {
        self.get_graphql_variable_with_options(variable, &NotifyPageOptions::default())
            .await
    }

    /// Get the elements of a GraphQL variable, one page at a time
    pub async fn get_graphql_variable_with_options(
        &self,
        variable: &str,
        page: &NotifyPageOptions,
    ) -> Result<GraphqlVariable> {
        let req = self
            .request(Method::GET, &format!("/graphql/variables/{variable}"))?
            .query(&page.to_query_params());
        self.send_json(req).await
    }

    fn create_graphql_variable_request(
        &self,
        variable: &str,
        items: Vec<String>,
    ) -> Result<RequestBuilder> {
        Ok(self
            .request(Method::POST, &format!("/graphql/variables/{variable}"))?
            .json(&GraphqlVariableRequest { items }))
    }

    /// Create a GraphQL variable, or append items to an existing one
    pub async fn create_graphql_variable(&self, variable: &str, items: Vec<String>) -> Result<()> {
        self.send_unit(self.create_graphql_variable_request(variable, items)?)
            .await
    }

    /// Update a GraphQL variable (add/delete elements)
    pub async fn patch_graphql_variable(
        &self,
        variable: &str,
        request: &PatchGraphqlVariableRequest,
    ) -> Result<()> {
        let req = self
            .request(Method::PATCH, &format!("/graphql/variables/{variable}"))?
            .json(request);
        self.send_unit(req).await
    }

    /// Delete a GraphQL variable
    pub async fn delete_graphql_variable(&self, variable: &str) -> Result<()> {
        self.send_unit(self.request(Method::DELETE, &format!("/graphql/variables/{variable}"))?)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::NotifyPageOptions;
    use crate::{Client, Config, Network};

    fn client() -> Client {
        Client::with_config(
            Config::new("app-key", Network::EthMainnet).with_notify_token("notify-token"),
        )
        .unwrap()
    }

    #[test]
    fn delete_webhook_sends_id_as_encoded_query_without_body() {
        let client = client();
        let req = client
            .notify()
            .delete_webhook_request("wh 1&x=y")
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(req.method(), reqwest::Method::DELETE);
        assert_eq!(req.url().path(), "/api/delete-webhook");
        assert_eq!(req.url().query(), Some("webhook_id=wh+1%26x%3Dy"));
        assert!(req.body().is_none());
        assert_eq!(req.headers()["X-Alchemy-Token"], "notify-token");
    }

    #[test]
    fn list_addresses_encodes_webhook_id() {
        let client = client();
        let req = client
            .notify()
            .list_webhook_addresses_request("wh_1#frag", &NotifyPageOptions::default())
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(req.url().path(), "/api/webhook-addresses");
        assert_eq!(req.url().query(), Some("webhook_id=wh_1%23frag"));
    }

    #[test]
    fn list_addresses_sends_pagination_params() {
        let client = client();
        let page = NotifyPageOptions {
            limit: Some(50),
            after: Some("cur 1".into()),
            page_key: None,
        };
        let req = client
            .notify()
            .list_webhook_addresses_request("wh_1", &page)
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(
            req.url().query(),
            Some("webhook_id=wh_1&limit=50&after=cur+1")
        );
    }

    #[test]
    fn create_graphql_variable_body_uses_items() {
        let client = client();
        let req = client
            .notify()
            .create_graphql_variable_request("myVar", vec!["0x1".into()])
            .unwrap()
            .build()
            .unwrap();
        assert_eq!(req.url().path(), "/api/graphql/variables/myVar");
        let body: serde_json::Value =
            serde_json::from_slice(req.body().unwrap().as_bytes().unwrap()).unwrap();
        assert_eq!(body, serde_json::json!({ "items": ["0x1"] }));
    }
}
