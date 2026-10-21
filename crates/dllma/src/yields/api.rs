//! Yields API endpoints
//!
//! `DefiLlama` shuts down the free yields endpoints (`yields.llama.fi/pools`
//! and `/chart/{pool}`) on 2026-11-13; from then on yields data needs the
//! API plan. Clients with an API key already use the Pro `/yields/v1/*`
//! routes, which return the same format. Without a key, [`YieldsApi::pools`]
//! and [`YieldsApi::chart`] keep using the free host and explain the
//! shutdown when it fails.

use crate::client::Client;
use crate::error::{Error, Result};

use super::types::{
    BorrowPool, LegacyPool, LendBorrowChartPoint, LsdRate, LstRate, PerpRate, YieldChartPoint,
    YieldPool, YieldsResponse,
};

/// Date (UTC) on which `DefiLlama` stops serving the free yields endpoints.
pub const FREE_YIELDS_SHUTDOWN: &str = "2026-11-13";

/// Add the free-endpoint shutdown to an error from `yields.llama.fi`.
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
             after that, yields data needs an API plan key: set DEFILLAMA_API_KEY or use \
             Client::with_api_key())"
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
}
