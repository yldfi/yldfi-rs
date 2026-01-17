//! Types for wallet activity

use serde::{Deserialize, Serialize};

/// Activity response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityResponse {
    /// Activity items
    pub activity: Vec<ActivityItem>,
    /// Pagination cursor
    pub next_offset: Option<String>,
    /// Warnings
    #[serde(default)]
    pub warnings: Vec<Warning>,
}

/// Activity item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityItem {
    /// Chain ID
    pub chain_id: i64,
    /// Block number
    pub block_number: Option<i64>,
    /// Block time
    pub block_time: Option<String>,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Activity type (approve, mint, burn, receive, send, swap, call)
    #[serde(rename = "type")]
    pub activity_type: Option<String>,
    /// Asset type (native, erc20, erc721, erc1155)
    pub asset_type: Option<String>,
    /// Token address
    pub token_address: Option<String>,
    /// From address
    pub from: Option<String>,
    /// To address
    pub to: Option<String>,
    /// Value
    pub value: Option<String>,
    /// Value in USD
    pub value_usd: Option<f64>,
    /// Token ID (for NFTs)
    pub id: Option<String>,
    /// Spender (for approvals)
    pub spender: Option<String>,
    /// Token metadata
    pub token_metadata: Option<TokenMetadata>,
    /// Decoded function call
    pub function: Option<FunctionCall>,
    /// Contract metadata
    pub contract_metadata: Option<ContractMetadata>,
    // Swap-specific fields
    /// From token address (for swaps)
    pub from_token_address: Option<String>,
    /// From token value (for swaps)
    pub from_token_value: Option<String>,
    /// From token metadata (for swaps)
    pub from_token_metadata: Option<TokenMetadata>,
    /// To token address (for swaps)
    pub to_token_address: Option<String>,
    /// To token value (for swaps)
    pub to_token_value: Option<String>,
    /// To token metadata (for swaps)
    pub to_token_metadata: Option<TokenMetadata>,
}

/// Token metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenMetadata {
    /// Token symbol
    pub symbol: Option<String>,
    /// Token name
    pub name: Option<String>,
    /// Decimals
    pub decimals: Option<u8>,
    /// Logo URL
    pub logo: Option<String>,
    /// Price in USD
    pub price_usd: Option<f64>,
    /// Pool size
    pub pool_size: Option<f64>,
    /// Token standard (e.g., ERC20)
    pub standard: Option<String>,
}

/// Decoded function call
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionCall {
    /// Function signature
    pub signature: Option<String>,
    /// Function name
    pub name: Option<String>,
    /// Decoded inputs
    pub inputs: Option<Vec<FunctionInput>>,
}

/// Function input parameter
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FunctionInput {
    /// Parameter name
    pub name: Option<String>,
    /// Parameter type
    #[serde(rename = "type")]
    pub param_type: Option<String>,
    /// Parameter value
    pub value: Option<serde_json::Value>,
}

/// Contract metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContractMetadata {
    /// Contract name
    pub name: Option<String>,
}

/// Warning
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Warning {
    /// Warning code
    pub code: String,
    /// Message
    pub message: String,
    /// Affected chain IDs
    pub chain_ids: Option<Vec<i64>>,
    /// Documentation URL
    pub docs_url: Option<String>,
}

/// Query options for activity
#[derive(Debug, Clone, Default)]
pub struct ActivityOptions {
    /// Filter by chain IDs (comma-separated)
    pub chain_ids: Option<String>,
    /// Pagination offset
    pub offset: Option<String>,
    /// Results limit (max 100)
    pub limit: Option<u32>,
}

impl ActivityOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref chain_ids) = self.chain_ids {
            params.push(format!("chain_ids={}", chain_ids));
        }
        if let Some(ref offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
