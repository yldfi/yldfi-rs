//! Types for the Bundler API (ERC-4337)

use serde::{Deserialize, Serialize};

/// `UserOperation` v0.6
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationV06 {
    /// Sender address (smart account)
    pub sender: String,
    /// Nonce (hex)
    pub nonce: String,
    /// Init code for account creation (hex)
    #[serde(default)]
    pub init_code: Option<String>,
    /// Call data (hex)
    pub call_data: String,
    /// Call gas limit (hex)
    pub call_gas_limit: String,
    /// Verification gas limit (hex)
    pub verification_gas_limit: String,
    /// Pre-verification gas (hex)
    pub pre_verification_gas: String,
    /// Max fee per gas (hex)
    pub max_fee_per_gas: String,
    /// Max priority fee per gas (hex)
    pub max_priority_fee_per_gas: String,
    /// Paymaster and data (hex)
    #[serde(default)]
    pub paymaster_and_data: Option<String>,
    /// Signature (hex)
    pub signature: String,
}

/// `UserOperation` v0.7
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationV07 {
    /// Sender address (smart account)
    pub sender: String,
    /// Nonce (hex)
    pub nonce: String,
    /// Factory address for account creation
    #[serde(default)]
    pub factory: Option<String>,
    /// Factory data (hex)
    #[serde(default)]
    pub factory_data: Option<String>,
    /// Call data (hex)
    pub call_data: String,
    /// Call gas limit (hex)
    pub call_gas_limit: String,
    /// Verification gas limit (hex)
    pub verification_gas_limit: String,
    /// Pre-verification gas (hex)
    pub pre_verification_gas: String,
    /// Max fee per gas (hex)
    pub max_fee_per_gas: String,
    /// Max priority fee per gas (hex)
    pub max_priority_fee_per_gas: String,
    /// Paymaster address
    #[serde(default)]
    pub paymaster: Option<String>,
    /// Paymaster verification gas limit (hex)
    #[serde(default)]
    pub paymaster_verification_gas_limit: Option<String>,
    /// Paymaster post-op gas limit (hex)
    #[serde(default)]
    pub paymaster_post_op_gas_limit: Option<String>,
    /// Paymaster data (hex)
    #[serde(default)]
    pub paymaster_data: Option<String>,
    /// Signature (hex)
    pub signature: String,
}

/// `UserOperation` (either v0.6 or v0.7)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserOperation {
    V06(UserOperationV06),
    V07(UserOperationV07),
}

/// `UserOperation` receipt
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationReceipt {
    /// `UserOperation` hash
    pub user_op_hash: String,
    /// Entry point address
    pub entry_point: String,
    /// Sender address
    pub sender: String,
    /// Nonce (hex)
    pub nonce: String,
    /// Paymaster address
    #[serde(default)]
    pub paymaster: Option<String>,
    /// Actual gas cost (hex)
    pub actual_gas_cost: String,
    /// Actual gas used (hex)
    pub actual_gas_used: String,
    /// Whether the operation succeeded
    pub success: bool,
    /// Revert reason if failed
    #[serde(default)]
    pub reason: Option<String>,
    /// Transaction receipt
    pub receipt: TransactionReceipt,
    /// Logs from this operation
    pub logs: Vec<Log>,
}

/// Transaction receipt
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionReceipt {
    /// Transaction hash
    pub transaction_hash: String,
    /// Transaction index
    pub transaction_index: String,
    /// Block hash
    pub block_hash: String,
    /// Block number
    pub block_number: String,
    /// Sender
    pub from: String,
    /// Recipient
    pub to: Option<String>,
    /// Cumulative gas used
    pub cumulative_gas_used: String,
    /// Gas used
    pub gas_used: String,
    /// Contract address (if created)
    #[serde(default)]
    pub contract_address: Option<String>,
    /// Logs
    pub logs: Vec<Log>,
    /// Logs bloom
    pub logs_bloom: String,
    /// Status (1 = success, 0 = failure)
    pub status: String,
    /// Effective gas price
    pub effective_gas_price: String,
}

/// Log entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Log {
    /// Contract address
    pub address: String,
    /// Topics
    pub topics: Vec<String>,
    /// Data (hex)
    pub data: String,
    /// Block number
    pub block_number: Option<String>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Log index
    pub log_index: Option<String>,
    /// Removed flag
    #[serde(default)]
    pub removed: bool,
}

/// `UserOperation` by hash response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOperationByHash {
    /// The `UserOperation`
    pub user_operation: serde_json::Value,
    /// Entry point address
    pub entry_point: String,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Block number
    pub block_number: Option<String>,
}

/// Gas estimation result
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasEstimation {
    /// Pre-verification gas (hex)
    pub pre_verification_gas: String,
    /// Verification gas limit (hex)
    pub verification_gas_limit: String,
    /// Call gas limit (hex)
    pub call_gas_limit: String,
    /// Paymaster verification gas limit (hex, v0.7 only)
    #[serde(default)]
    pub paymaster_verification_gas_limit: Option<String>,
    /// Paymaster post-op gas limit (hex, v0.7 only)
    #[serde(default)]
    pub paymaster_post_op_gas_limit: Option<String>,
}

/// State override for gas estimation
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundlerStateOverride {
    /// Balance (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
    /// Nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
    /// Code (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// State overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<std::collections::HashMap<String, String>>,
    /// State diff
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_diff: Option<std::collections::HashMap<String, String>>,
}
