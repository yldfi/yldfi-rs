//! Entities API client

use super::types::{EntitySearchResult, Entity, CategoriesResponse};
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for entity endpoints
#[derive(Debug, Default, Serialize)]
pub struct EntityQuery {
    #[serde(skip_serializing_if = "Option::is_none", rename = "query")]
    pub search_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
}

impl EntityQuery {
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.search_query = Some(query.into());
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }
}

/// API for entity operations
pub struct EntitiesApi<'a> {
    client: &'a Client,
}

impl<'a> EntitiesApi<'a> {
    #[must_use] 
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Search for entities
    pub async fn search(&self, query: &EntityQuery) -> Result<EntitySearchResult> {
        self.client.get_with_query("/entities/search", query).await
    }

    /// Get entity by ID
    pub async fn get_entity(&self, entity_id: &str) -> Result<Entity> {
        let path = format!("/entities/{entity_id}");
        self.client.get(&path).await
    }

    /// Get all entity categories
    pub async fn get_categories(&self) -> Result<CategoriesResponse> {
        self.client.get("/entities/categories").await
    }

    /// Get entities in a category
    pub async fn get_category_entities(
        &self,
        category_id: &str,
        query: Option<&EntityQuery>,
    ) -> Result<EntitySearchResult> {
        let path = format!("/entities/categories/{category_id}");
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }
}
