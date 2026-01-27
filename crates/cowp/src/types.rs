//! Types for the `CoW` Protocol API responses

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Error returned when parsing a chain from a string fails
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseChainError {
    input: String,
}

impl fmt::Display for ParseChainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown chain: {}", self.input)
    }
}

impl std::error::Error for ParseChainError {}

/// Supported chains for `CoW` Protocol
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Chain {
    /// Ethereum mainnet
    Mainnet,
    /// Gnosis Chain (xDai)
    Gnosis,
    /// Arbitrum One
    Arbitrum,
    /// Sepolia testnet
    Sepolia,
}

impl Chain {
    /// Convert from EVM chain ID
    #[must_use] 
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            1 => Some(Chain::Mainnet),
            100 => Some(Chain::Gnosis),
            42161 => Some(Chain::Arbitrum),
            11155111 => Some(Chain::Sepolia),
            _ => None,
        }
    }

    /// Get the API base URL for this chain
    #[must_use] 
    pub fn api_url(&self) -> &'static str {
        match self {
            Chain::Mainnet => "https://api.cow.fi/mainnet",
            Chain::Gnosis => "https://api.cow.fi/xdai",
            Chain::Arbitrum => "https://api.cow.fi/arbitrum_one",
            Chain::Sepolia => "https://api.cow.fi/sepolia",
        }
    }

    /// Get the chain ID
    #[must_use] 
    pub fn chain_id(&self) -> u64 {
        match self {
            Chain::Mainnet => 1,
            Chain::Gnosis => 100,
            Chain::Arbitrum => 42161,
            Chain::Sepolia => 11155111,
        }
    }

    /// Parse chain from string (returns Option for backward compatibility)
    #[must_use] 
    pub fn try_from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }

    /// Get chain name
    #[must_use] 
    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Mainnet => "mainnet",
            Chain::Gnosis => "gnosis",
            Chain::Arbitrum => "arbitrum",
            Chain::Sepolia => "sepolia",
        }
    }
}

impl std::fmt::Display for Chain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for Chain {
    type Err = ParseChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mainnet" | "ethereum" | "eth" | "1" => Ok(Chain::Mainnet),
            "gnosis" | "xdai" | "100" => Ok(Chain::Gnosis),
            "arbitrum" | "arb" | "42161" => Ok(Chain::Arbitrum),
            "sepolia" | "11155111" => Ok(Chain::Sepolia),
            _ => Err(ParseChainError {
                input: s.to_string(),
            }),
        }
    }
}

impl TryFrom<yldfi_common::Chain> for Chain {
    type Error = &'static str;

    fn try_from(chain: yldfi_common::Chain) -> Result<Self, Self::Error> {
        match chain {
            yldfi_common::Chain::Ethereum => Ok(Self::Mainnet),
            yldfi_common::Chain::Gnosis => Ok(Self::Gnosis),
            yldfi_common::Chain::Arbitrum => Ok(Self::Arbitrum),
            yldfi_common::Chain::Sepolia => Ok(Self::Sepolia),
            _ => Err("Chain not supported by CoW Protocol"),
        }
    }
}

impl From<Chain> for yldfi_common::Chain {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Mainnet => Self::Ethereum,
            Chain::Gnosis => Self::Gnosis,
            Chain::Arbitrum => Self::Arbitrum,
            Chain::Sepolia => Self::Sepolia,
        }
    }
}

/// Order kind (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderKind {
    /// Sell order - exact input amount
    Sell,
    /// Buy order - exact output amount
    Buy,
}

/// Quote request parameters
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    /// Token to sell
    pub sell_token: String,
    /// Token to buy
    pub buy_token: String,
    /// Address that will receive the tokens
    pub receiver: Option<String>,
    /// Amount to sell (for sell orders) or buy (for buy orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount_before_fee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount_after_fee: Option<String>,
    /// Order kind (buy or sell)
    pub kind: OrderKind,
    /// Address placing the order
    pub from: String,
    /// Signing scheme
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_scheme: Option<SigningScheme>,
    /// Price quality preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_quality: Option<PriceQuality>,
    /// Whether to only check for EIP-1271 signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_chain_order: Option<bool>,
    /// App data (bytes32 hash)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
}

impl QuoteRequest {
    /// Create a sell order quote request
    pub fn sell(
        sell_token: impl Into<String>,
        buy_token: impl Into<String>,
        sell_amount: impl Into<String>,
        from: impl Into<String>,
    ) -> Self {
        Self {
            sell_token: sell_token.into(),
            buy_token: buy_token.into(),
            receiver: None,
            sell_amount_before_fee: Some(sell_amount.into()),
            buy_amount_after_fee: None,
            kind: OrderKind::Sell,
            from: from.into(),
            signing_scheme: None,
            price_quality: None,
            on_chain_order: None,
            app_data: None,
        }
    }

    /// Create a buy order quote request
    pub fn buy(
        sell_token: impl Into<String>,
        buy_token: impl Into<String>,
        buy_amount: impl Into<String>,
        from: impl Into<String>,
    ) -> Self {
        Self {
            sell_token: sell_token.into(),
            buy_token: buy_token.into(),
            receiver: None,
            sell_amount_before_fee: None,
            buy_amount_after_fee: Some(buy_amount.into()),
            kind: OrderKind::Buy,
            from: from.into(),
            signing_scheme: None,
            price_quality: None,
            on_chain_order: None,
            app_data: None,
        }
    }

    /// Set the receiver address
    #[must_use]
    pub fn with_receiver(mut self, receiver: impl Into<String>) -> Self {
        self.receiver = Some(receiver.into());
        self
    }

    /// Set the signing scheme
    #[must_use]
    pub fn with_signing_scheme(mut self, scheme: SigningScheme) -> Self {
        self.signing_scheme = Some(scheme);
        self
    }

    /// Set price quality preference
    #[must_use]
    pub fn with_price_quality(mut self, quality: PriceQuality) -> Self {
        self.price_quality = Some(quality);
        self
    }

    /// Set app data
    #[must_use]
    pub fn with_app_data(mut self, app_data: impl Into<String>) -> Self {
        self.app_data = Some(app_data.into());
        self
    }

    /// Validate the quote request parameters
    ///
    /// Checks that addresses and amounts are properly formatted.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Token addresses are not valid Ethereum addresses (0x + 40 hex chars)
    /// - From address is not a valid Ethereum address
    /// - Amounts contain non-numeric characters
    pub fn validate(&self) -> Result<(), &'static str> {
        // Validate addresses
        if !is_valid_address(&self.sell_token) {
            return Err("invalid sell_token address");
        }
        if !is_valid_address(&self.buy_token) {
            return Err("invalid buy_token address");
        }
        if !is_valid_address(&self.from) {
            return Err("invalid from address");
        }
        if let Some(ref receiver) = self.receiver {
            if !is_valid_address(receiver) {
                return Err("invalid receiver address");
            }
        }

        // Validate amounts (should be numeric strings)
        if let Some(ref amount) = self.sell_amount_before_fee {
            if !is_valid_amount(amount) {
                return Err("invalid sell_amount_before_fee");
            }
        }
        if let Some(ref amount) = self.buy_amount_after_fee {
            if !is_valid_amount(amount) {
                return Err("invalid buy_amount_after_fee");
            }
        }

        Ok(())
    }
}

/// Check if a string is a valid Ethereum address (0x + 40 hex chars)
fn is_valid_address(s: &str) -> bool {
    if !s.starts_with("0x") || s.len() != 42 {
        return false;
    }
    s[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Check if a string is a valid amount (numeric string, no decimals)
fn is_valid_amount(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

/// Signing scheme for orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SigningScheme {
    /// EIP-712 typed data signature
    Eip712,
    /// EIP-1271 smart contract signature
    Eip1271,
    /// Pre-signed order
    PreSign,
}

/// Price quality preference
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriceQuality {
    /// Fast quote, may be less accurate
    Fast,
    /// Optimal quote, takes longer
    Optimal,
    /// Verified quote with solver competition
    Verified,
}

/// Quote response from `CoW` Protocol
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    /// Quote ID for placing order
    pub id: Option<i64>,
    /// Quote details
    pub quote: QuoteDetails,
    /// Address placing the order
    pub from: String,
    /// Expiration timestamp
    pub expiration: String,
    /// Whether the quote is verified
    #[serde(default)]
    pub verified: bool,
}

/// Quote details
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteDetails {
    /// Token to sell
    pub sell_token: String,
    /// Token to buy
    pub buy_token: String,
    /// Receiver address (may be null if same as sender)
    pub receiver: Option<String>,
    /// Amount to sell (before fee)
    pub sell_amount: String,
    /// Amount to buy (after fee)
    pub buy_amount: String,
    /// Order validity duration
    pub valid_to: u64,
    /// App data hash
    pub app_data: String,
    /// Fee amount
    pub fee_amount: String,
    /// Order kind
    pub kind: OrderKind,
    /// Partially fillable
    pub partially_fillable: bool,
    /// Sell token balance source
    #[serde(default)]
    pub sell_token_balance: Option<String>,
    /// Buy token balance destination
    #[serde(default)]
    pub buy_token_balance: Option<String>,
    /// Signing scheme
    #[serde(default)]
    pub signing_scheme: Option<SigningScheme>,
}

/// Order for submission
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderCreation {
    /// Token to sell
    pub sell_token: String,
    /// Token to buy
    pub buy_token: String,
    /// Sell amount
    pub sell_amount: String,
    /// Buy amount
    pub buy_amount: String,
    /// Order validity
    pub valid_to: u64,
    /// App data
    pub app_data: String,
    /// Fee amount
    pub fee_amount: String,
    /// Order kind
    pub kind: OrderKind,
    /// Partially fillable
    pub partially_fillable: bool,
    /// Receiver
    pub receiver: String,
    /// Signature
    pub signature: String,
    /// Signing scheme
    pub signing_scheme: SigningScheme,
    /// Address placing order
    pub from: String,
    /// Quote ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

/// Order response after submission
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OrderResponse {
    /// Order UID
    #[serde(rename = "UID")]
    pub uid: String,
}

/// Order details
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Order UID
    pub uid: String,
    /// Token to sell
    pub sell_token: String,
    /// Token to buy
    pub buy_token: String,
    /// Sell amount
    pub sell_amount: String,
    /// Buy amount
    pub buy_amount: String,
    /// Order kind
    pub kind: OrderKind,
    /// Order status
    pub status: OrderStatus,
    /// Created timestamp
    pub created_date: String,
    /// Executed sell amount
    #[serde(default)]
    pub executed_sell_amount: Option<String>,
    /// Executed buy amount
    #[serde(default)]
    pub executed_buy_amount: Option<String>,
    /// Executed fee amount
    #[serde(default)]
    pub executed_fee_amount: Option<String>,
    /// Owner address
    pub owner: String,
    /// Receiver address
    #[serde(default)]
    pub receiver: Option<String>,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderStatus {
    /// Order is open
    Open,
    /// Order is fulfilled
    Fulfilled,
    /// Order is cancelled
    Cancelled,
    /// Order expired
    Expired,
    /// Order is presigning
    Presignaturepending,
}

/// API error response
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    /// Error type
    pub error_type: String,
    /// Error description
    pub description: String,
}

/// Trade details
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Block number
    pub block_number: u64,
    /// Log index
    pub log_index: u64,
    /// Order UID
    pub order_uid: String,
    /// Buyer address
    pub owner: String,
    /// Token sold
    pub sell_token: String,
    /// Token bought
    pub buy_token: String,
    /// Amount sold
    pub sell_amount: String,
    /// Amount bought
    pub buy_amount: String,
    /// Transaction hash
    pub tx_hash: String,
}
