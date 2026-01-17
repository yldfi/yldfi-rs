//! Types for the Debug API

use serde::{Deserialize, Serialize};

/// Tracer type for debug methods
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum TracerType {
    /// Call tracer - traces all calls in a transaction
    #[serde(rename = "callTracer")]
    CallTracer,
    /// Prestate tracer - traces state before execution
    #[serde(rename = "prestateTracer")]
    PrestateTracer,
    /// 4byte tracer - traces 4-byte function selectors
    #[serde(rename = "4byteTracer")]
    FourByteTracer,
}

/// Tracer configuration
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerConfig {
    /// Only return top-level calls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub only_top_call: Option<bool>,
    /// Include logs in traces
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_log: Option<bool>,
}

/// Tracer options for debug methods
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TracerOptions {
    /// Tracer type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracer: Option<TracerType>,
    /// Tracer configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tracer_config: Option<TracerConfig>,
    /// Timeout in string format (e.g., "10s")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
}

impl TracerOptions {
    /// Create call tracer options
    pub fn call_tracer() -> Self {
        Self {
            tracer: Some(TracerType::CallTracer),
            tracer_config: Some(TracerConfig {
                only_top_call: Some(false),
                with_log: Some(true),
            }),
            timeout: None,
        }
    }

    /// Create prestate tracer options
    pub fn prestate_tracer() -> Self {
        Self {
            tracer: Some(TracerType::PrestateTracer),
            tracer_config: None,
            timeout: None,
        }
    }
}

/// State overrides for debug_traceCall
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateOverride {
    /// Balance override (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    /// Nonce override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
    /// Code override (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Storage overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<std::collections::HashMap<String, String>>,
    /// Storage diff (merge with existing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<std::collections::HashMap<String, String>>,
}

/// Call frame from call tracer
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallFrame {
    /// Call type (CALL, DELEGATECALL, STATICCALL, CREATE, CREATE2)
    #[serde(rename = "type")]
    pub call_type: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    #[serde(default)]
    pub to: Option<String>,
    /// Value transferred (hex)
    #[serde(default)]
    pub value: Option<String>,
    /// Gas provided (hex)
    pub gas: String,
    /// Gas used (hex)
    pub gas_used: String,
    /// Input data (hex)
    pub input: String,
    /// Output data (hex)
    #[serde(default)]
    pub output: Option<String>,
    /// Error message if call failed
    #[serde(default)]
    pub error: Option<String>,
    /// Revert reason if reverted
    #[serde(default)]
    pub revert_reason: Option<String>,
    /// Nested calls
    #[serde(default)]
    pub calls: Vec<CallFrame>,
    /// Logs emitted
    #[serde(default)]
    pub logs: Vec<TraceLog>,
}

/// Log from trace
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceLog {
    /// Contract address
    pub address: String,
    /// Log topics
    pub topics: Vec<String>,
    /// Log data (hex)
    pub data: String,
    /// Position in block (only in block traces)
    #[serde(default)]
    pub position: Option<u64>,
}

/// Block trace result
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockTrace {
    /// Transaction hash
    #[serde(default)]
    pub tx_hash: Option<String>,
    /// Trace result (depends on tracer)
    pub result: serde_json::Value,
}

/// Transaction call object for debug_traceCall
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallObject {
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
    /// Max fee per gas (hex, EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee (hex, EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,
    /// Value to send (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Call data (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
}

/// Options for debug_traceCall
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraceCallOptions {
    /// Tracer options
    #[serde(flatten)]
    pub tracer: TracerOptions,
    /// State overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_overrides: Option<std::collections::HashMap<String, StateOverride>>,
    /// Block overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_overrides: Option<BlockOverrides>,
}

/// Block overrides for debug_traceCall
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockOverrides {
    /// Block number (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    /// Block difficulty (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    /// Block timestamp (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<String>,
    /// Gas limit (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<String>,
    /// Coinbase address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coinbase: Option<String>,
    /// Random value (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub random: Option<String>,
    /// Base fee (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee: Option<String>,
}
