//! Types for the Entities API

use serde::{Deserialize, Serialize};

/// Entity (wallet, protocol, exchange, etc)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity ID
    pub id: Option<String>,
    /// Entity name
    pub name: Option<String>,
    /// Entity description
    pub description: Option<String>,
    /// Entity logo
    pub logo: Option<String>,
    /// Category ID
    pub category_id: Option<String>,
    /// Category name
    pub category_name: Option<String>,
    /// Addresses
    pub addresses: Option<Vec<EntityAddress>>,
}

/// Entity address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAddress {
    /// Address
    pub address: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// Label
    pub label: Option<String>,
}

/// Entity category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityCategory {
    /// Category ID
    pub id: Option<String>,
    /// Category name
    pub name: Option<String>,
    /// Category description
    pub description: Option<String>,
    /// Entity count in category
    pub entity_count: Option<i64>,
}

/// Entity search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySearchResult {
    /// Cursor
    pub cursor: Option<String>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    pub result: Vec<Entity>,
}

/// Categories response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriesResponse {
    /// Categories
    pub result: Vec<EntityCategory>,
}
