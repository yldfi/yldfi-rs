//! Fees and revenue API endpoints

use crate::client::Client;
use crate::error::Result;

use super::types::{FeesOverview, FeesOverviewOptions, ProtocolFeesSummary};

/// Fees API client
pub struct FeesApi<'a> {
    client: &'a Client,
}

impl<'a> FeesApi<'a> {
    /// Create a new Fees API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get aggregated protocol fees overview
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let fees = client.fees().overview().await?;
    /// println!("24h fees: ${:.0}M", fees.total24h.unwrap_or(0.0) / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn overview(&self) -> Result<FeesOverview> {
        self.client.get_main("/overview/fees").await
    }

    /// Get aggregated protocol fees overview with options
    ///
    /// # Arguments
    ///
    /// * `options` - Query options (exclude charts, data type filter)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::fees::FeesOverviewOptions;
    ///
    /// let client = dllma::Client::new()?;
    /// let options = FeesOverviewOptions::new().exclude_charts();
    /// let fees = client.fees().overview_with_options(&options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn overview_with_options(
        &self,
        options: &FeesOverviewOptions,
    ) -> Result<FeesOverview> {
        let path = format!("/overview/fees{}", options.to_query_string());
        self.client.get_main(&path).await
    }

    /// Get fees for a specific chain
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "Arbitrum", "BSC")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let fees = client.fees().chain("Ethereum").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chain(&self, chain: &str) -> Result<FeesOverview> {
        let path = format!("/overview/fees/{chain}");
        self.client.get_main(&path).await
    }

    /// Get fees summary for a specific protocol
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol slug (e.g., "uniswap", "aave", "lido")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let uniswap = client.fees().protocol("uniswap").await?;
    /// println!("Uniswap 24h fees: ${:.0}M", uniswap.total24h.unwrap_or(0.0) / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn protocol(&self, protocol: &str) -> Result<ProtocolFeesSummary> {
        let path = format!("/summary/fees/{protocol}");
        self.client.get_main(&path).await
    }
}
