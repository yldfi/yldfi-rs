//! Error types for the Kong API client

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Kong API
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DomainError {
    /// GraphQL query error
    #[error("GraphQL error: {0}")]
    GraphQL(String),

    /// Vault not found
    #[error("Vault not found: {0}")]
    VaultNotFound(String),

    /// Strategy not found
    #[error("Strategy not found: {0}")]
    StrategyNotFound(String),

    /// Invalid chain ID
    #[error("Invalid chain ID: {0}")]
    InvalidChainId(u64),

    /// URL parse error
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// API endpoint removed
    #[error("API endpoint '{endpoint}' has been removed. {alternative}")]
    ApiEndpointRemoved {
        endpoint: String,
        alternative: String,
    },
}

/// Error type for Kong API operations
pub type Error = ApiError<DomainError>;

/// Result type for Kong API operations
pub type Result<T> = std::result::Result<T, Error>;

/// Create a GraphQL error
pub fn graphql_error(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::GraphQL(message.into()))
}

/// Create a vault not found error
pub fn vault_not_found(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::VaultNotFound(address.into()))
}

/// Create a strategy not found error
pub fn strategy_not_found(address: impl Into<String>) -> Error {
    ApiError::domain(DomainError::StrategyNotFound(address.into()))
}

/// Create from HTTP response status and body
#[must_use]
pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    // Check for GraphQL errors in JSON response
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(body) {
        if let Some(errors) = json.get("errors").and_then(|e| e.as_array()) {
            if let Some(first) = errors.first() {
                if let Some(msg) = first.get("message").and_then(|m| m.as_str()) {
                    return graphql_error(msg);
                }
            }
        }
    }
    ApiError::from_response(status, body, retry_after)
}
