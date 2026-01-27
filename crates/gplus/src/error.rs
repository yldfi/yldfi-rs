//! Error types for the GoPlus Security API client

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for GoPlus API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Token not found
    #[error("Token not found: {0}")]
    TokenNotFound(String),

    /// Chain not supported
    #[error("Chain not supported: {0}")]
    UnsupportedChain(u64),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),
}

/// Error type for GoPlus API operations
pub type Error = ApiError<DomainError>;

/// Result type for GoPlus API operations
pub type Result<T> = std::result::Result<T, Error>;

/// Create a token not found error
pub fn token_not_found(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::TokenNotFound(address.into()))
}

/// Create an unsupported chain error
pub fn unsupported_chain(chain_id: u64) -> Error {
    ApiError::domain(DomainError::UnsupportedChain(chain_id))
}

/// Create an invalid address error
pub fn invalid_address(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidAddress(address.into()))
}
