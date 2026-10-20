//! Types for the Notify/Webhooks API
//!
//! The Notify API uses `snake_case` field names throughout.

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
    /// Polygon PoS mainnet (`MATIC_MAINNET`)
    #[serde(rename = "MATIC_MAINNET", alias = "POLYGON_MAINNET")]
    PolygonMainnet,
    /// Polygon Amoy testnet (`MATIC_AMOY`)
    #[serde(rename = "MATIC_AMOY", alias = "POLYGON_AMOY")]
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
    #[serde(default)]
    pub signing_key: Option<String>,
    /// Webhook version (`V1` or `V2`)
    #[serde(default)]
    pub version: Option<String>,
    /// Creation timestamp (milliseconds since the Unix epoch)
    #[serde(default)]
    pub time_created: Option<u64>,
    /// Webhook name
    #[serde(default)]
    pub name: Option<String>,
    /// App ID
    #[serde(default)]
    pub app_id: Option<String>,
}

/// `{ "data": ... }` envelope used by create/update responses
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DataEnvelope<T> {
    pub data: T,
}

/// Response for list webhooks
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListWebhooksResponse {
    pub data: Vec<Webhook>,
    #[serde(default)]
    pub total_count: Option<u64>,
}

/// Request to create a webhook
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookRequest {
    /// Network for the webhook
    pub network: WebhookNetwork,
    /// Type of webhook
    pub webhook_type: WebhookType,
    /// URL to receive webhook events
    pub webhook_url: String,
    /// Webhook name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Addresses to track (for `ADDRESS_ACTIVITY`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<String>>,
    /// NFT filters to track (for `NFT_ACTIVITY`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nft_filters: Option<Vec<NftFilter>>,
    /// Stringified GraphQL query (for GRAPHQL type)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graphql_query: Option<String>,
}

/// Request to update webhook addresses
///
/// Both lists are required by the API; send an empty list if there is
/// nothing to add or remove.
#[derive(Debug, Clone, Serialize)]
pub struct UpdateWebhookAddressesRequest {
    /// Webhook ID
    pub webhook_id: String,
    /// Addresses to add
    pub addresses_to_add: Vec<String>,
    /// Addresses to remove
    pub addresses_to_remove: Vec<String>,
}

/// Request to replace webhook addresses
#[derive(Debug, Clone, Serialize)]
pub struct ReplaceWebhookAddressesRequest {
    /// Webhook ID
    pub webhook_id: String,
    /// New list of addresses
    pub addresses: Vec<String>,
}

/// Request to update a webhook
#[derive(Debug, Clone, Serialize)]
pub struct UpdateWebhookRequest {
    /// Webhook ID
    pub webhook_id: String,
    /// New status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_active: Option<bool>,
    /// New webhook name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// NFT filter for webhooks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftFilter {
    /// Contract address
    pub contract_address: String,
    /// Token ID (optional, for specific NFTs)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_id: Option<String>,
}

/// Request to update NFT filters
#[derive(Debug, Clone, Serialize)]
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
pub struct ListAddressesResponse {
    pub data: Vec<String>,
    #[serde(default)]
    pub pagination: Option<PaginationInfo>,
    #[serde(default)]
    pub total_count: Option<u64>,
}

/// Pagination info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaginationInfo {
    #[serde(default)]
    pub cursors: Option<Cursors>,
    /// Total number of items
    #[serde(default)]
    pub total_count: Option<u64>,
}

/// Cursor info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Cursors {
    #[serde(default)]
    pub after: Option<String>,
    #[serde(default)]
    pub before: Option<String>,
}

/// Response for list NFT filters
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListNftFiltersResponse {
    pub data: Vec<NftFilter>,
    #[serde(default)]
    pub pagination: Option<PaginationInfo>,
    #[serde(default)]
    pub total_count: Option<u64>,
}

/// Elements of a Custom Webhook (GraphQL) variable
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GraphqlVariable {
    /// Variable elements (addresses or bytes32 values)
    pub data: Vec<String>,
    #[serde(default)]
    pub pagination: Option<PaginationInfo>,
}

/// Request to create a GraphQL variable (or append to an existing one)
#[derive(Debug, Clone, Serialize)]
pub struct GraphqlVariableRequest {
    /// Elements to store; must be non-empty
    pub items: Vec<String>,
}

/// Request to patch a GraphQL variable
#[derive(Debug, Clone, Default, Serialize)]
pub struct PatchGraphqlVariableRequest {
    /// Elements to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add: Option<Vec<String>>,
    /// Elements to delete
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delete: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn webhook_envelope_parses_snake_case() {
        let parsed: DataEnvelope<Webhook> = serde_json::from_value(json!({
            "data": {
                "id": "wh_test",
                "network": "MATIC_MAINNET",
                "webhook_type": "ADDRESS_ACTIVITY",
                "webhook_url": "https://example.com/hook",
                "is_active": true,
                "time_created": 1_700_000_000_000_u64,
                "version": "V2",
                "signing_key": "whsec_test"
            }
        }))
        .unwrap();
        let wh = parsed.data;
        assert_eq!(wh.id, "wh_test");
        assert_eq!(wh.network, WebhookNetwork::PolygonMainnet);
        assert_eq!(wh.webhook_type, WebhookType::AddressActivity);
        assert!(wh.is_active);
        assert_eq!(wh.time_created, Some(1_700_000_000_000));
        assert_eq!(wh.signing_key.as_deref(), Some("whsec_test"));
    }

    #[test]
    fn list_addresses_reads_nested_total_count() {
        let parsed: ListAddressesResponse = serde_json::from_value(json!({
            "data": ["0x0000000000000000000000000000000000000001"],
            "pagination": { "cursors": { "after": "cur1" }, "total_count": 1 }
        }))
        .unwrap();
        let pagination = parsed.pagination.unwrap();
        assert_eq!(pagination.total_count, Some(1));
        assert_eq!(pagination.cursors.unwrap().after.as_deref(), Some("cur1"));
    }

    #[test]
    fn requests_serialize_snake_case() {
        let create = serde_json::to_value(CreateWebhookRequest {
            network: WebhookNetwork::EthMainnet,
            webhook_type: WebhookType::NftActivity,
            webhook_url: "https://example.com/hook".into(),
            name: Some("test".into()),
            addresses: None,
            nft_filters: Some(vec![NftFilter {
                contract_address: "0xabc".into(),
                token_id: None,
            }]),
            graphql_query: None,
        })
        .unwrap();
        assert_eq!(
            create,
            json!({
                "network": "ETH_MAINNET",
                "webhook_type": "NFT_ACTIVITY",
                "webhook_url": "https://example.com/hook",
                "name": "test",
                "nft_filters": [{ "contract_address": "0xabc" }]
            })
        );

        let update = serde_json::to_value(UpdateWebhookAddressesRequest {
            webhook_id: "wh_test".into(),
            addresses_to_add: vec!["0x1".into()],
            addresses_to_remove: vec![],
        })
        .unwrap();
        assert_eq!(
            update,
            json!({
                "webhook_id": "wh_test",
                "addresses_to_add": ["0x1"],
                "addresses_to_remove": []
            })
        );

        let status = serde_json::to_value(UpdateWebhookRequest {
            webhook_id: "wh_test".into(),
            is_active: Some(false),
            name: None,
        })
        .unwrap();
        assert_eq!(
            status,
            json!({ "webhook_id": "wh_test", "is_active": false })
        );
    }

    #[test]
    fn graphql_variable_requests_use_spec_fields() {
        let create = serde_json::to_value(GraphqlVariableRequest {
            items: vec!["0x1".into()],
        })
        .unwrap();
        assert_eq!(create, json!({ "items": ["0x1"] }));

        let patch = serde_json::to_value(PatchGraphqlVariableRequest {
            add: Some(vec!["0x2".into()]),
            delete: Some(vec!["0x1".into()]),
        })
        .unwrap();
        assert_eq!(patch, json!({ "add": ["0x2"], "delete": ["0x1"] }));
    }
}
