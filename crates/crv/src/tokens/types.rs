//! Types for the Tokens API

use serde::{Deserialize, Serialize};

/// Response for tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokensResponse {
    /// Whether the request was successful
    pub success: bool,
    /// Token data
    pub data: TokensData,
}

/// Tokens data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensData {
    /// List of tokens
    pub tokens: Vec<TokenInfo>,
}

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Token address
    pub address: String,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// USD price
    pub usd_price: Option<f64>,
    /// Token name
    pub name: Option<String>,
}
