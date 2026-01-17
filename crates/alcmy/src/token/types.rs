//! Types for the Token API (RPC methods)

use serde::{Deserialize, Serialize};

/// Token balance result
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTokenBalance {
    /// Token contract address
    pub contract_address: String,
    /// Token balance (hex-encoded)
    pub token_balance: Option<String>,
    /// Error if balance couldn't be fetched
    pub error: Option<String>,
}

/// Response for alchemy_getTokenBalances
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTokenBalancesResponse {
    /// Address queried
    pub address: String,
    /// Token balances
    pub token_balances: Vec<RpcTokenBalance>,
    /// Page key for pagination
    pub page_key: Option<String>,
}

/// Token metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTokenMetadata {
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Token logo URL
    pub logo: Option<String>,
}

/// Token specification for getTokenBalances
#[derive(Debug, Clone)]
pub enum TokenSpec {
    /// Get ERC-20 token balances
    Erc20,
    /// Get native token balance
    NativeToken,
    /// Get default tokens (common tokens)
    DefaultTokens,
    /// Get balances for specific token addresses
    Addresses(Vec<String>),
}

impl Serialize for TokenSpec {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TokenSpec::Erc20 => serializer.serialize_str("erc20"),
            TokenSpec::NativeToken => serializer.serialize_str("NATIVE_TOKEN"),
            TokenSpec::DefaultTokens => serializer.serialize_str("DEFAULT_TOKENS"),
            TokenSpec::Addresses(addrs) => addrs.serialize(serializer),
        }
    }
}

/// Options for getTokenBalances
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RpcTokenBalancesOptions {
    /// Page key for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
    /// Maximum number of tokens to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u32>,
}
