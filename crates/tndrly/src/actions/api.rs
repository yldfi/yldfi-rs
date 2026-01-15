//! Web3 Actions API operations

use super::types::*;
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Web3 Actions API client
pub struct ActionsApi<'a> {
    client: &'a Client,
}

impl<'a> ActionsApi<'a> {
    /// Create a new Actions API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Create a new Web3 Action
    ///
    /// # Example
    ///
    /// ```ignore
    /// use tndrly::actions::{CreateActionRequest, ActionTrigger, TriggerConfig};
    ///
    /// let source = r#"
    /// const { Tenderly } = require('@tenderly/actions');
    ///
    /// module.exports = async (context, event) => {
    ///     console.log('Event received:', event);
    ///     return { status: 'ok' };
    /// };
    /// "#;
    ///
    /// let request = CreateActionRequest::new("My Action", ActionTrigger::Alert, source)
    ///     .trigger_config(TriggerConfig::alert("alert-123"))
    ///     .description("Handle alert events");
    ///
    /// let action = client.actions().create(&request).await?;
    /// ```
    pub async fn create(&self, request: &CreateActionRequest) -> Result<Action> {
        self.client.post("/actions/publishFile", request).await
    }

    /// List all Web3 Actions
    ///
    /// Returns an empty list if no actions exist.
    pub async fn list(&self) -> Result<ListActionsResponse> {
        // The API returns null when there are no actions, so we need to handle that
        let response: Option<ListActionsResponse> = self.client.get("/actions").await?;
        Ok(response.unwrap_or(ListActionsResponse {
            actions: Vec::new(),
        }))
    }

    /// Get an action by ID
    pub async fn get(&self, id: &str) -> Result<Action> {
        self.client
            .get(&format!("/actions/action/{}", encode_path_segment(id)))
            .await
    }

    /// Update an action
    pub async fn update(&self, id: &str, request: &CreateActionRequest) -> Result<Action> {
        self.client
            .patch(
                &format!("/actions/action/{}", encode_path_segment(id)),
                request,
            )
            .await
    }

    /// Delete an action
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/actions/action/{}", encode_path_segment(id)))
            .await
    }

    /// Enable an action
    pub async fn enable(&self, id: &str) -> Result<Action> {
        let request = serde_json::json!({ "enabled": true });
        self.client
            .patch(
                &format!("/actions/action/{}", encode_path_segment(id)),
                &request,
            )
            .await
    }

    /// Disable an action
    pub async fn disable(&self, id: &str) -> Result<Action> {
        let request = serde_json::json!({ "enabled": false });
        self.client
            .patch(
                &format!("/actions/action/{}", encode_path_segment(id)),
                &request,
            )
            .await
    }

    /// Invoke an action manually
    ///
    /// Useful for testing or manual triggering.
    pub async fn invoke(
        &self,
        id: &str,
        request: &InvokeActionRequest,
    ) -> Result<InvokeActionResponse> {
        self.client
            .post(
                &format!("/actions/action/{}/invoke", encode_path_segment(id)),
                request,
            )
            .await
    }

    /// Get execution logs for an action
    pub async fn logs(&self, id: &str) -> Result<ListActionLogsResponse> {
        self.client
            .get(&format!("/actions/action/{}/logs", encode_path_segment(id)))
            .await
    }

    /// Get a specific execution log
    pub async fn get_log(&self, action_id: &str, log_id: &str) -> Result<ActionLog> {
        self.client
            .get(&format!(
                "/actions/action/{}/logs/{}",
                encode_path_segment(action_id),
                encode_path_segment(log_id)
            ))
            .await
    }

    /// Get the source code of an action
    pub async fn source(&self, id: &str) -> Result<String> {
        #[derive(serde::Deserialize)]
        struct SourceResponse {
            source_code: String,
        }
        let response: SourceResponse = self
            .client
            .get(&format!(
                "/actions/action/{}/source",
                encode_path_segment(id)
            ))
            .await?;
        Ok(response.source_code)
    }

    /// Update the source code of an action
    pub async fn update_source(&self, id: &str, source_code: &str) -> Result<Action> {
        let request = serde_json::json!({ "source_code": source_code });
        self.client
            .patch(
                &format!("/actions/action/{}/source", encode_path_segment(id)),
                &request,
            )
            .await
    }

    /// Stop a specific Web3 Action
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.actions().stop("action-123").await?;
    /// ```
    pub async fn stop(&self, id: &str) -> Result<()> {
        self.client
            .post_no_response(
                &format!("/actions/action/{}/stop", encode_path_segment(id)),
                &serde_json::json!({}),
            )
            .await
    }

    /// Resume a stopped Web3 Action
    ///
    /// # Example
    ///
    /// ```ignore
    /// client.actions().resume("action-123").await?;
    /// ```
    pub async fn resume(&self, id: &str) -> Result<()> {
        self.client
            .post_no_response(
                &format!("/actions/action/{}/resume", encode_path_segment(id)),
                &serde_json::json!({}),
            )
            .await
    }

    /// Stop multiple Web3 Actions
    ///
    /// If `action_ids` is empty, stops all actions in the project.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Stop specific actions
    /// client.actions().stop_many(vec!["action-1".to_string(), "action-2".to_string()]).await?;
    ///
    /// // Stop all actions in project
    /// client.actions().stop_many(vec![]).await?;
    /// ```
    pub async fn stop_many(&self, action_ids: Vec<String>) -> Result<()> {
        let request = StopResumeActionsRequest {
            actions: action_ids,
        };
        self.client
            .post_no_response("/actions/stop", &request)
            .await
    }

    /// Resume multiple stopped Web3 Actions
    ///
    /// If `action_ids` is empty, resumes all actions in the project.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Resume specific actions
    /// client.actions().resume_many(vec!["action-1".to_string(), "action-2".to_string()]).await?;
    ///
    /// // Resume all actions in project
    /// client.actions().resume_many(vec![]).await?;
    /// ```
    pub async fn resume_many(&self, action_ids: Vec<String>) -> Result<()> {
        let request = StopResumeActionsRequest {
            actions: action_ids,
        };
        self.client
            .post_no_response("/actions/resume", &request)
            .await
    }

    /// Get execution history (calls) for an action
    ///
    /// # Example
    ///
    /// ```ignore
    /// let calls = client.actions().calls("action-123", None).await?;
    /// for call in calls.executions {
    ///     println!("Execution: {} - {:?}", call.id, call.status);
    /// }
    /// ```
    pub async fn calls(
        &self,
        id: &str,
        query: Option<ActionCallsQuery>,
    ) -> Result<ActionCallsResponse> {
        let path = format!("/actions/action/{}/calls", encode_path_segment(id));
        match query {
            Some(q) => self.client.get_with_query(&path, &q).await,
            None => self.client.get(&path).await,
        }
    }

    /// Get a specific execution call details
    ///
    /// # Example
    ///
    /// ```ignore
    /// let call = client.actions().get_call("action-123", "exec-456").await?;
    /// println!("Status: {:?}", call.status);
    /// ```
    pub async fn get_call(&self, action_id: &str, execution_id: &str) -> Result<ActionCall> {
        self.client
            .get(&format!(
                "/actions/action/{}/calls/call/{}",
                encode_path_segment(action_id),
                encode_path_segment(execution_id)
            ))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_action_request() {
        let request = CreateActionRequest::new(
            "Test Action",
            ActionTrigger::Alert,
            "module.exports = () => {}",
        )
        .description("A test action")
        .trigger_config(TriggerConfig::alert("alert-123"))
        .secret("API_KEY", "secret123")
        .enabled(true);

        assert_eq!(request.name, "Test Action");
        assert_eq!(request.trigger, ActionTrigger::Alert);
        assert!(request.description.is_some());
        assert!(request.trigger_config.is_some());
        assert!(request.secrets.is_some());
    }

    #[test]
    fn test_trigger_config() {
        let alert_config = TriggerConfig::alert("alert-123");
        assert_eq!(alert_config.alert_id, Some("alert-123".to_string()));

        let periodic_config = TriggerConfig::periodic("0 * * * *");
        assert_eq!(periodic_config.cron, Some("0 * * * *".to_string()));

        let tx_config = TriggerConfig::transaction("1", "0x1234");
        assert_eq!(tx_config.network_id, Some("1".to_string()));
        assert_eq!(tx_config.address, Some("0x1234".to_string()));
    }

    #[test]
    fn test_invoke_request() {
        let request = InvokeActionRequest::with_payload(serde_json::json!({
            "test": true
        }));
        assert!(request.payload.is_some());
    }

    #[test]
    fn test_create_action_request_serialization() {
        // Verify JSON structure matches Tenderly API expectations
        let request = CreateActionRequest::new(
            "Test Action",
            ActionTrigger::Alert,
            "module.exports = () => {}",
        )
        .description("A test action")
        .trigger_config(TriggerConfig::alert("alert-123"))
        .secret("API_KEY", "secret123");

        let json = serde_json::to_value(&request).unwrap();

        // Verify field names and enums serialize correctly
        assert_eq!(json["name"], "Test Action");
        assert_eq!(json["trigger"], "alert"); // snake_case enum
        assert_eq!(json["execution"], "sequential"); // snake_case enum
        assert_eq!(json["source_code"], "module.exports = () => {}");
        assert_eq!(json["runtime"], "nodejs18");
        assert_eq!(json["enabled"], true);

        // Verify trigger_config is nested correctly
        assert!(json["trigger_config"].is_object());
        assert_eq!(json["trigger_config"]["alert_id"], "alert-123");

        // Verify secrets are serialized correctly
        assert!(json["secrets"].is_array());
        assert_eq!(json["secrets"][0]["name"], "API_KEY");
        assert_eq!(json["secrets"][0]["value"], "secret123"); // Value is exposed in serialization
    }

    #[test]
    fn test_action_calls_query_serialization() {
        // Verify perPage uses correct casing (camelCase for this endpoint)
        let query = ActionCallsQuery::new().page(1).per_page(25);

        let json = serde_json::to_value(&query).unwrap();

        assert_eq!(json["page"], 1);
        assert_eq!(json["perPage"], 25); // Must be camelCase
        assert!(
            json.get("per_page").is_none(),
            "per_page should be renamed to perPage"
        );
    }
}
