//! Error types for the Dune API client
//!
//! This module provides the error types for the Dune API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Dune API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Invalid API key
    #[error("Invalid API key")]
    InvalidApiKey,

    /// Query execution failed
    #[error("Query execution failed: {0}")]
    ExecutionFailed(String),

    /// Query execution timed out
    #[error("Query execution timed out after {0} seconds")]
    ExecutionTimeout(u64),

    /// Insufficient credits
    #[error("Insufficient credits")]
    InsufficientCredits,

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),
}

/// Error type for Dune API operations
pub type Error = ApiError<DomainError>;

/// Result type for Dune API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create an invalid API key error
pub fn invalid_api_key() -> Error {
    ApiError::domain(DomainError::InvalidApiKey)
}

/// Create an execution failed error
pub fn execution_failed(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::ExecutionFailed(message.into()))
}

/// Create an execution timeout error
pub fn execution_timeout(seconds: u64) -> Error {
    ApiError::domain(DomainError::ExecutionTimeout(seconds))
}

/// Create an insufficient credits error
pub fn insufficient_credits() -> Error {
    ApiError::domain(DomainError::InsufficientCredits)
}

/// Create a not found error
pub fn not_found(resource: impl Into<String>) -> Error {
    ApiError::domain(DomainError::NotFound(resource.into()))
}
