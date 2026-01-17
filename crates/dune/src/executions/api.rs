//! Executions API implementation

use super::types::*;
use crate::client::Client;
use crate::error::{self, Error, Result};
use std::time::Duration;
use tokio::time::sleep;

/// Executions API
pub struct ExecutionsApi<'a> {
    client: &'a Client,
}

impl<'a> ExecutionsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Execute a saved query by ID
    pub async fn execute(&self, query_id: i64) -> Result<ExecuteQueryResponse> {
        self.execute_with_options(query_id, &ExecuteQueryRequest::default())
            .await
    }

    /// Execute a saved query with options
    pub async fn execute_with_options(
        &self,
        query_id: i64,
        request: &ExecuteQueryRequest,
    ) -> Result<ExecuteQueryResponse> {
        let url = format!("{}/v1/query/{}/execute", self.client.base_url(), query_id);
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else if response.status() == 402 {
            Err(error::insufficient_credits())
        } else if response.status() == 429 {
            Err(Error::rate_limited(None))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Execute raw SQL
    pub async fn execute_sql(&self, request: &ExecuteSqlRequest) -> Result<ExecuteQueryResponse> {
        let url = format!("{}/v1/sql/execute", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 402 {
            Err(error::insufficient_credits())
        } else if response.status() == 429 {
            Err(Error::rate_limited(None))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get execution status
    pub async fn status(&self, execution_id: &str) -> Result<ExecutionStatus> {
        let url = format!(
            "{}/v1/execution/{}/status",
            self.client.base_url(),
            execution_id
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Execution {}", execution_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get execution results
    pub async fn results(&self, execution_id: &str) -> Result<ExecutionResult> {
        self.results_with_options(execution_id, &GetResultsOptions::default())
            .await
    }

    /// Get execution results with options
    pub async fn results_with_options(
        &self,
        execution_id: &str,
        options: &GetResultsOptions,
    ) -> Result<ExecutionResult> {
        let url = format!(
            "{}/v1/execution/{}/results{}",
            self.client.base_url(),
            execution_id,
            options.to_query_string()
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Execution {}", execution_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get execution results as CSV
    pub async fn results_csv(&self, execution_id: &str) -> Result<String> {
        self.results_csv_with_options(execution_id, &GetResultsOptions::default())
            .await
    }

    /// Get execution results as CSV with options
    pub async fn results_csv_with_options(
        &self,
        execution_id: &str,
        options: &GetResultsOptions,
    ) -> Result<String> {
        let url = format!(
            "{}/v1/execution/{}/results/csv{}",
            self.client.base_url(),
            execution_id,
            options.to_query_string()
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Execution {}", execution_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Cancel an execution
    pub async fn cancel(&self, execution_id: &str) -> Result<CancelExecutionResponse> {
        let url = format!(
            "{}/v1/execution/{}/cancel",
            self.client.base_url(),
            execution_id
        );
        let response = self.client.http().post(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Execution {}", execution_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get the latest results for a saved query (uses cached results if available)
    pub async fn query_results(&self, query_id: i64) -> Result<ExecutionResult> {
        self.query_results_with_options(query_id, &GetResultsOptions::default())
            .await
    }

    /// Get the latest results for a saved query with options
    pub async fn query_results_with_options(
        &self,
        query_id: i64,
        options: &GetResultsOptions,
    ) -> Result<ExecutionResult> {
        let url = format!(
            "{}/v1/query/{}/results{}",
            self.client.base_url(),
            query_id,
            options.to_query_string()
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get the latest results for a saved query as CSV
    pub async fn query_results_csv(&self, query_id: i64) -> Result<String> {
        self.query_results_csv_with_options(query_id, &GetResultsOptions::default())
            .await
    }

    /// Get the latest results for a saved query as CSV with options
    pub async fn query_results_csv_with_options(
        &self,
        query_id: i64,
        options: &GetResultsOptions,
    ) -> Result<String> {
        let url = format!(
            "{}/v1/query/{}/results/csv{}",
            self.client.base_url(),
            query_id,
            options.to_query_string()
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {}", query_id)))
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Execute a query and wait for results (convenience method)
    ///
    /// This method polls for completion with exponential backoff.
    /// Timeout is in seconds (default 300 = 5 minutes).
    pub async fn run_query(
        &self,
        query_id: i64,
        timeout_secs: Option<u64>,
    ) -> Result<ExecutionResult> {
        self.run_query_with_options(query_id, &ExecuteQueryRequest::default(), timeout_secs)
            .await
    }

    /// Execute a query with options and wait for results
    pub async fn run_query_with_options(
        &self,
        query_id: i64,
        request: &ExecuteQueryRequest,
        timeout_secs: Option<u64>,
    ) -> Result<ExecutionResult> {
        let timeout = timeout_secs.unwrap_or(300);
        let exec = self.execute_with_options(query_id, request).await?;
        self.wait_for_results(&exec.execution_id, timeout).await
    }

    /// Execute SQL and wait for results (convenience method)
    pub async fn run_sql(&self, sql: &str, timeout_secs: Option<u64>) -> Result<ExecutionResult> {
        let request = ExecuteSqlRequest::new(sql);
        self.run_sql_with_options(&request, timeout_secs).await
    }

    /// Execute SQL with options and wait for results
    pub async fn run_sql_with_options(
        &self,
        request: &ExecuteSqlRequest,
        timeout_secs: Option<u64>,
    ) -> Result<ExecutionResult> {
        let timeout = timeout_secs.unwrap_or(300);
        let exec = self.execute_sql(request).await?;
        self.wait_for_results(&exec.execution_id, timeout).await
    }

    /// Wait for an execution to complete and return results
    async fn wait_for_results(
        &self,
        execution_id: &str,
        timeout_secs: u64,
    ) -> Result<ExecutionResult> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);
        let mut poll_interval = Duration::from_millis(500);
        let max_interval = Duration::from_secs(5);

        loop {
            if start.elapsed() > timeout {
                return Err(error::execution_timeout(timeout_secs));
            }

            let status = self.status(execution_id).await?;

            if status.is_execution_finished {
                if status.state.is_success() {
                    return self.results(execution_id).await;
                } else {
                    let msg = status
                        .error
                        .map(|e| e.message.unwrap_or_default())
                        .unwrap_or_else(|| {
                            format!("Execution failed with state: {:?}", status.state)
                        });
                    return Err(error::execution_failed(msg));
                }
            }

            sleep(poll_interval).await;
            poll_interval = std::cmp::min(poll_interval * 2, max_interval);
        }
    }
}
