//! Types for transactions

use crate::activity::Warning;
use serde::{Deserialize, Serialize};

/// Transactions response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionsResponse {
    /// Wallet address (may not be present)
    pub wallet_address: Option<String>,
    /// Transactions
    pub transactions: Vec<Transaction>,
    /// Transaction errors
    pub errors: Option<TransactionErrors>,
    /// Warnings
    #[serde(default)]
    pub warnings: Vec<Warning>,
    /// Pagination cursor
    pub next_offset: Option<String>,
    /// Request timestamp
    pub request_time: Option<String>,
    /// Response timestamp
    pub response_time: Option<String>,
}

/// Transaction errors
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionErrors {
    /// Error message
    pub error_message: Option<String>,
    /// Transaction-specific errors
    pub transaction_errors: Option<Vec<TransactionErrorInfo>>,
}

/// Transaction error information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionErrorInfo {
    /// Address
    pub address: Option<String>,
    /// Chain ID
    pub chain_id: Option<i64>,
    /// Error description
    pub description: Option<String>,
}

/// Transaction
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    /// Wallet or contract address
    pub address: String,
    /// Block hash
    pub block_hash: String,
    /// Block number
    pub block_number: i64,
    /// Block time
    pub block_time: String,
    /// Block version
    pub block_version: Option<i64>,
    /// Chain name
    pub chain: String,
    /// Chain ID
    pub chain_id: i64,
    /// From address
    pub from: String,
    /// To address
    pub to: Option<String>,
    /// Call data
    pub data: Option<String>,
    /// Gas price (hex)
    pub gas_price: Option<String>,
    /// Gas used (hex)
    pub gas_used: Option<String>,
    /// Effective gas price (hex)
    pub effective_gas_price: Option<String>,
    /// Transaction hash
    pub hash: String,
    /// Transaction index
    pub index: Option<i64>,
    /// Max fee per gas (hex)
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee per gas (hex)
    pub max_priority_fee_per_gas: Option<String>,
    /// Nonce (hex)
    pub nonce: Option<String>,
    /// Transaction type
    pub transaction_type: Option<String>,
    /// Value (hex)
    pub value: Option<String>,
    /// Transaction success
    pub success: Option<bool>,
    /// Decoded function call
    pub decoded: Option<DecodedCall>,
    /// Event logs
    pub logs: Option<Vec<TransactionLog>>,
}

/// Decoded function call
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DecodedCall {
    /// Function name
    pub name: Option<String>,
    /// Decoded inputs
    pub inputs: Option<Vec<DecodedInput>>,
}

/// Decoded input parameter
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DecodedInput {
    /// Parameter name
    pub name: Option<String>,
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: Option<String>,
    /// Parameter value
    pub value: Option<serde_json::Value>,
}

/// Transaction log
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TransactionLog {
    /// Contract address
    pub address: String,
    /// Log data
    pub data: String,
    /// Topics
    pub topics: Vec<String>,
    /// Decoded event
    pub decoded: Option<DecodedEvent>,
}

/// Decoded event
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DecodedEvent {
    /// Event name
    pub name: Option<String>,
    /// Decoded inputs
    pub inputs: Option<Vec<DecodedInput>>,
}

/// Query options for transactions
#[derive(Debug, Clone, Default)]
pub struct TransactionsOptions {
    /// Filter by chain IDs (comma-separated)
    pub chain_ids: Option<String>,
    /// Results limit (max 100)
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<String>,
    /// Decode transaction logs
    pub decode: Option<bool>,
}

impl TransactionsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref chain_ids) = self.chain_ids {
            params.push(format!("chain_ids={chain_ids}"));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(ref offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if let Some(decode) = self.decode {
            params.push(format!("decode={decode}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
