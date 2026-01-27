//! Error types for the `DefiLlama` API client
//!
//! This module provides the error types for the `DefiLlama` API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for `DefiLlama` API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// URL parse error
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Error type for `DefiLlama` API operations
pub type Error = ApiError<DomainError>;

/// Result type for `DefiLlama` API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create a not found error
pub fn not_found(resource: impl Into<String>) -> Error {
    ApiError::domain(DomainError::NotFound(resource.into()))
}

/// Create an invalid parameter error
pub fn invalid_param(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidParam(message.into()))
}

/// Create from HTTP response status and body
///
/// Handles DefiLlama-specific error patterns (404 as `NotFound`)
#[must_use] 
pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    if status == 404 {
        return not_found(body);
    }
    ApiError::from_response(status, body, retry_after)
}
