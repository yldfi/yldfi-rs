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

/// Sanitize a string to remove potential API keys and secrets
pub fn sanitize_error_message(msg: &str) -> String {
    let mut result = msg.to_string();
    for pattern in API_KEY_PATTERNS.iter() {
        result = pattern
            .replace_all(&result, |caps: &regex::Captures| {
                // Preserve the prefix (?|&) if present
                if let Some(m) = caps.get(1) {
                    format!("{}[REDACTED]", m.as_str())
                } else {
                    "[REDACTED]".to_string()
                }
            })
            .to_string();
    }
    result
}

/// Main error type for the library
#[derive(Error, Debug)]
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

    /// Generic error with context
    #[error("{0}")]
    Other(String),
}

/// RPC-specific errors
#[derive(Error, Debug)]
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

/// Implement RetryableError for RpcError to determine which errors should be retried
impl crate::rpc::retry::RetryableError for RpcError {
    fn is_retryable(&self) -> bool {
        match self {
            // Transient errors that should be retried
            RpcError::Timeout(_) => true,
            RpcError::RateLimited(_) => true,
            RpcError::ConnectionFailed(_) => true,
            RpcError::Http(_) => true,

            // Provider errors - check if transient
            RpcError::Provider(msg) => {
                let msg_lower = msg.to_lowercase();
                // Retry on common transient errors
                msg_lower.contains("timeout")
                    || msg_lower.contains("connection")
                    || msg_lower.contains("temporarily")
                    || msg_lower.contains("503")
                    || msg_lower.contains("502")
                    || msg_lower.contains("504")
                    || msg_lower.contains("network")
            }

            // Non-retryable errors
            RpcError::AllEndpointsFailed => false,
            RpcError::NoHealthyEndpoints => false,
            RpcError::BlockRangeTooLarge { .. } => false,
            RpcError::ResponseTooLarge(_) => false,
            RpcError::InvalidResponse(_) => false,
            RpcError::ProxyNotSupported(_) => false,
        }
    }
}

/// Implement RetryableError for the main Error type
impl crate::rpc::retry::RetryableError for Error {
    fn is_retryable(&self) -> bool {
        match self {
            Error::Rpc(rpc_err) => rpc_err.is_retryable(),
            Error::Io(io_err) => {
                // Retry on transient IO errors
                matches!(
                    io_err.kind(),
                    std::io::ErrorKind::TimedOut
                        | std::io::ErrorKind::ConnectionReset
                        | std::io::ErrorKind::ConnectionAborted
                        | std::io::ErrorKind::Interrupted
                )
            }
            // Other error types are generally not retryable
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
        assert!(sanitized.contains("[REDACTED]"));
        assert!(sanitized.contains("foo=bar"));
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
}
