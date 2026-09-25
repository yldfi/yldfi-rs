//! Error types for the Pyth Hermes API client

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for Pyth API
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DomainError {
    /// Price feed not found
    #[error("Price feed not found: {0}")]
    FeedNotFound(String),

    /// Invalid feed ID format
    #[error("Invalid feed ID: {0}")]
    InvalidFeedId(String),

    /// Price data is stale
    #[error("Stale price data")]
    StalePrice,

    /// URL parse error
    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    /// Insecure URL scheme (HTTP instead of HTTPS)
    #[error("Insecure URL scheme: HTTPS required for non-localhost URLs")]
    InsecureScheme,

    /// Invalid URL configuration
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Hermes rejected the request (HTTP 401/403).
    ///
    /// Since the Pyth Core upgrade (2026-08-26) all Hermes endpoints require
    /// an API key sent as `Authorization: Bearer <key>`.
    #[error("Pyth Hermes returned HTTP {status}: {}", unauthorized_hint(.api_key_set))]
    Unauthorized {
        /// HTTP status code (401 or 403)
        status: u16,
        /// Whether an API key was sent with the request
        api_key_set: bool,
    },
}

fn unauthorized_hint(api_key_set: &bool) -> &'static str {
    if *api_key_set {
        "the configured Pyth API key was rejected. Check that the key (PYTH_API_KEY or config) \
         is valid; manage keys at https://pythdata.app"
    } else {
        "an API key is required since the Pyth Core upgrade. Get one at https://pythdata.app \
         and set PYTH_API_KEY or use Client::with_api_key()"
    }
}

/// Error type for Pyth API operations
pub type Error = ApiError<DomainError>;

/// Result type for Pyth API operations
pub type Result<T> = std::result::Result<T, Error>;

/// Create a feed not found error
pub fn feed_not_found(feed_id: impl Into<String>) -> Error {
    ApiError::domain(DomainError::FeedNotFound(feed_id.into()))
}

/// Create an invalid feed ID error
pub fn invalid_feed_id(feed_id: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidFeedId(feed_id.into()))
}

/// Create a stale price error
#[must_use]
pub fn stale_price() -> Error {
    ApiError::domain(DomainError::StalePrice)
}

/// Create an insecure scheme error
#[must_use]
pub fn insecure_scheme() -> Error {
    ApiError::domain(DomainError::InsecureScheme)
}

/// Create an invalid URL error
pub fn invalid_url(msg: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidUrl(msg.into()))
}

/// Create an unauthorized error (missing or rejected API key)
#[must_use]
pub fn unauthorized(status: u16, api_key_set: bool) -> Error {
    ApiError::domain(DomainError::Unauthorized {
        status,
        api_key_set,
    })
}

/// Returns true if the error indicates a missing or rejected API key.
#[must_use]
pub fn is_unauthorized(err: &Error) -> bool {
    matches!(err, ApiError::Domain(DomainError::Unauthorized { .. }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feed_not_found() {
        let err = feed_not_found("test-feed");
        assert!(err.to_string().contains("Price feed not found"));
        assert!(err.to_string().contains("test-feed"));
    }

    #[test]
    fn test_invalid_feed_id() {
        let err = invalid_feed_id("bad-id");
        assert!(err.to_string().contains("Invalid feed ID"));
        assert!(err.to_string().contains("bad-id"));
    }

    #[test]
    fn test_unauthorized_messages() {
        let missing = unauthorized(401, false).to_string();
        assert!(missing.contains("401"));
        assert!(missing.contains("API key is required"));
        assert!(missing.contains("PYTH_API_KEY"));

        let rejected = unauthorized(403, true);
        assert!(rejected.to_string().contains("rejected"));
        assert!(is_unauthorized(&rejected));
        assert!(!rejected.is_retryable());
        assert!(!is_unauthorized(&stale_price()));
    }

    #[test]
    fn test_stale_price() {
        let err = stale_price();
        assert!(err.to_string().contains("Stale price data"));
    }
}
