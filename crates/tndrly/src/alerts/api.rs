//! Alerts API operations

use super::types::*;
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Alerts API client
pub struct AlertsApi<'a> {
    client: &'a Client,
}

impl<'a> AlertsApi<'a> {
    /// Create a new Alerts API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new alert
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tndrly::alerts::{CreateAlertRequest, AlertType, AlertTarget, AlertParameters};
    ///
    /// // Create an alert for ERC20 transfers
    /// let request = CreateAlertRequest::new(
    ///     "USDC Transfers",
    ///     AlertType::Erc20Transfer,
    ///     "1",
    ///     AlertTarget::Address,
    /// )
    /// .address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    ///
    /// let alert = client.alerts().create(&request).await?;
    /// ```
    pub async fn create(&self, request: &CreateAlertRequest) -> Result<Alert> {
        self.client.post("/alert", request).await
    }

    /// List all alerts
    pub async fn list(&self) -> Result<ListAlertsResponse> {
        self.client.get("/alerts").await
    }

    /// Get an alert by ID
    pub async fn get(&self, id: &str) -> Result<Alert> {
        self.client
            .get(&format!("/alert/{}", encode_path_segment(id)))
            .await
    }

    /// Update an alert (partial update)
    pub async fn update(&self, id: &str, request: &CreateAlertRequest) -> Result<Alert> {
        self.client
            .patch(&format!("/alert/{}", encode_path_segment(id)), request)
            .await
    }

    /// Delete an alert
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/alert/{}", encode_path_segment(id)))
            .await
    }

    /// Enable an alert
    pub async fn enable(&self, id: &str) -> Result<Alert> {
        let request = serde_json::json!({ "enabled": true });
        self.client
            .patch(&format!("/alert/{}", encode_path_segment(id)), &request)
            .await
    }

    /// Disable an alert
    pub async fn disable(&self, id: &str) -> Result<Alert> {
        let request = serde_json::json!({ "enabled": false });
        self.client
            .patch(&format!("/alert/{}", encode_path_segment(id)), &request)
            .await
    }

    /// Add a destination to an alert
    pub async fn add_destination(
        &self,
        alert_id: &str,
        request: &AddDestinationRequest,
    ) -> Result<AlertDestination> {
        self.client
            .post(
                &format!("/alert/{}/destinations", encode_path_segment(alert_id)),
                request,
            )
            .await
    }

    /// Remove a destination from an alert
    pub async fn remove_destination(&self, alert_id: &str, destination_id: &str) -> Result<()> {
        self.client
            .delete(&format!(
                "/alert/{}/destinations/{}",
                encode_path_segment(alert_id),
                encode_path_segment(destination_id)
            ))
            .await
    }

    // Webhook management

    /// Create a webhook destination
    pub async fn create_webhook(&self, request: &CreateWebhookRequest) -> Result<Webhook> {
        self.client.post("/webhooks", request).await
    }

    /// List all webhooks
    ///
    /// Returns an empty list if no webhooks exist.
    pub async fn list_webhooks(&self) -> Result<ListWebhooksResponse> {
        // API returns null when no webhooks exist
        let response: Option<ListWebhooksResponse> = self.client.get("/webhooks").await?;
        Ok(response.unwrap_or(ListWebhooksResponse {
            webhooks: Vec::new(),
        }))
    }

    /// Get a webhook by ID
    pub async fn get_webhook(&self, id: &str) -> Result<Webhook> {
        self.client
            .get(&format!("/webhooks/{}", encode_path_segment(id)))
            .await
    }

    /// Delete a webhook
    pub async fn delete_webhook(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/webhooks/{}", encode_path_segment(id)))
            .await
    }

    /// Test a webhook by sending a test event
    pub async fn test_webhook(&self, id: &str, tx_hash: &str, network: &str) -> Result<()> {
        let request = serde_json::json!({
            "transaction_hash": tx_hash,
            "network": network
        });
        self.client
            .post_no_response(
                &format!("/webhooks/{}/test", encode_path_segment(id)),
                &request,
            )
            .await
    }

    /// Get alert execution history
    ///
    /// Returns the execution history for all alerts in the project.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get first page of alert history
    /// let history = client.alerts().history(None).await?;
    ///
    /// // Get with pagination
    /// let query = AlertHistoryQuery::new().page(2).per_page(50);
    /// let history = client.alerts().history(Some(query)).await?;
    /// ```
    pub async fn history(&self, query: Option<AlertHistoryQuery>) -> Result<AlertHistoryResponse> {
        match query {
            Some(q) => self.client.get_with_query("/alert-history", &q).await,
            None => self.client.get("/alert-history").await,
        }
    }

    /// Send a test alert
    ///
    /// Triggers an alert with a specific transaction for testing purposes.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let request = TestAlertRequest::new(
    ///     "alert-123",
    ///     "1",
    ///     "0xeb22a8b76c48c912d662bffef2272e9a6413ddbe6da541c84a51c249f04ffaf9"
    /// );
    /// client.alerts().test_alert(&request).await?;
    /// ```
    pub async fn test_alert(&self, request: &TestAlertRequest) -> Result<()> {
        self.client.post_no_response("/test-alerts", request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_alert_request() {
        let request = CreateAlertRequest::new(
            "Test Alert",
            AlertType::SuccessfulTransaction,
            "1",
            AlertTarget::Address,
        )
        .address("0x1234")
        .enabled(true);

        assert_eq!(request.name, "Test Alert");
        assert_eq!(request.alert_type, AlertType::SuccessfulTransaction);
        assert_eq!(request.network, "1");
        assert!(request.addresses.is_some());
        assert!(request.enabled);
    }

    #[test]
    fn test_alert_parameters() {
        let params = AlertParameters::function_call("transfer(address,uint256)");
        assert_eq!(
            params.function_signature,
            Some("transfer(address,uint256)".to_string())
        );

        let params = AlertParameters::whale("1000000000000000000");
        assert_eq!(params.threshold, Some("1000000000000000000".to_string()));
    }

    #[test]
    fn test_add_destination_request() {
        let dest = AddDestinationRequest::webhook("webhook-123");
        assert_eq!(dest.destination_type, DestinationType::Webhook);
        assert_eq!(dest.destination_id, "webhook-123");

        let dest = AddDestinationRequest::slack("https://hooks.slack.com/...");
        assert_eq!(dest.destination_type, DestinationType::Slack);
    }

    #[test]
    fn test_create_alert_request_serialization() {
        // Verify JSON structure matches Tenderly API expectations
        let request = CreateAlertRequest::new(
            "Test Alert",
            AlertType::Erc20Transfer,
            "1",
            AlertTarget::Address,
        )
        .address("0x1234");

        let json = serde_json::to_value(&request).unwrap();

        // Verify field names serialize correctly
        assert!(json["name"].is_string());
        assert_eq!(json["alert_type"], "erc20_transfer"); // snake_case
        assert_eq!(json["network"], "1");
        assert_eq!(json["target"], "address"); // snake_case
        assert!(json["addresses"].is_array());
        assert_eq!(json["enabled"], true);
    }

    #[test]
    fn test_alert_history_query_serialization() {
        // Verify perPage uses correct casing (camelCase for this endpoint)
        let query = AlertHistoryQuery::new().page(2).per_page(50);

        let json = serde_json::to_value(&query).unwrap();

        assert_eq!(json["page"], 2);
        assert_eq!(json["perPage"], 50); // Must be camelCase, not per_page
        assert!(
            json.get("per_page").is_none(),
            "per_page should be renamed to perPage"
        );
    }
}
