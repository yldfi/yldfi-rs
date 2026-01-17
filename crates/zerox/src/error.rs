//! Error types for the 0x API client
//!
//! This module provides the error types for the 0x API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for 0x API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// Unsupported chain
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    /// No route found for swap
    #[error("No route found for swap")]
    NoRouteFound,

    /// Missing API key
    #[error("API key is required for this endpoint")]
    MissingApiKey,

    /// Insufficient liquidity
    #[error("Insufficient liquidity for requested swap")]
    InsufficientLiquidity,

    /// Validation error
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
}

/// Error type for 0x API operations
pub type Error = ApiError<DomainError>;

/// Result type for 0x API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create an invalid parameter error
pub fn invalid_param(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidParam(message.into()))
}

/// Create an unsupported chain error
pub fn unsupported_chain(chain: impl Into<String>) -> Error {
    ApiError::domain(DomainError::UnsupportedChain(chain.into()))
}

/// Create a no route found error
pub fn no_route_found() -> Error {
    ApiError::domain(DomainError::NoRouteFound)
}

/// Create a missing API key error
pub fn missing_api_key() -> Error {
    ApiError::domain(DomainError::MissingApiKey)
}

/// Create an insufficient liquidity error
pub fn insufficient_liquidity() -> Error {
    ApiError::domain(DomainError::InsufficientLiquidity)
}

/// Create a validation error
pub fn validation_error(field: impl Into<String>, message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::ValidationError {
        field: field.into(),
        message: message.into(),
    })
}
