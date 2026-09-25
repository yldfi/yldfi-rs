//! Error types for the Alchemy API client
//!
//! This module provides the error types for the Alchemy API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;
use yldfi_common::api::{extract_retry_after, sanitize_error_body};

/// Domain-specific errors for Alchemy API
#[derive(Error, Debug)]
pub enum DomainError {
    /// JSON-RPC error
    #[error("RPC error ({code}): {message}")]
    Rpc { code: i64, message: String },

    /// Invalid API key
    #[error("Invalid API key")]
    InvalidApiKey,

    /// Rate limited (HTTP 429), including the (sanitized, truncated) response
    /// body so callers can distinguish e.g. a monthly compute-unit cap from a
    /// per-second throughput limit.
    #[error(
        "Rate limited (HTTP 429){}{}",
        .retry_after.map(|s| format!(", retry after {s}s")).unwrap_or_default(),
        if .message.is_empty() { String::new() } else { format!(": {}", .message) }
    )]
    RateLimited {
        /// Seconds to wait before retrying (from the `Retry-After` header)
        retry_after: Option<u64>,
        /// Sanitized and truncated response body
        message: String,
    },

    /// An Alchemy auth token (dashboard auth token / access key) is required
    /// for this API but was not configured.
    #[error(
        "{api} requires an Alchemy auth token (not the app API key). \
         Create one in the Alchemy dashboard and configure it with \
         `Config::with_auth_token` or the ALCHEMY_AUTH_TOKEN environment variable"
    )]
    MissingAuthToken {
        /// Name of the API that requires the token
        api: &'static str,
    },

    /// The requested network has no Beacon (consensus layer) endpoint
    #[error(
        "Beacon API is not available for network '{0}' \
         (supported: eth-mainnet, eth-sepolia, eth-holesky)"
    )]
    UnsupportedBeaconNetwork(&'static str),
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

/// Create a missing auth token error
pub fn missing_auth_token(api: &'static str) -> Error {
    ApiError::domain(DomainError::MissingAuthToken { api })
}

/// Create a rate limited error from a `Retry-After` value and raw body.
///
/// The body is sanitized (secrets redacted) and truncated.
pub fn rate_limited(retry_after: Option<u64>, body: &str) -> Error {
    ApiError::domain(DomainError::RateLimited {
        retry_after,
        message: sanitize_error_body(body.trim()),
    })
}

/// Build a rate limited error from a 429 response, preserving the
/// `Retry-After` header and the response body.
pub(crate) async fn rate_limited_from_response(response: reqwest::Response) -> Error {
    let retry_after = extract_retry_after(response.headers());
    let body = response.text().await.unwrap_or_default();
    rate_limited(retry_after, &body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_limited_includes_body_and_retry_after() {
        let err = rate_limited(
            Some(30),
            r#"{"error":{"code":429,"message":"Monthly capacity limit exceeded."}}"#,
        );
        let msg = err.to_string();
        assert!(msg.contains("retry after 30s"), "{msg}");
        assert!(msg.contains("Monthly capacity limit exceeded"), "{msg}");
    }

    #[test]
    fn rate_limited_truncates_and_sanitizes_body() {
        let long = format!("apikey=supersecret {}", "x".repeat(2000));
        let msg = rate_limited(None, &long).to_string();
        assert!(!msg.contains("supersecret"), "{msg}");
        assert!(msg.contains("truncated"), "{msg}");
        assert!(msg.len() < 700, "{}", msg.len());
    }

    #[test]
    fn rate_limited_empty_body() {
        assert_eq!(
            rate_limited(None, "").to_string(),
            "Rate limited (HTTP 429)"
        );
    }
}
