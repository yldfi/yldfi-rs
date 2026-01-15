//! Types for the Block API

use serde::{Deserialize, Serialize};

/// Block data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Block number
    pub number: Option<String>,
    /// Block hash
    pub hash: Option<String>,
    /// Parent hash
    pub parent_hash: Option<String>,
    /// Nonce
    pub nonce: Option<String>,
    /// SHA3 uncles
    pub sha3_uncles: Option<String>,
    /// Logs bloom
    pub logs_bloom: Option<String>,
    /// Transactions root
    pub transactions_root: Option<String>,
    /// State root
    pub state_root: Option<String>,
    /// Receipts root
    pub receipts_root: Option<String>,
    /// Miner address
    pub miner: Option<String>,
    /// Difficulty
    pub difficulty: Option<String>,
    /// Total difficulty
    pub total_difficulty: Option<String>,
    /// Extra data
    pub extra_data: Option<String>,
    /// Block size
    pub size: Option<String>,
    /// Gas limit
    pub gas_limit: Option<String>,
    /// Gas used
    pub gas_used: Option<String>,
    /// Timestamp
    pub timestamp: Option<String>,
    /// Transaction count
    pub transaction_count: Option<i32>,
    /// Transactions (when included)
    pub transactions: Option<Vec<serde_json::Value>>,
}

/// Date to block response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateToBlock {
    /// Block number
    pub block: Option<i64>,
    /// Date
    pub date: Option<String>,
    /// Timestamp
    pub timestamp: Option<i64>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Hash
    pub hash: Option<String>,
    /// Parent hash
    pub parent_hash: Option<String>,
}

/// Latest block number response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestBlock {
    /// Block number
    pub block: Option<i64>,
}
