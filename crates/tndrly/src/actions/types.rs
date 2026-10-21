//! Types for Web3 Actions API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trigger type for a Web3 Action (`triggerType`)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum ActionTrigger {
    /// Triggered by an alert
    Alert,
    /// Triggered by a webhook call
    Webhook,
    /// Triggered on a schedule
    Periodic,
    /// Triggered by a block being mined
    Block,
    /// Triggered by a transaction
    Transaction,
    /// Triggered by a transaction (simplified filter)
    #[serde(rename = "TRANSACTIONSIMPLE")]
    TransactionSimple,
    /// Unknown trigger type (for forward compatibility)
    #[serde(other)]
    Unknown,
}

impl ActionTrigger {
    /// Get the lowercase string representation (as used in the trigger object's `type`)
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Alert => "alert",
            Self::Webhook => "webhook",
            Self::Periodic => "periodic",
            Self::Block => "block",
            Self::Transaction => "transaction",
            Self::TransactionSimple => "transactionsimple",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for ActionTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ActionTrigger {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace(['-', '_'], "").as_str() {
            "alert" => Ok(Self::Alert),
            "webhook" => Ok(Self::Webhook),
            "periodic" | "cron" | "schedule" => Ok(Self::Periodic),
            "block" => Ok(Self::Block),
            "transaction" | "tx" => Ok(Self::Transaction),
            "transactionsimple" => Ok(Self::TransactionSimple),
            _ => Err(format!("Invalid action trigger: {s}")),
        }
    }
}

/// Invocation type for a Web3 Action (`invocationType`)
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
#[non_exhaustive]
pub enum InvocationType {
    /// Synchronous invocation
    Sync,
    /// Asynchronous invocation
    #[default]
    Async,
    /// Unknown invocation type (for forward compatibility)
    #[serde(other)]
    Unknown,
}

impl InvocationType {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sync => "sync",
            Self::Async => "async",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for InvocationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for InvocationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sync" | "synchronous" => Ok(Self::Sync),
            "async" | "asynchronous" => Ok(Self::Async),
            _ => Err(format!("Invalid invocation type: {s}")),
        }
    }
}

/// Default runtime for new actions
pub const DEFAULT_RUNTIME: &str = "V2";

/// Default exported function name for new actions
pub const DEFAULT_FUNCTION: &str = "actionFn";

/// Request to publish (create) a Web3 Action: `POST /actions/publishFile`
///
/// Serializes as `{"action": {...}, "deploy": bool}`.
#[derive(Debug, Clone, Serialize)]
pub struct CreateActionRequest {
    /// Action definition
    pub action: ActionPayload,

    /// Whether to deploy the action immediately
    pub deploy: bool,
}

/// Action definition sent to `POST /actions/publishFile`
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionPayload {
    /// Action name
    pub name: String,

    /// Action description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source code (JavaScript)
    pub source: String,

    /// Trigger type
    pub trigger_type: ActionTrigger,

    /// Runtime (e.g. "V2")
    pub runtime: String,

    /// Exported function to invoke (e.g. "actionFn")
    pub function: String,

    /// Invocation type
    pub invocation_type: InvocationType,

    /// Trigger configuration object
    pub trigger: TriggerConfig,
}

impl CreateActionRequest {
    /// Create a new action request
    ///
    /// Defaults: runtime `V2`, function `actionFn`, `ASYNC` invocation,
    /// `deploy: true`, and a minimal trigger object `{"type": "<trigger>"}`
    /// (most triggers need a full config; see [`TriggerConfig`]).
    pub fn new(name: impl Into<String>, trigger: ActionTrigger, source: impl Into<String>) -> Self {
        Self {
            action: ActionPayload {
                name: name.into(),
                description: None,
                source: source.into(),
                trigger_type: trigger,
                runtime: DEFAULT_RUNTIME.to_string(),
                function: DEFAULT_FUNCTION.to_string(),
                invocation_type: InvocationType::Async,
                trigger: TriggerConfig::of_type(trigger),
            },
            deploy: true,
        }
    }

    /// Set description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.action.description = Some(desc.into());
        self
    }

    /// Set trigger configuration
    #[must_use]
    pub fn trigger_config(mut self, config: TriggerConfig) -> Self {
        self.action.trigger = config;
        self
    }

    /// Set runtime
    #[must_use]
    pub fn runtime(mut self, runtime: impl Into<String>) -> Self {
        self.action.runtime = runtime.into();
        self
    }

    /// Set the exported function name
    #[must_use]
    pub fn function(mut self, function: impl Into<String>) -> Self {
        self.action.function = function.into();
        self
    }

    /// Set invocation type
    #[must_use]
    pub fn invocation_type(mut self, invocation_type: InvocationType) -> Self {
        self.action.invocation_type = invocation_type;
        self
    }

    /// Set whether to deploy immediately
    #[must_use]
    pub fn deploy(mut self, deploy: bool) -> Self {
        self.deploy = deploy;
        self
    }
}

/// Trigger configuration object (`trigger`)
///
/// The shape depends on the trigger type, e.g.
/// `{"type": "block", "block": {"network": ["1"], "blocks": 10}}`.
/// Use the constructors for documented shapes or [`TriggerConfig::from_value`]
/// for anything else.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TriggerConfig(pub serde_json::Value);

impl TriggerConfig {
    /// Wrap a raw trigger object
    #[must_use]
    pub fn from_value(value: serde_json::Value) -> Self {
        Self(value)
    }

    /// Minimal trigger object `{"type": "<trigger>"}`
    #[must_use]
    pub fn of_type(trigger: ActionTrigger) -> Self {
        Self(serde_json::json!({ "type": trigger.as_str() }))
    }

    /// Block trigger: run every `blocks` blocks on the given networks
    #[must_use]
    pub fn block(networks: Vec<String>, blocks: u64) -> Self {
        Self(serde_json::json!({
            "type": "block",
            "block": { "network": networks, "blocks": blocks }
        }))
    }

    /// Periodic trigger from a cron expression
    pub fn periodic(cron: impl Into<String>) -> Self {
        Self(serde_json::json!({
            "type": "periodic",
            "periodic": { "cron": cron.into() }
        }))
    }

    /// Periodic trigger from an interval (e.g. "1h")
    pub fn interval(interval: impl Into<String>) -> Self {
        Self(serde_json::json!({
            "type": "periodic",
            "periodic": { "interval": interval.into() }
        }))
    }

    /// Webhook trigger
    #[must_use]
    pub fn webhook(authenticated: bool) -> Self {
        Self(serde_json::json!({
            "type": "webhook",
            "webhook": { "authenticated": authenticated }
        }))
    }
}

impl From<serde_json::Value> for TriggerConfig {
    fn from(value: serde_json::Value) -> Self {
        Self(value)
    }
}

/// Web3 Action details (`GET /actions`, `GET /actions/action/{id}`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    /// Action ID
    pub id: String,

    /// Project ID
    #[serde(default)]
    pub project_id: Option<String>,

    /// Action name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Status (e.g. "DEPLOYED", "PUBLISHED")
    #[serde(default)]
    pub status: Option<String>,

    /// Whether the action is stopped
    #[serde(default)]
    pub stopped: bool,

    /// Whether the action is editable
    #[serde(default)]
    pub editable: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Current version (code, runtime and trigger)
    #[serde(default)]
    pub version: Option<ActionVersion>,

    /// Delivery channels
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub delivery_channels: Vec<serde_json::Value>,
}

impl Action {
    /// Trigger type of the current version
    #[must_use]
    pub fn trigger_type(&self) -> Option<ActionTrigger> {
        self.version.as_ref().and_then(|v| v.trigger_type)
    }

    /// Runtime of the current version
    #[must_use]
    pub fn runtime(&self) -> Option<&str> {
        self.version.as_ref().and_then(|v| v.runtime.as_deref())
    }
}

/// A published version of a Web3 Action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionVersion {
    /// Version ID
    pub id: String,

    /// Version index
    #[serde(default)]
    pub index: Option<u64>,

    /// Action ID
    #[serde(default)]
    pub action_id: Option<String>,

    /// Parent action ID
    #[serde(default)]
    pub parent_action_id: Option<String>,

    /// Runtime (e.g. "V2")
    #[serde(default)]
    pub runtime: Option<String>,

    /// Exported function (e.g. "actionFn")
    #[serde(default)]
    pub function: Option<String>,

    /// Trigger type
    #[serde(default)]
    pub trigger_type: Option<ActionTrigger>,

    /// Trigger configuration object
    #[serde(default)]
    pub trigger: serde_json::Value,

    /// Commitish
    #[serde(default)]
    pub commitish: Option<String>,

    /// Source code
    #[serde(default)]
    pub source: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Whether deployment was requested
    #[serde(default)]
    pub deploy_requested: bool,

    /// Deployment error, if any
    #[serde(default)]
    pub deploy_error: Option<String>,

    /// Invocation type
    #[serde(default)]
    pub invocation_type: Option<InvocationType>,
}

/// Response from `POST /actions/publishFile`: published versions keyed by action name
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishActionResponse {
    /// Published action versions, keyed by action name
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub actions: HashMap<String, ActionVersion>,
}

/// Deserialize null as default value
fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Action execution (call) status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[non_exhaustive]
pub enum ActionLogStatus {
    /// Enabled
    Enabled,
    /// Disabled
    Disabled,
    /// Execution succeeded
    Succeeded,
    /// Execution submitted
    Submitted,
    /// Execution skipped
    Skipped,
    /// Execution failed
    Failed,
    /// Execution pending
    Pending,
    /// Execution will be retried
    Retry,
    /// Execution timed out
    TimedOut,
    /// Execution was rate limited
    RateLimited,
    /// Unknown status (for forward compatibility)
    #[serde(other)]
    Unknown,
}

impl ActionLogStatus {
    /// Get the API string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Enabled => "ENABLED",
            Self::Disabled => "DISABLED",
            Self::Succeeded => "SUCCEEDED",
            Self::Submitted => "SUBMITTED",
            Self::Skipped => "SKIPPED",
            Self::Failed => "FAILED",
            Self::Pending => "PENDING",
            Self::Retry => "RETRY",
            Self::TimedOut => "TIMED_OUT",
            Self::RateLimited => "RATE_LIMITED",
            Self::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for ActionLogStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ActionLogStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "enabled" => Ok(Self::Enabled),
            "disabled" => Ok(Self::Disabled),
            "succeeded" | "success" | "ok" => Ok(Self::Succeeded),
            "submitted" => Ok(Self::Submitted),
            "skipped" => Ok(Self::Skipped),
            "failed" | "failure" | "error" => Ok(Self::Failed),
            "pending" | "queued" => Ok(Self::Pending),
            "retry" => Ok(Self::Retry),
            "timed_out" | "timeout" => Ok(Self::TimedOut),
            "rate_limited" => Ok(Self::RateLimited),
            _ => Err(format!("Invalid action log status: {s}")),
        }
    }
}

/// Request to stop or resume multiple actions
#[derive(Debug, Clone, Serialize)]
pub struct StopResumeActionsRequest {
    /// Action IDs to stop/resume. Empty = all actions.
    pub actions: Vec<String>,
}

/// Query parameters for action calls/executions
#[derive(Debug, Clone, Default)]
pub struct ActionCallsQuery {
    /// Page number (1-indexed)
    pub page: Option<u32>,

    /// Items per page (`pageSize`, max 100)
    pub page_size: Option<u32>,

    /// Only return calls with these statuses (repeated `status` param)
    pub status: Vec<ActionLogStatus>,

    /// Only return calls after this timestamp
    pub after: Option<String>,

    /// Only return calls before this timestamp
    pub before: Option<String>,
}

impl ActionCallsQuery {
    /// Create a new query
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set items per page
    #[must_use]
    pub fn page_size(mut self, page_size: u32) -> Self {
        self.page_size = Some(page_size);
        self
    }

    /// Filter by a status (can be called multiple times)
    #[must_use]
    pub fn status(mut self, status: ActionLogStatus) -> Self {
        self.status.push(status);
        self
    }

    /// Only return calls after this timestamp
    #[must_use]
    pub fn after(mut self, after: impl Into<String>) -> Self {
        self.after = Some(after.into());
        self
    }

    /// Only return calls before this timestamp
    #[must_use]
    pub fn before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }

    /// Build the query string pairs (`status` is repeated once per value)
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut pairs = Vec::new();
        if let Some(page) = self.page {
            pairs.push(("page", page.to_string()));
        }
        if let Some(page_size) = self.page_size {
            pairs.push(("pageSize", page_size.to_string()));
        }
        for status in &self.status {
            pairs.push(("status", status.as_str().to_string()));
        }
        if let Some(after) = &self.after {
            pairs.push(("after", after.clone()));
        }
        if let Some(before) = &self.before {
            pairs.push(("before", before.clone()));
        }
        pairs
    }
}

/// Action call/execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionCall {
    /// Execution ID
    pub id: String,

    /// Action ID
    #[serde(default)]
    pub action_id: Option<String>,

    /// Action version ID
    #[serde(default)]
    pub version_id: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Call type (e.g. "AUTOMATIC", "MANUAL", "RERUN")
    #[serde(default, rename = "type")]
    pub call_type: Option<String>,

    /// Status
    #[serde(default)]
    pub status: Option<ActionLogStatus>,

    /// When the status last changed
    #[serde(default)]
    pub status_updated_at: Option<String>,

    /// Trigger payload
    #[serde(default)]
    pub payload: serde_json::Value,

    /// Storage type (e.g. "WRITE", "COPY_ON_WRITE", "EMPTY")
    #[serde(default)]
    pub storage_type: Option<String>,

    /// Storage ID
    #[serde(default)]
    pub storage_id: Option<String>,

    /// Memory used
    #[serde(default)]
    pub memory: Option<f64>,

    /// Execution time
    #[serde(default)]
    pub time: Option<f64>,

    /// Parsed logs
    #[serde(default)]
    pub parsed_logs: Option<serde_json::Value>,

    /// Parsed error
    #[serde(default)]
    pub parsed_error: Option<serde_json::Value>,
}
