//! Types for Web3 Actions API

use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};

/// Trigger type for a Web3 Action
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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
}

impl ActionTrigger {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Alert => "alert",
            Self::Webhook => "webhook",
            Self::Periodic => "periodic",
            Self::Block => "block",
            Self::Transaction => "transaction",
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
        match s.to_lowercase().as_str() {
            "alert" => Ok(Self::Alert),
            "webhook" => Ok(Self::Webhook),
            "periodic" | "cron" | "schedule" => Ok(Self::Periodic),
            "block" => Ok(Self::Block),
            "transaction" | "tx" => Ok(Self::Transaction),
            _ => Err(format!("Invalid action trigger: {}", s)),
        }
    }
}

/// Execution type for a Web3 Action
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ActionExecution {
    /// Sequential execution
    Sequential,
    /// Parallel execution
    Parallel,
}

impl ActionExecution {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::Parallel => "parallel",
        }
    }
}

impl std::fmt::Display for ActionExecution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for ActionExecution {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sequential" | "seq" => Ok(Self::Sequential),
            "parallel" | "par" => Ok(Self::Parallel),
            _ => Err(format!("Invalid action execution: {}", s)),
        }
    }
}

/// Request to create a Web3 Action
#[derive(Debug, Clone, Serialize)]
pub struct CreateActionRequest {
    /// Action name
    pub name: String,

    /// Action description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Trigger type
    pub trigger: ActionTrigger,

    /// Trigger configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_config: Option<TriggerConfig>,

    /// Execution type
    #[serde(default = "default_execution")]
    pub execution: ActionExecution,

    /// Source code (JavaScript or TypeScript)
    pub source_code: String,

    /// Runtime (e.g., "nodejs18")
    #[serde(default = "default_runtime")]
    pub runtime: String,

    /// Environment secrets
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secrets: Option<Vec<ActionSecret>>,

    /// Whether the action is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

// Used by serde(default = "...") attribute; rustc doesn't recognize serde's usage
#[allow(dead_code)]
fn default_execution() -> ActionExecution {
    ActionExecution::Sequential
}

// Used by serde(default = "...") attribute; rustc doesn't recognize serde's usage
#[allow(dead_code)]
fn default_runtime() -> String {
    "nodejs18".to_string()
}

// Used by serde(default = "...") attribute; rustc doesn't recognize serde's usage
#[allow(dead_code)]
fn default_true() -> bool {
    true
}

impl CreateActionRequest {
    /// Create a new action request
    pub fn new(
        name: impl Into<String>,
        trigger: ActionTrigger,
        source_code: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: None,
            trigger,
            trigger_config: None,
            execution: ActionExecution::Sequential,
            source_code: source_code.into(),
            runtime: "nodejs18".to_string(),
            secrets: None,
            enabled: true,
        }
    }

    /// Set description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set trigger configuration
    #[must_use]
    pub fn trigger_config(mut self, config: TriggerConfig) -> Self {
        self.trigger_config = Some(config);
        self
    }

    /// Set execution type
    #[must_use]
    pub fn execution(mut self, execution: ActionExecution) -> Self {
        self.execution = execution;
        self
    }

    /// Set runtime
    #[must_use]
    pub fn runtime(mut self, runtime: impl Into<String>) -> Self {
        self.runtime = runtime.into();
        self
    }

    /// Add a secret
    #[must_use]
    pub fn secret(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.secrets
            .get_or_insert_with(Vec::new)
            .push(ActionSecret::new(name, value));
        self
    }

    /// Set enabled state
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Trigger configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TriggerConfig {
    /// Alert ID (for Alert trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alert_id: Option<String>,

    /// Network ID (for Block/Transaction triggers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,

    /// Cron expression (for Periodic trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron: Option<String>,

    /// Address filter (for Transaction trigger)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
}

impl TriggerConfig {
    /// Create config for alert trigger
    pub fn alert(alert_id: impl Into<String>) -> Self {
        Self {
            alert_id: Some(alert_id.into()),
            ..Default::default()
        }
    }

    /// Create config for periodic trigger
    pub fn periodic(cron: impl Into<String>) -> Self {
        Self {
            cron: Some(cron.into()),
            ..Default::default()
        }
    }

    /// Create config for block trigger
    pub fn block(network_id: impl Into<String>) -> Self {
        Self {
            network_id: Some(network_id.into()),
            ..Default::default()
        }
    }

    /// Create config for transaction trigger
    pub fn transaction(network_id: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            network_id: Some(network_id.into()),
            address: Some(address.into()),
            ..Default::default()
        }
    }
}

/// Action secret
///
/// The value is protected with `SecretString` to prevent accidental leakage in logs.
/// The `Debug` implementation redacts the value.
///
/// # Warning
///
/// The `Serialize` implementation **does expose the secret value** because it must
/// be sent to the Tenderly API. Be careful not to serialize `ActionSecret` or any
/// containing struct for logging purposes, as this will expose the secret.
#[derive(Clone)]
pub struct ActionSecret {
    /// Secret name
    pub name: String,
    /// Secret value (protected)
    value: SecretString,
}

impl ActionSecret {
    /// Create a new action secret
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: SecretString::from(value.into()),
        }
    }

    /// Expose the secret value (use with care)
    pub fn expose_value(&self) -> &str {
        self.value.expose_secret()
    }
}

impl std::fmt::Debug for ActionSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActionSecret")
            .field("name", &self.name)
            .field("value", &"[REDACTED]")
            .finish()
    }
}

impl Serialize for ActionSecret {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("ActionSecret", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("value", self.value.expose_secret())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for ActionSecret {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            name: String,
            value: String,
        }
        let helper = Helper::deserialize(deserializer)?;
        Ok(Self::new(helper.name, helper.value))
    }
}

/// Web3 Action details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// Action ID
    pub id: String,

    /// Action name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Trigger type
    pub trigger: ActionTrigger,

    /// Execution type
    pub execution: ActionExecution,

    /// Runtime
    pub runtime: String,

    /// Whether enabled
    #[serde(default)]
    pub enabled: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Last updated timestamp
    #[serde(default)]
    pub updated_at: Option<String>,

    /// Last execution timestamp
    #[serde(default)]
    pub last_executed_at: Option<String>,
}

/// Response when listing actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActionsResponse {
    /// List of actions (handles null as empty)
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub actions: Vec<Action>,
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

/// Action execution log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionLog {
    /// Log ID
    pub id: String,

    /// Action ID
    pub action_id: String,

    /// Execution status
    pub status: ActionLogStatus,

    /// Execution timestamp
    #[serde(default)]
    pub executed_at: Option<String>,

    /// Duration in milliseconds
    #[serde(default)]
    pub duration_ms: Option<u64>,

    /// Error message (if failed)
    #[serde(default)]
    pub error: Option<String>,

    /// Console output
    #[serde(default)]
    pub output: Option<String>,
}

/// Action execution status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ActionLogStatus {
    /// Execution succeeded
    Success,
    /// Execution failed
    Failed,
    /// Execution timed out
    Timeout,
    /// Execution pending
    Pending,
    /// Execution in progress
    Running,
}

impl ActionLogStatus {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Success => "success",
            Self::Failed => "failed",
            Self::Timeout => "timeout",
            Self::Pending => "pending",
            Self::Running => "running",
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
        match s.to_lowercase().as_str() {
            "success" | "succeeded" | "ok" => Ok(Self::Success),
            "failed" | "failure" | "error" => Ok(Self::Failed),
            "timeout" | "timed_out" => Ok(Self::Timeout),
            "pending" | "queued" => Ok(Self::Pending),
            "running" | "in_progress" => Ok(Self::Running),
            _ => Err(format!("Invalid action log status: {}", s)),
        }
    }
}

/// Response when listing action logs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListActionLogsResponse {
    /// List of execution logs
    #[serde(default)]
    pub logs: Vec<ActionLog>,
}

/// Request to invoke an action manually
#[derive(Debug, Clone, Default, Serialize)]
pub struct InvokeActionRequest {
    /// Custom payload to pass to the action
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

impl InvokeActionRequest {
    /// Create an empty invoke request
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with a payload
    pub fn with_payload(payload: serde_json::Value) -> Self {
        Self {
            payload: Some(payload),
        }
    }
}

/// Response from invoking an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvokeActionResponse {
    /// Execution ID
    pub execution_id: String,

    /// Status
    pub status: ActionLogStatus,

    /// Result (if synchronous)
    #[serde(default)]
    pub result: Option<serde_json::Value>,
}

/// Request to stop or resume multiple actions
#[derive(Debug, Clone, Serialize)]
pub struct StopResumeActionsRequest {
    /// Action IDs to stop/resume. Empty = all actions.
    pub actions: Vec<String>,
}

/// Query parameters for action calls/executions
#[derive(Debug, Clone, Default, Serialize)]
pub struct ActionCallsQuery {
    /// Page number (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Items per page
    #[serde(rename = "perPage", skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

impl ActionCallsQuery {
    /// Create a new query
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
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

/// Action call/execution details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCall {
    /// Execution ID
    pub id: String,

    /// Action ID
    #[serde(default)]
    pub action_id: Option<String>,

    /// Status
    #[serde(default)]
    pub status: Option<ActionLogStatus>,

    /// Start time
    #[serde(default)]
    pub started_at: Option<String>,

    /// End time
    #[serde(default)]
    pub finished_at: Option<String>,

    /// Duration in milliseconds
    #[serde(default)]
    pub duration_ms: Option<u64>,

    /// Trigger information
    #[serde(default)]
    pub trigger: Option<serde_json::Value>,

    /// Output/result
    #[serde(default)]
    pub output: Option<serde_json::Value>,

    /// Error message (if failed)
    #[serde(default)]
    pub error: Option<String>,

    /// Logs from execution
    #[serde(default)]
    pub logs: Option<Vec<String>>,
}

/// Response for action calls list
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionCallsResponse {
    /// List of executions
    #[serde(default)]
    pub executions: Vec<ActionCall>,
}
