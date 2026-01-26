//! Unified Uniswap client combining on-chain and subgraph queries

use alloy::primitives::Address;

use crate::error::Result;
use crate::lens::LensClient;
use crate::subgraph::{SubgraphClient, SubgraphConfig};
use crate::types::{PoolData, PoolDayData, PoolState, Swap};

/// Configuration for the unified Uniswap client
#[derive(Debug, Clone)]
pub struct Config {
    /// RPC endpoint URL (required for on-chain queries)
    pub rpc_url: String,
    /// The Graph API key (optional - enables historical queries)
    pub subgraph_api_key: Option<String>,
    /// Subgraph ID (defaults to mainnet V3)
    pub subgraph_id: Option<String>,
    /// Factory address (defaults to mainnet)
    pub factory: Option<Address>,
}

impl Config {
    /// Create a new config with just an RPC URL (on-chain only)
    pub fn new(rpc_url: impl Into<String>) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            subgraph_api_key: None,
            subgraph_id: None,
            factory: None,
        }
    }

    /// Add subgraph API key to enable historical queries
    #[must_use]
    pub fn with_subgraph_key(mut self, api_key: impl Into<String>) -> Self {
        self.subgraph_api_key = Some(api_key.into());
        self
    }

    /// Set a custom subgraph ID
    #[must_use]
    pub fn with_subgraph_id(mut self, id: impl Into<String>) -> Self {
        self.subgraph_id = Some(id.into());
        self
    }

    /// Set a custom factory address
    #[must_use]
    pub fn with_factory(mut self, factory: Address) -> Self {
        self.factory = Some(factory);
        self
    }
}

/// Unified Uniswap client for both on-chain and historical data
#[derive(Debug)]
pub struct Client {
    lens: LensClient,
    subgraph: Option<SubgraphClient>,
}

impl Client {
    /// Create a new Uniswap client
    pub fn new(config: Config) -> Result<Self> {
        let factory = config
            .factory
            .unwrap_or(crate::lens::factories::MAINNET);
        let lens = LensClient::new(&config.rpc_url, factory)?;

        let subgraph = if let Some(api_key) = config.subgraph_api_key {
            let mut sg_config = SubgraphConfig::mainnet_v3(&api_key);
            if let Some(id) = config.subgraph_id {
                sg_config = sg_config.with_subgraph_id(id);
            }
            Some(SubgraphClient::new(sg_config)?)
        } else {
            None
        };

        Ok(Self { lens, subgraph })
    }

    /// Create a mainnet client with just RPC (no subgraph)
    pub fn mainnet(rpc_url: &str) -> Result<Self> {
        Self::new(Config::new(rpc_url))
    }

    /// Create a mainnet client with subgraph support
    pub fn mainnet_with_subgraph(rpc_url: &str, api_key: &str) -> Result<Self> {
        Self::new(Config::new(rpc_url).with_subgraph_key(api_key))
    }

    // ========================================================================
    // On-chain queries (always available)
    // ========================================================================

    /// Get current pool state from on-chain
    pub async fn get_pool_state(&self, pool: Address) -> Result<PoolState> {
        self.lens.get_pool_state(pool).await
    }

    /// Get current pool liquidity from on-chain
    pub async fn get_liquidity(&self, pool: Address) -> Result<u128> {
        self.lens.get_liquidity(pool).await
    }

    /// Get token balance for an account
    pub async fn get_token_balance(&self, token: Address, account: Address) -> Result<alloy::primitives::U256> {
        self.lens.get_token_balance(token, account).await
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        self.lens.get_block_number().await
    }

    // ========================================================================
    // Historical queries (requires subgraph API key)
    // ========================================================================

    /// Check if subgraph queries are available
    pub fn has_subgraph(&self) -> bool {
        self.subgraph.is_some()
    }

    /// Get the subgraph client (if configured)
    pub fn subgraph(&self) -> Option<&SubgraphClient> {
        self.subgraph.as_ref()
    }

    /// Get current ETH price in USD (requires subgraph)
    pub async fn get_eth_price(&self) -> Result<f64> {
        self.require_subgraph()?.get_eth_price().await
    }

    /// Get top pools by TVL (requires subgraph)
    pub async fn get_top_pools(&self, limit: u32) -> Result<Vec<PoolData>> {
        self.require_subgraph()?.get_top_pools(limit).await
    }

    /// Get pool data by address (requires subgraph)
    pub async fn get_pool_data(&self, address: &str) -> Result<Option<PoolData>> {
        self.require_subgraph()?.get_pool(address).await
    }

    /// Get recent swaps for a pool (requires subgraph)
    pub async fn get_swaps(&self, pool: &str, limit: u32) -> Result<Vec<Swap>> {
        self.require_subgraph()?.get_swaps(pool, limit).await
    }

    /// Get daily pool data (requires subgraph)
    pub async fn get_pool_day_data(&self, pool: &str, days: u32) -> Result<Vec<PoolDayData>> {
        self.require_subgraph()?.get_pool_day_data(pool, days).await
    }

    /// Require subgraph to be configured, return error if not
    fn require_subgraph(&self) -> Result<&SubgraphClient> {
        self.subgraph
            .as_ref()
            .ok_or_else(crate::error::subgraph_key_required)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = Config::new("https://eth.llamarpc.com")
            .with_subgraph_key("test-key")
            .with_subgraph_id("custom-id");

        assert_eq!(config.rpc_url, "https://eth.llamarpc.com");
        assert_eq!(config.subgraph_api_key, Some("test-key".to_string()));
        assert_eq!(config.subgraph_id, Some("custom-id".to_string()));
    }
}
