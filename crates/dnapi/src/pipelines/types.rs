//! Types for the Pipelines API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pipeline execution request
#[derive(Debug, Clone, Serialize)]
pub struct ExecutePipelineRequest {
    /// Pipeline definition
    pub pipeline: Pipeline,
    /// Query parameters for all queries in the pipeline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<HashMap<String, String>>,
    /// Performance tier (medium, large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
}

/// Pipeline definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    /// Pipeline nodes
    pub nodes: Vec<PipelineNode>,
}

/// Pipeline node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineNode {
    /// Node type (query_execution, materialized_view_refresh)
    #[serde(rename = "type")]
    pub node_type: String,
    /// Query ID (for query_execution type)
    pub query_id: Option<i64>,
    /// Materialized view name (for materialized_view_refresh type)
    pub matview_name: Option<String>,
    /// Dependencies (IDs of nodes that must complete first)
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// Node ID
    pub id: Option<String>,
}

impl PipelineNode {
    /// Create a query execution node
    pub fn query(query_id: i64) -> Self {
        Self {
            node_type: "query_execution".to_string(),
            query_id: Some(query_id),
            matview_name: None,
            depends_on: vec![],
            id: None,
        }
    }

    /// Create a materialized view refresh node
    pub fn matview(name: &str) -> Self {
        Self {
            node_type: "materialized_view_refresh".to_string(),
            query_id: None,
            matview_name: Some(name.to_string()),
            depends_on: vec![],
            id: None,
        }
    }

    /// Set node ID
    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    /// Set dependencies
    pub fn depends_on(mut self, deps: Vec<String>) -> Self {
        self.depends_on = deps;
        self
    }
}

/// Response from executing a pipeline
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutePipelineResponse {
    /// Pipeline execution ID
    pub pipeline_execution_id: Option<String>,
    /// Node executions
    #[serde(default)]
    pub node_executions: Vec<NodeExecution>,
}

/// Node execution info
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeExecution {
    /// Node ID
    pub node_id: Option<String>,
    /// Execution ID
    pub execution_id: Option<String>,
    /// State
    pub state: Option<String>,
}

/// Pipeline execution status
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipelineExecutionStatus {
    /// Pipeline execution ID
    pub pipeline_execution_id: Option<String>,
    /// Overall state
    pub state: Option<String>,
    /// Whether execution is finished
    pub is_execution_finished: Option<bool>,
    /// Node statuses
    #[serde(default)]
    pub node_executions: Vec<NodeExecutionStatus>,
}

/// Node execution status
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NodeExecutionStatus {
    /// Node ID
    pub node_id: Option<String>,
    /// Node type
    #[serde(rename = "type")]
    pub node_type: Option<String>,
    /// Execution ID
    pub execution_id: Option<String>,
    /// State
    pub state: Option<String>,
    /// Query ID
    pub query_id: Option<i64>,
    /// Materialized view name
    pub matview_name: Option<String>,
}
