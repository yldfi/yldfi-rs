//! Types for the Pipelines API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Pipeline execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutePipelineRequest {
    /// Pipeline definition
    pub pipeline: Pipeline,
}

/// Pipeline definition
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Pipeline {
    /// Pipeline nodes, executed in dependency order
    #[serde(default)]
    pub nodes: Vec<PipelineNode>,
}

impl Pipeline {
    /// Set query parameters on every query execution node that has none
    #[must_use]
    pub fn with_query_parameters(mut self, params: &HashMap<String, serde_json::Value>) -> Self {
        for exec in self
            .nodes
            .iter_mut()
            .filter_map(|n| n.query_execution.as_mut())
        {
            if exec.query_parameters.is_none() {
                exec.query_parameters = Some(params.clone());
            }
        }
        self
    }

    /// Set the performance tier on every node that has none
    #[must_use]
    pub fn with_performance(mut self, performance: &str) -> Self {
        for node in &mut self.nodes {
            if let Some(exec) = node.query_execution.as_mut() {
                exec.performance
                    .get_or_insert_with(|| performance.to_string());
            }
            if let Some(refresh) = node.materialized_view_refresh.as_mut() {
                refresh
                    .performance
                    .get_or_insert_with(|| performance.to_string());
            }
        }
        self
    }
}

/// Pipeline node: either a query execution or a materialized view refresh
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineNode {
    /// Node ID, unique within the pipeline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// IDs of nodes that must complete first
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dependencies: Vec<i64>,
    /// Query execution configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_execution: Option<PipelineQueryExecution>,
    /// Materialized view refresh configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub materialized_view_refresh: Option<PipelineMatviewRefresh>,
}

/// Query execution node configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineQueryExecution {
    /// Query ID
    pub query_id: i64,
    /// Performance tier (small, medium, large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
    /// Query parameters as key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<HashMap<String, serde_json::Value>>,
}

/// Materialized view refresh node configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PipelineMatviewRefresh {
    /// Materialized view name
    pub name: String,
    /// Performance tier (small, medium, large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
}

impl PipelineNode {
    /// Create a query execution node
    #[must_use]
    pub fn query(id: i64, query_id: i64) -> Self {
        Self {
            id: Some(id),
            query_execution: Some(PipelineQueryExecution {
                query_id,
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Create a materialized view refresh node
    #[must_use]
    pub fn matview(id: i64, name: &str) -> Self {
        Self {
            id: Some(id),
            materialized_view_refresh: Some(PipelineMatviewRefresh {
                name: name.to_string(),
                performance: None,
            }),
            ..Default::default()
        }
    }

    /// Set dependencies
    #[must_use]
    pub fn depends_on(mut self, deps: Vec<i64>) -> Self {
        self.dependencies = deps;
        self
    }
}

/// Response from executing a pipeline
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutePipelineResponse {
    /// Pipeline execution ID
    pub pipeline_execution_id: Option<String>,
}

/// A query's pipeline: the query and every materialized view it depends on
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct QueryPipeline {
    /// The pipeline definition, ready for
    /// [`PipelinesApi::execute`](super::PipelinesApi::execute)
    #[serde(default)]
    pub pipeline: Pipeline,
}

/// Options for building a query's pipeline
/// (`GET /v1/query/{query_id}/pipeline`)
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct QueryPipelineOptions {
    /// Performance tier for the nodes (`small`, `medium` or `large`)
    pub performance: Option<String>,
    /// Query parameters for the query execution nodes
    pub query_parameters: Option<HashMap<String, serde_json::Value>>,
}

impl QueryPipelineOptions {
    /// Options with nothing set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Query pairs for the request; the parameters go as one JSON object
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(ref performance) = self.performance {
            params.push(("performance", performance.clone()));
        }
        if let Some(ref query_parameters) = self.query_parameters {
            let object: serde_json::Map<String, serde_json::Value> = query_parameters
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            params.push((
                "query_parameters",
                serde_json::Value::Object(object).to_string(),
            ));
        }
        params
    }
}

/// Request to build and execute a query's pipeline
/// (`POST /v1/query/{query_id}/pipeline/execute`)
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ExecuteQueryPipelineRequest {
    /// Performance tier (`small`, `medium` or `large`)
    pub performance: String,
    /// Query parameters as key-value pairs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<HashMap<String, serde_json::Value>>,
}

impl ExecuteQueryPipelineRequest {
    /// Execute at the `performance` tier
    #[must_use]
    pub fn new(performance: impl Into<String>) -> Self {
        Self {
            performance: performance.into(),
            query_parameters: None,
        }
    }

    /// Set a query parameter
    #[must_use]
    pub fn param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.query_parameters
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value);
        self
    }
}

/// Pipeline execution status
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipelineExecutionStatus {
    /// Overall status
    pub status: Option<String>,
    /// Error message if the pipeline failed
    pub error: Option<String>,
    /// Node statuses
    #[serde(default)]
    pub node_executions: Vec<PipelineNodeExecution>,
}

/// Status of one pipeline node
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipelineNodeExecution {
    /// Node ID
    pub id: Option<i64>,
    /// Query execution status (query nodes)
    pub query_execution_status: Option<PipelineQueryExecutionStatus>,
    /// Materialized view refresh status (matview nodes)
    pub materialized_view_refresh_status: Option<PipelineMatviewRefreshStatus>,
}

/// Query execution status within a pipeline
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipelineQueryExecutionStatus {
    /// Status
    pub status: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Query ID
    pub query_id: Option<i64>,
    /// Execution ID
    pub execution_id: Option<String>,
}

/// Materialized view refresh status within a pipeline
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PipelineMatviewRefreshStatus {
    /// Status
    pub status: Option<String>,
    /// Error message
    pub error: Option<String>,
    /// Materialized view name
    pub name: Option<String>,
    /// Execution ID
    pub execution_id: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_request_uses_spec_node_shape() {
        let pipeline = Pipeline {
            nodes: vec![
                PipelineNode::query(1, 1234),
                PipelineNode::matview(2, "dune.test_team.result_test").depends_on(vec![1]),
            ],
        }
        .with_performance("medium")
        .with_query_parameters(&HashMap::from([(
            "chain".to_string(),
            serde_json::json!("ethereum"),
        )]));
        let value = serde_json::to_value(ExecutePipelineRequest { pipeline }).unwrap();
        assert_eq!(
            value,
            serde_json::json!({
                "pipeline": {"nodes": [
                    {"id": 1, "query_execution": {
                        "query_id": 1234,
                        "performance": "medium",
                        "query_parameters": {"chain": "ethereum"}
                    }},
                    {"id": 2, "dependencies": [1], "materialized_view_refresh": {
                        "name": "dune.test_team.result_test",
                        "performance": "medium"
                    }}
                ]}
            })
        );
    }

    #[test]
    fn pipeline_status_parses_spec_shape() {
        let json = r#"{
            "status": "running",
            "node_executions": [
                {"id": 1, "query_execution_status": {
                    "status": "completed", "query_id": 1234, "execution_id": "01TESTEXEC"
                }},
                {"id": 2, "materialized_view_refresh_status": {
                    "status": "pending", "name": "dune.test_team.result_test"
                }}
            ]
        }"#;
        let status: PipelineExecutionStatus = serde_json::from_str(json).unwrap();
        assert_eq!(status.status.as_deref(), Some("running"));
        let q = status.node_executions[0]
            .query_execution_status
            .as_ref()
            .unwrap();
        assert_eq!(q.query_id, Some(1234));
        let m = status.node_executions[1]
            .materialized_view_refresh_status
            .as_ref()
            .unwrap();
        assert_eq!(m.name.as_deref(), Some("dune.test_team.result_test"));
    }
}
