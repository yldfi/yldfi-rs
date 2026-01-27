//! ETF API endpoints (Pro)

use crate::client::Client;
use crate::error::Result;

use super::types::{EtfFlow, EtfHistoryPoint, EtfOverview, EtfSnapshot, FdvPerformance};

/// ETF API client (Pro only)
pub struct EtfApi<'a> {
    client: &'a Client,
}

impl<'a> EtfApi<'a> {
    /// Create a new ETF API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get Bitcoin ETF overview
    ///
    /// **Requires Pro API key**
    ///
    /// Returns current snapshot of all BTC ETFs with AUM, flows, fees, and volume.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let overview = client.etf().overview().await?;
    /// println!("Total BTC ETF AUM: ${:.0}B", overview.total_aum.unwrap_or(0.0) / 1_000_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn overview(&self) -> Result<EtfOverview> {
        self.client.get_pro("/etfs/overview").await
    }

    /// Get Ethereum ETF overview
    ///
    /// **Requires Pro API key**
    ///
    /// Returns current snapshot of all ETH ETFs with AUM, flows, fees, and volume.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let overview = client.etf().overview_eth().await?;
    /// println!("Total ETH ETF AUM: ${:.0}B", overview.total_aum.unwrap_or(0.0) / 1_000_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn overview_eth(&self) -> Result<EtfOverview> {
        self.client.get_pro("/etfs/overviewEth").await
    }

    /// Get Bitcoin ETF historical data
    ///
    /// **Requires Pro API key**
    ///
    /// Returns daily historical data for all BTC ETFs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let history = client.etf().history().await?;
    /// for point in history.iter().take(5) {
    ///     println!("{}: ${:.0}B AUM",
    ///         point.date.as_deref().unwrap_or("?"),
    ///         point.total_aum.unwrap_or(0.0) / 1_000_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn history(&self) -> Result<Vec<EtfHistoryPoint>> {
        self.client.get_pro("/etfs/history").await
    }

    /// Get Ethereum ETF historical data
    ///
    /// **Requires Pro API key**
    ///
    /// Returns daily historical data for all ETH ETFs.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let history = client.etf().history_eth().await?;
    /// for point in history.iter().take(5) {
    ///     println!("{}: ${:.0}B AUM",
    ///         point.date.as_deref().unwrap_or("?"),
    ///         point.total_aum.unwrap_or(0.0) / 1_000_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn history_eth(&self) -> Result<Vec<EtfHistoryPoint>> {
        self.client.get_pro("/etfs/historyEth").await
    }

    /// Get FDV performance metrics by category
    ///
    /// **Requires Pro API key**
    ///
    /// Returns category performance data weighted by market cap.
    ///
    /// # Arguments
    ///
    /// * `period` - Time period (e.g., "1d", "7d", "30d")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let perf = client.etf().fdv_performance("7d").await?;
    /// for p in perf.iter().take(5) {
    ///     println!("{}: {:.2}%",
    ///         p.name.as_deref().unwrap_or("?"),
    ///         p.performance.unwrap_or(0.0) * 100.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fdv_performance(&self, period: &str) -> Result<Vec<FdvPerformance>> {
        let path = format!("/fdv/performance/{period}");
        self.client.get_pro(&path).await
    }

    /// Get ETF daily flows
    ///
    /// **Requires Pro API key**
    ///
    /// Returns daily USD flows aggregated by asset (BTC, ETH).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let flows = client.etf().flows().await?;
    /// for flow in flows.iter().take(5) {
    ///     println!("{}: ${:.0}M flow",
    ///         flow.day.as_deref().unwrap_or("?"),
    ///         flow.total_flow_usd.unwrap_or(0.0) / 1_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn flows(&self) -> Result<Vec<EtfFlow>> {
        self.client.get_pro("/etfs/flows").await
    }

    /// Get ETF snapshot
    ///
    /// **Requires Pro API key**
    ///
    /// Returns current snapshot of all ETFs with AUM, flows, volume, and fees.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-api-key")?;
    /// let snapshot = client.etf().snapshot().await?;
    /// for etf in snapshot.iter().take(5) {
    ///     println!("{}: ${:.0}M AUM",
    ///         etf.ticker.as_deref().unwrap_or("?"),
    ///         etf.aum.unwrap_or(0.0) / 1_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn snapshot(&self) -> Result<Vec<EtfSnapshot>> {
        self.client.get_pro("/etfs/snapshot").await
    }
}
