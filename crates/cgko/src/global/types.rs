//! Types for global data endpoints

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// Deserialize an optional ID that CoinGecko sends as either a string or an
/// integer (e.g. trending categories use numeric IDs like `102120914`).
fn string_or_number_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    match Option::<serde_json::Value>::deserialize(deserializer)? {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(s)) => Ok(Some(s)),
        Some(serde_json::Value::Number(n)) => Ok(Some(n.to_string())),
        Some(other) => Err(D::Error::custom(format!(
            "expected string or number id, got {other}"
        ))),
    }
}

/// Global data response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalResponse {
    pub data: GlobalData,
}

/// Global market data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GlobalData {
    pub active_cryptocurrencies: Option<u64>,
    pub upcoming_icos: Option<u64>,
    pub ongoing_icos: Option<u64>,
    pub ended_icos: Option<u64>,
    pub markets: Option<u64>,
    pub total_market_cap: Option<HashMap<String, f64>>,
    pub total_volume: Option<HashMap<String, f64>>,
    pub market_cap_percentage: Option<HashMap<String, f64>>,
    pub market_cap_change_percentage_24h_usd: Option<f64>,
    pub updated_at: Option<u64>,
}

/// `DeFi` global data response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefiGlobalResponse {
    pub data: DefiGlobalData,
}

/// `DeFi` global data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DefiGlobalData {
    pub defi_market_cap: Option<String>,
    pub eth_market_cap: Option<String>,
    pub defi_to_eth_ratio: Option<String>,
    pub trading_volume_24h: Option<String>,
    pub defi_dominance: Option<String>,
    pub top_coin_name: Option<String>,
    pub top_coin_defi_dominance: Option<f64>,
}

/// Trending response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrendingResponse {
    pub coins: Vec<TrendingCoinItem>,
    #[serde(default)]
    pub nfts: Vec<TrendingNft>,
    #[serde(default)]
    pub categories: Vec<TrendingCategory>,
}

/// Trending coin wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrendingCoinItem {
    pub item: TrendingCoin,
}

/// Trending coin
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrendingCoin {
    pub id: String,
    pub coin_id: Option<u64>,
    pub name: String,
    pub symbol: String,
    pub market_cap_rank: Option<u32>,
    pub thumb: Option<String>,
    pub small: Option<String>,
    pub large: Option<String>,
    pub slug: Option<String>,
    pub price_btc: Option<f64>,
    pub score: Option<u32>,
}

/// Trending NFT
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrendingNft {
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub thumb: Option<String>,
}

/// Trending category
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TrendingCategory {
    /// Category ID (the API sends an integer; strings are also accepted)
    #[serde(default, deserialize_with = "string_or_number_opt")]
    pub id: Option<String>,
    pub name: Option<String>,
    pub market_cap_1h_change: Option<f64>,
    pub slug: Option<String>,
}

/// Search response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchResponse {
    pub coins: Vec<SearchCoin>,
    pub exchanges: Vec<SearchExchange>,
    pub categories: Vec<SearchCategory>,
    pub nfts: Vec<SearchNft>,
}

/// Search coin result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchCoin {
    pub id: String,
    pub name: String,
    pub api_symbol: Option<String>,
    pub symbol: String,
    pub market_cap_rank: Option<u32>,
    pub thumb: Option<String>,
    pub large: Option<String>,
}

/// Search exchange result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchExchange {
    pub id: Option<String>,
    pub name: Option<String>,
    pub market_type: Option<String>,
    pub thumb: Option<String>,
    pub large: Option<String>,
}

/// Search category result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchCategory {
    /// Category ID (string or integer depending on endpoint version)
    #[serde(default, deserialize_with = "string_or_number_opt")]
    pub id: Option<String>,
    pub name: Option<String>,
}

/// Search NFT result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SearchNft {
    pub id: Option<String>,
    pub name: Option<String>,
    pub symbol: Option<String>,
    pub thumb: Option<String>,
}

/// Ping response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PingResponse {
    pub gecko_says: String,
}

/// Exchange rates response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeRatesResponse {
    pub rates: HashMap<String, ExchangeRate>,
}

/// Exchange rate
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExchangeRate {
    pub name: String,
    pub unit: String,
    pub value: f64,
    #[serde(rename = "type")]
    pub rate_type: String,
}

/// Asset platform
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetPlatform {
    pub id: String,
    pub chain_identifier: Option<i64>,
    pub name: String,
    pub shortname: Option<String>,
    pub native_coin_id: Option<String>,
}

/// API key usage response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiKeyUsage {
    pub plan: Option<String>,
    pub rate_limit_request_per_minute: Option<u32>,
    pub monthly_call_credit: Option<u64>,
    pub current_total_monthly_calls: Option<u64>,
    pub current_remaining_monthly_calls: Option<u64>,
}

/// Global market cap chart data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketCapChart {
    pub market_cap_chart: MarketCapChartData,
}

/// Market cap chart data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MarketCapChartData {
    pub market_cap: Vec<(u64, f64)>,
    pub volume: Option<Vec<(u64, f64)>>,
}

/// Token list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenList {
    pub name: Option<String>,
    pub logo_uri: Option<String>,
    pub keywords: Option<Vec<String>>,
    pub timestamp: Option<String>,
    pub tokens: Vec<TokenListItem>,
}

/// Token list item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenListItem {
    pub chain_id: Option<i64>,
    pub address: String,
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "logoURI")]
    pub logo_uri: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trending_category_accepts_integer_and_string_ids() {
        // Live shape from /search/trending (id is an integer)
        let c: TrendingCategory = serde_json::from_str(
            r#"{"id":102120914,"name":"Robinhood Chain Meme","market_cap_1h_change":5.8,
                "slug":"robinhood-chain-meme","coins_count":"336"}"#,
        )
        .unwrap();
        assert_eq!(c.id.as_deref(), Some("102120914"));
        let c: TrendingCategory = serde_json::from_str(r#"{"id":"defi"}"#).unwrap();
        assert_eq!(c.id.as_deref(), Some("defi"));
        let c: TrendingCategory = serde_json::from_str(r#"{"id":null}"#).unwrap();
        assert!(c.id.is_none());
        let c: TrendingCategory = serde_json::from_str(r#"{}"#).unwrap();
        assert!(c.id.is_none());
    }

    #[test]
    fn search_category_accepts_integer_ids() {
        let c: SearchCategory = serde_json::from_str(r#"{"id":42,"name":"x"}"#).unwrap();
        assert_eq!(c.id.as_deref(), Some("42"));
    }
}
