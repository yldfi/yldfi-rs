//! Types for the Transfers API

use serde::{Deserialize, Serialize};

/// Transfer category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransferCategory {
    /// External (ETH) transfers
    External,
    /// Internal (contract-to-contract) transfers
    Internal,
    /// ERC-20 token transfers
    Erc20,
    /// ERC-721 NFT transfers
    Erc721,
    /// ERC-1155 multi-token transfers
    Erc1155,
    /// Special NFT transfers
    #[serde(rename = "specialnft")]
    SpecialNft,
}

/// Sort order for transfers
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferOrder {
    Ascending,
    Descending,
}

impl Serialize for TransferOrder {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            TransferOrder::Ascending => serializer.serialize_str("asc"),
            TransferOrder::Descending => serializer.serialize_str("desc"),
        }
    }
}

/// Options for getAssetTransfers
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransfersOptions {
    /// Starting block (hex or "0x0")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_block: Option<String>,

    /// Ending block (hex or "latest")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_block: Option<String>,

    /// Filter by sender address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_address: Option<String>,

    /// Filter by receiver address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_address: Option<String>,

    /// Filter by contract addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_addresses: Option<Vec<String>>,

    /// Categories of transfers to include
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<Vec<TransferCategory>>,

    /// Whether to exclude zero-value transfers
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_zero_value: Option<bool>,

    /// Maximum number of results (hex string)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_count: Option<String>,

    /// Sort order
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<TransferOrder>,

    /// Whether to include block metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_metadata: Option<bool>,

    /// Pagination key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
}

impl AssetTransfersOptions {
    /// Create options for getting transfers from an address
    pub fn from_address(address: &str) -> Self {
        Self {
            from_address: Some(address.to_string()),
            category: Some(vec![
                TransferCategory::External,
                TransferCategory::Erc20,
                TransferCategory::Erc721,
                TransferCategory::Erc1155,
            ]),
            ..Default::default()
        }
    }

    /// Create options for getting transfers to an address
    pub fn to_address(address: &str) -> Self {
        Self {
            to_address: Some(address.to_string()),
            category: Some(vec![
                TransferCategory::External,
                TransferCategory::Erc20,
                TransferCategory::Erc721,
                TransferCategory::Erc1155,
            ]),
            ..Default::default()
        }
    }

    /// Set block range
    pub fn with_block_range(mut self, from: &str, to: &str) -> Self {
        self.from_block = Some(from.to_string());
        self.to_block = Some(to.to_string());
        self
    }

    /// Include internal transfers
    pub fn with_internal_transfers(mut self) -> Self {
        let mut categories = self.category.unwrap_or_default();
        if !categories.contains(&TransferCategory::Internal) {
            categories.push(TransferCategory::Internal);
        }
        self.category = Some(categories);
        self
    }

    /// Exclude zero-value transfers
    pub fn exclude_zero_value(mut self) -> Self {
        self.exclude_zero_value = Some(true);
        self
    }

    /// Set maximum results
    pub fn with_max_count(mut self, count: u32) -> Self {
        self.max_count = Some(format!("0x{:x}", count));
        self
    }

    /// Set sort order
    pub fn with_order(mut self, order: TransferOrder) -> Self {
        self.order = Some(order);
        self
    }

    /// Include block metadata
    pub fn with_metadata(mut self) -> Self {
        self.with_metadata = Some(true);
        self
    }
}

/// Asset transfer entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransfer {
    /// Block number (hex)
    pub block_num: String,

    /// Unique ID for this transfer
    pub unique_id: String,

    /// Transaction hash
    pub hash: String,

    /// Sender address
    pub from: String,

    /// Receiver address
    pub to: Option<String>,

    /// Transfer value (decimal string)
    pub value: Option<f64>,

    /// ERC721 token ID (for NFT transfers)
    pub erc721_token_id: Option<String>,

    /// ERC1155 metadata
    pub erc1155_metadata: Option<Vec<Erc1155Metadata>>,

    /// Token ID (for NFT transfers)
    pub token_id: Option<String>,

    /// Asset type (ETH, token symbol, etc.)
    pub asset: Option<String>,

    /// Transfer category
    pub category: TransferCategory,

    /// Raw contract info
    pub raw_contract: Option<RawContract>,

    /// Block metadata (if withMetadata=true)
    pub metadata: Option<TransferMetadata>,
}

/// ERC1155 metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc1155Metadata {
    /// Token ID
    pub token_id: String,
    /// Amount transferred
    pub value: String,
}

/// Raw contract information
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawContract {
    /// Contract address
    pub address: Option<String>,
    /// Raw value (hex)
    pub value: Option<String>,
    /// Token decimals
    pub decimal: Option<String>,
}

/// Transfer metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TransferMetadata {
    /// Block timestamp
    pub block_timestamp: String,
}

/// Response for getAssetTransfers
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetTransfersResponse {
    /// List of transfers
    pub transfers: Vec<AssetTransfer>,
    /// Pagination key for next page
    pub page_key: Option<String>,
}
