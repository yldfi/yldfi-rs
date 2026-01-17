//! Error types for the CoinGecko API client
//!
//! This module provides the error types for the CoinGecko API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for CoinGecko API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Not found
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Error type for CoinGecko API operations
pub type Error = ApiError<DomainError>;

/// Result type for CoinGecko API operations
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
