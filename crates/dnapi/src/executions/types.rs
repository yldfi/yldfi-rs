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
    /// Finished, but the result was too large and only a partial result is
    /// available (fetch it with `allow_partial_results=true`)
    #[serde(rename = "QUERY_STATE_COMPLETED_PARTIAL")]
    CompletedPartial,
    #[serde(rename = "QUERY_STATE_FAILED")]
    Failed,
    /// Dune docs spell it `CANCELED`; accept both spellings
    #[serde(rename = "QUERY_STATE_CANCELLED", alias = "QUERY_STATE_CANCELED")]
    Cancelled,
    #[serde(rename = "QUERY_STATE_EXPIRED")]
    Expired,
    /// A state this client does not know about
    #[serde(other)]
    Unknown,
}

impl ExecutionState {
    /// Check if the execution is finished
    #[must_use]
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            ExecutionState::Completed
                | ExecutionState::CompletedPartial
                | ExecutionState::Failed
                | ExecutionState::Cancelled
                | ExecutionState::Expired
        )
    }

    /// Check if the execution succeeded (including with partial results)
    #[must_use]
    pub fn is_success(&self) -> bool {
        matches!(
            self,
            ExecutionState::Completed | ExecutionState::CompletedPartial
        )
    }

    /// Check if only a partial result is available
    #[must_use]
    pub fn is_partial(&self) -> bool {
        matches!(self, ExecutionState::CompletedPartial)
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

/// Request to execute raw SQL (`POST /v1/sql/execute`)
///
/// The endpoint takes only `sql` and `performance`; it has no query parameters.
#[derive(Debug, Clone, Serialize)]
pub struct ExecuteSqlRequest {
    /// SQL query to execute
    pub sql: String,
    /// Performance tier (medium or large)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<String>,
}

impl ExecuteSqlRequest {
    #[must_use]
    pub fn new(sql: &str) -> Self {
        Self {
            sql: sql.to_string(),
            performance: None,
        }
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
    /// Sort expression, e.g. `block_time desc`
    pub sort_by: Option<String>,
    /// Filter expression, e.g. `amount > 100`
    pub filters: Option<String>,
    /// Columns to return (comma-separated)
    pub columns: Option<String>,
    /// Number of rows to sample (incompatible with offset, limit and filters)
    pub sample_count: Option<u32>,
    /// Allow partial results for large datasets
    pub allow_partial_results: Option<bool>,
    /// Ignore the per-request max credits limit
    pub ignore_max_credits_per_request: Option<bool>,
}

impl GetResultsOptions {
    /// Query pairs for the request (values are URL-encoded by the HTTP client)
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset", offset.to_string()));
        }
        if let Some(ref sort_by) = self.sort_by {
            params.push(("sort_by", sort_by.clone()));
        }
        if let Some(ref filters) = self.filters {
            params.push(("filters", filters.clone()));
        }
        if let Some(ref columns) = self.columns {
            params.push(("columns", columns.clone()));
        }
        if let Some(sample_count) = self.sample_count {
            params.push(("sample_count", sample_count.to_string()));
        }
        if let Some(allow_partial) = self.allow_partial_results {
            params.push(("allow_partial_results", allow_partial.to_string()));
        }
        if let Some(ignore) = self.ignore_max_credits_per_request {
            params.push(("ignore_max_credits_per_request", ignore.to_string()));
        }
        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_sql_request_uses_spec_field_names() {
        let body = serde_json::to_value(ExecuteSqlRequest::new("SELECT 1").large()).unwrap();
        assert_eq!(
            body,
            serde_json::json!({"sql": "SELECT 1", "performance": "large"})
        );

        let body = serde_json::to_value(ExecuteSqlRequest::new("SELECT 1")).unwrap();
        assert_eq!(body, serde_json::json!({"sql": "SELECT 1"}));
    }

    #[test]
    fn execution_state_completed_partial_is_terminal_success() {
        let state: ExecutionState =
            serde_json::from_str("\"QUERY_STATE_COMPLETED_PARTIAL\"").unwrap();
        assert_eq!(state, ExecutionState::CompletedPartial);
        assert!(state.is_finished());
        assert!(state.is_success());
        assert!(state.is_partial());
        assert!(!ExecutionState::Completed.is_partial());
    }

    #[test]
    fn execution_state_cancelled_accepts_both_spellings() {
        for raw in ["\"QUERY_STATE_CANCELLED\"", "\"QUERY_STATE_CANCELED\""] {
            let state: ExecutionState = serde_json::from_str(raw).unwrap();
            assert_eq!(state, ExecutionState::Cancelled);
        }
    }

    #[test]
    fn execution_state_unknown_fallback() {
        let state: ExecutionState = serde_json::from_str("\"QUERY_STATE_SOMETHING_NEW\"").unwrap();
        assert_eq!(state, ExecutionState::Unknown);
        assert!(!state.is_finished());
        assert!(!state.is_success());

        let status: ExecutionStatus = serde_json::from_value(serde_json::json!({
            "execution_id": "01TESTEXECUTIONID",
            "state": "QUERY_STATE_SOMETHING_NEW",
            "is_execution_finished": false
        }))
        .unwrap();
        assert_eq!(status.state, ExecutionState::Unknown);
    }
}
