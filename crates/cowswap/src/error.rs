//! Error types for the CoW Protocol API client
//!
//! This module provides the error types for the CoW Protocol API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for CoW Protocol
#[derive(Error, Debug)]
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

/// Error type for CoW Protocol API operations
pub type Error = ApiError<DomainError>;

/// Result type for CoW Protocol API operations
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
pub fn insufficient_liquidity() -> Error {
    ApiError::domain(DomainError::InsufficientLiquidity)
}

/// Create an order not found error
pub fn order_not_found(order_id: impl Into<String>) -> Error {
    ApiError::domain(DomainError::OrderNotFound(order_id.into()))
}
