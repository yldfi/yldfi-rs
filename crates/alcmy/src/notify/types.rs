//! Types for the Notify/Webhooks API

use serde::{Deserialize, Serialize};

/// Webhook type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookType {
    /// Mined transactions
    MinedTransaction,
    /// Dropped transactions
    DroppedTransaction,
    /// Address activity
    AddressActivity,
    /// NFT activity
    NftActivity,
    /// NFT metadata updates
    NftMetadataUpdate,
    /// Custom GraphQL webhook
    Graphql,
}

/// Webhook network
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookNetwork {
    EthMainnet,
    EthSepolia,
    EthHolesky,
    PolygonMainnet,
    PolygonAmoy,
    ArbMainnet,
    ArbSepolia,
    OptMainnet,
    OptSepolia,
    BaseMainnet,
    BaseSepolia,
    ZksyncMainnet,
    ZksyncSepolia,
    #[serde(other)]
    Other,
}

/// Webhook status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebhookStatus {
    Active,
    Inactive,
}

/// Webhook definition
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Webhook {
    /// Webhook ID
    pub id: String,
    /// Webhook network
    pub network: WebhookNetwork,
    /// Webhook type
    pub webhook_type: WebhookType,
    /// Webhook URL
    pub webhook_url: String,
    /// Whether the webhook is active
    pub is_active: bool,
    /// Signing key for webhook verification
    pub signing_key: Option<String>,
    /// Webhook version
    pub version: Option<String>,
    /// Time-to-live in seconds
    pub time_created: Option<String>,
    /// App ID
    pub app_id: Option<String>,
}

/// Response for list webhooks
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListWebhooksResponse {
    pub data: Vec<Webhook>,
    pub total_count: Option<u64>,
}

/// Request to create a webhook
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhookRequest {
    /// Network for the webhook
    pub network: WebhookNetwork,
    /// Type of webhook
    pub webhook_type: WebhookType,
    /// URL to receive webhook events
    pub webhook_url: String,
    /// Addresses to track (for `ADDRESS_ACTIVITY`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,
    /// GraphQL query (for GRAPHQL type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphql_query: Option<String>,
}

/// Request to update webhook addresses
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWebhookAddressesRequest {
    /// Webhook ID
    pub webhook_id: String,
    /// Addresses to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses_to_add: Option<Vec<String>>,
    /// Addresses to remove
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses_to_remove: Option<Vec<String>>,
}

/// Request to replace webhook addresses
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceWebhookAddressesRequest {
    /// Webhook ID
    pub webhook_id: String,
    /// New list of addresses
    pub addresses: Vec<String>,
}

/// Request to update a webhook
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWebhookRequest {
    /// Webhook ID
    pub webhook_id: String,
    /// New webhook URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhook_url: Option<String>,
    /// New status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
}

/// NFT filter for webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NftFilter {
    /// Contract address
    pub contract_address: String,
    /// Token ID (optional, for specific NFTs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
}

/// Request to update NFT filters
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNftFiltersRequest {
    /// Webhook ID
    pub webhook_id: String,
    /// Filters to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nft_filters_to_add: Option<Vec<NftFilter>>,
    /// Filters to remove
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nft_filters_to_remove: Option<Vec<NftFilter>>,
}

/// Response for list addresses
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAddressesResponse {
    pub data: Vec<String>,
    pub pagination: Option<PaginationInfo>,
    pub total_count: Option<u64>,
}

/// Pagination info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    pub cursors: Option<Cursors>,
}

/// Cursor info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cursors {
    pub after: Option<String>,
    pub before: Option<String>,
}

/// Response for list NFT filters
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListNftFiltersResponse {
    pub data: Vec<NftFilter>,
    pub pagination: Option<PaginationInfo>,
    pub total_count: Option<u64>,
}

/// GraphQL variable
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlVariable {
    pub name: String,
    pub values: Vec<String>,
}

/// Request to create/update a GraphQL variable
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphqlVariableRequest {
    /// Variable name
    pub variable: String,
    /// Values
    pub values: Vec<String>,
}

/// Request to patch a GraphQL variable
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PatchGraphqlVariableRequest {
    /// Values to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values_to_add: Option<Vec<String>>,
    /// Values to remove
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values_to_remove: Option<Vec<String>>,
}
