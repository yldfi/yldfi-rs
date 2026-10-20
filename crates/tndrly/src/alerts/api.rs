//! Alerts API operations

use super::types::{
    Alert, AlertHistoryQuery, AlertHistoryResponse, AlertResponse, CreateAlertRequest,
    CreateWebhookRequest, ListAlertsResponse, ListWebhooksResponse, TestAlertRequest, Webhook,
};
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Alerts API client
pub struct AlertsApi<'a> {
    client: &'a Client,
}

impl<'a> AlertsApi<'a> {
    /// Create a new Alerts API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new alert
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tndrly::alerts::{AlertExpression, AlertType, CreateAlertRequest};
    ///
    /// // Alert on failed transactions to a contract on mainnet
    /// let request = CreateAlertRequest::new("Failed USDC txs", AlertType::FailedTx)
    ///     .address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
    ///     .network("1")
    ///     .expression(AlertExpression::tx_status(false))
    ///     .delivery_channel("channel-id");
    ///
    /// let alert = client.alerts().create(&request).await?;
    /// ```
    pub async fn create(&self, request: &CreateAlertRequest) -> Result<Alert> {
        let response: AlertResponse = self.client.post("/alert", request).await?;
        Ok(response.alert)
    }

    /// List all alerts
    pub async fn list(&self) -> Result<ListAlertsResponse> {
        self.client.get("/alerts").await
    }

    /// Get an alert by ID
    ///
    /// `GET /alert/{id}` is not in Tenderly's published OpenAPI spec; both a
    /// bare alert and an `{"alert": ...}` envelope are accepted.
    pub async fn get(&self, id: &str) -> Result<Alert> {
        let response: AlertOrEnvelope = self
            .client
            .get(&format!("/alert/{}", encode_path_segment(id)))
            .await?;
        Ok(response.into_alert())
    }

    /// Update an alert
    ///
    /// The API requires the full alert definition (`name`, `simple_type`,
    /// `expressions`, `delivery_channels`), not a partial patch.
    pub async fn update(&self, id: &str, request: &CreateAlertRequest) -> Result<Alert> {
        let response: AlertOrEnvelope = self
            .client
            .patch(&format!("/alert/{}", encode_path_segment(id)), request)
            .await?;
        Ok(response.into_alert())
    }

    /// Delete an alert
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/alert/{}", encode_path_segment(id)))
            .await
    }

    /// Enable an alert
    ///
    /// Sends a partial `PATCH {"enabled": true}`. The published spec marks the
    /// full alert definition as required on `PATCH`; if the API rejects this,
    /// use [`update`](Self::update) with the full definition.
    pub async fn enable(&self, id: &str) -> Result<Alert> {
        self.set_enabled(id, true).await
    }

    /// Disable an alert
    ///
    /// See [`enable`](Self::enable) for the partial-`PATCH` caveat.
    pub async fn disable(&self, id: &str) -> Result<Alert> {
        self.set_enabled(id, false).await
    }

    async fn set_enabled(&self, id: &str, enabled: bool) -> Result<Alert> {
        let request = serde_json::json!({ "enabled": enabled });
        let response: AlertOrEnvelope = self
            .client
            .patch(&format!("/alert/{}", encode_path_segment(id)), &request)
            .await?;
        Ok(response.into_alert())
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

/// `{"alert": ...}` envelope (spec) or a bare alert
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum AlertOrEnvelope {
    Envelope(AlertResponse),
    Bare(Alert),
}

impl AlertOrEnvelope {
    fn into_alert(self) -> Alert {
        match self {
            Self::Envelope(r) => r.alert,
            Self::Bare(a) => a,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alerts::{AlertExpression, AlertType, ExpressionType};

    #[test]
    fn test_create_alert_request_serialization() {
        // POST /alert requires name, simple_type, expressions and delivery_channels
        let request = CreateAlertRequest::new("Failed txs", AlertType::FailedTx)
            .address("0x0000000000000000000000000000000000000001")
            .network("1")
            .expression(AlertExpression::tx_status(false))
            .delivery_channel("00000000-0000-0000-0000-000000000001");

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["name"], "Failed txs");
        assert_eq!(json["simple_type"], "failed_tx");
        assert_eq!(json["enabled"], true);
        assert_eq!(json["expressions"][0]["type"], "contract_address");
        assert_eq!(
            json["expressions"][0]["expression"],
            serde_json::json!({ "address": "0x0000000000000000000000000000000000000001" })
        );
        assert_eq!(json["expressions"][1]["type"], "network");
        assert_eq!(json["expressions"][1]["expression"]["network_id"], "1");
        assert_eq!(json["expressions"][2]["type"], "tx_status");
        assert_eq!(
            json["expressions"][2]["expression"]["transaction_success"],
            false
        );
        assert_eq!(
            json["delivery_channels"][0],
            serde_json::json!({ "id": "00000000-0000-0000-0000-000000000001", "enabled": true })
        );
        assert!(json.get("alert_type").is_none());
        assert!(json.get("description").is_none());
    }

    #[test]
    fn test_create_alert_request_empty_delivery_channels_serialized() {
        let json =
            serde_json::to_value(CreateAlertRequest::new("x", AlertType::SuccessfulTx)).unwrap();
        assert_eq!(json["delivery_channels"], serde_json::json!([]));
        assert_eq!(json["expressions"], serde_json::json!([]));
    }

    #[test]
    fn test_alert_type_from_str_aliases() {
        assert_eq!(
            "failed_transaction".parse::<AlertType>(),
            Ok(AlertType::FailedTx)
        );
        assert_eq!("event".parse::<AlertType>(), Ok(AlertType::LogEmitted));
        assert_eq!(
            "erc20_transfer".parse::<AlertType>(),
            Ok(AlertType::Erc20TransferLogEmitted)
        );
        assert!("whale_alert".parse::<AlertType>().is_err());
    }

    #[test]
    fn test_create_alert_response_envelope() {
        // POST /alert and PATCH /alert/{id} return {"alert": {...}}
        let json = serde_json::json!({
            "alert": {
                "id": "00000000-0000-0000-0000-000000000010",
                "project_id": "00000000-0000-0000-0000-000000000002",
                "name": "Failed txs",
                "description": "",
                "enabled": true,
                "color": "#ff5252",
                "severity": "danger",
                "created_at": "2024-01-01T00:00:00Z",
                "updated_at": "2024-01-01T00:00:01Z",
                "is_editable": true,
                "delivery_channels": [{
                    "delivery_channel_id": "00000000-0000-0000-0000-000000000001",
                    "delivery_channel": {
                        "id": "00000000-0000-0000-0000-000000000001",
                        "type": "email",
                        "owner_id": "00000000-0000-0000-0000-000000000003",
                        "project_id": null,
                        "label": "Email: alerts@example.com",
                        "reference_id": "alerts@example.com",
                        "enabled": true,
                        "created_at": "2024-01-01T00:00:00Z",
                        "information": { "email": "alerts@example.com" }
                    },
                    "enabled": true,
                    "created_at": "2024-01-01T00:00:00Z"
                }],
                "expressions": [
                    { "type": "contract_address",
                      "expression": { "address": "0x0000000000000000000000000000000000000001",
                                      "transaction_type": null } },
                    { "type": "network", "expression": { "network_id": "1" } },
                    { "type": "tx_status", "expression": { "transaction_success": false } },
                    { "type": "some_future_type", "expression": { "new_field": 1 } }
                ]
            }
        });
        let alert = serde_json::from_value::<AlertOrEnvelope>(json)
            .unwrap()
            .into_alert();
        assert_eq!(alert.severity.as_deref(), Some("danger"));
        assert!(alert.is_editable);
        let channel = &alert.delivery_channels[0];
        assert_eq!(
            channel.delivery_channel_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000001")
        );
        assert_eq!(
            channel.delivery_channel.as_ref().unwrap().label.as_deref(),
            Some("Email: alerts@example.com")
        );
        assert_eq!(
            alert.expressions[0],
            AlertExpression::contract_address("0x0000000000000000000000000000000000000001")
        );
        assert_eq!(alert.expressions[2], AlertExpression::tx_status(false));
        assert_eq!(
            alert.expressions[3].expression_type,
            ExpressionType::Unknown
        );
        assert_eq!(alert.expressions[3].expression.extra["new_field"], 1);
    }

    #[test]
    fn test_list_alerts_response() {
        let json = serde_json::json!({
            "alerts": [{
                "id": "00000000-0000-0000-0000-000000000010",
                "name": "Successful txs",
                "enabled": false,
                "expressions": [{ "type": "tx_status", "expression": { "transaction_success": true } }]
            }]
        });
        let resp: ListAlertsResponse = serde_json::from_value(json).unwrap();
        assert_eq!(resp.alerts.len(), 1);
        assert!(resp.alerts[0].delivery_channels.is_empty());
        assert_eq!(
            resp.alerts[0].expressions[0],
            AlertExpression::tx_status(true)
        );
    }

    #[test]
    fn test_alert_history_response_uses_alert_logs() {
        let json = serde_json::json!({
            "alert_logs": [{
                "alert_id": "00000000-0000-0000-0000-000000000010",
                "tx_hash": "0x0000000000000000000000000000000000000000000000000000000000000001",
                "network_id": "1",
                "delivery_channel_id": "00000000-0000-0000-0000-000000000001",
                "checked_at": "2024-01-01T00:00:00Z",
                "is_test": true
            }]
        });
        let resp: AlertHistoryResponse = serde_json::from_value(json).unwrap();
        let log = &resp.alert_logs[0];
        assert_eq!(log.network_id.as_deref(), Some("1"));
        assert!(log.is_test);
        assert_eq!(log.checked_at.as_deref(), Some("2024-01-01T00:00:00Z"));
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
