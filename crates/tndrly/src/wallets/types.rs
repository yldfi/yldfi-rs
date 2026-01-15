//! Types for Wallet API

use serde::{Deserialize, Serialize};

/// Request to add a wallet to the project
#[derive(Debug, Clone, Serialize)]
pub struct AddWalletRequest {
    /// Wallet address
    pub address: String,

    /// Network IDs to monitor this wallet on (e.g., ["1", "137"])
    pub network_ids: Vec<String>,

    /// Display name for the wallet
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

impl AddWalletRequest {
    /// Create a new add wallet request
    ///
    /// You must add at least one network using `.network()` before submitting.
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            network_ids: Vec::new(),
            display_name: None,
        }
    }

    /// Validate the request before sending
    ///
    /// Returns an error if:
    /// - No networks have been specified (at least one is required)
    pub fn validate(&self) -> Result<(), String> {
        if self.network_ids.is_empty() {
            return Err(
                "At least one network_id is required. Use .network() to add networks.".to_string(),
            );
        }
        Ok(())
    }

    /// Add a network to monitor this wallet on
    #[must_use]
    pub fn network(mut self, network_id: impl Into<String>) -> Self {
        self.network_ids.push(network_id.into());
        self
    }

    /// Add multiple networks at once
    #[must_use]
    pub fn networks(mut self, network_ids: Vec<String>) -> Self {
        self.network_ids.extend(network_ids);
        self
    }

    /// Set display name
    #[must_use]
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }
}

/// Response from adding a wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddWalletResponse {
    /// Account ID
    #[serde(default)]
    pub account_id: Option<String>,

    /// Account type (should be "wallet")
    #[serde(default, rename = "type")]
    pub account_type: Option<String>,

    /// Wallet details per network
    #[serde(default)]
    pub contracts: Vec<WalletOnNetwork>,
}

/// Wallet information on a specific network (wrapper returned by API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletOnNetwork {
    /// Wallet ID (e.g., "eth:1:0x...")
    pub id: String,

    /// Project ID
    #[serde(default)]
    pub project_id: Option<String>,

    /// Display name (user-set)
    #[serde(default)]
    pub display_name: Option<String>,

    /// Nested account details
    #[serde(default)]
    pub account: Option<WalletAccount>,

    /// Tags
    #[serde(default)]
    pub tags: Vec<WalletTag>,

    /// Account type (e.g., "wallet")
    #[serde(default)]
    pub account_type: Option<String>,

    /// When the wallet was added
    #[serde(default)]
    pub added_at: Option<String>,
}

impl WalletOnNetwork {
    /// Get the wallet address
    pub fn address(&self) -> Option<&str> {
        self.account.as_ref().map(|a| a.address.as_str())
    }

    /// Get the network ID
    pub fn network_id(&self) -> Option<&str> {
        self.account.as_ref().and_then(|a| a.network_id.as_deref())
    }

    /// Get the balance in hex format
    pub fn balance(&self) -> Option<&str> {
        self.account.as_ref().and_then(|a| a.balance.as_deref())
    }

    /// Check if this is a wallet (not a contract)
    pub fn is_wallet(&self) -> bool {
        self.account
            .as_ref()
            .is_some_and(|a| a.account_type.as_deref() == Some("wallet"))
    }
}

/// Nested wallet account details from API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    /// Account ID
    #[serde(default)]
    pub id: Option<String>,

    /// Wallet address
    pub address: String,

    /// Network ID
    #[serde(default)]
    pub network_id: Option<String>,

    /// Balance in hex format
    #[serde(default)]
    pub balance: Option<String>,

    /// Account type (e.g., "wallet", "contract")
    #[serde(default, rename = "type")]
    pub account_type: Option<String>,

    /// Whether the wallet is public
    #[serde(default)]
    pub public: bool,

    /// ENS domain if any
    #[serde(default)]
    pub ens_domain: Option<String>,

    /// Whether in project
    #[serde(default)]
    pub in_project: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Wallet tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTag {
    /// Tag name
    pub tag: String,

    /// When the tag was created
    #[serde(default)]
    pub created_at: Option<String>,
}
