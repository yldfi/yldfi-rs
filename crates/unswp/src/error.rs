//! Error types for the Uniswap client

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Uniswap operations
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DomainError {
    /// Pool not found
    #[error("Pool not found: {0}")]
    PoolNotFound(String),

    /// Invalid pool address format
    #[error("Invalid pool address: {0}")]
    InvalidPoolAddress(String),

    /// Invalid token address format
    #[error("Invalid token address: {0}")]
    InvalidTokenAddress(String),

    /// Subgraph API key required for historical queries
    #[error("Subgraph API key required for historical queries")]
    SubgraphKeyRequired,

    /// Subgraph query error
    #[error("Subgraph query error: {0}")]
    SubgraphError(String),

    /// RPC provider error
    #[error("RPC error: {0}")]
    RpcError(String),

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Lens query error
    #[error("Lens query error: {0}")]
    LensError(String),

    /// Quote calculation error
    #[error("Quote error: {0}")]
    QuoteError(String),
}

/// Error type for Uniswap operations
pub type Error = ApiError<DomainError>;

/// Result type for Uniswap operations
pub type Result<T> = std::result::Result<T, Error>;

/// Create a pool not found error
pub fn pool_not_found(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::PoolNotFound(address.into()))
}

/// Create an invalid pool address error
pub fn invalid_pool_address(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidPoolAddress(address.into()))
}

/// Create an invalid token address error
pub fn invalid_token_address(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidTokenAddress(address.into()))
}

/// Create a subgraph key required error
#[must_use] 
pub fn subgraph_key_required() -> Error {
    ApiError::domain(DomainError::SubgraphKeyRequired)
}

/// Create a subgraph error
pub fn subgraph_error(msg: impl Into<String>) -> Error {
    ApiError::domain(DomainError::SubgraphError(msg.into()))
}

/// Create an RPC error
pub fn rpc_error(msg: impl Into<String>) -> Error {
    ApiError::domain(DomainError::RpcError(msg.into()))
}

/// Create a lens error
pub fn lens_error(msg: impl Into<String>) -> Error {
    ApiError::domain(DomainError::LensError(msg.into()))
}

/// Create a quote error
pub fn quote_error(msg: impl Into<String>) -> Error {
    ApiError::domain(DomainError::QuoteError(msg.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_not_found() {
        let err = pool_not_found("0x1234");
        assert!(err.to_string().contains("Pool not found"));
        assert!(err.to_string().contains("0x1234"));
    }

    #[test]
    fn test_subgraph_key_required() {
        let err = subgraph_key_required();
        assert!(err.to_string().contains("Subgraph API key required"));
    }
}
