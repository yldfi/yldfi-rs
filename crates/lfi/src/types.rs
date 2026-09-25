//! Types for the LI.FI API
//!
//! This module contains request and response types for the LI.FI cross-chain
//! bridge and DEX aggregator API.

use serde::{Deserialize, Serialize};

/// Chain ID type - uses numeric chain IDs
pub type ChainId = u64;

/// Serialize an optional list as a single comma-separated value.
///
/// `serde_urlencoded` (used for GET query strings) cannot serialize sequences,
/// and LI.FI accepts list-valued query parameters as comma-joined strings
/// (e.g. `chains=1,8453`, `allowBridges=hop,across`).
fn serialize_comma_list<T, S>(value: &Option<Vec<T>>, serializer: S) -> Result<S::Ok, S::Error>
where
    T: std::fmt::Display,
    S: serde::Serializer,
{
    match value {
        Some(items) => {
            let joined = items
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(",");
            serializer.serialize_some(&joined)
        }
        None => serializer.serialize_none(),
    }
}

/// Validate a slippage value expressed as a decimal fraction.
///
/// LI.FI expects slippage as a fraction (`0.005` = 0.5%). Returns an error
/// message if the value is outside `(0, 1)`.
///
/// # Errors
///
/// Returns an error string if the value is not finite or not in `(0, 1)`.
pub fn validate_slippage_fraction(slippage: f64) -> Result<(), String> {
    if !slippage.is_finite() || slippage <= 0.0 || slippage >= 1.0 {
        return Err(format!(
            "slippage must be a fraction between 0 and 1 (e.g. 0.005 = 0.5%), got {slippage}"
        ));
    }
    Ok(())
}

/// Convert a slippage percentage (e.g. `0.5` for 0.5%) to the decimal
/// fraction LI.FI expects (`0.005`).
///
/// # Errors
///
/// Returns an error string if the percentage is not in `(0, 100)`.
pub fn slippage_percent_to_fraction(percent: f64) -> Result<f64, String> {
    if !percent.is_finite() || percent <= 0.0 || percent >= 100.0 {
        return Err(format!(
            "slippage percent must be between 0 and 100 (e.g. 0.5 = 0.5%), got {percent}"
        ));
    }
    Ok(percent / 100.0)
}

/// Common chain IDs
pub mod chains {
    use super::ChainId;

    /// Ethereum Mainnet
    pub const ETHEREUM: ChainId = 1;
    /// Optimism
    pub const OPTIMISM: ChainId = 10;
    /// BNB Smart Chain
    pub const BSC: ChainId = 56;
    /// Gnosis Chain
    pub const GNOSIS: ChainId = 100;
    /// Polygon
    pub const POLYGON: ChainId = 137;
    /// Fantom
    pub const FANTOM: ChainId = 250;
    /// zkSync Era
    pub const ZKSYNC: ChainId = 324;
    /// Polygon zkEVM
    pub const POLYGON_ZKEVM: ChainId = 1101;
    /// Base
    pub const BASE: ChainId = 8453;
    /// Arbitrum One
    pub const ARBITRUM: ChainId = 42161;
    /// Avalanche C-Chain
    pub const AVALANCHE: ChainId = 43114;
    /// Linea
    pub const LINEA: ChainId = 59144;
    /// Blast
    pub const BLAST: ChainId = 81457;
    /// Scroll
    pub const SCROLL: ChainId = 534352;
    /// Mantle
    pub const MANTLE: ChainId = 5000;
    /// Mode
    pub const MODE: ChainId = 34443;
    /// Solana (non-EVM, uses different ID scheme)
    pub const SOLANA: ChainId = 1151111081099710;
}

// ============================================================================
// Token Types
// ============================================================================

/// Token information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// Token contract address
    pub address: String,
    /// Chain ID
    pub chain_id: ChainId,
    /// Token symbol
    pub symbol: String,
    /// Token decimals
    pub decimals: u8,
    /// Token name
    pub name: String,
    /// Coin key (optional, used by LI.FI for token identification)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coin_key: Option<String>,
    /// Logo URI
    #[serde(default, rename = "logoURI", skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    /// USD price
    #[serde(default, rename = "priceUSD", skip_serializing_if = "Option::is_none")]
    pub price_usd: Option<String>,
    /// Tags (e.g., "stablecoin", "`major_asset`")
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Token amount with token info
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAmount {
    /// Token information
    #[serde(flatten)]
    pub token: Token,
    /// Amount in base units (wei)
    pub amount: String,
    /// Amount in human-readable format
    #[serde(default, rename = "amountUSD", skip_serializing_if = "Option::is_none")]
    pub amount_usd: Option<String>,
}

// ============================================================================
// Quote Request/Response Types
// ============================================================================

/// Request parameters for getting a quote
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    /// Source chain ID
    pub from_chain: ChainId,
    /// Destination chain ID
    pub to_chain: ChainId,
    /// Source token address
    pub from_token: String,
    /// Destination token address
    pub to_token: String,
    /// Amount in base units (wei)
    pub from_amount: String,
    /// Sender address
    pub from_address: String,
    /// Recipient address (defaults to `from_address` if not specified)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_address: Option<String>,
    /// Slippage tolerance as a decimal fraction (e.g., `0.005` for 0.5%).
    ///
    /// This is sent to LI.FI unchanged; LI.FI interprets `0.5` as **50%**.
    /// Use [`slippage_percent_to_fraction`] to convert from percent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    /// Integrator identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrator: Option<String>,
    /// Fee percentage for integrator (0-3%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
    /// Referrer address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    /// Allowed bridges
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_comma_list"
    )]
    pub allow_bridges: Option<Vec<String>>,
    /// Denied bridges
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_comma_list"
    )]
    pub deny_bridges: Option<Vec<String>>,
    /// Allowed exchanges
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_comma_list"
    )]
    pub allow_exchanges: Option<Vec<String>>,
    /// Denied exchanges
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_comma_list"
    )]
    pub deny_exchanges: Option<Vec<String>>,
    /// Prefer specific route types
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<RouteOrder>,
    /// Allow switching chains even if destination has no swap
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_switch_chain: Option<bool>,
}

impl QuoteRequest {
    /// Create a new quote request
    pub fn new(
        from_chain: ChainId,
        to_chain: ChainId,
        from_token: impl Into<String>,
        to_token: impl Into<String>,
        from_amount: impl Into<String>,
        from_address: impl Into<String>,
    ) -> Self {
        Self {
            from_chain,
            to_chain,
            from_token: from_token.into(),
            to_token: to_token.into(),
            from_amount: from_amount.into(),
            from_address: from_address.into(),
            to_address: None,
            slippage: None,
            integrator: None,
            fee: None,
            referrer: None,
            allow_bridges: None,
            deny_bridges: None,
            allow_exchanges: None,
            deny_exchanges: None,
            order: None,
            allow_switch_chain: None,
        }
    }

    /// Set the destination address (recipient)
    #[must_use]
    pub fn with_to_address(mut self, to_address: impl Into<String>) -> Self {
        self.to_address = Some(to_address.into());
        self
    }

    /// Set slippage tolerance as a decimal fraction (`0.005` = 0.5%)
    #[must_use]
    pub fn with_slippage(mut self, slippage: f64) -> Self {
        self.slippage = Some(slippage);
        self
    }

    /// Set integrator identifier
    #[must_use]
    pub fn with_integrator(mut self, integrator: impl Into<String>) -> Self {
        self.integrator = Some(integrator.into());
        self
    }

    /// Set integrator fee percentage (0-3%)
    #[must_use]
    pub fn with_fee(mut self, fee: f64) -> Self {
        self.fee = Some(fee);
        self
    }

    /// Set referrer address
    #[must_use]
    pub fn with_referrer(mut self, referrer: impl Into<String>) -> Self {
        self.referrer = Some(referrer.into());
        self
    }

    /// Set allowed bridges
    #[must_use]
    pub fn with_allowed_bridges(mut self, bridges: Vec<String>) -> Self {
        self.allow_bridges = Some(bridges);
        self
    }

    /// Set denied bridges
    #[must_use]
    pub fn with_denied_bridges(mut self, bridges: Vec<String>) -> Self {
        self.deny_bridges = Some(bridges);
        self
    }

    /// Set allowed exchanges
    #[must_use]
    pub fn with_allowed_exchanges(mut self, exchanges: Vec<String>) -> Self {
        self.allow_exchanges = Some(exchanges);
        self
    }

    /// Set denied exchanges
    #[must_use]
    pub fn with_denied_exchanges(mut self, exchanges: Vec<String>) -> Self {
        self.deny_exchanges = Some(exchanges);
        self
    }

    /// Set route ordering preference
    #[must_use]
    pub fn with_order(mut self, order: RouteOrder) -> Self {
        self.order = Some(order);
        self
    }
}

/// Route ordering preference
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RouteOrder {
    /// Recommend the best route (default)
    #[default]
    Recommended,
    /// Fastest execution
    Fastest,
    /// Cheapest (best output)
    Cheapest,
    /// Most secure (safest bridges)
    Safest,
}

/// Quote response containing the best route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Quote {
    /// Unique quote ID
    pub id: String,
    /// Type of the quote (e.g., "lifi")
    #[serde(rename = "type")]
    pub quote_type: String,
    /// Tool used (bridge/exchange name)
    pub tool: String,
    /// Tool details
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_details: Option<ToolDetails>,
    /// Action details
    pub action: Action,
    /// Estimate of the swap/bridge
    pub estimate: Estimate,
    /// Transaction request to execute
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_request: Option<TransactionRequest>,
    /// Included steps
    #[serde(default)]
    pub included_steps: Vec<Step>,
    /// Integrator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrator: Option<String>,
}

// ============================================================================
// Routes Request/Response Types (Advanced API)
// ============================================================================

/// Request parameters for getting multiple routes (advanced API)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutesRequest {
    /// Source chain ID
    pub from_chain_id: ChainId,
    /// Source token address
    pub from_token_address: String,
    /// Amount in base units (wei)
    pub from_amount: String,
    /// Sender address
    pub from_address: String,
    /// Destination chain ID
    pub to_chain_id: ChainId,
    /// Destination token address
    pub to_token_address: String,
    /// Recipient address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_address: Option<String>,
    /// Route options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<RoutesOptions>,
}

impl RoutesRequest {
    /// Create a new routes request
    pub fn new(
        from_chain_id: ChainId,
        from_token_address: impl Into<String>,
        from_amount: impl Into<String>,
        from_address: impl Into<String>,
        to_chain_id: ChainId,
        to_token_address: impl Into<String>,
    ) -> Self {
        Self {
            from_chain_id,
            from_token_address: from_token_address.into(),
            from_amount: from_amount.into(),
            from_address: from_address.into(),
            to_chain_id,
            to_token_address: to_token_address.into(),
            to_address: None,
            options: None,
        }
    }

    /// Set the destination address
    #[must_use]
    pub fn with_to_address(mut self, to_address: impl Into<String>) -> Self {
        self.to_address = Some(to_address.into());
        self
    }

    /// Set route options
    #[must_use]
    pub fn with_options(mut self, options: RoutesOptions) -> Self {
        self.options = Some(options);
        self
    }
}

/// Options for route requests
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutesOptions {
    /// Slippage tolerance as a decimal fraction (e.g., `0.005` for 0.5%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    /// Integrator identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrator: Option<String>,
    /// Fee percentage for integrator
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee: Option<f64>,
    /// Referrer address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub referrer: Option<String>,
    /// Allowed bridges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridges: Option<BridgeOptions>,
    /// Allowed exchanges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exchanges: Option<ExchangeOptions>,
    /// Route ordering preference
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<RouteOrder>,
    /// Maximum price impact allowed (in percent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price_impact: Option<f64>,
    /// Allow unsafe transactions (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_destination_call: Option<bool>,
}

impl RoutesOptions {
    /// Create new route options
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set slippage tolerance as a decimal fraction (`0.005` = 0.5%)
    #[must_use]
    pub fn with_slippage(mut self, slippage: f64) -> Self {
        self.slippage = Some(slippage);
        self
    }

    /// Set integrator
    #[must_use]
    pub fn with_integrator(mut self, integrator: impl Into<String>) -> Self {
        self.integrator = Some(integrator.into());
        self
    }

    /// Set bridge options
    #[must_use]
    pub fn with_bridges(mut self, bridges: BridgeOptions) -> Self {
        self.bridges = Some(bridges);
        self
    }

    /// Set exchange options
    #[must_use]
    pub fn with_exchanges(mut self, exchanges: ExchangeOptions) -> Self {
        self.exchanges = Some(exchanges);
        self
    }

    /// Set route order
    #[must_use]
    pub fn with_order(mut self, order: RouteOrder) -> Self {
        self.order = Some(order);
        self
    }
}

/// Bridge options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BridgeOptions {
    /// Allowed bridges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    /// Denied bridges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
}

/// Exchange options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExchangeOptions {
    /// Allowed exchanges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    /// Denied exchanges
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
}

/// Routes response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutesResponse {
    /// Available routes
    pub routes: Vec<Route>,
    /// Unavailable routes (with reasons)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unavailable_routes: Option<UnavailableRoutes>,
}

/// Unavailable routes with reasons
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnavailableRoutes {
    /// Routes filtered by tool
    #[serde(default)]
    pub filtered_out: Vec<FilteredRoute>,
    /// Routes that failed
    #[serde(default)]
    pub failed: Vec<FailedRoute>,
}

/// A route that was filtered out
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilteredRoute {
    /// Path identifier (e.g. `1:ETH-across-42161:ETH`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overall_path: Option<String>,
    /// Reason for filtering
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// A route that failed
///
/// LI.FI reports failures per overall path, with a map of sub-path to the
/// tool errors encountered for that sub-path.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedRoute {
    /// Path identifier (e.g. `1:ETH~1:USDC-1:USDC-across-42161:USDC`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overall_path: Option<String>,
    /// Errors keyed by sub-path
    #[serde(default)]
    pub subpaths: std::collections::HashMap<String, Vec<ToolError>>,
}

/// An error returned by a tool while computing a (sub-)route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolError {
    /// Error type (e.g. `NO_QUOTE`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_type: Option<String>,
    /// Error code (e.g. `TOOL_SPECIFIC_ERROR`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Tool that produced the error
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Human-readable message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Action that was attempted (raw)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<serde_json::Value>,
}

// ============================================================================
// Route Types
// ============================================================================

/// A complete route for a cross-chain swap
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    /// Unique route ID
    pub id: String,
    /// Source chain ID
    pub from_chain_id: ChainId,
    /// Source token
    pub from_token: Token,
    /// Source amount
    pub from_amount: String,
    /// Source amount in USD
    #[serde(
        default,
        rename = "fromAmountUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_amount_usd: Option<String>,
    /// Sender address
    pub from_address: String,
    /// Destination chain ID
    pub to_chain_id: ChainId,
    /// Destination token
    pub to_token: Token,
    /// Destination amount (estimated)
    pub to_amount: String,
    /// Destination amount in USD (estimated)
    #[serde(
        default,
        rename = "toAmountUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub to_amount_usd: Option<String>,
    /// Minimum destination amount (after slippage)
    pub to_amount_min: String,
    /// Recipient address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_address: Option<String>,
    /// Route steps
    pub steps: Vec<Step>,
    /// Gas cost in USD
    #[serde(
        default,
        rename = "gasCostUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub gas_cost_usd: Option<String>,
    /// Total execution time in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_duration: Option<u64>,
    /// Tags for this route
    #[serde(default)]
    pub tags: Vec<String>,
    /// Insurance available
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub insurance: Option<Insurance>,
}

/// Insurance information for a route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Insurance {
    /// Insurance state
    pub state: String,
    /// Fee in USD
    #[serde(
        default,
        rename = "feeAmountUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub fee_amount_usd: Option<String>,
}

// ============================================================================
// Step Types
// ============================================================================

/// A single step in a route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// Step ID
    pub id: String,
    /// Step type (swap, cross, lifi)
    #[serde(rename = "type")]
    pub step_type: StepType,
    /// Tool used for this step
    pub tool: String,
    /// Tool details
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_details: Option<ToolDetails>,
    /// Action details
    pub action: Action,
    /// Estimate for this step
    pub estimate: Estimate,
    /// Transaction request (if available)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_request: Option<TransactionRequest>,
    /// Included steps (for composite steps)
    #[serde(default)]
    pub included_steps: Vec<Step>,
}

/// Type of step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepType {
    /// Same-chain swap
    Swap,
    /// Cross-chain transfer
    Cross,
    /// LI.FI composite step
    Lifi,
    /// Protocol-specific step
    Protocol,
    /// Custom step
    Custom,
}

/// Tool details
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDetails {
    /// Tool key
    pub key: String,
    /// Tool name
    pub name: String,
    /// Tool logo URI
    #[serde(default, rename = "logoURI", skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
}

// ============================================================================
// Action Types
// ============================================================================

/// Action details for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    /// Source chain ID
    pub from_chain_id: ChainId,
    /// Source token
    pub from_token: Token,
    /// Source amount
    pub from_amount: String,
    /// Destination chain ID
    pub to_chain_id: ChainId,
    /// Destination token
    pub to_token: Token,
    /// Slippage tolerance (may not be present for some steps)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage: Option<f64>,
    /// Sender address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_address: Option<String>,
    /// Recipient address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_address: Option<String>,
    /// Destination call data (for contract interactions on destination)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub destination_call_data: Option<String>,
}

// ============================================================================
// Estimate Types
// ============================================================================

/// Estimate for a step
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Estimate {
    /// Tool used for this estimate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool: Option<String>,
    /// Source amount
    pub from_amount: String,
    /// Source amount in USD
    #[serde(
        default,
        rename = "fromAmountUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_amount_usd: Option<String>,
    /// Destination amount
    pub to_amount: String,
    /// Destination amount in USD
    #[serde(
        default,
        rename = "toAmountUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub to_amount_usd: Option<String>,
    /// Minimum destination amount (after slippage)
    pub to_amount_min: String,
    /// Approval address (for token approvals)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_address: Option<String>,
    /// Execution duration in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_duration: Option<u64>,
    /// Fee costs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_costs: Option<Vec<FeeCost>>,
    /// Gas costs
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_costs: Option<Vec<GasCost>>,
    /// Data
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<EstimateData>,
    /// Whether approval needs to be reset (for tokens with non-standard approval)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval_reset: Option<bool>,
    /// Whether to skip the approval step
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_approval: Option<bool>,
    /// Whether to skip permit signing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skip_permit: Option<bool>,
}

/// Fee cost
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeCost {
    /// Fee name
    pub name: String,
    /// Fee description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Fee token
    pub token: Token,
    /// Fee amount
    pub amount: String,
    /// Fee amount in USD
    #[serde(default, rename = "amountUSD", skip_serializing_if = "Option::is_none")]
    pub amount_usd: Option<String>,
    /// Percentage fee
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub percentage: Option<String>,
    /// Whether included in source amount
    #[serde(default)]
    pub included: bool,
    /// Fee split between integrator and LI.FI
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fee_split: Option<FeeSplit>,
}

/// Fee split between integrator and LI.FI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeSplit {
    /// Integrator fee amount
    pub integrator_fee: String,
    /// LI.FI fee amount
    pub lifi_fee: String,
}

/// Gas cost
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasCost {
    /// Type of gas cost
    #[serde(rename = "type")]
    pub gas_type: String,
    /// Estimate gas
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub estimate: Option<String>,
    /// Gas limit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,
    /// Gas amount
    pub amount: String,
    /// Amount in USD
    #[serde(default, rename = "amountUSD", skip_serializing_if = "Option::is_none")]
    pub amount_usd: Option<String>,
    /// Gas price
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Token used for gas
    pub token: Token,
}

/// Additional estimate data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EstimateData {
    /// Exchange rate
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exchange_rate: Option<String>,
    /// Price impact percentage
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_impact: Option<String>,
}

// ============================================================================
// Transaction Request Types
// ============================================================================

/// Transaction request to execute on-chain
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionRequest {
    /// Contract address to call
    pub to: String,
    /// Call data
    pub data: String,
    /// ETH value to send (in wei)
    pub value: String,
    /// Sender address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Chain ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<ChainId>,
    /// Gas limit
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_limit: Option<String>,
    /// Gas price
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    /// Max fee per gas (EIP-1559)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee per gas (EIP-1559)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,
}

// ============================================================================
// Status Types
// ============================================================================

/// Status request parameters
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusRequest {
    /// Transaction hash to check
    pub tx_hash: String,
    /// Bridge used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge: Option<String>,
    /// Source chain ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_chain: Option<ChainId>,
    /// Destination chain ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_chain: Option<ChainId>,
}

impl StatusRequest {
    /// Create a new status request
    pub fn new(tx_hash: impl Into<String>) -> Self {
        Self {
            tx_hash: tx_hash.into(),
            bridge: None,
            from_chain: None,
            to_chain: None,
        }
    }

    /// Set the bridge used
    #[must_use]
    pub fn with_bridge(mut self, bridge: impl Into<String>) -> Self {
        self.bridge = Some(bridge.into());
        self
    }

    /// Set the source chain
    #[must_use]
    pub fn with_from_chain(mut self, chain: ChainId) -> Self {
        self.from_chain = Some(chain);
        self
    }

    /// Set the destination chain
    #[must_use]
    pub fn with_to_chain(mut self, chain: ChainId) -> Self {
        self.to_chain = Some(chain);
        self
    }
}

/// Status response for tracking cross-chain transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusResponse {
    /// Transaction ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transaction_id: Option<String>,
    /// Sending transaction details
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sending: Option<TransactionInfo>,
    /// Receiving transaction details
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiving: Option<TransactionInfo>,
    /// LI.FI explorer link
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lifi_explorer_link: Option<String>,
    /// Source chain ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_chain_id: Option<ChainId>,
    /// Destination chain ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_chain_id: Option<ChainId>,
    /// Bridge used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bridge: Option<String>,
    /// Overall status
    pub status: TransactionStatus,
    /// Substatus
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substatus: Option<String>,
    /// Substatus message
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub substatus_message: Option<String>,
    /// Metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<StatusMetadata>,
}

/// Transaction information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionInfo {
    /// Transaction hash
    pub tx_hash: String,
    /// Transaction link
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_link: Option<String>,
    /// Amount
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<String>,
    /// Token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token: Option<Token>,
    /// Chain ID
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<ChainId>,
    /// Gas price
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,
    /// Gas used
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gas_used: Option<String>,
    /// Gas amount in USD
    #[serde(
        default,
        rename = "gasAmountUSD",
        skip_serializing_if = "Option::is_none"
    )]
    pub gas_amount_usd: Option<String>,
    /// Timestamp
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<u64>,
}

/// Transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TransactionStatus {
    /// Transaction not started
    NotStarted,
    /// Pending
    Pending,
    /// Transaction is being processed
    Done,
    /// Failed
    Failed,
    /// Invalid
    Invalid,
    /// Not found
    NotFound,
}

/// Status metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusMetadata {
    /// Integrator
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integrator: Option<String>,
}

// ============================================================================
// Chain Types
// ============================================================================

/// Chain information from LI.FI
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chain {
    /// Chain ID
    pub id: ChainId,
    /// Chain key
    pub key: String,
    /// Chain name
    pub name: String,
    /// Coin symbol (native token)
    pub coin: String,
    /// Chain type (EVM, SVM, etc.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_type: Option<String>,
    /// Logo URI
    #[serde(default, rename = "logoURI", skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    /// Native token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_token: Option<Token>,
    /// Tokens on this chain (when fetching with tokens)
    #[serde(default)]
    pub tokens: Vec<Token>,
    /// Whether mainnet
    #[serde(default)]
    pub mainnet: bool,
    /// Multicall address
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub multicall_address: Option<String>,
    /// Metamask settings
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metamask: Option<MetamaskChainInfo>,
}

/// Metamask chain information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetamaskChainInfo {
    /// Chain ID in hex
    pub chain_id: String,
    /// Chain name
    pub chain_name: String,
    /// Native currency
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_currency: Option<NativeCurrency>,
    /// RPC URLs
    #[serde(default)]
    pub rpc_urls: Vec<String>,
    /// Block explorer URLs
    #[serde(default)]
    pub block_explorer_urls: Vec<String>,
}

/// Native currency info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeCurrency {
    /// Currency name
    pub name: String,
    /// Currency symbol
    pub symbol: String,
    /// Decimals
    pub decimals: u8,
}

/// Chains response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainsResponse {
    /// List of chains
    pub chains: Vec<Chain>,
}

// ============================================================================
// Connections Types
// ============================================================================

/// Request for available connections
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionsRequest {
    /// Source chain ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_chain: Option<ChainId>,
    /// Destination chain ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_chain: Option<ChainId>,
    /// Source token address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_token: Option<String>,
    /// Destination token address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_token: Option<String>,
    /// Allow specific bridges
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_comma_list"
    )]
    pub allow_bridges: Option<Vec<String>>,
    /// Deny specific bridges
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_comma_list"
    )]
    pub deny_bridges: Option<Vec<String>>,
}

impl ConnectionsRequest {
    /// Create a new connections request
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set source chain
    #[must_use]
    pub fn with_from_chain(mut self, chain: ChainId) -> Self {
        self.from_chain = Some(chain);
        self
    }

    /// Set destination chain
    #[must_use]
    pub fn with_to_chain(mut self, chain: ChainId) -> Self {
        self.to_chain = Some(chain);
        self
    }

    /// Set source token
    #[must_use]
    pub fn with_from_token(mut self, token: impl Into<String>) -> Self {
        self.from_token = Some(token.into());
        self
    }

    /// Set destination token
    #[must_use]
    pub fn with_to_token(mut self, token: impl Into<String>) -> Self {
        self.to_token = Some(token.into());
        self
    }
}

/// Connections response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionsResponse {
    /// Available connections
    pub connections: Vec<Connection>,
}

/// A connection between chains
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Connection {
    /// Source chain ID
    pub from_chain_id: ChainId,
    /// Destination chain ID
    pub to_chain_id: ChainId,
    /// Source tokens
    #[serde(default)]
    pub from_tokens: Vec<ConnectionToken>,
    /// Destination tokens
    #[serde(default)]
    pub to_tokens: Vec<ConnectionToken>,
}

/// Token reference in a connection.
///
/// `/v1/connections` only returns `address` and `chainId` for each token;
/// the remaining metadata is optional.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionToken {
    /// Token contract address
    pub address: String,
    /// Chain ID
    pub chain_id: ChainId,
    /// Token symbol (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    /// Token decimals (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimals: Option<u8>,
    /// Token name (if provided)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

// ============================================================================
// Tokens Types
// ============================================================================

/// Request for available tokens
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensRequest {
    /// Filter by chains (sent as a comma-separated list)
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "serialize_comma_list"
    )]
    pub chains: Option<Vec<ChainId>>,
}

impl TokensRequest {
    /// Create a new tokens request
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by specific chains
    #[must_use]
    pub fn with_chains(mut self, chains: Vec<ChainId>) -> Self {
        self.chains = Some(chains);
        self
    }
}

/// Tokens response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensResponse {
    /// Tokens organized by chain ID
    pub tokens: std::collections::HashMap<String, Vec<Token>>,
}

// ============================================================================
// Tools Types (Bridges and Exchanges)
// ============================================================================

/// Supported chain pair for a tool (from -> to)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupportedChainPair {
    /// Source chain ID
    pub from_chain_id: ChainId,
    /// Destination chain ID
    pub to_chain_id: ChainId,
}

/// Supported chain entry - can be either a pair or a single chain ID
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SupportedChainEntry {
    /// A pair of chain IDs (from -> to) for bridges
    Pair(SupportedChainPair),
    /// A single chain ID for exchanges
    Single(ChainId),
}

/// Tool information (bridge or exchange)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    /// Tool key
    pub key: String,
    /// Tool name
    pub name: String,
    /// Tool type (optional, not always present in API response)
    #[serde(rename = "type", default)]
    pub tool_type: Option<ToolType>,
    /// Logo URI
    #[serde(default, rename = "logoURI", skip_serializing_if = "Option::is_none")]
    pub logo_uri: Option<String>,
    /// Supported chains (can be chain pairs for bridges or single IDs for exchanges)
    #[serde(default)]
    pub supported_chains: Vec<SupportedChainEntry>,
}

/// Tool type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    /// Bridge
    Bridge,
    /// Exchange/DEX
    Exchange,
}

/// Tools response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsResponse {
    /// Bridges
    #[serde(default)]
    pub bridges: Vec<Tool>,
    /// Exchanges
    #[serde(default)]
    pub exchanges: Vec<Tool>,
}

// ============================================================================
// Gas Prices Types
// ============================================================================

/// Gas price information for a single chain
///
/// Gas prices are returned as integers in wei.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GasPrice {
    /// Standard gas price (wei)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub standard: Option<u64>,
    /// Fast gas price (wei)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fast: Option<u64>,
    /// Fastest gas price (wei)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fastest: Option<u64>,
    /// Last updated timestamp (unix seconds)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<u64>,
}

/// Gas prices response - map of chain ID to gas price
///
/// The keys are chain IDs as strings (e.g., "1" for Ethereum, "10" for Optimism).
pub type GasPricesResponse = std::collections::HashMap<String, GasPrice>;

// ============================================================================
// Error Types
// ============================================================================

/// API error response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    /// Error message
    pub message: String,
    /// Error code (LI.FI returns numeric codes, e.g. `1001`)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub code: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn query_string<Q: Serialize>(q: &Q) -> String {
        reqwest::Client::new()
            .get("https://example.invalid/x")
            .query(q)
            .build()
            .expect("query must serialize")
            .url()
            .query()
            .unwrap_or_default()
            .to_string()
    }

    #[test]
    fn slippage_percent_converts_to_fraction() {
        assert!((slippage_percent_to_fraction(0.5).unwrap() - 0.005).abs() < 1e-12);
        assert!((slippage_percent_to_fraction(3.0).unwrap() - 0.03).abs() < 1e-12);
        assert!(slippage_percent_to_fraction(0.0).is_err());
        assert!(slippage_percent_to_fraction(-1.0).is_err());
        assert!(slippage_percent_to_fraction(100.0).is_err());
        assert!(slippage_percent_to_fraction(f64::NAN).is_err());
    }

    #[test]
    fn slippage_fraction_validation() {
        assert!(validate_slippage_fraction(0.005).is_ok());
        assert!(validate_slippage_fraction(0.0).is_err());
        assert!(validate_slippage_fraction(1.0).is_err());
    }

    #[test]
    fn token_uses_lifi_field_names() {
        let json = r#"{"address":"0x0000000000000000000000000000000000000000","chainId":1,
            "symbol":"ETH","decimals":18,"name":"ETH","coinKey":"ETH",
            "logoURI":"https://logo","priceUSD":"2692.34","tags":[]}"#;
        let token: Token = serde_json::from_str(json).unwrap();
        assert_eq!(token.price_usd.as_deref(), Some("2692.34"));
        assert_eq!(token.logo_uri.as_deref(), Some("https://logo"));
        let out = serde_json::to_value(&token).unwrap();
        assert_eq!(out["priceUSD"], "2692.34");
        assert_eq!(out["logoURI"], "https://logo");
        assert!(out.get("priceUsd").is_none());
    }

    #[test]
    fn estimate_usd_fields_parse() {
        let json = r#"{"fromAmount":"1","fromAmountUSD":"2692.34","toAmount":"2","toAmountUSD":"2686.5",
            "toAmountMin":"1","gasCosts":[{"type":"SEND","amount":"1","amountUSD":"0.5",
            "token":{"address":"0x0","chainId":1,"symbol":"ETH","decimals":18,"name":"ETH"}}]}"#;
        let est: Estimate = serde_json::from_str(json).unwrap();
        assert_eq!(est.from_amount_usd.as_deref(), Some("2692.34"));
        assert_eq!(est.to_amount_usd.as_deref(), Some("2686.5"));
        assert_eq!(est.gas_costs.unwrap()[0].amount_usd.as_deref(), Some("0.5"));
    }

    #[test]
    fn unavailable_routes_parse_live_shape() {
        let json = r#"{
            "filteredOut":[{"overallPath":"1:ETH-across-42161:ETH","reason":"Across does not send ETH to contracts"}],
            "failed":[{"overallPath":"1:ETH~1:USDC-1:USDC-across-42161:USDC",
                "subpaths":{"1:ETH~1:USDC":[{"errorType":"NO_QUOTE","code":"TOOL_SPECIFIC_ERROR",
                "tool":"lifiIntentsDex","message":"empty quote","durationMs":28,"action":{"fromChainId":1}}]}}]
        }"#;
        let u: UnavailableRoutes = serde_json::from_str(json).unwrap();
        assert_eq!(u.filtered_out.len(), 1);
        let failed = &u.failed[0];
        let errs = &failed.subpaths["1:ETH~1:USDC"];
        assert_eq!(errs[0].tool.as_deref(), Some("lifiIntentsDex"));
        assert_eq!(errs[0].code.as_deref(), Some("TOOL_SPECIFIC_ERROR"));
    }

    #[test]
    fn connection_tokens_accept_address_only() {
        let json = r#"{"fromChainId":1,"toChainId":42161,
            "fromTokens":[{"address":"0x0000000000000000000000000000000000000000","chainId":1}],
            "toTokens":[{"address":"0xaf88d065e77c8cC2239327C5EDb3A432268e5831","chainId":42161}]}"#;
        let c: Connection = serde_json::from_str(json).unwrap();
        assert_eq!(c.from_tokens[0].chain_id, 1);
        assert!(c.to_tokens[0].symbol.is_none());
    }

    #[test]
    fn tokens_request_serializes_comma_joined_chains() {
        let q = query_string(&TokensRequest::new().with_chains(vec![1, 8453]));
        assert_eq!(q, "chains=1%2C8453");
        assert_eq!(query_string(&TokensRequest::new()), "");
    }

    #[test]
    fn quote_request_serializes_fraction_and_lists() {
        let mut req = QuoteRequest::new(1, 42161, "0xa", "0xb", "100", "0xc").with_slippage(0.005);
        req.allow_bridges = Some(vec!["hop".into(), "across".into()]);
        let q = query_string(&req);
        assert!(q.contains("slippage=0.005"), "{q}");
        assert!(q.contains("allowBridges=hop%2Cacross"), "{q}");
    }

    #[test]
    fn api_error_accepts_numeric_code() {
        let e: ApiError = serde_json::from_str(r#"{"message":"No routes","code":1001}"#).unwrap();
        assert_eq!(e.message, "No routes");
    }
}
