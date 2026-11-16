//! Types for the Enso Finance API
//!
//! This module contains request and response types for the Enso Finance API,
//! including routing, bundling, and position management.

use serde::{Deserialize, Serialize};

/// `amountOut` is a string for single-token routes, a list of strings when
/// several `tokenOut` are requested, and null for async routes. Kept as one
/// string: a list is joined with commas in `tokenOut` order, null is empty.
fn amount_out<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(String::new()),
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => Ok(s),
                serde_json::Value::Number(n) => Ok(n.to_string()),
                other => Err(D::Error::custom(format!("expected an amount, got {other}"))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|amounts| amounts.join(",")),
        other => Err(D::Error::custom(format!("expected an amount, got {other}"))),
    }
}

/// [`amount_out`] for optional amounts, where null is `None`
fn opt_amount_out<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(v) => amount_out(v).map(Some).map_err(serde::de::Error::custom),
    }
}

/// A USD price Enso documents as a nullable string; numbers are accepted too
fn opt_price<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    match serde_json::Value::deserialize(deserializer)? {
        serde_json::Value::Null => Ok(None),
        serde_json::Value::Number(n) => Ok(n.as_f64()),
        serde_json::Value::String(s) if s.trim().is_empty() => Ok(None),
        serde_json::Value::String(s) => s.trim().parse().map(Some).map_err(D::Error::custom),
        other => Err(D::Error::custom(format!("expected a price, got {other}"))),
    }
}

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

/// Routing strategy for Enso (API values: `router`, `delegate`,
/// `ensowallet-v2`, `router-legacy`, `delegate-legacy`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RoutingStrategy {
    /// Use the router contract directly
    #[default]
    Router,
    /// Use delegate call through smart wallet
    Delegate,
    /// Use Enso smart wallet (`ensowallet-v2`)
    #[serde(rename = "ensowallet-v2", alias = "ensowallet")]
    Ensowallet,
    /// Legacy router
    RouterLegacy,
    /// Legacy delegate
    DelegateLegacy,
}

impl RoutingStrategy {
    /// API string value
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Router => "router",
            Self::Delegate => "delegate",
            Self::Ensowallet => "ensowallet-v2",
            Self::RouterLegacy => "router-legacy",
            Self::DelegateLegacy => "delegate-legacy",
        }
    }
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteResponse {
    /// Estimated output amount. With several `tokenOut`, the amounts in
    /// that order, comma-separated; empty for async routes (Enso sends null)
    #[serde(deserialize_with = "amount_out")]
    pub amount_out: String,
    /// Minimum output amount after slippage
    #[serde(default, alias = "amount_out_min", deserialize_with = "opt_amount_out")]
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
    /// Transactions that must be sent before `tx` (e.g. token approvals)
    #[serde(default)]
    pub pre_transactions: Vec<PreTransaction>,
}

/// Transaction that must be executed before the main route/bundle `tx`
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreTransaction {
    /// Pre-transaction type (e.g. `approve`)
    #[serde(rename = "type")]
    pub kind: String,
    /// Transaction to send
    pub tx: TransactionData,
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
    /// Token involved (e.g. the token being approved)
    #[serde(default)]
    pub token: Option<String>,
    /// Spender being approved
    #[serde(default)]
    pub spender: Option<String>,
    /// Amount (smallest units)
    #[serde(default)]
    pub amount: Option<String>,
}

/// Approval transaction returned by the bundle endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleApproval {
    /// Sender address
    pub from: String,
    /// Token contract to call
    pub to: String,
    /// Encoded approve calldata
    pub data: String,
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionData {
    /// Target contract address
    pub to: String,
    /// Sender address
    pub from: String,
    /// Encoded calldata
    pub data: String,
    /// Value to send (for native token swaps); empty when Enso leaves it out
    #[serde(default)]
    pub value: String,
}

/// Route step in the swap path
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
///
/// `POST /api/v1/shortcuts/bundle` takes `chainId`, `fromAddress` and
/// `routingStrategy` as **query parameters** and the bare `actions` array as
/// the JSON body; see [`BundleRequest::query_params`] and
/// [`BundleRequest::body`].
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

    /// Query parameters for the bundle endpoint
    #[must_use]
    pub fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("chainId", self.chain_id.to_string()),
            ("fromAddress", self.from_address.clone()),
        ];
        if let Some(strategy) = self.routing_strategy {
            params.push(("routingStrategy", strategy.as_str().to_string()));
        }
        params
    }

    /// JSON body for the bundle endpoint (the bare actions array)
    #[must_use]
    pub fn body(&self) -> &Vec<BundleAction> {
        &self.actions
    }
}

/// Bundle response
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Approvals that must be sent before `tx`
    #[serde(default)]
    pub approvals: Vec<BundleApproval>,
    /// Transactions that must be sent before `tx`
    #[serde(default)]
    pub pre_transactions: Vec<PreTransaction>,
}

/// Token price information
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalance {
    /// Token address
    #[serde(alias = "token")]
    pub address: String,
    /// Balance in smallest units
    #[serde(alias = "amount")]
    pub balance: String,
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token name
    #[serde(default)]
    pub name: Option<String>,
    /// Token decimals
    #[serde(default)]
    pub decimals: Option<u8>,
    /// Chain ID
    #[serde(default)]
    pub chain_id: Option<u64>,
    /// USD price of one whole token (not the value of the balance)
    #[serde(default, alias = "usd_value", deserialize_with = "opt_price")]
    pub price: Option<f64>,
    /// Logo URI
    #[serde(default)]
    pub logo_uri: Option<String>,
}

/// A list of strings Enso documents as an array but sometimes gives as a
/// single string; null is `None`
fn opt_string_list<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    match Option::<serde_json::Value>::deserialize(deserializer)? {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(vec![s])),
        Some(serde_json::Value::Array(items)) => items
            .into_iter()
            .map(|v| match v {
                serde_json::Value::String(s) => Ok(s),
                other => Err(D::Error::custom(format!("expected a string, got {other}"))),
            })
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
        Some(other) => Err(D::Error::custom(format!(
            "expected a string list, got {other}"
        ))),
    }
}

/// Append each value of a list query parameter as a repeated key
fn push_list(params: &mut Vec<(&'static str, String)>, key: &'static str, values: &[String]) {
    params.extend(values.iter().map(|v| (key, v.clone())));
}

/// A network Enso supports (`GET /api/v1/networks`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Network {
    /// Chain ID
    pub id: u64,
    /// Network name
    pub name: String,
    /// Whether Enso routes on this network
    #[serde(default)]
    pub is_connected: bool,
}

/// A chain a protocol or action is available on
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChainRef {
    /// Chain ID
    pub id: u64,
    /// Network name
    #[serde(default)]
    pub name: String,
}

/// A protocol integration (`GET /api/v1/protocols`, `/standards/{slug}`,
/// `/projects/{project}/protocols`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Protocol {
    /// Overarching project (e.g. `uniswap`)
    pub project: String,
    /// Protocol slug (e.g. `uniswap-v2`)
    pub slug: String,
    /// Display name
    #[serde(default)]
    pub name: Option<String>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Website
    #[serde(default)]
    pub url: Option<String>,
    /// Logo URIs (Enso's spec example gives a single string; both are read)
    #[serde(default, deserialize_with = "opt_string_list")]
    pub logos_uri: Option<Vec<String>>,
    /// Chains the protocol is available on
    #[serde(default)]
    pub chains: Option<Vec<ChainRef>>,
}

/// A project and its protocols (`GET /api/v1/projects`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Project {
    /// Project ID (e.g. `aave`)
    pub id: String,
    /// Chain IDs the project is on
    #[serde(default)]
    pub chains: Vec<u64>,
    /// Protocol slugs of the project
    #[serde(default)]
    pub protocols: Vec<String>,
}

/// Slug and URL of a standard's protocol or fork
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct StandardProtocol {
    /// Protocol slug
    pub slug: String,
    /// Website
    #[serde(default)]
    pub url: String,
}

/// An action a standard supports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StandardAction {
    /// Action (e.g. `deposit`)
    pub action: String,
    /// Action name
    #[serde(default)]
    pub name: String,
    /// Contract functions the action calls
    #[serde(default)]
    pub function_names: Vec<String>,
    /// Chains the action is available on
    #[serde(default)]
    pub supported_chains: Vec<ChainRef>,
    /// Input names
    #[serde(default)]
    pub inputs: Vec<String>,
}

/// A standard: a protocol, its forks and the actions they support
/// (`GET /api/v1/standards`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Standard {
    /// The protocol
    pub protocol: StandardProtocol,
    /// Forks of the protocol using the same integration
    #[serde(default)]
    pub forks: Vec<StandardProtocol>,
    /// Supported actions
    #[serde(default)]
    pub actions: Vec<StandardAction>,
}

/// Metadata of one action input
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ActionInput {
    /// Input type (e.g. `address`, `uint256`)
    #[serde(rename = "type")]
    pub kind: String,
    /// Description
    #[serde(default)]
    pub description: String,
    /// Whether the input may be left out
    #[serde(default)]
    pub optional: Option<bool>,
}

/// A variant of an action
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ActionVariant {
    /// Action
    pub action: String,
    /// Inputs by name
    #[serde(default)]
    pub inputs: std::collections::BTreeMap<String, ActionInput>,
    /// Bundle name
    #[serde(default)]
    pub bundle_name: Option<String>,
}

/// A bundle action and its inputs (`GET /api/v1/actions`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Action {
    /// Action (e.g. `swap`)
    pub action: String,
    /// Inputs by name
    #[serde(default)]
    pub inputs: std::collections::BTreeMap<String, ActionInput>,
    /// Variants of the action
    #[serde(default)]
    pub variants: Vec<ActionVariant>,
}

/// Enso smart wallet of an address (`GET /api/v1/wallet`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Wallet {
    /// Smart wallet address
    pub address: String,
    /// Whether the wallet contract is deployed
    #[serde(default)]
    pub is_deployed: bool,
}

/// Approval transaction (`GET /api/v1/wallet/approve`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApproveTransaction {
    /// Transaction to send
    pub tx: TransactionData,
    /// Gas estimate
    #[serde(default, deserialize_with = "opt_amount_out")]
    pub gas: Option<String>,
    /// Token being approved
    #[serde(default)]
    pub token: Option<String>,
    /// Amount approved (smallest units)
    #[serde(default, deserialize_with = "opt_amount_out")]
    pub amount: Option<String>,
    /// Spender being approved
    #[serde(default)]
    pub spender: Option<String>,
}

/// Quote request (`GET /api/v1/shortcuts/quote`)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct QuoteRequest {
    /// Chain ID
    pub chain_id: u64,
    /// Sender address
    pub from_address: Option<String>,
    /// Routing strategy
    pub routing_strategy: Option<RoutingStrategy>,
    /// Input token addresses
    pub token_in: Vec<String>,
    /// Output token addresses
    pub token_out: Vec<String>,
    /// Input amounts (smallest units), in `token_in` order
    pub amount_in: Vec<String>,
    /// Aggregators to leave out (e.g. `1inch`)
    pub ignore_aggregators: Vec<String>,
    /// Standards to leave out
    pub ignore_standards: Vec<String>,
    /// Latency/quality trade-off: `light`, `medium`, `high` or `max`
    pub effort: Option<String>,
    /// Swaps the quote may use: `any`, `rfq` or `amm`
    pub swap_type: Option<String>,
}

impl QuoteRequest {
    /// Quote for swapping `amount_in` of `token_in` to `token_out`
    #[must_use]
    pub fn new(
        chain_id: u64,
        token_in: impl Into<String>,
        token_out: impl Into<String>,
        amount_in: impl Into<String>,
    ) -> Self {
        Self {
            chain_id,
            token_in: vec![token_in.into()],
            token_out: vec![token_out.into()],
            amount_in: vec![amount_in.into()],
            ..Self::default()
        }
    }

    /// Set the sender address
    #[must_use]
    pub fn with_from_address(mut self, from_address: impl Into<String>) -> Self {
        self.from_address = Some(from_address.into());
        self
    }

    /// Set the routing strategy
    #[must_use]
    pub fn with_routing_strategy(mut self, strategy: RoutingStrategy) -> Self {
        self.routing_strategy = Some(strategy);
        self
    }

    /// Several input tokens, with their amounts in the same order
    #[must_use]
    pub fn with_tokens_in(mut self, tokens: Vec<String>, amounts: Vec<String>) -> Self {
        self.token_in = tokens;
        self.amount_in = amounts;
        self
    }

    /// Several output tokens
    #[must_use]
    pub fn with_tokens_out(mut self, tokens: Vec<String>) -> Self {
        self.token_out = tokens;
        self
    }

    /// Leave these aggregators out
    #[must_use]
    pub fn with_ignore_aggregators(mut self, aggregators: Vec<String>) -> Self {
        self.ignore_aggregators = aggregators;
        self
    }

    /// Leave these standards out
    #[must_use]
    pub fn with_ignore_standards(mut self, standards: Vec<String>) -> Self {
        self.ignore_standards = standards;
        self
    }

    /// Set the effort (`light`, `medium`, `high` or `max`)
    #[must_use]
    pub fn with_effort(mut self, effort: impl Into<String>) -> Self {
        self.effort = Some(effort.into());
        self
    }

    /// Set the swap type (`any`, `rfq` or `amm`)
    #[must_use]
    pub fn with_swap_type(mut self, swap_type: impl Into<String>) -> Self {
        self.swap_type = Some(swap_type.into());
        self
    }

    /// Query parameters; lists are sent as repeated keys
    #[must_use]
    pub fn query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![("chainId", self.chain_id.to_string())];
        if let Some(from) = &self.from_address {
            params.push(("fromAddress", from.clone()));
        }
        if let Some(strategy) = self.routing_strategy {
            params.push(("routingStrategy", strategy.as_str().to_string()));
        }
        push_list(&mut params, "tokenIn", &self.token_in);
        push_list(&mut params, "tokenOut", &self.token_out);
        push_list(&mut params, "amountIn", &self.amount_in);
        push_list(&mut params, "ignoreAggregators", &self.ignore_aggregators);
        push_list(&mut params, "ignoreStandards", &self.ignore_standards);
        if let Some(effort) = &self.effort {
            params.push(("effort", effort.clone()));
        }
        if let Some(swap_type) = &self.swap_type {
            params.push(("swapType", swap_type.clone()));
        }
        params
    }
}

/// Quote response (no transaction)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteResponse {
    /// Estimated output amount. With several `tokenOut`, the amounts in
    /// that order, comma-separated
    #[serde(default, deserialize_with = "amount_out")]
    pub amount_out: String,
    /// Price impact in basis points; `None` when Enso has no USD price
    #[serde(default, deserialize_with = "opt_price")]
    pub price_impact: Option<f64>,
    /// Route steps
    #[serde(default)]
    pub route: Vec<RouteStep>,
    /// Fee collected for each `tokenIn`
    #[serde(default)]
    pub fee_amount: Vec<String>,
    /// Enso fee collected for each `tokenIn`
    #[serde(default)]
    pub enso_fee_amount: Vec<String>,
}

/// Filters for `GET /api/v1/tokens`; every field is optional
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct TokensRequest {
    /// Chain ID
    pub chain_id: Option<u64>,
    /// Project (e.g. `aave`)
    pub project: Option<String>,
    /// Protocol slug (e.g. `aave-v2`)
    pub protocol_slug: Option<String>,
    /// Token addresses
    pub address: Vec<String>,
    /// Primary (interaction) addresses of `DeFi` tokens
    pub primary_address: Vec<String>,
    /// Tokens with any of these underlying tokens
    pub underlying_tokens: Vec<String>,
    /// Tokens with exactly these underlying tokens
    pub underlying_tokens_exact: Vec<String>,
    /// Token names
    pub name: Vec<String>,
    /// Token symbols
    pub symbol: Vec<String>,
    /// Token type: `defi` or `base`
    pub token_type: Option<String>,
    /// Liquidity type: `SingleToken` or `MultiToken`
    pub liquidity_type: Option<String>,
    /// Page number
    pub page: Option<u64>,
    /// Page size (max 1000)
    pub page_size: Option<u64>,
    /// Pagination cursor
    pub cursor: Option<u64>,
    /// Include symbol, name and logos
    pub include_metadata: Option<bool>,
    /// Include underlying tokens
    pub include_underlying: Option<bool>,
    /// Minimum APY
    pub apy_from: Option<f64>,
    /// Maximum APY
    pub apy_to: Option<f64>,
    /// Minimum TVL
    pub tvl_from: Option<f64>,
    /// Maximum TVL
    pub tvl_to: Option<f64>,
}

impl TokensRequest {
    /// No filters
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only this chain
    #[must_use]
    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Only this project
    #[must_use]
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Only this protocol
    #[must_use]
    pub fn with_protocol_slug(mut self, slug: impl Into<String>) -> Self {
        self.protocol_slug = Some(slug.into());
        self
    }

    /// Only these token addresses
    #[must_use]
    pub fn with_addresses(mut self, addresses: Vec<String>) -> Self {
        self.address = addresses;
        self
    }

    /// Only these symbols
    #[must_use]
    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbol = symbols;
        self
    }

    /// Only tokens with any of these underlying tokens
    #[must_use]
    pub fn with_underlying_tokens(mut self, tokens: Vec<String>) -> Self {
        self.underlying_tokens = tokens;
        self
    }

    /// Only this token type (`defi` or `base`)
    #[must_use]
    pub fn with_token_type(mut self, token_type: impl Into<String>) -> Self {
        self.token_type = Some(token_type.into());
        self
    }

    /// Include symbol, name and logos
    #[must_use]
    pub fn with_metadata(mut self, include: bool) -> Self {
        self.include_metadata = Some(include);
        self
    }

    /// Page number and size
    #[must_use]
    pub fn with_page(mut self, page: u64, page_size: u64) -> Self {
        self.page = Some(page);
        self.page_size = Some(page_size);
        self
    }

    /// Continue from a pagination cursor
    #[must_use]
    pub fn with_cursor(mut self, cursor: u64) -> Self {
        self.cursor = Some(cursor);
        self
    }

    /// Query parameters; lists are sent as repeated keys
    #[must_use]
    pub fn query_params(&self) -> Vec<(&'static str, String)> {
        let scalars = [
            ("chainId", self.chain_id.map(|v| v.to_string())),
            ("project", self.project.clone()),
            ("protocolSlug", self.protocol_slug.clone()),
            ("type", self.token_type.clone()),
            ("liquidityType", self.liquidity_type.clone()),
            ("page", self.page.map(|v| v.to_string())),
            ("pageSize", self.page_size.map(|v| v.to_string())),
            ("cursor", self.cursor.map(|v| v.to_string())),
            (
                "includeMetadata",
                self.include_metadata.map(|v| v.to_string()),
            ),
            (
                "includeUnderlying",
                self.include_underlying.map(|v| v.to_string()),
            ),
            ("apyFrom", self.apy_from.map(|v| v.to_string())),
            ("apyTo", self.apy_to.map(|v| v.to_string())),
            ("tvlFrom", self.tvl_from.map(|v| v.to_string())),
            ("tvlTo", self.tvl_to.map(|v| v.to_string())),
        ];
        let mut params: Vec<(&'static str, String)> = scalars
            .into_iter()
            .filter_map(|(k, v)| v.map(|v| (k, v)))
            .collect();
        push_list(&mut params, "address", &self.address);
        push_list(&mut params, "primaryAddress", &self.primary_address);
        push_list(&mut params, "underlyingTokens", &self.underlying_tokens);
        push_list(
            &mut params,
            "underlyingTokensExact",
            &self.underlying_tokens_exact,
        );
        push_list(&mut params, "name", &self.name);
        push_list(&mut params, "symbol", &self.symbol);
        params
    }
}

/// Pagination metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PaginationMeta {
    /// Total (Enso documents it as the number of pages)
    #[serde(default)]
    pub total: u64,
    /// Last page number
    #[serde(default)]
    pub last_page: Option<u64>,
    /// Current page number
    #[serde(default)]
    pub current_page: Option<u64>,
    /// Items per page
    #[serde(default)]
    pub per_page: u64,
    /// Previous page
    #[serde(default)]
    pub prev: Option<u64>,
    /// Next page
    #[serde(default)]
    pub next: Option<u64>,
    /// Cursor for the next page
    #[serde(default)]
    pub cursor: Option<u64>,
}

/// An underlying token of a `DeFi` token
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UnderlyingToken {
    /// Token address
    pub address: String,
    /// Chain ID
    pub chain_id: u64,
    /// Token type (`base` or `defi`)
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// Decimals
    #[serde(default)]
    pub decimals: Option<u8>,
    /// Symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Name
    #[serde(default)]
    pub name: Option<String>,
    /// Logo URIs
    #[serde(default, deserialize_with = "opt_string_list")]
    pub logos_uri: Option<Vec<String>>,
}

/// A token Enso knows, base or `DeFi` (`GET /api/v1/tokens`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TokenInfo {
    /// Token address
    pub address: String,
    /// Chain ID
    pub chain_id: u64,
    /// Token type (`base` or `defi`)
    #[serde(rename = "type", default)]
    pub kind: Option<String>,
    /// Decimals
    #[serde(default)]
    pub decimals: Option<u8>,
    /// Symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Name
    #[serde(default)]
    pub name: Option<String>,
    /// Logo URIs
    #[serde(default, deserialize_with = "opt_string_list")]
    pub logos_uri: Option<Vec<String>>,
    /// Underlying tokens of a `DeFi` token
    #[serde(default)]
    pub underlying_tokens: Option<Vec<UnderlyingToken>>,
    /// Project (e.g. `aave`)
    #[serde(default)]
    pub project: Option<String>,
    /// Protocol slug (e.g. `aave-v2`)
    #[serde(default)]
    pub protocol: Option<String>,
    /// Deposit liquidity type
    #[serde(default)]
    pub deposit_type: Option<u64>,
    /// Redeem liquidity type
    #[serde(default)]
    pub redeem_type: Option<u64>,
    /// APY (percent)
    #[serde(default, deserialize_with = "opt_price")]
    pub apy: Option<f64>,
    /// Base APY (percent)
    #[serde(default, deserialize_with = "opt_price")]
    pub apy_base: Option<f64>,
    /// Reward APY (percent)
    #[serde(default, deserialize_with = "opt_price")]
    pub apy_reward: Option<f64>,
    /// Base borrow APY (percent)
    #[serde(default, deserialize_with = "opt_price")]
    pub borrow_apy_base: Option<f64>,
    /// Reward borrow APY (percent)
    #[serde(default, deserialize_with = "opt_price")]
    pub borrow_apy_reward: Option<f64>,
    /// TVL (USD)
    #[serde(default, deserialize_with = "opt_price")]
    pub tvl: Option<f64>,
    /// Address to interact with for a `DeFi` token
    #[serde(default)]
    pub primary_address: Option<String>,
}

/// A page of tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TokensPage {
    /// Pagination metadata
    #[serde(default)]
    pub meta: PaginationMeta,
    /// Tokens on this page
    #[serde(default)]
    pub data: Vec<TokenInfo>,
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
    #[test]
    fn route_amount_out_reads_string_list_and_null() {
        #[derive(serde::Deserialize)]
        struct A {
            #[serde(deserialize_with = "super::amount_out")]
            amount_out: String,
        }
        let a: A = serde_json::from_str(r#"{"amount_out":"100"}"#).unwrap();
        assert_eq!(a.amount_out, "100");
        let a: A = serde_json::from_str(r#"{"amount_out":["100","200"]}"#).unwrap();
        assert_eq!(a.amount_out, "100,200");
        let a: A = serde_json::from_str(r#"{"amount_out":null}"#).unwrap();
        assert_eq!(a.amount_out, "");
    }

    #[test]
    fn balance_price_reads_string_number_and_null() {
        let b: TokenBalance =
            serde_json::from_str(r#"{"token":"0x1","amount":"5","price":"1.25"}"#).unwrap();
        assert_eq!(b.price, Some(1.25));
        let b: TokenBalance =
            serde_json::from_str(r#"{"token":"0x1","amount":"5","price":2}"#).unwrap();
        assert_eq!(b.price, Some(2.0));
        let b: TokenBalance =
            serde_json::from_str(r#"{"token":"0x1","amount":"5","price":null}"#).unwrap();
        assert_eq!(b.price, None);
    }

    use super::*;

    #[test]
    fn route_response_keeps_pre_transactions() {
        let route: RouteResponse = serde_json::from_value(serde_json::json!({
            "amountOut": "990",
            "gas": "100000",
            "route": [],
            "createdAt": 1,
            "tx": {"to": "0x1111111111111111111111111111111111111111",
                "from": "0x2222222222222222222222222222222222222222",
                "data": "0x", "value": "0"},
            "preTransactions": [{
                "type": "approve",
                "description": "Approve token",
                "token": "0x3333333333333333333333333333333333333333",
                "spender": "0x1111111111111111111111111111111111111111",
                "amount": "1000",
                "tx": {"to": "0x3333333333333333333333333333333333333333",
                    "from": "0x2222222222222222222222222222222222222222",
                    "data": "0x095ea7b3", "value": "0"}
            }]
        }))
        .unwrap();
        assert_eq!(route.pre_transactions.len(), 1);
        assert_eq!(route.pre_transactions[0].kind, "approve");
        assert_eq!(route.pre_transactions[0].tx.data, "0x095ea7b3");
        let json = serde_json::to_value(&route).unwrap();
        assert_eq!(json["preTransactions"][0]["type"], "approve");
    }

    #[test]
    fn bundle_response_keeps_approvals() {
        let bundle: BundleResponse = serde_json::from_value(serde_json::json!({
            "tx": {"to": "0x1111111111111111111111111111111111111111",
                "from": "0x2222222222222222222222222222222222222222",
                "data": "0x", "value": "0"},
            "approvals": [{"from": "0x2222222222222222222222222222222222222222",
                "to": "0x3333333333333333333333333333333333333333",
                "data": "0x095ea7b3", "description": "Approve"}]
        }))
        .unwrap();
        assert_eq!(bundle.approvals.len(), 1);
        assert_eq!(
            bundle.approvals[0].to,
            "0x3333333333333333333333333333333333333333"
        );
        assert!(bundle.pre_transactions.is_empty());
    }

    #[test]
    fn balance_price_is_unit_price() {
        let balance: TokenBalance = serde_json::from_value(serde_json::json!({
            "token": "0x3333333333333333333333333333333333333333",
            "amount": "2000000",
            "chainId": 1,
            "decimals": 6,
            "price": 1.0,
            "name": "Token",
            "symbol": "TKN",
            "logoUri": "https://example.invalid/t.png"
        }))
        .unwrap();
        assert_eq!(balance.price, Some(1.0));
        assert_eq!(
            balance.logo_uri.as_deref(),
            Some("https://example.invalid/t.png")
        );
    }

    #[test]
    fn test_bundle_request_splits_query_and_body() {
        let req = BundleRequest::new(
            1,
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            vec![BundleAction::swap("0xa", "0xb", "100")],
        )
        .with_routing_strategy(RoutingStrategy::Router);
        assert_eq!(
            req.query_params(),
            vec![
                ("chainId", "1".to_string()),
                (
                    "fromAddress",
                    "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string()
                ),
                ("routingStrategy", "router".to_string()),
            ]
        );
        let body = serde_json::to_value(req.body()).unwrap();
        assert!(body.is_array(), "body must be the bare actions array");
        assert_eq!(body[0]["protocol"], "enso");
        assert_eq!(body[0]["action"], "route");
    }

    #[test]
    fn test_routing_strategy_values() {
        assert_eq!(
            serde_json::to_value(RoutingStrategy::Ensowallet).unwrap(),
            "ensowallet-v2"
        );
        assert_eq!(
            serde_json::to_value(RoutingStrategy::RouterLegacy).unwrap(),
            "router-legacy"
        );
        assert_eq!(RoutingStrategy::DelegateLegacy.as_str(), "delegate-legacy");
    }

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
