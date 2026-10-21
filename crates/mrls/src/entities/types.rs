//! Types for the Entities API

use crate::de::string_or_int;
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
    #[serde(default, deserialize_with = "string_or_int")]
    pub id: Option<String>,
    /// Category name
    pub name: Option<String>,
    /// Category description
    pub description: Option<String>,
    /// Entity count in category
    pub entity_count: Option<i64>,
}

/// Entity search response (`GET /entities/search`)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySearchResult {
    /// Page
    pub page: Option<i32>,
    /// Page size
    pub page_size: Option<i32>,
    /// Matches, grouped by kind
    pub result: EntitySearchMatches,
}

/// Entity search matches
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntitySearchMatches {
    /// Matching entities
    #[serde(default)]
    pub entities: Vec<SearchEntity>,
    /// Matching addresses
    #[serde(default)]
    pub addresses: Vec<SearchAddress>,
    /// Matching categories
    #[serde(default)]
    pub categories: Vec<EntityCategory>,
}

/// Entity in a search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchEntity {
    /// Entity ID
    #[serde(default, deserialize_with = "string_or_int")]
    pub id: Option<String>,
    /// Entity name
    pub name: Option<String>,
    /// Entity type
    #[serde(rename = "type")]
    pub entity_type: Option<String>,
    /// Bio
    pub bio: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Logo
    pub logo: Option<String>,
    /// Website
    pub website: Option<String>,
    /// Twitter
    pub twitter: Option<String>,
}

/// Address in a search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAddress {
    /// Address
    pub address: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// Primary label
    pub primary_label: Option<String>,
    /// Whether the address is the same on several chains
    pub is_multi_chain: Option<bool>,
    /// Owning entity
    pub entity: Option<SearchAddressEntity>,
}

/// Entity reference on a search address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAddressEntity {
    /// Entity ID
    #[serde(default, deserialize_with = "string_or_int")]
    pub id: Option<String>,
    /// Entity name
    pub name: Option<String>,
}

/// Categories response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoriesResponse {
    /// Categories
    pub result: Vec<EntityCategory>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entity_search_result_is_grouped_object() {
        let json = r#"{
            "page": 1,
            "page_size": 10,
            "result": {
                "entities": [{"id": "test-entity", "name": "Test Entity", "type": "exchange"}],
                "addresses": [{
                    "address": "0x0000000000000000000000000000000000000001",
                    "chain": "eth",
                    "primary_label": "Test Hot Wallet",
                    "is_multi_chain": false,
                    "entity": {"id": "test-entity", "name": "Test Entity"}
                }],
                "categories": [{"id": 7, "name": "exchanges"}]
            }
        }"#;
        let resp: EntitySearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(
            resp.result.entities[0].entity_type.as_deref(),
            Some("exchange")
        );
        let addr = &resp.result.addresses[0];
        assert_eq!(
            addr.entity.as_ref().unwrap().name.as_deref(),
            Some("Test Entity")
        );
        assert_eq!(resp.result.categories[0].id.as_deref(), Some("7"));
    }
}
