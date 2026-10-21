//! Types for ecosystem data (Pro)

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Deserialize a Vec that may be null or missing as an empty Vec
fn null_to_empty_vec<'de, D, T>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<Vec<T>>::deserialize(deserializer).map(std::option::Option::unwrap_or_default)
}

pub use crate::tvl::{ChainAssets, ProtocolInflows, TokenProtocol};

/// TVL components of one chart cell (one name on one day)
///
/// Which components are present depends on the protocols involved.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TvlComponents {
    /// TVL in USD
    pub tvl: Option<f64>,
    /// Staking TVL
    pub staking: Option<f64>,
    /// Pool2 TVL
    pub pool2: Option<f64>,
    /// Double-counted TVL
    pub doublecounted: Option<f64>,
    /// Borrowed amount
    pub borrowed: Option<f64>,
    /// Offers
    pub offers: Option<f64>,
    /// Vesting
    pub vesting: Option<f64>,
    /// Treasury
    pub treasury: Option<f64>,
    /// Liquid staking TVL
    pub liquidstaking: Option<f64>,
    /// Overlap between double-counted and liquid staking TVL
    pub dc_and_ls_overlap: Option<f64>,
}

/// Historical chart: Unix timestamp (as string key) -> name -> TVL components
pub type TvlChart = HashMap<String, HashMap<String, TvlComponents>>;

/// TVL by category (`/api/categories`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct CategoriesResponse {
    /// Protocol names keyed by category
    #[serde(default)]
    pub categories: HashMap<String, Vec<String>>,
    /// Daily TVL per category
    #[serde(default)]
    pub chart: TvlChart,
}

/// Protocol fork relationships (`/api/forks`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ForksResponse {
    /// Forking protocol names keyed by the original protocol
    #[serde(default)]
    pub forks: HashMap<String, Vec<String>>,
    /// Daily TVL per original protocol (summed over its forks)
    #[serde(default)]
    pub chart: TvlChart,
}

/// Oracle usage (`/api/oracles`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OraclesResponse {
    /// Protocol names keyed by oracle
    #[serde(default)]
    pub oracles: HashMap<String, Vec<String>>,
    /// Chain names keyed by oracle
    #[serde(default)]
    pub chains_by_oracle: HashMap<String, Vec<String>>,
    /// Daily TVL secured per oracle
    #[serde(default)]
    pub chart: TvlChart,
}

/// Entity/company information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    /// Entity name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Protocols owned/affiliated
    #[serde(default)]
    pub protocols: Vec<String>,
    /// Total TVL
    pub tvl: Option<f64>,
}

/// Protocol treasury data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Treasury {
    /// Protocol name
    pub name: String,
    /// Token holdings
    #[serde(default)]
    pub tokens: HashMap<String, TreasuryToken>,
    /// Total value in USD
    pub total_value_usd: Option<f64>,
    /// Own token value
    pub own_token_value: Option<f64>,
    /// Other token value
    pub other_token_value: Option<f64>,
    /// Stablecoin value
    pub stablecoin_value: Option<f64>,
}

/// Treasury token holding
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreasuryToken {
    /// Token symbol
    pub symbol: Option<String>,
    /// Amount held
    pub amount: Option<f64>,
    /// Value in USD
    pub value_usd: Option<f64>,
}

/// Hack/exploit data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Hack {
    /// Protocol name
    pub name: String,
    /// Date of hack (Unix timestamp)
    #[serde(default, deserialize_with = "crate::de::opt_flex_u64")]
    pub date: Option<u64>,
    /// Chains affected
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub chain: Vec<String>,
    /// Classification (e.g. protocol logic, infrastructure)
    pub classification: Option<String>,
    /// Hack technique
    pub technique: Option<String>,
    /// Amount lost in USD
    pub amount: Option<f64>,
    /// Whether this was a bridge hack
    pub bridge_hack: Option<bool>,
    /// Target type (e.g. DEX, lending)
    pub target_type: Option<String>,
    /// Link to more info
    pub source: Option<String>,
    /// Amount of funds returned in USD
    pub returned_funds: Option<f64>,
    /// `DefiLlama` protocol ID, if listed
    pub defillama_id: Option<serde_json::Value>,
    /// Contract language
    pub language: Option<String>,
}

/// Funding rounds response (`/api/raises`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct RaisesResponse {
    /// Funding rounds
    #[serde(default)]
    pub raises: Vec<Raise>,
}

/// Funding raise data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Raise {
    /// Protocol name
    pub name: String,
    /// Round type (Seed, Series A, etc.)
    pub round: Option<String>,
    /// Amount raised in millions of USD
    pub amount: Option<f64>,
    /// Date of raise (Unix timestamp)
    #[serde(default, deserialize_with = "crate::de::opt_flex_u64")]
    pub date: Option<u64>,
    /// Lead investors
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub lead_investors: Vec<String>,
    /// Other investors
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub other_investors: Vec<String>,
    /// Valuation in millions of USD
    pub valuation: Option<f64>,
    /// Chains
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub chains: Vec<String>,
    /// Sector description
    pub sector: Option<String>,
    /// Category
    pub category: Option<String>,
    /// Category group
    pub category_group: Option<String>,
    /// Source link
    pub source: Option<String>,
    /// `DefiLlama` protocol ID, if listed
    pub defillama_id: Option<String>,
}

/// Historical liquidity data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityData {
    /// Token symbol
    pub symbol: String,
    /// Historical data points
    #[serde(default)]
    pub data: Vec<LiquidityPoint>,
}

/// Liquidity data point
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidityPoint {
    /// Unix timestamp
    pub date: u64,
    /// Liquidity in USD
    pub liquidity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn categories_response_parses() {
        let json = r#"{"categories":{"Dexs":["Foo Swap","Bar AMM"]},
            "chart":{"1700000000":{"Dexs":{"tvl":1000.5,"staking":2.0,"dcAndLsOverlap":3.0}}}}"#;
        let r: CategoriesResponse = serde_json::from_str(json).unwrap();
        assert_eq!(r.categories["Dexs"], vec!["Foo Swap", "Bar AMM"]);
        let cell = &r.chart["1700000000"]["Dexs"];
        assert_eq!(cell.tvl, Some(1000.5));
        assert_eq!(cell.dc_and_ls_overlap, Some(3.0));
        assert!(cell.pool2.is_none());
    }

    #[test]
    fn forks_and_oracles_parse() {
        let f: ForksResponse = serde_json::from_str(
            r#"{"forks":{"Foo V2":["Bar Swap"]},"chart":{"1700000000":{"Foo V2":{"tvl":5}}}}"#,
        )
        .unwrap();
        assert_eq!(f.forks["Foo V2"], vec!["Bar Swap"]);
        assert_eq!(f.chart["1700000000"]["Foo V2"].tvl, Some(5.0));

        let o: OraclesResponse = serde_json::from_str(
            r#"{"oracles":{"FooOracle":["Bar Lend"]},"chainsByOracle":{"FooOracle":["ExampleChain"]},
                "chart":{"1700000000":{"FooOracle":{"tvl":7,"borrowed":1}}}}"#,
        )
        .unwrap();
        assert_eq!(o.oracles["FooOracle"], vec!["Bar Lend"]);
        assert_eq!(o.chains_by_oracle["FooOracle"], vec!["ExampleChain"]);
        assert_eq!(o.chart["1700000000"]["FooOracle"].borrowed, Some(1.0));
    }

    #[test]
    fn raises_response_parses() {
        let json = r#"{"raises":[{"date":1700000000,"name":"Foo Labs","round":"Seed","amount":16,
            "chains":["ExampleChain"],"sector":"Example sector","category":"DeFi",
            "categoryGroup":"DeFi & CeFi","source":"https://example.com/news",
            "leadInvestors":["Lead Fund"],"otherInvestors":["Other Fund"],"valuation":null,
            "defillamaId":"parent#foo"}]}"#;
        let r: RaisesResponse = serde_json::from_str(json).unwrap();
        let raise = &r.raises[0];
        assert_eq!(raise.date, Some(1_700_000_000));
        assert_eq!(raise.other_investors, vec!["Other Fund"]);
        assert_eq!(raise.category_group.as_deref(), Some("DeFi & CeFi"));
        assert!(raise.valuation.is_none());
    }

    #[test]
    fn hacks_parse_numeric_date_and_chain_array() {
        let json = r#"[{"date":1700000000,"name":"Foo Protocol","classification":"Protocol Logic",
            "technique":"Example","amount":4800000,"chain":["ExampleChain"],"bridgeHack":false,
            "targetType":"Gaming","source":"https://example.com/post","returnedFunds":null,
            "defillamaId":null,"language":"Solidity"}]"#;
        let h: Vec<Hack> = serde_json::from_str(json).unwrap();
        assert_eq!(h[0].date, Some(1_700_000_000));
        assert_eq!(h[0].chain, vec!["ExampleChain"]);
        assert_eq!(h[0].bridge_hack, Some(false));
    }

    #[test]
    fn token_protocols_and_inflows_parse() {
        let tp: Vec<TokenProtocol> = serde_json::from_str(
            r#"[{"name":"Foo Bridge","category":"Bridge","amountUsd":{"coingecko:foo":12.5}}]"#,
        )
        .unwrap();
        assert_eq!(tp[0].amount_usd["coingecko:foo"], 12.5);

        let i: ProtocolInflows = serde_json::from_str(
            r#"{"outflows":-10.5,"oldTokens":{"date":1700000000,"tvl":{"FOO":1.0}},
                "currentTokens":{"date":1700086400,"tvl":{"FOO":2.0}}}"#,
        )
        .unwrap();
        assert_eq!(i.outflows, Some(-10.5));
        assert_eq!(i.current_tokens.unwrap().tvl["FOO"], 2.0);
    }
}
