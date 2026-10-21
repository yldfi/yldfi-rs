//! Ecosystem API endpoints (Pro)

use crate::client::Client;
use crate::error::Result;

use super::types::{
    CategoriesResponse, ChainAssets, Entity, ForksResponse, Hack, LiquidityData, OraclesResponse,
    ProtocolInflows, Raise, RaisesResponse, TokenProtocol, Treasury,
};

/// Ecosystem API client (mostly Pro)
pub struct EcosystemApi<'a> {
    client: &'a Client,
}

impl<'a> EcosystemApi<'a> {
    /// Create a new Ecosystem API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get protocols per category and daily TVL by category
    ///
    /// **Requires Pro API key**
    pub async fn categories(&self) -> Result<CategoriesResponse> {
        self.client.get_pro("/api/categories").await
    }

    /// Get protocol fork relationships and daily TVL of forks
    ///
    /// **Requires Pro API key**
    pub async fn forks(&self) -> Result<ForksResponse> {
        self.client.get_pro("/api/forks").await
    }

    /// Get protocols and chains per oracle and daily TVL secured
    ///
    /// **Requires Pro API key**
    pub async fn oracles(&self) -> Result<OraclesResponse> {
        self.client.get_pro("/api/oracles").await
    }

    /// Get entity/company information
    ///
    /// **Requires Pro API key**
    pub async fn entities(&self) -> Result<Vec<Entity>> {
        self.client.get_pro("/api/entities").await
    }

    /// Get protocol treasury balances
    ///
    /// **Requires Pro API key**
    pub async fn treasuries(&self) -> Result<Vec<Treasury>> {
        self.client.get_pro("/api/treasuries").await
    }

    /// Get historical exploits database
    ///
    /// **Requires Pro API key**
    pub async fn hacks(&self) -> Result<Vec<Hack>> {
        self.client.get_pro("/api/hacks").await
    }

    /// Get funding rounds database
    ///
    /// **Requires Pro API key**
    pub async fn raises(&self) -> Result<Vec<Raise>> {
        let resp: RaisesResponse = self.client.get_pro("/api/raises").await?;
        Ok(resp.raises)
    }

    /// Get historical liquidity for a token
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `token` - Token symbol or address
    pub async fn liquidity(&self, token: &str) -> Result<LiquidityData> {
        let path = format!("/api/historicalLiquidity/{token}");
        self.client.get_pro(&path).await
    }

    /// Get protocols that hold a specific token
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `symbol` - Token symbol (e.g., "ETH", "USDC")
    pub async fn token_protocols(&self, symbol: &str) -> Result<Vec<TokenProtocol>> {
        let path = format!("/api/tokenProtocols/{symbol}");
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
        let path = format!("/api/inflows/{protocol}/{timestamp}");
        self.client.get_pro(&path).await
    }

    /// Get asset breakdown across all chains
    ///
    /// **Requires Pro API key**
    pub async fn chain_assets(&self) -> Result<ChainAssets> {
        self.client.get_pro("/api/chainAssets").await
    }
}
