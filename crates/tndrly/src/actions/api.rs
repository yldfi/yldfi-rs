//! Web3 Actions API operations

use super::types::{
    Action, ActionCall, ActionCallsQuery, CreateActionRequest, PublishActionResponse,
    StopResumeActionsRequest,
};
use crate::client::{encode_path_segment, Client};
use crate::error::Result;

/// Web3 Actions API client
pub struct ActionsApi<'a> {
    client: &'a Client,
}

impl<'a> ActionsApi<'a> {
    /// Create a new Actions API client
    #[must_use]
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
    /// let request = CreateActionRequest::new("my-action", ActionTrigger::Periodic, source)
    ///     .trigger_config(TriggerConfig::periodic("0 * * * *"))
    ///     .description("Hourly job");
    ///
    /// let published = client.actions().create(&request).await?;
    /// ```
    pub async fn create(&self, request: &CreateActionRequest) -> Result<PublishActionResponse> {
        self.client.post("/actions/publishFile", request).await
    }

    /// List all Web3 Actions
    ///
    /// The API returns a bare array (or `null` when there are no actions,
    /// which is returned as an empty list).
    pub async fn list(&self) -> Result<Vec<Action>> {
        let response: Option<Vec<Action>> = self.client.get("/actions").await?;
        Ok(response.unwrap_or_default())
    }

    /// Get an action by ID
    pub async fn get(&self, id: &str) -> Result<Action> {
        self.client
            .get(&format!("/actions/action/{}", encode_path_segment(id)))
            .await
    }

    /// Delete an action
    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/actions/action/{}", encode_path_segment(id)))
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
    /// for call in calls {
    ///     println!("Execution: {} - {:?}", call.id, call.status);
    /// }
    /// ```
    pub async fn calls(
        &self,
        id: &str,
        query: Option<ActionCallsQuery>,
    ) -> Result<Vec<ActionCall>> {
        let path = format!("/actions/action/{}/calls", encode_path_segment(id));
        let response: Option<Vec<ActionCall>> = match query {
            Some(q) => {
                self.client
                    .get_with_query(&path, &q.to_query_pairs())
                    .await?
            }
            None => self.client.get(&path).await?,
        };
        Ok(response.unwrap_or_default())
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
    use crate::actions::{
        ActionLogStatus, ActionTrigger, ActionVersion, InvocationType, TriggerConfig,
    };

    #[test]
    fn test_trigger_config() {
        let block = TriggerConfig::block(vec!["1".to_string(), "137".to_string()], 10);
        assert_eq!(block.0["type"], "block");
        assert_eq!(block.0["block"]["network"][1], "137");
        assert_eq!(block.0["block"]["blocks"], 10);

        let periodic = TriggerConfig::periodic("0 * * * *");
        assert_eq!(periodic.0["periodic"]["cron"], "0 * * * *");

        let webhook = TriggerConfig::webhook(false);
        assert_eq!(webhook.0["webhook"]["authenticated"], false);
    }

    #[test]
    fn test_create_action_request_serialization() {
        // POST /actions/publishFile takes {"action": {...camelCase...}, "deploy": bool}
        let request = CreateActionRequest::new(
            "block-action",
            ActionTrigger::Block,
            "module.exports = { actionFn }",
        )
        .description("A test action")
        .trigger_config(TriggerConfig::block(vec!["1".to_string()], 10))
        .invocation_type(InvocationType::Sync);

        let json = serde_json::to_value(&request).unwrap();

        assert_eq!(json["deploy"], true);
        let action = &json["action"];
        assert_eq!(action["name"], "block-action");
        assert_eq!(action["description"], "A test action");
        assert_eq!(action["source"], "module.exports = { actionFn }");
        assert_eq!(action["triggerType"], "BLOCK");
        assert_eq!(action["runtime"], "V2");
        assert_eq!(action["function"], "actionFn");
        assert_eq!(action["invocationType"], "SYNC");
        assert_eq!(action["trigger"]["type"], "block");
        assert_eq!(action["trigger"]["block"]["blocks"], 10);
        assert!(json.get("name").is_none());
        assert!(action.get("source_code").is_none());
    }

    #[test]
    fn test_create_action_request_default_trigger() {
        let request = CreateActionRequest::new("x", ActionTrigger::TransactionSimple, "src");
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["action"]["triggerType"], "TRANSACTIONSIMPLE");
        assert_eq!(json["action"]["trigger"]["type"], "transactionsimple");
        assert_eq!(json["action"]["invocationType"], "ASYNC");
    }

    #[test]
    fn test_list_actions_response_is_bare_array() {
        let json = serde_json::json!([{
            "id": "00000000-0000-0000-0000-000000000001",
            "projectId": "00000000-0000-0000-0000-000000000002",
            "name": "periodic-action",
            "description": "",
            "status": "DEPLOYED",
            "stopped": false,
            "editable": true,
            "createdAt": "2024-01-01T00:00:00Z",
            "version": {
                "id": "00000000-0000-0000-0000-000000000003",
                "index": 1,
                "actionId": "00000000-0000-0000-0000-000000000001",
                "parentActionId": "",
                "runtime": "V2",
                "function": "actionFn",
                "triggerType": "PERIODIC",
                "trigger": { "type": "periodic", "periodic": { "cron": "0 * * * *" } },
                "commitish": null,
                "source": "module.exports = { actionFn }",
                "createdAt": "2024-01-01T00:00:00Z",
                "deployRequested": true,
                "deployError": null,
                "invocationType": "ASYNC"
            },
            "deliveryChannels": null
        }]);
        let actions: Vec<Action> = serde_json::from_value(json).unwrap();
        let action = &actions[0];
        assert_eq!(
            action.project_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000002")
        );
        assert_eq!(action.status.as_deref(), Some("DEPLOYED"));
        assert!(action.editable);
        assert_eq!(action.trigger_type(), Some(ActionTrigger::Periodic));
        assert_eq!(action.runtime(), Some("V2"));
        let version = action.version.as_ref().unwrap();
        assert_eq!(version.invocation_type, Some(InvocationType::Async));
        assert_eq!(version.trigger["periodic"]["cron"], "0 * * * *");
        assert!(version.deploy_requested);
        assert!(action.delivery_channels.is_empty());
    }

    #[test]
    fn test_publish_action_response() {
        let json = serde_json::json!({
            "actions": {
                "block-action": {
                    "id": "00000000-0000-0000-0000-000000000003",
                    "index": 1,
                    "actionId": "00000000-0000-0000-0000-000000000001",
                    "runtime": "V2",
                    "function": "actionFn",
                    "triggerType": "BLOCK",
                    "trigger": { "type": "block", "block": { "network": ["1"], "blocks": 10 } },
                    "source": "src",
                    "createdAt": "2024-01-01T00:00:00Z",
                    "deployRequested": true,
                    "invocationType": "ASYNC"
                }
            }
        });
        let resp: PublishActionResponse = serde_json::from_value(json).unwrap();
        let version: &ActionVersion = &resp.actions["block-action"];
        assert_eq!(version.trigger_type, Some(ActionTrigger::Block));
        assert_eq!(
            version.action_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000001")
        );
    }

    #[test]
    fn test_action_calls_query_pairs() {
        // Listing calls pages with pageSize and repeats status
        let query = ActionCallsQuery::new()
            .page(2)
            .page_size(25)
            .status(ActionLogStatus::Failed)
            .status(ActionLogStatus::TimedOut);
        let req = reqwest::Client::new()
            .get("https://example.com/calls")
            .query(&query.to_query_pairs())
            .build()
            .unwrap();
        assert_eq!(
            req.url().query(),
            Some("page=2&pageSize=25&status=FAILED&status=TIMED_OUT")
        );
    }

    #[test]
    fn test_action_call_deserializes_camel_case() {
        let json = serde_json::json!([{
            "id": "00000000-0000-0000-0000-000000000004",
            "actionId": "00000000-0000-0000-0000-000000000001",
            "versionId": "00000000-0000-0000-0000-000000000003",
            "createdAt": "2024-01-01T00:00:00Z",
            "type": "AUTOMATIC",
            "status": "SUCCEEDED",
            "statusUpdatedAt": "2024-01-01T00:00:01Z",
            "payload": { "type": "block", "block": { "network": "1", "blockNumber": 100 } },
            "storageType": "WRITE",
            "storageId": "00000000-0000-0000-0000-000000000005",
            "memory": null,
            "time": 16
        }]);
        let calls: Vec<ActionCall> = serde_json::from_value(json).unwrap();
        let call = &calls[0];
        assert_eq!(call.status, Some(ActionLogStatus::Succeeded));
        assert_eq!(call.call_type.as_deref(), Some("AUTOMATIC"));
        assert_eq!(
            call.version_id.as_deref(),
            Some("00000000-0000-0000-0000-000000000003")
        );
        assert_eq!(call.storage_type.as_deref(), Some("WRITE"));
        assert_eq!(call.memory, None);
        assert_eq!(call.time, Some(16.0));
        assert_eq!(call.payload["block"]["blockNumber"], 100);
    }
}
