//! Types for the Portfolio/Data API

use serde::{Deserialize, Serialize};

/// Address-network pair for queries
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressNetwork {
    /// Wallet address
    pub address: String,
    /// Network names to query
    pub networks: Vec<String>,
}

/// Request for token balances
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalancesRequest {
    pub addresses: Vec<AddressNetwork>,
}

/// Token balance entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceEntry {
    /// Token contract address
    pub address: String,
    /// Network name
    pub network: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Raw balance (hex or decimal string)
    pub balance: String,
    /// Token type (native, erc20, etc.)
    pub token_type: Option<String>,
    /// Token logo URL
    pub logo: Option<String>,
    /// USD value (if available)
    pub usd_value: Option<f64>,
    /// Token price in USD
    pub token_price_usd: Option<f64>,
}

/// Response for token balances
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalancesResponse {
    pub data: Vec<WalletTokenBalances>,
}

/// Token balances for a wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTokenBalances {
    pub address: String,
    pub token_balances: Vec<TokenBalanceEntry>,
}

/// Request for token info
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfoRequest {
    pub addresses: Vec<TokenAddressInfo>,
}

/// Token address info for lookup
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAddressInfo {
    /// Network name
    pub network: String,
    /// Token contract address
    pub address: String,
}

/// Token info entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Token contract address
    pub address: String,
    /// Network name
    pub network: String,
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals
    pub decimals: Option<u8>,
    /// Token logo URL
    pub logo: Option<String>,
    /// Error if token info couldn't be fetched
    pub error: Option<String>,
}

/// Response for token info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfoResponse {
    pub data: Vec<TokenInfo>,
}

/// Request for NFTs by address
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftsByAddressRequest {
    pub addresses: Vec<AddressNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
}

/// NFT entry from portfolio
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioNft {
    /// Contract address
    pub contract_address: String,
    /// Token ID
    pub token_id: String,
    /// Network name
    pub network: String,
    /// Token type (ERC721, ERC1155)
    pub token_type: Option<String>,
    /// NFT name
    pub name: Option<String>,
    /// NFT description
    pub description: Option<String>,
    /// Image URL
    pub image_url: Option<String>,
    /// Balance (for ERC1155)
    pub balance: Option<String>,
    /// Collection name
    pub collection_name: Option<String>,
}

/// Response for NFTs by address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftsByAddressResponse {
    pub data: Vec<WalletNfts>,
    pub page_key: Option<String>,
}

/// NFTs for a wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletNfts {
    pub address: String,
    pub nfts: Vec<PortfolioNft>,
}

/// Request for NFT contracts by address
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftContractsByAddressRequest {
    pub addresses: Vec<AddressNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
}

/// NFT contract entry
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioNftContract {
    /// Contract address
    pub contract_address: String,
    /// Network name
    pub network: String,
    /// Contract name
    pub name: Option<String>,
    /// Contract symbol
    pub symbol: Option<String>,
    /// Token type (ERC721, ERC1155)
    pub token_type: Option<String>,
    /// Number of tokens owned
    pub total_balance: Option<String>,
    /// Number of distinct tokens owned
    pub num_distinct_tokens_owned: Option<u64>,
    /// Whether the contract is spam
    pub is_spam: Option<bool>,
}

/// Response for NFT contracts by address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftContractsByAddressResponse {
    pub data: Vec<WalletNftContracts>,
    pub page_key: Option<String>,
}

/// NFT contracts for a wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletNftContracts {
    pub address: String,
    pub contracts: Vec<PortfolioNftContract>,
}
