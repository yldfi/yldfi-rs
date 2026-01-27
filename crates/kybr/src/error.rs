//! Error types for the `KyberSwap` API client
//!
//! This module provides the error types for the `KyberSwap` API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for `KyberSwap`
#[derive(Error, Debug)]
#[non_exhaustive]
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

/// Error type for `KyberSwap` API operations
pub type Error = ApiError<DomainError>;

/// Result type for `KyberSwap` API operations
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
#[must_use]
pub fn no_route_found() -> Error {
    ApiError::domain(DomainError::NoRouteFound)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_param_error() {
        let err = invalid_param("missing token address");
        let display = format!("{err}");
        assert!(display.contains("Invalid parameter"));
        assert!(display.contains("missing token address"));
    }

    #[test]
    fn test_unsupported_chain_error() {
        let err = unsupported_chain("solana");
        let display = format!("{err}");
        assert!(display.contains("Unsupported chain"));
        assert!(display.contains("solana"));
    }

    #[test]
    fn test_no_route_found_error() {
        let err = no_route_found();
        let display = format!("{err}");
        assert!(display.contains("No route found"));
    }

    #[test]
    fn test_domain_error_variants() {
        // Test that all variants are constructable
        let _ = DomainError::InvalidParam("test".to_string());
        let _ = DomainError::UnsupportedChain("test".to_string());
        let _ = DomainError::NoRouteFound;
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}
