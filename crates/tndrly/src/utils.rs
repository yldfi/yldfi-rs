//! Utility functions for tndrly
//!
//! Re-exports from yldfi-common for Ethereum address and transaction validation.

pub use yldfi_common::eth::{
    is_valid_address, is_valid_bytes32, is_valid_tx_hash, normalize_address, pad_to_32_bytes,
};

/// Convert a non-negative decimal string of up to 256 bits to minimal `0x` hex
///
/// Returns `None` if `s` is empty, contains a non-digit or exceeds `2^256 - 1`.
pub(crate) fn decimal_to_hex(s: &str) -> Option<String> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    // Big-endian base-256 accumulator: bytes = bytes * 10 + digit
    let mut bytes: Vec<u8> = Vec::new();
    for digit in s.bytes().map(|b| b - b'0') {
        let mut carry = u16::from(digit);
        for byte in bytes.iter_mut().rev() {
            let v = u16::from(*byte) * 10 + carry;
            *byte = (v & 0xff) as u8;
            carry = v >> 8;
        }
        if carry > 0 {
            bytes.insert(0, carry as u8);
        }
        if bytes.len() > 32 {
            return None;
        }
    }
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    let trimmed = hex.trim_start_matches('0');
    Some(format!(
        "0x{}",
        if trimmed.is_empty() { "0" } else { trimmed }
    ))
}

/// Hex-encode a quantity for JSON-RPC style request bodies: decimal strings
/// are converted, `0x` values and anything unparseable pass through unchanged
pub(crate) fn to_hex_quantity(value: &str) -> String {
    if value.starts_with("0x") {
        return value.to_string();
    }
    decimal_to_hex(value).unwrap_or_else(|| value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimal_to_hex_beyond_u128() {
        assert_eq!(decimal_to_hex("0").as_deref(), Some("0x0"));
        assert_eq!(decimal_to_hex("255").as_deref(), Some("0xff"));
        assert_eq!(decimal_to_hex("0001").as_deref(), Some("0x1"));
        // 2^128 overflows u128
        assert_eq!(
            decimal_to_hex("340282366920938463463374607431768211456").as_deref(),
            Some("0x100000000000000000000000000000000")
        );
        // 2^256 - 1
        assert_eq!(
            decimal_to_hex(
                "115792089237316195423570985008687907853269984665640564039457584007913129639935"
            )
            .as_deref(),
            Some("0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff")
        );
        // 2^256 is out of range
        assert_eq!(
            decimal_to_hex(
                "115792089237316195423570985008687907853269984665640564039457584007913129639936"
            ),
            None
        );
        assert_eq!(decimal_to_hex(""), None);
        assert_eq!(decimal_to_hex("1e18"), None);
        assert_eq!(decimal_to_hex("-1"), None);
    }

    #[test]
    fn test_to_hex_quantity() {
        assert_eq!(to_hex_quantity("1000000000000000000"), "0xde0b6b3a7640000");
        assert_eq!(to_hex_quantity("0x10"), "0x10");
        assert_eq!(to_hex_quantity("abc"), "abc");
    }
}
