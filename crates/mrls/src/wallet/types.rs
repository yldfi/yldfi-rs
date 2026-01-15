//! Types for the Wallet API

use serde::{Deserialize, Serialize};

/// Native balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeBalance {
    /// Balance in wei
    pub balance: String,
}

/// Token balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    /// Token address
    pub token_address: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token logo URL
    pub logo: Option<String>,
    /// Token thumbnail URL
    pub thumbnail: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Balance (raw)
    pub balance: String,
    /// USD price
    pub usd_price: Option<f64>,
    /// USD value
    pub usd_value: Option<f64>,
    /// Possible spam
    pub possible_spam: Option<bool>,
}

/// Wallet transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    /// Transaction hash
    pub hash: String,
    /// Nonce
    pub nonce: Option<String>,
    /// Transaction index
    pub transaction_index: Option<String>,
    /// From address
    pub from_address: String,
    /// To address
    pub to_address: Option<String>,
    /// Value in wei
    pub value: String,
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
}

/// Paginated response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Current page
    pub page: Option<i32>,
    /// Page size
    pub page_size: Option<i32>,
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Results
    pub result: Vec<T>,
}

/// Net worth response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorth {
    /// Total net worth in USD
    pub total_networth_usd: String,
    /// Chains breakdown
    pub chains: Vec<ChainNetWorth>,
}

/// Chain-specific net worth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainNetWorth {
    /// Chain identifier
    pub chain: String,
    /// Native balance in USD
    pub native_balance_usd: String,
    /// Token balance in USD
    pub token_balance_usd: String,
    /// Total net worth in USD
    pub networth_usd: String,
}

/// Active chains response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveChains {
    /// Address
    pub address: String,
    /// List of active chains
    pub active_chains: Vec<ActiveChain>,
}

/// Active chain info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveChain {
    /// Chain identifier
    pub chain: String,
    /// Chain ID
    pub chain_id: String,
    /// First transaction details
    pub first_transaction: Option<TransactionInfo>,
    /// Last transaction details
    pub last_transaction: Option<TransactionInfo>,
}

/// Basic transaction info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Transaction hash
    pub transaction_hash: Option<String>,
}
