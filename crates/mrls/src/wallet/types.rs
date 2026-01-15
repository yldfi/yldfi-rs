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

/// Token approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenApproval {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Token decimals
    pub token_decimals: Option<u8>,
    /// Spender address
    pub spender_address: Option<String>,
    /// Spender name (if known)
    pub spender_name: Option<String>,
    /// Allowance
    pub allowance: Option<String>,
    /// Allowance formatted
    pub allowance_formatted: Option<String>,
    /// USD value at risk
    pub usd_at_risk: Option<f64>,
    /// Is unlimited
    pub is_unlimited: Option<bool>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
}

/// Wallet history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletHistoryEntry {
    /// Transaction hash
    pub hash: Option<String>,
    /// From address
    pub from_address: Option<String>,
    /// To address
    pub to_address: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Category (send, receive, token send, etc)
    pub category: Option<String>,
    /// Summary
    pub summary: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
    /// NFT transfers
    pub nft_transfers: Option<Vec<serde_json::Value>>,
    /// ERC20 transfers
    pub erc20_transfers: Option<Vec<serde_json::Value>>,
    /// Native transfers
    pub native_transfers: Option<Vec<serde_json::Value>>,
}

/// Wallet stats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStats {
    /// Address
    pub address: Option<String>,
    /// NFTs owned
    pub nfts_owned: Option<i64>,
    /// Collections owned
    pub collections_owned: Option<i64>,
    /// NFT transfers
    pub nft_transfers: Option<i64>,
    /// Token transfers
    pub token_transfers: Option<i64>,
    /// Transactions count
    pub transactions_count: Option<i64>,
}

/// Wallet profitability summary
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletProfitability {
    /// Total profit USD
    pub total_realized_profit_usd: Option<f64>,
    /// Total loss USD
    pub total_realized_loss_usd: Option<f64>,
    /// Total count profitable
    pub total_count_of_profitable_trades: Option<i64>,
    /// Total count losing
    pub total_count_of_losing_trades: Option<i64>,
    /// Total count of trades
    pub total_count_of_trades: Option<i64>,
}

/// Token profitability detail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenProfitability {
    /// Token address
    pub token_address: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Token logo
    pub token_logo: Option<String>,
    /// Realized profit USD
    pub realized_profit_usd: Option<f64>,
    /// Average buy price USD
    pub avg_buy_price_usd: Option<f64>,
    /// Average sell price USD
    pub avg_sell_price_usd: Option<f64>,
    /// Total tokens bought
    pub total_tokens_bought: Option<String>,
    /// Total tokens sold
    pub total_tokens_sold: Option<String>,
    /// Count of trades
    pub count_of_trades: Option<i64>,
}
