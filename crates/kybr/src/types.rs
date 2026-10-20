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
    /// Include DEXs (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_dexs: Option<String>,
    /// Exclude DEXs (comma-separated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_dexs: Option<String>,
    /// Gas include mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_include: Option<bool>,
    /// Slippage tolerance in bips (1 = 0.01%)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_tolerance_bps: Option<u32>,
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
            slippage_tolerance_bps: None,
        }
    }

    #[must_use]
    pub fn with_recipient(mut self, to: impl Into<String>) -> Self {
        self.to = Some(to.into());
        self
    }

    #[must_use]
    pub fn with_slippage_bps(mut self, bps: u32) -> Self {
        self.slippage_tolerance_bps = Some(bps);
        self
    }

    #[must_use]
    pub fn with_save_gas(mut self, save_gas: bool) -> Self {
        self.save_gas = Some(save_gas);
        self
    }

    /// Validate the route request parameters
    ///
    /// Checks that addresses, amounts, and slippage are properly formatted.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Token addresses are not valid Ethereum addresses (0x + 40 hex chars)
    /// - Amounts contain non-numeric characters
    /// - Slippage tolerance is unreasonably high (> 5000 bps = 50%)
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

        // Validate slippage tolerance (max 50% = 5000 bps is already extreme)
        if let Some(bps) = self.slippage_tolerance_bps {
            if bps > 5000 {
                return Err("slippage_tolerance_bps too high (max 5000 = 50%)");
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
    /// Minimum output after slippage
    #[serde(default)]
    pub amount_out_min: Option<String>,
    /// Estimated gas
    #[serde(default)]
    pub gas: Option<String>,
    /// Gas price used for estimation
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Gas in USD
    #[serde(default)]
    pub gas_usd: Option<String>,
    /// Price impact percentage
    #[serde(default)]
    pub price_impact: Option<f64>,
    /// Swap route
    #[serde(default)]
    pub route: Vec<Vec<SwapStep>>,
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
