//! Yields API endpoints
//!
//! Some yields endpoints are free (pools, chart) and some require Pro API key.

use crate::client::Client;
use crate::error::Result;

use super::types::{YieldPool, YieldsResponse, YieldChartPoint, LegacyPool, BorrowPool, LendBorrowChartPoint, PerpRate, LsdRate};

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
    /// This is a **free** endpoint that doesn't require an API key.
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
        let resp: YieldsResponse<Vec<YieldPool>> = self.client.get_yields("/pools").await?;
        Ok(resp.data)
    }

    /// Get historical APY/TVL chart for a pool
    ///
    /// This is a **free** endpoint that doesn't require an API key.
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
        let path = format!("/chart/{pool}");
        let resp: YieldsResponse<Vec<YieldChartPoint>> = self.client.get_yields(&path).await?;
        Ok(resp.data)
    }

    /// Get legacy pools with contract addresses
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
        let resp: YieldsResponse<Vec<LegacyPool>> = self.client.get_pro("/yields/poolsOld").await?;
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
    ///         pool.apy_borrow.unwrap_or(0.0));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn pools_borrow(&self) -> Result<Vec<BorrowPool>> {
        let resp: YieldsResponse<Vec<BorrowPool>> =
            self.client.get_pro("/yields/poolsBorrow").await?;
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
        let path = format!("/yields/chartLendBorrow/{pool}");
        let resp: YieldsResponse<Vec<LendBorrowChartPoint>> = self.client.get_pro(&path).await?;
        Ok(resp.data)
    }

    /// Get perpetual futures funding rates
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
        let resp: YieldsResponse<Vec<PerpRate>> = self.client.get_pro("/yields/perps").await?;
        Ok(resp.data)
    }

    /// Get liquid staking derivative rates
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let lsd = client.yields().lsd_rates().await?;
    /// for rate in lsd.iter().take(5) {
    ///     println!("{}: {:.2}% APY", rate.name, rate.apy.unwrap_or(0.0));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lsd_rates(&self) -> Result<Vec<LsdRate>> {
        // lsdRates returns array directly, not wrapped in {status, data}
        self.client.get_pro("/yields/lsdRates").await
    }
}
