//! Types for the Velora (`ParaSwap`) API
//!
//! This module contains request and response types for the `ParaSwap` API,
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

/// Default Augustus router version requested from `/prices`
///
/// When `version` is omitted the Velora API falls back to the legacy v5
/// router, so requests default to Augustus v6.2.
pub const DEFAULT_PRICE_VERSION: &str = "6.2";

/// Price request parameters for getting swap quotes
#[derive(Debug, Clone, Default)]
pub struct PriceRequest {
    /// Augustus router version (`"6.2"` or legacy `"5"`).
    /// `None` sends [`DEFAULT_PRICE_VERSION`].
    pub version: Option<String>,
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

    /// Set the Augustus router version (e.g. `"6.2"`, or `"5"` for legacy)
    ///
    /// Note that the approval spender (`tokenTransferProxy`) and swap
    /// contract in the returned route depend on this version.
    #[must_use]
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Convert to query parameters
    #[must_use]
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
            (
                "version".to_string(),
                self.version
                    .clone()
                    .unwrap_or_else(|| DEFAULT_PRICE_VERSION.to_string()),
            ),
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

        params
    }
}

/// Price response with routing information
///
/// Velora's `/transactions` endpoint rejects any `priceRoute` that is not
/// byte-for-byte what `/prices` returned ("priceRoute must be unmodified"),
/// including the `hmac` it signs. The response therefore keeps the raw
/// `priceRoute` JSON alongside the typed view; serialization emits the raw
/// value untouched.
#[derive(Debug, Clone)]
pub struct PriceResponse {
    /// Typed view of the price route (for reading amounts, gas, etc.)
    pub price_route: PriceRoute,
    /// The price route exactly as returned by the API
    pub raw_price_route: serde_json::Value,
}

impl PriceResponse {
    /// Build a response from a raw `priceRoute` JSON value.
    ///
    /// # Errors
    ///
    /// Returns an error if the value does not match the expected shape.
    pub fn from_raw_price_route(raw: serde_json::Value) -> Result<Self, serde_json::Error> {
        let price_route = PriceRoute::deserialize(&raw)?;
        Ok(Self {
            price_route,
            raw_price_route: raw,
        })
    }
}

impl<'de> Deserialize<'de> for PriceResponse {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Raw {
            price_route: serde_json::Value,
        }
        let raw = Raw::deserialize(deserializer)?;
        Self::from_raw_price_route(raw.price_route).map_err(serde::de::Error::custom)
    }
}

impl Serialize for PriceResponse {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = serializer.serialize_struct("PriceResponse", 1)?;
        st.serialize_field("priceRoute", &self.raw_price_route)?;
        st.end()
    }
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
    #[serde(default)]
    pub gas_cost: Option<String>,
    /// Gas cost in USD
    #[serde(default, rename = "gasCostUSD")]
    pub gas_cost_usd: Option<String>,
    /// Side of the swap
    pub side: String,
    /// Source token USD value
    #[serde(default, rename = "srcUSD")]
    pub src_usd: Option<String>,
    /// Destination token USD value
    #[serde(default, rename = "destUSD")]
    pub dest_usd: Option<String>,
    /// Destination amount after partner fee
    #[serde(default)]
    pub dest_amount_after_fee: Option<String>,
    /// Augustus router version (e.g. "6.2")
    #[serde(default)]
    pub version: Option<String>,
    /// Partner name
    #[serde(default)]
    pub partner: Option<String>,
    /// HMAC over the route (validated by `/transactions`)
    #[serde(default)]
    pub hmac: Option<String>,
    /// Max impact percentage
    #[serde(default)]
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
    /// Source token decimals (required by Velora for tokens not in its list)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_decimals: Option<u8>,
    /// Destination token decimals
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_decimals: Option<u8>,
    /// Source amount (the fixed amount on a SELL).
    ///
    /// With `slippage` set, Velora wants only the fixed side's amount:
    /// `srcAmount` on SELL, `destAmount` on BUY. Sending the other one too is
    /// rejected ("Cannot specify both"), so [`TransactionRequest::new`] fills
    /// exactly one of the two based on the route's `side`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src_amount: Option<String>,
    /// Destination amount (the fixed amount on a BUY).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dest_amount: Option<String>,
    /// Price route exactly as returned by `/prices` (must be unmodified)
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
    /// Ignore gas estimation (sent as the `ignoreGasEstimate` query param)
    #[serde(skip)]
    pub ignore_gas: Option<bool>,
    /// Ignore balance and allowance checks (sent as the `ignoreChecks` query param)
    #[serde(skip)]
    pub ignore_checks: Option<bool>,
}

impl TransactionRequest {
    /// Create a new transaction request from a price response.
    ///
    /// The raw `priceRoute` is passed through untouched.
    #[must_use]
    pub fn new(price: &PriceResponse, user_address: impl Into<String>, slippage: u32) -> Self {
        let route = &price.price_route;
        let is_buy = route.side.eq_ignore_ascii_case("BUY");
        Self {
            src_token: route.src_token.clone(),
            dest_token: route.dest_token.clone(),
            src_decimals: Some(route.src_decimals),
            dest_decimals: Some(route.dest_decimals),
            src_amount: (!is_buy).then(|| route.src_amount.clone()),
            dest_amount: is_buy.then(|| route.dest_amount.clone()),
            price_route: price.raw_price_route.clone(),
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

    /// Create a transaction request from a raw `priceRoute` JSON value
    /// (e.g. copied from the `/prices` output).
    ///
    /// # Errors
    ///
    /// Returns an error if the value is not a valid price route.
    pub fn from_raw_price_route(
        price_route: serde_json::Value,
        user_address: impl Into<String>,
        slippage: u32,
    ) -> Result<Self, serde_json::Error> {
        let price = PriceResponse::from_raw_price_route(price_route)?;
        Ok(Self::new(&price, user_address, slippage))
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

    /// Query parameters for `POST /transactions/{chainId}`.
    ///
    /// `ignoreGasEstimate` and `ignoreChecks` are query parameters of the
    /// endpoint, not body fields.
    #[must_use]
    pub fn to_query_params(&self) -> Vec<(String, String)> {
        let mut params = Vec::new();
        if let Some(v) = self.ignore_gas {
            params.push(("ignoreGasEstimate".to_string(), v.to_string()));
        }
        if let Some(v) = self.ignore_checks {
            params.push(("ignoreChecks".to_string(), v.to_string()));
        }
        params
    }
}

/// Transaction response ready for signing
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    /// Token contract address
    pub address: String,
    /// Token symbol
    pub symbol: String,
    /// Token name (optional, not always returned by API)
    #[serde(default)]
    pub name: Option<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    fn test_price_request_sends_only_documented_params() {
        let request = PriceRequest::sell("0xsrc", "0xdst", "1000");
        let params = request.to_query_params(1);
        assert!(params.iter().all(|(k, _)| k != "excludePoolsWithLowTVL"));
        assert!(params.iter().any(|(k, v)| k == "network" && v == "1"));
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

    const LIVE_PRICE: &str = r#"{"priceRoute":{"blockNumber":26056822,"network":1,
        "srcToken":"0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee","srcDecimals":18,
        "srcAmount":"1000000000000000000",
        "destToken":"0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48","destDecimals":6,
        "destAmount":"2690528459","bestRoute":[],"gasCostUSD":"0.072117","gasCost":"235640",
        "side":"SELL","version":"6.2","contractAddress":"0x6a000f20005980200259b80c5102003040001068",
        "tokenTransferProxy":"0x6a000f20005980200259b80c5102003040001068",
        "contractMethod":"swapExactAmountIn","partnerFee":0.01,"srcUSD":"2690.5700000000",
        "destUSD":"2690.3481935932","destAmountAfterFee":"2690259406","partner":"anon",
        "maxImpactReached":false,"hmac":"c056035782855c7aacae85e82615ddd5cb3255d1"}}"#;

    #[test]
    fn price_response_round_trip_is_lossless() {
        let original: serde_json::Value = serde_json::from_str(LIVE_PRICE).unwrap();
        let parsed: PriceResponse = serde_json::from_str(LIVE_PRICE).unwrap();
        assert_eq!(
            parsed.price_route.src_usd.as_deref(),
            Some("2690.5700000000")
        );
        assert_eq!(parsed.price_route.gas_cost_usd.as_deref(), Some("0.072117"));
        assert_eq!(
            parsed.price_route.hmac.as_deref(),
            Some("c056035782855c7aacae85e82615ddd5cb3255d1")
        );
        assert_eq!(serde_json::to_value(&parsed).unwrap(), original);
    }

    #[test]
    fn transaction_request_passes_route_through_and_omits_dest_amount() {
        let original: serde_json::Value = serde_json::from_str(LIVE_PRICE).unwrap();
        let req = TransactionRequest::from_raw_price_route(
            original["priceRoute"].clone(),
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            100,
        )
        .unwrap();
        let body = serde_json::to_value(&req).unwrap();
        assert_eq!(body["priceRoute"], original["priceRoute"]);
        assert!(body.get("destAmount").is_none(), "{body}");
        assert_eq!(body["slippage"], 100);
        assert_eq!(body["srcDecimals"], 18);
        assert_eq!(body["destDecimals"], 6);
        assert_eq!(body["srcAmount"], "1000000000000000000");
        assert!(body.get("ignoreChecks").is_none(), "{body}");
        assert!(req.to_query_params().is_empty());
    }

    #[test]
    fn transaction_request_buy_sends_dest_amount_only() {
        let mut route: serde_json::Value = serde_json::from_str(LIVE_PRICE).unwrap();
        route["priceRoute"]["side"] = "BUY".into();
        let req = TransactionRequest::from_raw_price_route(
            route["priceRoute"].clone(),
            "0x0000000000000000000000000000000000000001",
            50,
        )
        .unwrap();
        let body = serde_json::to_value(&req).unwrap();
        assert_eq!(body["destAmount"], "2690528459");
        assert!(body.get("srcAmount").is_none(), "{body}");
        assert_eq!(body["slippage"], 50);
    }

    #[test]
    fn transaction_request_ignore_flags_are_query_params() {
        let original: serde_json::Value = serde_json::from_str(LIVE_PRICE).unwrap();
        let req = TransactionRequest::from_raw_price_route(
            original["priceRoute"].clone(),
            "0x0000000000000000000000000000000000000001",
            100,
        )
        .unwrap()
        .with_ignore_gas(true)
        .with_ignore_checks(true);
        let body = serde_json::to_value(&req).unwrap();
        for key in ["ignoreGas", "ignoreGasEstimate", "ignoreChecks"] {
            assert!(body.get(key).is_none(), "{key} in body: {body}");
        }
        assert_eq!(
            req.to_query_params(),
            vec![
                ("ignoreGasEstimate".to_string(), "true".to_string()),
                ("ignoreChecks".to_string(), "true".to_string()),
            ]
        );
    }
}
