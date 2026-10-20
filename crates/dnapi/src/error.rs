//! Error types for the Dune API client
//!
//! This module provides the error types for the Dune API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Dune API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Invalid API key
    #[error("Invalid API key")]
    InvalidApiKey,

    /// Query execution failed
    #[error("Query execution failed: {0}")]
    ExecutionFailed(String),

    /// Query execution timed out
    #[error("Query execution timed out after {0} seconds")]
    ExecutionTimeout(u64),

    /// Insufficient credits
    #[error("Insufficient credits")]
    InsufficientCredits,

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),
}

/// Error type for Dune API operations
pub type Error = ApiError<DomainError>;

/// Result type for Dune API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create an invalid API key error
#[must_use]
pub fn invalid_api_key() -> Error {
    ApiError::domain(DomainError::InvalidApiKey)
}

/// Create an execution failed error
pub fn execution_failed(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::ExecutionFailed(message.into()))
}

/// Create an execution timeout error
#[must_use]
pub fn execution_timeout(seconds: u64) -> Error {
    ApiError::domain(DomainError::ExecutionTimeout(seconds))
}

/// Create an insufficient credits error
#[must_use]
pub fn insufficient_credits() -> Error {
    ApiError::domain(DomainError::InsufficientCredits)
}

/// Create a not found error
pub fn not_found(resource: impl Into<String>) -> Error {
    ApiError::domain(DomainError::NotFound(resource.into()))
}

/// Convert a non-success response into an error.
///
/// 429 becomes [`ApiError::RateLimited`] with the (capped) `Retry-After`
/// value, 5xx becomes the retryable [`ApiError::ServerError`], and bodies are
/// sanitized. Callers handle their endpoint-specific statuses (404, 402) first.
pub(crate) async fn handle_error_response(response: reqwest::Response) -> Error {
    yldfi_common::api::handle_error_response(response).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn response(status: u16, retry_after: Option<&str>, body: &str) -> reqwest::Response {
        let mut builder = http::Response::builder().status(status);
        if let Some(value) = retry_after {
            builder = builder.header("retry-after", value);
        }
        builder.body(body.to_string()).unwrap().into()
    }

    #[tokio::test]
    async fn rate_limit_keeps_retry_after() {
        let err = handle_error_response(response(429, Some("42"), "")).await;
        assert!(err.is_retryable());
        assert_eq!(err.retry_after(), Some(Duration::from_secs(42)));
    }

    #[tokio::test]
    async fn server_error_is_retryable() {
        let err = handle_error_response(response(503, None, "unavailable")).await;
        assert!(matches!(err, ApiError::ServerError { status: 503, .. }));
        assert!(err.is_retryable());
    }

    #[tokio::test]
    async fn client_error_body_is_sanitized() {
        let err = handle_error_response(response(400, None, "bad api_key=hunter2")).await;
        assert!(matches!(err, ApiError::Api { status: 400, .. }));
        assert!(!err.is_retryable());
        assert!(!err.to_string().contains("hunter2"), "{err}");
    }
}
