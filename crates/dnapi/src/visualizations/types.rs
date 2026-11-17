//! Types for the Visualizations API

use serde::{Deserialize, Serialize};

/// A visualization attached to a query
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Visualization {
    /// Visualization ID
    pub id: i64,
    /// Query the visualization belongs to
    pub query_id: Option<i64>,
    /// Visualization type, e.g. `table`, `chart`, `counter`
    #[serde(rename = "type")]
    pub visualization_type: Option<String>,
    /// Name
    pub name: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Type-specific options
    pub options: Option<serde_json::Map<String, serde_json::Value>>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// A visualization as listed for a query
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VisualizationSummary {
    /// Visualization ID
    pub id: i64,
    /// Visualization type
    #[serde(rename = "type")]
    pub visualization_type: Option<String>,
    /// Name
    pub name: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Response from listing a query's visualizations
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ListVisualizationsResponse {
    /// Visualizations
    #[serde(default)]
    pub results: Vec<VisualizationSummary>,
    /// Total number of visualizations on the query
    pub total_count: Option<i64>,
}

/// Request to create or replace a visualization
///
/// `POST /v1/queries/{query_id}/visualizations` and
/// `PUT /v1/visualizations/{visualization_id}` take the same body.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct VisualizationRequest {
    /// Name
    pub name: String,
    /// Visualization type, e.g. `table`, `chart`, `counter`
    #[serde(rename = "type")]
    pub visualization_type: String,
    /// Description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Type-specific options
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<serde_json::Map<String, serde_json::Value>>,
}

impl VisualizationRequest {
    /// A visualization of `visualization_type` named `name`
    #[must_use]
    pub fn new(name: impl Into<String>, visualization_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visualization_type: visualization_type.into(),
            description: None,
            options: None,
        }
    }

    /// Set the description
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the type-specific options
    #[must_use]
    pub fn options(mut self, options: serde_json::Map<String, serde_json::Value>) -> Self {
        self.options = Some(options);
        self
    }
}

/// Response from creating or updating a visualization
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct VisualizationIdResponse {
    /// Visualization ID
    pub id: i64,
}

/// Response from deleting a visualization
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DeleteVisualizationResponse {
    /// Whether the visualization was deleted
    pub ok: Option<bool>,
}
