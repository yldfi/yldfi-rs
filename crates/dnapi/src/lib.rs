//! Dune Analytics API client for Rust
//!
//! An unofficial Rust client for the [Dune Analytics API](https://docs.dune.com/api-reference).
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> dune::error::Result<()> {
//! let client = dune::Client::new("your-api-key")?;
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
//! - **Pipelines**: Execute coordinated query workflows
//! - **Usage**: Track API consumption and credits

mod client;
pub mod error;

pub mod executions;
pub mod matviews;
pub mod pipelines;
pub mod queries;
pub mod tables;
pub mod usage;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the Dune Analytics API
pub const DEFAULT_BASE_URL: &str = "https://api.dune.com/api";

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    Config::new(api_key)
}
