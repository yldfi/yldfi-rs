//! Error types for the LI.FI API client
//!
//! This module provides the error types for the LI.FI API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for LI.FI API
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DomainError {
    /// No route found for the requested swap
    #[error("No route found for the requested swap")]
    NoRouteFound,

    /// No transaction data in response
    #[error("No transaction data in quote response")]
    NoTransaction,

    /// Invalid chain ID
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(u64),

    /// Invalid token address
    #[error("Invalid token address: {0}")]
    InvalidTokenAddress(String),

    /// Slippage too high
    #[error("Slippage exceeded maximum allowed: {0}%")]
    SlippageExceeded(f64),
}

/// Error type for LI.FI API operations
pub type Error = ApiError<DomainError>;

/// Result type for LI.FI API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create a no route found error
#[must_use]
pub fn no_route_found() -> Error {
    ApiError::domain(DomainError::NoRouteFound)
}

/// Create a no transaction error
#[must_use]
pub fn no_transaction() -> Error {
    ApiError::domain(DomainError::NoTransaction)
}

/// Create an invalid chain ID error
#[must_use]
pub fn invalid_chain_id(chain_id: u64) -> Error {
    ApiError::domain(DomainError::InvalidChainId(chain_id))
}

/// Create an invalid token address error
pub fn invalid_token_address(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidTokenAddress(address.into()))
}

/// Create a slippage exceeded error
#[must_use]
pub fn slippage_exceeded(slippage: f64) -> Error {
    ApiError::domain(DomainError::SlippageExceeded(slippage))
}
