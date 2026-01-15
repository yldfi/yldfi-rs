//! Types for Networks API

use serde::{Deserialize, Serialize};

/// Network slugs for different Tenderly services
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkSlugs {
    /// Slug for the block explorer
    #[serde(default)]
    pub explorer_slug: String,

    /// Slug for Node RPC service
    #[serde(default)]
    pub node_rpc_slug: String,

    /// Slug for Virtual TestNet RPC service
    #[serde(default)]
    pub vnet_rpc_slug: String,
}

/// Supported features for a network
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupportedFeatures {
    /// Whether Virtual TestNets are supported
    #[serde(default)]
    pub virtual_testnet: bool,

    /// Whether Node RPC is supported
    #[serde(default)]
    pub node: bool,

    /// Whether block explorer is supported
    #[serde(default)]
    pub explorer: bool,

    /// Whether transaction simulation is supported
    #[serde(default)]
    pub simulator: bool,

    /// Whether transaction monitoring is supported
    #[serde(default)]
    pub monitoring: bool,
}

/// Supported network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Network {
    /// Network name (e.g., "Mainnet", "Arbitrum One")
    pub network_name: String,

    /// Chain ID as string (e.g., "1", "42161")
    pub chain_id: String,

    /// Network slugs for different services
    #[serde(default)]
    pub network_slugs: NetworkSlugs,

    /// Supported features on this network
    #[serde(default)]
    pub supported_features: SupportedFeatures,
}

impl Network {
    /// Get the chain ID as u64
    pub fn chain_id_u64(&self) -> Option<u64> {
        self.chain_id.parse().ok()
    }

    /// Check if this is a testnet (heuristic based on name)
    pub fn is_testnet(&self) -> bool {
        let name_lower = self.network_name.to_lowercase();
        name_lower.contains("testnet")
            || name_lower.contains("sepolia")
            || name_lower.contains("hoodi")
            || name_lower.contains("fuji")
            || name_lower.contains("amoy")
    }

    /// Check if simulations are supported
    pub fn simulation_supported(&self) -> bool {
        self.supported_features.simulator
    }

    /// Check if Virtual TestNets are supported
    pub fn vnet_supported(&self) -> bool {
        self.supported_features.virtual_testnet
    }

    /// Check if Node RPC is supported
    pub fn node_supported(&self) -> bool {
        self.supported_features.node
    }

    /// Check if monitoring is supported
    pub fn monitoring_supported(&self) -> bool {
        self.supported_features.monitoring
    }

    /// Get the primary slug (explorer slug)
    pub fn slug(&self) -> &str {
        &self.network_slugs.explorer_slug
    }

    /// Get the VNet RPC slug
    pub fn vnet_slug(&self) -> &str {
        &self.network_slugs.vnet_rpc_slug
    }
}

/// Type alias for the API response (array of networks)
pub type SupportedNetworksResponse = Vec<Network>;
