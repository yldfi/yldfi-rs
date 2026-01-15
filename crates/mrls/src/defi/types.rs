//! Types for the DeFi API

use serde::{Deserialize, Serialize};

/// Pair price
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairPrice {
    /// Token 0 address
    pub token0_address: Option<String>,
    /// Token 1 address
    pub token1_address: Option<String>,
    /// Token 0 to token 1 price
    pub token0_to_token1_price: Option<String>,
    /// Token 1 to token 0 price
    pub token1_to_token0_price: Option<String>,
    /// Pair address
    pub pair_address: Option<String>,
}

/// Pair reserves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairReserves {
    /// Reserve 0
    pub reserve0: Option<String>,
    /// Reserve 1
    pub reserve1: Option<String>,
}

/// Pair address response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PairAddress {
    /// Token 0 address
    pub token0_address: Option<String>,
    /// Token 1 address
    pub token1_address: Option<String>,
    /// Pair address
    pub pair_address: Option<String>,
}

/// DeFi summary for a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiSummary {
    /// Total USD value
    pub total_usd_value: Option<f64>,
    /// Active protocols
    pub active_protocols: Option<i32>,
    /// Protocols
    pub protocols: Option<Vec<ProtocolSummary>>,
}

/// Protocol summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSummary {
    /// Protocol ID
    pub protocol_id: Option<String>,
    /// Protocol name
    pub protocol_name: Option<String>,
    /// Protocol logo
    pub protocol_logo: Option<String>,
    /// Total USD value
    pub total_usd_value: Option<f64>,
    /// Position count
    pub position_count: Option<i32>,
}

/// DeFi position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiPosition {
    /// Protocol ID
    pub protocol_id: Option<String>,
    /// Protocol name
    pub protocol_name: Option<String>,
    /// Protocol logo
    pub protocol_logo: Option<String>,
    /// Protocol URL
    pub protocol_url: Option<String>,
    /// Position label
    pub label: Option<String>,
    /// Position type
    pub position_type: Option<String>,
    /// USD value
    pub usd_value: Option<f64>,
    /// Chain
    pub chain: Option<String>,
    /// Tokens
    pub tokens: Option<Vec<DefiToken>>,
    /// Position details
    pub position: Option<serde_json::Value>,
}

/// Token in DeFi position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiToken {
    /// Token address
    pub address: Option<String>,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token logo
    pub logo: Option<String>,
    /// Decimals
    pub decimals: Option<u8>,
    /// Balance
    pub balance: Option<String>,
    /// Balance formatted
    pub balance_formatted: Option<String>,
    /// USD value
    pub usd_value: Option<f64>,
    /// USD price
    pub usd_price: Option<f64>,
}

/// DeFi positions response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiPositionsResponse {
    /// Cursor
    pub cursor: Option<String>,
    /// Page size
    pub page_size: Option<i32>,
    /// Positions
    pub result: Vec<DefiPosition>,
}
