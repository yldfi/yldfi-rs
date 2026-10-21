//! Types for the Portfolio/Data API

use crate::nft::{
    AcquiredAt, CollectionInfo, ContractMetadata, NftImage, NftRawMetadata, NftTokenType,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Address-network pair for queries
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddressNetwork {
    /// Wallet address
    pub address: String,
    /// Network names to query
    pub networks: Vec<String>,
}

/// Networks that failed on an otherwise successful (HTTP 200) response
///
/// The API fans out across every requested network; networks that error or
/// time out are listed here instead of failing the whole request. The key is
/// omitted when every network succeeded.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialError {
    /// Human-readable summary
    #[serde(default)]
    pub message: String,
    /// One entry per failed network
    #[serde(default)]
    pub partial_errors: Vec<PartialErrorItem>,
}

/// A single failed network
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialErrorItem {
    /// Network name
    pub network: String,
    /// Failure reason (opaque text)
    #[serde(default)]
    pub message: String,
}

/// Block used to compute balances on one network (`includeBlockMetadata`)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BlockMetadataEntry {
    /// Block number (hex)
    pub block_number: String,
    /// Block hash
    pub block_hash: String,
    /// Block timestamp (ISO-8601)
    pub block_timestamp: String,
}

/// Block metadata keyed by network; failed networks map to `None`
pub type BlockMetadata = HashMap<String, Option<BlockMetadataEntry>>;

/// Request for token balances
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalancesRequest {
    pub addresses: Vec<AddressNetwork>,
    /// Include each chain's native token (default true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_native_tokens: Option<bool>,
    /// Include ERC-20 tokens (default true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_erc20_tokens: Option<bool>,
    /// Return `data.blockMetadata` (default false)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_block_metadata: Option<bool>,
    /// Pagination cursor from a previous response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
}

/// Token balance entry from the API
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalanceEntry {
    /// Wallet address
    pub address: String,
    /// Network name
    pub network: String,
    /// Token contract address (null for native token)
    pub token_address: Option<String>,
    /// Raw balance (hex string)
    pub token_balance: String,
    /// Token name (from metadata)
    #[serde(default)]
    pub name: Option<String>,
    /// Token symbol (from metadata)
    #[serde(default)]
    pub symbol: Option<String>,
    /// Token decimals (from metadata)
    #[serde(default)]
    pub decimals: Option<u8>,
    /// Token logo URL (from metadata)
    #[serde(default)]
    pub logo: Option<String>,
}

/// Inner data structure for token balances
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalancesData {
    pub tokens: Vec<TokenBalanceEntry>,
    /// Cursor for the next page
    #[serde(default)]
    pub page_key: Option<String>,
    /// Per-network block metadata (only with `includeBlockMetadata`)
    #[serde(default)]
    pub block_metadata: Option<BlockMetadata>,
}

/// Response for token balances
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenBalancesResponse {
    pub data: TokenBalancesData,
    /// Networks that failed (absent when all succeeded)
    #[serde(default)]
    pub error: Option<PartialError>,
}

/// Token balances for a wallet (legacy, kept for backward compatibility)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTokenBalances {
    pub address: String,
    pub token_balances: Vec<TokenBalanceEntry>,
}

/// Options for `assets/tokens/by-address` (Tokens By Wallet)
///
/// Every field is optional; the API defaults are metadata, prices, native
/// and ERC-20 tokens on, block metadata off.
#[derive(Debug, Clone, Default)]
pub struct TokensByAddressOptions {
    pub with_metadata: Option<bool>,
    pub with_prices: Option<bool>,
    pub include_native_tokens: Option<bool>,
    pub include_erc20_tokens: Option<bool>,
    pub include_block_metadata: Option<bool>,
    pub page_key: Option<String>,
}

/// Request for tokens (with metadata and prices) by wallet
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensByAddressRequest {
    pub addresses: Vec<AddressNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_prices: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_native_tokens: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_erc20_tokens: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_block_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
}

/// Token metadata attached to a wallet token
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTokenMetadata {
    #[serde(default)]
    pub decimals: Option<u8>,
    #[serde(default)]
    pub logo: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub symbol: Option<String>,
}

/// Token price in one currency
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletTokenPrice {
    /// Currency code (e.g. "usd")
    pub currency: String,
    /// Price as a decimal string
    pub value: String,
    /// Last update time (ISO-8601)
    #[serde(default)]
    pub last_updated_at: Option<String>,
}

/// Token held by a wallet, with metadata and prices
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletToken {
    /// Wallet address
    pub address: String,
    /// Network name
    pub network: String,
    /// Token contract address (null for the native token)
    #[serde(default)]
    pub token_address: Option<String>,
    /// Raw balance (hex string)
    pub token_balance: String,
    /// Token metadata
    #[serde(default)]
    pub token_metadata: Option<WalletTokenMetadata>,
    /// Token prices
    #[serde(default)]
    pub token_prices: Vec<WalletTokenPrice>,
    /// Per-token metadata or pricing failure
    #[serde(default)]
    pub error: Option<String>,
}

/// Inner data for tokens by wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensByAddressData {
    pub tokens: Vec<WalletToken>,
    /// Cursor for the next page
    #[serde(default)]
    pub page_key: Option<String>,
    /// Per-network block metadata (only with `includeBlockMetadata`)
    #[serde(default)]
    pub block_metadata: Option<BlockMetadata>,
}

/// Response for tokens by wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokensByAddressResponse {
    pub data: TokensByAddressData,
    /// Networks that failed (absent when all succeeded)
    #[serde(default)]
    pub error: Option<PartialError>,
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

/// NFT owned by a wallet (v3 NFT shape plus `network`/`address`)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioNft {
    /// Network name
    pub network: String,
    /// Owner wallet address
    pub address: String,
    /// Contract metadata
    #[serde(default)]
    pub contract: Option<ContractMetadata>,
    /// Token ID
    #[serde(default)]
    pub token_id: Option<String>,
    /// Token type (ERC721, ERC1155)
    #[serde(default)]
    pub token_type: Option<NftTokenType>,
    /// NFT name
    #[serde(default)]
    pub name: Option<String>,
    /// NFT description
    #[serde(default)]
    pub description: Option<String>,
    /// Image URLs
    #[serde(default)]
    pub image: Option<NftImage>,
    /// Raw metadata
    #[serde(default)]
    pub raw: Option<NftRawMetadata>,
    /// Collection info
    #[serde(default)]
    pub collection: Option<CollectionInfo>,
    /// Token URI
    #[serde(default)]
    pub token_uri: Option<String>,
    /// Last metadata refresh time
    #[serde(default)]
    pub time_last_updated: Option<String>,
    /// Acquisition info
    #[serde(default)]
    pub acquired_at: Option<AcquiredAt>,
}

/// Inner data for NFTs by address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftsByAddressData {
    #[serde(default)]
    pub owned_nfts: Vec<PortfolioNft>,
    #[serde(default)]
    pub total_count: Option<u64>,
    /// Cursor for the next page
    #[serde(default)]
    pub page_key: Option<String>,
}

/// Response for NFTs by address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftsByAddressResponse {
    pub data: NftsByAddressData,
    /// Networks that failed (absent when all succeeded)
    #[serde(default)]
    pub error: Option<PartialError>,
}

/// Request for NFT contracts by address
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftContractsByAddressRequest {
    pub addresses: Vec<AddressNetwork>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub with_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_size: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_key: Option<String>,
}

/// NFT collection held by a wallet
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortfolioNftContract {
    /// Network name
    pub network: String,
    /// Owner wallet address
    pub address: String,
    /// Contract metadata
    #[serde(default)]
    pub contract: Option<ContractMetadata>,
}

/// Inner data for NFT contracts by address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftContractsByAddressData {
    #[serde(default)]
    pub contracts: Vec<PortfolioNftContract>,
    #[serde(default)]
    pub total_count: Option<u64>,
    /// Cursor for the next page
    #[serde(default)]
    pub page_key: Option<String>,
}

/// Response for NFT contracts by address
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NftContractsByAddressResponse {
    pub data: NftContractsByAddressData,
    /// Networks that failed (absent when all succeeded)
    #[serde(default)]
    pub error: Option<PartialError>,
}

#[cfg(test)]
mod tests {
    use super::*;

    const WALLET: &str = "0x0000000000000000000000000000000000000001";
    const NFT_CONTRACT: &str = "0x0000000000000000000000000000000000000002";

    #[test]
    fn tokens_by_address_parses_metadata_prices_and_partial_errors() {
        let response: TokensByAddressResponse = serde_json::from_value(serde_json::json!({
            "data": {
                "tokens": [{
                    "address": WALLET,
                    "network": "eth-mainnet",
                    "tokenAddress": null,
                    "tokenBalance": "0x01",
                    "tokenMetadata": { "decimals": 18, "name": "Ether", "symbol": "ETH" },
                    "tokenPrices": [{
                        "currency": "usd",
                        "value": "1.5",
                        "lastUpdatedAt": "2026-01-01T00:00:00Z"
                    }]
                }],
                "pageKey": "next",
                "blockMetadata": {
                    "eth-mainnet": {
                        "blockNumber": "0x1",
                        "blockHash": "0xab",
                        "blockTimestamp": "2026-01-01T00:00:00.000Z"
                    },
                    "base-mainnet": null
                }
            },
            "error": {
                "message": "Failed to fetch tokens on certain networks",
                "partialErrors": [{ "network": "base-mainnet", "message": "Internal server error" }]
            }
        }))
        .unwrap();
        let token = &response.data.tokens[0];
        assert!(token.token_address.is_none());
        assert_eq!(token.token_metadata.as_ref().unwrap().decimals, Some(18));
        assert_eq!(token.token_prices[0].value, "1.5");
        assert_eq!(response.data.page_key.as_deref(), Some("next"));
        let blocks = response.data.block_metadata.unwrap();
        assert!(blocks["base-mainnet"].is_none());
        assert_eq!(blocks["eth-mainnet"].as_ref().unwrap().block_number, "0x1");
        let error = response.error.unwrap();
        assert_eq!(error.partial_errors.len(), 1);
        assert_eq!(error.partial_errors[0].network, "base-mainnet");
    }

    #[test]
    fn tokens_by_address_request_uses_spec_fields() {
        let body = serde_json::to_value(TokensByAddressRequest {
            addresses: vec![AddressNetwork {
                address: WALLET.into(),
                networks: vec!["eth-mainnet".into()],
            }],
            with_metadata: Some(true),
            with_prices: Some(false),
            include_native_tokens: None,
            include_erc20_tokens: Some(true),
            include_block_metadata: None,
            page_key: None,
        })
        .unwrap();
        assert_eq!(
            body,
            serde_json::json!({
                "addresses": [{ "address": WALLET, "networks": ["eth-mainnet"] }],
                "withMetadata": true,
                "withPrices": false,
                "includeErc20Tokens": true
            })
        );
    }

    #[test]
    fn token_balances_keeps_page_key_and_partial_errors() {
        let response: TokenBalancesResponse = serde_json::from_value(serde_json::json!({
            "data": {
                "tokens": [{
                    "address": WALLET,
                    "network": "eth-mainnet",
                    "tokenAddress": null,
                    "tokenBalance": "0x02"
                }],
                "pageKey": "pk"
            },
            "error": {
                "message": "Failed to fetch tokens on certain networks",
                "partialErrors": [{ "network": "matic-mainnet", "message": "timeout" }]
            }
        }))
        .unwrap();
        assert_eq!(response.data.page_key.as_deref(), Some("pk"));
        assert_eq!(response.error.unwrap().partial_errors.len(), 1);

        let ok: TokenBalancesResponse =
            serde_json::from_value(serde_json::json!({ "data": { "tokens": [] } })).unwrap();
        assert!(ok.error.is_none() && ok.data.page_key.is_none());
    }

    #[test]
    fn nfts_by_address_parses_owned_nfts_envelope() {
        let response: NftsByAddressResponse = serde_json::from_value(serde_json::json!({
            "data": {
                "ownedNfts": [{
                    "network": "eth-mainnet",
                    "address": WALLET,
                    "contract": {
                        "address": NFT_CONTRACT,
                        "name": "Example",
                        "symbol": "EX",
                        "totalSupply": "10",
                        "tokenType": "ERC721"
                    },
                    "tokenId": "7",
                    "tokenType": "ERC721",
                    "name": "Example #7",
                    "tokenUri": "ipfs://example/7",
                    "timeLastUpdated": "2026-01-01T00:00:00Z"
                }],
                "totalCount": 1,
                "pageKey": "pk"
            },
            "error": {
                "message": "Failed to fetch NFTs on certain networks",
                "partialErrors": [{ "network": "base-mainnet", "message": "Internal server error" }]
            }
        }))
        .unwrap();
        let nft = &response.data.owned_nfts[0];
        assert_eq!(nft.contract.as_ref().unwrap().address, NFT_CONTRACT);
        assert_eq!(nft.token_id.as_deref(), Some("7"));
        assert_eq!(nft.token_type, Some(NftTokenType::Erc721));
        assert_eq!(response.data.total_count, Some(1));
        assert_eq!(response.data.page_key.as_deref(), Some("pk"));
        assert_eq!(response.error.unwrap().partial_errors.len(), 1);
    }

    #[test]
    fn nft_contracts_by_address_parses_contracts_envelope() {
        let response: NftContractsByAddressResponse = serde_json::from_value(serde_json::json!({
            "data": {
                "contracts": [{
                    "network": "eth-mainnet",
                    "address": WALLET,
                    "contract": { "address": NFT_CONTRACT, "tokenType": "ERC1155" }
                }],
                "totalCount": 1
            }
        }))
        .unwrap();
        let entry = &response.data.contracts[0];
        assert_eq!(entry.contract.as_ref().unwrap().address, NFT_CONTRACT);
        assert!(response.data.page_key.is_none());
        assert!(response.error.is_none());
    }
}
