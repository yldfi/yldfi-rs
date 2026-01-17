//! Ecosystem API endpoints (Pro)

use crate::client::Client;
use crate::error::Result;

use super::types::*;

/// Ecosystem API client (mostly Pro)
pub struct EcosystemApi<'a> {
    client: &'a Client,
}

impl<'a> EcosystemApi<'a> {
    /// Create a new Ecosystem API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get TVL by category
    ///
    /// **Requires Pro API key**
    pub async fn categories(&self) -> Result<Vec<Category>> {
        self.client.get_pro("/categories").await
    }

    /// Get protocol fork relationships
    ///
    /// **Requires Pro API key**
    pub async fn forks(&self) -> Result<Vec<Fork>> {
        self.client.get_pro("/forks").await
    }

    /// Get oracle protocol data
    ///
    /// **Requires Pro API key**
    pub async fn oracles(&self) -> Result<Vec<Oracle>> {
        self.client.get_pro("/oracles").await
    }

    /// Get entity/company information
    ///
    /// **Requires Pro API key**
    pub async fn entities(&self) -> Result<Vec<Entity>> {
        self.client.get_pro("/entities").await
    }

    /// Get protocol treasury balances
    ///
    /// **Requires Pro API key**
    pub async fn treasuries(&self) -> Result<Vec<Treasury>> {
        self.client.get_pro("/treasuries").await
    }

    /// Get historical exploits database
    ///
    /// **Requires Pro API key**
    pub async fn hacks(&self) -> Result<Vec<Hack>> {
        self.client.get_pro("/hacks").await
    }

    /// Get funding rounds database
    ///
    /// **Requires Pro API key**
    pub async fn raises(&self) -> Result<Vec<Raise>> {
        self.client.get_pro("/raises").await
    }

    /// Get historical liquidity for a token
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `token` - Token symbol or address
    pub async fn liquidity(&self, token: &str) -> Result<LiquidityData> {
        let path = format!("/historicalLiquidity/{}", token);
        self.client.get_pro(&path).await
    }

    /// Get protocols that hold a specific token
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `symbol` - Token symbol (e.g., "ETH", "USDC")
    pub async fn token_protocols(&self, symbol: &str) -> Result<TokenProtocols> {
        let path = format!("/tokenProtocols/{}", symbol);
        self.client.get_pro(&path).await
    }

    /// Get daily capital flows for a protocol
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol slug
    /// * `timestamp` - Unix timestamp
    pub async fn inflows(&self, protocol: &str, timestamp: u64) -> Result<ProtocolInflows> {
        let path = format!("/inflows/{}/{}", protocol, timestamp);
        self.client.get_pro(&path).await
    }

    /// Get asset breakdown across all chains
    ///
    /// **Requires Pro API key**
    pub async fn chain_assets(&self) -> Result<Vec<ChainAssets>> {
        self.client.get_pro("/chainAssets").await
    }
}
