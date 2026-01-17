//! Types for token holders

use serde::{Deserialize, Serialize};

/// Token holders response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenHoldersResponse {
    /// Token address
    pub token_address: String,
    /// Chain ID
    pub chain_id: i64,
    /// Holders list
    pub holders: Vec<Holder>,
    /// Pagination cursor
    pub next_offset: Option<String>,
}

/// Token holder
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Holder {
    /// Wallet address
    pub wallet_address: String,
    /// Balance
    pub balance: String,
    /// First acquired timestamp
    pub first_acquired: String,
    /// Has initiated transfer
    pub has_initiated_transfer: bool,
}

/// Query options for token holders
#[derive(Debug, Clone, Default)]
pub struct TokenHoldersOptions {
    /// Results limit (max 500)
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<String>,
}

impl TokenHoldersOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(ref offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
