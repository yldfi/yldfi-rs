//! Error types for the Tenderly API client
//!
//! This module provides the error types for the Tenderly API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Tenderly API
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DomainError {
    /// Authentication failed or credentials missing
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Invalid input parameters
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
}

/// Error type for Tenderly API operations
pub type Error = ApiError<DomainError>;

/// Result type for Tenderly API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors

/// Create an authentication error
pub fn auth(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::Auth(message.into()))
}

/// Create a configuration error
pub fn config(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::Config(message.into()))
}

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
/// Handles Tenderly-specific error patterns:
/// - 401 -> Auth error
/// - 404 -> `NotFound`
/// - 429 -> `RateLimited`
/// - 5xx -> `ServerError`
#[allow(dead_code)]
pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    match status {
        401 => auth("Invalid or missing API key"),
        404 => not_found(body),
        _ => ApiError::from_response(status, body, retry_after),
    }
}

/// Check if an error is a not found error
#[allow(dead_code)]
pub fn is_not_found(error: &Error) -> bool {
    matches!(error, ApiError::Domain(DomainError::NotFound(_)))
}
