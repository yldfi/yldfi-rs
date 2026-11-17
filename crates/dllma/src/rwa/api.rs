//! RWA API endpoints (Pro)

use crate::client::{path_segment as seg, with_query, Client};
use crate::error::Result;
use crate::metrics::BreakdownPoint;

use super::types::{RwaAsset, RwaBreakdownQuery, RwaChartPoint, RwaData, RwaList, RwaStats};

/// Real-world asset (RWA) API client (Pro only)
pub struct RwaApi<'a> {
    client: &'a Client,
}

impl<'a> RwaApi<'a> {
    /// Create a new RWA API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all current RWA assets (`GET /rwa/current`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let assets = client.rwa().current().await?;
    /// for a in assets.iter().take(5) {
    ///     let aum: f64 = a.active_aum.values().sum();
    ///     println!("{:?}: ${aum:.0}", a.ticker);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn current(&self) -> Result<Vec<RwaAsset>> {
        self.client.get_pro("/rwa/current").await
    }

    /// List RWA ids and filter values (`GET /rwa/list`)
    ///
    /// **Requires Pro API key**
    pub async fn list(&self) -> Result<RwaList> {
        self.client.get_pro("/rwa/list").await
    }

    /// Get RWA aggregates by chain, category, platform and asset group
    /// (`GET /rwa/stats`)
    ///
    /// **Requires Pro API key**
    pub async fn stats(&self) -> Result<RwaStats> {
        self.client.get_pro("/rwa/stats").await
    }

    /// List the RWA assets on a chain, by display name
    /// (`GET /rwa/chain/{chain}`)
    ///
    /// **Requires Pro API key**
    pub async fn chain(&self, chain: &str) -> Result<Vec<RwaAsset>> {
        let resp: RwaData<Vec<RwaAsset>> = self
            .client
            .get_pro(&format!("/rwa/chain/{}", seg(chain)))
            .await?;
        Ok(resp.data)
    }

    /// Get a chain's RWA totals over time (`GET /rwa/chart/chain/{chain}`)
    ///
    /// **Requires Pro API key**
    pub async fn chain_chart(&self, chain: &str) -> Result<Vec<RwaChartPoint>> {
        self.client
            .get_pro(&format!("/rwa/chart/chain/{}", seg(chain)))
            .await
    }

    /// Get one RWA measure over time, one series per chain
    /// (`GET /rwa/chart/chain-breakdown`)
    ///
    /// **Requires Pro API key**
    pub async fn chain_breakdown(&self, query: &RwaBreakdownQuery) -> Result<Vec<BreakdownPoint>> {
        let flag = |b: Option<bool>| b.map(|b| if b { "true" } else { "false" });
        let path = with_query(
            "/rwa/chart/chain-breakdown".to_owned(),
            [
                ("key", query.metric.map(super::RwaMetric::as_str)),
                ("includeStablecoin", flag(query.include_stablecoin)),
                ("includeGovernance", flag(query.include_governance)),
            ],
        );
        self.client.get_pro(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Error;
    use crate::rwa::RwaMetric;
    use crate::test_server::TestServer;

    fn pro(body: &str) -> (Client, TestServer) {
        let server = TestServer::json(body);
        let client = Client::with_api_key("KEY")
            .unwrap()
            .with_test_base(&server.url);
        (client, server)
    }

    const ASSET: &str = r#"{"id":"1","canonicalMarketId":"ondo/usdy","ticker":"USDY",
        "chain":["Ethereum"],"category":null,"type":"Token","stablecoin":false,
        "onChainMcap":{"Ethereum":100},"activeAum":{"Ethereum":80},
        "defiActiveTvl":{"Ethereum":{"aave-v3":10}},"activeAumData":true}"#;

    #[tokio::test]
    async fn needs_api_key() {
        let client = Client::new().unwrap();
        let err = client.rwa().current().await.unwrap_err();
        assert!(matches!(err, Error::Api { status: 401, .. }), "{err}");
    }

    #[tokio::test]
    async fn current() {
        let (client, server) = pro(&format!("[{ASSET}]"));
        let a = client.rwa().current().await.unwrap();
        assert_eq!(server.path(), "/KEY/rwa/current");
        let a = &a[0];
        assert_eq!(a.canonical_market_id.as_deref(), Some("ondo/usdy"));
        assert_eq!(a.kind.as_deref(), Some("Token"));
        assert!(a.category.is_empty());
        assert_eq!(a.active_aum["Ethereum"], 80.0);
        assert_eq!(a.defi_active_tvl["Ethereum"]["aave-v3"], 10.0);
        assert_eq!(a.active_aum_data, Some(true));
    }

    #[tokio::test]
    async fn list() {
        let (client, server) = pro(
            r#"{"canonicalMarketIds":["ondo/usdy"],"assetGroups":["Treasuries"],
            "idMap":{"ondo/usdy":"1"}}"#,
        );
        let l = client.rwa().list().await.unwrap();
        assert_eq!(server.path(), "/KEY/rwa/list");
        assert_eq!(l.asset_groups, ["Treasuries"]);
        assert_eq!(l.id_map["ondo/usdy"], "1");
    }

    #[tokio::test]
    async fn stats() {
        let (client, server) = pro(
            r#"{"totalOnChainMcap":10,"totalActiveMcap":8,"assetCount":2,"byChain":{"Ethereum":{
            "base":{"onChainMcap":5,"assetCount":1,"assetIssuers":["Ondo"]},
            "stablecoinsOnly":{"activeAum":3}}}}"#,
        );
        let s = client.rwa().stats().await.unwrap();
        assert_eq!(server.path(), "/KEY/rwa/stats");
        assert_eq!(s.total_on_chain_mcap, Some(10.0));
        assert_eq!(s.total_active_mcap, Some(8.0));
        let eth = &s.by_chain["Ethereum"];
        assert_eq!(eth.base.asset_issuers, ["Ondo"]);
        assert_eq!(eth.stablecoins_only.active_aum, Some(3.0));
        assert_eq!(eth.governance_only.asset_count, None);
    }

    #[tokio::test]
    async fn chain() {
        let (client, server) = pro(&format!(r#"{{"data":[{ASSET}]}}"#));
        let a = client.rwa().chain("OP Mainnet").await.unwrap();
        assert_eq!(server.path(), "/KEY/rwa/chain/OP%20Mainnet");
        assert_eq!(a[0].ticker.as_deref(), Some("USDY"));
    }

    #[tokio::test]
    async fn chain_chart() {
        let (client, server) =
            pro(r#"[{"timestamp":1700000000,"onChainMcap":1,"activeAum":null,"defiActiveTvl":2}]"#);
        let c = client.rwa().chain_chart("Ethereum").await.unwrap();
        assert_eq!(server.path(), "/KEY/rwa/chart/chain/Ethereum");
        assert_eq!(c[0].timestamp, Some(1_700_000_000));
        assert_eq!(c[0].active_aum, None);
        assert_eq!(c[0].defi_active_tvl, Some(2.0));
    }

    #[tokio::test]
    async fn chain_breakdown() {
        let (client, server) = pro(r#"[{"timestamp":1700000000,"Ethereum":5,"Solana":1}]"#);
        let query = RwaBreakdownQuery::new()
            .metric(RwaMetric::ActiveAum)
            .include_stablecoin(true)
            .include_governance(false);
        let rows = client.rwa().chain_breakdown(&query).await.unwrap();
        assert_eq!(
            server.path(),
            "/KEY/rwa/chart/chain-breakdown?key=activeAum&includeStablecoin=true&includeGovernance=false"
        );
        assert_eq!(rows[0].values["Ethereum"], 5.0);

        let (client, server) = pro("[]");
        client
            .rwa()
            .chain_breakdown(&RwaBreakdownQuery::new())
            .await
            .unwrap();
        assert_eq!(server.path(), "/KEY/rwa/chart/chain-breakdown");
    }
}
