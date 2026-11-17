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

/// Most wallet-network pairs one bulk request may add
pub const BULK_ADD_WALLETS_MAX_PAIRS: usize = 100;

/// Request to add many wallets to the project at once
///
/// Each wallet lists its networks; every address-network combination is one
/// pair, and a request takes at most [`BULK_ADD_WALLETS_MAX_PAIRS`] distinct
/// pairs.
///
/// ```
/// use tndrly::wallets::{BulkAddWalletsRequest, BulkWallet};
///
/// let request = BulkAddWalletsRequest::new()
///     .wallet(
///         BulkWallet::new("0xe2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2")
///             .network("1")
///             .network("8453")
///             .display_name("Treasury")
///             .tag("treasury"),
///     )
///     .wallet(BulkWallet::new("0xb0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0").network("1"));
/// assert!(request.validate().is_ok());
/// ```
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct BulkAddWalletsRequest {
    /// Wallets to add
    pub wallets: Vec<BulkWallet>,
}

impl BulkAddWalletsRequest {
    /// Create an empty request
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a wallet
    #[must_use]
    pub fn wallet(mut self, wallet: BulkWallet) -> Self {
        self.wallets.push(wallet);
        self
    }

    /// Check the request before sending
    ///
    /// # Errors
    /// If there are no wallets, a wallet has no network, or there are more
    /// than [`BULK_ADD_WALLETS_MAX_PAIRS`] distinct wallet-network pairs.
    pub fn validate(&self) -> Result<(), String> {
        if self.wallets.is_empty() {
            return Err("At least one wallet is required. Use .wallet() to add one.".to_string());
        }
        if let Some(w) = self.wallets.iter().find(|w| w.network_ids.is_empty()) {
            return Err(format!(
                "Wallet {} has no network_id. Use .network() to add one.",
                w.address
            ));
        }
        let pairs: std::collections::HashSet<(String, &str)> = self
            .wallets
            .iter()
            .flat_map(|w| {
                w.network_ids
                    .iter()
                    .map(|n| (w.address.to_lowercase(), n.as_str()))
            })
            .collect();
        if pairs.len() > BULK_ADD_WALLETS_MAX_PAIRS {
            return Err(format!(
                "{} wallet-network pairs exceed the limit of {BULK_ADD_WALLETS_MAX_PAIRS} per request",
                pairs.len()
            ));
        }
        Ok(())
    }
}

/// One wallet of a [`BulkAddWalletsRequest`]
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct BulkWallet {
    /// Wallet address
    pub address: String,

    /// Networks to add it on (e.g. `["1", "8453"]`)
    pub network_ids: Vec<String>,

    /// Display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Tags (at most 10)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl BulkWallet {
    /// A wallet; add at least one network with [`network`](Self::network)
    pub fn new(address: impl Into<String>) -> Self {
        Self {
            address: address.into(),
            network_ids: Vec::new(),
            display_name: None,
            tags: Vec::new(),
        }
    }

    /// Add a network
    #[must_use]
    pub fn network(mut self, network_id: impl Into<String>) -> Self {
        self.network_ids.push(network_id.into());
        self
    }

    /// Set the display name
    #[must_use]
    pub fn display_name(mut self, name: impl Into<String>) -> Self {
        self.display_name = Some(name.into());
        self
    }

    /// Add a tag
    #[must_use]
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Response of a bulk add: the requested addresses sorted into those added
/// and those the project already had (left unchanged)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct BulkAddWalletsResponse {
    /// Wallets this request added, one entry per wallet-network pair
    #[serde(default, deserialize_with = "null_as_empty")]
    pub added: Vec<WalletOnNetwork>,

    /// Addresses the project already had, as wallets or contracts
    #[serde(default, deserialize_with = "null_as_empty")]
    pub existing: Vec<WalletOnNetwork>,
}

fn null_as_empty<'de, D>(deserializer: D) -> Result<Vec<WalletOnNetwork>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<Vec<WalletOnNetwork>>::deserialize(deserializer)?.unwrap_or_default())
}

/// Response from adding a wallet: one entry per network (API returns a raw array)
pub type AddWalletResponse = Vec<WalletOnNetwork>;

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
    #[must_use]
    pub fn address(&self) -> Option<&str> {
        self.account.as_ref().map(|a| a.address.as_str())
    }

    /// Get the network ID
    #[must_use]
    pub fn network_id(&self) -> Option<&str> {
        self.account.as_ref().and_then(|a| a.network_id.as_deref())
    }

    /// Get the balance in hex format
    #[must_use]
    pub fn balance(&self) -> Option<&str> {
        self.account.as_ref().and_then(|a| a.balance.as_deref())
    }

    /// Check if this is a wallet (not a contract)
    #[must_use]
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
