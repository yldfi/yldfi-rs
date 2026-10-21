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
    /// Init code for account creation (hex; `None` is sent as `"0x"`)
    #[serde(default, serialize_with = "empty_hex_if_none")]
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
    /// Paymaster and data (hex; `None` is sent as `"0x"`)
    #[serde(default, serialize_with = "empty_hex_if_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub factory: Option<String>,
    /// Factory data (hex)
    #[serde(default, skip_serializing_if = "Option::is_none")]
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paymaster: Option<String>,
    /// Paymaster verification gas limit (hex)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paymaster_verification_gas_limit: Option<String>,
    /// Paymaster post-op gas limit (hex)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paymaster_post_op_gas_limit: Option<String>,
    /// Paymaster data (hex)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub paymaster_data: Option<String>,
    /// Signature (hex)
    pub signature: String,
}

/// `UserOperation` (either v0.6 or v0.7)
///
/// Deserialization picks the version from the fields present: v0.6 always
/// carries `initCode` and `paymasterAndData` (as `"0x"` when empty), which
/// v0.7 split into `factory`/`factoryData` and `paymaster*`. An op with
/// neither v0.6 field is v0.7.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum UserOperation {
    V06(UserOperationV06),
    V07(UserOperationV07),
}

impl<'de> Deserialize<'de> for UserOperation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;

        let value = serde_json::Value::deserialize(deserializer)?;
        let is_v06 = value
            .as_object()
            .is_some_and(|o| o.contains_key("initCode") || o.contains_key("paymasterAndData"));
        if is_v06 {
            serde_json::from_value(value)
                .map(UserOperation::V06)
                .map_err(D::Error::custom)
        } else {
            serde_json::from_value(value)
                .map(UserOperation::V07)
                .map_err(D::Error::custom)
        }
    }
}

fn empty_hex_if_none<S>(value: &Option<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(value.as_deref().unwrap_or("0x"))
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
    pub user_operation: UserOperation,
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
    /// Nonce (sent as a hex quantity)
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "crate::serde_hex::opt_u64_as_hex"
    )]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundler_state_override_nonce_is_hex_quantity() {
        let value = serde_json::to_value(BundlerStateOverride {
            nonce: Some(1),
            ..Default::default()
        })
        .unwrap();
        assert_eq!(value, serde_json::json!({ "nonce": "0x1" }));
    }

    fn base_op() -> serde_json::Value {
        serde_json::json!({
            "sender": "0x0000000000000000000000000000000000000001",
            "nonce": "0x0",
            "callData": "0x",
            "callGasLimit": "0x1",
            "verificationGasLimit": "0x2",
            "preVerificationGas": "0x3",
            "maxFeePerGas": "0x4",
            "maxPriorityFeePerGas": "0x5",
            "signature": "0x"
        })
    }

    #[test]
    fn v07_op_with_factory_and_paymaster_parses_as_v07() {
        let mut op = base_op();
        let obj = op.as_object_mut().unwrap();
        obj.insert(
            "factory".into(),
            "0x0000000000000000000000000000000000000002".into(),
        );
        obj.insert("factoryData".into(), "0xab".into());
        obj.insert(
            "paymaster".into(),
            "0x0000000000000000000000000000000000000003".into(),
        );
        obj.insert("paymasterData".into(), "0xcd".into());
        let parsed: UserOperation = serde_json::from_value(op).unwrap();
        let UserOperation::V07(v07) = parsed else {
            panic!("expected v0.7");
        };
        assert_eq!(
            v07.factory.as_deref(),
            Some("0x0000000000000000000000000000000000000002")
        );
        assert_eq!(v07.paymaster_data.as_deref(), Some("0xcd"));
    }

    #[test]
    fn v06_op_parses_as_v06() {
        let mut op = base_op();
        let obj = op.as_object_mut().unwrap();
        obj.insert("initCode".into(), "0x".into());
        obj.insert("paymasterAndData".into(), "0xef".into());
        let parsed: UserOperation = serde_json::from_value(op).unwrap();
        let UserOperation::V06(v06) = parsed else {
            panic!("expected v0.6");
        };
        assert_eq!(v06.paymaster_and_data.as_deref(), Some("0xef"));
    }

    #[test]
    fn user_operation_serialization_omits_nulls() {
        let v07: UserOperation = serde_json::from_value(base_op()).unwrap();
        assert!(matches!(v07, UserOperation::V07(_)));
        let value = serde_json::to_value(&v07).unwrap();
        let obj = value.as_object().unwrap();
        assert!(!obj.values().any(serde_json::Value::is_null), "{value}");
        assert!(!obj.contains_key("paymaster") && !obj.contains_key("factory"));

        let mut op = base_op();
        op.as_object_mut()
            .unwrap()
            .insert("initCode".into(), "0x".into());
        let mut v06: UserOperation = serde_json::from_value(op).unwrap();
        if let UserOperation::V06(inner) = &mut v06 {
            inner.init_code = None;
            inner.paymaster_and_data = None;
        }
        let value = serde_json::to_value(&v06).unwrap();
        assert_eq!(value["initCode"], "0x");
        assert_eq!(value["paymasterAndData"], "0x");
    }

    #[test]
    fn user_operation_by_hash_decodes_typed_op() {
        let parsed: UserOperationByHash = serde_json::from_value(serde_json::json!({
            "userOperation": base_op(),
            "entryPoint": "0x0000000000000000000000000000000000000004",
            "transactionHash": null,
            "blockHash": null,
            "blockNumber": null
        }))
        .unwrap();
        assert!(matches!(parsed.user_operation, UserOperation::V07(_)));
    }
}
