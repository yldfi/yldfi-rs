//! Types for the Velora (ParaSwap) API
//!
//! This module contains request and response types for the ParaSwap API,
//! including price routing, transaction building, and token lists.

use serde::{Deserialize, Serialize};

/// Supported chains for Velora/ParaSwap API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Chain {
    /// Ethereum mainnet (chain ID: 1)
    Ethereum,
    /// Polygon (chain ID: 137)
    Polygon,
    /// BNB Chain (chain ID: 56)
    Bsc,
    /// Avalanche C-Chain (chain ID: 43114)
    Avalanche,
    /// Fantom (chain ID: 250)
    Fantom,
    /// Arbitrum One (chain ID: 42161)
    Arbitrum,
    /// Optimism (chain ID: 10)
    Optimism,
    /// Base (chain ID: 8453)
    Base,
    /// zkSync Era (chain ID: 324)
    ZkSync,
    /// Polygon zkEVM (chain ID: 1101)
    PolygonZkEvm,
}

impl Chain {
    /// Get the chain ID
    #[must_use]
    pub const fn chain_id(&self) -> u64 {
        match self {
            Self::Ethereum => 1,
            Self::Polygon => 137,
            Self::Bsc => 56,
            Self::Avalanche => 43114,
            Self::Fantom => 250,
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Base => 8453,
            Self::ZkSync => 324,
            Self::PolygonZkEvm => 1101,
        }
    }

    /// Get the chain name as used in the API
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Bsc => "bsc",
            Self::Avalanche => "avalanche",
            Self::Fantom => "fantom",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
            Self::ZkSync => "zksync",
            Self::PolygonZkEvm => "polygon-zkevm",
        }
    }

    /// Parse chain from chain ID
    #[must_use]
    pub const fn from_chain_id(id: u64) -> Option<Self> {
        match id {
            1 => Some(Self::Ethereum),
            137 => Some(Self::Polygon),
            56 => Some(Self::Bsc),
            43114 => Some(Self::Avalanche),
            250 => Some(Self::Fantom),
            42161 => Some(Self::Arbitrum),
            10 => Some(Self::Optimism),
            8453 => Some(Self::Base),
            324 => Some(Self::ZkSync),
            1101 => Some(Self::PolygonZkEvm),
            _ => None,
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
            yldfi_common::Chain::Ethereum => Ok(Self::Ethereum),
            yldfi_common::Chain::Polygon => Ok(Self::Polygon),
            yldfi_common::Chain::Bsc => Ok(Self::Bsc),
            yldfi_common::Chain::Avalanche => Ok(Self::Avalanche),
            yldfi_common::Chain::Fantom => Ok(Self::Fantom),
            yldfi_common::Chain::Arbitrum => Ok(Self::Arbitrum),
            yldfi_common::Chain::Optimism => Ok(Self::Optimism),
            yldfi_common::Chain::Base => Ok(Self::Base),
            yldfi_common::Chain::ZkSync => Ok(Self::ZkSync),
            yldfi_common::Chain::PolygonZkEvm => Ok(Self::PolygonZkEvm),
            _ => Err("Chain not supported by Velora/ParaSwap API"),
        }
    }
}

impl From<Chain> for yldfi_common::Chain {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Ethereum => Self::Ethereum,
            Chain::Polygon => Self::Polygon,
            Chain::Bsc => Self::Bsc,
            Chain::Avalanche => Self::Avalanche,
            Chain::Fantom => Self::Fantom,
            Chain::Arbitrum => Self::Arbitrum,
            Chain::Optimism => Self::Optimism,
            Chain::Base => Self::Base,
            Chain::ZkSync => Self::ZkSync,
            Chain::PolygonZkEvm => Self::PolygonZkEvm,
        }
    }
}

/// Side of the swap (SELL or BUY)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Side {
    /// Sell a specific amount of source token
    #[default]
    Sell,
    /// Buy a specific amount of destination token
    Buy,
}

/// Price request parameters for getting swap quotes
#[derive(Debug, Clone, Default)]
pub struct PriceRequest {
    /// Address of the source token
    pub src_token: String,
    /// Address of the destination token
    pub dest_token: String,
    /// Amount to swap (in source token's smallest unit)
    pub amount: String,
    /// Side of the swap (SELL or BUY)
    pub side: Side,
    /// Decimals of source token (optional, improves accuracy)
    pub src_decimals: Option<u8>,
    /// Decimals of destination token (optional, improves accuracy)
    pub dest_decimals: Option<u8>,
    /// User address (optional, enables more accurate routing)
    pub user_address: Option<String>,
    /// Partner address for referral fees
    pub partner: Option<String>,
    /// Exclude specific DEXs from the route
    pub exclude_dexs: Option<String>,
    /// Include only specific DEXs in the route
    pub include_dexs: Option<String>,
    /// Exclude pools with low TVL
    pub exclude_pools_with_low_tvl: Option<bool>,
}

impl PriceRequest {
    /// Create a new price request for selling tokens
    #[must_use]
    pub fn sell(
        src_token: impl Into<String>,
        dest_token: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        Self {
            src_token: src_token.into(),
            dest_token: dest_token.into(),
            amount: amount.into(),
            side: Side::Sell,
            ..Default::default()
        }
    }

    /// Create a new price request for buying tokens
    #[must_use]
    pub fn buy(
        src_token: impl Into<String>,
        dest_token: impl Into<String>,
        amount: impl Into<String>,
    ) -> Self {
        Self {
            src_token: src_token.into(),
            dest_token: dest_token.into(),
            amount: amount.into(),
            side: Side::Buy,
            ..Default::default()
        }
    }

    /// Set source token decimals
    #[must_use]
    pub fn with_src_decimals(mut self, decimals: u8) -> Self {
        self.src_decimals = Some(decimals);
        self
    }

    /// Set destination token decimals
    #[must_use]
    pub fn with_dest_decimals(mut self, decimals: u8) -> Self {
        self.dest_decimals = Some(decimals);
        self
    }

    /// Set user address
    #[must_use]
    pub fn with_user_address(mut self, address: impl Into<String>) -> Self {
        self.user_address = Some(address.into());
        self
    }

    /// Set partner address for referral
    #[must_use]
    pub fn with_partner(mut self, partner: impl Into<String>) -> Self {
        self.partner = Some(partner.into());
        self
    }

    /// Exclude specific DEXs
    #[must_use]
    pub fn with_exclude_dexs(mut self, dexs: impl Into<String>) -> Self {
        self.exclude_dexs = Some(dexs.into());
        self
    }

    /// Convert to query parameters
    pub fn to_query_params(&self, network: u64) -> Vec<(String, String)> {
        let mut params = vec![
            ("srcToken".to_string(), self.src_token.clone()),
            ("destToken".to_string(), self.dest_token.clone()),
            ("amount".to_string(), self.amount.clone()),
            (
                "side".to_string(),
                match self.side {
                    Side::Sell => "SELL".to_string(),
                    Side::Buy => "BUY".to_string(),
                },
            ),
            ("network".to_string(), network.to_string()),
        ];

        if let Some(decimals) = self.src_decimals {
            params.push(("srcDecimals".to_string(), decimals.to_string()));
        }
        if let Some(decimals) = self.dest_decimals {
            params.push(("destDecimals".to_string(), decimals.to_string()));
        }
        if let Some(ref addr) = self.user_address {
            params.push(("userAddress".to_string(), addr.clone()));
        }
        if let Some(ref partner) = self.partner {
            params.push(("partner".to_string(), partner.clone()));
        }
        if let Some(ref dexs) = self.exclude_dexs {
            params.push(("excludeDEXS".to_string(), dexs.clone()));
        }
        if let Some(ref dexs) = self.include_dexs {
            params.push(("includeDEXS".to_string(), dexs.clone()));
        }
        if let Some(exclude) = self.exclude_pools_with_low_tvl {
            params.push(("excludePoolsWithLowTVL".to_string(), exclude.to_string()));
        }

        params
    }
}

/// Price response with routing information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    /// Price route containing all swap details
    pub price_route: PriceRoute,
}

/// Detailed price route information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceRoute {
    /// Block number when the quote was generated
    pub block_number: u64,
    /// Network/chain ID
    pub network: u64,
    /// Source token address
    pub src_token: String,
    /// Source token decimals
    pub src_decimals: u8,
    /// Source amount in smallest units
    pub src_amount: String,
    /// Destination token address
    pub dest_token: String,
    /// Destination token decimals
    pub dest_decimals: u8,
    /// Destination amount in smallest units
    pub dest_amount: String,
    /// Best route details
    pub best_route: Vec<Route>,
    /// Token transfer proxy address (for approvals)
    pub token_transfer_proxy: String,
    /// Contract address for the swap
    pub contract_address: String,
    /// Contract method to call
    pub contract_method: String,
    /// Partner fee percentage (if applicable)
    #[serde(default)]
    pub partner_fee: f64,
    /// Estimated gas cost
    pub gas_cost: Option<String>,
    /// Gas cost in USD
    pub gas_cost_usd: Option<String>,
    /// Side of the swap
    pub side: String,
    /// Source token USD value
    pub src_usd: Option<String>,
    /// Destination token USD value
    pub dest_usd: Option<String>,
    /// Max impact percentage
    pub max_impact_reached: Option<bool>,
    /// Price impact percentage
    #[serde(default)]
    pub price_impact: Option<String>,
}

/// Route segment in a multi-hop swap
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    /// Percentage of the swap going through this route
    pub percent: f64,
    /// Swap steps in this route
    pub swaps: Vec<Swap>,
}

/// Individual swap step within a route
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Swap {
    /// Source token address
    pub src_token: String,
    /// Source token decimals
    pub src_decimals: u8,
    /// Destination token address
    pub dest_token: String,
    /// Destination token decimals
    pub dest_decimals: u8,
    /// Pool addresses used in this swap
    pub swap_exchanges: Vec<SwapExchange>,
}

/// Exchange details for a swap step
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapExchange {
    /// Exchange/DEX name
    pub exchange: String,
    /// Source amount
    pub src_amount: String,
    /// Destination amount
    pub dest_amount: String,
    /// Percentage of the swap
    pub percent: f64,
    /// Pool addresses
    #[serde(default)]
    pub pool_addresses: Vec<String>,
    /// Additional exchange data
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

/// Transaction build request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// Source token address
    pub src_token: String,
    /// Destination token address
    pub dest_token: String,
    /// Source amount
    pub src_amount: String,
    /// Destination amount (from price route)
    pub dest_amount: String,
    /// Price route from PriceResponse
    pub price_route: serde_json::Value,
    /// Slippage tolerance in basis points (e.g., 100 = 1%)
    pub slippage: u32,
    /// User address executing the swap
    pub user_address: String,
    /// Partner address for referral
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner: Option<String>,
    /// Receiver address (if different from user)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Deadline timestamp (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
    /// Permit data for gasless approvals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permit: Option<String>,
    /// Ignore gas estimation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_gas: Option<bool>,
    /// Ignore balance and allowance checks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ignore_checks: Option<bool>,
}

impl TransactionRequest {
    /// Create a new transaction request from a price route
    #[must_use]
    pub fn new(price_route: &PriceRoute, user_address: impl Into<String>, slippage: u32) -> Self {
        Self {
            src_token: price_route.src_token.clone(),
            dest_token: price_route.dest_token.clone(),
            src_amount: price_route.src_amount.clone(),
            dest_amount: price_route.dest_amount.clone(),
            price_route: serde_json::to_value(price_route).unwrap_or_default(),
            slippage,
            user_address: user_address.into(),
            partner: None,
            receiver: None,
            deadline: None,
            permit: None,
            ignore_gas: None,
            ignore_checks: None,
        }
    }

    /// Set receiver address
    #[must_use]
    pub fn with_receiver(mut self, receiver: impl Into<String>) -> Self {
        self.receiver = Some(receiver.into());
        self
    }

    /// Set partner address
    #[must_use]
    pub fn with_partner(mut self, partner: impl Into<String>) -> Self {
        self.partner = Some(partner.into());
        self
    }

    /// Set deadline timestamp
    #[must_use]
    pub fn with_deadline(mut self, deadline: impl Into<String>) -> Self {
        self.deadline = Some(deadline.into());
        self
    }

    /// Ignore gas estimation
    #[must_use]
    pub fn with_ignore_gas(mut self, ignore: bool) -> Self {
        self.ignore_gas = Some(ignore);
        self
    }

    /// Ignore balance and allowance checks
    #[must_use]
    pub fn with_ignore_checks(mut self, ignore: bool) -> Self {
        self.ignore_checks = Some(ignore);
        self
    }
}

/// Transaction response ready for signing
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionResponse {
    /// Sender address
    pub from: String,
    /// Router contract address
    pub to: String,
    /// Chain ID
    pub chain_id: u64,
    /// Value to send (for native token swaps)
    pub value: String,
    /// Encoded transaction data
    pub data: String,
    /// Suggested gas price
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Estimated gas limit
    #[serde(default)]
    pub gas: Option<String>,
}

/// Token information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// Token contract address
    pub address: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: String,
    /// Token decimals
    pub decimals: u8,
    /// Token logo URL
    #[serde(default)]
    pub img: Option<String>,
    /// Is native token
    #[serde(default)]
    pub is_native: Option<bool>,
}

/// Token list response
#[derive(Debug, Clone, Deserialize)]
pub struct TokenListResponse {
    /// List of tokens
    pub tokens: Vec<Token>,
}

/// API error response
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    /// Error message
    pub error: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id() {
        assert_eq!(Chain::Ethereum.chain_id(), 1);
        assert_eq!(Chain::Polygon.chain_id(), 137);
        assert_eq!(Chain::Arbitrum.chain_id(), 42161);
    }

    #[test]
    fn test_chain_from_id() {
        assert_eq!(Chain::from_chain_id(1), Some(Chain::Ethereum));
        assert_eq!(Chain::from_chain_id(137), Some(Chain::Polygon));
        assert_eq!(Chain::from_chain_id(999999), None);
    }

    #[test]
    fn test_price_request() {
        let request = PriceRequest::sell(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
        )
        .with_src_decimals(18)
        .with_dest_decimals(6);

        assert_eq!(request.side, Side::Sell);
        assert_eq!(request.src_decimals, Some(18));
        assert_eq!(request.dest_decimals, Some(6));
    }
}
