//! Volume API endpoints (DEX, Options, Derivatives)

use crate::client::Client;
use crate::error::Result;

use super::types::*;

/// Volumes API client
pub struct VolumesApi<'a> {
    client: &'a Client,
}

impl<'a> VolumesApi<'a> {
    /// Create a new Volumes API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    // ==================== DEX Endpoints ====================

    /// Get aggregated DEX volumes overview
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let dex = client.volumes().dex_overview().await?;
    /// println!("24h DEX volume: ${:.0}M", dex.total24h.unwrap_or(0.0) / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dex_overview(&self) -> Result<VolumeOverview> {
        self.client.get_main("/overview/dexs").await
    }

    /// Get aggregated DEX volumes overview with options
    ///
    /// # Arguments
    ///
    /// * `options` - Query options (exclude charts, data type filter)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// use llama::volumes::VolumeOverviewOptions;
    ///
    /// let client = llama::Client::new()?;
    /// let options = VolumeOverviewOptions::new().exclude_charts();
    /// let dex = client.volumes().dex_overview_with_options(&options).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dex_overview_with_options(
        &self,
        options: &VolumeOverviewOptions,
    ) -> Result<VolumeOverview> {
        let path = format!("/overview/dexs{}", options.to_query_string());
        self.client.get_main(&path).await
    }

    /// Get DEX volumes for a specific chain
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "Arbitrum", "BSC")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let dex = client.volumes().dex_chain("Ethereum").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dex_chain(&self, chain: &str) -> Result<VolumeOverview> {
        let path = format!("/overview/dexs/{}", chain);
        self.client.get_main(&path).await
    }

    /// Get volume summary for a specific DEX protocol
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol slug (e.g., "uniswap", "curve", "pancakeswap")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let uniswap = client.volumes().dex_protocol("uniswap").await?;
    /// println!("Uniswap 24h: ${:.0}M", uniswap.total24h.unwrap_or(0.0) / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn dex_protocol(&self, protocol: &str) -> Result<ProtocolVolumeSummary> {
        let path = format!("/summary/dexs/{}", protocol);
        self.client.get_main(&path).await
    }

    // ==================== Options Endpoints ====================

    /// Get aggregated options trading volumes overview
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let options = client.volumes().options_overview().await?;
    /// println!("24h Options volume: ${:.0}M", options.total24h.unwrap_or(0.0) / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn options_overview(&self) -> Result<VolumeOverview> {
        self.client.get_main("/overview/options").await
    }

    /// Get options volumes for a specific chain
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "Arbitrum")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let options = client.volumes().options_chain("Ethereum").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn options_chain(&self, chain: &str) -> Result<VolumeOverview> {
        let path = format!("/overview/options/{}", chain);
        self.client.get_main(&path).await
    }

    /// Get volume summary for a specific options protocol
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol slug (e.g., "lyra", "dopex", "hegic")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let lyra = client.volumes().options_protocol("lyra").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn options_protocol(&self, protocol: &str) -> Result<ProtocolVolumeSummary> {
        let path = format!("/summary/options/{}", protocol);
        self.client.get_main(&path).await
    }

    // ==================== Derivatives Endpoints (Pro) ====================

    /// Get aggregated derivatives volumes overview
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let derivs = client.volumes().derivatives_overview().await?;
    /// println!("24h Derivatives volume: ${:.0}M", derivs.total24h.unwrap_or(0.0) / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn derivatives_overview(&self) -> Result<VolumeOverview> {
        self.client.get_pro("/overview/derivatives").await
    }

    /// Get volume summary for a specific derivatives protocol
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol slug (e.g., "gmx", "gains-network", "hyperliquid")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let gmx = client.volumes().derivatives_protocol("gmx").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn derivatives_protocol(&self, protocol: &str) -> Result<ProtocolVolumeSummary> {
        let path = format!("/summary/derivatives/{}", protocol);
        self.client.get_pro(&path).await
    }

    // ==================== Open Interest Endpoints ====================

    /// Get open interest overview for perpetual futures
    ///
    /// Returns aggregated open interest data across all perpetual futures protocols.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let oi = client.volumes().open_interest().await?;
    /// for protocol in oi.protocols.iter().take(5) {
    ///     println!("{}: ${:.0}B OI",
    ///         protocol.name.as_deref().unwrap_or("?"),
    ///         protocol.total24h.unwrap_or(0.0) / 1_000_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn open_interest(&self) -> Result<OpenInterestOverview> {
        self.client.get_main("/overview/open-interest").await
    }
}
