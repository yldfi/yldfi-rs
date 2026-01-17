//! Types for the Materialized Views API

use serde::{Deserialize, Serialize};

/// Materialized view
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Matview {
    /// View ID (fully qualified name)
    pub id: Option<String>,
    /// Query ID
    pub query_id: Option<i64>,
    /// SQL ID
    pub sql_id: Option<String>,
    /// Whether the view is private
    pub is_private: Option<bool>,
    /// Table size in bytes
    pub table_size_bytes: Option<i64>,
}

/// Request to create/update a materialized view
#[derive(Debug, Clone, Serialize)]
pub struct UpsertMatviewRequest {
    /// View name
    pub name: String,
    /// Query ID to materialize
    pub query_id: i64,
    /// Cron expression for refresh schedule
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cron_expression: Option<String>,
    /// Expiration timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// Whether the view is private
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
    /// Performance tier (medium, large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
}

impl UpsertMatviewRequest {
    /// Create a new materialized view request
    pub fn new(name: &str, query_id: i64) -> Self {
        Self {
            name: name.to_string(),
            query_id,
            cron_expression: None,
            expires_at: None,
            is_private: None,
            performance: None,
        }
    }

    /// Set the cron expression for refresh schedule
    pub fn cron(mut self, cron_expression: &str) -> Self {
        self.cron_expression = Some(cron_expression.to_string());
        self
    }

    /// Set as private
    pub fn private(mut self, is_private: bool) -> Self {
        self.is_private = Some(is_private);
        self
    }

    /// Set performance tier to large
    pub fn large(mut self) -> Self {
        self.performance = Some("large".to_string());
        self
    }
}

/// Response from creating/updating a materialized view
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpsertMatviewResponse {
    /// View name
    pub name: Option<String>,
    /// Execution ID
    pub execution_id: Option<String>,
}

/// Request to refresh a materialized view
#[derive(Debug, Clone, Serialize, Default)]
pub struct RefreshMatviewRequest {
    /// Performance tier (medium, large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
}

/// Response from refreshing a materialized view
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RefreshMatviewResponse {
    /// Execution ID
    pub execution_id: Option<String>,
}

/// Response from deleting a materialized view
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeleteMatviewResponse {
    /// Success message
    pub message: Option<String>,
}

/// Response from listing materialized views
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListMatviewsResponse {
    /// List of materialized views
    #[serde(default)]
    pub materialized_views: Vec<Matview>,
    /// Next page offset
    pub next_offset: Option<i64>,
}

/// Options for listing materialized views
#[derive(Debug, Clone, Default)]
pub struct ListMatviewsOptions {
    /// Maximum number of results
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<i64>,
}

impl ListMatviewsOptions {
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={}", limit));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={}", offset));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
