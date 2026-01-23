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
