//! Error types for the Curve API client

use thiserror::Error;

/// Result type for Curve API operations
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when using the Curve API client
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// HTTP request error
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// API returned an error response
    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),
}

impl Error {
    /// Create an API error
    pub fn api(status: u16, message: impl Into<String>) -> Self {
        Self::Api {
            status,
            message: message.into(),
        }
    }

    /// Create an invalid parameter error
    pub fn invalid_param(message: impl Into<String>) -> Self {
        Self::InvalidParam(message.into())
    }
}
