//! Types for Alerts API
//!
//! # API Limitations
//!
//! The Tenderly Alerts API uses an undocumented request format for creating alerts.
//! While the types in this module are designed based on available documentation,
//! the actual API requires an `expressions` array with a specific structure:
//!
//! ```json
//! {
//!   "name": "Alert Name",
//!   "description": "optional",
//!   "enabled": true,
//!   "expressions": [
//!     { "type": "method_call", "expression": { ... } },
//!     { "type": "state_change", "expression": { ... } }
//!   ],
//!   "delivery_channels": [...]
//! }
//! ```
//!
//! Known expression types: `method_call`, `state_change`, `contract_address`, `emitted_log`
//!
//! The exact structure of the `expression` object for each type is not publicly documented.
//! Read operations (list, get, history) work correctly with the current types.
//!
//! Similarly, the webhooks API requires a `source_type` field with undocumented valid values.
//!
//! See: <https://docs.tenderly.co/alerts/api>

use serde::{Deserialize, Serialize};

/// Alert trigger types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AlertType {
    /// Transaction success
    SuccessfulTransaction,
    /// Transaction failure
    FailedTransaction,
    /// Function call
    FunctionCall,
    /// Event emitted
    EventEmitted,
    /// ERC20 token transfer
    Erc20Transfer,
    /// ERC721 token transfer
    Erc721Transfer,
    /// State change
    StateChange,
    /// ETH balance change
    BalanceChange,
    /// Contract deployed
    ContractDeployed,
    /// Block mined
    BlockMined,
    /// Whale alert (large transfers)
    WhaleAlert,
    /// Custom expression
    Expression,
}

impl AlertType {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SuccessfulTransaction => "successful_transaction",
            Self::FailedTransaction => "failed_transaction",
            Self::FunctionCall => "function_call",
            Self::EventEmitted => "event_emitted",
            Self::Erc20Transfer => "erc20_transfer",
            Self::Erc721Transfer => "erc721_transfer",
            Self::StateChange => "state_change",
            Self::BalanceChange => "balance_change",
            Self::ContractDeployed => "contract_deployed",
            Self::BlockMined => "block_mined",
            Self::WhaleAlert => "whale_alert",
            Self::Expression => "expression",
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
            "successful_transaction" | "success" => Ok(Self::SuccessfulTransaction),
            "failed_transaction" | "failed" => Ok(Self::FailedTransaction),
            "function_call" | "function" => Ok(Self::FunctionCall),
            "event_emitted" | "event" => Ok(Self::EventEmitted),
            "erc20_transfer" | "erc20" => Ok(Self::Erc20Transfer),
            "erc721_transfer" | "erc721" => Ok(Self::Erc721Transfer),
            "state_change" | "state" => Ok(Self::StateChange),
            "balance_change" | "balance" => Ok(Self::BalanceChange),
            "contract_deployed" | "deployed" => Ok(Self::ContractDeployed),
            "block_mined" | "block" => Ok(Self::BlockMined),
            "whale_alert" | "whale" => Ok(Self::WhaleAlert),
            "expression" | "custom" => Ok(Self::Expression),
            _ => Err(format!("Invalid alert type: {}", s)),
        }
    }
}

/// Alert target type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AlertTarget {
    /// Single address
    Address,
    /// All addresses on a network
    Network,
    /// All addresses in a project
    Project,
    /// All addresses with a specific tag
    Tag,
}

impl AlertTarget {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Address => "address",
            Self::Network => "network",
            Self::Project => "project",
            Self::Tag => "tag",
        }
    }
}

impl std::fmt::Display for AlertTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AlertTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "address" | "addr" => Ok(Self::Address),
            "network" | "net" => Ok(Self::Network),
            "project" | "proj" => Ok(Self::Project),
            "tag" => Ok(Self::Tag),
            _ => Err(format!("Invalid alert target: {}", s)),
        }
    }
}

/// Alert destination type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DestinationType {
    /// Email notification
    Email,
    /// Slack webhook
    Slack,
    /// Discord webhook
    Discord,
    /// Telegram bot
    Telegram,
    /// PagerDuty
    PagerDuty,
    /// Custom webhook
    Webhook,
    /// Web3 Action
    Web3Action,
}

impl DestinationType {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Slack => "slack",
            Self::Discord => "discord",
            Self::Telegram => "telegram",
            Self::PagerDuty => "pagerduty",
            Self::Webhook => "webhook",
            Self::Web3Action => "web3_action",
        }
    }
}

impl std::fmt::Display for DestinationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for DestinationType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace('-', "_").as_str() {
            "email" => Ok(Self::Email),
            "slack" => Ok(Self::Slack),
            "discord" => Ok(Self::Discord),
            "telegram" => Ok(Self::Telegram),
            "pagerduty" | "pager_duty" => Ok(Self::PagerDuty),
            "webhook" => Ok(Self::Webhook),
            "web3_action" | "web3action" | "action" => Ok(Self::Web3Action),
            _ => Err(format!("Invalid destination type: {}", s)),
        }
    }
}

/// Request to create an alert
#[derive(Debug, Clone, Serialize)]
pub struct CreateAlertRequest {
    /// Alert name
    pub name: String,

    /// Alert type
    pub alert_type: AlertType,

    /// Network ID
    pub network: String,

    /// Target type
    pub target: AlertTarget,

    /// Target addresses (for Address target type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,

    /// Target tag (for Tag target type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Alert parameters (type-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<AlertParameters>,

    /// Whether the alert is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

// Used by serde(default = "...") attribute; rustc doesn't recognize serde's usage
#[allow(dead_code)]
fn default_true() -> bool {
    true
}

impl CreateAlertRequest {
    /// Create a new alert request
    pub fn new(
        name: impl Into<String>,
        alert_type: AlertType,
        network: impl Into<String>,
        target: AlertTarget,
    ) -> Self {
        Self {
            name: name.into(),
            alert_type,
            network: network.into(),
            target,
            addresses: None,
            tag: None,
            parameters: None,
            enabled: true,
        }
    }

    /// Set target addresses
    #[must_use]
    pub fn addresses(mut self, addresses: Vec<String>) -> Self {
        self.addresses = Some(addresses);
        self
    }

    /// Add a single address
    #[must_use]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.addresses
            .get_or_insert_with(Vec::new)
            .push(address.into());
        self
    }

    /// Set target tag
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Set alert parameters
    #[must_use]
    pub fn parameters(mut self, params: AlertParameters) -> Self {
        self.parameters = Some(params);
        self
    }

    /// Set enabled state
    #[must_use]
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Alert parameters (type-specific configuration)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertParameters {
    /// Function signature (for FunctionCall type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_signature: Option<String>,

    /// Event signature (for EventEmitted type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event_signature: Option<String>,

    /// Token address (for ERC20/ERC721 transfers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_address: Option<String>,

    /// Minimum value threshold (for WhaleAlert)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<String>,

    /// Custom expression (for Expression type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expression: Option<String>,

    /// State variable to watch (for StateChange)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_variable: Option<String>,
}

impl AlertParameters {
    /// Create parameters for a function call alert
    pub fn function_call(signature: impl Into<String>) -> Self {
        Self {
            function_signature: Some(signature.into()),
            ..Default::default()
        }
    }

    /// Create parameters for an event alert
    pub fn event(signature: impl Into<String>) -> Self {
        Self {
            event_signature: Some(signature.into()),
            ..Default::default()
        }
    }

    /// Create parameters for a whale alert
    pub fn whale(threshold: impl Into<String>) -> Self {
        Self {
            threshold: Some(threshold.into()),
            ..Default::default()
        }
    }

    /// Create parameters for a custom expression
    pub fn expression(expr: impl Into<String>) -> Self {
        Self {
            expression: Some(expr.into()),
            ..Default::default()
        }
    }
}

/// Alert details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,

    /// Alert name
    pub name: String,

    /// Alert type
    pub alert_type: AlertType,

    /// Network ID
    pub network: String,

    /// Target type
    pub target: AlertTarget,

    /// Target addresses
    #[serde(default)]
    pub addresses: Vec<String>,

    /// Target tag
    #[serde(default)]
    pub tag: Option<String>,

    /// Whether enabled
    #[serde(default)]
    pub enabled: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Destinations
    #[serde(default)]
    pub destinations: Vec<AlertDestination>,
}

/// Alert destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertDestination {
    /// Destination ID
    pub id: String,

    /// Destination type
    pub destination_type: DestinationType,

    /// Destination-specific config
    #[serde(default)]
    pub config: serde_json::Value,

    /// Whether enabled
    #[serde(default)]
    pub enabled: bool,
}

/// Response when listing alerts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListAlertsResponse {
    /// List of alerts
    #[serde(default)]
    pub alerts: Vec<Alert>,
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

/// Request to add a destination to an alert
#[derive(Debug, Clone, Serialize)]
pub struct AddDestinationRequest {
    /// Destination type
    pub destination_type: DestinationType,

    /// Destination ID (webhook ID, email, etc.)
    pub destination_id: String,
}

impl AddDestinationRequest {
    /// Create a webhook destination
    pub fn webhook(webhook_id: impl Into<String>) -> Self {
        Self {
            destination_type: DestinationType::Webhook,
            destination_id: webhook_id.into(),
        }
    }

    /// Create an email destination
    pub fn email(email: impl Into<String>) -> Self {
        Self {
            destination_type: DestinationType::Email,
            destination_id: email.into(),
        }
    }

    /// Create a Slack destination
    pub fn slack(webhook_url: impl Into<String>) -> Self {
        Self {
            destination_type: DestinationType::Slack,
            destination_id: webhook_url.into(),
        }
    }

    /// Create a Discord destination
    pub fn discord(webhook_url: impl Into<String>) -> Self {
        Self {
            destination_type: DestinationType::Discord,
            destination_id: webhook_url.into(),
        }
    }
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

/// Alert execution history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistoryEntry {
    /// Entry ID
    pub id: String,

    /// Alert ID
    pub alert_id: String,

    /// Alert name
    #[serde(default)]
    pub alert_name: Option<String>,

    /// Network ID
    #[serde(default)]
    pub network_id: Option<String>,

    /// Transaction hash that triggered the alert
    #[serde(default)]
    pub transaction_hash: Option<String>,

    /// Block number
    #[serde(default)]
    pub block_number: Option<u64>,

    /// Timestamp when triggered
    #[serde(default)]
    pub created_at: Option<String>,

    /// Whether the alert was successfully delivered
    #[serde(default)]
    pub delivered: bool,
}

/// Response for alert history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertHistoryResponse {
    /// Alert history entries
    #[serde(default)]
    pub alert_history: Vec<AlertHistoryEntry>,
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
