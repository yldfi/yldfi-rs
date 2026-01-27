//! Ethereum utilities for address and transaction hash validation
//!
//! This module provides:
//! - Validation functions: `is_valid_address`, `is_valid_tx_hash`, etc.
//! - Newtype wrappers: `Address`, `TxHash` for type-safe validated values
//!
//! # Example
//!
//! ```
//! use yldfi_common::eth::{Address, TxHash};
//! use std::str::FromStr;
//!
//! // Parse and validate an address
//! let addr = Address::from_str("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").unwrap();
//! assert_eq!(addr.as_str(), "0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
//!
//! // Invalid addresses are rejected
//! assert!(Address::from_str("invalid").is_err());
//! ```

use std::fmt;
use std::str::FromStr;

// ============================================================================
// Address Newtype
// ============================================================================

/// A validated Ethereum address.
///
/// This type guarantees that the contained string is a valid, normalized
/// Ethereum address (lowercase with `0x` prefix).
///
/// # Example
///
/// ```
/// use yldfi_common::eth::Address;
/// use std::str::FromStr;
///
/// let addr = Address::from_str("0xD8DA6BF26964AF9D7EED9E03E53415D37AA96045").unwrap();
/// assert_eq!(addr.as_str(), "0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Address(String);

impl Address {
    /// Create a new Address from a string, validating and normalizing it.
    ///
    /// Returns `None` if the address is invalid.
    #[must_use]
    pub fn new(address: &str) -> Option<Self> {
        normalize_address(address).map(Self)
    }

    /// Get the address as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }

    /// The zero address (0x0000...0000).
    #[must_use]
    pub fn zero() -> Self {
        Self("0x0000000000000000000000000000000000000000".to_string())
    }

    /// Check if this is the zero address.
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == "0x0000000000000000000000000000000000000000"
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Address {
    type Err = AddressParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or(AddressParseError)
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Error returned when parsing an invalid address.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressParseError;

impl fmt::Display for AddressParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid Ethereum address")
    }
}

impl std::error::Error for AddressParseError {}

// ============================================================================
// TxHash Newtype
// ============================================================================

/// A validated Ethereum transaction hash (32 bytes).
///
/// This type guarantees that the contained string is a valid, normalized
/// transaction hash (lowercase with `0x` prefix, 66 characters total).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TxHash(String);

impl TxHash {
    /// Create a new `TxHash` from a string, validating and normalizing it.
    ///
    /// Returns `None` if the hash is invalid.
    #[must_use]
    pub fn new(hash: &str) -> Option<Self> {
        if is_valid_tx_hash(hash) {
            Some(Self(hash.to_lowercase()))
        } else {
            None
        }
    }

    /// Get the hash as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert to the inner String.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl fmt::Display for TxHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for TxHash {
    type Err = TxHashParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s).ok_or(TxHashParseError)
    }
}

impl AsRef<str> for TxHash {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Error returned when parsing an invalid transaction hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TxHashParseError;

impl fmt::Display for TxHashParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid transaction hash")
    }
}

impl std::error::Error for TxHashParseError {}

// ============================================================================
// Validation Functions
// ============================================================================

/// Validates an Ethereum address format.
///
/// Returns `true` if the address:
/// - Is a 40-character hex string prefixed with `0x` (42 chars total)
/// - Contains only valid hexadecimal characters (0-9, a-f, A-F)
///
/// Note: This is format validation only. It does not perform checksum validation.
///
/// # Examples
///
/// ```
/// use yldfi_common::eth::is_valid_address;
///
/// assert!(is_valid_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"));
/// assert!(is_valid_address("0x0000000000000000000000000000000000000000"));
/// assert!(!is_valid_address("invalid"));
/// assert!(!is_valid_address("0x123")); // Too short
/// ```
#[must_use]
pub fn is_valid_address(address: &str) -> bool {
    if !address.starts_with("0x") && !address.starts_with("0X") {
        return false;
    }

    if address.len() != 42 {
        return false;
    }

    address[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalizes an Ethereum address to lowercase with `0x` prefix.
///
/// Returns `None` if the address is not valid.
///
/// # Examples
///
/// ```
/// use yldfi_common::eth::normalize_address;
///
/// assert_eq!(
///     normalize_address("0xD8DA6BF26964AF9D7EED9E03E53415D37AA96045"),
///     Some("0xd8da6bf26964af9d7eed9e03e53415d37aa96045".to_string())
/// );
/// assert_eq!(normalize_address("invalid"), None);
/// ```
#[must_use]
pub fn normalize_address(address: &str) -> Option<String> {
    if !is_valid_address(address) {
        return None;
    }
    Some(address.to_lowercase())
}

/// Validates a transaction hash format.
///
/// Returns `true` if the hash:
/// - Is a 64-character hex string prefixed with `0x` (66 chars total)
/// - Contains only valid hexadecimal characters
///
/// # Examples
///
/// ```
/// use yldfi_common::eth::is_valid_tx_hash;
///
/// assert!(is_valid_tx_hash("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"));
/// assert!(!is_valid_tx_hash("0x123")); // Too short
/// assert!(!is_valid_tx_hash("invalid"));
/// ```
#[must_use]
pub fn is_valid_tx_hash(hash: &str) -> bool {
    if !hash.starts_with("0x") && !hash.starts_with("0X") {
        return false;
    }

    if hash.len() != 66 {
        return false;
    }

    hash[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Validates a bytes32 hash format (same as tx hash).
///
/// Alias for `is_valid_tx_hash` for semantic clarity when validating
/// block hashes, storage slots, or other 32-byte values.
#[must_use]
pub fn is_valid_bytes32(hash: &str) -> bool {
    is_valid_tx_hash(hash)
}

/// Pads a hex string to 32 bytes (64 hex chars + 0x prefix).
///
/// Useful for storage slots and other 32-byte values.
///
/// # Examples
///
/// ```
/// use yldfi_common::eth::pad_to_32_bytes;
///
/// assert_eq!(pad_to_32_bytes("0x1"), "0x0000000000000000000000000000000000000000000000000000000000000001");
/// assert_eq!(pad_to_32_bytes("0x64"), "0x0000000000000000000000000000000000000000000000000000000000000064");
/// assert_eq!(pad_to_32_bytes("1"), "0x0000000000000000000000000000000000000000000000000000000000000001");
/// ```
#[must_use]
pub fn pad_to_32_bytes(value: &str) -> String {
    let hex = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
        .unwrap_or(value);
    format!("0x{hex:0>64}")
}

/// HTTP status code classification for API responses
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatusKind {
    /// 2xx - Success
    Success,
    /// 400 - Bad request
    BadRequest,
    /// 401 - Unauthorized
    Unauthorized,
    /// 403 - Forbidden
    Forbidden,
    /// 404 - Not found
    NotFound,
    /// 429 - Rate limited
    RateLimited,
    /// 5xx - Server error
    ServerError,
    /// Other status codes
    Other,
}

impl HttpStatusKind {
    /// Classify an HTTP status code
    #[must_use]
    pub fn from_status(status: u16) -> Self {
        match status {
            200..=299 => Self::Success,
            400 => Self::BadRequest,
            401 => Self::Unauthorized,
            403 => Self::Forbidden,
            404 => Self::NotFound,
            429 => Self::RateLimited,
            500..=599 => Self::ServerError,
            _ => Self::Other,
        }
    }

    /// Check if this status is retryable
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::RateLimited | Self::ServerError)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_addresses() {
        assert!(is_valid_address(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        ));
        assert!(is_valid_address(
            "0x0000000000000000000000000000000000000000"
        ));
        assert!(is_valid_address(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        ));
        assert!(is_valid_address(
            "0XABCDEF1234567890ABCDEF1234567890ABCDEF12"
        ));
    }

    #[test]
    fn test_invalid_addresses() {
        assert!(!is_valid_address(""));
        assert!(!is_valid_address("0x"));
        assert!(!is_valid_address("0x123"));
        assert!(!is_valid_address("invalid"));
        assert!(!is_valid_address(
            "d8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
        )); // No prefix
        assert!(!is_valid_address(
            "0xGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"
        )); // Invalid hex
    }

    #[test]
    fn test_normalize_address() {
        assert_eq!(
            normalize_address("0xD8DA6BF26964AF9D7EED9E03E53415D37AA96045"),
            Some("0xd8da6bf26964af9d7eed9e03e53415d37aa96045".to_string())
        );
        assert_eq!(normalize_address("invalid"), None);
    }

    #[test]
    fn test_valid_tx_hashes() {
        assert!(is_valid_tx_hash(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        ));
        assert!(is_valid_tx_hash(
            "0xABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890"
        ));
    }

    #[test]
    fn test_invalid_tx_hashes() {
        assert!(!is_valid_tx_hash("0x123"));
        assert!(!is_valid_tx_hash("invalid"));
        assert!(!is_valid_tx_hash(""));
    }

    #[test]
    fn test_pad_to_32_bytes() {
        assert_eq!(
            pad_to_32_bytes("0x1"),
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            pad_to_32_bytes("1"),
            "0x0000000000000000000000000000000000000000000000000000000000000001"
        );
        assert_eq!(
            pad_to_32_bytes("0x64"),
            "0x0000000000000000000000000000000000000000000000000000000000000064"
        );
    }

    #[test]
    fn test_http_status_kind() {
        assert_eq!(HttpStatusKind::from_status(200), HttpStatusKind::Success);
        assert_eq!(
            HttpStatusKind::from_status(401),
            HttpStatusKind::Unauthorized
        );
        assert_eq!(
            HttpStatusKind::from_status(429),
            HttpStatusKind::RateLimited
        );
        assert_eq!(
            HttpStatusKind::from_status(500),
            HttpStatusKind::ServerError
        );
        assert_eq!(
            HttpStatusKind::from_status(503),
            HttpStatusKind::ServerError
        );

        assert!(HttpStatusKind::RateLimited.is_retryable());
        assert!(HttpStatusKind::ServerError.is_retryable());
        assert!(!HttpStatusKind::Unauthorized.is_retryable());
    }

    #[test]
    fn test_address_newtype() {
        // Valid address
        let addr = Address::new("0xD8DA6BF26964AF9D7EED9E03E53415D37AA96045");
        assert!(addr.is_some());
        let addr = addr.unwrap();
        assert_eq!(addr.as_str(), "0xd8da6bf26964af9d7eed9e03e53415d37aa96045");
        assert!(!addr.is_zero());

        // Invalid address
        assert!(Address::new("invalid").is_none());
        assert!(Address::new("0x123").is_none());

        // Zero address
        let zero = Address::zero();
        assert!(zero.is_zero());

        // FromStr
        let parsed: Address = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
            .parse()
            .unwrap();
        assert_eq!(addr, parsed);

        // Invalid FromStr
        let err = "invalid".parse::<Address>();
        assert!(err.is_err());
    }

    #[test]
    fn test_tx_hash_newtype() {
        // Valid hash
        let hash =
            TxHash::new("0x1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF1234567890ABCDEF");
        assert!(hash.is_some());
        let hash = hash.unwrap();
        assert_eq!(
            hash.as_str(),
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );

        // Invalid hash
        assert!(TxHash::new("invalid").is_none());
        assert!(TxHash::new("0x123").is_none());

        // FromStr
        let parsed: TxHash = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
            .parse()
            .unwrap();
        assert_eq!(hash, parsed);
    }
}
