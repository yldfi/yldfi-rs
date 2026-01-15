//! Types for Virtual TestNets API

use serde::{Deserialize, Serialize};

/// Request to create a new Virtual TestNet
#[derive(Debug, Clone, Serialize)]
pub struct CreateVNetRequest {
    /// Unique slug for the VNet
    pub slug: String,

    /// Display name
    pub display_name: String,

    /// Fork configuration
    pub fork_config: ForkConfig,

    /// Virtual network configuration
    pub virtual_network_config: VirtualNetworkConfig,

    /// State sync configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_state_config: Option<SyncStateConfig>,

    /// Explorer page configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_page_config: Option<ExplorerPageConfig>,
}

impl CreateVNetRequest {
    /// Create a new VNet request with minimal configuration
    pub fn new(slug: impl Into<String>, display_name: impl Into<String>, network_id: u64) -> Self {
        Self {
            slug: slug.into(),
            display_name: display_name.into(),
            fork_config: ForkConfig {
                network_id,
                block_number: None,
            },
            virtual_network_config: VirtualNetworkConfig {
                chain_config: ChainConfig {
                    chain_id: network_id,
                },
                base_fee_per_gas: None,
            },
            sync_state_config: None,
            explorer_page_config: None,
        }
    }

    /// Fork from a specific block
    #[must_use]
    pub fn block_number(mut self, block: u64) -> Self {
        self.fork_config.block_number = Some(block);
        self
    }

    /// Set a custom chain ID
    #[must_use]
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.virtual_network_config.chain_config.chain_id = chain_id;
        self
    }

    /// Set base fee per gas
    #[must_use]
    pub fn base_fee_per_gas(mut self, fee: u64) -> Self {
        self.virtual_network_config.base_fee_per_gas = Some(fee);
        self
    }

    /// Enable state sync
    #[must_use]
    pub fn sync_state(mut self, enabled: bool) -> Self {
        self.sync_state_config = Some(SyncStateConfig { enabled });
        self
    }

    /// Enable explorer page
    #[must_use]
    pub fn explorer_page(mut self, enabled: bool, verification_visibility: &str) -> Self {
        self.explorer_page_config = Some(ExplorerPageConfig {
            enabled,
            verification_visibility: verification_visibility.to_string(),
        });
        self
    }
}

/// Fork configuration for requests
#[derive(Debug, Clone, Serialize)]
pub struct ForkConfig {
    /// Network ID to fork from
    pub network_id: u64,

    /// Block number to fork from (None = latest)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
}

/// Fork configuration from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkConfigResponse {
    /// Network ID
    pub network_id: u64,

    /// Block number as hex string (e.g., "0x170abab")
    #[serde(default)]
    pub block_number: Option<String>,
}

/// Virtual network configuration for requests
#[derive(Debug, Clone, Serialize)]
pub struct VirtualNetworkConfig {
    /// Chain configuration (required by Tenderly API)
    pub chain_config: ChainConfig,

    /// Base fee per gas (for EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_fee_per_gas: Option<u64>,
}

/// Virtual network configuration from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualNetworkConfigResponse {
    /// Nested chain configuration
    #[serde(default)]
    pub chain_config: Option<ChainConfig>,

    /// Base fee per gas
    #[serde(default)]
    pub base_fee_per_gas: Option<u64>,

    /// Pre-funded accounts
    #[serde(default)]
    pub accounts: Option<Vec<serde_json::Value>>,
}

impl VirtualNetworkConfigResponse {
    /// Get the chain ID from nested chain_config
    #[must_use]
    pub fn chain_id(&self) -> Option<u64> {
        self.chain_config.as_ref().map(|c| c.chain_id)
    }
}

/// Chain configuration nested in VirtualNetworkConfigResponse
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Chain ID
    pub chain_id: u64,
}

/// State sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStateConfig {
    /// Whether to sync state from the parent network
    pub enabled: bool,
}

/// Explorer page configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerPageConfig {
    /// Whether explorer page is enabled
    pub enabled: bool,

    /// Verification visibility setting
    pub verification_visibility: String,
}

/// Virtual TestNet details from API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VNet {
    /// VNet ID
    pub id: String,

    /// Slug
    pub slug: String,

    /// Display name
    pub display_name: String,

    /// Fork configuration
    pub fork_config: ForkConfigResponse,

    /// Virtual network configuration
    pub virtual_network_config: VirtualNetworkConfigResponse,

    /// RPC endpoints (array of {name, url} objects)
    #[serde(default, deserialize_with = "deserialize_rpcs")]
    pub rpcs: Option<VNetRpcs>,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Status
    #[serde(default)]
    pub status: Option<String>,
}

fn deserialize_rpcs<'de, D>(deserializer: D) -> std::result::Result<Option<VNetRpcs>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let endpoints: Option<Vec<RpcEndpoint>> = Option::deserialize(deserializer)?;
    Ok(endpoints.map(|e| VNetRpcs { endpoints: e }))
}

/// Single RPC endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpoint {
    /// Endpoint name (e.g., "Admin RPC", "Public RPC")
    pub name: String,

    /// RPC URL
    pub url: String,
}

/// Collection of RPC endpoints for a VNet
#[derive(Debug, Clone, Default, Serialize)]
pub struct VNetRpcs {
    /// All RPC endpoints
    pub endpoints: Vec<RpcEndpoint>,
}

impl VNetRpcs {
    /// Get the public RPC URL
    #[must_use]
    pub fn public(&self) -> Option<&str> {
        self.endpoints
            .iter()
            .find(|e| e.name.to_lowercase().contains("public"))
            .map(|e| e.url.as_str())
    }

    /// Get the admin RPC URL
    #[must_use]
    pub fn admin(&self) -> Option<&str> {
        self.endpoints
            .iter()
            .find(|e| e.name.to_lowercase().contains("admin"))
            .map(|e| e.url.as_str())
    }
}

/// Response when creating a VNet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVNetResponse {
    /// The created VNet
    #[serde(flatten)]
    pub vnet: VNet,
}

// Note: The VNets API returns a raw array, so list() returns Vec<VNet> directly.
// No wrapper type needed.

/// Query parameters for listing VNets
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListVNetsQuery {
    /// Filter by slug (partial match)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Page number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

impl ListVNetsQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by slug
    #[must_use]
    pub fn slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    /// Set page number
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

/// Request to delete multiple VNets
#[derive(Debug, Clone, Serialize)]
pub struct DeleteVNetsRequest {
    /// List of VNet IDs to delete
    pub vnet_ids: Vec<String>,
}

impl DeleteVNetsRequest {
    /// Create a delete request for a single VNet
    pub fn single(id: impl Into<String>) -> Self {
        Self {
            vnet_ids: vec![id.into()],
        }
    }

    /// Create a delete request for multiple VNets
    pub fn multiple(ids: Vec<String>) -> Self {
        Self { vnet_ids: ids }
    }
}

/// Request to fork a VNet
#[derive(Debug, Clone, Serialize)]
pub struct ForkVNetRequest {
    /// ID of the source VNet to fork from
    #[serde(rename = "vnet_id")]
    pub source_vnet_id: String,

    /// Slug for the new forked VNet
    pub slug: String,

    /// Display name for the forked VNet
    pub display_name: String,

    /// Block number to fork from (on the source VNet)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_number: Option<u64>,
}

impl ForkVNetRequest {
    /// Create a fork request
    pub fn new(
        source_vnet_id: impl Into<String>,
        slug: impl Into<String>,
        display_name: impl Into<String>,
    ) -> Self {
        Self {
            source_vnet_id: source_vnet_id.into(),
            slug: slug.into(),
            display_name: display_name.into(),
            block_number: None,
        }
    }

    /// Fork from a specific block
    #[must_use]
    pub fn block_number(mut self, block: u64) -> Self {
        self.block_number = Some(block);
        self
    }
}

/// Transaction on a VNet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VNetTransaction {
    /// Unique transaction ID
    #[serde(default)]
    pub id: Option<String>,

    /// VNet ID this transaction belongs to
    #[serde(default)]
    pub vnet_id: Option<String>,

    /// Transaction hash (may be None for fixture/admin operations)
    #[serde(default, alias = "hash")]
    pub tx_hash: Option<String>,

    /// Block number (hex string, e.g., "0x123abc")
    #[serde(default)]
    pub block_number: Option<String>,

    /// Block hash
    #[serde(default)]
    pub block_hash: Option<String>,

    /// From address
    #[serde(default)]
    pub from: Option<String>,

    /// To address
    #[serde(default)]
    pub to: Option<String>,

    /// Value (hex string)
    #[serde(default)]
    pub value: Option<String>,

    /// Gas limit (hex string)
    #[serde(default)]
    pub gas: Option<String>,

    /// Gas used (hex string)
    #[serde(default)]
    pub gas_used: Option<String>,

    /// Gas price (hex string)
    #[serde(default)]
    pub gas_price: Option<String>,

    /// Transaction status ("success", "failed")
    #[serde(default)]
    pub status: Option<String>,

    /// Transaction input data
    #[serde(default)]
    pub input: Option<String>,

    /// Transaction nonce (hex string)
    #[serde(default)]
    pub nonce: Option<String>,

    /// Transaction index in block (hex string)
    #[serde(default, alias = "transaction_index")]
    pub tx_index: Option<String>,

    /// Transaction type (hex string, e.g., "0x0" for legacy, "0x2" for EIP-1559)
    #[serde(default, rename = "type")]
    pub tx_type: Option<String>,

    /// Max priority fee per gas (EIP-1559, hex string)
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<String>,

    /// Max fee per gas (EIP-1559, hex string)
    #[serde(default)]
    pub max_fee_per_gas: Option<String>,

    /// Transaction origin (e.g., "rpc", "internal")
    #[serde(default)]
    pub origin: Option<String>,

    /// Transaction kind (e.g., "blockchain", "fixture")
    #[serde(default)]
    pub kind: Option<String>,

    /// RPC method used (e.g., "eth_sendRawTransaction")
    #[serde(default)]
    pub rpc_method: Option<String>,

    /// State overrides applied
    #[serde(default)]
    pub state_overrides: Option<serde_json::Value>,

    /// Block overrides applied
    #[serde(default)]
    pub block_overrides: Option<serde_json::Value>,

    /// Transaction category
    #[serde(default)]
    pub category: Option<String>,

    /// Function name if decoded
    #[serde(default)]
    pub function_name: Option<String>,

    /// Contract address if this was a contract creation
    #[serde(default)]
    pub contract_address: Option<String>,

    /// Dashboard URL for viewing transaction details
    #[serde(default)]
    pub dashboard_url: Option<String>,

    /// Creation timestamp (ISO 8601)
    #[serde(default)]
    pub created_at: Option<String>,

    /// Timestamp (may be block timestamp)
    #[serde(default)]
    pub timestamp: Option<String>,
}

impl VNetTransaction {
    /// Parse block number from hex string to u64
    #[must_use]
    pub fn block_number_as_u64(&self) -> Option<u64> {
        self.block_number.as_ref().and_then(|s| parse_hex_u64(s))
    }

    /// Parse gas from hex string to u64
    #[must_use]
    pub fn gas_as_u64(&self) -> Option<u64> {
        self.gas.as_ref().and_then(|s| parse_hex_u64(s))
    }

    /// Parse gas_used from hex string to u64
    #[must_use]
    pub fn gas_used_as_u64(&self) -> Option<u64> {
        self.gas_used.as_ref().and_then(|s| parse_hex_u64(s))
    }

    /// Parse nonce from hex string to u64
    #[must_use]
    pub fn nonce_as_u64(&self) -> Option<u64> {
        self.nonce.as_ref().and_then(|s| parse_hex_u64(s))
    }

    /// Check if transaction succeeded
    #[must_use]
    pub fn is_success(&self) -> bool {
        self.status.as_ref().is_some_and(|s| s == "success")
    }

    /// Check if transaction failed
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.status.as_ref().is_some_and(|s| s == "failed")
    }
}

/// Parse a hex string (with or without 0x prefix) to u64
fn parse_hex_u64(s: &str) -> Option<u64> {
    let s = s.strip_prefix("0x").unwrap_or(s);
    u64::from_str_radix(s, 16).ok()
}

/// Query parameters for listing VNet transactions
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListVNetTransactionsQuery {
    /// Filter by address (sender or recipient)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,

    /// Filter by status ("success" or "failed")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,

    /// Page number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

impl ListVNetTransactionsQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by address
    #[must_use]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    /// Filter by status ("success" or "failed")
    #[must_use]
    pub fn status(mut self, status: impl Into<String>) -> Self {
        self.status = Some(status.into());
        self
    }

    /// Filter for successful transactions
    #[must_use]
    pub fn success(mut self) -> Self {
        self.status = Some("success".to_string());
        self
    }

    /// Filter for failed transactions
    #[must_use]
    pub fn failed(mut self) -> Self {
        self.status = Some("failed".to_string());
        self
    }

    /// Set page number
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

/// Request to simulate a transaction on a VNet
#[derive(Debug, Clone, Serialize)]
pub struct VNetSimulationRequest {
    /// Sender address
    pub from: String,

    /// Recipient address
    pub to: String,

    /// Calldata
    pub input: String,

    /// Value in wei
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<u64>,

    /// Gas price (legacy transactions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,

    /// Max fee per gas in wei (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,

    /// Max priority fee per gas in wei (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,

    /// Transaction type (0 = legacy, 1 = access list, 2 = EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub transaction_type: Option<u8>,

    /// Nonce
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,
}

impl VNetSimulationRequest {
    /// Create a new simulation request
    pub fn new(from: impl Into<String>, to: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            input: input.into(),
            value: None,
            gas: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            transaction_type: None,
            nonce: None,
        }
    }

    /// Set value in wei
    #[must_use]
    pub fn value(mut self, wei: impl Into<String>) -> Self {
        self.value = Some(wei.into());
        self
    }

    /// Set gas limit
    #[must_use]
    pub fn gas(mut self, gas: u64) -> Self {
        self.gas = Some(gas);
        self
    }

    /// Set max fee per gas (EIP-1559)
    ///
    /// Automatically sets transaction type to 2.
    #[must_use]
    pub fn max_fee_per_gas(mut self, fee: impl Into<String>) -> Self {
        self.max_fee_per_gas = Some(fee.into());
        self.transaction_type = Some(2);
        self
    }

    /// Set max priority fee per gas (EIP-1559)
    ///
    /// Automatically sets transaction type to 2.
    #[must_use]
    pub fn max_priority_fee_per_gas(mut self, fee: impl Into<String>) -> Self {
        self.max_priority_fee_per_gas = Some(fee.into());
        self.transaction_type = Some(2);
        self
    }

    /// Set transaction type explicitly
    #[must_use]
    pub fn transaction_type(mut self, tx_type: u8) -> Self {
        self.transaction_type = Some(tx_type);
        self
    }

    /// Set nonce
    #[must_use]
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }
}

/// Request to update a Virtual TestNet
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateVNetRequest {
    /// New display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// New slug
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Sync state configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync_state_config: Option<SyncStateConfig>,

    /// Explorer page configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_page_config: Option<ExplorerPageConfig>,
}

impl UpdateVNetRequest {
    /// Create a new update request
    pub fn new() -> Self {
        Self::default()
    }

    /// Set display name
    #[must_use]
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set slug
    #[must_use]
    pub fn slug(mut self, slug: impl Into<String>) -> Self {
        self.slug = Some(slug.into());
        self
    }

    /// Set sync state enabled
    #[must_use]
    pub fn sync_state(mut self, enabled: bool) -> Self {
        self.sync_state_config = Some(SyncStateConfig { enabled });
        self
    }

    /// Set explorer page configuration
    #[must_use]
    pub fn explorer_page(mut self, enabled: bool, verification_visibility: &str) -> Self {
        self.explorer_page_config = Some(ExplorerPageConfig {
            enabled,
            verification_visibility: verification_visibility.to_string(),
        });
        self
    }
}

/// Request to send a transaction on a Virtual TestNet
#[derive(Debug, Clone, Serialize)]
pub struct SendVNetTransactionRequest {
    /// Sender address
    pub from: String,

    /// Recipient address (can be empty for contract creation)
    pub to: String,

    /// Calldata / input
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input: Option<String>,

    /// Value in wei
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Gas limit
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas: Option<u64>,

    /// Gas price in wei (legacy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_price: Option<String>,

    /// Max fee per gas (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,

    /// Max priority fee per gas (EIP-1559)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,

    /// Access list (EIP-2930)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_list: Option<Vec<AccessListItem>>,
}

/// Access list item for EIP-2930 transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessListItem {
    /// Address being accessed
    pub address: String,

    /// Storage keys being accessed
    #[serde(default)]
    pub storage_keys: Vec<String>,
}

impl SendVNetTransactionRequest {
    /// Create a new send transaction request
    pub fn new(from: impl Into<String>, to: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            input: Some(input.into()),
            value: None,
            gas: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: None,
        }
    }

    /// Create a simple ETH transfer
    pub fn transfer(
        from: impl Into<String>,
        to: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            input: None,
            value: Some(value.into()),
            gas: None,
            gas_price: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            access_list: None,
        }
    }

    /// Set value in wei
    #[must_use]
    pub fn value(mut self, wei: impl Into<String>) -> Self {
        self.value = Some(wei.into());
        self
    }

    /// Set gas limit
    #[must_use]
    pub fn gas(mut self, gas: u64) -> Self {
        self.gas = Some(gas);
        self
    }

    /// Set gas price (legacy)
    #[must_use]
    pub fn gas_price(mut self, price: impl Into<String>) -> Self {
        self.gas_price = Some(price.into());
        self
    }

    /// Set max fee per gas (EIP-1559)
    #[must_use]
    pub fn max_fee_per_gas(mut self, fee: impl Into<String>) -> Self {
        self.max_fee_per_gas = Some(fee.into());
        self
    }

    /// Set max priority fee per gas (EIP-1559)
    #[must_use]
    pub fn max_priority_fee_per_gas(mut self, fee: impl Into<String>) -> Self {
        self.max_priority_fee_per_gas = Some(fee.into());
        self
    }

    /// Set access list (EIP-2930)
    #[must_use]
    pub fn access_list(mut self, list: Vec<AccessListItem>) -> Self {
        self.access_list = Some(list);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vnet_transaction_deserialization() {
        // Example response from Tenderly API (based on issue #12)
        let json = r#"{
            "id": "tx-123",
            "vnet_id": "vnet-456",
            "tx_hash": "0xabc123def456",
            "block_number": "0x170abab",
            "block_hash": "0xblockhash123",
            "from": "0x1234567890abcdef1234567890abcdef12345678",
            "to": "0xabcdef1234567890abcdef1234567890abcdef12",
            "value": "0xde0b6b3a7640000",
            "gas": "0x5208",
            "gas_used": "0x5208",
            "gas_price": "0x3b9aca00",
            "status": "success",
            "input": "0x",
            "nonce": "0x1",
            "transaction_index": "0x0",
            "origin": "external",
            "category": "transfer",
            "function_name": null,
            "contract_address": null,
            "dashboard_url": "https://dashboard.tenderly.co/...",
            "created_at": "2024-01-15T10:30:00Z",
            "timestamp": "2024-01-15T10:30:00Z"
        }"#;

        let tx: VNetTransaction = serde_json::from_str(json).unwrap();

        assert_eq!(tx.id.as_deref(), Some("tx-123"));
        assert_eq!(tx.vnet_id.as_deref(), Some("vnet-456"));
        assert_eq!(tx.tx_hash.as_deref(), Some("0xabc123def456"));
        assert_eq!(tx.block_number.as_deref(), Some("0x170abab"));
        assert_eq!(tx.status.as_deref(), Some("success"));
        assert!(tx.is_success());
        assert!(!tx.is_failed());
    }

    #[test]
    fn test_vnet_transaction_failed_status() {
        let json = r#"{
            "tx_hash": "0xfailed123",
            "status": "failed"
        }"#;

        let tx: VNetTransaction = serde_json::from_str(json).unwrap();

        assert!(tx.is_failed());
        assert!(!tx.is_success());
    }

    #[test]
    fn test_vnet_transaction_hex_parsing() {
        let json = r#"{
            "tx_hash": "0xtest",
            "block_number": "0x170abab",
            "gas": "0x5208",
            "gas_used": "0x4e20",
            "nonce": "0xa"
        }"#;

        let tx: VNetTransaction = serde_json::from_str(json).unwrap();

        assert_eq!(tx.block_number_as_u64(), Some(24_161_195)); // 0x170abab
        assert_eq!(tx.gas_as_u64(), Some(21_000));
        assert_eq!(tx.gas_used_as_u64(), Some(20_000));
        assert_eq!(tx.nonce_as_u64(), Some(10));
    }

    #[test]
    fn test_vnet_transaction_alias_hash() {
        // Test that both 'hash' and 'tx_hash' are accepted (for backwards compat)
        let json_with_hash = r#"{"hash": "0x123"}"#;
        let tx: VNetTransaction = serde_json::from_str(json_with_hash).unwrap();
        assert_eq!(tx.tx_hash.as_deref(), Some("0x123"));

        let json_with_tx_hash = r#"{"tx_hash": "0x456"}"#;
        let tx: VNetTransaction = serde_json::from_str(json_with_tx_hash).unwrap();
        assert_eq!(tx.tx_hash.as_deref(), Some("0x456"));
    }

    #[test]
    fn test_vnet_transaction_fixture_without_hash() {
        // Fixture transactions (admin operations) don't have tx_hash
        let json = r#"{
            "id": "fixture-123",
            "kind": "fixture",
            "status": "success",
            "origin": "rpc"
        }"#;

        let tx: VNetTransaction = serde_json::from_str(json).unwrap();
        assert!(tx.tx_hash.is_none());
        assert_eq!(tx.kind.as_deref(), Some("fixture"));
        assert!(tx.is_success());
    }

    #[test]
    fn test_vnet_transactions_array_deserialization() {
        // API returns a raw array, not a wrapped object
        let json = r#"[
            {"tx_hash": "0x111", "status": "success", "kind": "blockchain"},
            {"tx_hash": "0x222", "status": "failed", "kind": "blockchain"},
            {"id": "fixture-1", "status": "success", "kind": "fixture"}
        ]"#;

        let txs: Vec<VNetTransaction> = serde_json::from_str(json).unwrap();

        assert_eq!(txs.len(), 3);
        assert_eq!(txs[0].tx_hash.as_deref(), Some("0x111"));
        assert!(txs[0].is_success());
        assert_eq!(txs[1].tx_hash.as_deref(), Some("0x222"));
        assert!(txs[1].is_failed());
        assert!(txs[2].tx_hash.is_none()); // Fixture has no tx_hash
        assert_eq!(txs[2].kind.as_deref(), Some("fixture"));
    }

    #[test]
    fn test_parse_hex_u64() {
        assert_eq!(parse_hex_u64("0x0"), Some(0));
        assert_eq!(parse_hex_u64("0x1"), Some(1));
        assert_eq!(parse_hex_u64("0xa"), Some(10));
        assert_eq!(parse_hex_u64("0x10"), Some(16));
        assert_eq!(parse_hex_u64("0x5208"), Some(21_000));
        assert_eq!(parse_hex_u64("0x170abab"), Some(24_161_195));
        // Without 0x prefix
        assert_eq!(parse_hex_u64("5208"), Some(21_000));
        assert_eq!(parse_hex_u64("170abab"), Some(24_161_195));
    }

    #[test]
    fn test_list_vnet_transactions_query_builder() {
        let query = ListVNetTransactionsQuery::new()
            .address("0x1234")
            .success()
            .page(2)
            .per_page(50);

        assert_eq!(query.address, Some("0x1234".to_string()));
        assert_eq!(query.status, Some("success".to_string()));
        assert_eq!(query.page, Some(2));
        assert_eq!(query.per_page, Some(50));

        let failed_query = ListVNetTransactionsQuery::new().failed();
        assert_eq!(failed_query.status, Some("failed".to_string()));
    }
}
