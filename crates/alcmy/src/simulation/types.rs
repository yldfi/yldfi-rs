//! Types for the Transaction Simulation API

use serde::{Deserialize, Serialize};

/// Transaction to simulate
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationTransaction {
    /// Sender address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Recipient address (required)
    pub to: String,
    /// Value to send (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Call data (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
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
}

/// Asset change from simulation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetChange {
    /// Asset type (native, erc20, erc721, erc1155)
    pub asset_type: String,
    /// Change type (transfer, approve, etc.)
    pub change_type: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Amount (for fungible tokens)
    #[serde(default)]
    pub amount: Option<String>,
    /// Raw amount (hex)
    #[serde(default)]
    pub raw_amount: Option<String>,
    /// Contract address (for tokens)
    #[serde(default)]
    pub contract_address: Option<String>,
    /// Token ID (for NFTs)
    #[serde(default)]
    pub token_id: Option<String>,
    /// Token name
    #[serde(default)]
    pub name: Option<String>,
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token decimals
    #[serde(default)]
    pub decimals: Option<u8>,
    /// Token logo URL
    #[serde(default)]
    pub logo: Option<String>,
}

/// Response for simulateAssetChanges
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateAssetChangesResponse {
    /// List of asset changes
    pub changes: Vec<AssetChange>,
    /// Gas used
    pub gas_used: Option<String>,
    /// Error if simulation failed
    pub error: Option<SimulationError>,
}

/// Simulation error
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulationError {
    /// Error message
    pub message: String,
    /// Revert reason (if applicable)
    #[serde(default)]
    pub revert_reason: Option<String>,
}

/// Response for simulateAssetChangesBundle
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateAssetChangesBundleResponse {
    /// Results for each transaction
    pub results: Vec<SimulateAssetChangesResponse>,
}

/// Execution format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionFormat {
    /// Nested call structure
    Nested,
    /// Flat call list
    Flat,
}

/// Decoded function call
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedCall {
    /// Function name
    pub name: Option<String>,
    /// Function signature
    pub signature: Option<String>,
    /// Decoded inputs
    pub inputs: Option<serde_json::Value>,
    /// Decoded outputs
    pub outputs: Option<serde_json::Value>,
}

/// Execution call trace
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionCall {
    /// Call type
    #[serde(rename = "type")]
    pub call_type: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: Option<String>,
    /// Value (hex)
    pub value: Option<String>,
    /// Gas (hex)
    pub gas: Option<String>,
    /// Gas used (hex)
    pub gas_used: Option<String>,
    /// Input data (hex)
    pub input: Option<String>,
    /// Output data (hex)
    pub output: Option<String>,
    /// Decoded call info
    pub decoded: Option<DecodedCall>,
    /// Error message
    pub error: Option<String>,
    /// Revert reason
    pub revert_reason: Option<String>,
    /// Nested calls (for NESTED format)
    #[serde(default)]
    pub calls: Vec<ExecutionCall>,
}

/// Decoded event log
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedLog {
    /// Event name
    pub name: Option<String>,
    /// Event signature
    pub signature: Option<String>,
    /// Decoded parameters
    pub params: Option<serde_json::Value>,
}

/// Execution log
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionLog {
    /// Contract address
    pub address: String,
    /// Log topics
    pub topics: Vec<String>,
    /// Log data (hex)
    pub data: String,
    /// Decoded log info
    pub decoded: Option<DecodedLog>,
}

/// Response for simulateExecution
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimulateExecutionResponse {
    /// Execution trace (call tree or flat list)
    pub calls: serde_json::Value,
    /// Event logs
    pub logs: Vec<ExecutionLog>,
}
