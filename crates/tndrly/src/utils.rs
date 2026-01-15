//! Utility functions for tndrly

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
/// use tndrly::utils::is_valid_address;
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

    let hex_part = &address[2..];
    if hex_part.len() != 40 {
        return false;
    }

    hex_part.chars().all(|c| c.is_ascii_hexdigit())
}

/// Normalizes an Ethereum address to lowercase with `0x` prefix.
///
/// Returns `None` if the address is not valid.
///
/// # Examples
///
/// ```
/// use tndrly::utils::normalize_address;
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
/// use tndrly::utils::is_valid_tx_hash;
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

    let hex_part = &hash[2..];
    if hex_part.len() != 64 {
        return false;
    }

    hex_part.chars().all(|c| c.is_ascii_hexdigit())
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
        assert!(!is_valid_tx_hash("")); // No prefix
    }
}
