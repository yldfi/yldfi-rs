//! Types for the Transaction API

use serde::{Deserialize, Serialize};

/// Transaction data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// Transaction hash
    pub hash: Option<String>,
    /// Nonce
    pub nonce: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// From address
    pub from_address: Option<String>,
    /// To address
    pub to_address: Option<String>,
    /// Value in wei
    pub value: Option<String>,
    /// Gas
    pub gas: Option<String>,
    /// Gas price
    pub gas_price: Option<String>,
    /// Input data
    pub input: Option<String>,
    /// Receipt cumulative gas used
    pub receipt_cumulative_gas_used: Option<String>,
    /// Receipt gas used
    pub receipt_gas_used: Option<String>,
    /// Receipt contract address
    pub receipt_contract_address: Option<String>,
    /// Receipt root
    pub receipt_root: Option<String>,
    /// Receipt status
    pub receipt_status: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Logs (when verbose)
    pub logs: Option<Vec<TransactionLog>>,
    /// Decoded call (when verbose)
    pub decoded_call: Option<DecodedCall>,
}

/// Transaction log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionLog {
    /// Log index
    pub log_index: Option<String>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// Address
    pub address: Option<String>,
    /// Data
    pub data: Option<String>,
    /// Topic 0
    pub topic0: Option<String>,
    /// Topic 1
    pub topic1: Option<String>,
    /// Topic 2
    pub topic2: Option<String>,
    /// Topic 3
    pub topic3: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Decoded event
    pub decoded_event: Option<DecodedEvent>,
}

/// Decoded function call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedCall {
    /// Function signature
    pub signature: Option<String>,
    /// Function label
    pub label: Option<String>,
    /// Function type
    #[serde(rename = "type")]
    pub call_type: Option<String>,
    /// Parameters
    pub params: Option<Vec<DecodedParam>>,
}

/// Decoded event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedEvent {
    /// Event signature
    pub signature: Option<String>,
    /// Event label
    pub label: Option<String>,
    /// Event type
    #[serde(rename = "type")]
    pub event_type: Option<String>,
    /// Parameters
    pub params: Option<Vec<DecodedParam>>,
}

/// Decoded parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecodedParam {
    /// Parameter name
    pub name: Option<String>,
    /// Parameter value
    pub value: Option<serde_json::Value>,
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: Option<String>,
}

/// Verbose transaction with internal transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerboseTransaction {
    /// Transaction hash
    pub hash: Option<String>,
    /// Nonce
    pub nonce: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// From address
    pub from_address: Option<String>,
    /// To address
    pub to_address: Option<String>,
    /// Value in wei
    pub value: Option<String>,
    /// Gas
    pub gas: Option<String>,
    /// Gas price
    pub gas_price: Option<String>,
    /// Input data
    pub input: Option<String>,
    /// Receipt status
    pub receipt_status: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Logs
    pub logs: Option<Vec<TransactionLog>>,
    /// Decoded call
    pub decoded_call: Option<DecodedCall>,
    /// Internal transactions
    pub internal_transactions: Option<Vec<InternalTransaction>>,
}

/// Internal transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InternalTransaction {
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// Type (call, create, etc)
    #[serde(rename = "type")]
    pub tx_type: Option<String>,
    /// From address
    pub from: Option<String>,
    /// To address
    pub to: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Gas
    pub gas: Option<String>,
    /// Gas used
    pub gas_used: Option<String>,
    /// Input
    pub input: Option<String>,
    /// Output
    pub output: Option<String>,
}
