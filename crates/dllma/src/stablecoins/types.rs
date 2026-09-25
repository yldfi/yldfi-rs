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
    #[serde(rename = "gecko_id")]
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
///
/// Deserialization goes through [`serde_json::Value`] rather than an untagged
/// enum so that it keeps working when `serde_json/arbitrary_precision` is
/// enabled anywhere in the dependency graph (it is, workspace-wide, via
/// `foundry-block-explorers`). With that feature numbers are buffered as a
/// private map token, which untagged enums / `f64` cannot match inside
/// `#[serde(flatten)]` or other buffered contexts.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(transparent)]
pub struct FlexNumber(pub f64);

/// Convert a JSON value (number or numeric string) to `f64`.
fn value_to_f64(v: &serde_json::Value) -> Option<f64> {
    match v {
        serde_json::Value::Number(n) => n.as_f64(),
        serde_json::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
}

impl<'de> Deserialize<'de> for FlexNumber {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let v = serde_json::Value::deserialize(deserializer)?;
        value_to_f64(&v)
            .map(FlexNumber)
            .ok_or_else(|| D::Error::custom(format!("expected number or numeric string, got {v}")))
    }
}

/// Deserialize a `u64` that may be encoded as a number or numeric string
/// (the stablecoin chart endpoints send `"date": "1790294400"`).
fn flex_u64<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let v = serde_json::Value::deserialize(deserializer)?;
    match &v {
        serde_json::Value::Number(n) => n.as_u64(),
        serde_json::Value::String(s) => s.trim().parse().ok(),
        _ => None,
    }
    .ok_or_else(|| D::Error::custom(format!("expected unsigned integer, got {v}")))
}

/// Circulating supply data, keyed by peg (e.g. `peggedUSD`)
///
/// Non-numeric entries (e.g. the nested `bridges` object that appears in
/// `bridgedTo`) are ignored rather than failing the whole response.
#[derive(Debug, Clone, Default, Serialize)]
#[serde(transparent)]
pub struct CirculatingSupply {
    /// Pegged amount (e.g., peggedUSD)
    pub pegged: HashMap<String, FlexNumber>,
}

impl<'de> Deserialize<'de> for CirculatingSupply {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;
        let pegged = raw
            .into_iter()
            .filter_map(|(k, v)| value_to_f64(&v).map(|n| (k, FlexNumber(n))))
            .collect();
        Ok(Self { pegged })
    }
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
    #[serde(rename = "gecko_id")]
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
///
/// Shape: `{"date": "1790294400", "totalCirculating": {"peggedUSD": ..}, ..}`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StablecoinChartPoint {
    /// Unix timestamp
    #[serde(deserialize_with = "flex_u64")]
    pub date: u64,
    /// Total circulating (native units) by peg
    pub total_circulating: Option<CirculatingSupply>,
    /// Total circulating in USD by peg
    #[serde(rename = "totalCirculatingUSD")]
    pub total_circulating_usd: Option<CirculatingSupply>,
    /// Total unreleased by peg
    pub total_unreleased: Option<CirculatingSupply>,
    /// Total minted in USD by peg
    #[serde(rename = "totalMintedUSD")]
    pub total_minted_usd: Option<CirculatingSupply>,
    /// Total bridged-to in USD by peg
    #[serde(rename = "totalBridgedToUSD")]
    pub total_bridged_to_usd: Option<CirculatingSupply>,
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
    #[serde(rename = "gecko_id")]
    pub gecko_id: Option<String>,
    /// Total circulating on chain
    #[serde(rename = "totalCirculatingUSD")]
    pub total_circulating_usd: Option<CirculatingSupply>,
    /// Token symbol
    pub token_symbol: Option<String>,
    /// Chain name
    pub name: Option<String>,
}

/// Stablecoin dominance data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StablecoinDominance {
    /// Unix timestamp
    #[serde(deserialize_with = "flex_u64")]
    pub date: u64,
    /// Total circulating in USD by peg
    #[serde(rename = "totalCirculatingUSD")]
    pub total_circulating_usd: Option<CirculatingSupply>,
    /// Stablecoin with the greatest market cap on this chain
    pub greatest_mcap: Option<GreatestMcap>,
}

/// Largest stablecoin by market cap (dominance endpoint)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GreatestMcap {
    /// CoinGecko ID
    pub gecko_id: Option<String>,
    /// Symbol
    pub symbol: Option<String>,
    /// Market cap in USD
    pub mcap: Option<FlexNumber>,
}

/// Stablecoin prices response: one entry per day
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(transparent)]
pub struct StablecoinPricesResponse(pub Vec<StablecoinPricePoint>);

/// Prices of all stablecoins at a point in time
///
/// Shape: `{"date": 1790294400, "prices": {"tether": 0.9998, ...}}`
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StablecoinPricePoint {
    /// Unix timestamp
    #[serde(deserialize_with = "flex_u64")]
    pub date: u64,
    /// Price in USD keyed by stablecoin gecko ID (null prices are skipped)
    #[serde(default, deserialize_with = "flex_price_map")]
    pub prices: HashMap<String, f64>,
}

fn flex_price_map<'de, D>(deserializer: D) -> Result<HashMap<String, f64>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = Option::<HashMap<String, serde_json::Value>>::deserialize(deserializer)?;
    Ok(raw
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(k, v)| value_to_f64(&v).map(|n| (k, n)))
        .collect())
}

#[cfg(test)]
mod tests {
    //! These tests run with `serde_json/arbitrary_precision` enabled (see the
    //! dev-dependency in Cargo.toml), matching how ethcli builds the crate.
    use super::*;

    #[test]
    fn arbitrary_precision_feature_is_enabled_for_tests() {
        // Guard: if this fails, the regression tests below no longer cover
        // the arbitrary_precision code path.
        let v: serde_json::Value = serde_json::from_str("0.1000000000000000000001").unwrap();
        assert_eq!(v.to_string(), "0.1000000000000000000001");
    }

    #[test]
    fn stablecoins_list_parses_with_arbitrary_precision() {
        let json = r#"{"peggedAssets":[{"id":"1","name":"Tether","symbol":"USDT",
            "gecko_id":"tether","pegType":"peggedUSD","pegMechanism":"fiat-backed",
            "circulating":{"peggedUSD":183727681018.3214},
            "circulatingPrevDay":{"peggedUSD":"183000000000.5"},
            "chainCirculating":{"Ethereum":{"current":{"peggedUSD":73210749042.0027},
                "circulatingPrevDay":{"peggedUSD":73699940071.65977}}},
            "price":0.9997797864679318,"priceSource":"defillama","chains":["Ethereum"]}]}"#;
        let resp: StablecoinsResponse = serde_json::from_str(json).unwrap();
        let usdt = &resp.pegged_assets[0];
        assert_eq!(usdt.gecko_id.as_deref(), Some("tether"));
        let circ = usdt.circulating.as_ref().unwrap();
        assert!((circ.pegged["peggedUSD"].0 - 183_727_681_018.321_4).abs() < 1.0);
        let prev = usdt.circulating_prev_day.as_ref().unwrap();
        assert!((prev.pegged["peggedUSD"].0 - 183_000_000_000.5).abs() < 1.0);
        let eth = &usdt.chain_circulating["Ethereum"];
        assert!(eth
            .current
            .as_ref()
            .unwrap()
            .pegged
            .contains_key("peggedUSD"));
        assert!((usdt.price.unwrap().0 - 0.99977).abs() < 1e-4);
    }

    #[test]
    fn circulating_supply_skips_non_numeric_entries() {
        let json = r#"{"peggedUSD":203897.05,"bridges":{"wormhole":{"BSC":{"amount":1.0}}}}"#;
        let c: CirculatingSupply = serde_json::from_str(json).unwrap();
        assert_eq!(c.pegged.len(), 1);
    }

    #[test]
    fn chart_dominance_and_prices_parse() {
        let chart: Vec<StablecoinChartPoint> = serde_json::from_str(
            r#"[{"date":"1790294400","totalCirculating":{"peggedUSD":1.5},
                 "totalCirculatingUSD":{"peggedUSD":2.5}}]"#,
        )
        .unwrap();
        assert_eq!(chart[0].date, 1_790_294_400);
        assert_eq!(
            chart[0].total_circulating_usd.as_ref().unwrap().pegged["peggedUSD"].0,
            2.5
        );

        let dom: Vec<StablecoinDominance> = serde_json::from_str(
            r#"[{"date":"1790294400","totalCirculatingUSD":{"peggedUSD":1.0},
                 "greatestMcap":{"gecko_id":"tether","symbol":"USDT","mcap":7.3e10}}]"#,
        )
        .unwrap();
        assert_eq!(
            dom[0].greatest_mcap.as_ref().unwrap().gecko_id.as_deref(),
            Some("tether")
        );

        let prices: StablecoinPricesResponse = serde_json::from_str(
            r#"[{"date":1790294400,"prices":{"tether":0.9998,"dead-coin":null}}]"#,
        )
        .unwrap();
        assert_eq!(prices.0[0].prices.len(), 1);
    }

    #[test]
    fn stablecoin_chains_parse_usd_key() {
        let chains: Vec<StablecoinChain> = serde_json::from_str(
            r#"[{"gecko_id":"manta","totalCirculatingUSD":{"peggedUSD":5877433.22},
                 "tokenSymbol":"MANTA","name":"Manta"}]"#,
        )
        .unwrap();
        assert!(chains[0].total_circulating_usd.is_some());
        assert_eq!(chains[0].gecko_id.as_deref(), Some("manta"));
    }
}
