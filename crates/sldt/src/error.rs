//! Error types for the Solodit API client

use thiserror::Error;

/// Result type alias for Solodit operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the Solodit API
#[derive(Debug, Error)]
pub enum Error {
    /// HTTP client initialization failed
    #[error("Client error: {0}")]
    Client(String),

    /// HTTP request failed
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// URL parsing error
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message
        message: String,
    },

    /// Missing or invalid API key
    #[error("Unauthorized: missing or invalid API key")]
    Unauthorized,

    /// Rate limit exceeded
    #[error("Rate limit exceeded (20 requests per 60 seconds)")]
    RateLimited,

    /// Finding not found
    #[error("Finding not found: {0}")]
    NotFound(String),

    /// Invalid response format
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

impl Error {
    /// Create a client initialization error
    pub fn client(message: impl Into<String>) -> Self {
        Self::Client(message.into())
    }

    /// Create an API error
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create an unauthorized error
    #[must_use]
    pub fn unauthorized() -> Self {
        Self::Unauthorized
    }

    /// Create a rate limited error
    #[must_use]
    pub fn rate_limited() -> Self {
        Self::RateLimited
    }

    /// Create a not found error
    pub fn not_found(slug: impl Into<String>) -> Self {
        Self::NotFound(slug.into())
    }

    /// Create an invalid response error
    pub fn invalid_response(msg: impl Into<String>) -> Self {
        Self::InvalidResponse(msg.into())
    }

    /// Check if this is an unauthorized error
    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Unauthorized)
    }

    /// Check if this is a rate limit error
    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited)
    }

    /// Check if this is a not found error
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}
