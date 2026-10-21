//! Types for the `DeFi` API

use crate::de::{string_or_f64, string_or_int};
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

/// `DeFi` summary for a wallet
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

/// `DeFi` position entry from `/wallets/{address}/defi/positions`
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
    /// Position
    pub position: Option<DefiPositionDetail>,
}

/// A single `DeFi` position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiPositionDetail {
    /// Position label (e.g. liquidity, supplied, borrowed)
    pub label: Option<String>,
    /// Position address
    pub address: Option<String>,
    /// Position USD value
    #[serde(default, deserialize_with = "string_or_f64")]
    pub balance_usd: Option<f64>,
    /// Unclaimed rewards USD value
    #[serde(default, deserialize_with = "string_or_f64")]
    pub total_unclaimed_usd_value: Option<f64>,
    /// Tokens in the position
    #[serde(default)]
    pub tokens: Vec<DefiToken>,
    /// Protocol-specific details (APY, health factor, range, ...)
    pub position_details: Option<serde_json::Value>,
}

/// Token in `DeFi` position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiToken {
    /// Token type (e.g. supplied, borrowed, reward)
    pub token_type: Option<String>,
    /// Token contract address
    pub contract_address: Option<String>,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token logo
    pub logo: Option<String>,
    /// Token thumbnail
    pub thumbnail: Option<String>,
    /// Decimals (API may return as string or integer)
    #[serde(default, deserialize_with = "string_or_int")]
    pub decimals: Option<String>,
    /// Balance
    pub balance: Option<String>,
    /// Balance formatted
    pub balance_formatted: Option<String>,
    /// USD value
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_value: Option<f64>,
    /// USD price
    #[serde(default, deserialize_with = "string_or_f64")]
    pub usd_price: Option<f64>,
}

/// Positions for one protocol from `/wallets/{address}/defi/{protocol}/positions`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolPositions {
    /// Protocol ID
    pub protocol_id: Option<String>,
    /// Protocol name
    pub protocol_name: Option<String>,
    /// Protocol logo
    pub protocol_logo: Option<String>,
    /// Protocol URL
    pub protocol_url: Option<String>,
    /// Total USD value
    #[serde(default, deserialize_with = "string_or_f64")]
    pub total_usd_value: Option<f64>,
    /// Total unclaimed USD value
    #[serde(default, deserialize_with = "string_or_f64")]
    pub total_unclaimed_usd_value: Option<f64>,
    /// Positions
    #[serde(default)]
    pub positions: Vec<DefiPositionDetail>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOKEN: &str = r#"{
        "token_type": "supplied",
        "name": "Test Token",
        "symbol": "TST",
        "contract_address": "0x0000000000000000000000000000000000000001",
        "decimals": 18,
        "balance": "1000000000000000000",
        "balance_formatted": "1",
        "usd_price": 2.5,
        "usd_value": 2.5
    }"#;

    #[test]
    fn wallet_positions_is_bare_array_with_nested_position() {
        let json = format!(
            r#"[{{
                "protocol_name": "Test Protocol",
                "protocol_id": "test-protocol",
                "position": {{
                    "label": "liquidity",
                    "tokens": [{TOKEN}],
                    "balance_usd": 2.5,
                    "total_unclaimed_usd_value": 0,
                    "position_details": {{"fee_tier": 0.003}}
                }}
            }}]"#
        );
        let positions: Vec<DefiPosition> = serde_json::from_str(&json).unwrap();
        let position = positions[0].position.as_ref().unwrap();
        assert_eq!(positions[0].protocol_id.as_deref(), Some("test-protocol"));
        assert_eq!(position.balance_usd, Some(2.5));
        assert_eq!(position.tokens[0].decimals.as_deref(), Some("18"));
        assert_eq!(position.tokens[0].token_type.as_deref(), Some("supplied"));
    }

    #[test]
    fn protocol_positions_is_object_with_positions() {
        let json = format!(
            r#"{{
                "protocol_name": "Test Protocol",
                "protocol_id": "test-protocol",
                "total_usd_value": 2.5,
                "positions": [{{
                    "label": "supplied",
                    "tokens": [{TOKEN}],
                    "balance_usd": 2.5,
                    "total_unclaimed_usd_value": null
                }}]
            }}"#
        );
        let resp: ProtocolPositions = serde_json::from_str(&json).unwrap();
        assert_eq!(resp.total_usd_value, Some(2.5));
        assert_eq!(resp.positions.len(), 1);
        assert_eq!(resp.positions[0].label.as_deref(), Some("supplied"));
    }
}
