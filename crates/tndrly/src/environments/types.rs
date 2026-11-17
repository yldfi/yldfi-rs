//! Types for the Environments API

use crate::vnets::{
    ExplorerPageConfig, ForkConfigResponse, RpcEndpoint, SyncStateConfig,
    VirtualNetworkConfigResponse,
};
use serde::{Deserialize, Serialize};

/// A multi-chain environment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Environment {
    /// Environment ID
    #[serde(default)]
    pub id: String,

    /// Display name
    #[serde(default)]
    pub display_name: Option<String>,

    /// Slug
    #[serde(default)]
    pub slug: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Last update timestamp
    #[serde(default)]
    pub updated_at: Option<String>,

    /// The networks the environment spans
    #[serde(default)]
    pub networks: Vec<EnvironmentNetwork>,

    /// Cross-chain bridge relaying
    #[serde(default)]
    pub bridge_config: Option<BridgeConfig>,

    /// The running instance and its Virtual `TestNets`
    #[serde(default)]
    pub active_instance: Option<EnvironmentInstance>,
}

/// One network of an environment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnvironmentNetwork {
    /// Network entry ID
    #[serde(default)]
    pub id: Option<String>,

    /// Network (chain) ID, e.g. `"1"`
    #[serde(default)]
    pub network_id: Option<String>,

    /// Network name
    #[serde(default)]
    pub network_name: Option<String>,

    /// `vnet` or `network_mirror`
    #[serde(default)]
    pub network_type: Option<String>,

    /// Fork block number
    #[serde(default)]
    pub block_number: Option<String>,

    /// Region
    #[serde(default)]
    pub region: Option<String>,

    /// Chain config overrides
    #[serde(default)]
    pub chain_config_overrides: Option<ChainConfigOverrides>,

    /// Explorer settings
    #[serde(default)]
    pub explorer_config: Option<ExplorerConfig>,

    /// Pre-funded accounts
    #[serde(default)]
    pub accounts: Vec<EnvironmentAccount>,
}

/// An environment instance: the `VNets` currently backing the environment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnvironmentInstance {
    /// Instance ID
    #[serde(default)]
    pub id: String,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Last update timestamp
    #[serde(default)]
    pub updated_at: Option<String>,

    /// Cross-chain bridge relaying
    #[serde(default)]
    pub bridge_config: Option<BridgeConfig>,

    /// One Virtual `TestNet` per network
    #[serde(default)]
    pub vnets: Vec<EnvironmentVNet>,
}

/// A Virtual `TestNet` inside an environment instance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnvironmentVNet {
    /// `VNet` ID
    #[serde(default)]
    pub id: String,

    /// Display name
    #[serde(default)]
    pub display_name: Option<String>,

    /// Slug
    #[serde(default)]
    pub slug: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// `running`, `stopped`, `cloning` or `provision_failed`
    #[serde(default)]
    pub status: Option<String>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// RPC endpoints
    #[serde(default)]
    pub rpcs: Vec<RpcEndpoint>,

    /// Fork configuration
    #[serde(default)]
    pub fork_config: Option<ForkConfigResponse>,

    /// Virtual network configuration
    #[serde(default)]
    pub virtual_network_config: Option<VirtualNetworkConfigResponse>,

    /// RPC naming and persistence settings
    #[serde(default)]
    pub rpc_config: Option<serde_json::Value>,

    /// Runtime settings (e.g. `running_duration`)
    #[serde(default)]
    pub runtime_config: Option<serde_json::Value>,

    /// State sync settings
    #[serde(default)]
    pub sync_state_config: Option<SyncStateConfig>,

    /// Explorer page settings
    #[serde(default)]
    pub explorer_page_config: Option<ExplorerPageConfig>,
}

impl EnvironmentVNet {
    fn rpc(&self, kind: &str) -> Option<&str> {
        self.rpcs
            .iter()
            .find(|e| e.name.to_lowercase().contains(kind))
            .map(|e| e.url.as_str())
    }

    /// The public RPC URL
    #[must_use]
    pub fn public_rpc_url(&self) -> Option<&str> {
        self.rpc("public")
    }

    /// The admin RPC URL
    #[must_use]
    pub fn admin_rpc_url(&self) -> Option<&str> {
        self.rpc("admin")
    }
}

/// Cross-chain bridge relaying between an environment's networks
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BridgeConfig {
    /// Whether bridging is enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// `autoRelay` or `delayedRelay`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub relay_mode: Option<String>,

    /// Relay delay in seconds
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_seconds: Option<u64>,
}

impl BridgeConfig {
    /// Bridging enabled, relaying messages immediately
    #[must_use]
    pub fn auto_relay() -> Self {
        Self {
            enabled: Some(true),
            relay_mode: Some("autoRelay".to_string()),
            delay_seconds: None,
        }
    }

    /// Bridging enabled, relaying messages after `delay_seconds`
    #[must_use]
    pub fn delayed_relay(delay_seconds: u64) -> Self {
        Self {
            enabled: Some(true),
            relay_mode: Some("delayedRelay".to_string()),
            delay_seconds: Some(delay_seconds),
        }
    }
}

/// Chain config overrides for an environment network
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ChainConfigOverrides {
    /// Chain ID to expose instead of the forked network's
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<String>,
}

/// Explorer settings for an environment network
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ExplorerConfig {
    /// Whether the explorer is enabled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,

    /// `src`, `abi` or `bytecode`
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_visibility: Option<String>,
}

/// A pre-funded account on an environment network
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct EnvironmentAccount {
    /// Address
    #[serde(default)]
    pub address: String,

    /// Balance in wei, `0x` hex (requests only)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,
}

/// Request to create a multi-chain environment
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CreateEnvironmentRequest {
    /// Display name
    pub display_name: String,

    /// Slug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// One entry per network to fork
    pub network_configs: Vec<EnvironmentNetworkConfig>,

    /// Cross-chain bridge relaying
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bridge_config: Option<BridgeConfig>,
}

impl CreateEnvironmentRequest {
    /// Create a request; add at least one network with
    /// [`network`](Self::network)
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            display_name: display_name.into(),
            slug: None,
            network_configs: Vec::new(),
            bridge_config: None,
        }
    }

    /// Set the slug
    #[must_use]
    pub fn slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    /// Add a network
    #[must_use]
    pub fn network(mut self, config: EnvironmentNetworkConfig) -> Self {
        self.network_configs.push(config);
        self
    }

    /// Set cross-chain bridge relaying
    #[must_use]
    pub fn bridge_config(mut self, config: BridgeConfig) -> Self {
        self.bridge_config = Some(config);
        self
    }

    /// Check that at least one network is configured
    ///
    /// # Errors
    /// If no network was added.
    pub fn validate(&self) -> Result<(), String> {
        if self.network_configs.is_empty() {
            return Err(
                "An environment needs at least one network. Use .network() to add one.".to_string(),
            );
        }
        Ok(())
    }
}

/// One network of a [`CreateEnvironmentRequest`]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct EnvironmentNetworkConfig {
    /// Network (chain) ID to fork, e.g. `"1"`
    pub network_id: String,

    /// Block to fork at (`0x` hex); latest when unset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<String>,

    /// Chain config overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_config_overrides: Option<ChainConfigOverrides>,

    /// Explorer settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_config: Option<ExplorerConfig>,

    /// Accounts to pre-fund
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub accounts: Vec<EnvironmentAccount>,
}

impl EnvironmentNetworkConfig {
    /// Fork this network at its latest block
    pub fn new(network_id: impl Into<String>) -> Self {
        Self {
            network_id: network_id.into(),
            block_number: None,
            chain_config_overrides: None,
            explorer_config: None,
            accounts: Vec::new(),
        }
    }

    /// Fork at this block (decimal or `0x` hex; sent as hex)
    #[must_use]
    pub fn block_number(mut self, block: impl AsRef<str>) -> Self {
        self.block_number = Some(crate::utils::to_hex_quantity(block.as_ref()));
        self
    }

    /// Expose a different chain ID
    #[must_use]
    pub fn chain_id(mut self, chain_id: impl Into<String>) -> Self {
        self.chain_config_overrides = Some(ChainConfigOverrides {
            chain_id: Some(chain_id.into()),
        });
        self
    }

    /// Enable the explorer with this verification visibility
    /// (`src`, `abi` or `bytecode`)
    #[must_use]
    pub fn explorer(mut self, verification_visibility: impl Into<String>) -> Self {
        self.explorer_config = Some(ExplorerConfig {
            enabled: Some(true),
            verification_visibility: Some(verification_visibility.into()),
        });
        self
    }

    /// Pre-fund an account (balance in wei, decimal or `0x` hex; sent as hex)
    #[must_use]
    pub fn account(mut self, address: impl Into<String>, balance: impl AsRef<str>) -> Self {
        self.accounts.push(EnvironmentAccount {
            address: address.into(),
            balance: Some(crate::utils::to_hex_quantity(balance.as_ref())),
        });
        self
    }
}

/// Pagination for listing environments
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct ListEnvironmentsQuery {
    /// Page number (from 1)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

impl ListEnvironmentsQuery {
    /// Create an empty query
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set results per page
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }
}

/// Filters for listing an environment instance's transactions
///
/// Every list filter matches any of its values; each value is sent as its
/// own query parameter (`status=success&status=failed`).
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ListEnvironmentTransactionsQuery {
    /// Only these `VNets` of the instance
    pub vnet_ids: Vec<String>,
    /// `success`, `failed`, `pending` or `unknown`
    pub status: Vec<String>,
    /// `write`, `read`, `compute`, `subscribe`, `config` or `chain_sync`
    pub category: Vec<String>,
    /// e.g. `blockchain`, `simulation`, `cheatcode`
    pub kind: Vec<String>,
    /// Sender addresses
    pub from: Vec<String>,
    /// Recipient addresses
    pub to: Vec<String>,
    /// Page number (from 1)
    pub page: Option<u32>,
    /// Results per page (at most 100)
    pub per_page: Option<u32>,
}

impl ListEnvironmentTransactionsQuery {
    /// Create an empty query
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Only this `VNet`
    #[must_use]
    pub fn vnet_id(mut self, id: impl Into<String>) -> Self {
        self.vnet_ids.push(id.into());
        self
    }

    /// Only this status
    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status.push(status.into());
        self
    }

    /// Only this category
    #[must_use]
    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category.push(category.into());
        self
    }

    /// Only this kind
    #[must_use]
    pub fn kind(mut self, kind: impl Into<String>) -> Self {
        self.kind.push(kind.into());
        self
    }

    /// Only from this address
    #[must_use]
    pub fn from(mut self, address: impl Into<String>) -> Self {
        self.from.push(address.into());
        self
    }

    /// Only to this address
    #[must_use]
    pub fn to(mut self, address: impl Into<String>) -> Self {
        self.to.push(address.into());
        self
    }

    /// Set the page number
    #[must_use]
    pub fn page(mut self, page: u32) -> Self {
        self.page = Some(page);
        self
    }

    /// Set results per page
    #[must_use]
    pub fn per_page(mut self, per_page: u32) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// The query as repeated `key=value` pairs
    pub(crate) fn to_pairs(&self) -> Vec<(&'static str, String)> {
        let lists: [(&'static str, &Vec<String>); 6] = [
            ("vnet_ids", &self.vnet_ids),
            ("status", &self.status),
            ("category", &self.category),
            ("kind", &self.kind),
            ("from", &self.from),
            ("to", &self.to),
        ];
        let mut pairs: Vec<(&'static str, String)> = lists
            .iter()
            .flat_map(|(key, values)| values.iter().map(move |v| (*key, v.clone())))
            .collect();
        pairs.extend(self.page.map(|p| ("page", p.to_string())));
        pairs.extend(self.per_page.map(|p| ("per_page", p.to_string())));
        pairs
    }
}

/// Result of simulating a transaction on an environment `VNet`
///
/// The decoded parts (logs, trace, asset/balance/exposure/state changes) are
/// left as JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EnvironmentSimulation {
    /// Block number
    #[serde(default)]
    pub block_number: Option<String>,

    /// Cumulative gas used in the block
    #[serde(default)]
    pub cumulative_gas_used: Option<String>,

    /// Gas used by the transaction
    #[serde(default)]
    pub gas_used: Option<String>,

    /// `true` if the transaction succeeded
    #[serde(default)]
    pub status: Option<bool>,

    /// Transaction type
    #[serde(default, rename = "type")]
    pub tx_type: Option<String>,

    /// Logs bloom
    #[serde(default)]
    pub logs_bloom: Option<String>,

    /// Metadata (e.g. the simulation `id`)
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,

    /// Decoded logs
    #[serde(default)]
    pub logs: Option<Vec<serde_json::Value>>,

    /// Call trace
    #[serde(default)]
    pub trace: Option<Vec<serde_json::Value>>,

    /// Asset transfers
    #[serde(default)]
    pub asset_changes: Option<Vec<serde_json::Value>>,

    /// Balance changes
    #[serde(default)]
    pub balance_changes: Option<Vec<serde_json::Value>>,

    /// Approval (exposure) changes
    #[serde(default)]
    pub exposure_changes: Option<Vec<serde_json::Value>>,

    /// Storage, balance and nonce changes
    #[serde(default)]
    pub state_changes: Option<Vec<serde_json::Value>>,

    /// Cross-chain interop information
    #[serde(default)]
    pub interop_info: Option<serde_json::Value>,
}
