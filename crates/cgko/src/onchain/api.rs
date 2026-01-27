//! Onchain/GeckoTerminal API endpoints

use super::types::{
    DexesResponse, HoldersChartResponse, MegafilterOptions, NetworksResponse, OhlcvResponse,
    OnchainCategoriesResponse, PoolInfoResponse, PoolResponse, PoolsResponse, TokenHoldersResponse,
    TokenInfoResponse, TokenOhlcvResponse, TokenPriceResponse, TokenResponse, TokenTradersResponse,
    TokenTradesResponse, TokensResponse, TradesResponse,
};
use crate::client::Client;
use crate::error::Result;

/// Onchain (`GeckoTerminal`) API
pub struct OnchainApi<'a> {
    client: &'a Client,
}

impl<'a> OnchainApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List supported networks
    pub async fn networks(&self) -> Result<NetworksResponse> {
        self.client.get("/onchain/networks").await
    }

    /// List DEXes on a network
    pub async fn dexes(&self, network: &str) -> Result<DexesResponse> {
        let path = format!("/onchain/networks/{network}/dexes");
        self.client.get(&path).await
    }

    /// Get trending pools across all networks
    pub async fn trending_pools(&self) -> Result<PoolsResponse> {
        self.client.get("/onchain/networks/trending_pools").await
    }

    /// Get trending pools on a network
    pub async fn trending_pools_network(&self, network: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/trending_pools");
        self.client.get(&path).await
    }

    /// Get top pools on a network
    pub async fn top_pools(&self, network: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/pools");
        self.client.get(&path).await
    }

    /// Get pool data
    pub async fn pool(&self, network: &str, address: &str) -> Result<PoolResponse> {
        let path = format!("/onchain/networks/{network}/pools/{address}");
        self.client.get(&path).await
    }

    /// Get new pools on a network
    pub async fn new_pools(&self, network: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/new_pools");
        self.client.get(&path).await
    }

    /// Get token data
    pub async fn token(&self, network: &str, address: &str) -> Result<TokenResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{address}");
        self.client.get(&path).await
    }

    /// Get token price
    pub async fn token_price(
        &self,
        network: &str,
        addresses: &[&str],
    ) -> Result<TokenPriceResponse> {
        let path = format!(
            "/onchain/simple/networks/{}/token_price/{}",
            network,
            addresses.join(",")
        );
        self.client.get(&path).await
    }

    /// Get pools for a token
    pub async fn token_pools(&self, network: &str, token_address: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/pools");
        self.client.get(&path).await
    }

    /// Get pool OHLCV data
    ///
    /// # Arguments
    /// * `network` - Network ID
    /// * `pool_address` - Pool address
    /// * `timeframe` - "minute", "hour", or "day"
    pub async fn pool_ohlcv(
        &self,
        network: &str,
        pool_address: &str,
        timeframe: &str,
    ) -> Result<OhlcvResponse> {
        let path = format!("/onchain/networks/{network}/pools/{pool_address}/ohlcv/{timeframe}");
        self.client.get(&path).await
    }

    /// Get pool trades
    pub async fn pool_trades(&self, network: &str, pool_address: &str) -> Result<TradesResponse> {
        let path = format!("/onchain/networks/{network}/pools/{pool_address}/trades");
        self.client.get(&path).await
    }

    /// Search pools
    pub async fn search_pools(&self, query: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/search/pools?query={query}");
        self.client.get(&path).await
    }

    /// Get new pools across all networks
    pub async fn new_pools_all(&self) -> Result<PoolsResponse> {
        self.client.get("/onchain/networks/new_pools").await
    }

    /// Get top pools across all networks
    pub async fn top_pools_all(&self) -> Result<PoolsResponse> {
        self.client.get("/onchain/networks/pools").await
    }

    /// Get multiple pools by addresses
    pub async fn pools_multi(&self, network: &str, addresses: &[&str]) -> Result<PoolsResponse> {
        let path = format!(
            "/onchain/networks/{}/pools/multi/{}",
            network,
            addresses.join(",")
        );
        self.client.get(&path).await
    }

    /// Get multiple tokens
    pub async fn tokens_multi(&self, network: &str, addresses: &[&str]) -> Result<TokensResponse> {
        let path = format!(
            "/onchain/networks/{}/tokens/multi/{}",
            network,
            addresses.join(",")
        );
        self.client.get(&path).await
    }

    /// Get token info (detailed)
    pub async fn token_info(&self, network: &str, address: &str) -> Result<TokenInfoResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{address}/info");
        self.client.get(&path).await
    }

    /// Get pools for a specific DEX on a network
    pub async fn dex_pools(&self, network: &str, dex: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/networks/{network}/dexes/{dex}/pools");
        self.client.get(&path).await
    }

    /// Get most recently updated tokens across all networks
    pub async fn recently_updated_tokens(&self) -> Result<TokensResponse> {
        self.client
            .get("/onchain/tokens/info_recently_updated")
            .await
    }

    /// Get pool token metadata
    pub async fn pool_info(&self, network: &str, pool_address: &str) -> Result<PoolInfoResponse> {
        let path = format!("/onchain/networks/{network}/pools/{pool_address}/info");
        self.client.get(&path).await
    }

    /// Get top token holders (Pro API only)
    pub async fn token_holders(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<TokenHoldersResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/top_holders");
        self.client.get(&path).await
    }

    /// Get top token traders (Pro API only)
    pub async fn token_traders(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<TokenTradersResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/top_traders");
        self.client.get(&path).await
    }

    /// Get historical token holders chart (Pro API only)
    pub async fn token_holders_chart(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<HoldersChartResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/holders_chart");
        self.client.get(&path).await
    }

    /// Get token OHLCV data (Pro API only)
    ///
    /// # Arguments
    /// * `network` - Network ID
    /// * `token_address` - Token address
    /// * `timeframe` - "minute", "hour", or "day"
    pub async fn token_ohlcv(
        &self,
        network: &str,
        token_address: &str,
        timeframe: &str,
    ) -> Result<TokenOhlcvResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/ohlcv/{timeframe}");
        self.client.get(&path).await
    }

    /// Get token trades across all pools (Pro API only)
    pub async fn token_trades(
        &self,
        network: &str,
        token_address: &str,
    ) -> Result<TokenTradesResponse> {
        let path = format!("/onchain/networks/{network}/tokens/{token_address}/trades");
        self.client.get(&path).await
    }

    /// Advanced pool filtering with megafilter (Pro API only)
    pub async fn megafilter(&self, options: &MegafilterOptions) -> Result<PoolsResponse> {
        let path = format!("/onchain/pools/megafilter{}", options.to_query_string());
        self.client.get(&path).await
    }

    /// Get trending search pools (Pro API only)
    pub async fn trending_search_pools(&self) -> Result<PoolsResponse> {
        self.client.get("/onchain/pools/trending_search").await
    }

    /// Get `GeckoTerminal` categories (Pro API only)
    pub async fn categories(&self) -> Result<OnchainCategoriesResponse> {
        self.client.get("/onchain/categories").await
    }

    /// Get pools by category (Pro API only)
    pub async fn category_pools(&self, category_id: &str) -> Result<PoolsResponse> {
        let path = format!("/onchain/categories/{category_id}/pools");
        self.client.get(&path).await
    }
}
