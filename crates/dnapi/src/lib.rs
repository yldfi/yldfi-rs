//! Dune Analytics API client for Rust
//!
//! An unofficial Rust client for the [Dune Analytics API](https://docs.dune.com/api-reference).
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> dnapi::error::Result<()> {
//! let client = dnapi::Client::new("your-api-key")?;
//!
//! // Execute a query and wait for results
//! let result = client.executions().run_query(1234, None).await?;
//! for row in result.result.unwrap().rows {
//!     println!("{:?}", row);
//! }
//!
//! // Execute raw SQL
//! let result = client.executions().run_sql("SELECT 1 as value", None).await?;
//! println!("{:?}", result.result);
//!
//! // Get cached results for a query
//! let result = client.executions().query_results(1234).await?;
//! println!("{:?}", result.result);
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **Queries**: Create, read, update, archive/unarchive queries
//! - **Executions**: Execute queries, run SQL, get results (JSON or CSV)
//! - **Tables**: Upload data, create tables, insert rows
//! - **Materialized Views**: Create, refresh, manage materialized views
//! - **Pipelines**: Execute coordinated query workflows, build a query's pipeline
//! - **Dashboards**: Create, read, update and archive dashboards
//! - **Visualizations**: Create, read, update and delete query visualizations
//! - **Folders**: Organize queries and dashboards into library folders
//! - **Schedules**: Run queries and refresh dashboards on a schedule
//! - **Datasets**: List and search the Dune catalog
//! - **Contracts**: Submit contracts for decoding and track submissions
//! - **Endpoints**: Read the results of custom API endpoints
//! - **Usage**: Track API consumption and credits

mod client;
pub mod error;

pub mod contracts;
pub mod dashboards;
pub mod datasets;
pub mod endpoints;
pub mod executions;
pub mod folders;
pub mod matviews;
pub mod pipelines;
pub mod queries;
pub mod schedules;
pub mod tables;
pub mod usage;
pub mod visualizations;

#[cfg(test)]
mod test_util;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Pagination for the list endpoints that take `limit` and `offset`
#[derive(Debug, Clone, Copy, Default)]
#[non_exhaustive]
pub struct PageOptions {
    /// Maximum number of results
    pub limit: Option<u32>,
    /// Number of results to skip
    pub offset: Option<i64>,
}

impl PageOptions {
    /// A page of at most `limit` results starting at `offset`
    #[must_use]
    pub fn new(limit: Option<u32>, offset: Option<i64>) -> Self {
        Self { limit, offset }
    }

    /// Query pairs for the request
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        if let Some(limit) = self.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(offset) = self.offset {
            params.push(("offset", offset.to_string()));
        }
        params
    }
}

/// Default base URL for the Dune Analytics API
pub const DEFAULT_BASE_URL: &str = "https://api.dune.com/api";

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    Config::new(api_key)
}
