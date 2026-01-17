//! Global and general API endpoints

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Global API
pub struct GlobalApi<'a> {
    client: &'a Client,
}

impl<'a> GlobalApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Check API status
    pub async fn ping(&self) -> Result<PingResponse> {
        self.client.get("/ping").await
    }

    /// Get global cryptocurrency data
    ///
    /// # Example
    /// ```no_run
    /// # async fn example() -> gecko::error::Result<()> {
    /// let client = gecko::Client::new()?;
    /// let global = client.global().data().await?;
    /// println!("Active cryptocurrencies: {:?}", global.data.active_cryptocurrencies);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn data(&self) -> Result<GlobalResponse> {
        self.client.get("/global").await
    }

    /// Get global DeFi data
    pub async fn defi(&self) -> Result<DefiGlobalResponse> {
        self.client.get("/global/decentralized_finance_defi").await
    }

    /// Get trending coins, NFTs, and categories
    pub async fn trending(&self) -> Result<TrendingResponse> {
        self.client.get("/search/trending").await
    }

    /// Search for coins, exchanges, categories, NFTs
    pub async fn search(&self, query: &str) -> Result<SearchResponse> {
        let path = format!("/search?query={}", query);
        self.client.get(&path).await
    }

    /// Get BTC exchange rates
    pub async fn exchange_rates(&self) -> Result<ExchangeRatesResponse> {
        self.client.get("/exchange_rates").await
    }

    /// Get asset platforms (blockchains)
    pub async fn asset_platforms(&self) -> Result<Vec<AssetPlatform>> {
        self.client.get("/asset_platforms").await
    }

    /// Get API key usage (Pro API only)
    pub async fn api_usage(&self) -> Result<ApiKeyUsage> {
        self.client.get("/key").await
    }

    /// Get global market cap chart (Pro API only)
    ///
    /// # Arguments
    /// * `days` - Data range (1, 7, 14, 30, 90, 180, 365, "max")
    pub async fn market_cap_chart(&self, days: &str) -> Result<MarketCapChart> {
        let path = format!("/global/market_cap_chart?days={}", days);
        self.client.get(&path).await
    }

    /// Get token list for a blockchain
    pub async fn token_list(&self, asset_platform_id: &str) -> Result<TokenList> {
        let path = format!("/token_lists/{}/all.json", asset_platform_id);
        self.client.get(&path).await
    }
}
