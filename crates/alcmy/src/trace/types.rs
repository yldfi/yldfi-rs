//! Types for the Trace API (Parity-style)

use serde::{Deserialize, Serialize};

/// Trace type for trace methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TraceType {
    /// Basic execution trace
    Trace,
    /// State diff trace
    StateDiff,
    /// VM trace
    VmTrace,
}

/// Call object for `trace_call`
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallRequest {
    /// Sender address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Recipient address
    pub to: String,
    /// Gas limit (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<String>,
    /// Gas price (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    /// Value (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Call data (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Trace action
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceAction {
    /// Call type
    pub call_type: Option<String>,
    /// Sender
    pub from: Option<String>,
    /// Recipient
    pub to: Option<String>,
    /// Gas (hex)
    pub gas: Option<String>,
    /// Input data (hex)
    pub input: Option<String>,
    /// Value (hex)
    pub value: Option<String>,
    /// Init code for create (hex)
    pub init: Option<String>,
    /// Created address
    pub address: Option<String>,
    /// Refund address (for suicide)
    pub refund_address: Option<String>,
    /// Balance (hex)
    pub balance: Option<String>,
}

/// Trace result
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceResult {
    /// Gas used (hex)
    pub gas_used: Option<String>,
    /// Output data (hex)
    pub output: Option<String>,
    /// Created address
    pub address: Option<String>,
    /// Code (hex)
    pub code: Option<String>,
}

/// Single trace entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trace {
    /// Action details
    pub action: TraceAction,
    /// Block hash
    pub block_hash: Option<String>,
    /// Block number
    pub block_number: Option<u64>,
    /// Result (if successful)
    pub result: Option<TraceResult>,
    /// Error message (if failed)
    pub error: Option<String>,
    /// Subtraces count
    pub subtraces: u32,
    /// Trace address (path in call tree)
    pub trace_address: Vec<u32>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Transaction position
    pub transaction_position: Option<u32>,
    /// Trace type
    #[serde(rename = "type")]
    pub trace_type: String,
}

/// State diff for an account
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDiffAccount {
    /// Balance change
    pub balance: Option<StateDiffValue>,
    /// Code change
    pub code: Option<StateDiffValue>,
    /// Nonce change
    pub nonce: Option<StateDiffValue>,
    /// Storage changes
    pub storage: Option<std::collections::HashMap<String, StateDiffValue>>,
}

/// State diff value (from -> to)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum StateDiffValue {
    /// Value unchanged
    Unchanged(String),
    /// Value changed
    Changed { from: String, to: String },
    /// Value added
    Added { from: (), to: String },
    /// Value removed
    Removed { from: String, to: () },
}

/// Trace call response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallResponse {
    /// Trace output
    #[serde(default)]
    pub trace: Vec<Trace>,
    /// State diff
    #[serde(default)]
    pub state_diff: Option<std::collections::HashMap<String, StateDiffAccount>>,
    /// VM trace
    #[serde(default)]
    pub vm_trace: Option<serde_json::Value>,
    /// Output (hex)
    pub output: Option<String>,
}

/// Filter for `trace_filter`
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceFilter {
    /// Starting block (hex or tag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_block: Option<String>,
    /// Ending block (hex or tag)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_block: Option<String>,
    /// Filter by sender addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_address: Option<Vec<String>>,
    /// Filter by recipient addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_address: Option<Vec<String>>,
    /// Skip first N traces
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<u32>,
    /// Maximum traces to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
}
