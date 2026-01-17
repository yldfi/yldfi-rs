//! Error types for the Curve API client
//!
//! This module provides the error types for the Curve API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Curve API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
}

/// Error type for Curve API operations
pub type Error = ApiError<DomainError>;

/// Result type for Curve API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create an invalid parameter error
pub fn invalid_param(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidParam(message.into()))
}
