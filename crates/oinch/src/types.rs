//! Types for the 1inch Swap API v6.0
//!
//! This module contains request and response types for interacting with
//! the 1inch DEX aggregator API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Supported chains for 1inch API
///
/// The 1inch API uses numeric chain IDs in the URL path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Chain {
    /// Ethereum mainnet (Chain ID: 1)
    Ethereum = 1,
    /// BNB Smart Chain (Chain ID: 56)
    Bsc = 56,
    /// Polygon (Chain ID: 137)
    Polygon = 137,
    /// Optimism (Chain ID: 10)
    Optimism = 10,
    /// Arbitrum One (Chain ID: 42161)
    Arbitrum = 42161,
    /// Gnosis Chain (Chain ID: 100)
    Gnosis = 100,
    /// Avalanche C-Chain (Chain ID: 43114)
    Avalanche = 43114,
    /// Fantom Opera (Chain ID: 250)
    Fantom = 250,
    /// Klaytn (Chain ID: 8217)
    Klaytn = 8217,
    /// Aurora (Chain ID: 1313161554)
    Aurora = 1_313_161_554,
    /// zkSync Era (Chain ID: 324)
    ZkSync = 324,
    /// Base (Chain ID: 8453)
    Base = 8453,
    /// Linea (Chain ID: 59144)
    Linea = 59144,
}

impl Chain {
    /// Get the numeric chain ID for API requests
    #[must_use]
    pub const fn chain_id(self) -> u64 {
        self as u64
    }

    /// Create a Chain from a numeric chain ID
    #[must_use]
    pub const fn from_chain_id(id: u64) -> Option<Self> {
        match id {
            1 => Some(Self::Ethereum),
            56 => Some(Self::Bsc),
            137 => Some(Self::Polygon),
            10 => Some(Self::Optimism),
            42161 => Some(Self::Arbitrum),
            100 => Some(Self::Gnosis),
            43114 => Some(Self::Avalanche),
            250 => Some(Self::Fantom),
            8217 => Some(Self::Klaytn),
            1_313_161_554 => Some(Self::Aurora),
            324 => Some(Self::ZkSync),
            8453 => Some(Self::Base),
            59144 => Some(Self::Linea),
            _ => None,
        }
    }

    /// Get the chain name as a string
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Ethereum => "Ethereum",
            Self::Bsc => "BNB Smart Chain",
            Self::Polygon => "Polygon",
            Self::Optimism => "Optimism",
            Self::Arbitrum => "Arbitrum One",
            Self::Gnosis => "Gnosis Chain",
            Self::Avalanche => "Avalanche",
            Self::Fantom => "Fantom",
            Self::Klaytn => "Klaytn",
            Self::Aurora => "Aurora",
            Self::ZkSync => "zkSync Era",
            Self::Base => "Base",
            Self::Linea => "Linea",
        }
    }

    /// Parse chain from a string (name or chain ID)
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        // Try parsing as chain ID first
        if let Ok(id) = s.parse::<u64>() {
            return Self::from_chain_id(id);
        }

        // Try parsing as name
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Some(Self::Ethereum),
            "bsc" | "bnb" | "binance" => Some(Self::Bsc),
            "polygon" | "matic" => Some(Self::Polygon),
            "optimism" | "op" => Some(Self::Optimism),
            "arbitrum" | "arb" => Some(Self::Arbitrum),
            "gnosis" | "xdai" => Some(Self::Gnosis),
            "avalanche" | "avax" => Some(Self::Avalanche),
            "fantom" | "ftm" => Some(Self::Fantom),
            "klaytn" | "klay" => Some(Self::Klaytn),
            "aurora" => Some(Self::Aurora),
            "zksync" | "era" => Some(Self::ZkSync),
            "base" => Some(Self::Base),
            "linea" => Some(Self::Linea),
            _ => None,
        }
    }
}

/// Error type for chain parsing
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseChainError(String);

impl fmt::Display for ParseChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown chain: {}", self.0)
    }
}

impl std::error::Error for ParseChainError {}

impl std::str::FromStr for Chain {
    type Err = ParseChainError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Self::parse(s).ok_or_else(|| ParseChainError(s.to_string()))
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.chain_id())
    }
}

impl TryFrom<yldfi_common::Chain> for Chain {
    type Error = &'static str;

    fn try_from(chain: yldfi_common::Chain) -> Result<Self, Self::Error> {
        match chain {
            yldfi_common::Chain::Ethereum => Ok(Self::Ethereum),
            yldfi_common::Chain::Bsc => Ok(Self::Bsc),
            yldfi_common::Chain::Polygon => Ok(Self::Polygon),
            yldfi_common::Chain::Optimism => Ok(Self::Optimism),
            yldfi_common::Chain::Arbitrum => Ok(Self::Arbitrum),
            yldfi_common::Chain::Gnosis => Ok(Self::Gnosis),
            yldfi_common::Chain::Avalanche => Ok(Self::Avalanche),
            yldfi_common::Chain::Fantom => Ok(Self::Fantom),
            yldfi_common::Chain::Klaytn => Ok(Self::Klaytn),
            yldfi_common::Chain::Aurora => Ok(Self::Aurora),
            yldfi_common::Chain::ZkSync => Ok(Self::ZkSync),
            yldfi_common::Chain::Base => Ok(Self::Base),
            yldfi_common::Chain::Linea => Ok(Self::Linea),
            _ => Err("Chain not supported by 1inch API"),
        }
    }
}

impl From<Chain> for yldfi_common::Chain {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Ethereum => Self::Ethereum,
            Chain::Bsc => Self::Bsc,
            Chain::Polygon => Self::Polygon,
            Chain::Optimism => Self::Optimism,
            Chain::Arbitrum => Self::Arbitrum,
            Chain::Gnosis => Self::Gnosis,
            Chain::Avalanche => Self::Avalanche,
            Chain::Fantom => Self::Fantom,
            Chain::Klaytn => Self::Klaytn,
            Chain::Aurora => Self::Aurora,
            Chain::ZkSync => Self::ZkSync,
            Chain::Base => Self::Base,
            Chain::Linea => Self::Linea,
        }
    }
}

/// Token information from 1inch API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Token contract address
    pub address: String,
    /// Token symbol (e.g., "ETH", "USDC")
    pub symbol: String,
    /// Token name (e.g., "Ethereum", "USD Coin")
    pub name: String,
    /// Number of decimals
    pub decimals: u8,
    /// Logo URI (optional)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    /// Whether token is native (e.g., ETH on Ethereum)
    #[serde(default)]
    pub is_native: bool,
    /// Tags associated with the token
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Token list response from /tokens endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenListResponse {
    /// Map of token address to token info
    pub tokens: HashMap<String, TokenInfo>,
}

/// Quote request parameters for GET /quote
#[derive(Debug, Clone, Default)]
pub struct QuoteRequest {
    /// Source token address (required)
    pub src: String,
    /// Destination token address (required)
    pub dst: String,
    /// Amount in minimal divisible units (required)
    pub amount: String,
    /// Limit maximum number of parts for routing (1-50)
    pub protocols: Option<String>,
    /// Partner fee percentage (0-3%)
    pub fee: Option<f64>,
    /// Gas price in wei
    pub gas_price: Option<String>,
    /// Token addresses to use as connectors
    pub connector_tokens: Option<String>,
    /// Maximum complexity level (0-3)
    pub complexity_level: Option<u8>,
    /// Include tokens info in response
    pub include_tokens_info: Option<bool>,
    /// Include protocols info in response
    pub include_protocols: Option<bool>,
    /// Include gas estimation in response
    pub include_gas: Option<bool>,
}

impl QuoteRequest {
    /// Create a new quote request with required parameters
    #[must_use]
    pub fn new(src: impl Into<String>, dst: impl Into<String>, amount: impl Into<String>) -> Self {
        Self {
            src: src.into(),
            dst: dst.into(),
            amount: amount.into(),
            ..Default::default()
        }
    }

    /// Set the protocols to use (comma-separated list)
    #[must_use]
    pub fn with_protocols(mut self, protocols: impl Into<String>) -> Self {
        self.protocols = Some(protocols.into());
        self
    }

    /// Set partner fee percentage (0-3%)
    #[must_use]
    pub fn with_fee(mut self, fee: f64) -> Self {
        self.fee = Some(fee);
        self
    }

    /// Set gas price in wei
    #[must_use]
    pub fn with_gas_price(mut self, gas_price: impl Into<String>) -> Self {
        self.gas_price = Some(gas_price.into());
        self
    }

    /// Set connector tokens (comma-separated addresses)
    #[must_use]
    pub fn with_connector_tokens(mut self, tokens: impl Into<String>) -> Self {
        self.connector_tokens = Some(tokens.into());
        self
    }

    /// Set complexity level (0=fastest, 3=most optimal)
    #[must_use]
    pub fn with_complexity_level(mut self, level: u8) -> Self {
        self.complexity_level = Some(level.min(3));
        self
    }

    /// Include tokens info in response
    #[must_use]
    pub fn with_tokens_info(mut self) -> Self {
        self.include_tokens_info = Some(true);
        self
    }

    /// Include protocols/routing info in response
    #[must_use]
    pub fn with_protocols_info(mut self) -> Self {
        self.include_protocols = Some(true);
        self
    }

    /// Include gas estimation in response
    #[must_use]
    pub fn with_gas_info(mut self) -> Self {
        self.include_gas = Some(true);
        self
    }

    /// Convert to query parameters for the API request
    #[must_use]
    pub fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("src", self.src.clone()),
            ("dst", self.dst.clone()),
            ("amount", self.amount.clone()),
        ];

        if let Some(ref protocols) = self.protocols {
            params.push(("protocols", protocols.clone()));
        }
        if let Some(fee) = self.fee {
            params.push(("fee", fee.to_string()));
        }
        if let Some(ref gas_price) = self.gas_price {
            params.push(("gasPrice", gas_price.clone()));
        }
        if let Some(ref connectors) = self.connector_tokens {
            params.push(("connectorTokens", connectors.clone()));
        }
        if let Some(level) = self.complexity_level {
            params.push(("complexityLevel", level.to_string()));
        }
        if self.include_tokens_info == Some(true) {
            params.push(("includeTokensInfo", "true".to_string()));
        }
        if self.include_protocols == Some(true) {
            params.push(("includeProtocols", "true".to_string()));
        }
        if self.include_gas == Some(true) {
            params.push(("includeGas", "true".to_string()));
        }

        params
    }
}

/// Quote response from the 1inch API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    /// Source token address
    pub src_token: Option<TokenInfo>,
    /// Destination token address
    pub dst_token: Option<TokenInfo>,
    /// Amount of source tokens (in minimal units)
    #[serde(default)]
    pub from_amount: Option<String>,
    /// Amount of destination tokens (in minimal units)
    #[serde(alias = "dstAmount")]
    pub to_amount: String,
    /// Estimated gas for the swap
    #[serde(default)]
    pub gas: Option<u64>,
    /// Routing protocols used
    #[serde(default)]
    pub protocols: Option<Vec<Vec<Vec<ProtocolInfo>>>>,
}

/// Protocol/DEX information in routing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolInfo {
    /// Protocol name (e.g., "`UNISWAP_V3`")
    pub name: String,
    /// Percentage of the swap going through this protocol (0-100)
    pub part: f64,
    /// Source token address for this leg
    pub from_token_address: String,
    /// Destination token address for this leg
    pub to_token_address: String,
}

/// Swap request parameters for GET /swap
///
/// Extends `QuoteRequest` with additional parameters needed for swap execution.
#[derive(Debug, Clone, Default)]
pub struct SwapRequest {
    /// Source token address (required)
    pub src: String,
    /// Destination token address (required)
    pub dst: String,
    /// Amount in minimal divisible units (required)
    pub amount: String,
    /// Address that will execute the swap (required)
    pub from: String,
    /// Slippage tolerance as percentage (required, e.g., 1 for 1%)
    pub slippage: f64,
    /// Protocols to use (comma-separated)
    pub protocols: Option<String>,
    /// Partner fee percentage (0-3%)
    pub fee: Option<f64>,
    /// Gas price in wei
    pub gas_price: Option<String>,
    /// Token addresses to use as connectors
    pub connector_tokens: Option<String>,
    /// Complexity level (0-3)
    pub complexity_level: Option<u8>,
    /// Address to receive destination tokens (if different from `from`)
    pub dest_receiver: Option<String>,
    /// Referrer address for partner rewards
    pub referrer: Option<String>,
    /// Disable gas estimation (default: false)
    pub disable_estimate: Option<bool>,
    /// Allow partial fill (default: false)
    pub allow_partial_fill: Option<bool>,
    /// Use permit signature for token approval
    pub permit: Option<String>,
    /// Include tokens info in response
    pub include_tokens_info: Option<bool>,
    /// Include protocols info in response
    pub include_protocols: Option<bool>,
    /// Include gas estimation in response
    pub include_gas: Option<bool>,
}

impl SwapRequest {
    /// Create a new swap request with required parameters
    #[must_use]
    pub fn new(
        src: impl Into<String>,
        dst: impl Into<String>,
        amount: impl Into<String>,
        from: impl Into<String>,
        slippage: f64,
    ) -> Self {
        Self {
            src: src.into(),
            dst: dst.into(),
            amount: amount.into(),
            from: from.into(),
            slippage,
            ..Default::default()
        }
    }

    /// Create from a `QuoteRequest` with additional swap parameters
    #[must_use]
    pub fn from_quote(quote: QuoteRequest, from: impl Into<String>, slippage: f64) -> Self {
        Self {
            src: quote.src,
            dst: quote.dst,
            amount: quote.amount,
            from: from.into(),
            slippage,
            protocols: quote.protocols,
            fee: quote.fee,
            gas_price: quote.gas_price,
            connector_tokens: quote.connector_tokens,
            complexity_level: quote.complexity_level,
            include_tokens_info: quote.include_tokens_info,
            include_protocols: quote.include_protocols,
            include_gas: quote.include_gas,
            ..Default::default()
        }
    }

    /// Set a different destination receiver address
    #[must_use]
    pub fn with_dest_receiver(mut self, receiver: impl Into<String>) -> Self {
        self.dest_receiver = Some(receiver.into());
        self
    }

    /// Set referrer address for partner rewards
    #[must_use]
    pub fn with_referrer(mut self, referrer: impl Into<String>) -> Self {
        self.referrer = Some(referrer.into());
        self
    }

    /// Disable gas estimation for the returned transaction
    #[must_use]
    pub fn with_estimate_disabled(mut self) -> Self {
        self.disable_estimate = Some(true);
        self
    }

    /// Allow partial fill of the swap
    #[must_use]
    pub fn with_partial_fill(mut self) -> Self {
        self.allow_partial_fill = Some(true);
        self
    }

    /// Set permit signature for gasless token approval
    #[must_use]
    pub fn with_permit(mut self, permit: impl Into<String>) -> Self {
        self.permit = Some(permit.into());
        self
    }

    /// Set the protocols to use (comma-separated list)
    #[must_use]
    pub fn with_protocols(mut self, protocols: impl Into<String>) -> Self {
        self.protocols = Some(protocols.into());
        self
    }

    /// Set partner fee percentage (0-3%)
    #[must_use]
    pub fn with_fee(mut self, fee: f64) -> Self {
        self.fee = Some(fee);
        self
    }

    /// Set gas price in wei
    #[must_use]
    pub fn with_gas_price(mut self, gas_price: impl Into<String>) -> Self {
        self.gas_price = Some(gas_price.into());
        self
    }

    /// Set connector tokens (comma-separated addresses)
    #[must_use]
    pub fn with_connector_tokens(mut self, tokens: impl Into<String>) -> Self {
        self.connector_tokens = Some(tokens.into());
        self
    }

    /// Set complexity level (0=fastest, 3=most optimal)
    #[must_use]
    pub fn with_complexity_level(mut self, level: u8) -> Self {
        self.complexity_level = Some(level.min(3));
        self
    }

    /// Include tokens info in response
    #[must_use]
    pub fn with_tokens_info(mut self) -> Self {
        self.include_tokens_info = Some(true);
        self
    }

    /// Include protocols/routing info in response
    #[must_use]
    pub fn with_protocols_info(mut self) -> Self {
        self.include_protocols = Some(true);
        self
    }

    /// Include gas estimation in response
    #[must_use]
    pub fn with_gas_info(mut self) -> Self {
        self.include_gas = Some(true);
        self
    }

    /// Convert to query parameters for the API request
    #[must_use]
    pub fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("src", self.src.clone()),
            ("dst", self.dst.clone()),
            ("amount", self.amount.clone()),
            ("from", self.from.clone()),
            ("slippage", self.slippage.to_string()),
        ];

        if let Some(ref protocols) = self.protocols {
            params.push(("protocols", protocols.clone()));
        }
        if let Some(fee) = self.fee {
            params.push(("fee", fee.to_string()));
        }
        if let Some(ref gas_price) = self.gas_price {
            params.push(("gasPrice", gas_price.clone()));
        }
        if let Some(ref connectors) = self.connector_tokens {
            params.push(("connectorTokens", connectors.clone()));
        }
        if let Some(level) = self.complexity_level {
            params.push(("complexityLevel", level.to_string()));
        }
        if let Some(ref receiver) = self.dest_receiver {
            params.push(("receiver", receiver.clone()));
        }
        if let Some(ref referrer) = self.referrer {
            params.push(("referrer", referrer.clone()));
        }
        if self.disable_estimate == Some(true) {
            params.push(("disableEstimate", "true".to_string()));
        }
        if self.allow_partial_fill == Some(true) {
            params.push(("allowPartialFill", "true".to_string()));
        }
        if let Some(ref permit) = self.permit {
            params.push(("permit", permit.clone()));
        }
        if self.include_tokens_info == Some(true) {
            params.push(("includeTokensInfo", "true".to_string()));
        }
        if self.include_protocols == Some(true) {
            params.push(("includeProtocols", "true".to_string()));
        }
        if self.include_gas == Some(true) {
            params.push(("includeGas", "true".to_string()));
        }

        params
    }
}

/// Transaction data for executing a swap
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
    /// Address to send the transaction to (1inch router)
    pub to: String,
    /// Encoded transaction data (calldata)
    pub data: String,
    /// ETH value to send with transaction (in wei)
    pub value: String,
    /// Estimated gas for the transaction
    #[serde(default)]
    pub gas: Option<u64>,
    /// Gas price in wei
    #[serde(default)]
    pub gas_price: Option<String>,
}

/// Swap response from the 1inch API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapResponse {
    /// Source token info
    pub src_token: Option<TokenInfo>,
    /// Destination token info
    pub dst_token: Option<TokenInfo>,
    /// Amount of source tokens (in minimal units)
    #[serde(default)]
    pub from_amount: Option<String>,
    /// Amount of destination tokens (in minimal units)
    #[serde(alias = "dstAmount")]
    pub to_amount: String,
    /// Transaction data for executing the swap
    pub tx: TransactionData,
    /// Routing protocols used
    #[serde(default)]
    pub protocols: Option<Vec<Vec<Vec<ProtocolInfo>>>>,
}

/// Liquidity source (protocol) information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquiditySource {
    /// Protocol ID
    pub id: String,
    /// Protocol display name
    pub title: String,
    /// Protocol icon URL
    #[serde(default)]
    pub img: Option<String>,
    /// Whether this protocol is currently enabled
    #[serde(default)]
    pub img_color: Option<String>,
}

/// Response from /liquidity-sources endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquiditySourcesResponse {
    /// List of available liquidity sources
    pub protocols: Vec<LiquiditySource>,
}

/// Approval transaction response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalTransaction {
    /// Address to send approval transaction to (token contract)
    pub to: String,
    /// Encoded approval transaction data
    pub data: String,
    /// Value (always 0 for approvals)
    pub value: String,
    /// Estimated gas for the approval
    #[serde(default)]
    pub gas_price: Option<String>,
}

/// Allowance response from /approve/allowance endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowanceResponse {
    /// Current allowance amount
    pub allowance: String,
}

/// Spender address response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpenderResponse {
    /// The 1inch router address that needs approval
    pub address: String,
}

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// HTTP status code
    #[serde(default)]
    pub status_code: Option<u16>,
    /// Error message
    pub error: Option<String>,
    /// Detailed error description
    pub description: Option<String>,
    /// Request ID for debugging
    #[serde(default)]
    pub request_id: Option<String>,
    /// Additional metadata
    #[serde(default)]
    pub meta: Option<Vec<ApiErrorMeta>>,
}

/// Additional error metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorMeta {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    /// Error value/message
    pub value: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_ids() {
        assert_eq!(Chain::Ethereum.chain_id(), 1);
        assert_eq!(Chain::Bsc.chain_id(), 56);
        assert_eq!(Chain::Polygon.chain_id(), 137);
        assert_eq!(Chain::Arbitrum.chain_id(), 42161);
        assert_eq!(Chain::Base.chain_id(), 8453);
    }

    #[test]
    fn test_chain_from_id() {
        assert_eq!(Chain::from_chain_id(1), Some(Chain::Ethereum));
        assert_eq!(Chain::from_chain_id(56), Some(Chain::Bsc));
        assert_eq!(Chain::from_chain_id(999999), None);
    }

    #[test]
    fn test_chain_parse() {
        assert_eq!(Chain::parse("ethereum"), Some(Chain::Ethereum));
        assert_eq!(Chain::parse("ETH"), Some(Chain::Ethereum));
        assert_eq!(Chain::parse("1"), Some(Chain::Ethereum));
        assert_eq!(Chain::parse("polygon"), Some(Chain::Polygon));
        assert_eq!(Chain::parse("137"), Some(Chain::Polygon));
        assert_eq!(Chain::parse("unknown"), None);
    }

    #[test]
    fn test_chain_from_str_trait() {
        use std::str::FromStr;
        assert_eq!("ethereum".parse::<Chain>(), Ok(Chain::Ethereum));
        assert_eq!("ETH".parse::<Chain>(), Ok(Chain::Ethereum));
        assert_eq!(Chain::from_str("polygon"), Ok(Chain::Polygon));
        assert!("unknown".parse::<Chain>().is_err());
    }

    #[test]
    fn test_quote_request_builder() {
        let request = QuoteRequest::new(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
        )
        .with_fee(0.5)
        .with_complexity_level(2)
        .with_tokens_info()
        .with_protocols_info();

        assert_eq!(request.fee, Some(0.5));
        assert_eq!(request.complexity_level, Some(2));
        assert_eq!(request.include_tokens_info, Some(true));
        assert_eq!(request.include_protocols, Some(true));

        let params = request.to_query_params();
        assert!(params.iter().any(|(k, _)| *k == "src"));
        assert!(params.iter().any(|(k, _)| *k == "dst"));
        assert!(params.iter().any(|(k, _)| *k == "amount"));
        assert!(params.iter().any(|(k, _)| *k == "fee"));
    }

    #[test]
    fn test_swap_request_builder() {
        let request = SwapRequest::new(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            1.0,
        )
        .with_referrer("0xReferrer")
        .with_partial_fill()
        .with_dest_receiver("0xReceiver");

        assert_eq!(request.slippage, 1.0);
        assert_eq!(request.referrer, Some("0xReferrer".to_string()));
        assert_eq!(request.allow_partial_fill, Some(true));
        assert_eq!(request.dest_receiver, Some("0xReceiver".to_string()));
    }

    #[test]
    fn test_swap_from_quote() {
        let quote = QuoteRequest::new("0xSrc", "0xDst", "1000")
            .with_fee(0.3)
            .with_protocols("UNISWAP_V3,SUSHISWAP");

        let swap = SwapRequest::from_quote(quote, "0xFrom", 0.5);

        assert_eq!(swap.src, "0xSrc");
        assert_eq!(swap.dst, "0xDst");
        assert_eq!(swap.amount, "1000");
        assert_eq!(swap.from, "0xFrom");
        assert_eq!(swap.slippage, 0.5);
        assert_eq!(swap.fee, Some(0.3));
        assert_eq!(swap.protocols, Some("UNISWAP_V3,SUSHISWAP".to_string()));
    }
}
