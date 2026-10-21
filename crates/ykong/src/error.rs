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

    /// Invalid or missing argument
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

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

/// Create an invalid argument error
pub fn invalid_argument(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidArgument(message.into()))
}

/// Create from HTTP response status and body
///
/// A GraphQL `errors` message is used when present, but 429 and 5xx keep
/// their rate-limited/server-error variants so they stay retryable.
#[must_use]
pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    let graphql_message = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|json| {
            json.get("errors")?
                .as_array()?
                .first()?
                .get("message")?
                .as_str()
                .map(str::to_string)
        });
    match graphql_message {
        Some(msg) if status != 429 && !(500..=599).contains(&status) => graphql_error(msg),
        Some(msg) => ApiError::from_response(status, &msg, retry_after),
        None => ApiError::from_response(status, body, retry_after),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_errors_with_graphql_body_stay_retryable() {
        let body = r#"{"errors":[{"message":"upstream timeout"}]}"#;
        let err = from_response(503, body, None);
        assert!(err.is_retryable());
        assert!(err.to_string().contains("upstream timeout"));

        let err = from_response(429, body, Some(3));
        assert!(err.is_retryable());

        let err = from_response(400, r#"{"errors":[{"message":"bad field"}]}"#, None);
        assert!(!err.is_retryable());
        assert!(matches!(err, ApiError::Domain(DomainError::GraphQL(_))));
    }
}
