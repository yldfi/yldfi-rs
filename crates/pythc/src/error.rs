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
    fn test_stale_price() {
        let err = stale_price();
        assert!(err.to_string().contains("Stale price data"));
    }
}
