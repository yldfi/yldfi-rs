//! Yields API endpoints
//!
//! `DefiLlama` shuts down the free yields endpoints (`yields.llama.fi/pools`
//! and `/chart/{pool}`) on 2026-11-13; from then on yields data needs the
//! API plan. Clients with an API key already use the Pro `/yields/v1/*`
//! routes, which return the same format. Without a key, [`YieldsApi::pools`]
//! and [`YieldsApi::chart`] keep using the free host and explain the
//! shutdown when it fails.
//!
//! The v1 routes serve the legacy response format, which `DefiLlama` has
//! frozen: it will not change or gain fields. The spec does not mark them
//! deprecated, but points new integrations to the v2 routes (`earn*`,
//! `borrow_*` and `loops` here), which have a different shape (see
//! [`super::EarnPool`]).

use crate::client::Client;
use crate::error::{Error, Result};

use super::types::{
    BorrowPool, LegacyPool, LendBorrowChartPoint, LsdRate, LstRate, PerpRate, YieldChartPoint,
    YieldPool, YieldsResponse,
};
use super::v2_types::{
    BorrowMarket, BorrowMarketHistoryPoint, BorrowRoute, EarnHistoryPoint, EarnPool, EarnPoolsPage,
    EarnQuery, HistoryRange, LoopStrategy, VerifiedEarnPool, VerifiedHistoryPoint,
    YieldsV2Response,
};

/// `path`, plus `?range=` when a range is given
fn with_range(path: String, range: Option<HistoryRange>) -> String {
    match range {
        Some(range) => format!("{path}?range={}", range.as_str()),
        None => path,
    }
}

/// Date (UTC) on which `DefiLlama` stops serving the free yields endpoints.
pub const FREE_YIELDS_SHUTDOWN: &str = "2026-11-13";

/// Add the free-endpoint shutdown to an error from `yields.llama.fi`.
///
/// ethcli-mcp keeps this wording unredacted by matching it exactly, in
/// `only_names_api_key`. Update that if this message changes.
///
/// Rate limits and transport failures are passed through unchanged; any
/// other failure (404, 402, 410, 5xx, an HTML page instead of JSON) is what
/// the shutdown is expected to look like.
fn free_yields_error(err: Error) -> Error {
    let status = match &err {
        Error::RateLimited { .. } | Error::Http(_) => return err,
        Error::Api { status, .. } | Error::ServerError { status, .. } => *status,
        Error::Domain(crate::error::DomainError::NotFound(_)) => 404,
        _ => 0,
    };
    Error::api(
        status,
        format!(
            "{err} (DefiLlama shuts down the free yields endpoints on {FREE_YIELDS_SHUTDOWN}; \
             after that, yields data needs an API plan key, set with DEFILLAMA_API_KEY)"
        ),
    )
}

/// Yields API client
pub struct YieldsApi<'a> {
    client: &'a Client,
}

impl<'a> YieldsApi<'a> {
    /// Create a new Yields API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get all yield pools with current APY
    ///
    /// With an API key this calls the Pro `/yields/v1/pools` route. Without
    /// one it calls the free `yields.llama.fi/pools`, which `DefiLlama` shuts
    /// down on 2026-11-13 (see [`FREE_YIELDS_SHUTDOWN`]).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let pools = client.yields().pools().await?;
    /// for pool in pools.iter().take(5) {
    ///     println!("{} on {}: {:.2}% APY",
    ///         pool.symbol, pool.chain, pool.apy.unwrap_or(0.0));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pools(&self) -> Result<Vec<YieldPool>> {
        if self.client.has_pro_access() {
            let resp: YieldsResponse<Vec<YieldPool>> =
                self.client.get_pro("/yields/v1/pools").await?;
            return Ok(resp.data);
        }
        let resp: YieldsResponse<Vec<YieldPool>> = self
            .client
            .get_yields("/pools")
            .await
            .map_err(free_yields_error)?;
        // The free endpoint has thousands of pools; an empty list means it
        // stopped serving data rather than that there are no pools.
        if resp.data.is_empty() {
            return Err(free_yields_error(Error::api(
                200,
                "yields.llama.fi/pools returned no pools",
            )));
        }
        Ok(resp.data)
    }

    /// Get historical APY/TVL chart for a pool
    ///
    /// With an API key this calls the Pro `/yields/v1/chart/{pool}` route.
    /// Without one it calls the free `yields.llama.fi/chart/{pool}`, which
    /// `DefiLlama` shuts down on 2026-11-13 (see [`FREE_YIELDS_SHUTDOWN`]).
    ///
    /// # Arguments
    ///
    /// * `pool` - Pool ID (from pools endpoint)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let chart = client.yields().chart("747c1d2a-c668-4682-b9f9-296708a3dd90").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chart(&self, pool: &str) -> Result<Vec<YieldChartPoint>> {
        if self.client.has_pro_access() {
            let path = format!("/yields/v1/chart/{pool}");
            let resp: YieldsResponse<Vec<YieldChartPoint>> = self.client.get_pro(&path).await?;
            return Ok(resp.data);
        }
        let path = format!("/chart/{pool}");
        let resp: YieldsResponse<Vec<YieldChartPoint>> = self
            .client
            .get_yields(&path)
            .await
            .map_err(free_yields_error)?;
        Ok(resp.data)
    }

    /// Get pools with old pool IDs and token addresses (`GET /yields/v1/pools`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let pools = client.yields().pools_old().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pools_old(&self) -> Result<Vec<LegacyPool>> {
        let resp: YieldsResponse<Vec<LegacyPool>> = self.client.get_pro("/yields/v1/pools").await?;
        Ok(resp.data)
    }

    /// Get borrowing rates across protocols
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let borrow_pools = client.yields().pools_borrow().await?;
    /// for pool in borrow_pools.iter().take(5) {
    ///     println!("{}: lend {:.2}%, borrow {:.2}%",
    ///         pool.symbol,
    ///         pool.apy.unwrap_or(0.0),
    ///         pool.apy_base_borrow.unwrap_or(0.0));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pools_borrow(&self) -> Result<Vec<BorrowPool>> {
        let resp: YieldsResponse<Vec<BorrowPool>> =
            self.client.get_pro("/yields/v1/poolsBorrow").await?;
        Ok(resp.data)
    }

    /// Get historical lend/borrow rates for a pool
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `pool` - Pool ID (from `pools_borrow` endpoint)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let chart = client.yields().chart_lend_borrow("pool-id").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chart_lend_borrow(&self, pool: &str) -> Result<Vec<LendBorrowChartPoint>> {
        let path = format!("/yields/v1/chartLendBorrow/{pool}");
        let resp: YieldsResponse<Vec<LendBorrowChartPoint>> = self.client.get_pro(&path).await?;
        Ok(resp.data)
    }

    /// Get perpetual futures funding rates (`GET /api/perpFundingRates`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let perps = client.yields().perps().await?;
    /// for perp in perps.iter().take(5) {
    ///     println!("{} on {}: {:.4}% funding rate",
    ///         perp.market, perp.marketplace, perp.funding_rate.unwrap_or(0.0));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn perps(&self) -> Result<Vec<PerpRate>> {
        let resp: YieldsResponse<Vec<PerpRate>> =
            self.client.get_pro("/api/perpFundingRates").await?;
        Ok(resp.data)
    }

    /// Get liquid staking derivative rates
    ///
    /// DefiLlama removed `/yields/lsdRates`; this returns an error without
    /// making a request. Use [`Self::lst_rates`] (`/api/lstRates`), which
    /// has a different response shape.
    #[deprecated(
        since = "0.1.5",
        note = "DefiLlama removed /yields/lsdRates; use `lst_rates()` (/api/lstRates)"
    )]
    pub async fn lsd_rates(&self) -> Result<Vec<LsdRate>> {
        Err(crate::error::Error::api(
            410,
            "DefiLlama removed /yields/lsdRates; use yields().lst_rates() (/api/lstRates)",
        ))
    }

    /// Get liquid staking token exchange rates and ETH peg
    /// (`GET /api/lstRates`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let rates = client.yields().lst_rates().await?;
    /// for rate in rates.iter().take(5) {
    ///     println!("{}: peg {:.3}%", rate.name, rate.eth_peg.unwrap_or(0.0));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lst_rates(&self) -> Result<Vec<LstRate>> {
        // Returns an array directly, not wrapped in {status, data}
        self.client.get_pro("/api/lstRates").await
    }

    // ==================== v2 (Pro) ====================

    /// Get the latest data for every earn pool (`GET /yields/v2/earn`)
    ///
    /// **Requires Pro API key**
    ///
    /// Pools are sorted by TVL, descending. Use [`Self::earn_query`] for a
    /// filtered, paginated subset.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// for pool in client.yields().earn().await?.iter().take(5) {
    ///     println!("{} on {}: {:?}% APY", pool.protocol, pool.chain, pool.apy.current.total);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn earn(&self) -> Result<Vec<EarnPool>> {
        let resp: YieldsV2Response<Vec<EarnPool>> = self.client.get_pro("/yields/v2/earn").await?;
        Ok(resp.data)
    }

    /// Get a filtered, paginated page of earn pools
    /// (`GET /yields/v2/earn/query`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::yields::EarnQuery;
    ///
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let query = EarnQuery::new().chain("ethereum").stablecoin(true).limit(50);
    /// let page = client.yields().earn_query(&query).await?;
    /// println!("{} of {:?} pools", page.data.len(), page.pagination.total);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn earn_query(&self, query: &EarnQuery) -> Result<EarnPoolsPage> {
        let path = format!("/yields/v2/earn/query{}", query.to_query_string());
        self.client.get_pro(&path).await
    }

    /// Get the daily history of an earn pool, oldest first
    /// (`GET /yields/v2/earn/{id}/history`)
    ///
    /// **Requires Pro API key**
    ///
    /// `range` defaults to the full series.
    pub async fn earn_history(
        &self,
        id: &str,
        range: Option<HistoryRange>,
    ) -> Result<Vec<EarnHistoryPoint>> {
        let path = with_range(format!("/yields/v2/earn/{id}/history"), range);
        let resp: YieldsV2Response<Vec<EarnHistoryPoint>> = self.client.get_pro(&path).await?;
        Ok(resp.data)
    }

    /// Get the measured (verified) yield of every simulated earn pool
    /// (`GET /yields/v2/earn/verified`)
    ///
    /// **Requires Pro API key**
    ///
    /// Each figure is replayed from a real holder's withdrawal, so it can
    /// differ from what the protocol reports. Join to [`Self::earn`] on
    /// `id`.
    pub async fn earn_verified(&self) -> Result<Vec<VerifiedEarnPool>> {
        let resp: YieldsV2Response<Vec<VerifiedEarnPool>> =
            self.client.get_pro("/yields/v2/earn/verified").await?;
        Ok(resp.data)
    }

    /// Get the measured yield of one earn pool, with the on-chain calls the
    /// simulation makes (`GET /yields/v2/earn/verified/{id}`)
    ///
    /// **Requires Pro API key**
    pub async fn earn_verified_pool(&self, id: &str) -> Result<VerifiedEarnPool> {
        let path = format!("/yields/v2/earn/verified/{id}");
        let resp: YieldsV2Response<VerifiedEarnPool> = self.client.get_pro(&path).await?;
        Ok(resp.data)
    }

    /// Get the daily measured history of an earn pool, oldest first
    /// (`GET /yields/v2/earn/verified/{id}/history`)
    ///
    /// **Requires Pro API key**
    ///
    /// Days the pool was not measured are left out. `range` defaults to the
    /// full series.
    pub async fn earn_verified_history(
        &self,
        id: &str,
        range: Option<HistoryRange>,
    ) -> Result<Vec<VerifiedHistoryPoint>> {
        let path = with_range(format!("/yields/v2/earn/verified/{id}/history"), range);
        let resp: YieldsV2Response<Vec<VerifiedHistoryPoint>> = self.client.get_pro(&path).await?;
        Ok(resp.data)
    }

    /// Get the latest data for every borrow market
    /// (`GET /yields/v2/borrow/markets`)
    ///
    /// **Requires Pro API key**
    pub async fn borrow_markets(&self) -> Result<Vec<BorrowMarket>> {
        let resp: YieldsV2Response<Vec<BorrowMarket>> =
            self.client.get_pro("/yields/v2/borrow/markets").await?;
        Ok(resp.data)
    }

    /// Get the daily borrow-side history of a borrow market, oldest first
    /// (`GET /yields/v2/borrow/markets/{id}/history`)
    ///
    /// **Requires Pro API key**
    ///
    /// The supply side of the same id is [`Self::earn_history`]. `range`
    /// defaults to the full series.
    pub async fn borrow_market_history(
        &self,
        id: &str,
        range: Option<HistoryRange>,
    ) -> Result<Vec<BorrowMarketHistoryPoint>> {
        let path = with_range(format!("/yields/v2/borrow/markets/{id}/history"), range);
        let resp: YieldsV2Response<Vec<BorrowMarketHistoryPoint>> =
            self.client.get_pro(&path).await?;
        Ok(resp.data)
    }

    /// Get every borrowing route: a collateral position paired with a debt
    /// market (`GET /yields/v2/borrow/routes`)
    ///
    /// **Requires Pro API key**
    pub async fn borrow_routes(&self) -> Result<Vec<BorrowRoute>> {
        let resp: YieldsV2Response<Vec<BorrowRoute>> =
            self.client.get_pro("/yields/v2/borrow/routes").await?;
        Ok(resp.data)
    }

    /// Get every leveraged looping strategy (`GET /yields/v2/loops`)
    ///
    /// **Requires Pro API key**
    pub async fn loops(&self) -> Result<Vec<LoopStrategy>> {
        let resp: YieldsV2Response<Vec<LoopStrategy>> =
            self.client.get_pro("/yields/v2/loops").await?;
        Ok(resp.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_yields_error_mentions_shutdown() {
        let err = free_yields_error(crate::error::from_response(410, "Gone", None));
        let msg = err.to_string();
        assert!(msg.contains(FREE_YIELDS_SHUTDOWN), "{msg}");
        assert!(msg.contains("DEFILLAMA_API_KEY"), "{msg}");
        assert!(matches!(err, Error::Api { status: 410, .. }));

        let err = free_yields_error(crate::error::from_response(404, "not found", None));
        assert!(matches!(err, Error::Api { status: 404, .. }));
    }

    #[test]
    fn free_yields_error_keeps_rate_limits() {
        let err = free_yields_error(Error::RateLimited {
            retry_after: Some(5),
        });
        assert!(matches!(
            err,
            Error::RateLimited {
                retry_after: Some(5)
            }
        ));
    }

    use crate::test_server::TestServer;

    /// A Pro client whose requests go to a server answering `body`
    fn pro(body: &str) -> (Client, TestServer) {
        let server = TestServer::json(body);
        let client = Client::with_api_key("KEY")
            .unwrap()
            .with_test_base(&server.url);
        (client, server)
    }

    const TOKEN: &str = r#"{"address":"0x01","symbol":"FOO","decimals":18}"#;

    #[tokio::test]
    async fn v2_needs_api_key() {
        let client = Client::new().unwrap();
        let err = client.yields().earn().await.unwrap_err();
        assert!(matches!(err, Error::Api { status: 401, .. }), "{err}");
    }

    #[tokio::test]
    async fn earn() {
        let body = format!(
            r#"{{"data":[{{"id":"p1","pool_key":"0xab-ethereum",
            "pool_token":{{"address":"0xab","symbol":"aFOO","name":"Foo"}},
            "display":{{"label":"FOO","detail":null}},"reference_asset_symbol":"FOO",
            "url":null,"protocol":"Foo","chain":"Ethereum","underlying_tokens":[{TOKEN}],
            "reward_tokens":[],"tvl_usd":1000.5,
            "apy":{{"current":{{"base":2.0,"reward":0.5,"total":2.5}},
                    "avg_7d":{{"base":1.9,"reward":null,"total":1.9}},
                    "avg_30d":{{"base":1.8,"reward":null,"total":1.8}}}},
            "intrinsic":{{"apy":{{"current":{{"base":3.0}},"avg_7d":{{"base":3.1}},
                "avg_30d":{{"base":3.2}}}},
                "source_pool":{{"id":"p0","pool_token":"0xcd","symbol":"stFOO"}}}},
            "stability_30d":{{"base":0.9,"base_reward":0.8,"base_intrinsic":null,
                "base_reward_intrinsic":null}},
            "price_per_share":1.01,"volume_1d_usd":null,
            "attributes":{{"stablecoin_only":false,"single_asset_exposure":true,
                "impermanent_loss_risk":false}},
            "holders":{{"count":10,"avg_position_usd":100.0,"top_10_share_pct":90.0}},
            "available_supply_usd":null}}]}}"#
        );
        let (client, server) = pro(&body);
        let pools = client.yields().earn().await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/earn");
        let p = &pools[0];
        assert_eq!(p.id, "p1");
        assert_eq!(
            p.pool_token.as_ref().unwrap().symbol.as_deref(),
            Some("aFOO")
        );
        assert_eq!(p.underlying_tokens[0].decimals, Some(18));
        assert_eq!(p.apy.current.total, Some(2.5));
        assert_eq!(p.apy.avg_30d.base, Some(1.8));
        let intrinsic = p.intrinsic.as_ref().unwrap();
        assert_eq!(intrinsic.apy.current.base, Some(3.0));
        assert_eq!(intrinsic.source_pool.as_ref().unwrap().id, "p0");
        assert_eq!(p.stability_30d.base_reward, Some(0.8));
        assert_eq!(p.attributes.single_asset_exposure, Some(true));
        assert_eq!(p.holders.count, Some(10.0));
    }

    #[tokio::test]
    async fn earn_query() {
        let (client, server) = pro(
            r#"{"data":[{"id":"p1","protocol":"Foo","chain":"Ethereum"}],
            "pagination":{"total":250,"page":2,"limit":100}}"#,
        );
        let query = EarnQuery::new()
            .chain("ethereum")
            .protocol("aave-v3")
            .min_tvl(1e6)
            .page(2);
        let page = client.yields().earn_query(&query).await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/yields/v2/earn/query?chain=ethereum&protocol=aave-v3&min_tvl=1000000&page=2"
        );
        assert_eq!(page.data[0].chain, "Ethereum");
        assert_eq!(page.pagination.total, Some(250));
        assert_eq!(page.pagination.page, Some(2));

        let (client, server) = pro(r#"{"data":[],"pagination":{}}"#);
        client.yields().earn_query(&EarnQuery::new()).await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/earn/query");
    }

    #[tokio::test]
    async fn earn_history() {
        let (client, server) = pro(r#"{"data":[{"date":"2026-11-01","tvl_usd":5.0,
            "apy":{"avg_1d":{"base":1.0,"reward":null,"total":1.0},"avg_7d":{"base":1.1},
                   "avg_30d":{"base":1.2}},
            "intrinsic":{"apy":{"avg_1d":{"base":3.0},"avg_7d":{"base":3.1},"avg_30d":{"base":3.2}}},
            "price_per_share":null}]}"#);
        let rows = client
            .yields()
            .earn_history("p1", Some(HistoryRange::Days30))
            .await
            .unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/earn/p1/history?range=30d");
        assert_eq!(rows[0].date, "2026-11-01");
        assert_eq!(rows[0].apy.avg_1d.total, Some(1.0));
        assert_eq!(
            rows[0].intrinsic.as_ref().unwrap().apy.avg_1d.base,
            Some(3.0)
        );

        let (client, server) = pro(r#"{"data":[]}"#);
        client.yields().earn_history("p1", None).await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/earn/p1/history");
    }

    const VERIFIED: &str = r#"{"id":"p1","apy":{"avg_1d":-0.1,"avg_7d":4.0,"avg_30d":4.2},
        "apy_as_of":"2026-11-01T00:00:00Z","apy_note":null,"apy_confidence":"high",
        "exit":{"status":"instant","eta_seconds":null,"fee_bps":0,"as_of":"2026-11-01T00:00:00Z"},
        "exit_liquidity":{"amount":"1000000","usd":1.0,"slippage_bps":2,"as_of":"2026-11-01"},
        "tvl":{"amount":"5000000","usd":5.0,"as_of":"2026-11-01"},"price_per_share":1.05,
        "denomination":{"address":"0x01","decimals":6}"#;

    #[tokio::test]
    async fn earn_verified() {
        let (client, server) = pro(&format!(r#"{{"data":[{VERIFIED}}}]}}"#));
        let pools = client.yields().earn_verified().await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/earn/verified");
        let p = &pools[0];
        assert_eq!(p.apy.avg_1d, Some(-0.1));
        assert_eq!(p.apy_confidence.as_deref(), Some("high"));
        assert_eq!(p.exit.status.as_deref(), Some("instant"));
        let liquidity = p.exit_liquidity.as_ref().unwrap();
        assert_eq!(liquidity.amount.as_deref(), Some("1000000"));
        assert_eq!(liquidity.slippage_bps, Some(2.0));
        assert_eq!(p.tvl.usd, Some(5.0));
        assert!(p.methods.is_empty());
    }

    #[tokio::test]
    async fn earn_verified_pool() {
        let (client, server) = pro(&format!(
            r#"{{"data":{VERIFIED},"methods":["redeem(uint256)"]}}}}"#
        ));
        let p = client.yields().earn_verified_pool("p1").await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/earn/verified/p1");
        assert_eq!(p.id, "p1");
        assert_eq!(p.denomination.unwrap().decimals, Some(6));
        assert_eq!(p.methods, vec!["redeem(uint256)"]);
    }

    #[tokio::test]
    async fn earn_verified_history() {
        let (client, server) = pro(
            r#"{"data":[{"date":"2026-11-01","outcome":"delayed","eta_seconds":3600,
            "confidence":"medium","apy":{"avg_1d":1.0,"avg_7d":null,"avg_30d":null},
            "price_per_share":null,"price_per_share_quoted":1.02,"reason":"rate_quality_floor",
            "exit_liquidity":null,"tvl":null,"tvl_quoted":{"amount":"7","usd":7.0}}]}"#,
        );
        let rows = client
            .yields()
            .earn_verified_history("p1", Some(HistoryRange::Max))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/yields/v2/earn/verified/p1/history?range=max"
        );
        let r = &rows[0];
        assert_eq!(r.outcome.as_deref(), Some("delayed"));
        assert_eq!(r.eta_seconds, Some(3600.0));
        assert_eq!(r.apy.avg_1d, Some(1.0));
        assert_eq!(r.price_per_share_quoted, Some(1.02));
        assert!(r.tvl.is_none());
        assert_eq!(r.tvl_quoted.as_ref().unwrap().usd, Some(7.0));
    }

    #[tokio::test]
    async fn borrow_markets() {
        let body = format!(
            r#"{{"data":[{{"id":"m1","pool_key":"k","pool_token":null,
            "display":{{"label":"FOO / BAR","detail":null}},"url":null,"protocol":"Foo",
            "chain":"Ethereum","collateral_token":{TOKEN},"borrow_token":null,"reward_tokens":[],
            "loan_to_value":0.805,"total_supply_usd":10.0,"total_borrow_usd":4.0,
            "available_borrow_usd":6.0,
            "supply_apy":{{"current":{{"base":1.0,"reward":null,"total":1.0}}}},
            "borrow_cost_apy":{{"current":{{"base":3.0,"reward":4.0,"total":-1.0}}}},
            "intrinsic":{{"supply":null,"borrow":{{"apy":{{"current":{{"base":3.0}}}},
                "source_pool":{{"id":"p0","pool_token":"0xcd","symbol":null}}}}}}}}]}}"#
        );
        let (client, server) = pro(&body);
        let markets = client.yields().borrow_markets().await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/borrow/markets");
        let m = &markets[0];
        assert_eq!(m.display.label.as_deref(), Some("FOO / BAR"));
        assert_eq!(m.collateral_token.as_ref().unwrap().address, "0x01");
        assert!(m.borrow_token.is_none());
        assert_eq!(m.loan_to_value, Some(0.805));
        assert_eq!(m.borrow_cost_apy.current.total, Some(-1.0));
        assert!(m.intrinsic.supply.is_none());
        assert_eq!(
            m.intrinsic.borrow.as_ref().unwrap().apy.current.base,
            Some(3.0)
        );
    }

    #[tokio::test]
    async fn borrow_market_history() {
        let (client, server) = pro(
            r#"{"data":[{"date":"2026-11-01","total_borrow_usd":4.0,"available_borrow_usd":6.0,
            "borrow_cost_apy":{"avg_1d":{"base":3.0,"reward":1.0,"total":2.0}},"intrinsic":null}]}"#,
        );
        let rows = client
            .yields()
            .borrow_market_history("m1", Some(HistoryRange::Days90))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/yields/v2/borrow/markets/m1/history?range=90d"
        );
        assert_eq!(rows[0].total_borrow_usd, Some(4.0));
        assert_eq!(rows[0].borrow_cost_apy.avg_1d.total, Some(2.0));
        assert!(rows[0].intrinsic.is_none());
    }

    #[tokio::test]
    async fn borrow_routes() {
        let body = format!(
            r#"{{"data":[{{"display":{{"detail":"Prime"}},"protocol":"Foo","chain":"Ethereum",
            "collateral":{{"id":"c1","pool_key":"ck","url":null,"token":{TOKEN},
                "loan_to_value":0.84,"total_supply_usd":10.0,
                "apy":{{"current":{{"base":3.0,"reward":null,"total":3.0}}}},
                "intrinsic":null,"reward_tokens":[]}},
            "borrow":{{"id":"b1","pool_key":"bk","url":null,"token":{TOKEN},
                "total_borrow_usd":4.0,"available_borrow_usd":6.0,
                "cost_apy":{{"current":{{"base":2.0,"reward":null,"total":2.0}}}},
                "intrinsic":null,"reward_tokens":[{TOKEN}]}},
            "net_apy":{{"current":{{"base":1.0,"base_reward":1.0,"base_intrinsic":null,
                "base_reward_intrinsic":null}}}}}}]}}"#
        );
        let (client, server) = pro(&body);
        let routes = client.yields().borrow_routes().await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/borrow/routes");
        let r = &routes[0];
        assert_eq!(r.display.detail.as_deref(), Some("Prime"));
        assert_eq!(r.collateral.id, "c1");
        assert_eq!(r.collateral.loan_to_value, Some(0.84));
        assert_eq!(r.borrow.cost_apy.current.total, Some(2.0));
        assert_eq!(r.borrow.reward_tokens.len(), 1);
        assert_eq!(r.net_apy.current.base, Some(1.0));
    }

    #[tokio::test]
    async fn loops() {
        let body = format!(
            r#"{{"data":[{{"id":"c1","display":{{"detail":null}},"protocol":"Foo",
            "chain":"Ethereum","token":{TOKEN},"url":null,"loan_to_value":0.7,
            "total_supply_usd":10.0,"total_borrow_usd":4.0,"available_borrow_usd":6.0,
            "supply_apy":{{"current":{{"base":3.0,"reward":null,"total":3.0}}}},
            "borrow_cost_apy":{{"current":{{"base":2.0,"reward":1.5,"total":0.5}}}},
            "loop_apy":{{"loop_count":5,"total":8.2}},
            "leverage":{{"supply_multiplier":2.94,"borrow_multiplier":1.94}},
            "intrinsic":null,"reward_tokens":{{"supply":[],"borrow":[{TOKEN}]}}}}]}}"#
        );
        let (client, server) = pro(&body);
        let loops = client.yields().loops().await.unwrap();
        assert_eq!(server.path(), "/KEY/yields/v2/loops");
        let l = &loops[0];
        assert_eq!(l.token.as_ref().unwrap().symbol.as_deref(), Some("FOO"));
        assert_eq!(l.loop_apy.loop_count, Some(5.0));
        assert_eq!(l.loop_apy.total, Some(8.2));
        assert_eq!(l.leverage.supply_multiplier, Some(2.94));
        assert!(l.reward_tokens.supply.is_empty());
        assert_eq!(l.reward_tokens.borrow.len(), 1);
    }
}
