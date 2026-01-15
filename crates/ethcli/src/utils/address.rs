//! Address resolution utilities
//!
//! Provides functions to resolve addresses from labels, ENS names, or raw addresses.
//! Checks the address book first before falling back to other methods.

use crate::config::AddressBook;
use alloy::primitives::Address;
use std::str::FromStr;

/// Simple address book lookup - returns the resolved address string or the original input.
///
/// This is useful when you just need a string address back (e.g., for passing to cast).
///
/// # Example
/// ```ignore
/// let addr = resolve_label("usdc"); // Returns "0xA0b86991..." if in address book
/// let addr = resolve_label("0x123..."); // Returns the same address
/// ```
pub fn resolve_label(label_or_address: &str) -> String {
    let book = AddressBook::load_default();
    book.resolve(label_or_address)
        .unwrap_or_else(|| label_or_address.to_string())
}

/// Resolve an address from a label or raw address string.
///
/// Returns the parsed Address and an optional label if resolved from address book.
/// Does NOT perform ENS resolution (sync only).
///
/// # Arguments
/// * `input` - Either a hex address (0x...) or an address book label
///
/// # Returns
/// * `Ok((Address, Some(label)))` - If resolved from address book
/// * `Ok((Address, None))` - If parsed directly from hex string
/// * `Err` - If input is neither a valid hex address nor a known label
///
/// # Example
/// ```ignore
/// let (addr, label) = resolve_from_book("usdc")?; // (0xA0b86991..., Some("usdc"))
/// let (addr, label) = resolve_from_book("0xA0b86991...")?; // (0xA0b86991..., None)
/// ```
pub fn resolve_from_book(input: &str) -> anyhow::Result<(Address, Option<String>)> {
    // Check if it's already a valid hex address
    if input.starts_with("0x") && input.len() == 42 {
        let addr = Address::from_str(input)
            .map_err(|e| anyhow::anyhow!("Invalid address '{}': {}", input, e))?;
        return Ok((addr, None));
    }

    // Check address book
    let book = AddressBook::load_default();
    if let Some(entry) = book.get(input) {
        let addr = Address::from_str(&entry.address)
            .map_err(|e| anyhow::anyhow!("Invalid stored address for '{}': {}", input, e))?;
        return Ok((addr, Some(input.to_string())));
    }

    // Not a valid address and not in address book
    Err(anyhow::anyhow!(
        "Unknown address or label: '{}'. Use 'ethcli address add' to save addresses.",
        input
    ))
}

/// Check if an input is a valid hex address format (without validating checksum).
pub fn is_hex_address(input: &str) -> bool {
    input.starts_with("0x")
        && input.len() == 42
        && input[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Check if an input looks like an ENS name.
pub fn is_ens_name(input: &str) -> bool {
    input.contains('.') && !input.starts_with("0x")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_hex_address() {
        assert!(is_hex_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));
        assert!(is_hex_address("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"));
        assert!(!is_hex_address("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB4")); // too short
        assert!(!is_hex_address(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB488"
        )); // too long
        assert!(!is_hex_address("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")); // no 0x prefix
        assert!(!is_hex_address("usdc"));
    }

    #[test]
    fn test_is_ens_name() {
        assert!(is_ens_name("vitalik.eth"));
        assert!(is_ens_name("foo.bar.eth"));
        assert!(!is_ens_name("usdc"));
        assert!(!is_ens_name("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"));
    }

    #[test]
    fn test_resolve_from_book_hex() {
        let result = resolve_from_book("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
        assert!(result.is_ok());
        let (addr, label) = result.unwrap();
        assert_eq!(
            format!("{:#x}", addr),
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        );
        assert!(label.is_none());
    }

    #[test]
    fn test_resolve_from_book_invalid() {
        // Unknown label should error
        let result = resolve_from_book("definitely_not_a_real_label_xyz123");
        assert!(result.is_err());
    }
}
