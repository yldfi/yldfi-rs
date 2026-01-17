//! Types for stablecoin data

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Stablecoin summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Stablecoin {
    /// Stablecoin ID
    pub id: String,
    /// Stablecoin name
    pub name: String,
    /// Symbol
    pub symbol: String,
    /// Gecko ID
    pub gecko_id: Option<String>,
    /// Peg type (e.g., "peggedUSD", "peggedEUR")
    pub peg_type: Option<String>,
    /// Peg mechanism (e.g., "fiat-backed", "crypto-backed", "algorithmic")
    pub peg_mechanism: Option<String>,
    /// Current circulating supply (market cap)
    pub circulating: Option<CirculatingSupply>,
    /// Previous day circulating
    pub circulating_prev_day: Option<CirculatingSupply>,
    /// Previous week circulating
    pub circulating_prev_week: Option<CirculatingSupply>,
    /// Previous month circulating
    pub circulating_prev_month: Option<CirculatingSupply>,
    /// Chain breakdown
    #[serde(default)]
    pub chain_circulating: HashMap<String, ChainCirculating>,
    /// Current price
    pub price: Option<FlexNumber>,
    /// Price source
    pub price_source: Option<String>,
}

/// A number that can be deserialized from either a number or string
#[derive(Debug, Clone, Serialize)]
pub struct FlexNumber(pub f64);

impl<'de> Deserialize<'de> for FlexNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum StringOrNumber {
            String(String),
            Number(f64),
        }

        match StringOrNumber::deserialize(deserializer)? {
            StringOrNumber::Number(n) => Ok(FlexNumber(n)),
            StringOrNumber::String(s) => s.parse().map(FlexNumber).map_err(Error::custom),
        }
    }
}

/// Circulating supply data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CirculatingSupply {
    /// Pegged amount (e.g., peggedUSD)
    #[serde(flatten)]
    pub pegged: HashMap<String, FlexNumber>,
}

/// Chain-specific circulating data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChainCirculating {
    /// Current circulating
    pub current: Option<CirculatingSupply>,
    /// Previous day circulating
    pub circulating_prev_day: Option<CirculatingSupply>,
    /// Previous week circulating
    pub circulating_prev_week: Option<CirculatingSupply>,
    /// Previous month circulating
    pub circulating_prev_month: Option<CirculatingSupply>,
}

/// Stablecoin with detailed chain data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StablecoinDetail {
    /// Stablecoin ID
    pub id: String,
    /// Stablecoin name
    pub name: String,
    /// Contract addresses by chain
    pub address: Option<String>,
    /// Symbol
    pub symbol: String,
    /// Gecko ID
    pub gecko_id: Option<String>,
    /// Peg type
    pub peg_type: Option<String>,
    /// Peg mechanism
    pub peg_mechanism: Option<String>,
    /// Total circulating
    pub circulating: Option<CirculatingSupply>,
    /// Chain breakdown
    #[serde(default)]
    pub chain_circulating: HashMap<String, ChainCirculating>,
    /// Chains this stablecoin is on
    #[serde(default)]
    pub chains: Vec<String>,
    /// Current price
    pub price: Option<FlexNumber>,
    /// Token addresses per chain
    #[serde(default)]
    pub chain_addresses: HashMap<String, String>,
}

/// Historical stablecoin chart data point
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StablecoinChartPoint {
    /// Unix timestamp
    pub date: u64,
    /// Circulating supply breakdown
    #[serde(flatten)]
    pub circulating: HashMap<String, FlexNumber>,
}

/// Response from stablecoins list endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StablecoinsResponse {
    /// List of stablecoins
    #[serde(default)]
    pub pegged_assets: Vec<Stablecoin>,
}

/// Chain with stablecoin data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StablecoinChain {
    /// Chain gecko ID
    pub gecko_id: Option<String>,
    /// Total circulating on chain
    pub total_circulating_usd: Option<CirculatingSupply>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Chain name
    pub name: Option<String>,
}

/// Stablecoin dominance data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StablecoinDominance {
    /// Unix timestamp
    pub date: u64,
    /// Dominance percentage by stablecoin
    #[serde(flatten)]
    pub dominance: HashMap<String, f64>,
}

/// Historical stablecoin price data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StablecoinPricePoint {
    /// Unix timestamp
    pub date: u64,
    /// Price in USD
    pub price: f64,
}

/// Stablecoin prices response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StablecoinPricesResponse(pub Vec<StablecoinPriceData>);

/// Price data for a stablecoin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StablecoinPriceData {
    /// Stablecoin symbol
    pub symbol: String,
    /// Price history
    pub prices: Vec<StablecoinPricePoint>,
}
