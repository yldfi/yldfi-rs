//! Types for `DeFi` positions

use serde::{Deserialize, Serialize};

/// `DeFi` positions response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefiPositionsResponse {
    /// `DeFi` positions
    pub positions: Vec<DefiPosition>,
    /// Aggregations
    pub aggregations: Option<DefiAggregations>,
}

/// `DeFi` position (discriminated by type field)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefiPosition {
    /// Position type (Erc4626, Tokenized, `UniswapV2`, Nft, `NftV4`)
    #[serde(rename = "type")]
    pub position_type: String,
    /// Chain ID
    pub chain_id: i64,
    /// USD value
    pub usd_value: Option<f64>,
    /// Logo URL
    pub logo: Option<String>,

    // Erc4626 fields
    /// Token address (Erc4626, Tokenized)
    pub token: Option<String>,
    /// Token name
    pub token_name: Option<String>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Underlying token address (Erc4626)
    pub underlying_token: Option<String>,
    /// Underlying token name (Erc4626)
    pub underlying_token_name: Option<String>,
    /// Underlying token symbol (Erc4626)
    pub underlying_token_symbol: Option<String>,
    /// Underlying token decimals (Erc4626)
    pub underlying_token_decimals: Option<u8>,

    // Tokenized fields
    /// Token type (Tokenized)
    pub token_type: Option<String>,

    // UniswapV2/Nft/NftV4 fields
    /// Protocol name
    pub protocol: Option<String>,
    /// Pool address (`UniswapV2`, Nft)
    pub pool: Option<String>,
    /// Pool ID (`NftV4`, as byte array)
    pub pool_id: Option<Vec<u8>>,
    /// Pool manager (`NftV4`)
    pub pool_manager: Option<String>,
    /// Salt (`NftV4`, as byte array)
    pub salt: Option<Vec<u8>>,

    // Token pair fields (UniswapV2, Nft, NftV4)
    /// Token 0 address
    pub token0: Option<String>,
    /// Token 0 name
    pub token0_name: Option<String>,
    /// Token 0 symbol
    pub token0_symbol: Option<String>,
    /// Token 0 decimals
    pub token0_decimals: Option<u8>,
    /// Token 0 price
    pub token0_price: Option<f64>,
    /// Token 1 address
    pub token1: Option<String>,
    /// Token 1 name
    pub token1_name: Option<String>,
    /// Token 1 symbol
    pub token1_symbol: Option<String>,
    /// Token 1 decimals
    pub token1_decimals: Option<u8>,
    /// Token 1 price
    pub token1_price: Option<f64>,

    // Balance fields
    /// LP balance (`UniswapV2`)
    pub lp_balance: Option<String>,
    /// Calculated balance
    pub calculated_balance: Option<String>,
    /// Price in USD
    pub price_in_usd: Option<f64>,

    // NFT positions (Nft, NftV4)
    /// Concentrated liquidity positions
    pub positions: Option<Vec<NftPositionDetails>>,
}

/// NFT position details (for Uniswap V3/V4)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftPositionDetails {
    /// Lower tick
    pub tick_lower: Option<i32>,
    /// Upper tick
    pub tick_upper: Option<i32>,
    /// Token ID
    pub token_id: Option<String>,
    /// Token 0 price
    pub token0_price: Option<f64>,
    /// Token 0 holdings
    pub token0_holdings: Option<String>,
    /// Token 0 rewards
    pub token0_rewards: Option<String>,
    /// Token 1 price
    pub token1_price: Option<f64>,
    /// Token 1 holdings
    pub token1_holdings: Option<String>,
    /// Token 1 rewards
    pub token1_rewards: Option<String>,
}

/// `DeFi` aggregations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefiAggregations {
    /// Total USD value
    pub total_usd_value: f64,
    /// Total by chain (`chain_id` -> value)
    pub total_by_chain: Option<serde_json::Value>,
}

/// Query options for `DeFi` positions
#[derive(Debug, Clone, Default)]
pub struct DefiPositionsOptions {
    /// Filter by chain IDs (comma-separated)
    pub chain_ids: Option<String>,
}

impl DefiPositionsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        if let Some(ref chain_ids) = self.chain_ids {
            format!("?chain_ids={chain_ids}")
        } else {
            String::new()
        }
    }
}
