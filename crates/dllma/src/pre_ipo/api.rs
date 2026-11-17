//! Pre-IPO API endpoints (Pro)

use crate::client::{with_query, Client};
use crate::error::Result;

use super::types::{
    FundingRound, PreIpoCompany, PreIpoIntegrations, PreIpoResponse, PreIpoSummary, Valuation,
};

/// Pre-IPO company API client (Pro only)
///
/// Companies are identified by the `id` from [`Self::companies`]
/// (case-insensitive), not by display name. Each method returns the
/// snapshot time with the data.
pub struct PreIpoApi<'a> {
    client: &'a Client,
}

impl<'a> PreIpoApi<'a> {
    /// Create a new pre-IPO API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn company<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        company: &str,
    ) -> Result<PreIpoResponse<T>> {
        let path = with_query(
            format!("/pre-ipo/v1/{endpoint}"),
            [("company", Some(company))],
        );
        self.client.get_pro(&path).await
    }

    /// List tracked pre-IPO companies (`GET /pre-ipo/v1/companies-list`)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let companies = client.pre_ipo().companies().await?;
    /// for c in companies.data.iter().take(5) {
    ///     println!("{}: {:?}", c.id, c.latest_estimated_valuation);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn companies(&self) -> Result<PreIpoResponse<Vec<PreIpoCompany>>> {
        self.client.get_pro("/pre-ipo/v1/companies-list").await
    }

    /// Get a company's monthly valuation estimates
    /// (`GET /pre-ipo/v1/valuations`)
    ///
    /// **Requires Pro API key**
    pub async fn valuations(&self, company: &str) -> Result<PreIpoResponse<Vec<Valuation>>> {
        self.company("valuations", company).await
    }

    /// Get a company's funding rounds (`GET /pre-ipo/v1/raises`)
    ///
    /// **Requires Pro API key**
    pub async fn raises(&self, company: &str) -> Result<PreIpoResponse<Vec<FundingRound>>> {
        self.company("raises", company).await
    }

    /// Get a company's profile and latest valuation
    /// (`GET /pre-ipo/v1/summary`)
    ///
    /// **Requires Pro API key**
    pub async fn summary(&self, company: &str) -> Result<PreIpoResponse<PreIpoSummary>> {
        self.company("summary", company).await
    }

    /// List perps and tokens tracking a company
    /// (`GET /pre-ipo/v1/integrations`)
    ///
    /// **Requires Pro API key**
    pub async fn integrations(&self, company: &str) -> Result<PreIpoResponse<PreIpoIntegrations>> {
        self.company("integrations", company).await
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
        let err = client.pre_ipo().companies().await.unwrap_err();
        assert!(matches!(err, Error::Api { status: 401, .. }), "{err}");
    }

    #[tokio::test]
    async fn companies() {
        let (client, server) = pro(
            r#"{"updatedAt":"2026-11-01T00:00:00Z","data":[{"id":"spacex","name":"SpaceX",
            "latestEstimatedValuation":4e11,"valuationChange1YPercentage":25.5,
            "premiumVsLatestFundingPercentage":null,"latestFundingDate":"2025-12-01"}]}"#,
        );
        let r = client.pre_ipo().companies().await.unwrap();
        assert_eq!(server.path(), "/KEY/pre-ipo/v1/companies-list");
        assert_eq!(r.updated_at.as_deref(), Some("2026-11-01T00:00:00Z"));
        let c = &r.data[0];
        assert_eq!(c.id, "spacex");
        assert_eq!(c.latest_estimated_valuation, Some(4e11));
        assert_eq!(c.valuation_change_1y_percentage, Some(25.5));
        assert_eq!(c.latest_funding_date.as_deref(), Some("2025-12-01"));
    }

    #[tokio::test]
    async fn valuations() {
        let (client, server) =
            pro(r#"{"data":[{"date":"2026-10-01","estimatedValuation":1.5e11}]}"#);
        let r = client.pre_ipo().valuations("Stripe").await.unwrap();
        assert_eq!(server.path(), "/KEY/pre-ipo/v1/valuations?company=Stripe");
        assert_eq!(r.data[0].estimated_valuation, Some(1.5e11));
    }

    #[tokio::test]
    async fn raises() {
        let (client, server) = pro(
            r#"{"data":[{"date":"2025-01-01","amountRaised":1e9,"valuation":null,
            "series":"Series H-4"}]}"#,
        );
        let r = client.pre_ipo().raises("stripe").await.unwrap();
        assert_eq!(server.path(), "/KEY/pre-ipo/v1/raises?company=stripe");
        assert_eq!(r.data[0].amount_raised, Some(1e9));
        assert_eq!(r.data[0].valuation, None);
        assert_eq!(r.data[0].series.as_deref(), Some("Series H-4"));
    }

    #[tokio::test]
    async fn summary() {
        let (client, server) = pro(
            r#"{"data":{"id":"stripe","summary":"Payments","latestRaise":{"date":"2025-01-01",
            "amountRaised":5,"valuation":10,"series":"Series I"}}}"#,
        );
        let r = client.pre_ipo().summary("stripe").await.unwrap();
        assert_eq!(server.path(), "/KEY/pre-ipo/v1/summary?company=stripe");
        assert_eq!(r.data.summary.as_deref(), Some("Payments"));
        assert_eq!(r.data.latest_raise.unwrap().valuation, Some(10.0));
    }

    #[tokio::test]
    async fn integrations() {
        let (client, server) = pro(
            r#"{"data":{"perps":[{"exchangeName":"X","defillamaContract":"x:SPACEX",
            "price":200,"annualizedFundingRate":5.2}],"tokenIssuers":[{"issuerName":"Y",
            "slug":null,"backing":"Trust-held SPV","marketCap":3}]}}"#,
        );
        let r = client.pre_ipo().integrations("spacex").await.unwrap();
        assert_eq!(server.path(), "/KEY/pre-ipo/v1/integrations?company=spacex");
        assert_eq!(
            r.data.perps[0].defillama_contract.as_deref(),
            Some("x:SPACEX")
        );
        assert_eq!(r.data.perps[0].annualized_funding_rate, Some(5.2));
        assert_eq!(
            r.data.token_issuers[0].backing.as_deref(),
            Some("Trust-held SPV")
        );
        assert_eq!(r.data.token_issuers[0].market_cap, Some(3.0));
    }
}
