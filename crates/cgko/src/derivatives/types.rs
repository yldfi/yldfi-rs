//! Types for derivatives endpoints

use serde::{Deserialize, Serialize};

/// Derivative ticker
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeTicker {
    pub market: Option<String>,
    pub symbol: Option<String>,
    pub index_id: Option<String>,
    /// Underlying coin ID (`None` for RWA underlyings)
    #[serde(default)]
    pub coin_id: Option<String>,
    /// Underlying RWA ID (`None` for coin underlyings)
    #[serde(default)]
    pub rwa_id: Option<String>,
    /// Derivatives exchange ID
    #[serde(default)]
    pub exchange_id: Option<String>,
    pub price: Option<String>,
    pub price_percentage_change_24h: Option<f64>,
    pub contract_type: Option<String>,
    pub index: Option<f64>,
    pub basis: Option<f64>,
    pub spread: Option<f64>,
    pub funding_rate: Option<f64>,
    /// Open interest, counted single-sided (each contract once) since 2026-11-10
    pub open_interest: Option<f64>,
    pub volume_24h: Option<f64>,
    pub last_traded_at: Option<u64>,
    /// Expiry time (Unix timestamp); `None` for perpetuals
    pub expired_at: Option<u64>,
}

/// Derivatives exchange list item
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchangeListItem {
    pub id: String,
    pub name: String,
}

/// Derivatives exchange data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchange {
    pub name: String,
    pub id: String,
    /// Open interest in BTC, counted single-sided (each contract once) since 2026-11-10
    pub open_interest_btc: Option<f64>,
    pub trade_volume_24h_btc: Option<String>,
    pub number_of_perpetual_pairs: Option<u32>,
    pub number_of_futures_pairs: Option<u32>,
    pub image: Option<String>,
    pub year_established: Option<u32>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
}

/// Detailed derivatives exchange data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchangeDetail {
    pub name: String,
    /// Open interest in BTC, counted single-sided (each contract once) since 2026-11-10
    pub open_interest_btc: Option<f64>,
    pub trade_volume_24h_btc: Option<String>,
    pub number_of_perpetual_pairs: Option<u32>,
    pub number_of_futures_pairs: Option<u32>,
    pub image: Option<String>,
    pub year_established: Option<u32>,
    pub country: Option<String>,
    pub description: Option<String>,
    pub url: Option<String>,
    pub tickers: Option<Vec<DerivativeExchangeTicker>>,
}

/// Derivative exchange ticker
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DerivativeExchangeTicker {
    pub symbol: Option<String>,
    pub base: Option<String>,
    pub target: Option<String>,
    /// Base coin ID (`None` for RWA underlyings)
    #[serde(default)]
    pub coin_id: Option<String>,
    /// Target coin ID
    #[serde(default)]
    pub target_coin_id: Option<String>,
    /// Underlying RWA ID (`None` for coin underlyings)
    #[serde(default)]
    pub rwa_id: Option<String>,
    pub trade_url: Option<String>,
    pub contract_type: Option<String>,
    pub last: Option<f64>,
    pub h24_percentage_change: Option<f64>,
    pub index: Option<f64>,
    pub index_basis_percentage: Option<f64>,
    pub bid_ask_spread: Option<f64>,
    pub funding_rate: Option<f64>,
    /// Open interest in USD, counted single-sided (each contract once) since 2026-11-10
    pub open_interest_usd: Option<f64>,
    pub h24_volume: Option<f64>,
    pub converted_volume: Option<ConvertedVolume>,
    pub converted_last: Option<ConvertedLast>,
    pub last_traded: Option<u64>,
    /// Expiry time (Unix timestamp); `None` for perpetuals
    pub expired_at: Option<u64>,
}

/// Converted volume
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertedVolume {
    /// Decimal string, as `CoinGecko` sends it
    pub btc: Option<String>,
    /// Decimal string, as `CoinGecko` sends it
    pub eth: Option<String>,
    /// Decimal string, as `CoinGecko` sends it
    pub usd: Option<String>,
}

/// Converted last price
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ConvertedLast {
    /// Decimal string, as `CoinGecko` sends it
    pub btc: Option<String>,
    /// Decimal string, as `CoinGecko` sends it
    pub eth: Option<String>,
    /// Decimal string, as `CoinGecko` sends it
    pub usd: Option<String>,
}

/// Options for derivatives exchanges query
#[derive(Debug, Clone, Default)]
pub struct DerivativeExchangesOptions {
    pub order: Option<String>,
    pub per_page: Option<u32>,
    pub page: Option<u32>,
}

impl DerivativeExchangesOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref o) = self.order {
            params.push(format!("order={o}"));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={pp}"));
        }
        if let Some(p) = self.page {
            params.push(format!("page={p}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Options for the derivatives tickers list (`/derivatives`)
///
/// Since 2026-11-20 02:00 UTC `CoinGecko` pages this endpoint: without
/// options it returns only the first 100 tickers.
#[derive(Debug, Clone, Default)]
pub struct DerivativeTickersOptions {
    /// Page through results (API default: 1)
    pub page: Option<u32>,
    /// Results per page, 1 to 250 (API default: 100)
    pub per_page: Option<u32>,
}

impl DerivativeTickersOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the number of results per page (1 to 250)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(p) = self.page {
            params.push(format!("page={p}"));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={pp}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Derivatives underlying asset (`/derivatives/underlyings/list`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnderlyingAsset {
    /// Underlying asset ID (coin ID or RWA ID)
    pub id: String,
    pub symbol: String,
    pub name: String,
    /// Underlying asset type: `coin` or `rwa`
    pub underlying_type: String,
}

/// Derivatives underlying asset with market data (`/derivatives/underlyings`)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UnderlyingAssetData {
    /// Underlying asset ID (coin ID or RWA ID)
    pub id: String,
    pub symbol: String,
    pub name: String,
    /// Underlying asset type: `coin` or `rwa`
    pub underlying_type: String,
    /// Underlying asset price in USD
    pub current_price: Option<f64>,
    pub price_change_percentage_24h: Option<f64>,
    /// Derivative open interest in USD
    pub open_interest: Option<f64>,
    pub open_interest_change_percentage_1h: Option<f64>,
    pub open_interest_change_percentage_24h: Option<f64>,
    pub funding_rate: Option<f64>,
    /// Derivative total trading volume in USD
    pub total_volume: Option<f64>,
}

/// Options for `/derivatives/underlyings` (Analyst plan and above)
#[derive(Debug, Clone, Default)]
pub struct UnderlyingsOptions {
    /// Coin IDs (see `/coins/list`)
    pub coin_ids: Option<Vec<String>>,
    /// RWA IDs (see `/rwas/list`)
    pub rwa_ids: Option<Vec<String>>,
    /// Filter by underlying type: `coin` or `rwa`
    pub underlying_type: Option<String>,
    /// Page through results (API default: 1)
    pub page: Option<u32>,
    /// Results per page, 1 to 250 (API default: 100)
    pub per_page: Option<u32>,
    /// Sort order: `open_interest_desc` (API default), `open_interest_asc`,
    /// `volume_desc`, `volume_asc`
    pub order: Option<String>,
}

impl UnderlyingsOptions {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only return these coin IDs
    #[must_use]
    pub fn coin_ids(mut self, ids: &[&str]) -> Self {
        self.coin_ids = Some(ids.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Only return these RWA IDs
    #[must_use]
    pub fn rwa_ids(mut self, ids: &[&str]) -> Self {
        self.rwa_ids = Some(ids.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Filter by underlying type (`coin` or `rwa`)
    #[must_use]
    pub fn underlying_type(mut self, underlying_type: impl Into<String>) -> Self {
        self.underlying_type = Some(underlying_type.into());
        self
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set the number of results per page (1 to 250)
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Set the sort order
    #[must_use]
    pub fn order(mut self, order: impl Into<String>) -> Self {
        self.order = Some(order.into());
        self
    }

    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(ref ids) = self.coin_ids {
            params.push(format!("coin_ids={}", ids.join(",")));
        }
        if let Some(ref ids) = self.rwa_ids {
            params.push(format!("rwa_ids={}", ids.join(",")));
        }
        if let Some(ref t) = self.underlying_type {
            params.push(format!("underlying_type={t}"));
        }
        if let Some(p) = self.page {
            params.push(format!("page={p}"));
        }
        if let Some(pp) = self.per_page {
            params.push(format!("per_page={pp}"));
        }
        if let Some(ref o) = self.order {
            params.push(format!("order={o}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tickers_options_query() {
        assert_eq!(DerivativeTickersOptions::new().to_query_string(), "");
        assert_eq!(
            DerivativeTickersOptions::new()
                .page(2)
                .per_page(250)
                .to_query_string(),
            "?page=2&per_page=250"
        );
    }

    #[test]
    fn underlyings_options_query() {
        let q = UnderlyingsOptions::new()
            .coin_ids(&["bitcoin", "ethereum"])
            .rwa_ids(&["gold"])
            .underlying_type("coin")
            .page(1)
            .per_page(50)
            .order("volume_desc")
            .to_query_string();
        assert_eq!(
            q,
            "?coin_ids=bitcoin,ethereum&rwa_ids=gold&underlying_type=coin&page=1&per_page=50&order=volume_desc"
        );
    }

    #[test]
    fn ticker_reads_underlying_and_exchange_ids() {
        let t: DerivativeTicker = serde_json::from_str(
            r#"{"market":"Example (Futures)","symbol":"XAUUSDT","index_id":"XAU",
                "coin_id":null,"rwa_id":"gold","exchange_id":"example_futures",
                "price":"3881.61","open_interest":491978285.74}"#,
        )
        .unwrap();
        assert!(t.coin_id.is_none());
        assert_eq!(t.rwa_id.as_deref(), Some("gold"));
        assert_eq!(t.exchange_id.as_deref(), Some("example_futures"));
    }

    #[test]
    fn exchange_ticker_matches_spec() {
        let t: DerivativeExchangeTicker = serde_json::from_str(
            r#"{"symbol":"BTCUSDT","base":"BTC","target":"USDT","coin_id":"bitcoin",
                "target_coin_id":"tether","rwa_id":null,"contract_type":"perpetual",
                "last":67208.2,"open_interest_usd":7.4e9,
                "converted_volume":{"btc":"114.65","eth":"4047.33","usd":"7710845.16"},
                "converted_last":{"btc":"1.0","eth":"35.3","usd":"67208.2"},
                "last_traded":1794655256,"expired_at":null}"#,
        )
        .unwrap();
        assert_eq!(t.coin_id.as_deref(), Some("bitcoin"));
        assert_eq!(t.target_coin_id.as_deref(), Some("tether"));
        assert_eq!(
            t.converted_volume.unwrap().usd.as_deref(),
            Some("7710845.16")
        );
    }

    #[test]
    fn ticker_expired_at_is_a_unix_timestamp() {
        let t: DerivativeTicker =
            serde_json::from_str(r#"{"symbol":"BTC-27NOV26","expired_at":1795071600}"#).unwrap();
        assert_eq!(t.expired_at, Some(1_795_071_600));
        let t: DerivativeTicker =
            serde_json::from_str(r#"{"symbol":"BTCUSDT","expired_at":null}"#).unwrap();
        assert!(t.expired_at.is_none());
    }

    #[test]
    fn underlyings_parse_documented_example() {
        let json = r#"[{"id":"bitcoin","symbol":"btc","name":"Bitcoin","underlying_type":"coin",
            "current_price":68708.23,"price_change_percentage_24h":-6.6178,
            "open_interest":45451655842.73,"open_interest_change_percentage_1h":0.40,
            "open_interest_change_percentage_24h":-0.63,"funding_rate":-0.00008,
            "total_volume":434673824874.8},
            {"id":"gold","symbol":"xau","name":"Gold","underlying_type":"rwa",
            "current_price":3898.78,"price_change_percentage_24h":-0.9,"open_interest":null,
            "open_interest_change_percentage_1h":-1.33,"open_interest_change_percentage_24h":3.70,
            "funding_rate":0.0121,"total_volume":null}]"#;
        let v: Vec<UnderlyingAssetData> = serde_json::from_str(json).unwrap();
        assert_eq!(v[0].underlying_type, "coin");
        assert_eq!(v[0].open_interest, Some(45_451_655_842.73));
        assert!(v[1].open_interest.is_none());
        assert!(v[1].total_volume.is_none());

        let list: Vec<UnderlyingAsset> = serde_json::from_str(
            r#"[{"id":"gold","symbol":"xau","name":"Gold","underlying_type":"rwa"}]"#,
        )
        .unwrap();
        assert_eq!(list[0].id, "gold");
    }
}
