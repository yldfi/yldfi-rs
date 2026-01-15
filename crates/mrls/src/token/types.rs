//! Types for the Token API

use serde::{Deserialize, Serialize};

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenMetadata {
    /// Token address
    pub address: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Token logo URL
    pub logo: Option<String>,
    /// Token thumbnail URL
    pub thumbnail: Option<String>,
    /// Block number when the token was created
    pub block_number: Option<String>,
    /// Validated status
    pub validated: Option<i32>,
    /// Created at timestamp
    pub created_at: Option<String>,
    /// Possible spam
    pub possible_spam: Option<bool>,
}

/// Token price
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPrice {
    /// Token address
    pub token_address: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// USD price formatted
    pub usd_price_formatted: Option<String>,
    /// 24h price change percentage
    #[serde(rename = "24hrPercentChange")]
    pub percent_change_24h: Option<String>,
    /// Exchange name
    pub exchange_name: Option<String>,
    /// Exchange address
    pub exchange_address: Option<String>,
    /// Native price
    pub native_price: Option<NativePrice>,
}

/// Native price info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativePrice {
    /// Value
    pub value: Option<String>,
    /// Decimals
    pub decimals: Option<u8>,
    /// Name
    pub name: Option<String>,
    /// Symbol
    pub symbol: Option<String>,
    /// Address
    pub address: Option<String>,
}

/// Token transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenTransfer {
    /// Transaction hash
    pub transaction_hash: String,
    /// Token address
    pub address: String,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block hash
    pub block_hash: Option<String>,
    /// From address
    pub from_address: String,
    /// To address
    pub to_address: String,
    /// Value
    pub value: String,
    /// Log index
    pub log_index: Option<i32>,
    /// Possible spam
    pub possible_spam: Option<bool>,
}

/// Token pair
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPair {
    /// Pair address
    pub pair_address: Option<String>,
    /// Pair label
    pub pair_label: Option<String>,
    /// Exchange name
    pub exchange_name: Option<String>,
    /// Exchange logo
    pub exchange_logo: Option<String>,
    /// USD price
    pub usd_price: Option<f64>,
    /// USD price 24hr change
    #[serde(rename = "usdPrice24hrPercentChange")]
    pub usd_price_24hr_percent_change: Option<f64>,
    /// Liquidity in USD
    pub liquidity_usd: Option<f64>,
}

/// Top token holder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    /// Holder address
    pub owner: String,
    /// Balance
    pub balance: String,
    /// Balance formatted
    pub balance_formatted: Option<String>,
    /// Is contract
    pub is_contract: Option<bool>,
    /// USD value
    pub usd_value: Option<f64>,
    /// Percentage of total supply
    pub percentage_relative_to_total_supply: Option<f64>,
}

/// Paginated token holders response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHoldersResponse {
    /// Cursor for pagination
    pub cursor: Option<String>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    pub result: Vec<TokenHolder>,
}
