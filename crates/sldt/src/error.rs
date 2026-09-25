//! Error types for the Solodit API client

use thiserror::Error;

/// Result type alias for Solodit operations
pub type Result<T> = std::result::Result<T, Error>;

/// Server message returned by Solodit when the API key header is absent.
pub const MSG_MISSING_API_KEY: &str = "Missing API key";

/// Server message returned by Solodit when the API key is not recognized.
pub const MSG_INVALID_API_KEY: &str = "Invalid API key";

/// Errors that can occur when using the Solodit API
///
/// Error messages never contain the API key.
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

    /// API returned an error response (e.g. 400 "Invalid request parameters")
    #[error("API error ({status}): {message}")]
    Api {
        /// HTTP status code
        status: u16,
        /// Error message (the server's `message` field when available)
        message: String,
    },

    /// 401 Unauthorized: the server's `message` distinguishes
    /// "Missing API key" from "Invalid API key".
    #[error("Unauthorized (401): {message}{}", unauthorized_hint(.message))]
    Unauthorized {
        /// Server-provided message
        message: String,
    },

    /// 429 Too Many Requests
    #[error("Rate limit exceeded (429): {message}{}", rate_limit_detail(*.limit, *.reset))]
    RateLimited {
        /// Server-provided message
        message: String,
        /// `X-RateLimit-Limit` header, if present
        limit: Option<u32>,
        /// `X-RateLimit-Remaining` header, if present
        remaining: Option<u32>,
        /// `X-RateLimit-Reset` header (Unix timestamp), if present
        reset: Option<u64>,
    },

    /// Finding not found
    #[error("Finding not found: {0}")]
    NotFound(String),

    /// Invalid response format
    #[error("Invalid response format: {0}")]
    InvalidResponse(String),
}

fn unauthorized_hint(message: &str) -> &'static str {
    let lower = message.to_ascii_lowercase();
    if lower.contains("invalid") {
        " - the key was rejected by Solodit. Keys are managed at https://solodit.cyfrin.io \
         (Profile > API Keys); regenerating a key invalidates the previous one, so make sure \
         you are using the most recently generated key."
    } else if lower.contains("missing") {
        " - no X-Cyfrin-API-Key header was received. Set SOLODIT_API_KEY or configure a key \
         (get one at https://solodit.cyfrin.io, Profile > API Keys)."
    } else {
        ""
    }
}

fn rate_limit_detail(limit: Option<u32>, reset: Option<u64>) -> String {
    let mut out = String::new();
    if let Some(limit) = limit {
        out.push_str(&format!(" (limit {limit} requests per window)"));
    }
    if let Some(reset) = reset {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if reset > now {
            out.push_str(&format!("; window resets at {reset} (in {}s)", reset - now));
        } else {
            out.push_str(&format!("; window resets at {reset}"));
        }
    }
    out
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

    /// Create an unauthorized error with the server-provided message
    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self::Unauthorized {
            message: message.into(),
        }
    }

    /// Create a rate limited error
    pub fn rate_limited(
        message: impl Into<String>,
        limit: Option<u32>,
        remaining: Option<u32>,
        reset: Option<u64>,
    ) -> Self {
        Self::RateLimited {
            message: message.into(),
            limit,
            remaining,
            reset,
        }
    }

    /// Create a not found error
    pub fn not_found(slug: impl Into<String>) -> Self {
        Self::NotFound(slug.into())
    }

    /// Create an invalid response error
    pub fn invalid_response(msg: impl Into<String>) -> Self {
        Self::InvalidResponse(msg.into())
    }

    /// HTTP status code associated with this error, if any
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Api { status, .. } => Some(*status),
            Self::Unauthorized { .. } => Some(401),
            Self::RateLimited { .. } => Some(429),
            Self::Http(e) => e.status().map(|s| s.as_u16()),
            _ => None,
        }
    }

    /// Check if this is an unauthorized error
    #[must_use]
    pub fn is_unauthorized(&self) -> bool {
        matches!(self, Self::Unauthorized { .. })
    }

    /// Check if this is a 401 caused by an absent API key
    #[must_use]
    pub fn is_missing_api_key(&self) -> bool {
        matches!(self, Self::Unauthorized { message } if message.eq_ignore_ascii_case(MSG_MISSING_API_KEY))
    }

    /// Check if this is a 401 caused by a rejected (invalid/revoked) API key
    #[must_use]
    pub fn is_invalid_api_key(&self) -> bool {
        matches!(self, Self::Unauthorized { message } if message.eq_ignore_ascii_case(MSG_INVALID_API_KEY))
    }

    /// Check if this is a rate limit error
    #[must_use]
    pub fn is_rate_limited(&self) -> bool {
        matches!(self, Self::RateLimited { .. })
    }

    /// Unix timestamp at which the rate limit window resets (429 only)
    #[must_use]
    pub fn rate_limit_reset(&self) -> Option<u64> {
        match self {
            Self::RateLimited { reset, .. } => *reset,
            _ => None,
        }
    }

    /// Check if this is a not found error
    #[must_use]
    pub fn is_not_found(&self) -> bool {
        matches!(self, Self::NotFound(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_key_hint() {
        let e = Error::unauthorized("Invalid API key");
        let s = e.to_string();
        assert!(s.contains("Invalid API key"));
        assert!(s.contains("Profile > API Keys"));
        assert!(s.contains("invalidates"));
        assert!(e.is_invalid_api_key());
        assert!(!e.is_missing_api_key());
    }

    #[test]
    fn missing_key_hint() {
        let e = Error::unauthorized("Missing API key");
        assert!(e.to_string().contains("X-Cyfrin-API-Key"));
        assert!(e.is_missing_api_key());
    }

    #[test]
    fn rate_limited_display() {
        let e = Error::rate_limited("Rate limit exceeded", Some(20), Some(0), Some(1));
        let s = e.to_string();
        assert!(s.contains("Rate limit exceeded"));
        assert!(s.contains("limit 20"));
        assert!(s.contains("resets at 1"));
        assert_eq!(e.rate_limit_reset(), Some(1));
        assert_eq!(e.status(), Some(429));
    }
}
