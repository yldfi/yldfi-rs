//! Types for webhooks

use serde::{Deserialize, Serialize};

/// Webhooks list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebhooksListResponse {
    /// Webhooks
    pub webhooks: Vec<Webhook>,
    /// Pagination cursor
    pub next_offset: Option<String>,
}

/// Webhook
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Webhook {
    /// Webhook ID
    pub id: String,
    /// Team ID
    pub team_id: Option<String>,
    /// Webhook name
    pub name: String,
    /// Callback URL
    pub url: String,
    /// Webhook type (transactions, activities, balances)
    #[serde(rename = "type")]
    pub webhook_type: String,
    /// Is active
    pub active: bool,
    /// Chain IDs (null if not filtered)
    pub chain_ids: Option<Vec<i64>>,
    /// Transaction type filter (sender, receiver)
    pub transaction_type: Option<String>,
    /// Counterparty address filter
    pub counterparty: Option<String>,
    /// Activity type filter
    pub activity_type: Option<String>,
    /// Asset type filter (native, erc20, erc721, erc1155)
    pub asset_type: Option<String>,
    /// Token address filter
    pub token_address: Option<String>,
    /// Created at
    pub created_at: Option<String>,
    /// Updated at
    pub updated_at: Option<String>,
}

/// Create webhook request
#[derive(Debug, Clone, Serialize)]
pub struct CreateWebhookRequest {
    /// Webhook name
    pub name: String,
    /// Callback URL
    pub url: String,
    /// Webhook type (transactions, activities, balances)
    #[serde(rename = "type")]
    pub webhook_type: String,
    /// Addresses to watch
    pub addresses: Vec<String>,
    /// Chain IDs (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_ids: Option<Vec<i64>>,
    /// Transaction type filter (sender, receiver) - transactions only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<String>,
    /// Counterparty address filter - transactions only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterparty: Option<String>,
    /// Activity type filter - activities only
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    /// Asset type filter (native, erc20, erc721, erc1155)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
    /// Token address filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_address: Option<String>,
}

impl CreateWebhookRequest {
    /// Create a new webhook request for transactions
    #[must_use] 
    pub fn transactions(name: &str, url: &str, addresses: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            webhook_type: "transactions".to_string(),
            addresses,
            chain_ids: None,
            transaction_type: None,
            counterparty: None,
            activity_type: None,
            asset_type: None,
            token_address: None,
        }
    }

    /// Create a new webhook request for activities
    #[must_use] 
    pub fn activities(name: &str, url: &str, addresses: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            webhook_type: "activities".to_string(),
            addresses,
            chain_ids: None,
            transaction_type: None,
            counterparty: None,
            activity_type: None,
            asset_type: None,
            token_address: None,
        }
    }

    /// Create a new webhook request for balances
    #[must_use] 
    pub fn balances(name: &str, url: &str, addresses: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            url: url.to_string(),
            webhook_type: "balances".to_string(),
            addresses,
            chain_ids: None,
            transaction_type: None,
            counterparty: None,
            activity_type: None,
            asset_type: None,
            token_address: None,
        }
    }
}

/// Update webhook request
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateWebhookRequest {
    /// Webhook name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Callback URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// Is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,
    /// Chain IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_ids: Option<Vec<i64>>,
    /// Transaction type filter (sender, receiver)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transaction_type: Option<String>,
    /// Counterparty address filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub counterparty: Option<String>,
    /// Activity type filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activity_type: Option<String>,
    /// Asset type filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_type: Option<String>,
    /// Token address filter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_address: Option<String>,
}

impl UpdateWebhookRequest {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }
}

/// Addresses list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddressesListResponse {
    /// Addresses
    pub addresses: Vec<String>,
    /// Pagination cursor
    pub next_offset: Option<String>,
}

/// Replace addresses request
#[derive(Debug, Clone, Serialize)]
pub struct ReplaceAddressesRequest {
    /// Addresses to replace with
    pub addresses: Vec<String>,
}

/// Update addresses request
#[derive(Debug, Clone, Default, Serialize)]
pub struct UpdateAddressesRequest {
    /// Addresses to add
    #[serde(skip_serializing_if = "Option::is_none")]
    pub add_addresses: Option<Vec<String>>,
    /// Addresses to remove
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remove_addresses: Option<Vec<String>>,
}

impl UpdateAddressesRequest {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use] 
    pub fn add(addresses: Vec<String>) -> Self {
        Self {
            add_addresses: Some(addresses),
            remove_addresses: None,
        }
    }

    #[must_use] 
    pub fn remove(addresses: Vec<String>) -> Self {
        Self {
            add_addresses: None,
            remove_addresses: Some(addresses),
        }
    }
}

/// Query options for webhooks list
#[derive(Debug, Clone, Default)]
pub struct WebhooksListOptions {
    /// Results limit
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<String>,
}

impl WebhooksListOptions {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use] 
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(ref offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Query options for addresses list
#[derive(Debug, Clone, Default)]
pub struct AddressesListOptions {
    /// Results limit
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<String>,
}

impl AddressesListOptions {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use] 
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(ref offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
