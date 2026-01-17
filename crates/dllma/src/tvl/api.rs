//! TVL and protocol API endpoints

use crate::client::Client;
use crate::error::Result;

use super::types::{
    ChainAssets, ChainHistoricalTvl, ChainsResponse, Protocol, ProtocolDetail, ProtocolInflows,
    TokenProtocol,
};

/// TVL API client
pub struct TvlApi<'a> {
    client: &'a Client,
}

impl<'a> TvlApi<'a> {
    /// Create a new TVL API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all protocols with their current TVL
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let protocols = client.tvl().protocols().await?;
    /// for protocol in protocols.iter().take(5) {
    ///     println!("{}: ${:.0}M TVL", protocol.name, protocol.tvl.unwrap_or(0.0) / 1_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn protocols(&self) -> Result<Vec<Protocol>> {
        self.client.get_main("/protocols").await
    }

    /// Get detailed protocol data with historical TVL
    ///
    /// # Arguments
    ///
    /// * `slug` - Protocol slug (e.g., "aave", "uniswap", "lido")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let aave = client.tvl().protocol("aave").await?;
    /// println!("{}: ${:.0}M TVL", aave.name, aave.tvl.unwrap_or(0.0) / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn protocol(&self, slug: &str) -> Result<ProtocolDetail> {
        let path = format!("/protocol/{}", slug);
        self.client.get_main(&path).await
    }

    /// Get current TVL for a protocol (just the number)
    ///
    /// # Arguments
    ///
    /// * `slug` - Protocol slug (e.g., "aave", "uniswap", "lido")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let tvl = client.tvl().protocol_tvl("aave").await?;
    /// println!("Aave TVL: ${:.0}M", tvl / 1_000_000.0);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn protocol_tvl(&self, slug: &str) -> Result<f64> {
        let path = format!("/tvl/{}", slug);
        self.client.get_main(&path).await
    }

    /// Get current TVL of all chains
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let chains = client.tvl().chains().await?;
    /// for chain in chains.0.iter().take(5) {
    ///     let name = chain.name.as_deref().or(chain.gecko_id.as_deref()).unwrap_or("Unknown");
    ///     println!("{}: ${:.0}B TVL", name, chain.tvl / 1_000_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chains(&self) -> Result<ChainsResponse> {
        self.client.get_main("/v2/chains").await
    }

    /// Get historical TVL for all chains combined
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let history = client.tvl().historical_tvl().await?;
    /// if let Some(latest) = history.0.last() {
    ///     println!("Latest total TVL: ${:.0}B", latest.tvl / 1_000_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn historical_tvl(&self) -> Result<ChainHistoricalTvl> {
        self.client.get_main("/v2/historicalChainTvl").await
    }

    /// Get historical TVL for a specific chain
    ///
    /// # Arguments
    ///
    /// * `chain` - Chain name (e.g., "Ethereum", "Arbitrum", "Polygon")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::new()?;
    /// let history = client.tvl().chain_historical_tvl("Ethereum").await?;
    /// if let Some(latest) = history.0.last() {
    ///     println!("Ethereum TVL: ${:.0}B", latest.tvl / 1_000_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chain_historical_tvl(&self, chain: &str) -> Result<ChainHistoricalTvl> {
        let path = format!("/v2/historicalChainTvl/{}", chain);
        self.client.get_main(&path).await
    }

    /// Get protocols holding a specific token (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// Returns which protocols hold a specific token and the amounts.
    ///
    /// # Arguments
    ///
    /// * `symbol` - Token symbol (e.g., "usdt", "weth", "wbtc")
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let protocols = client.tvl().token_protocols("usdt").await?;
    /// for p in protocols.iter().take(5) {
    ///     println!("{}: {:?}", p.name, p.category);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn token_protocols(&self, symbol: &str) -> Result<Vec<TokenProtocol>> {
        let path = format!("/tokenProtocols/{}", symbol);
        self.client.get_pro(&path).await
    }

    /// Get capital inflows/outflows for a protocol at a timestamp (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// Shows daily capital flows (inflows and outflows) for a protocol.
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol slug (e.g., "compound-v3", "aave")
    /// * `timestamp` - Unix timestamp for the date
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let flows = client.tvl().inflows("compound-v3", 1700000000).await?;
    /// if let Some(outflows) = flows.outflows {
    ///     println!("Net flows: ${:.2}M", outflows / 1_000_000.0);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn inflows(&self, protocol: &str, timestamp: u64) -> Result<ProtocolInflows> {
        let path = format!("/inflows/{}/{}", protocol, timestamp);
        self.client.get_pro(&path).await
    }

    /// Get asset breakdown for all chains (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// Returns canonical, native, and third-party asset breakdowns per chain.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> llama::error::Result<()> {
    /// let client = llama::Client::with_api_key("your-api-key")?;
    /// let assets = client.tvl().chain_assets().await?;
    /// for (chain, breakdown) in assets.0.iter().take(3) {
    ///     if let Some(canonical) = &breakdown.canonical {
    ///         println!("{}: {} canonical", chain, canonical.total);
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn chain_assets(&self) -> Result<ChainAssets> {
        self.client.get_pro("/chainAssets").await
    }
}
