//! Custom Endpoints API for reading the results of the API endpoints you
//! create from Dune queries
//!
//! Results come back in the same shape as a query's results, so this module
//! reuses [`ExecutionResult`](crate::executions::ExecutionResult) and
//! [`GetResultsOptions`](crate::executions::GetResultsOptions).

mod api;

pub use api::EndpointsApi;
