//! v2 chart and metrics endpoints (Pro)

use std::collections::HashMap;

use crate::client::{path_segment as seg, with_query, Client};
use crate::error::{invalid_param, Result};

use super::types::{
    Breakdown, BreakdownPoint, ChartPoint, Currency, Dimension, DimensionCategory,
    DimensionOverview, DimensionProtocolOverview, FinancialStatement, OracleOverview,
    ProtocolMetrics, Scope, TreasuryKey, TvlKey,
};

/// Path of a dimension chart, checking that the scope has that breakdown
pub(crate) fn dimension_chart_path(
    metric: Dimension,
    scope: &Scope,
    by: Option<Breakdown>,
) -> Result<String> {
    use Breakdown as B;
    let ok = match (scope, by) {
        (_, None) => true,
        (Scope::All | Scope::Category(_), Some(b)) => matches!(b, B::Chain | B::Protocol),
        (Scope::Chain(_) | Scope::CategoryChain { .. }, Some(b)) => b == B::Protocol,
        (Scope::Protocol(_), Some(b)) => matches!(b, B::Chain | B::Label | B::Version),
    };
    let base = format!("/api/v2/chart/{}{}", metric.as_str(), scope.path());
    finish_chart_path(base, scope, by, ok)
}

/// Path of an oracle chart, checking the scope and breakdown
pub(crate) fn oracle_chart_path(scope: &Scope, by: Option<Breakdown>) -> Result<String> {
    use Breakdown as B;
    let ok = match (scope, by) {
        (Scope::All, None | Some(B::Chain | B::Protocol))
        | (Scope::Chain(_), None | Some(B::Protocol))
        | (Scope::Protocol(_), None | Some(B::Chain)) => true,
        (Scope::Category(_) | Scope::CategoryChain { .. }, _) => {
            return Err(invalid_param(format!(
                "oracle charts have no {} scope",
                scope.name()
            )))
        }
        _ => false,
    };
    let base = format!("/api/v2/chart/oracle{}", scope.path());
    finish_chart_path(base, scope, by, ok)
}

fn finish_chart_path(
    base: String,
    scope: &Scope,
    by: Option<Breakdown>,
    ok: bool,
) -> Result<String> {
    match by {
        Some(b) if !ok => Err(invalid_param(format!(
            "no {} for the {} scope",
            b.as_str(),
            scope.name()
        ))),
        Some(b) => Ok(format!("{base}/{}", b.as_str())),
        None => Ok(base),
    }
}

/// Path of a protocol TVL or treasury breakdown (`family` is `tvl` or
/// `treasury`)
fn protocol_breakdown_path(
    family: &str,
    protocol: &str,
    by: Breakdown,
    key: Option<&str>,
    currency: Option<Currency>,
) -> Result<String> {
    if !matches!(by, Breakdown::Chain | Breakdown::Token) {
        return Err(invalid_param(format!(
            "{family} charts have no {}",
            by.as_str()
        )));
    }
    if currency.is_some() && by != Breakdown::Token {
        return Err(invalid_param(
            "currency only applies to the token breakdown",
        ));
    }
    let path = format!(
        "/api/v2/chart/{family}/protocol/{}/{}",
        seg(protocol),
        by.as_str()
    );
    Ok(with_query(
        path,
        [("key", key), ("currency", currency.map(Currency::as_str))],
    ))
}

/// v2 chart and metrics API client (Pro only)
///
/// Dimension metrics (fees, volumes, open interest, ...) take a
/// [`Dimension`] and a [`Scope`]; breakdown charts also take a
/// [`Breakdown`]. TVL, treasury, oracle and fork charts have their own
/// methods. Total charts return [`ChartPoint`]s and breakdowns
/// [`BreakdownPoint`]s, whatever shape the endpoint sends them in.
///
/// `data_type` picks the metric's series, e.g. `dailyRevenue` for
/// [`Dimension::Fees`] or `dailyNotionalVolume` for [`Dimension::Dexs`];
/// `None` gives the metric's default.
pub struct MetricsApi<'a> {
    client: &'a Client,
}

impl<'a> MetricsApi<'a> {
    /// Create a new v2 chart and metrics API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get the total chart of a dimension metric
    /// (`GET /api/v2/chart/{metric}[/chain|protocol|category/...]`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::metrics::{Dimension, Scope};
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let fees = client
    ///     .metrics()
    ///     .chart(Dimension::Fees, &Scope::protocol("aave"), Some("dailyRevenue"))
    ///     .await?;
    /// if let Some(last) = fees.last() {
    ///     println!("{}: ${:.0}", last.timestamp, last.value);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chart(
        &self,
        metric: Dimension,
        scope: &Scope,
        data_type: Option<&str>,
    ) -> Result<Vec<ChartPoint>> {
        let path = dimension_chart_path(metric, scope, None)?;
        self.client
            .get_pro(&with_query(path, [("dataType", data_type)]))
            .await
    }

    /// Get a dimension metric chart split by chain, protocol, label or
    /// version (`GET /api/v2/chart/{metric}[/...]/{by}-breakdown`)
    ///
    /// **Requires Pro API key**
    ///
    /// Breakdowns by scope: [`Scope::All`] and [`Scope::Category`] by chain
    /// or protocol; [`Scope::Chain`] and [`Scope::CategoryChain`] by
    /// protocol; [`Scope::Protocol`] by chain, label or version. Others
    /// return an invalid-parameter error without a request.
    pub async fn chart_breakdown(
        &self,
        metric: Dimension,
        scope: &Scope,
        by: Breakdown,
        data_type: Option<&str>,
    ) -> Result<Vec<BreakdownPoint>> {
        let path = dimension_chart_path(metric, scope, Some(by))?;
        self.client
            .get_pro(&with_query(path, [("dataType", data_type)]))
            .await
    }

    /// Get the totals of a dimension metric overall, on a chain, in a
    /// category or in a category on a chain
    /// (`GET /api/v2/metrics/{metric}[/...]`)
    ///
    /// **Requires Pro API key**
    ///
    /// For one protocol use [`Self::protocol_overview`]; [`Scope::Protocol`]
    /// returns an invalid-parameter error here.
    pub async fn overview(
        &self,
        metric: Dimension,
        scope: &Scope,
        data_type: Option<&str>,
    ) -> Result<DimensionOverview> {
        if let Scope::Protocol(_) = scope {
            return Err(invalid_param(
                "use protocol_overview for a protocol's dimension metrics",
            ));
        }
        let path = format!("/api/v2/metrics/{}{}", metric.as_str(), scope.path());
        self.client
            .get_pro(&with_query(path, [("dataType", data_type)]))
            .await
    }

    /// Get the totals of a dimension metric for one protocol
    /// (`GET /api/v2/metrics/{metric}/protocol/{protocol}`)
    ///
    /// **Requires Pro API key**
    pub async fn protocol_overview(
        &self,
        metric: Dimension,
        protocol: &str,
        data_type: Option<&str>,
    ) -> Result<DimensionProtocolOverview> {
        let path = format!(
            "/api/v2/metrics/{}/protocol/{}",
            metric.as_str(),
            seg(protocol)
        );
        self.client
            .get_pro(&with_query(path, [("dataType", data_type)]))
            .await
    }

    /// List the categories with data for a dimension metric
    /// (`GET /api/v2/metrics/{metric}/categories`)
    ///
    /// **Requires Pro API key**
    pub async fn categories(
        &self,
        metric: Dimension,
        data_type: Option<&str>,
    ) -> Result<Vec<DimensionCategory>> {
        let path = format!("/api/v2/metrics/{}/categories", metric.as_str());
        self.client
            .get_pro(&with_query(path, [("dataType", data_type)]))
            .await
    }

    /// Get a protocol's TVL chart
    /// (`GET /api/v2/chart/tvl/protocol/{protocol}`)
    ///
    /// **Requires Pro API key**
    ///
    /// `key` picks staking, borrowed, etc.; `None` is plain TVL.
    pub async fn tvl_chart(&self, protocol: &str, key: Option<TvlKey>) -> Result<Vec<ChartPoint>> {
        let path = format!("/api/v2/chart/tvl/protocol/{}", seg(protocol));
        let path = with_query(path, [("key", key.map(TvlKey::as_str))]);
        self.client.get_pro(&path).await
    }

    /// Get a protocol's TVL chart by chain or token
    /// (`GET /api/v2/chart/tvl/protocol/{protocol}/{chain|token}-breakdown`)
    ///
    /// **Requires Pro API key**
    ///
    /// `by` is [`Breakdown::Chain`] or [`Breakdown::Token`]; `currency`
    /// (token breakdown only) defaults to USD.
    pub async fn tvl_breakdown(
        &self,
        protocol: &str,
        by: Breakdown,
        key: Option<TvlKey>,
        currency: Option<Currency>,
    ) -> Result<Vec<BreakdownPoint>> {
        let path = protocol_breakdown_path("tvl", protocol, by, key.map(TvlKey::as_str), currency)?;
        self.client.get_pro(&path).await
    }

    /// Get a protocol's metadata and current TVL by chain
    /// (`GET /api/v2/metrics/tvl/protocol/{protocol}`)
    ///
    /// **Requires Pro API key**
    pub async fn tvl_metrics(&self, protocol: &str) -> Result<ProtocolMetrics> {
        let path = format!("/api/v2/metrics/tvl/protocol/{}", seg(protocol));
        self.client.get_pro(&path).await
    }

    /// Get a protocol's treasury chart
    /// (`GET /api/v2/chart/treasury/protocol/{protocol}`)
    ///
    /// **Requires Pro API key**
    ///
    /// `None` leaves out the protocol's own tokens.
    pub async fn treasury_chart(
        &self,
        protocol: &str,
        key: Option<TreasuryKey>,
    ) -> Result<Vec<ChartPoint>> {
        let path = format!("/api/v2/chart/treasury/protocol/{}", seg(protocol));
        let path = with_query(path, [("key", key.map(TreasuryKey::as_str))]);
        self.client.get_pro(&path).await
    }

    /// Get a protocol's treasury chart by chain or token
    /// (`GET /api/v2/chart/treasury/protocol/{protocol}/{chain|token}-breakdown`)
    ///
    /// **Requires Pro API key**
    ///
    /// `by` is [`Breakdown::Chain`] or [`Breakdown::Token`]; `currency`
    /// (token breakdown only) defaults to USD.
    pub async fn treasury_breakdown(
        &self,
        protocol: &str,
        by: Breakdown,
        key: Option<TreasuryKey>,
        currency: Option<Currency>,
    ) -> Result<Vec<BreakdownPoint>> {
        let path = protocol_breakdown_path(
            "treasury",
            protocol,
            by,
            key.map(TreasuryKey::as_str),
            currency,
        )?;
        self.client.get_pro(&path).await
    }

    /// Get a protocol's metadata and current treasury by chain
    /// (`GET /api/v2/metrics/treasury/protocol/{protocol}`)
    ///
    /// **Requires Pro API key**
    pub async fn treasury_metrics(&self, protocol: &str) -> Result<ProtocolMetrics> {
        let path = format!("/api/v2/metrics/treasury/protocol/{}", seg(protocol));
        self.client.get_pro(&path).await
    }

    /// Get the value secured by oracles, overall, on a chain
    /// ([`Scope::Chain`]) or by one oracle ([`Scope::Protocol`])
    /// (`GET /api/v2/chart/oracle[/chain/{chain}|/protocol/{oracle}]`)
    ///
    /// **Requires Pro API key**
    pub async fn oracle_chart(&self, scope: &Scope) -> Result<Vec<ChartPoint>> {
        let path = oracle_chart_path(scope, None)?;
        self.client.get_pro(&path).await
    }

    /// Get the value secured by oracles split by chain or oracle
    /// (`GET /api/v2/chart/oracle[/...]/{by}-breakdown`)
    ///
    /// **Requires Pro API key**
    ///
    /// [`Scope::All`] by chain or oracle ([`Breakdown::Protocol`]);
    /// [`Scope::Chain`] by oracle; [`Scope::Protocol`] (one oracle) by
    /// chain.
    pub async fn oracle_breakdown(
        &self,
        scope: &Scope,
        by: Breakdown,
    ) -> Result<Vec<BreakdownPoint>> {
        let path = oracle_chart_path(scope, Some(by))?;
        self.client.get_pro(&path).await
    }

    /// List oracles and the protocols using them
    /// (`GET /api/v2/metrics/oracle`)
    ///
    /// **Requires Pro API key**
    pub async fn oracles(&self) -> Result<OracleOverview> {
        self.client.get_pro("/api/v2/metrics/oracle").await
    }

    /// Get the TVL of a protocol's forks
    /// (`GET /api/v2/chart/fork/protocol/{protocol}`)
    ///
    /// **Requires Pro API key**
    pub async fn fork_chart(&self, protocol: &str) -> Result<Vec<ChartPoint>> {
        let path = format!("/api/v2/chart/fork/protocol/{}", seg(protocol));
        self.client.get_pro(&path).await
    }

    /// Get the TVL of forks, one series per forked protocol
    /// (`GET /api/v2/chart/fork/protocol-breakdown`)
    ///
    /// **Requires Pro API key**
    pub async fn fork_breakdown(&self) -> Result<Vec<BreakdownPoint>> {
        self.client
            .get_pro("/api/v2/chart/fork/protocol-breakdown")
            .await
    }

    /// List forked protocols and their forks, keyed by original
    /// (`GET /api/v2/metrics/fork`)
    ///
    /// **Requires Pro API key**
    pub async fn forks(&self) -> Result<HashMap<String, Vec<String>>> {
        self.client.get_pro("/api/v2/metrics/fork").await
    }

    /// Get a protocol's income statement
    /// (`GET /api/v2/metrics/financial-statement/protocol/{protocol}`)
    ///
    /// **Requires Pro API key**
    pub async fn financial_statement(&self, protocol: &str) -> Result<FinancialStatement> {
        let path = format!(
            "/api/v2/metrics/financial-statement/protocol/{}",
            seg(protocol)
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

    const PAIRS: &str = "[[1700000000, 12.5]]";
    const PAIR_BREAKDOWN: &str = r#"[[1700000000, {"Ethereum": 3, "Solana": 1}]]"#;
    const ROW_BREAKDOWN: &str = r#"[{"timestamp": 1700000000, "Chainlink": 3}]"#;

    #[tokio::test]
    async fn needs_api_key() {
        let client = Client::new().unwrap();
        let err = client
            .metrics()
            .chart(Dimension::Fees, &Scope::All, None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Api { status: 401, .. }), "{err}");
    }

    #[test]
    fn dimension_chart_paths() {
        let p = |scope: &Scope, by| dimension_chart_path(Dimension::Fees, scope, by).unwrap();
        let cat_chain = Scope::category_chain("dexs", "OP Mainnet");
        let cases: [(&Scope, Option<Breakdown>, &str); 14] = [
            (&Scope::All, None, ""),
            (&Scope::All, Some(Breakdown::Chain), "/chain-breakdown"),
            (
                &Scope::All,
                Some(Breakdown::Protocol),
                "/protocol-breakdown",
            ),
            (&Scope::chain("Ethereum"), None, "/chain/Ethereum"),
            (
                &Scope::chain("Ethereum"),
                Some(Breakdown::Protocol),
                "/chain/Ethereum/protocol-breakdown",
            ),
            (&Scope::protocol("aave"), None, "/protocol/aave"),
            (
                &Scope::protocol("aave"),
                Some(Breakdown::Chain),
                "/protocol/aave/chain-breakdown",
            ),
            (
                &Scope::protocol("aave"),
                Some(Breakdown::Label),
                "/protocol/aave/label-breakdown",
            ),
            (
                &Scope::protocol("aave"),
                Some(Breakdown::Version),
                "/protocol/aave/version-breakdown",
            ),
            (&Scope::category("dexs"), None, "/category/dexs"),
            (
                &Scope::category("dexs"),
                Some(Breakdown::Chain),
                "/category/dexs/chain-breakdown",
            ),
            (
                &Scope::category("dexs"),
                Some(Breakdown::Protocol),
                "/category/dexs/protocol-breakdown",
            ),
            (&cat_chain, None, "/category/dexs/chain/OP%20Mainnet"),
            (
                &cat_chain,
                Some(Breakdown::Protocol),
                "/category/dexs/chain/OP%20Mainnet/protocol-breakdown",
            ),
        ];
        for (scope, by, suffix) in cases {
            assert_eq!(p(scope, by), format!("/api/v2/chart/fees{suffix}"));
        }

        let bad = [
            (Scope::All, Breakdown::Token),
            (Scope::All, Breakdown::Label),
            (Scope::chain("Ethereum"), Breakdown::Chain),
            (Scope::protocol("aave"), Breakdown::Protocol),
            (Scope::protocol("aave"), Breakdown::Token),
            (Scope::category("dexs"), Breakdown::Version),
            (cat_chain, Breakdown::Chain),
        ];
        for (scope, by) in bad {
            let err = dimension_chart_path(Dimension::Fees, &scope, Some(by)).unwrap_err();
            assert!(matches!(err, Error::Domain(_)), "{scope:?} {by:?}: {err}");
        }
    }

    #[test]
    fn oracle_chart_paths() {
        let ok = [
            (Scope::All, None, ""),
            (Scope::All, Some(Breakdown::Chain), "/chain-breakdown"),
            (Scope::All, Some(Breakdown::Protocol), "/protocol-breakdown"),
            (Scope::chain("Ethereum"), None, "/chain/Ethereum"),
            (
                Scope::chain("Ethereum"),
                Some(Breakdown::Protocol),
                "/chain/Ethereum/protocol-breakdown",
            ),
            (Scope::protocol("Chainlink"), None, "/protocol/Chainlink"),
            (
                Scope::protocol("Chainlink"),
                Some(Breakdown::Chain),
                "/protocol/Chainlink/chain-breakdown",
            ),
        ];
        for (scope, by, suffix) in ok {
            assert_eq!(
                oracle_chart_path(&scope, by).unwrap(),
                format!("/api/v2/chart/oracle{suffix}")
            );
        }
        assert!(oracle_chart_path(&Scope::category("dexs"), None).is_err());
        assert!(oracle_chart_path(&Scope::chain("Ethereum"), Some(Breakdown::Chain)).is_err());
        assert!(oracle_chart_path(&Scope::All, Some(Breakdown::Label)).is_err());
    }

    #[tokio::test]
    async fn chart() {
        let (client, server) = pro(PAIRS);
        let points = client
            .metrics()
            .chart(
                Dimension::Fees,
                &Scope::protocol("aave"),
                Some("dailyRevenue"),
            )
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/fees/protocol/aave?dataType=dailyRevenue"
        );
        assert_eq!(points, vec![ChartPoint::new(1_700_000_000, 12.5)]);

        let (client, server) = pro(PAIRS);
        client
            .metrics()
            .chart(Dimension::OpenInterest, &Scope::All, None)
            .await
            .unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/chart/open-interest");
    }

    #[tokio::test]
    async fn chart_breakdown() {
        let (client, server) = pro(PAIR_BREAKDOWN);
        let points = client
            .metrics()
            .chart_breakdown(
                Dimension::Dexs,
                &Scope::category_chain("dexs", "Ethereum"),
                Breakdown::Protocol,
                Some("dailyVolume"),
            )
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/dexs/category/dexs/chain/Ethereum/protocol-breakdown?dataType=dailyVolume"
        );
        assert_eq!(points[0].values["Ethereum"], 3.0);

        // An invalid combination fails before any request
        let client = Client::with_api_key("KEY").unwrap();
        let err = client
            .metrics()
            .chart_breakdown(Dimension::Dexs, &Scope::All, Breakdown::Version, None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Domain(_)), "{err}");
    }

    #[tokio::test]
    async fn overview() {
        let (client, server) = pro(r#"{"allChains":["Ethereum"],"total24h":10,"change_1d":-1.5,
            "category":"dexs","allCategories":["dexs"],"chain":"Ethereum"}"#);
        let o = client
            .metrics()
            .overview(
                Dimension::Fees,
                &Scope::category_chain("dexs", "Ethereum"),
                Some("dailyFees"),
            )
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/metrics/fees/category/dexs/chain/Ethereum?dataType=dailyFees"
        );
        assert_eq!(o.all_chains, ["Ethereum"]);
        assert_eq!(o.total24h, Some(10.0));
        assert_eq!(o.change_1d, Some(-1.5));
        assert_eq!(o.chain.as_deref(), Some("Ethereum"));

        for (scope, path) in [
            (Scope::All, "/KEY/api/v2/metrics/active-users"),
            (
                Scope::chain("Base"),
                "/KEY/api/v2/metrics/active-users/chain/Base",
            ),
            (
                Scope::category("lending"),
                "/KEY/api/v2/metrics/active-users/category/lending",
            ),
        ] {
            let (client, server) = pro("{}");
            client
                .metrics()
                .overview(Dimension::ActiveUsers, &scope, None)
                .await
                .unwrap();
            assert_eq!(server.path(), path);
        }

        let client = Client::with_api_key("KEY").unwrap();
        let err = client
            .metrics()
            .overview(Dimension::Fees, &Scope::protocol("aave"), None)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::Domain(_)), "{err}");
    }

    #[tokio::test]
    async fn protocol_overview() {
        let (client, server) =
            pro(r#"{"id":"1","name":"Aave","chains":["Ethereum"],"total24h":5,"cmcId":"7278"}"#);
        let o = client
            .metrics()
            .protocol_overview(Dimension::Fees, "aave", None)
            .await
            .unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/metrics/fees/protocol/aave");
        assert_eq!(o.name, "Aave");
        assert_eq!(o.total24h, Some(5.0));
        assert_eq!(o.cmc_id.as_deref(), Some("7278"));
    }

    #[tokio::test]
    async fn categories() {
        let (client, server) =
            pro(r#"[{"category":"Dexs","total24h":1,"total7d":null,"chains":["Ethereum"]}]"#);
        let c = client
            .metrics()
            .categories(Dimension::Dexs, Some("dailyVolume"))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/metrics/dexs/categories?dataType=dailyVolume"
        );
        assert_eq!(c[0].category, "Dexs");
        assert_eq!(c[0].total7d, None);
    }

    #[tokio::test]
    async fn tvl_chart_and_breakdown() {
        let (client, server) = pro(PAIRS);
        let points = client
            .metrics()
            .tvl_chart("aave", Some(TvlKey::Borrowed))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/tvl/protocol/aave?key=borrowed"
        );
        assert_eq!(points[0].value, 12.5);

        let (client, server) = pro(PAIRS);
        client.metrics().tvl_chart("aave", None).await.unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/chart/tvl/protocol/aave");

        let (client, server) = pro(PAIR_BREAKDOWN);
        client
            .metrics()
            .tvl_breakdown("aave", Breakdown::Chain, Some(TvlKey::All), None)
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/tvl/protocol/aave/chain-breakdown?key=all"
        );

        let (client, server) = pro(PAIR_BREAKDOWN);
        let points = client
            .metrics()
            .tvl_breakdown("aave", Breakdown::Token, None, Some(Currency::Raw))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/tvl/protocol/aave/token-breakdown?currency=raw"
        );
        assert_eq!(points[0].values["Solana"], 1.0);

        let client = Client::with_api_key("KEY").unwrap();
        for (by, currency) in [
            (Breakdown::Protocol, None),
            (Breakdown::Chain, Some(Currency::Usd)),
        ] {
            let err = client
                .metrics()
                .tvl_breakdown("aave", by, None, currency)
                .await
                .unwrap_err();
            assert!(matches!(err, Error::Domain(_)), "{err}");
        }
    }

    #[tokio::test]
    async fn treasury_chart_and_breakdown() {
        let (client, server) = pro(PAIRS);
        client
            .metrics()
            .treasury_chart("aave", Some(TreasuryKey::OwnTokens))
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/treasury/protocol/aave?key=OwnTokens"
        );

        let (client, server) = pro(PAIR_BREAKDOWN);
        client
            .metrics()
            .treasury_breakdown("aave", Breakdown::Chain, None, None)
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/treasury/protocol/aave/chain-breakdown"
        );

        let (client, server) = pro(PAIR_BREAKDOWN);
        client
            .metrics()
            .treasury_breakdown(
                "aave",
                Breakdown::Token,
                Some(TreasuryKey::All),
                Some(Currency::Token),
            )
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/treasury/protocol/aave/token-breakdown?key=all&currency=token"
        );
    }

    const PROTOCOL_METRICS: &str = r#"{"id":"parent#aave","name":"Aave","chains":[],
        "currentChainTvls":{"Ethereum":10.5,"Ethereum-borrowed":2},"isParentProtocol":true,
        "raises":[{"date":1594771200,"name":"Aave","round":"Seed","amount":3,"valuation":null,
            "leadInvestors":["X"],"otherInvestors":[],"defillamaId":"111"}],
        "hallmarks":[[1619395200,"Start"]],"tags":null,"tokenRights":{"rights":[]},
        "gecko_id":"aave","cmcId":"7278","audit_links":["https://a"]}"#;

    #[tokio::test]
    async fn tvl_and_treasury_metrics() {
        let (client, server) = pro(PROTOCOL_METRICS);
        let m = client.metrics().tvl_metrics("aave").await.unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/metrics/tvl/protocol/aave");
        assert_eq!(m.id, "parent#aave");
        assert_eq!(m.current_chain_tvls["Ethereum-borrowed"], 2.0);
        assert_eq!(m.raises[0].amount, Some(3.0));
        assert_eq!(m.raises[0].lead_investors, ["X"]);
        assert_eq!(m.hallmarks[0].timestamp, Some(1_619_395_200));
        assert_eq!(m.gecko_id.as_deref(), Some("aave"));
        assert_eq!(m.cmc_id.as_deref(), Some("7278"));
        assert_eq!(m.audit_links, ["https://a"]);
        assert!(m.tags.is_empty());
        assert!(m.token_rights.is_some());

        let (client, server) = pro(PROTOCOL_METRICS);
        client
            .metrics()
            .treasury_metrics("parent#aave")
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/metrics/treasury/protocol/parent%23aave"
        );
    }

    #[tokio::test]
    async fn oracle_endpoints() {
        let (client, server) = pro(PAIRS);
        client
            .metrics()
            .oracle_chart(&Scope::chain("Ethereum"))
            .await
            .unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/chart/oracle/chain/Ethereum");

        let (client, server) = pro(ROW_BREAKDOWN);
        let rows = client
            .metrics()
            .oracle_breakdown(&Scope::protocol("Chainlink"), Breakdown::Chain)
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/chart/oracle/protocol/Chainlink/chain-breakdown"
        );
        assert_eq!(rows[0].values["Chainlink"], 3.0);

        let (client, server) = pro(r#"{"oracles":{"Chainlink":["Aave"]}}"#);
        let o = client.metrics().oracles().await.unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/metrics/oracle");
        assert_eq!(o.oracles["Chainlink"], ["Aave"]);
    }

    #[tokio::test]
    async fn fork_endpoints() {
        let (client, server) = pro(PAIRS);
        client.metrics().fork_chart("uniswap-v2").await.unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/chart/fork/protocol/uniswap-v2");

        let (client, server) = pro(ROW_BREAKDOWN);
        client.metrics().fork_breakdown().await.unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/chart/fork/protocol-breakdown");

        let (client, server) = pro(r#"{"Uniswap V2":["SushiSwap"]}"#);
        let forks = client.metrics().forks().await.unwrap();
        assert_eq!(server.path(), "/KEY/api/v2/metrics/fork");
        assert_eq!(forks["Uniswap V2"], ["SushiSwap"]);
    }

    #[tokio::test]
    async fn financial_statement() {
        let (client, server) = pro(
            r#"{"id":"1","name":"Aave V3","dimensions":{"fees":"aave-v3"},
            "methodology":{"Gross Protocol Revenue":"Interest"},"methodologyURL":"https://m",
            "aggregates":{"yearly":{"2024":{"Gross Protocol Revenue":{"value":100,
                "by-label":{"Borrow Interest":90}}}},"quarterly":{},"monthly":{}}}"#,
        );
        let s = client
            .metrics()
            .financial_statement("aave-v3")
            .await
            .unwrap();
        assert_eq!(
            server.path(),
            "/KEY/api/v2/metrics/financial-statement/protocol/aave-v3"
        );
        assert_eq!(s.dimensions["fees"].adapter.as_deref(), Some("aave-v3"));
        assert_eq!(s.methodology_url.as_deref(), Some("https://m"));
        let item = &s.aggregates.yearly["2024"]["Gross Protocol Revenue"];
        assert_eq!(item.value, Some(100.0));
        assert_eq!(item.by_label["Borrow Interest"], 90.0);
    }
}
