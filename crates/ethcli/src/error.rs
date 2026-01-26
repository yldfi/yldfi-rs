//! Error types for ethcli

use regex::Regex;
use std::sync::LazyLock;
use thiserror::Error;

/// Regex patterns for API keys that should be sanitized from error messages
static API_KEY_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        // Query parameter patterns
        Regex::new(r"(?i)(\?|&)(api_?key|apikey|key|token|secret|auth|password)=[^&\s]+")
            .expect("valid query param regex"),
        // Path segment patterns (e.g., /v1/key123abc/)
        Regex::new(r"/v\d+/[a-zA-Z0-9_-]{20,}(/|$)").expect("valid path segment regex"),
        // Bearer tokens
        Regex::new(r"(?i)bearer\s+[a-zA-Z0-9_.-]+").expect("valid bearer token regex"),
    ]
});

/// Redacted placeholder for sensitive data in error messages.
pub const REDACTED: &str = "[REDACTED]";

/// Sanitize a string to remove potential API keys and secrets.
///
/// Replaces detected API keys, tokens, and secrets with [`REDACTED`].
/// Returns a `Cow` to avoid allocation when the input contains no sensitive data.
pub fn sanitize_error_message(msg: &str) -> std::borrow::Cow<'_, str> {
    // First, check if any pattern matches to avoid allocation
    let needs_sanitization = API_KEY_PATTERNS.iter().any(|p| p.is_match(msg));
    if !needs_sanitization {
        return std::borrow::Cow::Borrowed(msg);
    }

    // Only allocate if we need to sanitize
    let mut result = msg.to_string();
    for pattern in API_KEY_PATTERNS.iter() {
        result = pattern
            .replace_all(&result, |caps: &regex::Captures| {
                // Preserve the prefix (?|&) if present
                if let Some(m) = caps.get(1) {
                    format!("{}{}", m.as_str(), REDACTED)
                } else {
                    REDACTED.to_string()
                }
            })
            .to_string();
    }
    std::borrow::Cow::Owned(result)
}

/// Keywords that indicate a transient/retryable error when found in error messages.
const TRANSIENT_ERROR_KEYWORDS: &[&str] = &[
    "timeout",
    "connection",
    "temporarily",
    "503", // Service Unavailable
    "502", // Bad Gateway
    "504", // Gateway Timeout
    "network",
];

/// Check if an error message indicates a transient error that should be retried.
///
/// This is used for provider errors where we don't have structured error types
/// and must infer retryability from the error message content.
fn is_transient_error_message(msg: &str) -> bool {
    let msg_lower = msg.to_lowercase();
    TRANSIENT_ERROR_KEYWORDS
        .iter()
        .any(|keyword| msg_lower.contains(keyword))
}

/// Transient IO error kinds that should be retried.
const TRANSIENT_IO_ERROR_KINDS: &[std::io::ErrorKind] = &[
    std::io::ErrorKind::TimedOut,
    std::io::ErrorKind::ConnectionReset,
    std::io::ErrorKind::ConnectionAborted,
    std::io::ErrorKind::Interrupted,
];

/// Check if an IO error is transient and should be retried.
fn is_transient_io_error(err: &std::io::Error) -> bool {
    TRANSIENT_IO_ERROR_KINDS.contains(&err.kind())
}

/// Main error type for the library
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// RPC-related errors
    #[error("RPC error: {0}")]
    Rpc(#[from] RpcError),

    /// ABI-related errors
    #[error("ABI error: {0}")]
    Abi(#[from] AbiError),

    /// Configuration errors
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),

    /// Output errors
    #[error("Output error: {0}")]
    Output(#[from] OutputError),

    /// Checkpoint errors
    #[error("Checkpoint error: {0}")]
    Checkpoint(#[from] CheckpointError),

    /// Generic IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Generic error for cases that don't fit other categories.
    ///
    /// When using this variant, prefer including context about the operation
    /// that failed, e.g., `Error::Other(format!("Failed to parse address: {}", input))`
    /// rather than just passing through a raw error message.
    #[error("{0}")]
    Other(String),
}

/// RPC-specific errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum RpcError {
    #[error("All endpoints failed for request")]
    AllEndpointsFailed,

    #[error("No endpoints configured. Add endpoints with: ethcli endpoints add <url>")]
    NoHealthyEndpoints,

    #[error("Request timeout after {0}ms")]
    Timeout(u64),

    #[error("Rate limited by endpoint: {0}")]
    RateLimited(String),

    #[error("Block range too large: max {max}, requested {requested}")]
    BlockRangeTooLarge { max: u64, requested: u64 },

    #[error("Response too large: {0} logs exceed limit")]
    ResponseTooLarge(usize),

    #[error("Invalid response from endpoint: {0}")]
    InvalidResponse(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Proxy support not implemented: {0}")]
    ProxyNotSupported(String),
}

/// ABI-related errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AbiError {
    #[error("Failed to fetch ABI from Etherscan: {0}")]
    EtherscanFetch(String),

    #[error("Contract not verified on Etherscan: {0}")]
    ContractNotVerified(String),

    #[error("Invalid event signature: {0}")]
    InvalidEventSignature(String),

    #[error("Failed to parse ABI: {0}")]
    ParseError(String),

    #[error("Event not found in ABI: {0}")]
    EventNotFound(String),

    #[error("Failed to decode log: {0}")]
    DecodeError(String),

    #[error("ABI file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to initialize HTTP client: {0}")]
    HttpClientInit(String),
}

/// Configuration errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ConfigError {
    #[error("Invalid config file: {0}")]
    InvalidFile(String),

    #[error("Invalid address format: {0}")]
    InvalidAddress(String),

    #[error("Invalid block number: {0}")]
    InvalidBlockNumber(String),

    #[error("Invalid chain: {0}")]
    InvalidChain(String),

    #[error("Missing required field: {0}")]
    MissingField(String),

    #[error("Config file parse error: {0}")]
    ParseError(#[from] toml::de::Error),
}

/// Output-related errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum OutputError {
    #[error("Failed to write JSON: {0}")]
    JsonWrite(String),

    #[error("Failed to write CSV: {0}")]
    CsvWrite(String),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Failed to create output file: {0}")]
    FileCreate(String),

    #[error("Unsupported format: {0}")]
    UnsupportedFormat(String),
}

/// Checkpoint-related errors
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CheckpointError {
    #[error("Failed to read checkpoint: {0}")]
    ReadError(String),

    #[error("Failed to write checkpoint: {0}")]
    WriteError(String),

    #[error("Checkpoint corrupted: {0}")]
    Corrupted(String),

    #[error("Checkpoint version mismatch: expected {expected}, found {found}")]
    VersionMismatch { expected: u32, found: u32 },
}

/// Result type alias for the library
pub type Result<T> = std::result::Result<T, Error>;

/// Newtype wrapper for "other" errors that don't fit specific categories.
///
/// This wrapper provides better type safety than a raw `String` and allows
/// for future additions (like source error chaining) without breaking changes.
///
/// # Example
/// ```
/// use ethcli::error::{Error, OtherError};
///
/// let err: Error = OtherError::new("something went wrong").into();
/// let err2: Error = OtherError::with_context("parsing", "invalid format").into();
/// ```
#[derive(Debug, Clone)]
pub struct OtherError {
    message: String,
}

impl OtherError {
    /// Create a new OtherError with a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Create an OtherError with context about the operation that failed
    pub fn with_context(operation: &str, detail: impl std::fmt::Display) -> Self {
        Self {
            message: format!("{}: {}", operation, detail),
        }
    }

    /// Get the error message
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl std::fmt::Display for OtherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OtherError {}

impl From<OtherError> for Error {
    fn from(e: OtherError) -> Self {
        Error::Other(e.message)
    }
}

/// Implement RetryableError for RpcError to determine which errors should be retried.
impl crate::rpc::retry::RetryableError for RpcError {
    fn is_retryable(&self) -> bool {
        match self {
            // Explicit transient errors - always retry
            RpcError::Timeout(_)
            | RpcError::RateLimited(_)
            | RpcError::ConnectionFailed(_)
            | RpcError::Http(_) => true,

            // Provider errors - infer from message content
            RpcError::Provider(msg) => is_transient_error_message(msg),

            // Terminal errors - don't retry (would fail again)
            RpcError::AllEndpointsFailed
            | RpcError::NoHealthyEndpoints
            | RpcError::BlockRangeTooLarge { .. }
            | RpcError::ResponseTooLarge(_)
            | RpcError::InvalidResponse(_)
            | RpcError::ProxyNotSupported(_) => false,
        }
    }
}

/// Implement RetryableError for the main Error type.
impl crate::rpc::retry::RetryableError for Error {
    fn is_retryable(&self) -> bool {
        match self {
            Error::Rpc(rpc_err) => rpc_err.is_retryable(),
            Error::Io(io_err) => is_transient_io_error(io_err),
            // Other error types (Config, Abi, Checkpoint, etc.) are not retryable
            _ => false,
        }
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Other(s)
    }
}

impl From<&str> for Error {
    fn from(s: &str) -> Self {
        Error::Other(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_api_key_query_param() {
        let url = "https://api.example.com?apikey=secret123&foo=bar";
        let sanitized = sanitize_error_message(url);
        assert!(!sanitized.contains("secret123"));
        assert!(sanitized.contains(REDACTED));
        assert!(sanitized.contains("foo=bar"));
    }

    #[test]
    fn test_transient_error_detection() {
        // Should be detected as transient
        assert!(is_transient_error_message("Connection timeout after 30s"));
        assert!(is_transient_error_message("503 Service Unavailable"));
        assert!(is_transient_error_message("network error"));
        assert!(is_transient_error_message(
            "Service temporarily unavailable"
        ));

        // Should NOT be detected as transient
        assert!(!is_transient_error_message("Invalid address format"));
        assert!(!is_transient_error_message("Unauthorized: bad API key"));
        assert!(!is_transient_error_message("Block not found"));
    }

    #[test]
    fn test_sanitize_multiple_keys() {
        let url = "https://api.example.com?key=abc123&token=xyz789";
        let sanitized = sanitize_error_message(url);
        assert!(!sanitized.contains("abc123"));
        assert!(!sanitized.contains("xyz789"));
    }

    #[test]
    fn test_sanitize_path_segment_key() {
        let url = "https://eth-mainnet.alchemyapi.io/v2/asdflkjhasdf1234567890123456";
        let sanitized = sanitize_error_message(url);
        assert!(!sanitized.contains("asdflkjhasdf1234567890123456"));
    }

    #[test]
    fn test_sanitize_no_key() {
        let msg = "Connection failed: timeout after 30s";
        let sanitized = sanitize_error_message(msg);
        assert_eq!(sanitized, msg);
    }

    #[test]
    fn test_sanitize_bearer_token() {
        let msg = "Request failed with Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let sanitized = sanitize_error_message(msg);
        assert!(!sanitized.contains("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9"));
    }

    #[test]
    fn test_api_key_patterns_compile() {
        // Force evaluation of the LazyLock to ensure all regex patterns compile.
        // This catches malformed patterns at test time rather than runtime.
        assert!(
            !API_KEY_PATTERNS.is_empty(),
            "API key patterns should be defined"
        );
        // Verify each pattern can match (doesn't panic)
        for pattern in API_KEY_PATTERNS.iter() {
            let _ = pattern.is_match("test string");
        }
    }
}
