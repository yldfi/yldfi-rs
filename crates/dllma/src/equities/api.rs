//! Equities API endpoints (Pro)

use std::collections::HashMap;

use crate::client::{with_query, Client};
use crate::error::Result;

use super::types::{
    Candle, Company, DatedValue, DimensionHistory, EarningsEvent, EarningsQuery, EquitySummary,
    Filing, FinancialStatements, Industry, OnchainMarkets, Sector, SectorDetail, SectorsOverview,
    Timeframe,
};

/// Public company (equities) API client (Pro only)
///
/// Companies are identified by ticker plus ISO 3166-1 alpha-2 country code
/// (both case-insensitive), as listed by [`Self::companies`].
pub struct EquitiesApi<'a> {
    client: &'a Client,
}

impl<'a> EquitiesApi<'a> {
    /// Create a new equities API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn company<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        ticker: &str,
        country: &str,
        timeframe: Option<Timeframe>,
    ) -> Result<T> {
        let path = with_query(
            format!("/equities/v1/{endpoint}"),
            [
                ("ticker", Some(ticker)),
                ("country", Some(country)),
                ("timeframe", timeframe.map(Timeframe::as_str)),
            ],
        );
        self.client.get_pro(&path).await
    }

    /// List tracked public companies, optionally in one sector or industry
    /// (`GET /equities/v1/companies-list`)
    ///
    /// **Requires Pro API key**
    ///
    /// `sector` and `industry` take a slug (`financial-services`) or label.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let companies = client.equities().companies(Some("technology"), None).await?;
    /// for c in companies.iter().take(5) {
    ///     println!("{} ({:?})", c.ticker, c.company_name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn companies(
        &self,
        sector: Option<&str>,
        industry: Option<&str>,
    ) -> Result<Vec<Company>> {
        let path = with_query(
            "/equities/v1/companies-list".to_owned(),
            [("sector", sector), ("industry", industry)],
        );
        self.client.get_pro(&path).await
    }

    /// Get a company's income statement, balance sheet and cash flow
    /// (`GET /equities/v1/statements`)
    ///
    /// **Requires Pro API key**
    pub async fn statements(&self, ticker: &str, country: &str) -> Result<FinancialStatements> {
        self.company("statements", ticker, country, None).await
    }

    /// Get a company's financial dimensions (revenue, earnings, shares
    /// outstanding, ...), keyed by name (`GET /equities/v1/dimensions`)
    ///
    /// **Requires Pro API key**
    pub async fn dimensions(
        &self,
        ticker: &str,
        country: &str,
    ) -> Result<HashMap<String, DimensionHistory>> {
        self.company("dimensions", ticker, country, None).await
    }

    /// Get a company's closing prices, newest first
    /// (`GET /equities/v1/price-history`)
    ///
    /// **Requires Pro API key**
    ///
    /// `timeframe` defaults to all history.
    pub async fn price_history(
        &self,
        ticker: &str,
        country: &str,
        timeframe: Option<Timeframe>,
    ) -> Result<Vec<DatedValue>> {
        self.company("price-history", ticker, country, timeframe)
            .await
    }

    /// Get a company's OHLCV candles, newest first
    /// (`GET /equities/v1/ohlcv`)
    ///
    /// **Requires Pro API key**
    ///
    /// `timeframe` defaults to all history.
    pub async fn ohlcv(
        &self,
        ticker: &str,
        country: &str,
        timeframe: Option<Timeframe>,
    ) -> Result<Vec<Candle>> {
        self.company("ohlcv", ticker, country, timeframe).await
    }

    /// Get a company's live market data and trailing financials
    /// (`GET /equities/v1/summary`)
    ///
    /// **Requires Pro API key**
    pub async fn summary(&self, ticker: &str, country: &str) -> Result<EquitySummary> {
        self.company("summary", ticker, country, None).await
    }

    /// List a company's regulatory filings (`GET /equities/v1/filings`)
    ///
    /// **Requires Pro API key**
    pub async fn filings(&self, ticker: &str, country: &str) -> Result<Vec<Filing>> {
        self.company("filings", ticker, country, None).await
    }

    /// List onchain perps and tokens tracking a company's stock
    /// (`GET /equities/v1/onchain`)
    ///
    /// **Requires Pro API key**
    pub async fn onchain(&self, ticker: &str, country: &str) -> Result<OnchainMarkets> {
        self.company("onchain", ticker, country, None).await
    }

    /// List sectors with aggregate market data (`GET /equities/v1/sectors`)
    ///
    /// **Requires Pro API key**
    pub async fn sectors(&self) -> Result<Vec<Sector>> {
        self.client.get_pro("/equities/v1/sectors").await
    }

    /// Get market-wide aggregates across sectors
    /// (`GET /equities/v1/sectors-overview`)
    ///
    /// **Requires Pro API key**
    pub async fn sectors_overview(&self) -> Result<SectorsOverview> {
        self.client.get_pro("/equities/v1/sectors-overview").await
    }

    /// Get one sector in detail, by slug or label
    /// (`GET /equities/v1/sector`)
    ///
    /// **Requires Pro API key**
    pub async fn sector(&self, sector: &str) -> Result<SectorDetail> {
        let path = with_query("/equities/v1/sector".to_owned(), [("sector", Some(sector))]);
        self.client.get_pro(&path).await
    }

    /// List industries, optionally in one sector or only one industry
    /// (`GET /equities/v1/industries`)
    ///
    /// **Requires Pro API key**
    pub async fn industries(
        &self,
        sector: Option<&str>,
        industry: Option<&str>,
    ) -> Result<Vec<Industry>> {
        let path = with_query(
            "/equities/v1/industries".to_owned(),
            [("sector", sector), ("industry", industry)],
        );
        self.client.get_pro(&path).await
    }

    /// List earnings dates (`GET /equities/v1/earnings-calendar`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::equities::{EarningsQuery, EarningsStatus};
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let query = EarningsQuery::new()
    ///     .from("2026-11-01")
    ///     .status(EarningsStatus::Estimated)
    ///     .limit(100);
    /// let events = client.equities().earnings_calendar(&query).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn earnings_calendar(&self, query: &EarningsQuery) -> Result<Vec<EarningsEvent>> {
        let limit = query.limit.map(|l| l.to_string());
        let path = with_query(
            "/equities/v1/earnings-calendar".to_owned(),
            [
                ("ticker", query.ticker.as_deref()),
                ("from", query.from.as_deref()),
                ("to", query.to.as_deref()),
                ("status", query.status.map(super::EarningsStatus::as_str)),
                ("limit", limit.as_deref()),
            ],
        );
        self.client.get_pro(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::test_server::TestServer;

    fn pro(body: &str) -> (Client, TestServer) {
        let server = TestServer::json(body);
        let client = Client::with_api_key("KEY")
            .unwrap()
            .with_test_base(&server.url);
        (client, server)
    }

    #[tokio::test]
    async fn needs_api_key() {
        let client = Client::new().unwrap();
        let err = client.equities().sectors().await.unwrap_err();
        assert!(matches!(err, Error::Api { status: 401, .. }), "{err}");
    }

    #[tokio::test]
    async fn companies() {
        let (client, server) = pro(
            r#"[{"ticker":"NVDA","companyName":"NVIDIA","country":"US","countryName":"United States",
            "sectorSlug":"technology","industrySlug":"semiconductors"}]"#,
        );
        let c = client
            .equities()
            .companies(Some("Financial Services"), Some("banks"))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/companies-list?sector=Financial+Services&industry=banks"
        );
        assert_eq!(c[0].ticker, "NVDA");
        assert_eq!(c[0].industry_slug.as_deref(), Some("semiconductors"));

        let (client, server) = pro("[]");
        client.equities().companies(None, None).await.unwrap();
        assert_eq!(server.path(), "/KEY/equities/v1/companies-list");
    }

    #[tokio::test]
    async fn statements() {
        let (client, server) = pro(
            r#"{"incomeStatement":{"labels":["Revenue","Operating Expenses"],
            "children":{"quarterly":{"Operating Expenses":{"labels":["R&D"]}},"annual":{}},
            "quarterly":{"periodEnding":["2026-03-31"],"values":[[10],[null]],
                "children":{"Operating Expenses":{"values":[[2]]}}},
            "annual":{"periodEnding":[],"values":[]}}}"#,
        );
        let s = client.equities().statements("AAPL", "US").await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/statements?ticker=AAPL&country=US"
        );
        let i = &s.income_statement;
        assert_eq!(i.labels[1], "Operating Expenses");
        assert_eq!(i.children.quarterly["Operating Expenses"].labels, ["R&D"]);
        assert_eq!(i.quarterly.period_ending, ["2026-03-31"]);
        assert_eq!(i.quarterly.values, vec![vec![Some(10.0)], vec![None]]);
        assert_eq!(
            i.quarterly.children["Operating Expenses"].values,
            vec![vec![Some(2.0)]]
        );
        assert!(s.balance_sheet.labels.is_empty());
    }

    #[tokio::test]
    async fn dimensions() {
        let (client, server) =
            pro(r#"{"revenue":{"annual":[["2025-12-31",7181325000]],"quarterly":[]}}"#);
        let d = client.equities().dimensions("COIN", "us").await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/dimensions?ticker=COIN&country=us"
        );
        assert_eq!(d["revenue"].annual[0].date, "2025-12-31");
        assert_eq!(d["revenue"].annual[0].value, Some(7_181_325_000.0));
    }

    #[tokio::test]
    async fn price_history() {
        let (client, server) = pro(r#"[["2026-03-20T00:00:00Z",247.99]]"#);
        let p = client
            .equities()
            .price_history("AAPL", "US", Some(Timeframe::YearToDate))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/price-history?ticker=AAPL&country=US&timeframe=YTD"
        );
        assert_eq!(p[0].value, Some(247.99));
    }

    #[tokio::test]
    async fn ohlcv() {
        let (client, server) = pro("[[1773964800,247.98,249.2,246,247.99,88268000]]");
        let c = client
            .equities()
            .ohlcv("AAPL", "US", Some(Timeframe::Days7))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/ohlcv?ticker=AAPL&country=US&timeframe=7D"
        );
        assert_eq!(c[0].timestamp, 1_773_964_800);
        assert_eq!(c[0].low, 246.0);
        assert_eq!(c[0].volume, 88_268_000.0);
    }

    #[tokio::test]
    async fn summary() {
        let (client, server) = pro(
            r#"{"currentPrice":1.5,"trailingPE":30,"revenueTTM":100,"holderEarningsTTM":5,
            "fiftyTwoWeekHigh":2,"employeeCount":10,"dividendYield":null}"#,
        );
        let s = client.equities().summary("AAPL", "US").await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/summary?ticker=AAPL&country=US"
        );
        assert_eq!(s.current_price, Some(1.5));
        assert_eq!(s.trailing_pe, Some(30.0));
        assert_eq!(s.revenue_ttm, Some(100.0));
        assert_eq!(s.holder_earnings_ttm, Some(5.0));
        assert_eq!(s.fifty_two_week_high, Some(2.0));
        assert_eq!(s.employee_count, Some(10));
        assert_eq!(s.dividend_yield, None);
    }

    #[tokio::test]
    async fn filings() {
        let (client, server) =
            pro(r#"[{"filingDate":"2026-01-30","form":"10-K","primaryDocumentUrl":"https://x"}]"#);
        let f = client.equities().filings("AAPL", "US").await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/filings?ticker=AAPL&country=US"
        );
        assert_eq!(f[0].form.as_deref(), Some("10-K"));
        assert_eq!(f[0].primary_document_url.as_deref(), Some("https://x"));
    }

    #[tokio::test]
    async fn onchain() {
        let (client, server) = pro(
            r#"{"perps":[{"pair":"AAPL-USDC","volume24h":1.5,"annualizedFundingRate":-70.1,
            "exchangeType":"DEX"}],"tokens":[{"issuer":"X","assetSlug":"AAPLx","activeMarketcap":3}]}"#,
        );
        let m = client.equities().onchain("AAPL", "US").await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/onchain?ticker=AAPL&country=US"
        );
        assert_eq!(m.perps[0].volume_24h, Some(1.5));
        assert_eq!(m.perps[0].exchange_type.as_deref(), Some("DEX"));
        assert_eq!(m.tokens[0].active_marketcap, Some(3.0));
    }

    const SECTOR: &str = r#"{"sector":"Technology","sectorSlug":"technology","rank":1,
        "marketCap":5,"revenueTTM":6,"trailingPE":7,"operatingProfitMarginTTM":8,
        "priceChangePercentageYtd":9,"dollarVolumeShare24h":10,"companyCount":3,
        "coverage":{"trailingPE":0.9},"industryCount":2,
        "topHoldings":[{"ticker":"NVDA","sectorWeight":20}]"#;

    #[tokio::test]
    async fn sectors() {
        let (client, server) = pro(&format!("[{SECTOR}}}]"));
        let s = client.equities().sectors().await.unwrap();
        assert_eq!(server.path(), "/KEY/equities/v1/sectors");
        let s = &s[0];
        assert_eq!(s.sector_slug.as_deref(), Some("technology"));
        assert_eq!(s.market_cap, Some(5.0));
        assert_eq!(s.revenue_ttm, Some(6.0));
        assert_eq!(s.trailing_pe, Some(7.0));
        assert_eq!(s.operating_profit_margin_ttm, Some(8.0));
        assert_eq!(s.price_change_percentage_ytd, Some(9.0));
        assert_eq!(s.dollar_volume_share_24h, Some(10.0));
        assert_eq!(s.coverage["trailingPE"], 0.9);
        assert_eq!(s.top_holdings[0].sector_weight, Some(20.0));
    }

    #[tokio::test]
    async fn sector() {
        let body = format!(
            r#"{SECTOR},"description":"Tech","valuation":{{"medianTrailingPE":25}},
            "concentration":{{"top10MarketCapShare":60,"megaCapCount":4}}}}"#
        );
        let (client, server) = pro(&body);
        let s = client
            .equities()
            .sector("Financial Services")
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/sector?sector=Financial+Services"
        );
        assert_eq!(s.industry_count, Some(2));
        assert_eq!(s.valuation.median_trailing_pe, Some(25.0));
        assert_eq!(s.concentration.top10_market_cap_share, Some(60.0));
        assert_eq!(s.concentration.mega_cap_count, Some(4));
    }

    #[tokio::test]
    async fn sectors_overview() {
        let (client, server) = pro(
            r#"{"marketCap":1,"medianTrailingPE":2,"medianOperatingProfitMarginTTM":3,
            "sectorCount":11,"bestSector1d":{"sector":"Energy","priceChangePercentage1d":2.5},
            "worstSector1d":null}"#,
        );
        let o = client.equities().sectors_overview().await.unwrap();
        assert_eq!(server.path(), "/KEY/equities/v1/sectors-overview");
        assert_eq!(o.median_trailing_pe, Some(2.0));
        assert_eq!(o.median_operating_profit_margin_ttm, Some(3.0));
        assert_eq!(o.sector_count, Some(11));
        let best = o.best_sector_1d.unwrap();
        assert_eq!(best.price_change_percentage_1d, Some(2.5));
        assert!(o.worst_sector_1d.is_none());
    }

    #[tokio::test]
    async fn industries() {
        let (client, server) = pro(
            r#"[{"industry":"Semiconductors","industrySlug":"semiconductors",
            "sectorWeight":30,"earningsTTM":4}]"#,
        );
        let i = client
            .equities()
            .industries(Some("technology"), None)
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/industries?sector=technology"
        );
        assert_eq!(i[0].sector_weight, Some(30.0));
        assert_eq!(i[0].earnings_ttm, Some(4.0));
    }

    #[tokio::test]
    async fn earnings_calendar() {
        let (client, server) = pro(
            r#"[{"ticker":"AAPL","iso2_country_code":"US","date_status":"estimated",
            "report_date":"2026-10-29","fiscal_year":2026,"max_drift_days":7,"revision_count":0}]"#,
        );
        let query = EarningsQuery::new()
            .ticker("AAPL")
            .from("2026-01-01")
            .to("2026-12-31")
            .status(super::super::EarningsStatus::Estimated)
            .limit(10);
        let e = client.equities().earnings_calendar(&query).await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/equities/v1/earnings-calendar?ticker=AAPL&from=2026-01-01&to=2026-12-31&status=estimated&limit=10"
        );
        assert_eq!(e[0].fiscal_year, Some(2026));
        assert_eq!(e[0].max_drift_days, Some(7));

        let (client, server) = pro("[]");
        client
            .equities()
            .earnings_calendar(&EarningsQuery::new())
            .await
            .unwrap();
        assert_eq!(server.path(), "/KEY/equities/v1/earnings-calendar");
    }
}
