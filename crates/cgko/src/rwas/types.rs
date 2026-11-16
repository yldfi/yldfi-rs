//! Types for real-world asset (RWA) endpoints
//!
//! Market figures are in USD and describe the aggregated onchain tokenized
//! market, not the underlying asset's spot market.

use crate::coins::CoinImage;
use crate::query::Query;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RWA list item (`/rwas/list`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaListItem {
    pub id: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    /// `stock`, `commodity` or `etf`
    pub asset_type: Option<String>,
}

/// Options for [`super::RwasApi::markets`]
#[derive(Debug, Clone, Default)]
pub struct RwaMarketsOptions {
    /// `stock`, `commodity` or `etf`
    pub asset_type: Option<String>,
    /// RWA IDs (up to 250); take priority over `names` and `symbols`
    pub ids: Option<Vec<String>>,
    /// RWA names; take priority over `symbols`
    pub names: Option<Vec<String>>,
    /// RWA symbols
    pub symbols: Option<Vec<String>>,
    /// Issuer ID (see [`super::RwasApi::issuers_list`])
    pub issuer: Option<String>,
    /// `market_cap_desc` (API default), `market_cap_asc`, `volume_asc`,
    /// `volume_desc`, `id_asc` or `id_desc`
    pub order: Option<String>,
    /// Results per page, 1 to 250 (API default: 100)
    pub per_page: Option<u32>,
    /// Page through results (API default: 1)
    pub page: Option<u32>,
    /// Include the 7-day sparkline (API default: false)
    pub sparkline: Option<bool>,
    /// Price change timeframes, comma-separated: `1h`, `24h`, `7d`, `14d`,
    /// `30d`, `200d`, `1y`
    pub price_change_percentage: Option<String>,
    /// Decimal places for prices: `full` or `0` to `18`
    pub precision: Option<String>,
}

impl RwaMarketsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only `stock`, `commodity` or `etf`
    #[must_use]
    pub fn asset_type(mut self, asset_type: impl Into<String>) -> Self {
        self.asset_type = Some(asset_type.into());
        self
    }

    /// Only these RWA IDs
    #[must_use]
    pub fn ids(mut self, ids: &[&str]) -> Self {
        self.ids = Some(ids.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Only these RWA names
    #[must_use]
    pub fn names(mut self, names: &[&str]) -> Self {
        self.names = Some(names.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Only these RWA symbols
    #[must_use]
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols = Some(symbols.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Only RWAs from this issuer
    #[must_use]
    pub fn issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Set the sort order
    #[must_use]
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Set the number of results per page (1 to 250)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Include the 7-day sparkline
    #[must_use]
    pub fn sparkline(mut self, sparkline: bool) -> Self {
        self.sparkline = Some(sparkline);
        self
    }

    /// Include price change for these timeframes, e.g. `1h,24h,7d`
    #[must_use]
    pub fn price_change_percentage(mut self, timeframes: impl Into<String>) -> Self {
        self.price_change_percentage = Some(timeframes.into());
        self
    }

    /// Decimal places for prices: `full` or `0` to `18`
    #[must_use]
    pub fn precision(mut self, precision: impl Into<String>) -> Self {
        self.precision = Some(precision.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        Query::new()
            .text("asset_type", self.asset_type.as_deref())
            .list("ids", self.ids.as_deref())
            .list("names", self.names.as_deref())
            .list("symbols", self.symbols.as_deref())
            .text("issuer", self.issuer.as_deref())
            .text("order", self.order.as_deref())
            .value("per_page", self.per_page)
            .value("page", self.page)
            .value("sparkline", self.sparkline)
            .text(
                "price_change_percentage",
                self.price_change_percentage.as_deref(),
            )
            .text("precision", self.precision.as_deref())
            .finish()
    }
}

/// RWA with market data (`/rwas/markets`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaMarket {
    pub id: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    /// `stock`, `commodity` or `etf`
    pub asset_type: Option<String>,
    /// Image URL
    pub image: Option<String>,
    pub tokenized_market_data: Option<TokenizedMarketData>,
}

/// Aggregated market data of the tokens tracking an RWA, in USD
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenizedMarketData {
    pub current_price: Option<f64>,
    pub market_cap: Option<f64>,
    pub total_volume: Option<f64>,
    pub high_24h: Option<f64>,
    pub low_24h: Option<f64>,
    pub price_change_24h: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub market_cap_change_percentage_24h: Option<f64>,
    /// ISO 8601 time
    pub last_updated: Option<String>,
    /// 7-day prices, when requested with `sparkline`
    #[serde(default)]
    pub sparkline_in_7d: Option<RwaSparkline>,
    #[serde(default)]
    pub price_change_percentage_1h_in_currency: Option<f64>,
    #[serde(default)]
    pub price_change_percentage_24h_in_currency: Option<f64>,
    #[serde(default)]
    pub price_change_percentage_7d_in_currency: Option<f64>,
    #[serde(default)]
    pub price_change_percentage_14d_in_currency: Option<f64>,
    #[serde(default)]
    pub price_change_percentage_30d_in_currency: Option<f64>,
    #[serde(default)]
    pub price_change_percentage_200d_in_currency: Option<f64>,
    #[serde(default)]
    pub price_change_percentage_1y_in_currency: Option<f64>,
}

/// 7-day sparkline
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaSparkline {
    #[serde(default)]
    pub price: Vec<f64>,
}

/// Perpetual futures market data, in USD
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PerpsMarketData {
    pub volume_24h: Option<f64>,
    pub open_interest_24h: Option<f64>,
}

/// Options for [`super::RwasApi::get_with_options`]
///
/// Without options `CoinGecko` returns metadata only.
#[derive(Debug, Clone, Default)]
pub struct RwaDataOptions {
    /// Include the 7-day sparkline (API default: false)
    pub sparkline: Option<bool>,
    /// Include the tokens tracking this RWA (API default: false)
    pub tokens: Option<bool>,
    /// Include tokenized market data (API default: false)
    pub tokenized_market_data: Option<bool>,
    /// Include perpetuals market data, per token too when `tokens` is set
    /// (API default: false)
    pub perps_market_data: Option<bool>,
}

impl RwaDataOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Include everything: sparkline, tokens, tokenized and perps market
    /// data
    #[must_use]
    pub fn all() -> Self {
        Self {
            sparkline: Some(true),
            tokens: Some(true),
            tokenized_market_data: Some(true),
            perps_market_data: Some(true),
        }
    }

    /// Include the 7-day sparkline
    #[must_use]
    pub fn sparkline(mut self, include: bool) -> Self {
        self.sparkline = Some(include);
        self
    }

    /// Include the tokens tracking this RWA
    #[must_use]
    pub fn tokens(mut self, include: bool) -> Self {
        self.tokens = Some(include);
        self
    }

    /// Include tokenized market data
    #[must_use]
    pub fn tokenized_market_data(mut self, include: bool) -> Self {
        self.tokenized_market_data = Some(include);
        self
    }

    /// Include perpetuals market data
    #[must_use]
    pub fn perps_market_data(mut self, include: bool) -> Self {
        self.perps_market_data = Some(include);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        Query::new()
            .value("sparkline", self.sparkline)
            .value("tokens", self.tokens)
            .value("tokenized_market_data", self.tokenized_market_data)
            .value("perps_market_data", self.perps_market_data)
            .finish()
    }
}

/// RWA data (`/rwas/{id}`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Rwa {
    pub id: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    /// `stock`, `commodity` or `etf`
    pub asset_type: Option<String>,
    pub image: Option<CoinImage>,
    /// Slug of the RWA's page on the `CoinGecko` website
    pub web_slug: Option<String>,
    /// Tokens tracking this RWA (with `tokens`)
    #[serde(default)]
    pub tokens: Vec<RwaToken>,
    /// With `tokenized_market_data`
    #[serde(default)]
    pub tokenized_market_data: Option<TokenizedMarketData>,
    /// Aggregate over all tokens (with `perps_market_data`)
    #[serde(default)]
    pub perps_market_data: Option<PerpsMarketData>,
    /// ISO 8601 time
    pub last_updated: Option<String>,
}

/// A token tracking an RWA; its ID is also a coin ID
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaToken {
    /// Coin ID (usable with the coins endpoints)
    pub id: String,
    pub symbol: Option<String>,
    pub name: Option<String>,
    /// Contract address per asset platform ID
    #[serde(default)]
    pub platforms: Option<HashMap<String, Option<String>>>,
    /// Issuer of the token (in RWA data, not issuer data)
    #[serde(default)]
    pub issuer_details: Option<RwaIssuerListItem>,
    /// Perpetuals market data of the token (RWA data with
    /// `perps_market_data`)
    #[serde(default)]
    pub perps_market_data: Option<PerpsMarketData>,
}

/// Options for [`super::RwasApi::market_chart`]
#[derive(Debug, Clone, Default)]
pub struct RwaMarketChartOptions {
    /// `daily`, `hourly` (past 100 days) or `5m` (past 10 days, Enterprise
    /// plan only); automatic when unset
    pub interval: Option<String>,
    /// Decimal places for prices: `full` or `0` to `18`
    pub precision: Option<String>,
}

impl RwaMarketChartOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// `daily`, `hourly` or `5m`
    #[must_use]
    pub fn interval(mut self, interval: impl Into<String>) -> Self {
        self.interval = Some(interval.into());
        self
    }

    /// Decimal places for prices: `full` or `0` to `18`
    #[must_use]
    pub fn precision(mut self, precision: impl Into<String>) -> Self {
        self.precision = Some(precision.into());
        self
    }

    pub(crate) fn add_to(&self, query: Query) -> Query {
        query
            .text("interval", self.interval.as_deref())
            .text("precision", self.precision.as_deref())
    }
}

/// RWA historical chart: `(unix_ms, value)` points, in USD
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaMarketChart {
    #[serde(default)]
    pub tokenized_prices: Vec<(f64, f64)>,
    #[serde(default)]
    pub tokenized_market_caps: Vec<(f64, f64)>,
    #[serde(default)]
    pub tokenized_total_volumes: Vec<(f64, f64)>,
}

/// Options for [`super::RwasApi::tickers_with_options`]
#[derive(Debug, Clone, Default)]
pub struct RwaTickersOptions {
    /// Only tokens from these issuers (see [`super::RwasApi::issuers_list`])
    pub issuer_ids: Option<Vec<String>>,
    /// Only these exchanges
    pub exchange_ids: Option<Vec<String>>,
    /// Page through results (100 tickers per page)
    pub page: Option<u32>,
    /// `volume_desc` (API default), `volume_asc`, `last_traded_at_desc` or
    /// `last_traded_at_asc`
    pub order: Option<String>,
    /// Include 2% order book depth (`cost_to_move_up_usd`,
    /// `cost_to_move_down_usd`; API default: false)
    pub depth: Option<bool>,
    /// `contract_address` (API default) or `symbol`: how DEX pair base and
    /// target are shown
    pub dex_pair_format: Option<String>,
}

impl RwaTickersOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only tokens from these issuers
    #[must_use]
    pub fn issuer_ids(mut self, ids: &[&str]) -> Self {
        self.issuer_ids = Some(ids.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Only these exchanges
    #[must_use]
    pub fn exchange_ids(mut self, ids: &[&str]) -> Self {
        self.exchange_ids = Some(ids.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the sort order
    #[must_use]
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    /// Include 2% order book depth
    #[must_use]
    pub fn depth(mut self, depth: bool) -> Self {
        self.depth = Some(depth);
        self
    }

    /// `contract_address` or `symbol`
    #[must_use]
    pub fn dex_pair_format(mut self, format: impl Into<String>) -> Self {
        self.dex_pair_format = Some(format.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        Query::new()
            .list("issuer_ids", self.issuer_ids.as_deref())
            .list("exchange_ids", self.exchange_ids.as_deref())
            .value("page", self.page)
            .text("order", self.order.as_deref())
            .value("depth", self.depth)
            .text("dex_pair_format", self.dex_pair_format.as_deref())
            .finish()
    }
}

/// Tickers of the tokens tracking an RWA (`/rwas/{id}/tickers`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaTickers {
    pub name: Option<String>,
    #[serde(default)]
    pub tickers: Vec<RwaTicker>,
}

/// A CEX or DEX ticker of a token tracking an RWA
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaTicker {
    pub base: Option<String>,
    pub target: Option<String>,
    pub market: Option<RwaTickerMarket>,
    pub last: Option<f64>,
    pub volume: Option<f64>,
    /// With `depth`
    #[serde(default)]
    pub cost_to_move_up_usd: Option<f64>,
    /// With `depth`
    #[serde(default)]
    pub cost_to_move_down_usd: Option<f64>,
    /// Last price in `btc`, `eth` and `usd`
    pub converted_last: Option<HashMap<String, f64>>,
    /// Volume in `btc`, `eth` and `usd`
    pub converted_volume: Option<HashMap<String, f64>>,
    pub bid_ask_spread_percentage: Option<f64>,
    pub timestamp: Option<String>,
    pub last_traded_at: Option<String>,
    pub last_fetch_at: Option<String>,
    pub is_anomaly: Option<bool>,
    pub is_stale: Option<bool>,
    pub trade_url: Option<String>,
    pub token_info_url: Option<String>,
    /// Coin ID of the token
    pub coin_id: Option<String>,
    #[serde(default)]
    pub target_coin_id: Option<String>,
    /// Market cap of the token in USD
    pub coin_mcap_usd: Option<f64>,
}

/// Exchange of an RWA ticker
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaTickerMarket {
    pub name: Option<String>,
    pub identifier: Option<String>,
    pub has_trading_incentive: Option<bool>,
    pub logo: Option<String>,
}

/// RWA issuer list item (`/rwas/issuers/list`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaIssuerListItem {
    pub id: String,
    pub name: Option<String>,
}

/// RWA issuer with market data and tokens (`/rwas/issuers/{id}`)
///
/// Market figures are summed over the issuer's tokens, in USD.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RwaIssuer {
    pub id: String,
    pub name: Option<String>,
    pub market_cap: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub volume_24h: Option<f64>,
    pub image: Option<CoinImage>,
    #[serde(default)]
    pub tokens: Vec<RwaToken>,
    /// ISO 8601 time
    pub updated_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markets_options_query() {
        assert_eq!(RwaMarketsOptions::new().to_query_string(), "");
        let q = RwaMarketsOptions::new()
            .asset_type("stock")
            .ids(&["gold", "ibm"])
            .names(&["Micron Technology"])
            .symbols(&["xau"])
            .issuer("xstocks-ecosystem")
            .order("volume_desc")
            .per_page(250)
            .page(2)
            .sparkline(true)
            .price_change_percentage("1h,24h")
            .precision("full")
            .to_query_string();
        assert_eq!(
            q,
            "?asset_type=stock&ids=gold,ibm&names=Micron+Technology&symbols=xau\
             &issuer=xstocks-ecosystem&order=volume_desc&per_page=250&page=2&sparkline=true\
             &price_change_percentage=1h%2C24h&precision=full"
        );
    }

    #[test]
    fn data_and_tickers_options_query() {
        assert_eq!(RwaDataOptions::new().to_query_string(), "");
        assert_eq!(
            RwaDataOptions::all().to_query_string(),
            "?sparkline=true&tokens=true&tokenized_market_data=true&perps_market_data=true"
        );
        assert_eq!(RwaTickersOptions::new().to_query_string(), "");
        let q = RwaTickersOptions::new()
            .issuer_ids(&["tether-ecosystem", "paxos-ecosystem"])
            .exchange_ids(&["uniswap_v3"])
            .page(2)
            .order("last_traded_at_desc")
            .depth(true)
            .dex_pair_format("symbol")
            .to_query_string();
        assert_eq!(
            q,
            "?issuer_ids=tether-ecosystem,paxos-ecosystem&exchange_ids=uniswap_v3&page=2\
             &order=last_traded_at_desc&depth=true&dex_pair_format=symbol"
        );
    }

    #[test]
    fn null_market_figures_read_as_none() {
        let m: TokenizedMarketData = serde_json::from_str(
            r#"{"current_price":null,"market_cap":5472900361,"last_updated":"2026-08-28T05:30:10Z"}"#,
        )
        .unwrap();
        assert!(m.current_price.is_none());
        assert_eq!(m.market_cap, Some(5_472_900_361.0));
        let i: RwaIssuer = serde_json::from_str(
            r#"{"id":"x","name":"X","market_cap":null,"market_cap_change_24h":null,
                "volume_24h":null,"updated_at":"2026-09-10T03:00:00Z"}"#,
        )
        .unwrap();
        assert!(i.market_cap.is_none() && i.tokens.is_empty());
    }
}
