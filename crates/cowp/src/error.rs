//! Error types for the `CoW` Protocol API client
//!
//! This module provides the error types for the `CoW` Protocol API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for `CoW` Protocol
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DomainError {
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// Unsupported chain
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    /// No quote available
    #[error("No quote available: {0}")]
    NoQuote(String),

    /// Insufficient liquidity
    #[error("Insufficient liquidity for swap")]
    InsufficientLiquidity,

    /// Order not found
    #[error("Order not found: {0}")]
    OrderNotFound(String),
}

/// Error type for `CoW` Protocol API operations
pub type Error = ApiError<DomainError>;

/// Result type for `CoW` Protocol API operations
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

/// Create a no quote error
pub fn no_quote(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::NoQuote(message.into()))
}

/// Create an insufficient liquidity error
#[must_use]
pub fn insufficient_liquidity() -> Error {
    ApiError::domain(DomainError::InsufficientLiquidity)
}

/// Create an order not found error
pub fn order_not_found(order_id: impl Into<String>) -> Error {
    ApiError::domain(DomainError::OrderNotFound(order_id.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_param_error() {
        let err = invalid_param("missing amount");
        let display = format!("{err}");
        assert!(display.contains("Invalid parameter"));
        assert!(display.contains("missing amount"));
    }

    #[test]
    fn test_unsupported_chain_error() {
        let err = unsupported_chain("fantom");
        let display = format!("{err}");
        assert!(display.contains("Unsupported chain"));
        assert!(display.contains("fantom"));
    }

    #[test]
    fn test_no_quote_error() {
        let err = no_quote("price impact too high");
        let display = format!("{err}");
        assert!(display.contains("No quote available"));
        assert!(display.contains("price impact too high"));
    }

    #[test]
    fn test_insufficient_liquidity_error() {
        let err = insufficient_liquidity();
        let display = format!("{err}");
        assert!(display.contains("Insufficient liquidity"));
    }

    #[test]
    fn test_order_not_found_error() {
        let err = order_not_found("0x1234abcd");
        let display = format!("{err}");
        assert!(display.contains("Order not found"));
        assert!(display.contains("0x1234abcd"));
    }

    #[test]
    fn test_domain_error_variants() {
        // Test that all variants are constructable
        let _ = DomainError::InvalidParam("test".to_string());
        let _ = DomainError::UnsupportedChain("test".to_string());
        let _ = DomainError::NoQuote("test".to_string());
        let _ = DomainError::InsufficientLiquidity;
        let _ = DomainError::OrderNotFound("test".to_string());
    }

    #[test]
    fn test_error_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Error>();
    }
}
