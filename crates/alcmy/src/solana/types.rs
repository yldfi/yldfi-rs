//! Types for the Solana DAS API

use serde::{Deserialize, Serialize};

/// Asset display options
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayOptions {
    /// Show fungible tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_fungible: Option<bool>,
    /// Show native balance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_native_balance: Option<bool>,
    /// Show inscription
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_inscription: Option<bool>,
    /// Show zero balance
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_zero_balance: Option<bool>,
}

/// Pagination options
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationOptions {
    /// Page size
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Page number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<u32>,
    /// Cursor for pagination
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// Sort by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortBy>,
}

/// Sort by options
#[derive(Debug, Clone, Serialize)]
pub struct SortBy {
    pub sort_by: String,
    pub sort_direction: String,
}

/// Asset interface type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interface {
    #[serde(rename = "V1_NFT")]
    V1Nft,
    #[serde(rename = "V1_PRINT")]
    V1Print,
    #[serde(rename = "LEGACY_NFT")]
    LegacyNft,
    #[serde(rename = "V2_NFT")]
    V2Nft,
    #[serde(rename = "FungibleAsset")]
    FungibleAsset,
    #[serde(rename = "FungibleToken")]
    FungibleToken,
    #[serde(rename = "Custom")]
    Custom,
    #[serde(rename = "Identity")]
    Identity,
    #[serde(rename = "Executable")]
    Executable,
    #[serde(rename = "ProgrammableNFT")]
    ProgrammableNft,
    #[serde(other)]
    Other,
}

/// Asset
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Asset interface
    pub interface: Interface,
    /// Asset ID
    pub id: String,
    /// Content
    pub content: Option<AssetContent>,
    /// Authorities
    #[serde(default)]
    pub authorities: Vec<Authority>,
    /// Compression info
    pub compression: Option<Compression>,
    /// Grouping
    #[serde(default)]
    pub grouping: Vec<Grouping>,
    /// Royalty
    pub royalty: Option<Royalty>,
    /// Creators
    #[serde(default)]
    pub creators: Vec<Creator>,
    /// Ownership
    pub ownership: Option<Ownership>,
    /// Supply (for fungible)
    pub supply: Option<Supply>,
    /// Mutable
    pub mutable: Option<bool>,
    /// Burnt
    pub burnt: Option<bool>,
    /// Token info (for fungible)
    pub token_info: Option<TokenInfo>,
}

/// Asset content
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetContent {
    /// Schema
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    /// JSON URI
    pub json_uri: Option<String>,
    /// Files
    #[serde(default)]
    pub files: Vec<AssetFile>,
    /// Metadata
    pub metadata: Option<AssetMetadata>,
    /// Links
    pub links: Option<AssetLinks>,
}

/// Asset file
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetFile {
    /// URI
    pub uri: Option<String>,
    /// CDN URI
    pub cdn_uri: Option<String>,
    /// MIME type
    pub mime: Option<String>,
}

/// Asset metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetMetadata {
    /// Name
    pub name: Option<String>,
    /// Symbol
    pub symbol: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Attributes
    #[serde(default)]
    pub attributes: Vec<AssetAttribute>,
}

/// Asset attribute
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetAttribute {
    pub trait_type: Option<String>,
    pub value: Option<serde_json::Value>,
}

/// Asset links
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetLinks {
    pub external_url: Option<String>,
    pub image: Option<String>,
    pub animation_url: Option<String>,
}

/// Authority
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Authority {
    pub address: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Compression info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Compression {
    pub eligible: bool,
    pub compressed: bool,
    pub data_hash: Option<String>,
    pub creator_hash: Option<String>,
    pub asset_hash: Option<String>,
    pub tree: Option<String>,
    pub seq: Option<u64>,
    pub leaf_id: Option<u64>,
}

/// Grouping
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grouping {
    pub group_key: String,
    pub group_value: String,
}

/// Royalty
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Royalty {
    pub royalty_model: Option<String>,
    pub target: Option<String>,
    pub percent: Option<f64>,
    pub basis_points: Option<u64>,
    pub primary_sale_happened: Option<bool>,
    pub locked: Option<bool>,
}

/// Creator
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Creator {
    pub address: String,
    pub share: u8,
    pub verified: bool,
}

/// Ownership
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ownership {
    pub frozen: bool,
    pub delegated: bool,
    pub delegate: Option<String>,
    pub ownership_model: String,
    pub owner: String,
}

/// Supply
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Supply {
    pub print_max_supply: Option<u64>,
    pub print_current_supply: Option<u64>,
    pub edition_nonce: Option<u64>,
}

/// Token info (for fungible tokens)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    pub symbol: Option<String>,
    pub balance: Option<u64>,
    pub supply: Option<u64>,
    pub decimals: Option<u8>,
    pub token_program: Option<String>,
    pub associated_token_address: Option<String>,
    pub price_info: Option<PriceInfo>,
}

/// Price info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceInfo {
    pub price_per_token: Option<f64>,
    pub total_price: Option<f64>,
    pub currency: Option<String>,
}

/// Asset proof
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetProof {
    pub root: String,
    pub proof: Vec<String>,
    pub node_index: u64,
    pub leaf: String,
    pub tree_id: String,
}

/// Get assets response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetsResponse {
    pub items: Vec<Asset>,
    pub total: Option<u64>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub cursor: Option<String>,
}

/// Search assets request
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchAssetsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creator_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authority_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<Interface>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub burnt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub json_uri: Option<String>,
    #[serde(flatten)]
    pub pagination: PaginationOptions,
    #[serde(flatten)]
    pub display: DisplayOptions,
}

/// Token account
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAccount {
    pub address: String,
    pub mint: String,
    pub owner: String,
    pub amount: u64,
    pub delegated_amount: Option<u64>,
    pub frozen: bool,
}

/// Get token accounts response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetTokenAccountsResponse {
    pub token_accounts: Vec<TokenAccount>,
    pub total: Option<u64>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
    pub cursor: Option<String>,
}

/// NFT edition
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftEdition {
    pub mint: String,
    pub edition_number: u64,
}

/// Get NFT editions response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetNftEditionsResponse {
    pub editions: Vec<NftEdition>,
    pub master_edition_address: String,
    pub supply: u64,
    pub max_supply: Option<u64>,
}

/// Asset signature
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSignature {
    pub signature: String,
    pub slot: u64,
}

/// Get asset signatures response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetSignaturesResponse {
    pub items: Vec<AssetSignature>,
    pub total: Option<u64>,
    pub limit: Option<u32>,
    pub page: Option<u32>,
}
