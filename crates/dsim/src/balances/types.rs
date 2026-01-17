//! Types for token balances

use crate::activity::Warning;
use serde::{Deserialize, Serialize};

/// Balances response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalancesResponse {
    /// Wallet address
    pub wallet_address: String,
    /// Token balances
    pub balances: Vec<BalanceData>,
    /// Pagination cursor
    pub next_offset: Option<String>,
    /// Errors
    pub errors: Option<BalanceErrors>,
    /// Warnings
    #[serde(default)]
    pub warnings: Vec<Warning>,
    /// Request timestamp
    pub request_time: Option<String>,
    /// Response timestamp
    pub response_time: Option<String>,
}

/// Balance errors
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceErrors {
    /// Error message
    pub error_message: Option<String>,
    /// Token-specific errors
    pub token_errors: Option<Vec<BalanceErrorInfo>>,
}

/// Balance error information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceErrorInfo {
    /// Chain ID
    pub chain_id: Option<i64>,
    /// Token address
    pub address: Option<String>,
    /// Error description
    pub description: Option<String>,
}

/// Single balance response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SingleBalanceResponse {
    /// Wallet address
    pub wallet_address: String,
    /// Token balances
    pub balances: Vec<BalanceData>,
    /// Warnings
    #[serde(default)]
    pub warnings: Vec<Warning>,
    /// Request timestamp
    pub request_time: Option<String>,
    /// Response timestamp
    pub response_time: Option<String>,
}

/// Balance data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceData {
    /// Chain name
    pub chain: String,
    /// Chain ID
    pub chain_id: i64,
    /// Token address
    pub address: String,
    /// Balance amount
    pub amount: String,
    /// Token symbol
    pub symbol: String,
    /// Token name (may be absent for native tokens)
    pub name: Option<String>,
    /// Token decimals
    pub decimals: u8,
    /// Price in USD
    pub price_usd: Option<f64>,
    /// Value in USD
    pub value_usd: Option<f64>,
    /// Pool size
    pub pool_size: Option<f64>,
    /// Low liquidity flag
    pub low_liquidity: Option<bool>,
    /// Historical prices
    pub historical_prices: Option<Vec<HistoricalPricePoint>>,
    /// Token metadata
    pub token_metadata: Option<BalanceTokenMetadata>,
    /// Pool metadata
    pub pool: Option<PoolMetadata>,
}

/// Historical price point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoricalPricePoint {
    /// Hours in the past
    pub offset_hours: i64,
    /// Price in USD
    pub price_usd: f64,
}

/// Balance token metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BalanceTokenMetadata {
    /// Logo URL
    pub logo: Option<String>,
    /// Token URL
    pub url: Option<String>,
}

/// Pool metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PoolMetadata {
    /// Pool type
    pub pool_type: Option<String>,
    /// Pool address
    pub address: Option<String>,
    /// Token 0
    pub token0: Option<String>,
    /// Token 1
    pub token1: Option<String>,
}

/// Query options for balances
#[derive(Debug, Clone, Default)]
pub struct BalancesOptions {
    /// Filter by chain IDs (comma-separated)
    pub chain_ids: Option<String>,
    /// Filter by token type (erc20, native)
    pub filters: Option<String>,
    /// Metadata to include (logo, url, pools)
    pub metadata: Option<String>,
    /// Exclude spam tokens
    pub exclude_spam_tokens: Option<bool>,
    /// Historical prices (hours in past, comma-separated)
    pub historical_prices: Option<String>,
    /// Pagination offset
    pub offset: Option<String>,
    /// Results limit (max 1000)
    pub limit: Option<u32>,
}

impl BalancesOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref chain_ids) = self.chain_ids {
            params.push(format!("chain_ids={}", chain_ids));
        }
        if let Some(ref filters) = self.filters {
            params.push(format!("filters={}", filters));
        }
        if let Some(ref metadata) = self.metadata {
            params.push(format!("metadata={}", metadata));
        }
        if let Some(exclude_spam) = self.exclude_spam_tokens {
            params.push(format!("exclude_spam_tokens={}", exclude_spam));
        }
        if let Some(ref historical) = self.historical_prices {
            params.push(format!("historical_prices={}", historical));
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

/// Query options for single token balance
#[derive(Debug, Clone, Default)]
pub struct SingleBalanceOptions {
    /// Chain IDs (required, comma-separated)
    pub chain_ids: String,
    /// Metadata to include (logo, url, pools)
    pub metadata: Option<String>,
    /// Historical prices (hours in past, comma-separated)
    pub historical_prices: Option<String>,
}

impl SingleBalanceOptions {
    pub fn new(chain_ids: &str) -> Self {
        Self {
            chain_ids: chain_ids.to_string(),
            ..Default::default()
        }
    }

    pub fn to_query_string(&self) -> String {
        let mut params = vec![format!("chain_ids={}", self.chain_ids)];
        if let Some(ref metadata) = self.metadata {
            params.push(format!("metadata={}", metadata));
        }
        if let Some(ref historical) = self.historical_prices {
            params.push(format!("historical_prices={}", historical));
        }
        format!("?{}", params.join("&"))
    }
}
