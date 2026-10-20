//! Types for the Solana DAS API
//!
//! Request parameters are camelCase (`ownerAddress`, `displayOptions`, ...)
//! and are sent by name (a JSON object, not a positional array). Responses
//! follow the Metaplex DAS schema, whose fields are snake_case
//! (`json_uri`, `ownership_model`, `token_info`, ...).

use serde::{Deserialize, Deserializer, Serialize};

/// Asset display options
///
/// Sent as `displayOptions` to `getAsset`/`getAssets` and as `options` to the
/// list/search methods.
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
    /// Show unverified collections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_unverified_collections: Option<bool>,
    /// Show collection metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub show_collection_metadata: Option<bool>,
}

impl DisplayOptions {
    /// True when no option is set (nothing to send)
    pub fn is_empty(&self) -> bool {
        self.show_fungible.is_none()
            && self.show_native_balance.is_none()
            && self.show_inscription.is_none()
            && self.show_zero_balance.is_none()
            && self.show_unverified_collections.is_none()
            && self.show_collection_metadata.is_none()
    }
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
    /// Return results before this ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    /// Return results after this ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    /// Sort by
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<SortBy>,
}

/// Sort by options
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SortBy {
    /// `id`, `created`, `updated`, `recent_action` or `none`
    pub sort_by: String,
    /// `asc` or `desc`
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
pub struct Asset {
    /// Asset interface
    pub interface: Interface,
    /// Asset ID
    pub id: String,
    /// Content
    #[serde(default)]
    pub content: Option<AssetContent>,
    /// Authorities
    #[serde(default)]
    pub authorities: Vec<Authority>,
    /// Compression info
    #[serde(default)]
    pub compression: Option<Compression>,
    /// Grouping
    #[serde(default)]
    pub grouping: Vec<Grouping>,
    /// Royalty
    #[serde(default)]
    pub royalty: Option<Royalty>,
    /// Creators
    #[serde(default)]
    pub creators: Vec<Creator>,
    /// Ownership
    #[serde(default)]
    pub ownership: Option<Ownership>,
    /// Supply (for fungible)
    #[serde(default)]
    pub supply: Option<Supply>,
    /// Mutable
    #[serde(default)]
    pub mutable: Option<bool>,
    /// Burnt
    #[serde(default)]
    pub burnt: Option<bool>,
    /// Token info (for fungible)
    #[serde(default)]
    pub token_info: Option<TokenInfo>,
}

/// Asset content
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetContent {
    /// Schema
    #[serde(rename = "$schema", default)]
    pub schema: Option<String>,
    /// JSON URI
    #[serde(default)]
    pub json_uri: Option<String>,
    /// Files
    #[serde(default)]
    pub files: Vec<AssetFile>,
    /// Metadata
    #[serde(default)]
    pub metadata: Option<AssetMetadata>,
    /// Links
    #[serde(default)]
    pub links: Option<AssetLinks>,
}

/// Asset file
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetFile {
    /// URI
    #[serde(default)]
    pub uri: Option<String>,
    /// CDN URI
    #[serde(default)]
    pub cdn_uri: Option<String>,
    /// MIME type
    #[serde(default)]
    pub mime: Option<String>,
}

/// Asset metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetMetadata {
    /// Name
    #[serde(default)]
    pub name: Option<String>,
    /// Symbol
    #[serde(default)]
    pub symbol: Option<String>,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Attributes
    #[serde(default)]
    pub attributes: Vec<AssetAttribute>,
}

/// Asset attribute
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetAttribute {
    #[serde(default)]
    pub trait_type: Option<String>,
    #[serde(default)]
    pub value: Option<serde_json::Value>,
}

/// Asset links
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetLinks {
    #[serde(default)]
    pub external_url: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub animation_url: Option<String>,
}

/// Authority
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Authority {
    pub address: String,
    #[serde(default)]
    pub scopes: Vec<String>,
}

/// Compression info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Compression {
    #[serde(default)]
    pub eligible: bool,
    #[serde(default)]
    pub compressed: bool,
    #[serde(default)]
    pub data_hash: Option<String>,
    #[serde(default)]
    pub creator_hash: Option<String>,
    #[serde(default)]
    pub asset_hash: Option<String>,
    #[serde(default)]
    pub tree: Option<String>,
    #[serde(default)]
    pub seq: Option<u64>,
    #[serde(default)]
    pub leaf_id: Option<u64>,
}

/// Grouping
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Grouping {
    pub group_key: String,
    pub group_value: String,
}

/// Royalty
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Royalty {
    #[serde(default)]
    pub royalty_model: Option<String>,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub percent: Option<f64>,
    #[serde(default)]
    pub basis_points: Option<u64>,
    #[serde(default)]
    pub primary_sale_happened: Option<bool>,
    #[serde(default)]
    pub locked: Option<bool>,
}

/// Creator
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Creator {
    pub address: String,
    #[serde(default)]
    pub share: u8,
    #[serde(default)]
    pub verified: bool,
}

/// Ownership
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Ownership {
    #[serde(default)]
    pub frozen: bool,
    #[serde(default)]
    pub delegated: bool,
    #[serde(default)]
    pub delegate: Option<String>,
    #[serde(default)]
    pub ownership_model: String,
    #[serde(default)]
    pub owner: String,
}

/// Supply
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Supply {
    #[serde(default)]
    pub print_max_supply: Option<u64>,
    #[serde(default)]
    pub print_current_supply: Option<u64>,
    #[serde(default)]
    pub edition_nonce: Option<u64>,
}

/// Token info (for fungible tokens)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenInfo {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub balance: Option<u64>,
    #[serde(default)]
    pub supply: Option<u64>,
    #[serde(default)]
    pub decimals: Option<u8>,
    #[serde(default)]
    pub token_program: Option<String>,
    #[serde(default)]
    pub associated_token_address: Option<String>,
    #[serde(default)]
    pub price_info: Option<PriceInfo>,
}

/// Price info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PriceInfo {
    #[serde(default)]
    pub price_per_token: Option<f64>,
    #[serde(default)]
    pub total_price: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
}

/// Asset proof
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AssetProof {
    pub root: String,
    pub proof: Vec<String>,
    pub node_index: u64,
    pub leaf: String,
    pub tree_id: String,
}

/// Get assets response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAssetsResponse {
    #[serde(default)]
    pub items: Vec<Asset>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub cursor: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub after: Option<String>,
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
    /// Display options, sent as `options`
    #[serde(rename = "options", skip_serializing_if = "DisplayOptions::is_empty")]
    pub display: DisplayOptions,
}

/// Token account
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TokenAccount {
    pub address: String,
    pub mint: String,
    pub owner: String,
    #[serde(default)]
    pub amount: u64,
    #[serde(default)]
    pub delegated_amount: Option<u64>,
    #[serde(default)]
    pub frozen: bool,
}

/// Get token accounts response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetTokenAccountsResponse {
    #[serde(default)]
    pub token_accounts: Vec<TokenAccount>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub cursor: Option<String>,
}

/// NFT edition (Metaplex DAS shape)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NftEdition {
    /// Edition mint
    pub mint: String,
    /// Edition account address
    #[serde(default)]
    pub edition_address: Option<String>,
    /// Edition number
    #[serde(default, alias = "edition_number")]
    pub edition: Option<u64>,
}

/// Get NFT editions response
///
/// The Metaplex DAS schema lists editions under `editions`; Alchemy's own
/// docs describe an `items` list of edition assets. Both are kept.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetNftEditionsResponse {
    #[serde(default)]
    pub editions: Vec<NftEdition>,
    #[serde(default)]
    pub items: Vec<serde_json::Value>,
    #[serde(default)]
    pub master_edition_address: Option<String>,
    #[serde(default)]
    pub supply: Option<u64>,
    #[serde(default)]
    pub max_supply: Option<u64>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub page: Option<u32>,
}

/// Asset signature (transaction touching a compressed asset)
///
/// Metaplex DAS returns each item as a `[signature, instruction]` tuple;
/// Alchemy's docs describe an object (`signature`, `slot`, `instruction`, ...).
/// Both forms are accepted.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSignature {
    /// Transaction signature
    pub signature: String,
    /// Instruction type (e.g. `MintToCollectionV1`, `Transfer`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruction: Option<String>,
    /// Slot
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<u64>,
    /// Block time
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_time: Option<i64>,
}

impl<'de> Deserialize<'de> for AssetSignature {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            signature: String,
            #[serde(default)]
            instruction: Option<String>,
            #[serde(default)]
            slot: Option<u64>,
            #[serde(default)]
            block_time: Option<i64>,
        }

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Tuple(String, Option<String>),
            Object(Object),
        }

        Ok(match Repr::deserialize(deserializer)? {
            Repr::Tuple(signature, instruction) => AssetSignature {
                signature,
                instruction,
                slot: None,
                block_time: None,
            },
            Repr::Object(o) => AssetSignature {
                signature: o.signature,
                instruction: o.instruction,
                slot: o.slot,
                block_time: o.block_time,
            },
        })
    }
}

/// Get asset signatures response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GetAssetSignaturesResponse {
    #[serde(default)]
    pub items: Vec<AssetSignature>,
    #[serde(default)]
    pub total: Option<u64>,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub page: Option<u32>,
    #[serde(default)]
    pub before: Option<String>,
    #[serde(default)]
    pub after: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asset_snake_case_response_parses() {
        let json = r#"{
            "interface": "V1_NFT",
            "id": "Asset1111111111111111111111111111111111111",
            "content": {
                "$schema": "https://schema.example/v1",
                "json_uri": "https://example.com/meta.json",
                "files": [{"uri": "https://example.com/a.png", "cdn_uri": "https://cdn.example.com/a.png", "mime": "image/png"}],
                "metadata": {"name": "Test", "symbol": "TST", "attributes": [{"trait_type": "k", "value": 1}]},
                "links": {"external_url": "https://example.com", "image": "https://example.com/a.png"}
            },
            "authorities": [{"address": "Auth111", "scopes": ["full"]}],
            "compression": {"eligible": false, "compressed": true, "data_hash": "dh", "tree": "Tree111", "seq": 5, "leaf_id": 7},
            "grouping": [{"group_key": "collection", "group_value": "Coll111"}],
            "royalty": {"royalty_model": "creators", "target": null, "percent": 0.05, "basis_points": 500, "primary_sale_happened": true, "locked": false},
            "creators": [{"address": "Creator111", "share": 100, "verified": true}],
            "ownership": {"frozen": false, "delegated": false, "delegate": null, "ownership_model": "single", "owner": "Owner111"},
            "supply": {"print_max_supply": 0, "print_current_supply": 0, "edition_nonce": 255},
            "mutable": true,
            "burnt": false,
            "token_info": {"symbol": "TST", "balance": 10, "decimals": 6, "token_program": "Prog111", "price_info": {"price_per_token": 1.5, "currency": "USDC"}}
        }"#;
        let asset: Asset = serde_json::from_str(json).unwrap();
        let content = asset.content.unwrap();
        assert_eq!(
            content.json_uri.as_deref(),
            Some("https://example.com/meta.json")
        );
        assert_eq!(
            content.files[0].cdn_uri.as_deref(),
            Some("https://cdn.example.com/a.png")
        );
        assert_eq!(asset.compression.unwrap().leaf_id, Some(7));
        assert_eq!(asset.grouping[0].group_value, "Coll111");
        assert_eq!(asset.royalty.unwrap().basis_points, Some(500));
        assert_eq!(asset.ownership.unwrap().ownership_model, "single");
        assert_eq!(asset.supply.unwrap().edition_nonce, Some(255));
        let token = asset.token_info.unwrap();
        assert_eq!(token.token_program.as_deref(), Some("Prog111"));
        assert_eq!(token.price_info.unwrap().price_per_token, Some(1.5));
    }

    #[test]
    fn asset_proof_and_token_accounts_parse() {
        let proof: AssetProof = serde_json::from_str(
            r#"{"root":"r","proof":["p1"],"node_index":3,"leaf":"l","tree_id":"t"}"#,
        )
        .unwrap();
        assert_eq!(proof.node_index, 3);
        assert_eq!(proof.tree_id, "t");

        let accounts: GetTokenAccountsResponse = serde_json::from_str(
            r#"{"total":1,"limit":1,"cursor":null,"token_accounts":[{"address":"A","mint":"M","owner":"O","amount":42,"delegated_amount":0,"frozen":false}]}"#,
        )
        .unwrap();
        assert_eq!(accounts.token_accounts[0].amount, 42);
        assert_eq!(accounts.token_accounts[0].delegated_amount, Some(0));
    }

    #[test]
    fn asset_signatures_accept_tuples_and_objects() {
        let tuples: GetAssetSignaturesResponse = serde_json::from_str(
            r#"{"total":2,"limit":2,"page":1,"items":[["sig1","MintToCollectionV1"],["sig2","Transfer"]]}"#,
        )
        .unwrap();
        assert_eq!(tuples.items[0].signature, "sig1");
        assert_eq!(tuples.items[1].instruction.as_deref(), Some("Transfer"));

        let objects: GetAssetSignaturesResponse = serde_json::from_str(
            r#"{"items":[{"signature":"sig3","slot":9,"blockTime":null,"instruction":"Burn"}]}"#,
        )
        .unwrap();
        assert_eq!(objects.items[0].slot, Some(9));
        assert_eq!(objects.items[0].instruction.as_deref(), Some("Burn"));
    }

    #[test]
    fn nft_editions_parse() {
        let resp: GetNftEditionsResponse = serde_json::from_str(
            r#"{"total":1,"limit":1,"page":1,"master_edition_address":"ME","supply":1,"max_supply":10,
                "editions":[{"mint":"E1","edition_address":"EA","edition":1}]}"#,
        )
        .unwrap();
        assert_eq!(resp.editions[0].edition, Some(1));
        assert_eq!(resp.master_edition_address.as_deref(), Some("ME"));
    }

    #[test]
    fn search_request_nests_options_and_camel_case_sort() {
        let req = SearchAssetsRequest {
            owner_address: Some("Owner111".into()),
            pagination: PaginationOptions {
                limit: Some(10),
                sort_by: Some(SortBy {
                    sort_by: "created".into(),
                    sort_direction: "asc".into(),
                }),
                ..Default::default()
            },
            display: DisplayOptions {
                show_fungible: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({
                "ownerAddress": "Owner111",
                "limit": 10,
                "sortBy": {"sortBy": "created", "sortDirection": "asc"},
                "options": {"showFungible": true}
            })
        );
    }
}
