//! Types for Alerts API
//!
//! An alert is a `simple_type` plus a list of `expressions` (conditions that
//! must all match) and the `delivery_channels` to notify:
//!
//! ```json
//! {
//!   "name": "Failed txs",
//!   "simple_type": "failed_tx",
//!   "expressions": [
//!     { "type": "contract_address", "expression": { "address": "0x..." } },
//!     { "type": "network", "expression": { "network_id": "1" } },
//!     { "type": "tx_status", "expression": { "transaction_success": false } }
//!   ],
//!   "delivery_channels": [{ "id": "<channel id>", "enabled": true }]
//! }
//! ```
//!
//! The webhooks endpoints (`/webhooks`) are not part of Tenderly's published
//! OpenAPI spec.
//!
//! See: <https://docs.tenderly.co/alerts/api>

use crate::delivery_channels::DeliveryChannel;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Alert kind (`simple_type`)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AlertType {
    /// Successful transaction
    SuccessfulTx,
    /// Failed transaction
    FailedTx,
    /// Call from an allowlisted caller
    AllowlistedCallers,
    /// Call from a blocklisted caller
    BlocklistedCallers,
    /// Event (log) emitted
    LogEmitted,
    /// ERC20 `Transfer` log emitted
    Erc20TransferLogEmitted,
    /// Emitted log parameter matches a condition
    EmittedLogParameter,
    /// Function called
    FunctionCalled,
    /// Called function parameter matches a condition
    CalledFunctionParameter,
    /// Contract state changed
    StateChange,
    /// ETH balance threshold
    EthBalance,
    /// Transaction value threshold
    TxValue,
    /// View function result matches a condition
    ViewFunction,
    /// Unknown type (for forward compatibility)
    #[serde(other)]
    Unknown,
}

impl AlertType {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SuccessfulTx => "successful_tx",
            Self::FailedTx => "failed_tx",
            Self::AllowlistedCallers => "allowlisted_callers",
            Self::BlocklistedCallers => "blocklisted_callers",
            Self::LogEmitted => "log_emitted",
            Self::Erc20TransferLogEmitted => "erc20_transfer_log_emitted",
            Self::EmittedLogParameter => "emitted_log_parameter",
            Self::FunctionCalled => "function_called",
            Self::CalledFunctionParameter => "called_function_parameter",
            Self::StateChange => "state_change",
            Self::EthBalance => "eth_balance",
            Self::TxValue => "tx_value",
            Self::ViewFunction => "view_function",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for AlertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AlertType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "successful_tx" | "successful_transaction" | "success" => Ok(Self::SuccessfulTx),
            "failed_tx" | "failed_transaction" | "failed" => Ok(Self::FailedTx),
            "allowlisted_callers" | "allowlist" => Ok(Self::AllowlistedCallers),
            "blocklisted_callers" | "blocklist" => Ok(Self::BlocklistedCallers),
            "log_emitted" | "event_emitted" | "event" => Ok(Self::LogEmitted),
            "erc20_transfer_log_emitted" | "erc20_transfer" | "erc20" => {
                Ok(Self::Erc20TransferLogEmitted)
            }
            "emitted_log_parameter" => Ok(Self::EmittedLogParameter),
            "function_called" | "function_call" | "function" => Ok(Self::FunctionCalled),
            "called_function_parameter" => Ok(Self::CalledFunctionParameter),
            "state_change" | "state" => Ok(Self::StateChange),
            "eth_balance" | "balance_change" | "balance" => Ok(Self::EthBalance),
            "tx_value" => Ok(Self::TxValue),
            "view_function" => Ok(Self::ViewFunction),
            _ => Err(format!("Invalid alert type: {s}")),
        }
    }
}

/// Alert expression type (`expressions[].type`)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ExpressionType {
    /// Method call position
    MethodCall,
    /// Caller is in an allowlist
    WhitelistedCallerAddresses,
    /// Caller is in a blocklist
    BlacklistedCallerAddresses,
    /// Transaction touches a contract address
    ContractAddress,
    /// Network filter
    Network,
    /// Contract tag filter
    Tag,
    /// Transaction success/failure
    TxStatus,
    /// Function parameter conditions
    FunctionParams,
    /// Transaction value condition
    TxValue,
    /// Emitted log (event)
    EmittedLog,
    /// State variable change
    StateChange,
    /// View function result
    ViewFunction,
    /// ETH balance threshold
    EthBalance,
    /// ERC20 transfer matcher
    Erc20TransferMatcher,
    /// Transaction error
    TxError,
    /// Internal transaction error
    TxInternalError,
    /// No action within a time window
    NoAction,
    /// Sandwich transaction
    SandwichTransaction,
    /// Unknown type (for forward compatibility)
    #[serde(other)]
    Unknown,
}

/// A condition on a function/log/state parameter
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ParameterCondition {
    /// Parameter name
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_name: Option<String>,

    /// Comparison operator (e.g. ">", "==", "contains")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,

    /// Parameter type (e.g. "uint", "address")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_type: Option<String>,

    /// Nested parameter type
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nested_parameter_type: Option<String>,

    /// Value to compare against
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparison_value: Option<String>,

    /// Compare the change (state change alerts)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare_change: Option<bool>,

    /// Compare against a threshold
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare_threshold: Option<bool>,

    /// Compare as a percentage (state change alerts)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compare_percentage: Option<bool>,
}

/// Expression body (`expressions[].expression`)
///
/// The fields that apply depend on the expression type; unset fields are
/// omitted. Unknown fields are kept in `extra`.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ExpressionBody {
    /// Contract/account address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Addresses (allow/blocklisted callers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,

    /// Network ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,

    /// Contract tag
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Transaction type ("direct", "source", "internal")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<String>,

    /// Transaction success (tx status)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_success: Option<bool>,

    /// Line number (method call)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_number: Option<i64>,

    /// Call position ("first", "last", "any")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub call_position: Option<String>,

    /// Function line number (function params)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_line_number: Option<i64>,

    /// Parameter conditions
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_conditions: Option<Vec<ParameterCondition>>,

    /// Single parameter condition (view function)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameter_condition: Option<ParameterCondition>,

    /// Transaction value (tx value)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_value: Option<String>,

    /// Comparison operator (tx value)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operator: Option<String>,

    /// Event name (emitted log)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_name: Option<String>,

    /// Event ID (emitted log)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_id: Option<String>,

    /// Match any event (emitted log)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_any: Option<bool>,

    /// Decode events (emitted log)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode_events: Option<bool>,

    /// Match non-project contracts (emitted log)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_non_project_contracts: Option<bool>,

    /// Encoded call input (view function)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,

    /// Balance threshold (ETH balance)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,

    /// Log name (ERC20 transfer matcher)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub log_name: Option<String>,

    /// Balances (ERC20 transfer matcher)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balances: Option<String>,

    /// Addresses to ignore (tx error)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addresses_to_ignore: Option<Vec<String>>,

    /// Addresses to match (internal tx error)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub addresses_to_match: Option<Vec<String>>,

    /// Expected log (no action)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_log: Option<serde_json::Value>,

    /// Expected transaction (no action)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub no_transaction: Option<serde_json::Value>,

    /// Window in seconds (no action)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub check_after_seconds: Option<i64>,

    /// Any other fields
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// One alert condition (`expressions[]`)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertExpression {
    /// Expression type
    #[serde(rename = "type")]
    pub expression_type: ExpressionType,

    /// Expression body
    #[serde(default)]
    pub expression: ExpressionBody,
}

impl AlertExpression {
    /// Create an expression from a type and body
    #[must_use]
    pub fn new(expression_type: ExpressionType, expression: ExpressionBody) -> Self {
        Self {
            expression_type,
            expression,
        }
    }

    /// Transactions touching a contract address
    pub fn contract_address(address: impl Into<String>) -> Self {
        Self::new(
            ExpressionType::ContractAddress,
            ExpressionBody {
                address: Some(address.into()),
                ..Default::default()
            },
        )
    }

    /// Transactions on a network
    pub fn network(network_id: impl Into<String>) -> Self {
        Self::new(
            ExpressionType::Network,
            ExpressionBody {
                network_id: Some(network_id.into()),
                ..Default::default()
            },
        )
    }

    /// Transactions touching contracts with a tag
    pub fn tag(tag: impl Into<String>) -> Self {
        Self::new(
            ExpressionType::Tag,
            ExpressionBody {
                tag: Some(tag.into()),
                ..Default::default()
            },
        )
    }

    /// Transaction success (`true`) or failure (`false`)
    #[must_use]
    pub fn tx_status(success: bool) -> Self {
        Self::new(
            ExpressionType::TxStatus,
            ExpressionBody {
                transaction_success: Some(success),
                ..Default::default()
            },
        )
    }
}

/// Delivery channel to attach when creating/updating an alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDeliveryChannelInput {
    /// Delivery channel ID
    pub id: String,

    /// Whether notifications to this channel are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl AlertDeliveryChannelInput {
    /// Attach an enabled delivery channel by ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            enabled: true,
        }
    }
}

// Used by serde(default = "...") attribute; rustc doesn't recognize serde's usage
#[allow(dead_code)]
fn default_true() -> bool {
    true
}

/// Request to create (`POST /alert`) or update (`PATCH /alert/{id}`) an alert
///
/// `name`, `simple_type`, `expressions` and `delivery_channels` are required
/// by the API (`delivery_channels` may be empty).
#[derive(Debug, Clone, Serialize)]
pub struct CreateAlertRequest {
    /// Alert name
    pub name: String,

    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Color (one of "#eeeeee", "#34ace0", "#ffda79", "#ff5252", "#33d9b2")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Alert kind
    pub simple_type: AlertType,

    /// Whether the alert is enabled
    pub enabled: bool,

    /// Conditions that must all match
    pub expressions: Vec<AlertExpression>,

    /// Delivery channels to notify
    pub delivery_channels: Vec<AlertDeliveryChannelInput>,
}

impl CreateAlertRequest {
    /// Create a new alert request with no expressions or delivery channels
    pub fn new(name: impl Into<String>, simple_type: AlertType) -> Self {
        Self {
            name: name.into(),
            description: None,
            color: None,
            simple_type,
            enabled: true,
            expressions: Vec::new(),
            delivery_channels: Vec::new(),
        }
    }

    /// Set description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set color
    #[must_use]
    pub fn color(mut self, color: impl Into<String>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set enabled state
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Add an expression
    #[must_use]
    pub fn expression(mut self, expression: AlertExpression) -> Self {
        self.expressions.push(expression);
        self
    }

    /// Add a `contract_address` expression
    #[must_use]
    pub fn address(self, address: impl Into<String>) -> Self {
        self.expression(AlertExpression::contract_address(address))
    }

    /// Add a `network` expression
    #[must_use]
    pub fn network(self, network_id: impl Into<String>) -> Self {
        self.expression(AlertExpression::network(network_id))
    }

    /// Add a `tag` expression
    #[must_use]
    pub fn tag(self, tag: impl Into<String>) -> Self {
        self.expression(AlertExpression::tag(tag))
    }

    /// Attach a delivery channel by ID
    #[must_use]
    pub fn delivery_channel(mut self, id: impl Into<String>) -> Self {
        self.delivery_channels
            .push(AlertDeliveryChannelInput::new(id));
        self
    }
}

/// Delivery channel attached to an alert (as returned by the API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDeliveryChannel {
    /// Delivery channel ID
    #[serde(default)]
    pub delivery_channel_id: Option<String>,

    /// Whether notifications to this channel are enabled
    #[serde(default)]
    pub enabled: bool,

    /// When the channel was attached
    #[serde(default)]
    pub created_at: Option<String>,

    /// Full channel details (included by some endpoints)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delivery_channel: Option<DeliveryChannel>,
}

/// Alert details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,

    /// Project ID
    #[serde(default)]
    pub project_id: Option<String>,

    /// Alert name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Whether enabled
    #[serde(default)]
    pub enabled: bool,

    /// Color
    #[serde(default)]
    pub color: Option<String>,

    /// Severity ("default", "info", "warning", "danger", "success")
    #[serde(default)]
    pub severity: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Last updated timestamp
    #[serde(default)]
    pub updated_at: Option<String>,

    /// Whether the alert can be edited
    #[serde(default)]
    pub is_editable: bool,

    /// Delivery channels
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub delivery_channels: Vec<AlertDeliveryChannel>,

    /// Conditions
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub expressions: Vec<AlertExpression>,
}

/// Response wrapping a single alert (`POST /alert`, `PATCH /alert/{id}`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertResponse {
    /// The alert
    pub alert: Alert,
}

/// Response when listing alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAlertsResponse {
    /// List of alerts
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub alerts: Vec<Alert>,
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

/// Request to create a webhook destination
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookRequest {
    /// Webhook name
    pub name: String,

    /// Webhook URL
    pub url: String,

    /// Optional description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl CreateWebhookRequest {
    /// Create a new webhook request
    pub fn new(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            description: None,
        }
    }

    /// Add a description
    #[must_use]
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// Webhook destination details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Webhook ID
    pub id: String,

    /// Webhook name
    pub name: String,

    /// Webhook URL
    pub url: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Whether enabled
    #[serde(default)]
    pub enabled: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Response when listing webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListWebhooksResponse {
    /// List of webhooks
    #[serde(default)]
    pub webhooks: Vec<Webhook>,
}

/// Query parameters for alert history
#[derive(Debug, Clone, Default, Serialize)]
pub struct AlertHistoryQuery {
    /// Page number (1-indexed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Items per page (max 100)
    #[serde(rename = "perPage", skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

impl AlertHistoryQuery {
    /// Create a new query with defaults
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page number
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

/// Alert log entry (`GET /alert-history`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertLog {
    /// Alert ID
    #[serde(default)]
    pub alert_id: Option<String>,

    /// Transaction hash that triggered the alert
    #[serde(default)]
    pub tx_hash: Option<String>,

    /// Network ID
    #[serde(default)]
    pub network_id: Option<String>,

    /// Delivery channel notified
    #[serde(default)]
    pub delivery_channel_id: Option<String>,

    /// When the alert was checked/fired
    #[serde(default)]
    pub checked_at: Option<String>,

    /// Whether this was a test alert
    #[serde(default)]
    pub is_test: bool,
}

/// Response for alert history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistoryResponse {
    /// Alert log entries
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub alert_logs: Vec<AlertLog>,
}

/// Request to send a test alert
#[derive(Debug, Clone, Serialize)]
pub struct TestAlertRequest {
    /// Alert ID to test
    pub alert_id: String,

    /// Network ID
    pub network_id: String,

    /// Transaction hash to use for the test
    pub tx_hash: String,
}

impl TestAlertRequest {
    /// Create a new test alert request
    pub fn new(
        alert_id: impl Into<String>,
        network_id: impl Into<String>,
        tx_hash: impl Into<String>,
    ) -> Self {
        Self {
            alert_id: alert_id.into(),
            network_id: network_id.into(),
            tx_hash: tx_hash.into(),
        }
    }
}
