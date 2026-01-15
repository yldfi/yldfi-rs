//! Types for Contract API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Request to add a contract
#[derive(Debug, Clone, Serialize)]
pub struct AddContractRequest {
    /// Network ID
    pub network_id: String,

    /// Contract address
    pub address: String,

    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Tags for organizing contracts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl AddContractRequest {
    /// Create a new add contract request
    pub fn new(network_id: impl Into<String>, address: impl Into<String>) -> Self {
        Self {
            network_id: network_id.into(),
            address: address.into(),
            display_name: None,
            tags: None,
        }
    }

    /// Set display name
    #[must_use]
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set tags
    #[must_use]
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }

    /// Add a single tag
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.get_or_insert_with(Vec::new).push(tag.into());
        self
    }
}

/// Contract in project (wrapper returned by API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contract {
    /// Project contract ID (e.g., "eth:1:0x...")
    pub id: String,

    /// Project ID
    #[serde(default)]
    pub project_id: Option<String>,

    /// Display name (user-set)
    #[serde(default)]
    pub display_name: Option<String>,

    /// Nested contract details
    #[serde(default)]
    pub contract: Option<ContractDetails>,

    /// When the contract was added to the project
    #[serde(default)]
    pub added_at: Option<String>,

    /// Verification type
    #[serde(default)]
    pub verification_type: Option<String>,

    /// Account type (e.g., "contract", "wallet")
    #[serde(default)]
    pub account_type: Option<String>,
}

impl Contract {
    /// Get the contract address
    pub fn address(&self) -> Option<&str> {
        self.contract.as_ref().map(|c| c.address.as_str())
    }

    /// Get the network ID
    pub fn network_id(&self) -> Option<&str> {
        self.contract.as_ref().and_then(|c| c.network_id.as_deref())
    }

    /// Get the contract name (from verification)
    pub fn contract_name(&self) -> Option<&str> {
        self.contract
            .as_ref()
            .and_then(|c| c.contract_name.as_deref())
    }

    /// Check if the contract is verified
    pub fn is_verified(&self) -> bool {
        self.contract
            .as_ref()
            .is_some_and(|c| c.verification_date.is_some())
    }

    /// Get tags from contract details
    pub fn tags(&self) -> Vec<String> {
        // Tags may be stored at different levels depending on API version
        Vec::new()
    }

    /// Check if this is an actual contract (not a wallet)
    ///
    /// The `/contracts` list endpoint returns both contracts and wallets.
    /// Use this to filter for actual contracts, especially before calling
    /// `contracts().get()` which only works for contracts.
    pub fn is_contract(&self) -> bool {
        self.account_type
            .as_ref()
            .is_some_and(|t| t == "contract" || t == "unverified_contract")
    }

    /// Check if this is a wallet (EOA)
    ///
    /// The `/contracts` list endpoint returns both contracts and wallets.
    pub fn is_wallet(&self) -> bool {
        self.account_type.as_ref().is_some_and(|t| t == "wallet")
    }
}

/// Nested contract details from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDetails {
    /// Contract ID
    #[serde(default)]
    pub id: Option<String>,

    /// Contract address
    pub address: String,

    /// Network ID
    #[serde(default)]
    pub network_id: Option<String>,

    /// Contract name (from verification)
    #[serde(default)]
    pub contract_name: Option<String>,

    /// Verification date
    #[serde(default)]
    pub verification_date: Option<String>,

    /// Token standard (e.g., "erc20")
    #[serde(default)]
    pub standard: Option<String>,

    /// Multiple standards
    #[serde(default)]
    pub standards: Vec<String>,

    /// Token data (for ERC20 tokens)
    #[serde(default)]
    pub token_data: Option<TokenData>,

    /// Whether the contract is public
    #[serde(default)]
    pub public: bool,

    /// Compiler version
    #[serde(default)]
    pub compiler_version: Option<String>,

    /// Contract language
    #[serde(default)]
    pub language: Option<String>,

    /// Whether in project
    #[serde(default)]
    pub in_project: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Balance (for display)
    #[serde(default)]
    pub balance: Option<String>,
}

/// Token data for ERC20/ERC721 contracts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    /// Token symbol
    #[serde(default)]
    pub symbol: Option<String>,

    /// Token name
    #[serde(default)]
    pub name: Option<String>,

    /// Decimals (for ERC20)
    #[serde(default)]
    pub decimals: Option<u8>,
}

/// Request to update a contract
#[derive(Debug, Clone, Serialize)]
pub struct UpdateContractRequest {
    /// New display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// New tags (replaces existing)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl UpdateContractRequest {
    /// Create a new update request
    pub fn new() -> Self {
        Self {
            display_name: None,
            tags: None,
        }
    }

    /// Set display name
    #[must_use]
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Set tags
    #[must_use]
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
}

impl Default for UpdateContractRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request to verify a contract
#[derive(Debug, Clone, Serialize)]
pub struct VerifyContractRequest {
    /// Network ID
    pub network_id: String,

    /// Contract address
    pub address: String,

    /// Contract name
    pub contract_name: String,

    /// Solidity source code
    pub source_code: String,

    /// Compiler version (e.g., "v0.8.19+commit.7dd6d404")
    pub compiler_version: String,

    /// Optimization settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimization: Option<OptimizationSettings>,

    /// Constructor arguments (ABI-encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub constructor_arguments: Option<String>,

    /// Libraries used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub libraries: Option<HashMap<String, String>>,

    /// EVM version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evm_version: Option<String>,
}

impl VerifyContractRequest {
    /// Create a new verify request
    pub fn new(
        network_id: impl Into<String>,
        address: impl Into<String>,
        contract_name: impl Into<String>,
        source_code: impl Into<String>,
        compiler_version: impl Into<String>,
    ) -> Self {
        Self {
            network_id: network_id.into(),
            address: address.into(),
            contract_name: contract_name.into(),
            source_code: source_code.into(),
            compiler_version: compiler_version.into(),
            optimization: None,
            constructor_arguments: None,
            libraries: None,
            evm_version: None,
        }
    }

    /// Set optimization settings
    #[must_use]
    pub fn optimization(mut self, enabled: bool, runs: u32) -> Self {
        self.optimization = Some(OptimizationSettings { enabled, runs });
        self
    }

    /// Set constructor arguments
    #[must_use]
    pub fn constructor_arguments(mut self, args: impl Into<String>) -> Self {
        self.constructor_arguments = Some(args.into());
        self
    }

    /// Add a library
    #[must_use]
    pub fn library(mut self, name: impl Into<String>, address: impl Into<String>) -> Self {
        self.libraries
            .get_or_insert_with(HashMap::new)
            .insert(name.into(), address.into());
        self
    }

    /// Set EVM version
    #[must_use]
    pub fn evm_version(mut self, version: impl Into<String>) -> Self {
        self.evm_version = Some(version.into());
        self
    }
}

/// Optimization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    /// Whether optimization is enabled
    pub enabled: bool,
    /// Number of optimization runs
    pub runs: u32,
}

/// Verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    /// Whether verification succeeded
    pub success: bool,

    /// Error message (if failed)
    #[serde(default)]
    pub error: Option<String>,

    /// Contract details (if successful)
    #[serde(default)]
    pub contract: Option<Contract>,
}

/// Request to encode state overrides
#[derive(Debug, Clone, Serialize)]
pub struct EncodeStateRequest {
    /// Network ID
    pub network_id: String,

    /// State overrides to encode
    pub state_overrides: HashMap<String, StateOverrideInput>,
}

/// Input format for state overrides
#[derive(Debug, Clone, Serialize)]
pub struct StateOverrideInput {
    /// Storage slot values to override
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<HashMap<String, String>>,

    /// Balance to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub balance: Option<String>,

    /// Nonce to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nonce: Option<u64>,

    /// Code to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
}

impl StateOverrideInput {
    /// Create empty state override
    pub fn new() -> Self {
        Self {
            storage: None,
            balance: None,
            nonce: None,
            code: None,
        }
    }

    /// Set a storage slot
    #[must_use]
    pub fn storage(mut self, slot: impl Into<String>, value: impl Into<String>) -> Self {
        self.storage
            .get_or_insert_with(HashMap::new)
            .insert(slot.into(), value.into());
        self
    }

    /// Set balance
    #[must_use]
    pub fn balance(mut self, balance: impl Into<String>) -> Self {
        self.balance = Some(balance.into());
        self
    }

    /// Set nonce
    #[must_use]
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set code
    #[must_use]
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl Default for StateOverrideInput {
    fn default() -> Self {
        Self::new()
    }
}

/// Response from encoding state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodeStateResponse {
    /// Encoded state overrides ready for use in simulations
    pub encoded_state: serde_json::Value,
}

/// Query parameters for listing contracts
#[derive(Debug, Clone, Default, Serialize)]
pub struct ListContractsQuery {
    /// Filter by tag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,

    /// Filter by network
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_id: Option<String>,

    /// Filter by verification status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verified: Option<bool>,

    /// Page number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,

    /// Results per page
    #[serde(skip_serializing_if = "Option::is_none")]
    pub per_page: Option<u32>,
}

impl ListContractsQuery {
    /// Create a new query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by tag
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Filter by network
    #[must_use]
    pub fn network_id(mut self, network_id: impl Into<String>) -> Self {
        self.network_id = Some(network_id.into());
        self
    }

    /// Filter by verification status
    #[must_use]
    pub fn verified(mut self, verified: bool) -> Self {
        self.verified = Some(verified);
        self
    }

    /// Set page
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

/// Request to rename a contract
#[derive(Debug, Clone, Serialize)]
pub struct RenameContractRequest {
    /// New display name for the contract
    pub display_name: String,
}

/// Request to bulk tag contracts
#[derive(Debug, Clone, Serialize)]
pub struct BulkTagRequest {
    /// Tag to apply
    pub tag: String,

    /// Contract IDs in format "eth:{network_id}:{address}"
    pub contract_ids: Vec<String>,
}

/// Response from bulk tag operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkTagResponse {
    /// Tag that was applied
    #[serde(default)]
    pub tag: Option<TagInfo>,
}

/// Tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// Tag name
    pub tag: String,

    /// When the tag was created
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Request to delete a tag from a contract
#[derive(Debug, Clone, Serialize)]
pub struct DeleteTagRequest {
    /// Tag to delete
    pub tag: String,
}
