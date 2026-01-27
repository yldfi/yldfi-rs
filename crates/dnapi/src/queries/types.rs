//! Types for the Queries API

use serde::{Deserialize, Serialize};

/// Query parameter definition
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    /// Parameter key/name
    pub key: String,
    /// Parameter type (text, number, date, enum, etc.)
    #[serde(rename = "type")]
    pub param_type: Option<String>,
    /// Default value
    pub value: Option<String>,
    /// Multiple values (for multiselect)
    #[serde(default)]
    pub values: Vec<String>,
    /// Description
    pub description: Option<String>,
    /// Enum options
    #[serde(default, rename = "enumOptions")]
    pub enum_options: Vec<String>,
    /// Allow freeform input for enum
    #[serde(rename = "isFreeformAllowed")]
    pub is_freeform_allowed: Option<bool>,
    /// Allow multiple selections
    #[serde(rename = "isMultiselect")]
    pub is_multiselect: Option<bool>,
}

/// Query response from get/create operations
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Query {
    /// Unique query ID
    pub query_id: i64,
    /// Query name
    pub name: String,
    /// Query description
    pub description: Option<String>,
    /// SQL query text
    pub query_sql: Option<String>,
    /// Query parameters
    #[serde(default)]
    pub parameters: Vec<Parameter>,
    /// Query owner (username or team handle)
    pub owner: Option<String>,
    /// Whether the query is private
    pub is_private: Option<bool>,
    /// Whether the query is archived
    pub is_archived: Option<bool>,
    /// Whether the query is unsaved
    pub is_unsaved: Option<bool>,
    /// Query engine (medium, large)
    pub query_engine: Option<String>,
    /// Query version
    pub version: Option<i64>,
    /// Tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Response from create query
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateQueryResponse {
    /// Created query ID
    pub query_id: i64,
}

/// Response from update query
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateQueryResponse {
    /// Updated query ID
    pub query_id: i64,
}

/// Request to create a query
#[derive(Debug, Clone, Serialize)]
pub struct CreateQueryRequest {
    /// Query name
    pub name: String,
    /// SQL query text
    pub query_sql: String,
    /// Query description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Query parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    /// Whether the query is private
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl CreateQueryRequest {
    /// Create a new query request
    #[must_use]
    pub fn new(name: &str, query_sql: &str) -> Self {
        Self {
            name: name.to_string(),
            query_sql: query_sql.to_string(),
            description: None,
            parameters: None,
            is_private: None,
            tags: None,
        }
    }

    /// Set the description
    #[must_use]
    pub fn description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set as private
    #[must_use]
    pub fn private(mut self, is_private: bool) -> Self {
        self.is_private = Some(is_private);
        self
    }

    /// Set tags
    #[must_use]
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = Some(tags);
        self
    }
}

/// Request to update a query
#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdateQueryRequest {
    /// Query name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// SQL query text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_sql: Option<String>,
    /// Query description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Query parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Vec<Parameter>>,
    /// Tags
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Response from list queries
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ListQueriesResponse {
    /// List of queries
    pub queries: Vec<Query>,
    /// Next page offset
    pub next_offset: Option<i64>,
}

/// Query list options
#[derive(Debug, Clone, Default)]
pub struct ListQueriesOptions {
    /// Maximum number of results
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<i64>,
}

impl ListQueriesOptions {
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
