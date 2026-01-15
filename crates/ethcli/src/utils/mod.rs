//! Shared utility functions
//!
//! Common helpers used across multiple modules.

pub mod address;
pub mod format;

use std::borrow::Cow;

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
}
