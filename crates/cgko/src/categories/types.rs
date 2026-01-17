//! Types for categories endpoints

use serde::{Deserialize, Serialize};

/// Category list item (basic info)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CategoryListItem {
    pub category_id: String,
    pub name: String,
}

/// Category with market data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub market_cap: Option<f64>,
    pub market_cap_change_24h: Option<f64>,
    pub content: Option<String>,
    pub top_3_coins: Option<Vec<String>>,
    pub volume_24h: Option<f64>,
    pub updated_at: Option<String>,
}
