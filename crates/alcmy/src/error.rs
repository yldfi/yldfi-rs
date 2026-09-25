//! Error types for the Alchemy API client
//!
//! This module provides the error types for the Alchemy API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Alchemy API
#[derive(Error, Debug)]
pub enum DomainError {
    /// JSON-RPC error
    #[error("RPC error ({code}): {message}")]
    Rpc { code: i64, message: String },

    /// Invalid API key
    #[error("Invalid API key")]
    InvalidApiKey,

    /// Method not supported by Alchemy on the selected network
    #[error("{method} is not supported on {network}: {reason}")]
    UnsupportedMethod {
        method: &'static str,
        network: &'static str,
        reason: &'static str,
    },
}

/// Error type for Alchemy API operations
pub type Error = ApiError<DomainError>;

/// Result type for Alchemy API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create an RPC error
pub fn rpc(code: i64, message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::Rpc {
        code,
        message: message.into(),
    })
}

/// Create an invalid API key error
pub fn invalid_api_key() -> Error {
    ApiError::domain(DomainError::InvalidApiKey)
}

/// Create an unsupported-method error
pub fn unsupported_method(
    method: &'static str,
    network: &'static str,
    reason: &'static str,
) -> Error {
    ApiError::domain(DomainError::UnsupportedMethod {
        method,
        network,
        reason,
    })
}
