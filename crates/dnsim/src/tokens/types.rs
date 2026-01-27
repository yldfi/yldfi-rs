//! Types for token info

use crate::balances::HistoricalPricePoint;
use serde::{Deserialize, Serialize};

/// Tokens response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokensResponse {
    /// Contract address
    pub contract_address: String,
    /// Token info list
    pub tokens: Vec<TokenInfo>,
    /// Pagination cursor
    pub next_offset: Option<String>,
}

/// Token info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    /// Chain name
    pub chain: String,
    /// Chain ID
    pub chain_id: i64,
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Decimals
    pub decimals: u8,
    /// Price in USD
    pub price_usd: Option<f64>,
    /// Historical prices
    pub historical_prices: Option<Vec<HistoricalPricePoint>>,
    /// Total supply
    pub total_supply: Option<String>,
    /// Market cap
    pub market_cap: Option<f64>,
    /// Logo URL
    pub logo: Option<String>,
}

/// Query options for token info
#[derive(Debug, Clone)]
pub struct TokenInfoOptions {
    /// Chain ID (required)
    pub chain_ids: String,
    /// Historical prices (hours in past, comma-separated, max 3 values)
    pub historical_prices: Option<String>,
    /// Pagination offset
    pub offset: Option<String>,
    /// Results limit
    pub limit: Option<u32>,
}

impl TokenInfoOptions {
    #[must_use]
    pub fn new(chain_ids: &str) -> Self {
        Self {
            chain_ids: chain_ids.to_string(),
            historical_prices: None,
            offset: None,
            limit: None,
        }
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = vec![format!("chain_ids={}", self.chain_ids)];
        if let Some(ref historical) = self.historical_prices {
            params.push(format!("historical_prices={historical}"));
        }
        if let Some(ref offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        format!("?{}", params.join("&"))
    }
}
