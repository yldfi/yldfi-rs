//! Networks API operations

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Networks API client
pub struct NetworksApi<'a> {
    client: &'a Client,
}

impl<'a> NetworksApi<'a> {
    /// Create a new Networks API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get list of supported networks
    ///
    /// Returns all blockchain networks supported by Tenderly, including
    /// information about which features are available on each network.
    ///
    /// This endpoint does not require authentication.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let networks = client.networks().supported().await?;
    /// for network in networks {
    ///     println!("{}: {} (chain ID: {})", network.slug(), network.network_name, network.chain_id);
    ///     if network.simulation_supported() {
    ///         println!("  - Simulations supported");
    ///     }
    ///     if network.vnet_supported() {
    ///         println!("  - Virtual TestNets supported");
    ///     }
    /// }
    /// ```
    pub async fn supported(&self) -> Result<Vec<Network>> {
        self.client.get_global("/supported-networks").await
    }

    /// Get a specific network by chain ID
    ///
    /// Returns None if the network is not supported.
    pub async fn get(&self, chain_id: &str) -> Result<Option<Network>> {
        let networks = self.supported().await?;
        Ok(networks.into_iter().find(|n| n.chain_id == chain_id))
    }

    /// Get a specific network by chain ID (numeric)
    ///
    /// Returns None if the network is not supported.
    pub async fn get_by_chain_id(&self, chain_id: u64) -> Result<Option<Network>> {
        self.get(&chain_id.to_string()).await
    }

    /// Get a specific network by slug
    ///
    /// Returns None if the network is not found.
    pub async fn get_by_slug(&self, slug: &str) -> Result<Option<Network>> {
        let networks = self.supported().await?;
        Ok(networks
            .into_iter()
            .find(|n| n.network_slugs.explorer_slug == slug))
    }

    /// List only mainnet networks
    pub async fn mainnets(&self) -> Result<Vec<Network>> {
        let networks = self.supported().await?;
        Ok(networks.into_iter().filter(|n| !n.is_testnet()).collect())
    }

    /// List only testnet networks
    pub async fn testnets(&self) -> Result<Vec<Network>> {
        let networks = self.supported().await?;
        Ok(networks.into_iter().filter(|n| n.is_testnet()).collect())
    }

    /// List networks that support simulations
    pub async fn with_simulation_support(&self) -> Result<Vec<Network>> {
        let networks = self.supported().await?;
        Ok(networks
            .into_iter()
            .filter(|n| n.simulation_supported())
            .collect())
    }

    /// List networks that support Virtual TestNets
    pub async fn with_vnet_support(&self) -> Result<Vec<Network>> {
        let networks = self.supported().await?;
        Ok(networks
            .into_iter()
            .filter(|n| n.vnet_supported())
            .collect())
    }
}
