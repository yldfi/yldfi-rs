//! Real-world asset (RWA) API endpoints

use super::types::{
    Rwa, RwaDataOptions, RwaIssuer, RwaIssuerListItem, RwaListItem, RwaMarket, RwaMarketChart,
    RwaMarketChartOptions, RwaMarketsOptions, RwaTickers, RwaTickersOptions,
};
use crate::client::Client;
use crate::error::Result;
use crate::query::Query;

/// Real-world asset (RWA) API
///
/// The list, markets, RWA data and issuer endpoints are available on every
/// plan (including the keyless and Demo API); tickers and the historical
/// chart need the Basic plan or above.
pub struct RwasApi<'a> {
    client: &'a Client,
}

impl<'a> RwasApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List every supported RWA (ID, symbol, name, asset type)
    ///
    /// # Arguments
    /// * `asset_type` - Optional filter: `stock`, `commodity` or `etf`
    pub async fn list(&self, asset_type: Option<&str>) -> Result<Vec<RwaListItem>> {
        let query = Query::new().text("asset_type", asset_type).finish();
        let path = format!("/rwas/list{query}");
        self.client.get(&path).await
    }

    /// List RWAs with price, market cap, volume and price changes
    pub async fn markets(&self, options: &RwaMarketsOptions) -> Result<Vec<RwaMarket>> {
        let path = format!("/rwas/markets{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get an RWA's metadata
    ///
    /// Use [`Self::get_with_options`] to include its tokens and market
    /// data.
    pub async fn get(&self, id: &str) -> Result<Rwa> {
        let path = format!("/rwas/{id}");
        self.client.get(&path).await
    }

    /// Get an RWA's metadata, with its tokens, tokenized market data and
    /// perpetuals market data as requested
    pub async fn get_with_options(&self, id: &str, options: &RwaDataOptions) -> Result<Rwa> {
        let path = format!("/rwas/{id}{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get CEX and DEX tickers of the tokens tracking an RWA, 100 per page
    /// (Basic plan and above)
    pub async fn tickers(&self, id: &str) -> Result<RwaTickers> {
        let path = format!("/rwas/{id}/tickers");
        self.client.get(&path).await
    }

    /// Get tickers of the tokens tracking an RWA, filtered and sorted
    /// (Basic plan and above)
    pub async fn tickers_with_options(
        &self,
        id: &str,
        options: &RwaTickersOptions,
    ) -> Result<RwaTickers> {
        let path = format!("/rwas/{id}/tickers{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get an RWA's historical tokenized price, market cap and volume
    /// (Basic plan and above; the Basic plan sees the past 2 years only)
    ///
    /// # Arguments
    /// * `id` - RWA ID
    /// * `days` - Days back from now, or `max`. Without an interval: 1 day
    ///   is 5-minutely, 2 to 90 days hourly, more daily.
    pub async fn market_chart(
        &self,
        id: &str,
        days: &str,
        options: &RwaMarketChartOptions,
    ) -> Result<RwaMarketChart> {
        let query = options
            .add_to(Query::new().text("days", Some(days)))
            .finish();
        let path = format!("/rwas/{id}/market_chart{query}");
        self.client.get(&path).await
    }

    /// List every RWA issuer (ID and name)
    pub async fn issuers_list(&self) -> Result<Vec<RwaIssuerListItem>> {
        self.client.get("/rwas/issuers/list").await
    }

    /// Get an issuer's market data and tokens
    pub async fn issuer(&self, id: &str) -> Result<RwaIssuer> {
        let path = format!("/rwas/issuers/{id}");
        self.client.get(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::Mock;

    const MARKET_DATA: &str = r#"{"current_price":4325.38,"market_cap":5226930984.54,
        "total_volume":370143676.04,"high_24h":4337.63,"low_24h":4265.22,
        "price_change_24h":31.5,"price_change_percentage_24h":0.73,
        "market_cap_change_24h":42665678.3,"market_cap_change_percentage_24h":0.82,
        "last_updated":"2026-09-16T06:49:00Z","sparkline_in_7d":{"price":[4398.79,4399.81]},
        "price_change_percentage_1h_in_currency":-0.07}"#;

    #[tokio::test]
    async fn list() {
        let mock =
            Mock::new(r#"[{"id":"gold","symbol":"XAU","name":"Gold","asset_type":"commodity"}]"#);
        let r = mock.client.rwas().list(Some("commodity")).await.unwrap();
        assert_eq!(mock.request(), "/rwas/list?asset_type=commodity");
        assert_eq!(r[0].asset_type.as_deref(), Some("commodity"));
    }

    #[tokio::test]
    async fn markets() {
        let body = format!(
            r#"[{{"id":"gold","symbol":"xau","name":"Gold","asset_type":"commodity",
                "image":"https://example.com/gold.png","tokenized_market_data":{MARKET_DATA}}}]"#
        );
        let mock = Mock::new(&body);
        let r = mock
            .client
            .rwas()
            .markets(&RwaMarketsOptions::new().ids(&["gold"]).sparkline(true))
            .await
            .unwrap();
        assert_eq!(mock.request(), "/rwas/markets?ids=gold&sparkline=true");
        let m = r[0].tokenized_market_data.as_ref().unwrap();
        assert_eq!(m.current_price, Some(4325.38));
        assert_eq!(m.sparkline_in_7d.as_ref().unwrap().price.len(), 2);
    }

    #[tokio::test]
    async fn get_with_options() {
        let body = format!(
            r#"{{"id":"gold","symbol":"xau","name":"Gold","asset_type":"commodity",
                "image":{{"thumb":"t","small":"s","large":"l"}},"web_slug":"gold",
                "tokens":[{{"id":"tether-gold","symbol":"xaut","name":"Tether Gold",
                    "platforms":{{"ethereum":"0x6874"}},
                    "issuer_details":{{"id":"tether-ecosystem","name":"Tether"}},
                    "perps_market_data":{{"volume_24h":1954758829.5,"open_interest_24h":null}}}}],
                "tokenized_market_data":{MARKET_DATA},
                "perps_market_data":{{"volume_24h":7416589039.87,"open_interest_24h":4536346045.72}},
                "last_updated":"2026-09-16T06:49:00Z"}}"#
        );
        let mock = Mock::new(&body);
        let r = mock
            .client
            .rwas()
            .get_with_options("gold", &RwaDataOptions::new().tokens(true))
            .await
            .unwrap();
        assert_eq!(mock.request(), "/rwas/gold?tokens=true");
        let token = &r.tokens[0];
        assert_eq!(
            token.issuer_details.as_ref().unwrap().id,
            "tether-ecosystem"
        );
        assert!(token
            .perps_market_data
            .as_ref()
            .unwrap()
            .open_interest_24h
            .is_none());
        assert_eq!(
            r.perps_market_data.unwrap().volume_24h,
            Some(7_416_589_039.87)
        );

        let mock = Mock::new(r#"{"id":"gold","symbol":"xau","name":"Gold"}"#);
        let r = mock.client.rwas().get("gold").await.unwrap();
        assert_eq!(mock.request(), "/rwas/gold");
        assert!(r.tokens.is_empty() && r.tokenized_market_data.is_none());
    }

    #[tokio::test]
    async fn tickers() {
        let mock = Mock::new(
            r#"{"name":"Gold","tickers":[{"base":"XAUT","target":"USD",
                "market":{"name":"Example Exchange","identifier":"example","has_trading_incentive":false,
                    "logo":"https://example.com/logo.png"},
                "last":4567.32,"volume":153.448,"cost_to_move_up_usd":542225.42,
                "cost_to_move_down_usd":521806.08,"converted_last":{"btc":0.057,"eth":1.83,"usd":4567.14},
                "converted_volume":{"btc":8.8,"eth":281.3,"usd":700818},
                "bid_ask_spread_percentage":0.013,"timestamp":"2026-08-28T05:38:53+00:00",
                "last_traded_at":"2026-08-28T05:38:53+00:00","last_fetch_at":"2026-08-28T05:38:40+00:00",
                "is_anomaly":false,"is_stale":false,"trade_url":null,"token_info_url":null,
                "coin_id":"tether-gold","coin_mcap_usd":2798942188.93}]}"#,
        );
        let r = mock
            .client
            .rwas()
            .tickers_with_options(
                "gold",
                &RwaTickersOptions::new().depth(true).order("volume_asc"),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            "/rwas/gold/tickers?order=volume_asc&depth=true"
        );
        let t = &r.tickers[0];
        assert_eq!(t.cost_to_move_up_usd, Some(542_225.42));
        assert_eq!(t.converted_volume.as_ref().unwrap()["usd"], 700_818.0);
        assert!(t.target_coin_id.is_none());

        let mock = Mock::new(r#"{"name":"Gold","tickers":[]}"#);
        mock.client.rwas().tickers("gold").await.unwrap();
        assert_eq!(mock.request(), "/rwas/gold/tickers");
    }

    #[tokio::test]
    async fn market_chart() {
        let mock = Mock::new(
            r#"{"tokenized_prices":[[1787806800000,4598.15]],
                "tokenized_market_caps":[[1787806800000,5494949679.3]],
                "tokenized_total_volumes":[[1787806800000,391917834.0]]}"#,
        );
        let r = mock
            .client
            .rwas()
            .market_chart(
                "gold",
                "30",
                &RwaMarketChartOptions::new()
                    .interval("daily")
                    .precision("2"),
            )
            .await
            .unwrap();
        assert_eq!(
            mock.request(),
            "/rwas/gold/market_chart?days=30&interval=daily&precision=2"
        );
        assert_eq!(r.tokenized_prices[0], (1_787_806_800_000.0, 4598.15));
    }

    #[tokio::test]
    async fn issuers() {
        let mock = Mock::new(r#"[{"id":"xstocks-ecosystem","name":"xStocks"}]"#);
        let r = mock.client.rwas().issuers_list().await.unwrap();
        assert_eq!(mock.request(), "/rwas/issuers/list");
        assert_eq!(r[0].id, "xstocks-ecosystem");

        let mock = Mock::new(
            r#"{"id":"xstocks-ecosystem","name":"xStocks","market_cap":752085126.66,
                "market_cap_change_24h":-79476493.49,"volume_24h":240421573.38,
                "image":{"thumb":"t","small":"s","large":"l"},
                "tokens":[{"id":"alphabet-xstock","symbol":"googlx","name":"Alphabet xStock",
                    "platforms":{"ethereum":"0xe92f","solana":"XsCP"}}],
                "updated_at":"2026-09-10T03:00:00Z"}"#,
        );
        let r = mock
            .client
            .rwas()
            .issuer("xstocks-ecosystem")
            .await
            .unwrap();
        assert_eq!(mock.request(), "/rwas/issuers/xstocks-ecosystem");
        assert_eq!(r.market_cap, Some(752_085_126.66));
        let platforms = r.tokens[0].platforms.as_ref().unwrap();
        assert_eq!(platforms["solana"].as_deref(), Some("XsCP"));
    }
}
