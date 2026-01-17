//! Error types for the KyberSwap API client
//!
//! This module provides the error types for the KyberSwap API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for KyberSwap
#[derive(Error, Debug)]
pub enum DomainError {
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// Unsupported chain
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    /// No route found
    #[error("No route found for swap")]
    NoRouteFound,
}

/// Error type for KyberSwap API operations
pub type Error = ApiError<DomainError>;

/// Result type for KyberSwap API operations
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
