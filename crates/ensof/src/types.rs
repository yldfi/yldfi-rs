//! Types for the Enso Finance API
//!
//! This module contains request and response types for the Enso Finance API,
//! including routing, bundling, and position management.

use serde::{Deserialize, Serialize};

/// Supported chains for Enso Finance API
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
    /// Arbitrum One (chain ID: 42161)
    Arbitrum,
    /// Optimism (chain ID: 10)
    Optimism,
    /// Base (chain ID: 8453)
    Base,
    /// Gnosis Chain (chain ID: 100)
    Gnosis,
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
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Base => 8453,
            Self::Gnosis => 100,
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
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
            Self::Gnosis => "gnosis",
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
            42161 => Some(Self::Arbitrum),
            10 => Some(Self::Optimism),
            8453 => Some(Self::Base),
            100 => Some(Self::Gnosis),
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
            yldfi_common::Chain::Arbitrum => Ok(Self::Arbitrum),
            yldfi_common::Chain::Optimism => Ok(Self::Optimism),
            yldfi_common::Chain::Base => Ok(Self::Base),
            yldfi_common::Chain::Gnosis => Ok(Self::Gnosis),
            _ => Err("Chain not supported by Enso Finance API"),
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
            Chain::Arbitrum => Self::Arbitrum,
            Chain::Optimism => Self::Optimism,
            Chain::Base => Self::Base,
            Chain::Gnosis => Self::Gnosis,
        }
    }
}

/// Routing strategy for Enso
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RoutingStrategy {
    /// Use the router contract directly
    #[default]
    Router,
    /// Use delegate call through smart wallet
    Delegate,
    /// Use Enso smart wallet
    Ensowallet,
}

/// Route request parameters for getting swap routes
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    /// Chain ID
    pub chain_id: u64,
    /// Sender/from address
    pub from_address: String,
    /// Receiver address
    pub receiver: String,
    /// Input token addresses
    pub token_in: Vec<String>,
    /// Output token addresses
    pub token_out: Vec<String>,
    /// Input amounts (in smallest units)
    pub amount_in: Vec<String>,
    /// Slippage tolerance in basis points (as string, e.g., "100" for 1%)
    pub slippage: String,
    /// Routing strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_strategy: Option<RoutingStrategy>,
    /// Spender address (for approvals)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spender: Option<String>,
    /// Disable estimate (for faster quotes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_estimate: Option<bool>,
}

impl RouteRequest {
    /// Create a new route request for a simple swap
    #[must_use]
    pub fn new(
        chain_id: u64,
        from_address: impl Into<String>,
        token_in: impl Into<String>,
        token_out: impl Into<String>,
        amount_in: impl Into<String>,
        slippage_bps: u16,
    ) -> Self {
        let from = from_address.into();
        Self {
            chain_id,
            from_address: from.clone(),
            receiver: from,
            token_in: vec![token_in.into()],
            token_out: vec![token_out.into()],
            amount_in: vec![amount_in.into()],
            slippage: slippage_bps.to_string(),
            routing_strategy: None,
            spender: None,
            disable_estimate: None,
        }
    }

    /// Set a different receiver address
    #[must_use]
    pub fn with_receiver(mut self, receiver: impl Into<String>) -> Self {
        self.receiver = receiver.into();
        self
    }

    /// Set the routing strategy
    #[must_use]
    pub fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.routing_strategy = Some(strategy);
        self
    }

    /// Set the spender address
    #[must_use]
    pub fn with_spender(mut self, spender: impl Into<String>) -> Self {
        self.spender = Some(spender.into());
        self
    }

    /// Disable estimation for faster quotes
    #[must_use]
    pub fn with_disable_estimate(mut self, disable: bool) -> Self {
        self.disable_estimate = Some(disable);
        self
    }

    /// Add multiple input tokens (for multi-input swaps)
    #[must_use]
    pub fn with_tokens_in(mut self, tokens: Vec<String>, amounts: Vec<String>) -> Self {
        self.token_in = tokens;
        self.amount_in = amounts;
        self
    }

    /// Add multiple output tokens (for multi-output swaps)
    #[must_use]
    pub fn with_tokens_out(mut self, tokens: Vec<String>) -> Self {
        self.token_out = tokens;
        self
    }
}

/// Route response with transaction data
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    /// Estimated output amount
    pub amount_out: String,
    /// Minimum output amount after slippage
    #[serde(default, alias = "amount_out_min")]
    pub min_amount_out: Option<String>,
    /// Transaction data
    pub tx: TransactionData,
    /// Route steps
    #[serde(default)]
    pub route: Vec<RouteStep>,
    /// Estimated gas
    #[serde(default)]
    pub gas: Option<String>,
    /// Price impact percentage (can be 0 integer or 0.0 float)
    #[serde(default, deserialize_with = "deserialize_price_impact")]
    pub price_impact: Option<f64>,
    /// Created timestamp
    #[serde(default)]
    pub created_at: Option<u64>,
}

/// Deserialize `price_impact` that can be an integer (0) or float (0.0)
fn deserialize_price_impact<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};

    struct PriceImpactVisitor;

    impl Visitor<'_> for PriceImpactVisitor {
        type Value = Option<f64>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number or null")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v as f64))
        }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v as f64))
        }

        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(v))
        }
    }

    deserializer.deserialize_any(PriceImpactVisitor)
}

/// Transaction data for execution
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
    /// Target contract address
    pub to: String,
    /// Sender address
    pub from: String,
    /// Encoded calldata
    pub data: String,
    /// Value to send (for native token swaps)
    pub value: String,
}

/// Route step in the swap path
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteStep {
    /// Protocol/DEX name
    pub protocol: String,
    /// Action type
    #[serde(default)]
    pub action: Option<String>,
    /// Token in addresses (can be single or multiple)
    #[serde(default)]
    pub token_in: Option<Vec<String>>,
    /// Token out addresses (can be single or multiple)
    #[serde(default)]
    pub token_out: Option<Vec<String>>,
    /// Amount in
    #[serde(default)]
    pub amount_in: Option<String>,
    /// Amount out
    #[serde(default)]
    pub amount_out: Option<String>,
    /// Portion percentage
    #[serde(default)]
    pub portion: Option<u8>,
    /// Chain ID for the step
    #[serde(default)]
    pub chain_id: Option<u64>,
}

/// Bundle action for multi-step transactions
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleAction {
    /// Protocol to interact with
    pub protocol: String,
    /// Action type (swap, deposit, withdraw, etc.)
    pub action: String,
    /// Action arguments (protocol-specific)
    pub args: serde_json::Value,
}

impl BundleAction {
    /// Create a new bundle action
    #[must_use]
    pub fn new(
        protocol: impl Into<String>,
        action: impl Into<String>,
        args: serde_json::Value,
    ) -> Self {
        Self {
            protocol: protocol.into(),
            action: action.into(),
            args,
        }
    }

    /// Create a swap action
    #[must_use]
    pub fn swap(
        token_in: impl Into<String>,
        token_out: impl Into<String>,
        amount_in: impl Into<String>,
    ) -> Self {
        Self {
            protocol: "enso".to_string(),
            action: "route".to_string(),
            args: serde_json::json!({
                "tokenIn": token_in.into(),
                "tokenOut": token_out.into(),
                "amountIn": amount_in.into()
            }),
        }
    }
}

/// Bundle request for multi-action transactions
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleRequest {
    /// Chain ID
    pub chain_id: u64,
    /// Sender address
    pub from_address: String,
    /// Actions to bundle
    pub actions: Vec<BundleAction>,
    /// Routing strategy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub routing_strategy: Option<RoutingStrategy>,
}

impl BundleRequest {
    /// Create a new bundle request
    #[must_use]
    pub fn new(chain_id: u64, from_address: impl Into<String>, actions: Vec<BundleAction>) -> Self {
        Self {
            chain_id,
            from_address: from_address.into(),
            actions,
            routing_strategy: None,
        }
    }

    /// Set the routing strategy
    #[must_use]
    pub fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.routing_strategy = Some(strategy);
        self
    }
}

/// Bundle response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleResponse {
    /// Transaction data
    pub tx: TransactionData,
    /// Estimated gas
    #[serde(default)]
    pub gas: Option<String>,
    /// Bundle hash/ID
    #[serde(default)]
    pub bundle_hash: Option<String>,
}

/// Token price information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenPrice {
    /// Token address
    pub address: String,
    /// Price in USD
    pub price: f64,
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token decimals
    #[serde(default)]
    pub decimals: Option<u8>,
}

/// Token balance information
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    /// Token address
    pub address: String,
    /// Balance in smallest units
    pub balance: String,
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token decimals
    #[serde(default)]
    pub decimals: Option<u8>,
    /// USD value
    #[serde(default)]
    pub usd_value: Option<f64>,
}

/// API error response
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    /// Error message
    #[serde(alias = "message")]
    pub error: String,
    /// Error code
    #[serde(default)]
    pub code: Option<String>,
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
    fn test_route_request() {
        let request = RouteRequest::new(
            1,
            "0xSender",
            "0xTokenIn",
            "0xTokenOut",
            "1000000000000000000",
            100,
        );

        assert_eq!(request.chain_id, 1);
        assert_eq!(request.slippage, "100");
        assert_eq!(request.token_in.len(), 1);
    }

    #[test]
    fn test_bundle_action() {
        let action = BundleAction::swap("0xIn", "0xOut", "1000");
        assert_eq!(action.protocol, "enso");
        assert_eq!(action.action, "route");
    }
}
