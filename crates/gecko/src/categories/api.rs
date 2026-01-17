//! Categories API endpoints

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Categories API
pub struct CategoriesApi<'a> {
    client: &'a Client,
}

impl<'a> CategoriesApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all categories (id and name only)
    pub async fn list(&self) -> Result<Vec<CategoryListItem>> {
        self.client.get("/coins/categories/list").await
    }

    /// List categories with market data
    pub async fn with_market_data(&self) -> Result<Vec<Category>> {
        self.client.get("/coins/categories").await
    }

    /// List categories with market data, sorted
    ///
    /// # Arguments
    /// * `order` - Sort order: "market_cap_desc", "market_cap_asc", "name_desc", "name_asc",
    ///   "market_cap_change_24h_desc", "market_cap_change_24h_asc"
    pub async fn with_market_data_sorted(&self, order: &str) -> Result<Vec<Category>> {
        let path = format!("/coins/categories?order={}", order);
        self.client.get(&path).await
    }
}
