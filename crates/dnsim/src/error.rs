//! Error types for the Dune SIM API client
//!
//! This module provides the error types for the Dune SIM API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Dune SIM API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Unauthorized
    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Bad request
    #[error("Bad request: {0}")]
    BadRequest(String),

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Error type for Dune SIM API operations
pub type Error = ApiError<DomainError>;

/// Result type for Dune SIM API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create an unauthorized error
pub fn unauthorized(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::Unauthorized(message.into()))
}

/// Create a not found error
pub fn not_found(resource: impl Into<String>) -> Error {
    ApiError::domain(DomainError::NotFound(resource.into()))
}

/// Create a bad request error
pub fn bad_request(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::BadRequest(message.into()))
}

/// Create from HTTP response status and body
///
/// Handles Dune SIM-specific error patterns
#[must_use]
pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    match status {
        400 => bad_request(body),
        401 => unauthorized(body),
        404 => not_found(body),
        _ => ApiError::from_response(status, body, retry_after),
    }
}
