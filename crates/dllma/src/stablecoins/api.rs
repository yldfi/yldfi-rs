//! Stablecoins API endpoints

use crate::client::Client;
use crate::error::Result;

use super::types::{StablecoinsResponse, StablecoinDetail, StablecoinChartPoint, StablecoinDominance, StablecoinChain, StablecoinPricesResponse};

/// Stablecoins API client
pub struct StablecoinsApi<'a> {
    client: &'a Client,
}

impl<'a> StablecoinsApi<'a> {
    /// Create a new Stablecoins API client
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all stablecoins with market data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let stables = client.stablecoins().list().await?;
    /// for stable in stables.pegged_assets.iter().take(5) {
    ///     println!("{}: {}", stable.name, stable.symbol);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(&self) -> Result<StablecoinsResponse> {
        self.client.get_stablecoins("/stablecoins").await
    }

    /// Get detailed data for a specific stablecoin
    ///
    /// # Arguments
    ///
    /// * `id` - Stablecoin ID (e.g., "1" for USDT, "2" for USDC)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let usdt = client.stablecoins().get("1").await?;
    /// println!("{}: {:?}", usdt.name, usdt.circulating);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<StablecoinDetail> {
        let path = format!("/stablecoin/{id}");
        self.client.get_stablecoins(&path).await
    }

    /// Get historical market cap for all stablecoins
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let history = client.stablecoins().charts_all().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn charts_all(&self) -> Result<Vec<StablecoinChartPoint>> {
        self.client.get_stablecoins("/stablecoincharts/all").await
    }

    /// Get historical stablecoin data for a specific chain
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "BSC", "Polygon")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let history = client.stablecoins().charts_chain("Ethereum").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn charts_chain(&self, chain: &str) -> Result<Vec<StablecoinChartPoint>> {
        let path = format!("/stablecoincharts/{chain}");
        self.client.get_stablecoins(&path).await
    }

    /// Get historical stablecoin data for a specific chain, filtered by stablecoin ID
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "BSC", "Polygon")
    /// * `stablecoin_id` - Stablecoin ID to filter by (e.g., "1" for USDT)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// // Get only USDT history on Ethereum
    /// let history = client.stablecoins().charts_chain_filtered("Ethereum", "1").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn charts_chain_filtered(
        &self,
        chain: &str,
        stablecoin_id: &str,
    ) -> Result<Vec<StablecoinChartPoint>> {
        let path = format!("/stablecoincharts/{chain}?stablecoin={stablecoin_id}");
        self.client.get_stablecoins(&path).await
    }

    /// Get stablecoin dominance for a specific chain
    ///
    /// Returns the percentage breakdown of stablecoins on a chain.
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "BSC", "Polygon")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let dominance = client.stablecoins().dominance("Ethereum").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dominance(&self, chain: &str) -> Result<Vec<StablecoinDominance>> {
        let path = format!("/stablecoindominance/{chain}");
        self.client.get_stablecoins(&path).await
    }

    /// List all chains with stablecoin data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let chains = client.stablecoins().chains().await?;
    /// for chain in chains.iter().take(5) {
    ///     let name = chain.name.as_deref().or(chain.gecko_id.as_deref()).unwrap_or("Unknown");
    ///     println!("{}", name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chains(&self) -> Result<Vec<StablecoinChain>> {
        self.client.get_stablecoins("/stablecoinchains").await
    }

    /// Get historical prices for all stablecoins
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let prices = client.stablecoins().prices().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn prices(&self) -> Result<StablecoinPricesResponse> {
        self.client.get_stablecoins("/stablecoinprices").await
    }
}
