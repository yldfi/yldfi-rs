//! Types for the 0x Swap API v2
//!
//! This module contains request and response types for the 0x Swap API,
//! including support for the Permit2 endpoints.

use serde::{Deserialize, Serialize};

/// Supported chains for 0x API
///
/// Each chain has a specific chain ID used for API requests and
/// transaction signing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Chain {
    /// Ethereum mainnet (chain ID: 1)
    Ethereum,
    /// Polygon (chain ID: 137)
    Polygon,
    /// Arbitrum One (chain ID: 42161)
    Arbitrum,
    /// Optimism (chain ID: 10)
    Optimism,
    /// Base (chain ID: 8453)
    Base,
    /// BNB Chain (chain ID: 56)
    Bsc,
    /// Avalanche C-Chain (chain ID: 43114)
    Avalanche,
    /// Fantom (chain ID: 250)
    Fantom,
    /// Celo (chain ID: 42220)
    Celo,
    /// Blast (chain ID: 81457)
    Blast,
    /// Linea (chain ID: 59144)
    Linea,
    /// Scroll (chain ID: 534352)
    Scroll,
    /// Mantle (chain ID: 5000)
    Mantle,
    /// Sepolia testnet (chain ID: 11155111)
    Sepolia,
}

impl Chain {
    /// Get the chain ID
    #[must_use]
    pub const fn chain_id(&self) -> u64 {
        match self {
            Self::Ethereum => 1,
            Self::Polygon => 137,
            Self::Arbitrum => 42161,
            Self::Optimism => 10,
            Self::Base => 8453,
            Self::Bsc => 56,
            Self::Avalanche => 43114,
            Self::Fantom => 250,
            Self::Celo => 42220,
            Self::Blast => 81457,
            Self::Linea => 59144,
            Self::Scroll => 534352,
            Self::Mantle => 5000,
            Self::Sepolia => 11155111,
        }
    }

    /// Get the chain name as used in the API URL
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Polygon => "polygon",
            Self::Arbitrum => "arbitrum",
            Self::Optimism => "optimism",
            Self::Base => "base",
            Self::Bsc => "bsc",
            Self::Avalanche => "avalanche",
            Self::Fantom => "fantom",
            Self::Celo => "celo",
            Self::Blast => "blast",
            Self::Linea => "linea",
            Self::Scroll => "scroll",
            Self::Mantle => "mantle",
            Self::Sepolia => "sepolia",
        }
    }

    /// Parse chain from string (case-insensitive)
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" | "1" => Some(Self::Ethereum),
            "polygon" | "matic" | "137" => Some(Self::Polygon),
            "arbitrum" | "arb" | "42161" => Some(Self::Arbitrum),
            "optimism" | "op" | "10" => Some(Self::Optimism),
            "base" | "8453" => Some(Self::Base),
            "bsc" | "bnb" | "binance" | "56" => Some(Self::Bsc),
            "avalanche" | "avax" | "43114" => Some(Self::Avalanche),
            "fantom" | "ftm" | "250" => Some(Self::Fantom),
            "celo" | "42220" => Some(Self::Celo),
            "blast" | "81457" => Some(Self::Blast),
            "linea" | "59144" => Some(Self::Linea),
            "scroll" | "534352" => Some(Self::Scroll),
            "mantle" | "mnt" | "5000" => Some(Self::Mantle),
            "sepolia" | "11155111" => Some(Self::Sepolia),
            _ => None,
        }
    }

    /// Parse chain from chain ID
    #[must_use]
    pub const fn from_chain_id(id: u64) -> Option<Self> {
        match id {
            1 => Some(Self::Ethereum),
            137 => Some(Self::Polygon),
            42161 => Some(Self::Arbitrum),
            10 => Some(Self::Optimism),
            8453 => Some(Self::Base),
            56 => Some(Self::Bsc),
            43114 => Some(Self::Avalanche),
            250 => Some(Self::Fantom),
            42220 => Some(Self::Celo),
            81457 => Some(Self::Blast),
            59144 => Some(Self::Linea),
            534352 => Some(Self::Scroll),
            5000 => Some(Self::Mantle),
            11155111 => Some(Self::Sepolia),
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
            yldfi_common::Chain::Arbitrum => Ok(Self::Arbitrum),
            yldfi_common::Chain::Optimism => Ok(Self::Optimism),
            yldfi_common::Chain::Base => Ok(Self::Base),
            yldfi_common::Chain::Bsc => Ok(Self::Bsc),
            yldfi_common::Chain::Avalanche => Ok(Self::Avalanche),
            yldfi_common::Chain::Fantom => Ok(Self::Fantom),
            yldfi_common::Chain::Celo => Ok(Self::Celo),
            yldfi_common::Chain::Blast => Ok(Self::Blast),
            yldfi_common::Chain::Linea => Ok(Self::Linea),
            yldfi_common::Chain::Scroll => Ok(Self::Scroll),
            yldfi_common::Chain::Mantle => Ok(Self::Mantle),
            yldfi_common::Chain::Sepolia => Ok(Self::Sepolia),
            _ => Err("Chain not supported by 0x API"),
        }
    }
}

impl From<Chain> for yldfi_common::Chain {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Ethereum => Self::Ethereum,
            Chain::Polygon => Self::Polygon,
            Chain::Arbitrum => Self::Arbitrum,
            Chain::Optimism => Self::Optimism,
            Chain::Base => Self::Base,
            Chain::Bsc => Self::Bsc,
            Chain::Avalanche => Self::Avalanche,
            Chain::Fantom => Self::Fantom,
            Chain::Celo => Self::Celo,
            Chain::Blast => Self::Blast,
            Chain::Linea => Self::Linea,
            Chain::Scroll => Self::Scroll,
            Chain::Mantle => Self::Mantle,
            Chain::Sepolia => Self::Sepolia,
        }
    }
}

/// Quote request parameters for the 0x Swap API v2
///
/// Use this to get a swap quote with full transaction data ready for execution.
/// The quote uses Permit2 for token approvals when possible.
#[derive(Debug, Clone, Default)]
pub struct QuoteRequest {
    /// Address of the token to sell
    pub sell_token: String,
    /// Address of the token to buy
    pub buy_token: String,
    /// Amount of sell token (in base units). Mutually exclusive with `buy_amount`.
    pub sell_amount: Option<String>,
    /// Amount of buy token (in base units). Mutually exclusive with `sell_amount`.
    pub buy_amount: Option<String>,
    /// Address that will execute the swap (required for full quote)
    pub taker: Option<String>,
    /// Slippage tolerance in basis points (e.g., 100 = 1%)
    pub slippage_bps: Option<u32>,
    /// Comma-separated list of liquidity sources to exclude
    pub excluded_sources: Option<String>,
    /// Enable gasless trading (requires affiliate address)
    pub gasless: Option<bool>,
    /// Affiliate address for fee collection
    pub affiliate_address: Option<String>,
    /// Affiliate fee in basis points
    pub affiliate_fee_bps: Option<u32>,
    /// Skip validation of the quote
    pub skip_validation: Option<bool>,
    /// Fee recipient address (for integrators)
    pub fee_recipient: Option<String>,
    /// Buy token fee percentage (for integrators)
    pub buy_token_percentage_fee: Option<String>,
    /// Intent on fulfillment (for gasless trades)
    pub intent_on_filling: Option<bool>,
}

impl QuoteRequest {
    /// Create a new quote request for selling a specific amount
    ///
    /// # Arguments
    /// * `sell_token` - Address of the token to sell
    /// * `buy_token` - Address of the token to buy
    /// * `sell_amount` - Amount to sell in base units (e.g., wei for ETH)
    #[must_use]
    pub fn sell(
        sell_token: impl Into<String>,
        buy_token: impl Into<String>,
        sell_amount: impl Into<String>,
    ) -> Self {
        Self {
            sell_token: sell_token.into(),
            buy_token: buy_token.into(),
            sell_amount: Some(sell_amount.into()),
            buy_amount: None,
            ..Default::default()
        }
    }

    /// Create a new quote request for buying a specific amount
    ///
    /// # Arguments
    /// * `sell_token` - Address of the token to sell
    /// * `buy_token` - Address of the token to buy
    /// * `buy_amount` - Amount to buy in base units
    #[must_use]
    pub fn buy(
        sell_token: impl Into<String>,
        buy_token: impl Into<String>,
        buy_amount: impl Into<String>,
    ) -> Self {
        Self {
            sell_token: sell_token.into(),
            buy_token: buy_token.into(),
            sell_amount: None,
            buy_amount: Some(buy_amount.into()),
            ..Default::default()
        }
    }

    /// Set the taker address (required for executable quotes)
    #[must_use]
    pub fn with_taker(mut self, taker: impl Into<String>) -> Self {
        self.taker = Some(taker.into());
        self
    }

    /// Set slippage tolerance in basis points (100 = 1%)
    #[must_use]
    pub fn with_slippage_bps(mut self, bps: u32) -> Self {
        self.slippage_bps = Some(bps);
        self
    }

    /// Set slippage tolerance as a percentage (1.0 = 1%)
    #[must_use]
    pub fn with_slippage_percent(mut self, percent: f64) -> Self {
        self.slippage_bps = Some((percent * 100.0) as u32);
        self
    }

    /// Exclude specific liquidity sources
    #[must_use]
    pub fn with_excluded_sources(mut self, sources: impl Into<String>) -> Self {
        self.excluded_sources = Some(sources.into());
        self
    }

    /// Enable gasless trading
    #[must_use]
    pub fn with_gasless(mut self, enabled: bool) -> Self {
        self.gasless = Some(enabled);
        self
    }

    /// Set affiliate address for fee collection
    #[must_use]
    pub fn with_affiliate(mut self, address: impl Into<String>, fee_bps: u32) -> Self {
        self.affiliate_address = Some(address.into());
        self.affiliate_fee_bps = Some(fee_bps);
        self
    }

    /// Skip quote validation
    #[must_use]
    pub fn with_skip_validation(mut self, skip: bool) -> Self {
        self.skip_validation = Some(skip);
        self
    }

    /// Set fee recipient for integrators
    #[must_use]
    pub fn with_fee_recipient(
        mut self,
        recipient: impl Into<String>,
        percentage: impl Into<String>,
    ) -> Self {
        self.fee_recipient = Some(recipient.into());
        self.buy_token_percentage_fee = Some(percentage.into());
        self
    }

    /// Convert to query parameters for the API request
    #[must_use]
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();

        params.push(("sellToken".to_string(), self.sell_token.clone()));
        params.push(("buyToken".to_string(), self.buy_token.clone()));

        if let Some(ref amount) = self.sell_amount {
            params.push(("sellAmount".to_string(), amount.clone()));
        }
        if let Some(ref amount) = self.buy_amount {
            params.push(("buyAmount".to_string(), amount.clone()));
        }
        if let Some(ref taker) = self.taker {
            params.push(("taker".to_string(), taker.clone()));
        }
        if let Some(bps) = self.slippage_bps {
            params.push(("slippageBps".to_string(), bps.to_string()));
        }
        if let Some(ref sources) = self.excluded_sources {
            params.push(("excludedSources".to_string(), sources.clone()));
        }
        if let Some(gasless) = self.gasless {
            params.push(("gasless".to_string(), gasless.to_string()));
        }
        if let Some(ref address) = self.affiliate_address {
            params.push(("affiliateAddress".to_string(), address.clone()));
        }
        if let Some(fee) = self.affiliate_fee_bps {
            params.push(("affiliateFeeBps".to_string(), fee.to_string()));
        }
        if let Some(skip) = self.skip_validation {
            params.push(("skipValidation".to_string(), skip.to_string()));
        }
        if let Some(ref recipient) = self.fee_recipient {
            params.push(("feeRecipient".to_string(), recipient.clone()));
        }
        if let Some(ref fee) = self.buy_token_percentage_fee {
            params.push(("buyTokenPercentageFee".to_string(), fee.clone()));
        }
        if let Some(intent) = self.intent_on_filling {
            params.push(("intentOnFilling".to_string(), intent.to_string()));
        }

        params
    }
}

/// Price request parameters (lighter weight, no transaction data)
///
/// Use this for indicative pricing without generating transaction data.
/// Faster and doesn't require a taker address.
pub type PriceRequest = QuoteRequest;

/// Quote response from the 0x Swap API v2
///
/// Contains the swap route, pricing, and transaction data ready for execution.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    /// Chain ID for the quote
    #[serde(default)]
    pub chain_id: Option<u64>,
    /// Amount of sell token
    pub sell_amount: String,
    /// Amount of buy token
    pub buy_amount: String,
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Exchange rate: buy amount / sell amount
    #[serde(default)]
    pub price: Option<String>,
    /// Estimated gas for the transaction (API returns "gas" not "estimatedGas")
    #[serde(default, alias = "gas")]
    pub estimated_gas: Option<String>,
    /// Gas price used for estimation
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Gross price (before fees)
    #[serde(default)]
    pub gross_buy_amount: Option<String>,
    /// Gross sell amount (before fees)
    #[serde(default)]
    pub gross_sell_amount: Option<String>,
    /// Liquidity sources used in the route
    #[serde(default)]
    pub liquidity_sources: Option<Vec<LiquiditySource>>,
    /// Detailed route information
    #[serde(default)]
    pub route: Option<Route>,
    /// Transaction data (only present when taker is provided)
    #[serde(default)]
    pub transaction: Option<Transaction>,
    /// Permit2 EIP-712 data for gasless approvals
    #[serde(default)]
    pub permit2: Option<Permit2Data>,
    /// Minimum buy amount after slippage
    #[serde(default)]
    pub min_buy_amount: Option<String>,
    /// Token tax metadata
    #[serde(default)]
    pub token_metadata: Option<TokenMetadataWrapper>,
    /// Issues/warnings about the quote
    #[serde(default)]
    pub issues: Option<QuoteIssues>,
    /// Whether liquidity is available for this swap
    #[serde(default)]
    pub liquidity_available: Option<bool>,
    /// Allowance target (Permit2 contract address)
    #[serde(default)]
    pub allowance_target: Option<String>,
}

impl QuoteResponse {
    /// Get the effective exchange rate
    #[must_use]
    pub fn exchange_rate(&self) -> Option<f64> {
        self.price.as_ref().and_then(|p| p.parse().ok())
    }

    /// Get estimated gas as u64
    #[must_use]
    pub fn gas_estimate(&self) -> Option<u64> {
        self.estimated_gas.as_ref().and_then(|g| g.parse().ok())
    }

    /// Check if this quote has executable transaction data
    #[must_use]
    pub fn has_transaction(&self) -> bool {
        self.transaction.is_some()
    }
}

/// Price response from the 0x Swap API v2
///
/// Lighter weight response for indicative pricing.
/// Does not include transaction data.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceResponse {
    /// Chain ID for the price
    #[serde(default)]
    pub chain_id: Option<u64>,
    /// Amount of sell token
    pub sell_amount: String,
    /// Amount of buy token
    pub buy_amount: String,
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Exchange rate: buy amount / sell amount
    #[serde(default)]
    pub price: Option<String>,
    /// Estimated gas for the swap (API returns "gas" not "estimatedGas")
    #[serde(default, alias = "gas")]
    pub estimated_gas: Option<String>,
    /// Gas price used for estimation
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Gross buy amount (before fees)
    #[serde(default)]
    pub gross_buy_amount: Option<String>,
    /// Gross sell amount (before fees)
    #[serde(default)]
    pub gross_sell_amount: Option<String>,
    /// Liquidity sources available for this swap
    #[serde(default)]
    pub liquidity_sources: Option<Vec<LiquiditySource>>,
    /// Route information
    #[serde(default)]
    pub route: Option<Route>,
    /// Token tax metadata
    #[serde(default)]
    pub token_metadata: Option<TokenMetadataWrapper>,
    /// Whether liquidity is available for this swap
    #[serde(default)]
    pub liquidity_available: Option<bool>,
    /// Minimum buy amount after slippage
    #[serde(default)]
    pub min_buy_amount: Option<String>,
    /// Allowance target (Permit2 contract address)
    #[serde(default)]
    pub allowance_target: Option<String>,
}

impl PriceResponse {
    /// Get the effective exchange rate
    #[must_use]
    pub fn exchange_rate(&self) -> Option<f64> {
        self.price.as_ref().and_then(|p| p.parse().ok())
    }

    /// Get estimated gas as u64
    #[must_use]
    pub fn gas_estimate(&self) -> Option<u64> {
        self.estimated_gas.as_ref().and_then(|g| g.parse().ok())
    }
}

/// Liquidity source information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LiquiditySource {
    /// Source name (e.g., "Uniswap_V3", "Curve")
    pub name: String,
    /// Proportion of the swap routed through this source (0-1)
    pub proportion: String,
}

impl LiquiditySource {
    /// Get proportion as a float (0.0 to 1.0)
    #[must_use]
    pub fn proportion_float(&self) -> Option<f64> {
        self.proportion.parse().ok()
    }

    /// Get proportion as a percentage (0 to 100)
    #[must_use]
    pub fn proportion_percent(&self) -> Option<f64> {
        self.proportion_float().map(|p| p * 100.0)
    }
}

/// Route information showing the swap path
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    /// Individual fills in the route
    #[serde(default)]
    pub fills: Vec<RouteFill>,
    /// Tokens involved in the route
    #[serde(default)]
    pub tokens: Vec<RouteToken>,
}

/// A single fill in the swap route
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteFill {
    /// Source of this fill (DEX name)
    pub source: String,
    /// Input token address
    #[serde(default)]
    pub from: Option<String>,
    /// Output token address
    #[serde(default)]
    pub to: Option<String>,
    /// Proportion of total swap amount in basis points
    #[serde(default)]
    pub proportion_bps: Option<String>,
}

/// Token in the route
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteToken {
    /// Token address
    pub address: String,
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,
}

/// Transaction data ready for execution
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    /// Target contract address
    pub to: String,
    /// Encoded calldata
    pub data: String,
    /// ETH value to send (in wei, as string)
    pub value: String,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
    /// Gas price (legacy)
    #[serde(default)]
    pub gas_price: Option<String>,
}

impl Transaction {
    /// Get gas limit as u64
    #[must_use]
    pub fn gas_limit(&self) -> Option<u64> {
        self.gas.as_ref().and_then(|g| g.parse().ok())
    }

    /// Get value as u128 (wei)
    #[must_use]
    pub fn value_wei(&self) -> Option<u128> {
        self.value.parse().ok()
    }

    /// Check if this is a native token (ETH) transaction
    #[must_use]
    pub fn is_native_token_tx(&self) -> bool {
        self.value_wei().is_some_and(|v| v > 0)
    }
}

/// Permit2 data for gasless approvals
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Permit2Data {
    /// EIP-712 typed data for signing
    #[serde(default)]
    pub eip712: Option<serde_json::Value>,
}

/// Token metadata wrapper containing buy/sell token tax info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadataWrapper {
    /// Buy token tax info
    #[serde(default)]
    pub buy_token: Option<TokenTaxInfo>,
    /// Sell token tax info
    #[serde(default)]
    pub sell_token: Option<TokenTaxInfo>,
}

/// Token tax information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenTaxInfo {
    /// Buy tax in basis points
    #[serde(default)]
    pub buy_tax_bps: Option<String>,
    /// Sell tax in basis points
    #[serde(default)]
    pub sell_tax_bps: Option<String>,
    /// Transfer tax in basis points
    #[serde(default)]
    pub transfer_tax_bps: Option<String>,
}

/// Token metadata (legacy format for backward compatibility)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenMetadata {
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token decimals
    #[serde(default)]
    pub decimals: Option<u8>,
    /// Token name
    #[serde(default)]
    pub name: Option<String>,
}

/// Issues or warnings about the quote
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteIssues {
    /// Allowance issues
    #[serde(default)]
    pub allowance: Option<AllowanceIssue>,
    /// Balance issues
    #[serde(default)]
    pub balance: Option<BalanceIssue>,
    /// Simulation issues
    #[serde(default)]
    pub simulation_incompleted: Option<bool>,
}

/// Allowance issue details
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowanceIssue {
    /// Current allowance
    #[serde(default)]
    pub actual: Option<String>,
    /// Required allowance
    #[serde(default)]
    pub expected: Option<String>,
}

/// Balance issue details
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BalanceIssue {
    /// Token address
    #[serde(default)]
    pub token: Option<String>,
    /// Current balance
    #[serde(default)]
    pub actual: Option<String>,
    /// Required balance
    #[serde(default)]
    pub expected: Option<String>,
}

/// Liquidity source information from the /sources endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Source identifier used in API requests
    pub name: String,
    /// Human-readable source name
    #[serde(default)]
    pub display_name: Option<String>,
}

/// Response from the /sources endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourcesResponse {
    /// Available liquidity sources
    pub sources: Vec<Source>,
}

/// API error response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Error code
    #[serde(default)]
    pub code: Option<i32>,
    /// Error reason/type
    #[serde(default)]
    pub reason: Option<String>,
    /// Detailed error message
    #[serde(default)]
    pub message: Option<String>,
    /// Validation errors
    #[serde(default)]
    pub validation_errors: Option<Vec<ValidationError>>,
}

/// Validation error details
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Error code
    #[serde(default)]
    pub code: Option<String>,
    /// Error message
    #[serde(default)]
    pub reason: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id() {
        assert_eq!(Chain::Ethereum.chain_id(), 1);
        assert_eq!(Chain::Polygon.chain_id(), 137);
        assert_eq!(Chain::Base.chain_id(), 8453);
    }

    #[test]
    fn test_chain_parse() {
        assert_eq!(Chain::parse("ethereum"), Some(Chain::Ethereum));
        assert_eq!(Chain::parse("ETH"), Some(Chain::Ethereum));
        assert_eq!(Chain::parse("1"), Some(Chain::Ethereum));
        assert_eq!(Chain::parse("polygon"), Some(Chain::Polygon));
        assert_eq!(Chain::parse("unknown"), None);
    }

    #[test]
    fn test_chain_from_chain_id() {
        assert_eq!(Chain::from_chain_id(1), Some(Chain::Ethereum));
        assert_eq!(Chain::from_chain_id(137), Some(Chain::Polygon));
        assert_eq!(Chain::from_chain_id(99999), None);
    }

    #[test]
    fn test_quote_request_sell() {
        let request = QuoteRequest::sell(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000000000000",
        )
        .with_taker("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .with_slippage_bps(100);

        assert_eq!(request.sell_amount, Some("1000000000000000000".to_string()));
        assert!(request.buy_amount.is_none());
        assert_eq!(request.slippage_bps, Some(100));
    }

    #[test]
    fn test_quote_request_buy() {
        let request = QuoteRequest::buy(
            "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "1000000000",
        );

        assert!(request.sell_amount.is_none());
        assert_eq!(request.buy_amount, Some("1000000000".to_string()));
    }

    #[test]
    fn test_quote_request_to_query_params() {
        let request = QuoteRequest::sell("0xA", "0xB", "100")
            .with_taker("0xC")
            .with_slippage_bps(50);

        let params = request.to_query_params();
        assert!(params.contains(&("sellToken".to_string(), "0xA".to_string())));
        assert!(params.contains(&("buyToken".to_string(), "0xB".to_string())));
        assert!(params.contains(&("sellAmount".to_string(), "100".to_string())));
        assert!(params.contains(&("taker".to_string(), "0xC".to_string())));
        assert!(params.contains(&("slippageBps".to_string(), "50".to_string())));
    }

    #[test]
    fn test_liquidity_source_proportion() {
        let source = LiquiditySource {
            name: "Uniswap_V3".to_string(),
            proportion: "0.75".to_string(),
        };

        assert_eq!(source.proportion_float(), Some(0.75));
        assert_eq!(source.proportion_percent(), Some(75.0));
    }

    #[test]
    fn test_transaction_helpers() {
        let tx = Transaction {
            to: "0x123".to_string(),
            data: "0xabcd".to_string(),
            value: "1000000000000000000".to_string(),
            gas: Some("200000".to_string()),
            gas_price: None,
        };

        assert_eq!(tx.gas_limit(), Some(200_000));
        assert_eq!(tx.value_wei(), Some(1_000_000_000_000_000_000));
        assert!(tx.is_native_token_tx());
    }
}
