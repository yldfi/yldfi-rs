//! Types for the Executions API

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Query execution state
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub enum ExecutionState {
    #[serde(rename = "QUERY_STATE_PENDING")]
    Pending,
    #[serde(rename = "QUERY_STATE_EXECUTING")]
    Executing,
    #[serde(rename = "QUERY_STATE_COMPLETED")]
    Completed,
    #[serde(rename = "QUERY_STATE_FAILED")]
    Failed,
    #[serde(rename = "QUERY_STATE_CANCELLED")]
    Cancelled,
    #[serde(rename = "QUERY_STATE_EXPIRED")]
    Expired,
}

impl ExecutionState {
    /// Check if the execution is finished
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            ExecutionState::Completed
                | ExecutionState::Failed
                | ExecutionState::Cancelled
                | ExecutionState::Expired
        )
    }

    /// Check if the execution succeeded
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionState::Completed)
    }
}

/// Response from executing a query
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecuteQueryResponse {
    /// Execution ID
    pub execution_id: String,
    /// Current state
    pub state: ExecutionState,
}

/// Request to execute a query
#[derive(Debug, Clone, Serialize, Default)]
pub struct ExecuteQueryRequest {
    /// Query parameters (key-value pairs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<HashMap<String, String>>,
    /// Performance tier (medium or large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
}

impl ExecuteQueryRequest {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a query parameter
    pub fn param(mut self, key: &str, value: &str) -> Self {
        self.query_parameters
            .get_or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set performance tier to large
    #[must_use]
    pub fn large(mut self) -> Self {
        self.performance = Some("large".to_string());
        self
    }
}

/// Request to execute raw SQL
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteSqlRequest {
    /// SQL query to execute
    pub query_sql: String,
    /// Query parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_parameters: Option<HashMap<String, String>>,
    /// Performance tier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
}

impl ExecuteSqlRequest {
    #[must_use]
    pub fn new(query_sql: &str) -> Self {
        Self {
            query_sql: query_sql.to_string(),
            query_parameters: None,
            performance: None,
        }
    }

    /// Set a query parameter
    pub fn param(mut self, key: &str, value: &str) -> Self {
        self.query_parameters
            .get_or_insert_with(HashMap::new)
            .insert(key.to_string(), value.to_string());
        self
    }

    /// Set performance tier to large
    #[must_use]
    pub fn large(mut self) -> Self {
        self.performance = Some("large".to_string());
        self
    }
}

/// Syntax error metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SyntaxErrorMetadata {
    /// Line number
    pub line: Option<i64>,
    /// Column number
    pub column: Option<i64>,
}

/// Query execution error
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryResultError {
    /// Error type
    #[serde(rename = "type")]
    pub error_type: Option<String>,
    /// Error message
    pub message: Option<String>,
    /// Syntax error metadata
    pub metadata: Option<SyntaxErrorMetadata>,
}

/// Execution result metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionResultMetadata {
    /// Column names
    #[serde(default)]
    pub column_names: Vec<String>,
    /// Column types
    #[serde(default)]
    pub column_types: Vec<String>,
    /// Row count in current page
    pub row_count: Option<i64>,
    /// Total row count across all pages
    pub total_row_count: Option<i64>,
    /// Result set bytes
    pub result_set_bytes: Option<i64>,
    /// Total result set bytes
    pub total_result_set_bytes: Option<i64>,
    /// Datapoint count (for billing)
    pub datapoint_count: Option<i64>,
    /// Execution time in milliseconds
    pub execution_time_millis: Option<i64>,
    /// Pending time in milliseconds
    pub pending_time_millis: Option<i64>,
}

/// Execution status response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionStatus {
    /// Execution ID
    pub execution_id: String,
    /// Query ID
    pub query_id: Option<i64>,
    /// Current state
    pub state: ExecutionState,
    /// Whether execution is finished
    pub is_execution_finished: bool,
    /// Timestamp when submitted
    pub submitted_at: Option<String>,
    /// Timestamp when execution started
    pub execution_started_at: Option<String>,
    /// Timestamp when execution ended
    pub execution_ended_at: Option<String>,
    /// Timestamp when cancelled
    pub cancelled_at: Option<String>,
    /// Timestamp when results expire
    pub expires_at: Option<String>,
    /// Execution cost in credits
    pub execution_cost_credits: Option<f64>,
    /// Queue position
    pub queue_position: Option<i64>,
    /// Result metadata
    pub result_metadata: Option<ExecutionResultMetadata>,
    /// Error information
    pub error: Option<QueryResultError>,
}

/// A single row of query results
pub type Row = HashMap<String, serde_json::Value>;

/// Query result data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QueryResultData {
    /// Result metadata
    pub metadata: Option<ExecutionResultMetadata>,
    /// Result rows
    #[serde(default)]
    pub rows: Vec<Row>,
}

/// Execution result response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExecutionResult {
    /// Execution ID
    pub execution_id: String,
    /// Query ID
    pub query_id: Option<i64>,
    /// Current state
    pub state: ExecutionState,
    /// Whether execution is finished
    pub is_execution_finished: bool,
    /// Timestamp when submitted
    pub submitted_at: Option<String>,
    /// Timestamp when execution started
    pub execution_started_at: Option<String>,
    /// Timestamp when execution ended
    pub execution_ended_at: Option<String>,
    /// Timestamp when cancelled
    pub cancelled_at: Option<String>,
    /// Timestamp when results expire
    pub expires_at: Option<String>,
    /// Error information
    pub error: Option<QueryResultError>,
    /// Query result data
    pub result: Option<QueryResultData>,
    /// Next page offset
    pub next_offset: Option<i64>,
    /// Next page URI
    pub next_uri: Option<String>,
}

/// Response from cancelling an execution
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CancelExecutionResponse {
    /// Whether cancellation was successful
    pub success: bool,
}

/// Options for getting execution results
#[derive(Debug, Clone, Default)]
pub struct GetResultsOptions {
    /// Maximum number of rows
    pub limit: Option<u32>,
    /// Pagination offset
    pub offset: Option<i64>,
    /// Column to sort by
    pub sort_by: Option<String>,
    /// Sort direction (asc, desc)
    pub order: Option<String>,
    /// Filter columns (comma-separated)
    pub columns: Option<String>,
    /// Sample count (for sampling)
    pub sample_count: Option<u32>,
    /// Allow partial results for large datasets
    pub allow_partial_results: Option<bool>,
}

impl GetResultsOptions {
    #[must_use]
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(format!("limit={limit}"));
        }
        if let Some(offset) = self.offset {
            params.push(format!("offset={offset}"));
        }
        if let Some(ref sort_by) = self.sort_by {
            params.push(format!("sort_by={sort_by}"));
        }
        if let Some(ref order) = self.order {
            params.push(format!("order={order}"));
        }
        if let Some(ref columns) = self.columns {
            params.push(format!("columns={columns}"));
        }
        if let Some(sample_count) = self.sample_count {
            params.push(format!("sample_count={sample_count}"));
        }
        if let Some(allow_partial) = self.allow_partial_results {
            params.push(format!("allow_partial_results={allow_partial}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}
