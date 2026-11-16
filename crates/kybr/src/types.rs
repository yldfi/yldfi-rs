//! Types for the `KyberSwap` API responses

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

/// Chains served by the `KyberSwap` aggregator API
///
/// Matches the active chains in `KyberSwap`'s supported-chains list
/// (`common-service.kyberswap.com/api/v1/aggregator/supported-chains`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Chain {
    Ethereum,
    Bsc,
    Polygon,
    Arbitrum,
    Optimism,
    Avalanche,
    Base,
    Linea,
    Sonic,
    Berachain,
    Unichain,
    Ronin,
    HyperEvm,
    Plasma,
    Monad,
    Etherlink,
    MegaEth,
    Robinhood,
    Arc,
}

impl Chain {
    /// All supported chains
    #[must_use]
    pub const fn all() -> &'static [Chain] {
        &[
            Chain::Ethereum,
            Chain::Bsc,
            Chain::Polygon,
            Chain::Arbitrum,
            Chain::Optimism,
            Chain::Avalanche,
            Chain::Base,
            Chain::Linea,
            Chain::Sonic,
            Chain::Berachain,
            Chain::Unichain,
            Chain::Ronin,
            Chain::HyperEvm,
            Chain::Plasma,
            Chain::Monad,
            Chain::Etherlink,
            Chain::MegaEth,
            Chain::Robinhood,
            Chain::Arc,
        ]
    }

    /// Convert from EVM chain ID
    #[must_use]
    pub fn from_chain_id(chain_id: u64) -> Option<Self> {
        match chain_id {
            1 => Some(Chain::Ethereum),
            56 => Some(Chain::Bsc),
            137 => Some(Chain::Polygon),
            42161 => Some(Chain::Arbitrum),
            10 => Some(Chain::Optimism),
            43114 => Some(Chain::Avalanche),
            8453 => Some(Chain::Base),
            59144 => Some(Chain::Linea),
            146 => Some(Chain::Sonic),
            80094 => Some(Chain::Berachain),
            130 => Some(Chain::Unichain),
            2020 => Some(Chain::Ronin),
            999 => Some(Chain::HyperEvm),
            9745 => Some(Chain::Plasma),
            143 => Some(Chain::Monad),
            42793 => Some(Chain::Etherlink),
            4326 => Some(Chain::MegaEth),
            4663 => Some(Chain::Robinhood),
            5042 => Some(Chain::Arc),
            _ => None,
        }
    }

    /// EVM chain ID
    #[must_use]
    pub const fn chain_id(&self) -> u64 {
        match self {
            Chain::Ethereum => 1,
            Chain::Bsc => 56,
            Chain::Polygon => 137,
            Chain::Arbitrum => 42161,
            Chain::Optimism => 10,
            Chain::Avalanche => 43114,
            Chain::Base => 8453,
            Chain::Linea => 59144,
            Chain::Sonic => 146,
            Chain::Berachain => 80094,
            Chain::Unichain => 130,
            Chain::Ronin => 2020,
            Chain::HyperEvm => 999,
            Chain::Plasma => 9745,
            Chain::Monad => 143,
            Chain::Etherlink => 42793,
            Chain::MegaEth => 4326,
            Chain::Robinhood => 4663,
            Chain::Arc => 5042,
        }
    }

    /// Chain name used in API paths
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Bsc => "bsc",
            Chain::Polygon => "polygon",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Avalanche => "avalanche",
            Chain::Base => "base",
            Chain::Linea => "linea",
            Chain::Sonic => "sonic",
            Chain::Berachain => "berachain",
            Chain::Unichain => "unichain",
            Chain::Ronin => "ronin",
            Chain::HyperEvm => "hyperevm",
            Chain::Plasma => "plasma",
            Chain::Monad => "monad",
            Chain::Etherlink => "etherlink",
            Chain::MegaEth => "megaeth",
            Chain::Robinhood => "robinhood",
            Chain::Arc => "arc",
        }
    }

    /// Parse chain from string (returns Option for backward compatibility)
    #[must_use]
    pub fn try_from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl FromStr for Chain {
    type Err = ParseChainError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Ok(Chain::Ethereum),
            "bsc" | "bnb" | "binance" => Ok(Chain::Bsc),
            "polygon" | "matic" => Ok(Chain::Polygon),
            "arbitrum" | "arb" => Ok(Chain::Arbitrum),
            "optimism" | "op" => Ok(Chain::Optimism),
            "avalanche" | "avax" => Ok(Chain::Avalanche),
            "base" => Ok(Chain::Base),
            "linea" => Ok(Chain::Linea),
            "sonic" => Ok(Chain::Sonic),
            "berachain" | "bera" => Ok(Chain::Berachain),
            "unichain" => Ok(Chain::Unichain),
            "ronin" => Ok(Chain::Ronin),
            "hyperevm" | "hyperliquid" => Ok(Chain::HyperEvm),
            "plasma" => Ok(Chain::Plasma),
            "monad" => Ok(Chain::Monad),
            "etherlink" => Ok(Chain::Etherlink),
            "megaeth" => Ok(Chain::MegaEth),
            "robinhood" => Ok(Chain::Robinhood),
            "arc" => Ok(Chain::Arc),
            _ => Err(ParseChainError {
                input: s.to_string(),
            }),
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
        Self::from_chain_id(chain.id()).ok_or("Chain not supported by KyberSwap")
    }
}

impl From<Chain> for yldfi_common::Chain {
    fn from(chain: Chain) -> Self {
        Self::from_id(chain.chain_id())
    }
}

/// Route request parameters
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteRequest {
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount with decimals
    pub amount_in: String,
    /// User address (optional for quote)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    /// Save gas mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_gas: Option<bool>,
    /// Only route through these sources (comma-separated, `includedSources`)
    #[serde(rename = "includedSources", skip_serializing_if = "Option::is_none")]
    pub include_dexs: Option<String>,
    /// Never route through these sources (comma-separated, `excludedSources`)
    #[serde(rename = "excludedSources", skip_serializing_if = "Option::is_none")]
    pub exclude_dexs: Option<String>,
    /// Account for gas cost when picking the route (`gasInclude`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_include: Option<bool>,
}

impl RouteRequest {
    pub fn new(
        token_in: impl Into<String>,
        token_out: impl Into<String>,
        amount_in: impl Into<String>,
    ) -> Self {
        Self {
            token_in: token_in.into(),
            token_out: token_out.into(),
            amount_in: amount_in.into(),
            to: None,
            save_gas: None,
            include_dexs: None,
            exclude_dexs: None,
            gas_include: None,
        }
    }

    #[must_use]
    pub fn with_recipient(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    #[must_use]
    pub fn with_save_gas(mut self, save_gas: bool) -> Self {
        self.save_gas = Some(save_gas);
        self
    }

    /// Only route through these sources (comma-separated, `includedSources`)
    #[must_use]
    pub fn with_included_sources(mut self, sources: impl Into<String>) -> Self {
        self.include_dexs = Some(sources.into());
        self
    }

    /// Never route through these sources (comma-separated, `excludedSources`)
    #[must_use]
    pub fn with_excluded_sources(mut self, sources: impl Into<String>) -> Self {
        self.exclude_dexs = Some(sources.into());
        self
    }

    /// Account for gas cost when picking the route (`gasInclude`)
    #[must_use]
    pub fn with_gas_include(mut self, gas_include: bool) -> Self {
        self.gas_include = Some(gas_include);
        self
    }

    /// Query parameters for `GET /{chain}/api/v1/routes`
    #[must_use]
    pub fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("tokenIn", self.token_in.clone()),
            ("tokenOut", self.token_out.clone()),
            ("amountIn", self.amount_in.clone()),
        ];
        if let Some(ref to) = self.to {
            params.push(("to", to.clone()));
        }
        if let Some(save_gas) = self.save_gas {
            params.push(("saveGas", save_gas.to_string()));
        }
        if let Some(ref sources) = self.include_dexs {
            params.push(("includedSources", sources.clone()));
        }
        if let Some(ref sources) = self.exclude_dexs {
            params.push(("excludedSources", sources.clone()));
        }
        if let Some(gas_include) = self.gas_include {
            params.push(("gasInclude", gas_include.to_string()));
        }
        params
    }

    /// Validate the route request parameters
    ///
    /// Checks that addresses and amounts are properly formatted.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Token addresses are not valid Ethereum addresses (0x + 40 hex chars)
    /// - Amounts contain non-numeric characters
    pub fn validate(&self) -> Result<(), &'static str> {
        // Validate token addresses
        if !is_valid_address(&self.token_in) {
            return Err("invalid token_in address");
        }
        if !is_valid_address(&self.token_out) {
            return Err("invalid token_out address");
        }

        // Validate recipient address if provided
        if let Some(ref to) = self.to {
            if !is_valid_address(to) {
                return Err("invalid recipient address");
            }
        }

        // Validate amount (should be numeric string)
        if !is_valid_amount(&self.amount_in) {
            return Err("invalid amount_in");
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

/// Routes response from `KyberSwap` API
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RoutesResponse {
    /// Response code (0 = success)
    pub code: i32,
    /// Response message
    pub message: String,
    /// Route data
    pub data: Option<RouteData>,
}

/// Route data
///
/// `route_summary` is kept as the raw JSON returned by the API: `/route/build`
/// validates it (including `routeID`, `checksum` and `timestamp`), so it must
/// be sent back unmodified. Use [`RouteData::summary`] for a typed view.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteData {
    /// Route summary exactly as returned by the API
    pub route_summary: serde_json::Value,
    /// Detailed router address
    #[serde(default)]
    pub router_address: Option<String>,
}

impl RouteData {
    /// Typed view of the route summary
    ///
    /// # Errors
    ///
    /// Returns an error if the summary does not match the expected shape.
    pub fn summary(&self) -> Result<RouteSummary, serde_json::Error> {
        RouteSummary::deserialize(&self.route_summary)
    }
}

/// Route summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RouteSummary {
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount
    pub amount_in: String,
    /// Output amount
    pub amount_out: String,
    /// Input amount in USD
    #[serde(default)]
    pub amount_in_usd: Option<String>,
    /// Output amount in USD
    #[serde(default)]
    pub amount_out_usd: Option<String>,
    /// Estimated gas
    #[serde(default)]
    pub gas: Option<String>,
    /// Gas price used for estimation
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Gas in USD
    #[serde(default)]
    pub gas_usd: Option<String>,
    /// Swap route
    #[serde(default)]
    pub route: Vec<Vec<SwapStep>>,
}

impl RouteSummary {
    /// Price impact in percent, derived from `amountInUsd` and
    /// `amountOutUsd` (v1 routes have no price-impact field)
    ///
    /// Positive means the output is worth less than the input.
    #[must_use]
    pub fn price_impact_percent(&self) -> Option<f64> {
        let amount_in: f64 = self.amount_in_usd.as_deref()?.parse().ok()?;
        let amount_out: f64 = self.amount_out_usd.as_deref()?.parse().ok()?;
        (amount_in > 0.0).then(|| (amount_in - amount_out) / amount_in * 100.0)
    }
}

/// Swap step in route
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SwapStep {
    /// Pool address
    pub pool: String,
    /// Token in address
    pub token_in: String,
    /// Token out address
    pub token_out: String,
    /// Swap amount
    #[serde(default)]
    pub swap_amount: Option<String>,
    /// Amount out
    #[serde(default)]
    pub amount_out: Option<String>,
    /// Pool type/exchange name
    #[serde(default)]
    pub exchange: Option<String>,
}

/// Build route request (to get transaction data)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRouteRequest {
    /// Route summary from [`RouteData::route_summary`], passed through
    /// unmodified (the API verifies its checksum)
    pub route_summary: serde_json::Value,
    /// Sender address
    pub sender: String,
    /// Recipient address
    pub recipient: String,
    /// Slippage tolerance in bips (sent as `slippageTolerance`, 10 = 0.1%)
    #[serde(rename = "slippageTolerance", skip_serializing_if = "Option::is_none")]
    pub slippage_tolerance_bps: Option<u32>,
    /// Deadline timestamp (unix seconds)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<u64>,
    /// Enable permit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_permit: Option<bool>,
}

/// Build route response with transaction data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRouteResponse {
    /// Response code
    pub code: i32,
    /// Response message
    pub message: String,
    /// Transaction data
    pub data: Option<BuildRouteData>,
}

/// Build route data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BuildRouteData {
    /// Router contract address
    pub router_address: String,
    /// Encoded call data
    pub data: String,
    /// ETH value to send (API field `transactionValue`)
    #[serde(default, rename = "transactionValue", alias = "value")]
    pub value: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
    /// Input amount
    #[serde(default)]
    pub amount_in: Option<String>,
    /// Expected output amount
    #[serde(default)]
    pub amount_out: Option<String>,
    /// Gas cost in USD
    #[serde(default)]
    pub gas_usd: Option<String>,
}

/// Parameters for `GET /{chain}/api/v1/swap`: find the best route and
/// encode it in one call
///
/// `KyberSwap` restricts this endpoint to partners that build every route
/// they request, since it includes RFQ liquidity; otherwise use
/// [`Client::get_route_data`](crate::Client::get_route_data) followed by
/// [`Client::build_route`](crate::Client::build_route).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct SwapRequest {
    /// Input token address (`0xEeee...EEeE` for the native token)
    pub token_in: String,
    /// Output token address (`0xEeee...EEeE` for the native token)
    pub token_out: String,
    /// Input amount in base units
    pub amount_in: String,
    /// Address the input tokens are transferred from
    pub sender: Option<String>,
    /// Address the output tokens are sent to
    pub recipient: Option<String>,
    /// Wallet that originates the transaction, for exclusive pools and rates
    pub origin: Option<String>,
    /// Slippage tolerance in bips (sent as `slippageTolerance`, 10 = 0.1%)
    pub slippage_tolerance_bps: Option<u32>,
    /// Deadline (unix seconds); the API defaults to 20 minutes from now
    pub deadline: Option<u64>,
    /// Only route through these sources (comma-separated)
    pub included_sources: Option<String>,
    /// Never route through these sources (comma-separated)
    pub excluded_sources: Option<String>,
    /// Exclude RFQ sources
    pub exclude_rfq_sources: Option<bool>,
    /// Only use sources that accept scaled input amounts
    pub only_scalable_sources: Option<bool>,
    /// Only route directly from `token_in` to `token_out`, without hops
    pub only_direct_pools: Option<bool>,
    /// Only return a single-path route
    pub only_single_path: Option<bool>,
    /// Account for gas cost when picking the route (API default: true)
    pub gas_include: Option<bool>,
    /// Gas price in wei to route with
    pub gas_price: Option<String>,
    /// Charge the fee in the input (`currency_in`) or output
    /// (`currency_out`) token
    pub charge_fee_by: Option<String>,
    /// Whether `fee_amount` is in bps (10000 = 100%) rather than wei
    pub is_in_bps: Option<bool>,
    /// Fee amount(s), comma-separated for several receivers
    pub fee_amount: Option<String>,
    /// Fee receiver address(es), comma-separated
    pub fee_receiver: Option<String>,
    /// Encoded permit calldata, to swap without an approval
    pub permit: Option<String>,
    /// Allow a slippage tolerance above the API's cap
    pub ignore_capped_slippage: Option<bool>,
    /// Estimate gas with `eth_estimateGas`, which also detects reverts
    pub enable_gas_estimation: Option<bool>,
    /// Source recorded on chain for the swap
    pub source: Option<String>,
    /// Referral info for the swap's `ClientData` event
    pub referral: Option<String>,
}

impl SwapRequest {
    /// A swap of `amount_in` of `token_in` into `token_out`
    #[must_use]
    pub fn new(
        token_in: impl Into<String>,
        token_out: impl Into<String>,
        amount_in: impl Into<String>,
    ) -> Self {
        Self {
            token_in: token_in.into(),
            token_out: token_out.into(),
            amount_in: amount_in.into(),
            ..Self::default()
        }
    }

    /// Set the sender (input tokens are taken from it)
    #[must_use]
    pub fn with_sender(mut self, sender: impl Into<String>) -> Self {
        self.sender = Some(sender.into());
        self
    }

    /// Set the recipient of the output tokens
    #[must_use]
    pub fn with_recipient(mut self, recipient: impl Into<String>) -> Self {
        self.recipient = Some(recipient.into());
        self
    }

    /// Set the origin wallet
    #[must_use]
    pub fn with_origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }

    /// Set slippage tolerance in bips (10 = 0.1%)
    #[must_use]
    pub fn with_slippage_bps(mut self, bps: u32) -> Self {
        self.slippage_tolerance_bps = Some(bps);
        self
    }

    /// Set the deadline (unix seconds)
    #[must_use]
    pub fn with_deadline(mut self, deadline: u64) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Only route through these sources (comma-separated)
    #[must_use]
    pub fn with_included_sources(mut self, sources: impl Into<String>) -> Self {
        self.included_sources = Some(sources.into());
        self
    }

    /// Never route through these sources (comma-separated)
    #[must_use]
    pub fn with_excluded_sources(mut self, sources: impl Into<String>) -> Self {
        self.excluded_sources = Some(sources.into());
        self
    }

    /// Exclude RFQ sources
    #[must_use]
    pub fn with_exclude_rfq_sources(mut self, exclude: bool) -> Self {
        self.exclude_rfq_sources = Some(exclude);
        self
    }

    /// Charge an integrator fee
    ///
    /// `charge_fee_by` is `currency_in` or `currency_out`; with `is_in_bps`
    /// the amount is in bps (10000 = 100%), otherwise in wei.
    #[must_use]
    pub fn with_fee(
        mut self,
        charge_fee_by: impl Into<String>,
        fee_amount: impl Into<String>,
        is_in_bps: bool,
        fee_receiver: impl Into<String>,
    ) -> Self {
        self.charge_fee_by = Some(charge_fee_by.into());
        self.fee_amount = Some(fee_amount.into());
        self.is_in_bps = Some(is_in_bps);
        self.fee_receiver = Some(fee_receiver.into());
        self
    }

    /// Estimate gas with `eth_estimateGas`
    #[must_use]
    pub fn with_gas_estimation(mut self, enabled: bool) -> Self {
        self.enable_gas_estimation = Some(enabled);
        self
    }

    /// Set the source recorded on chain
    #[must_use]
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Query parameters for `GET /{chain}/api/v1/swap`
    #[must_use]
    pub fn to_query_params(&self) -> Vec<(&'static str, String)> {
        let mut params = vec![
            ("tokenIn", self.token_in.clone()),
            ("tokenOut", self.token_out.clone()),
            ("amountIn", self.amount_in.clone()),
        ];
        let strings = [
            ("sender", &self.sender),
            ("recipient", &self.recipient),
            ("origin", &self.origin),
            ("includedSources", &self.included_sources),
            ("excludedSources", &self.excluded_sources),
            ("gasPrice", &self.gas_price),
            ("chargeFeeBy", &self.charge_fee_by),
            ("feeAmount", &self.fee_amount),
            ("feeReceiver", &self.fee_receiver),
            ("permit", &self.permit),
            ("source", &self.source),
            ("referral", &self.referral),
        ];
        for (key, value) in strings {
            if let Some(value) = value {
                params.push((key, value.clone()));
            }
        }
        let bools = [
            ("excludeRFQSources", self.exclude_rfq_sources),
            ("onlyScalableSources", self.only_scalable_sources),
            ("onlyDirectPools", self.only_direct_pools),
            ("onlySinglePath", self.only_single_path),
            ("gasInclude", self.gas_include),
            ("isInBps", self.is_in_bps),
            ("ignoreCappedSlippage", self.ignore_capped_slippage),
            ("enableGasEstimation", self.enable_gas_estimation),
        ];
        for (key, value) in bools {
            if let Some(value) = value {
                params.push((key, value.to_string()));
            }
        }
        if let Some(bps) = self.slippage_tolerance_bps {
            params.push(("slippageTolerance", bps.to_string()));
        }
        if let Some(deadline) = self.deadline {
            params.push(("deadline", deadline.to_string()));
        }
        params
    }
}

/// Response of `GET /{chain}/api/v1/swap`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SwapResponse {
    /// Response code (0 = success)
    pub code: i32,
    /// Response message
    #[serde(default)]
    pub message: Option<String>,
    /// Route and encoded transaction
    #[serde(default)]
    pub data: Option<SwapData>,
    /// Request ID, for support
    #[serde(default)]
    pub request_id: Option<String>,
}

/// The route and its encoded transaction, from `GET /{chain}/api/v1/swap`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SwapData {
    /// The route found
    pub route_summary: RouteSummary,
    /// Router contract to send the transaction to
    pub router_address: String,
    /// Encoded calldata
    pub data: String,
    /// Native token value to send, in wei
    #[serde(default)]
    pub transaction_value: Option<String>,
    /// Input amount
    #[serde(default)]
    pub amount_in: Option<String>,
    /// Input amount in USD
    #[serde(default)]
    pub amount_in_usd: Option<String>,
    /// Output amount
    #[serde(default)]
    pub amount_out: Option<String>,
    /// Output amount in USD
    #[serde(default)]
    pub amount_out_usd: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
    /// Gas cost in USD
    #[serde(default)]
    pub gas_usd: Option<String>,
    /// Additional cost in USD (e.g. L1 fee)
    #[serde(default)]
    pub additional_cost_usd: Option<String>,
    /// What the additional cost is for
    #[serde(default)]
    pub additional_cost_message: Option<String>,
}

/// Token info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    pub address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_aliases_parse() {
        for (name, chain) in [
            ("ETH", Chain::Ethereum),
            ("binance", Chain::Bsc),
            ("bnb", Chain::Bsc),
            ("sonic", Chain::Sonic),
            ("bera", Chain::Berachain),
            ("hyperevm", Chain::HyperEvm),
        ] {
            assert_eq!(name.parse::<Chain>().unwrap(), chain, "{name}");
        }
        // Chains KyberSwap has deactivated
        for name in [
            "fantom",
            "scroll",
            "zksync",
            "blast",
            "mantle",
            "polygon-zkevm",
            "gnosis",
        ] {
            assert!(name.parse::<Chain>().is_err(), "{name}");
        }
    }

    #[test]
    fn chain_mappings_round_trip() {
        for chain in Chain::all() {
            assert_eq!(chain.as_str().parse::<Chain>().unwrap(), *chain);
            assert_eq!(Chain::from_chain_id(chain.chain_id()), Some(*chain));
            let common = yldfi_common::Chain::from(*chain);
            assert!(
                !matches!(common, yldfi_common::Chain::Other(_)),
                "{chain:?}"
            );
            assert_eq!(Chain::try_from(common), Ok(*chain));
        }
    }

    const LIVE_ROUTE_DATA: &str = r#"{"routeSummary":{"tokenIn":"0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
        "amountIn":"1000000000000000000","amountInUsd":"2691.83","tokenOut":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        "amountOut":"2694223548","amountOutUsd":"2688.29","gas":"697210","gasPrice":"105254963","gasUsd":"0.19",
        "l1FeeUsd":"0","extraFee":{"feeAmount":"","chargeFeeBy":"","isInBps":false,"feeReceiver":""},
        "route":[[{"pool":"0xe6ff","tokenIn":"0xc02a","tokenOut":"0x2260","swapAmount":"1","amountOut":"2",
        "exchange":"uniswapv3","poolType":"uniswapv3","poolExtra":{"swapFee":100},"extra":{"_cs":"1"}}]],
        "routerAddress":"0x6131B5fae19EA4f9D964eAc0408E4408b66337b5","routeID":"e30c6d4ctNFIE6V9",
        "checksum":"4239051851851966504","timestamp":1790368014},
        "routerAddress":"0x6131B5fae19EA4f9D964eAc0408E4408b66337b5"}"#;

    #[test]
    fn build_data_reads_transaction_value() {
        let d: BuildRouteData = serde_json::from_str(
            r#"{"routerAddress":"0x6131","data":"0x","transactionValue":"1000000000000000000","gas":"330498"}"#,
        )
        .unwrap();
        assert_eq!(d.value.as_deref(), Some("1000000000000000000"));
    }

    #[test]
    fn route_summary_is_preserved_for_build() {
        let original: serde_json::Value = serde_json::from_str(LIVE_ROUTE_DATA).unwrap();
        let data: RouteData = serde_json::from_str(LIVE_ROUTE_DATA).unwrap();
        assert_eq!(data.summary().unwrap().amount_out, "2694223548");

        let build = BuildRouteRequest {
            route_summary: data.route_summary.clone(),
            sender: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".into(),
            recipient: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".into(),
            slippage_tolerance_bps: Some(50),
            deadline: None,
            enable_permit: None,
        };
        let body = serde_json::to_value(&build).unwrap();
        assert_eq!(body["routeSummary"], original["routeSummary"]);
        assert_eq!(body["routeSummary"]["checksum"], "4239051851851966504");
        assert_eq!(body["slippageTolerance"], 50);
        assert!(body.get("slippageToleranceBps").is_none());
    }
}
