//! Types for the OpenOcean API responses

use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Supported chains for OpenOcean
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum Chain {
    /// Ethereum mainnet
    Eth,
    /// BNB Chain (BSC)
    Bsc,
    /// Polygon
    Polygon,
    /// Fantom
    Fantom,
    /// Avalanche C-Chain
    Avax,
    /// Arbitrum One
    Arbitrum,
    /// Optimism
    Optimism,
    /// Gnosis Chain
    Gnosis,
    /// Base
    Base,
    /// Linea
    Linea,
    /// zkSync Era
    Zksync,
    /// Scroll
    Scroll,
    /// Mantle
    Mantle,
    /// Blast
    Blast,
    /// Solana
    Solana,
    /// Sui
    Sui,
}

impl Chain {
    /// Convert from EVM chain ID (returns None for non-EVM chains like Solana/Sui)
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            1 => Some(Chain::Eth),
            56 => Some(Chain::Bsc),
            137 => Some(Chain::Polygon),
            250 => Some(Chain::Fantom),
            43114 => Some(Chain::Avax),
            42161 => Some(Chain::Arbitrum),
            10 => Some(Chain::Optimism),
            100 => Some(Chain::Gnosis),
            8453 => Some(Chain::Base),
            59144 => Some(Chain::Linea),
            324 => Some(Chain::Zksync),
            534352 => Some(Chain::Scroll),
            5000 => Some(Chain::Mantle),
            81457 => Some(Chain::Blast),
            // Solana and Sui are not EVM chains, no chain ID mapping
            _ => None,
        }
    }

    /// Get the chain ID for API requests
    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Eth => "eth",
            Chain::Bsc => "bsc",
            Chain::Polygon => "polygon",
            Chain::Fantom => "fantom",
            Chain::Avax => "avax",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Gnosis => "gnosis",
            Chain::Base => "base",
            Chain::Linea => "linea",
            Chain::Zksync => "zksync",
            Chain::Scroll => "scroll",
            Chain::Mantle => "mantle",
            Chain::Blast => "blast",
            Chain::Solana => "solana",
            Chain::Sui => "sui",
        }
    }

    /// Parse chain from string (returns Option for backward compatibility)
    pub fn try_from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl FromStr for Chain {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "eth" | "ethereum" | "mainnet" => Ok(Chain::Eth),
            "bsc" | "bnb" | "binance" => Ok(Chain::Bsc),
            "polygon" | "matic" => Ok(Chain::Polygon),
            "fantom" | "ftm" => Ok(Chain::Fantom),
            "avax" | "avalanche" => Ok(Chain::Avax),
            "arbitrum" | "arb" => Ok(Chain::Arbitrum),
            "optimism" | "op" => Ok(Chain::Optimism),
            "gnosis" | "xdai" => Ok(Chain::Gnosis),
            "base" => Ok(Chain::Base),
            "linea" => Ok(Chain::Linea),
            "zksync" | "era" => Ok(Chain::Zksync),
            "scroll" => Ok(Chain::Scroll),
            "mantle" | "mnt" => Ok(Chain::Mantle),
            "blast" => Ok(Chain::Blast),
            "solana" | "sol" => Ok(Chain::Solana),
            "sui" => Ok(Chain::Sui),
            _ => Err(format!("Unknown chain: {}", s)),
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<yldfi_common::Chain> for Chain {
    type Error = &'static str;

    fn try_from(chain: yldfi_common::Chain) -> Result<Self, Self::Error> {
        match chain {
            yldfi_common::Chain::Ethereum => Ok(Self::Eth),
            yldfi_common::Chain::Bsc => Ok(Self::Bsc),
            yldfi_common::Chain::Polygon => Ok(Self::Polygon),
            yldfi_common::Chain::Fantom => Ok(Self::Fantom),
            yldfi_common::Chain::Avalanche => Ok(Self::Avax),
            yldfi_common::Chain::Arbitrum => Ok(Self::Arbitrum),
            yldfi_common::Chain::Optimism => Ok(Self::Optimism),
            yldfi_common::Chain::Gnosis => Ok(Self::Gnosis),
            yldfi_common::Chain::Base => Ok(Self::Base),
            yldfi_common::Chain::Linea => Ok(Self::Linea),
            yldfi_common::Chain::ZkSync => Ok(Self::Zksync),
            yldfi_common::Chain::Scroll => Ok(Self::Scroll),
            yldfi_common::Chain::Mantle => Ok(Self::Mantle),
            yldfi_common::Chain::Blast => Ok(Self::Blast),
            _ => Err("Chain not supported by OpenOcean"),
        }
    }
}

impl From<Chain> for yldfi_common::Chain {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Eth => Self::Ethereum,
            Chain::Bsc => Self::Bsc,
            Chain::Polygon => Self::Polygon,
            Chain::Fantom => Self::Fantom,
            Chain::Avax => Self::Avalanche,
            Chain::Arbitrum => Self::Arbitrum,
            Chain::Optimism => Self::Optimism,
            Chain::Gnosis => Self::Gnosis,
            Chain::Base => Self::Base,
            Chain::Linea => Self::Linea,
            Chain::Zksync => Self::ZkSync,
            Chain::Scroll => Self::Scroll,
            Chain::Mantle => Self::Mantle,
            Chain::Blast => Self::Blast,
            // Non-EVM chains fallback to custom ID
            Chain::Solana => Self::Other(501),
            Chain::Sui => Self::Other(784),
        }
    }
}

/// Quote request parameters
#[derive(Debug, Clone, Serialize)]
pub struct QuoteRequest {
    /// Input token address
    pub in_token_address: String,
    /// Output token address
    pub out_token_address: String,
    /// Amount with decimals (e.g., "1000000000000000000" for 1 ETH)
    pub amount: String,
    /// Slippage in percentage (e.g., 1 for 1%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    /// Gas price in Gwei with decimals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    /// Disabled DEX IDs (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_dex_ids: Option<String>,
}

impl QuoteRequest {
    /// Create a new quote request
    pub fn new(
        in_token: impl Into<String>,
        out_token: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        Self {
            in_token_address: in_token.into(),
            out_token_address: out_token.into(),
            amount: amount.into(),
            slippage: None,
            gas_price: None,
            disabled_dex_ids: None,
        }
    }

    /// Set slippage tolerance
    #[must_use]
    pub fn with_slippage(mut self, slippage: f64) -> Self {
        self.slippage = Some(slippage);
        self
    }

    /// Set gas price in Gwei
    #[must_use]
    pub fn with_gas_price(mut self, gas_price: impl Into<String>) -> Self {
        self.gas_price = Some(gas_price.into());
        self
    }

    /// Disable specific DEX IDs
    #[must_use]
    pub fn with_disabled_dexs(mut self, dex_ids: impl Into<String>) -> Self {
        self.disabled_dex_ids = Some(dex_ids.into());
        self
    }
}

/// Swap request parameters (includes quote params + user address)
#[derive(Debug, Clone, Serialize)]
pub struct SwapRequest {
    /// Input token address
    pub in_token_address: String,
    /// Output token address
    pub out_token_address: String,
    /// Amount with decimals
    pub amount: String,
    /// User's wallet address
    pub account: String,
    /// Slippage in percentage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    /// Gas price in Gwei with decimals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    /// Referrer address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
}

impl SwapRequest {
    /// Create a new swap request
    pub fn new(
        in_token: impl Into<String>,
        out_token: impl Into<String>,
        amount: impl Into<String>,
        account: impl Into<String>,
    ) -> Self {
        Self {
            in_token_address: in_token.into(),
            out_token_address: out_token.into(),
            amount: amount.into(),
            account: account.into(),
            slippage: None,
            gas_price: None,
            referrer: None,
        }
    }

    /// Set slippage tolerance
    #[must_use]
    pub fn with_slippage(mut self, slippage: f64) -> Self {
        self.slippage = Some(slippage);
        self
    }

    /// Set gas price
    #[must_use]
    pub fn with_gas_price(mut self, gas_price: impl Into<String>) -> Self {
        self.gas_price = Some(gas_price.into());
        self
    }

    /// Set referrer address
    #[must_use]
    pub fn with_referrer(mut self, referrer: impl Into<String>) -> Self {
        self.referrer = Some(referrer.into());
        self
    }
}

/// Quote response from OpenOcean API
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QuoteResponse {
    /// Response code (200 = success)
    pub code: i32,
    /// Response data
    pub data: Option<QuoteData>,
    /// Error message if any
    pub error: Option<String>,
}

/// Quote data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteData {
    /// Input token info
    pub in_token: TokenInfo,
    /// Output token info
    pub out_token: TokenInfo,
    /// Input amount (with decimals)
    pub in_amount: String,
    /// Output amount (with decimals)
    pub out_amount: String,
    /// Estimated gas
    pub estimated_gas: String,
    /// Minimum output after slippage
    #[serde(default)]
    pub min_out_amount: Option<String>,
    /// Price impact percentage
    #[serde(default)]
    pub price_impact: Option<String>,
    /// Route path
    #[serde(default)]
    pub path: Option<RoutePath>,
}

/// Swap response (includes transaction data)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SwapResponse {
    /// Response code (200 = success)
    pub code: i32,
    /// Response data
    pub data: Option<SwapData>,
    /// Error message if any
    pub error: Option<String>,
}

/// Swap data with transaction details
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapData {
    /// Input token info
    pub in_token: TokenInfo,
    /// Output token info
    pub out_token: TokenInfo,
    /// Input amount
    pub in_amount: String,
    /// Output amount
    pub out_amount: String,
    /// Minimum output after slippage
    pub min_out_amount: String,
    /// Estimated gas
    pub estimated_gas: String,
    /// Contract address to call
    pub to: String,
    /// Call data for the transaction
    pub data: String,
    /// ETH value to send
    pub value: String,
    /// Gas price used
    #[serde(default)]
    pub gas_price: Option<String>,
}

/// Token information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Token address
    pub address: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    #[serde(default)]
    pub name: Option<String>,
    /// Token decimals
    pub decimals: u8,
    /// USD price
    #[serde(default)]
    pub usd: Option<String>,
}

/// Routing path
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RoutePath {
    /// Route routes
    #[serde(default)]
    pub routes: Vec<RouteSegment>,
}

/// Route segment in the swap path
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSegment {
    /// DEX name
    #[serde(default)]
    pub dex_name: Option<String>,
    /// Percentage of total amount
    #[serde(default)]
    pub percentage: Option<f64>,
    /// Sub-routes
    #[serde(default)]
    pub sub_routes: Vec<SubRoute>,
}

/// Sub-route within a segment
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubRoute {
    /// From token
    #[serde(default)]
    pub from: Option<String>,
    /// To token
    #[serde(default)]
    pub to: Option<String>,
    /// DEX name
    #[serde(default)]
    pub dex_name: Option<String>,
}

/// DEX information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DexInfo {
    /// DEX ID
    pub id: String,
    /// DEX name
    pub name: String,
    /// Whether enabled
    pub enabled: bool,
}

/// Token list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenListResponse {
    /// Response code
    pub code: i32,
    /// Token list
    pub data: Option<Vec<TokenInfo>>,
}

/// DEX list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DexListResponse {
    /// Response code
    pub code: i32,
    /// DEX list
    pub data: Option<Vec<DexInfo>>,
}
