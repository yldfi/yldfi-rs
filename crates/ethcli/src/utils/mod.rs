//! Shared utility functions
//!
//! Common helpers used across multiple modules.

pub mod address;
pub mod format;
pub mod progress;
pub mod table;

use std::borrow::Cow;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Global shared HTTP client (PERF-003 fix: avoid duplicate connection pools)
///
/// This client is shared across AbiFetcher, etherscan::Client, and any other
/// module that needs to make HTTP requests. Using a shared client improves
/// performance by reusing TCP connections and TLS sessions.
///
/// We store Result<Client, String> to handle initialization errors.
static SHARED_HTTP_CLIENT: OnceLock<Result<reqwest::Client, String>> = OnceLock::new();

/// Get or create the shared HTTP client
///
/// The client is configured with:
/// - 30 second timeout
/// - 10 idle connections per host
/// - 90 second idle connection timeout
///
/// Returns an error if client initialization fails (rare, usually TLS backend issues).
pub fn get_shared_http_client() -> Result<&'static reqwest::Client, String> {
    SHARED_HTTP_CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(Duration::from_secs(90))
                .build()
                .map_err(|e| format!("Failed to initialize HTTP client: {}", e))
        })
        .as_ref()
        .map_err(|e| e.clone())
}

// Re-export commonly used items
pub use progress::{is_tty, ProgressBar, Spinner};
pub use table::{format_address, format_number, format_usd, Alignment, Table};

/// Get the current Unix timestamp in seconds
///
/// Returns 0 if the system time is before Unix epoch (shouldn't happen)
#[inline]
pub fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Get the current Unix timestamp in milliseconds
///
/// Returns 0 if the system time is before Unix epoch (shouldn't happen)
#[inline]
pub fn unix_timestamp_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}

/// Validate that a string looks like an Ethereum address
///
/// Returns true if the string starts with "0x" and is 42 characters long
/// with only hex characters (case-insensitive).
pub fn is_valid_eth_address(s: &str) -> bool {
    if !s.starts_with("0x") || s.len() != 42 {
        return false;
    }
    s[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate that a string looks like a transaction hash
///
/// Returns true if the string starts with "0x" and is 66 characters long
/// with only hex characters (case-insensitive).
pub fn is_valid_tx_hash(s: &str) -> bool {
    if !s.starts_with("0x") || s.len() != 66 {
        return false;
    }
    s[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Validate that a string doesn't contain CLI flag injection characters
///
/// Returns true if the string doesn't start with a dash and doesn't contain
/// shell metacharacters that could be dangerous.
pub fn is_safe_cli_value(s: &str) -> bool {
    // Empty strings are safe but useless
    if s.is_empty() {
        return true;
    }
    // Reject strings that look like CLI flags
    if s.starts_with('-') {
        return false;
    }
    // Reject strings with shell metacharacters
    !s.contains(['|', ';', '&', '$', '`', '\\', '\n', '\r'])
}

/// Token metadata (ERC20/ERC721/ERC1155)
#[derive(Debug, Clone)]
pub struct TokenMetadata {
    /// Token name
    pub name: Option<String>,
    /// Token symbol
    pub symbol: Option<String>,
    /// Token decimals (for ERC20)
    pub decimals: Option<u8>,
}

/// Decode a string from ABI-encoded hex data (returned by name() or symbol())
pub fn decode_string_from_hex(hex: &str) -> Option<String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() < 128 {
        // Minimum: offset (32 bytes) + length (32 bytes) = 64 hex chars
        // Try as bytes32 (some tokens return bytes32 for name/symbol)
        return decode_bytes32_string(hex);
    }

    let bytes = hex::decode(hex).ok()?;
    if bytes.len() < 64 {
        return None;
    }

    // ABI-encoded string: offset (32 bytes) + length (32 bytes) + data
    // Skip first 32 bytes (offset), read next 32 bytes as length
    let length = u64::from_be_bytes(bytes[56..64].try_into().ok()?) as usize;

    // Prevent integer overflow: use checked_add for bounds calculation
    let end = 64usize.checked_add(length)?;
    if bytes.len() < end {
        return None;
    }

    let string_bytes = &bytes[64..end];
    String::from_utf8(string_bytes.to_vec())
        .ok()
        .map(|s| s.trim_end_matches('\0').to_string())
        .filter(|s| !s.is_empty())
}

/// Decode a bytes32 as a string (for tokens that return bytes32 for name/symbol)
pub fn decode_bytes32_string(hex: &str) -> Option<String> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.len() != 64 {
        return None;
    }

    let bytes = hex::decode(hex).ok()?;
    // Find the null terminator or use the full 32 bytes
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(32);
    String::from_utf8(bytes[..end].to_vec())
        .ok()
        .filter(|s| !s.is_empty() && s.chars().all(|c| c.is_ascii_graphic() || c == ' '))
}

/// Decode a uint8 from hex data (returned by decimals())
pub fn decode_uint8_from_hex(hex: &str) -> Option<u8> {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    if hex.is_empty() {
        return None;
    }

    // Handle all-zeros case (e.g., "0" or "00...00" for tokens with 0 decimals)
    let trimmed = hex.trim_start_matches('0');
    if trimmed.is_empty() {
        return Some(0);
    }

    // Parse as u64 first, then check if it fits in u8
    let value = u64::from_str_radix(trimmed, 16).ok()?;
    if value > 255 {
        return None;
    }
    Some(value as u8)
}

/// URL-encode a string for safe use in query parameters
/// Only encodes characters that are unsafe in URLs
pub fn urlencoding_encode(input: &str) -> Cow<'_, str> {
    let needs_encoding = input
        .bytes()
        .any(|b| !matches!(b, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~'));

    if !needs_encoding {
        return Cow::Borrowed(input);
    }

    let mut encoded = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    Cow::Owned(encoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_string_from_hex() {
        // "USDC" encoded as ABI string
        let hex = "0x0000000000000000000000000000000000000000000000000000000000000020000000000000000000000000000000000000000000000000000000000000000455534443000000000000000000000000000000000000000000000000000000";
        let result = decode_string_from_hex(hex);
        assert_eq!(result, Some("USDC".to_string()));
    }

    #[test]
    fn test_decode_string_from_hex_invalid() {
        // Invalid hex characters
        assert_eq!(decode_string_from_hex("0xGGGG"), None);
        // Too short
        assert_eq!(decode_string_from_hex("0x"), None);
        // Empty
        assert_eq!(decode_string_from_hex(""), None);
        // Odd length hex (invalid)
        assert_eq!(decode_string_from_hex("0x123"), None);
    }

    #[test]
    fn test_decode_string_from_hex_short_fallback_to_bytes32() {
        // Short hex should try bytes32 decoding
        // "ETH" as bytes32
        let hex = "4554480000000000000000000000000000000000000000000000000000000000";
        assert_eq!(decode_string_from_hex(hex), Some("ETH".to_string()));
    }

    #[test]
    fn test_decode_bytes32_string() {
        // "MKR" as bytes32 (null-padded)
        let hex = "4d4b520000000000000000000000000000000000000000000000000000000000";
        let result = decode_bytes32_string(hex);
        assert_eq!(result, Some("MKR".to_string()));
    }

    #[test]
    fn test_decode_bytes32_string_invalid() {
        // Wrong length (not 64 hex chars)
        assert_eq!(decode_bytes32_string("4d4b52"), None);
        assert_eq!(decode_bytes32_string(""), None);
        // Invalid hex
        assert_eq!(
            decode_bytes32_string(
                "GGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGGG"
            ),
            None
        );
        // All nulls (empty string)
        assert_eq!(
            decode_bytes32_string(
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
            None
        );
    }

    #[test]
    fn test_decode_bytes32_string_non_ascii() {
        // Non-ASCII bytes should return None (filter rejects non-ASCII graphic chars)
        // 0x80 is not ASCII graphic
        let hex = "8000000000000000000000000000000000000000000000000000000000000000";
        assert_eq!(decode_bytes32_string(hex), None);
    }

    #[test]
    fn test_decode_uint8() {
        assert_eq!(decode_uint8_from_hex("0x06"), Some(6));
        assert_eq!(decode_uint8_from_hex("0x12"), Some(18));
        assert_eq!(decode_uint8_from_hex("0x00"), Some(0));
        assert_eq!(decode_uint8_from_hex("0xff"), Some(255));
    }

    #[test]
    fn test_decode_uint8_edge_cases() {
        // Values > 255 should return None
        assert_eq!(decode_uint8_from_hex("0x100"), None);
        assert_eq!(decode_uint8_from_hex("0x1000"), None);
        assert_eq!(decode_uint8_from_hex("0xffff"), None);
        // Empty should return None
        assert_eq!(decode_uint8_from_hex(""), None);
        assert_eq!(decode_uint8_from_hex("0x"), None);
        // Invalid hex
        assert_eq!(decode_uint8_from_hex("0xGG"), None);
        // Leading zeros should work
        assert_eq!(decode_uint8_from_hex("0x0000000000000012"), Some(18));
    }

    #[test]
    fn test_urlencoding_encode() {
        assert_eq!(urlencoding_encode("hello"), Cow::Borrowed("hello"));
        assert_eq!(
            urlencoding_encode("hello world"),
            Cow::<str>::Owned("hello%20world".to_string())
        );
        assert_eq!(
            urlencoding_encode("a=b&c=d"),
            Cow::<str>::Owned("a%3Db%26c%3Dd".to_string())
        );
    }

    #[test]
    fn test_urlencoding_encode_unicode() {
        // Unicode characters should be percent-encoded
        assert_eq!(
            urlencoding_encode("café"),
            Cow::<str>::Owned("caf%C3%A9".to_string())
        );
        assert_eq!(
            urlencoding_encode("日本"),
            Cow::<str>::Owned("%E6%97%A5%E6%9C%AC".to_string())
        );
        // Emoji
        assert_eq!(
            urlencoding_encode("🦀"),
            Cow::<str>::Owned("%F0%9F%A6%80".to_string())
        );
    }

    #[test]
    fn test_urlencoding_encode_special_chars() {
        // Reserved URL characters
        assert_eq!(
            urlencoding_encode("foo/bar"),
            Cow::<str>::Owned("foo%2Fbar".to_string())
        );
        assert_eq!(
            urlencoding_encode("a?b=c"),
            Cow::<str>::Owned("a%3Fb%3Dc".to_string())
        );
        assert_eq!(
            urlencoding_encode("#hash"),
            Cow::<str>::Owned("%23hash".to_string())
        );
        // Unreserved characters should NOT be encoded
        assert_eq!(urlencoding_encode("a-b_c.d~e"), Cow::Borrowed("a-b_c.d~e"));
    }

    #[test]
    fn test_urlencoding_encode_empty() {
        assert_eq!(urlencoding_encode(""), Cow::Borrowed(""));
    }

    #[test]
    fn test_unix_timestamp() {
        let ts = unix_timestamp_secs();
        // Should be after 2024-01-01 and before 2100-01-01
        assert!(ts > 1704067200); // 2024-01-01
        assert!(ts < 4102444800); // 2100-01-01

        let ms = unix_timestamp_ms();
        // Milliseconds should be roughly 1000x the seconds
        assert!(ms > (ts as u128) * 1000 - 1000);
        assert!(ms < (ts as u128) * 1000 + 1000);
    }

    #[test]
    fn test_is_valid_eth_address() {
        // Valid addresses
        assert!(is_valid_eth_address(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        ));
        assert!(is_valid_eth_address(
            "0x0000000000000000000000000000000000000000"
        ));
        assert!(is_valid_eth_address(
            "0xffffffffffffffffffffffffffffffffffffffff"
        ));

        // Invalid addresses
        assert!(!is_valid_eth_address(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB4"
        )); // Too short
        assert!(!is_valid_eth_address(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48a"
        )); // Too long
        assert!(!is_valid_eth_address(
            "A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        )); // No 0x
        assert!(!is_valid_eth_address(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB4G"
        )); // Invalid hex
        assert!(!is_valid_eth_address("")); // Empty
    }

    #[test]
    fn test_is_valid_tx_hash() {
        // Valid hashes
        assert!(is_valid_tx_hash(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        ));
        assert!(is_valid_tx_hash(
            "0x0000000000000000000000000000000000000000000000000000000000000000"
        ));

        // Invalid hashes
        assert!(!is_valid_tx_hash("0x1234567890abcdef")); // Too short
        assert!(!is_valid_tx_hash(
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        )); // No 0x
        assert!(!is_valid_tx_hash(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeg"
        )); // Invalid hex
        assert!(!is_valid_tx_hash("")); // Empty
    }

    #[test]
    fn test_is_safe_cli_value() {
        // Safe values
        assert!(is_safe_cli_value("hello"));
        assert!(is_safe_cli_value("0x1234"));
        assert!(is_safe_cli_value("https://eth.llamarpc.com"));
        assert!(is_safe_cli_value("100000000000"));
        assert!(is_safe_cli_value("transfer(address,uint256)"));
        assert!(is_safe_cli_value("")); // Empty is safe

        // Unsafe values (flag injection)
        assert!(!is_safe_cli_value("--help"));
        assert!(!is_safe_cli_value("-h"));
        assert!(!is_safe_cli_value("-"));

        // Unsafe values (shell metacharacters)
        assert!(!is_safe_cli_value("foo|bar"));
        assert!(!is_safe_cli_value("foo;bar"));
        assert!(!is_safe_cli_value("foo&bar"));
        assert!(!is_safe_cli_value("$PATH"));
        assert!(!is_safe_cli_value("`whoami`"));
        assert!(!is_safe_cli_value("foo\\nbar"));
        assert!(!is_safe_cli_value("foo\nbar"));
    }
}
