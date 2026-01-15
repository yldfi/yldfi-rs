//! Error types for the Tenderly API client

use thiserror::Error;

/// Result type alias for Tenderly operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when interacting with the Tenderly API
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON serialization/deserialization failed
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response
    #[error("Tenderly API error ({status}): {message}")]
    Api { status: u16, message: String },

    /// Authentication failed or credentials missing
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Resource not found
    #[error("Resource not found: {0}")]
    NotFound(String),

    /// Rate limit exceeded
    ///
    /// The `retry_after` field contains the number of seconds to wait before retrying,
    /// if the server provided a `Retry-After` header.
    #[error("Rate limit exceeded{}", .retry_after.map(|s| format!(" (retry after {} seconds)", s)).unwrap_or_default())]
    RateLimited {
        /// Seconds to wait before retrying (from Retry-After header)
        retry_after: Option<u64>,
    },

    /// Invalid input parameters
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// URL parsing error
    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),
}

impl Error {
    /// Create an API error from status code and message
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn auth(message: impl Into<String>) -> Self {
        Self::Auth(message.into())
    }

    /// Create a configuration error
    pub fn config(message: impl Into<String>) -> Self {
        Self::Config(message.into())
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }

    /// Create an invalid parameter error
    pub fn invalid_param(message: impl Into<String>) -> Self {
        Self::InvalidParam(message.into())
    }

    /// Create a rate limited error with optional retry-after duration
    pub fn rate_limited(retry_after: Option<u64>) -> Self {
        Self::RateLimited { retry_after }
    }

    /// Check if this is a rate limit error
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }

    /// Get the retry-after duration if this is a rate limit error
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimited { retry_after } => *retry_after,
            _ => None,
        }
    }

    /// Check if this is a not found error
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}
