//! ETF API endpoints (Pro)

use crate::client::Client;
use crate::error::Result;

use super::types::{EtfFlow, EtfHistoryPoint, EtfOverview, EtfSnapshot, FdvPerformance};

/// Error for ETF endpoints DefiLlama has removed from the Pro API
fn removed(path: &str, replacement: &str) -> crate::error::Error {
    crate::error::Error::api(
        410,
        format!("DefiLlama removed {path}; use {replacement} instead"),
    )
}

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
    /// Removed upstream: always returns an error without making a request.
    #[deprecated(
        since = "0.1.5",
        note = "DefiLlama removed /etfs/overview; use `snapshot()` (/etfs/snapshot) and filter by asset"
    )]
    pub async fn overview(&self) -> Result<EtfOverview> {
        Err(removed("/etfs/overview", "snapshot() (/etfs/snapshot)"))
    }

    /// Get Ethereum ETF overview
    ///
    /// Removed upstream: always returns an error without making a request.
    #[deprecated(
        since = "0.1.5",
        note = "DefiLlama removed /etfs/overviewEth; use `snapshot()` (/etfs/snapshot) and filter by asset"
    )]
    pub async fn overview_eth(&self) -> Result<EtfOverview> {
        Err(removed("/etfs/overviewEth", "snapshot() (/etfs/snapshot)"))
    }

    /// Get Bitcoin ETF historical data
    ///
    /// Removed upstream: always returns an error without making a request.
    #[deprecated(
        since = "0.1.5",
        note = "DefiLlama removed /etfs/history; use `flows()` (/etfs/flows) for daily per-asset flows"
    )]
    pub async fn history(&self) -> Result<Vec<EtfHistoryPoint>> {
        Err(removed("/etfs/history", "flows() (/etfs/flows)"))
    }

    /// Get Ethereum ETF historical data
    ///
    /// Removed upstream: always returns an error without making a request.
    #[deprecated(
        since = "0.1.5",
        note = "DefiLlama removed /etfs/historyEth; use `flows()` (/etfs/flows) for daily per-asset flows"
    )]
    pub async fn history_eth(&self) -> Result<Vec<EtfHistoryPoint>> {
        Err(removed("/etfs/historyEth", "flows() (/etfs/flows)"))
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
