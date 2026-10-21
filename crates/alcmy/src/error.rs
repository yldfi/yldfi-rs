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

    /// The app's monthly compute-unit capacity is used up (HTTP 429 with
    /// Alchemy's "Monthly capacity limit exceeded" body), including the
    /// sanitized, truncated response body.
    ///
    /// Retrying won't help until the cap resets, so this is a domain error and
    /// not retryable. Other 429s (per-second throughput limits) are returned
    /// as the retryable [`ApiError::RateLimited`].
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

    /// The Notify API (dashboard: Webhooks) requires the account's Notify auth token
    /// (sent as `X-Alchemy-Token`), which was not configured.
    #[error(
        "Alchemy Notify API (dashboard: Webhooks) requires the Webhooks auth token \
         (not the app API key). Copy it from the AUTH TOKEN button at the top right of the Alchemy dashboard Webhooks page \
         (sidebar Data -> Webhooks, https://dashboard.alchemy.com/webhooks) \
         and configure it with \
         `Config::with_notify_token` or the ALCHEMY_NOTIFY_TOKEN environment variable"
    )]
    MissingNotifyToken,

    /// The Gas Manager Admin API (dashboard: Gas Sponsorship) requires an access key (sent as
    /// `Authorization: Bearer`), which was not configured.
    #[error(
        "Alchemy Gas Manager Admin API (dashboard: Gas Sponsorship) requires an access key \
         (not the app API key). Create one under Alchemy dashboard -> Security -> \
         Create Access Key with Gas Manager (Gas Sponsorship) permissions \
         (billing/team admins only; https://www.alchemy.com/docs/how-to-create-access-keys) \
         and configure it with \
         `Config::with_access_key` or the ALCHEMY_ACCESS_KEY environment variable"
    )]
    MissingAccessKey,

    /// The requested network has no Beacon (consensus layer) endpoint
    #[error(
        "Beacon API is not available for network '{0}' \
         (supported: eth-mainnet, eth-sepolia, eth-hoodi)"
    )]
    UnsupportedBeaconNetwork(&'static str),

    /// Method not supported by Alchemy on the selected network
    #[error("{method} is not supported on {network}: {reason}")]
    UnsupportedMethod {
        method: &'static str,
        network: &'static str,
        reason: &'static str,
    },
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

/// Create a missing Notify auth token error
pub fn missing_notify_token() -> Error {
    ApiError::domain(DomainError::MissingNotifyToken)
}

/// Create a missing Gas Manager access key error
pub fn missing_access_key() -> Error {
    ApiError::domain(DomainError::MissingAccessKey)
}

/// Create a rate limited error from a `Retry-After` value and the raw 429 body.
///
/// Returns the retryable [`ApiError::RateLimited`], unless the body says the
/// monthly capacity is used up, in which case it returns the non-retryable
/// [`DomainError::RateLimited`] with the sanitized, truncated body.
pub fn rate_limited(retry_after: Option<u64>, body: &str) -> Error {
    if body.to_ascii_lowercase().contains("monthly capacity") {
        ApiError::domain(DomainError::RateLimited {
            retry_after,
            message: sanitize_error_body(body.trim()),
        })
    } else {
        ApiError::rate_limited(retry_after)
    }
}

/// Convert a non-success response into an error.
///
/// 429 goes through [`rate_limited`]; everything else through
/// [`ApiError::from_response`] (5xx are retryable, bodies are sanitized).
/// `Retry-After` is read with the shared, capped parser.
pub(crate) async fn handle_error_response(response: reqwest::Response) -> Error {
    let status = response.status().as_u16();
    let retry_after = extract_retry_after(response.headers());
    let body = response.text().await.unwrap_or_default();
    if status == 429 {
        rate_limited(retry_after, &body)
    } else {
        ApiError::from_response(status, &body, retry_after)
    }
}

/// Convert a transport or decode error, dropping the request URL.
///
/// Alchemy app keys live in the URL path, and `reqwest::Error`'s `Display`
/// includes the URL, so every `send()`/`json()`/`text()` error goes through
/// this instead of the plain `From` conversion.
pub(crate) fn http(e: reqwest::Error) -> Error {
    ApiError::Http(e.without_url())
}

/// Create an unsupported-method error
pub fn unsupported_method(
    method: &'static str,
    network: &'static str,
    reason: &'static str,
) -> Error {
    ApiError::domain(DomainError::UnsupportedMethod {
        method,
        network,
        reason,
    })
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

    #[test]
    fn throughput_429_is_retryable_with_retry_after() {
        let err = rate_limited(
            Some(30),
            r#"{"error":{"code":429,"message":"Your app has exceeded its compute units per second capacity."}}"#,
        );
        assert!(matches!(
            err,
            ApiError::RateLimited {
                retry_after: Some(30)
            }
        ));
        assert!(err.is_retryable());
        assert_eq!(err.retry_after(), Some(Duration::from_secs(30)));
    }

    #[test]
    fn monthly_capacity_429_is_not_retryable_and_keeps_body() {
        let err = rate_limited(
            Some(30),
            r#"{"error":{"code":429,"message":"Monthly capacity limit exceeded."}}"#,
        );
        assert!(!err.is_retryable());
        let msg = err.to_string();
        assert!(msg.contains("retry after 30s"), "{msg}");
        assert!(msg.contains("Monthly capacity limit exceeded"), "{msg}");
    }

    #[test]
    fn monthly_capacity_body_is_truncated_and_sanitized() {
        let long = format!(
            "Monthly capacity limit exceeded apikey=supersecret {}",
            "x".repeat(2000)
        );
        let msg = rate_limited(None, &long).to_string();
        assert!(!msg.contains("supersecret"), "{msg}");
        assert!(msg.contains("truncated"), "{msg}");
        assert!(msg.len() < 700, "{}", msg.len());
    }

    #[test]
    fn rate_limited_empty_body() {
        assert_eq!(rate_limited(None, "").to_string(), "Rate limited");
    }

    #[tokio::test]
    async fn handle_error_response_429_is_retryable_with_capped_retry_after() {
        let err = handle_error_response(response(429, Some("99999"), "slow down")).await;
        assert!(err.is_retryable());
        assert_eq!(err.retry_after(), Some(Duration::from_secs(3600)));
    }

    #[tokio::test]
    async fn handle_error_response_5xx_is_retryable() {
        let err = handle_error_response(response(503, None, "unavailable")).await;
        assert!(matches!(err, ApiError::ServerError { status: 503, .. }));
        assert!(err.is_retryable());
    }

    #[tokio::test]
    async fn handle_error_response_4xx_sanitizes_body() {
        let err = handle_error_response(response(400, None, "bad request apikey=hunter2")).await;
        assert!(matches!(err, ApiError::Api { status: 400, .. }));
        assert!(!err.is_retryable());
        assert!(!err.to_string().contains("hunter2"), "{err}");
    }

    #[tokio::test]
    async fn http_error_display_omits_key_in_url() {
        let raw = reqwest::Client::new()
            .get("http://127.0.0.1:1/v2/FAKEKEY")
            .send()
            .await
            .expect_err("nothing listens on port 1");
        assert!(raw.to_string().contains("FAKEKEY"), "{raw}");

        let err = http(raw);
        assert!(matches!(err, ApiError::Http(_)));
        assert!(!err.to_string().contains("FAKEKEY"), "{err}");
        assert!(!format!("{err:?}").contains("FAKEKEY"), "{err:?}");
    }
}
