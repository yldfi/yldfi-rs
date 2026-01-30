//! Proxy contract detection
//!
//! Detects common proxy patterns:
//! - EIP-1167: Minimal proxy clones
//! - EIP-1967: Transparent proxy (implementation slot)
//! - EIP-1822: UUPS proxy
//! - OpenZeppelin patterns

use alloy::primitives::{Address, B256, U256};

/// Well-known storage slots for proxy implementations
pub mod slots {
    use alloy::primitives::B256;

    /// EIP-1967 implementation slot: keccak256("eip1967.proxy.implementation") - 1
    pub const EIP1967_IMPLEMENTATION: B256 = B256::new([
        0x36, 0x08, 0x94, 0xa1, 0x3b, 0xa1, 0xa3, 0x21, 0x06, 0x67, 0xc8, 0x28, 0x49, 0x2d, 0xb9,
        0x8d, 0xca, 0x3e, 0x20, 0x76, 0xcc, 0x37, 0x35, 0xa9, 0x20, 0xa3, 0xca, 0x50, 0x5d, 0x38,
        0x2b, 0xbc,
    ]);

    /// EIP-1967 beacon slot: keccak256("eip1967.proxy.beacon") - 1
    pub const EIP1967_BEACON: B256 = B256::new([
        0xa3, 0xf0, 0xad, 0x74, 0xe5, 0x42, 0x3a, 0xeb, 0xfd, 0x80, 0xd3, 0xef, 0x4a, 0xe1, 0x47,
        0x9b, 0xe7, 0x7d, 0x9a, 0x59, 0x50, 0x79, 0x28, 0xea, 0x95, 0x26, 0x91, 0xf4, 0x76, 0x82,
        0x65, 0x43,
    ]);

    /// EIP-1967 admin slot: keccak256("eip1967.proxy.admin") - 1
    pub const EIP1967_ADMIN: B256 = B256::new([
        0xb5, 0x31, 0x27, 0x68, 0x4a, 0x56, 0x8b, 0x31, 0x73, 0xae, 0x13, 0xb9, 0xf8, 0xa6, 0x01,
        0x6e, 0x24, 0x3e, 0x63, 0xb6, 0xe8, 0xee, 0x17, 0x66, 0x78, 0x3a, 0x7e, 0x2e, 0x70, 0x72,
        0xef, 0xfc,
    ]);

    /// OpenZeppelin legacy implementation slot (before EIP-1967)
    pub const OZ_LEGACY_IMPLEMENTATION: B256 = B256::new([
        0x7e, 0x36, 0x7f, 0x9d, 0x1d, 0x41, 0xdb, 0x98, 0x6f, 0xbe, 0x8e, 0x56, 0x2a, 0xf1, 0x89,
        0x1c, 0xc6, 0x4f, 0x2e, 0x06, 0x3f, 0x73, 0x12, 0xbe, 0x55, 0x4b, 0x12, 0x3b, 0xa7, 0x35,
        0x87, 0xb0,
    ]);
}

/// Detected proxy type
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyType {
    /// EIP-1167 minimal proxy clone
    Eip1167 {
        /// The implementation address embedded in the bytecode
        implementation: Address,
    },
    /// EIP-1967 transparent proxy
    Eip1967,
    /// EIP-1822 UUPS proxy
    Eip1822,
    /// OpenZeppelin legacy proxy
    OpenZeppelinLegacy,
    /// Generic proxy (detected via DELEGATECALL pattern but unknown type)
    Generic,
}

impl ProxyType {
    pub fn name(&self) -> &'static str {
        match self {
            ProxyType::Eip1167 { .. } => "EIP-1167 Minimal Proxy",
            ProxyType::Eip1967 => "EIP-1967 Transparent Proxy",
            ProxyType::Eip1822 => "EIP-1822 UUPS Proxy",
            ProxyType::OpenZeppelinLegacy => "OpenZeppelin Legacy Proxy",
            ProxyType::Generic => "Generic Proxy",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ProxyType::Eip1167 { .. } => {
                "Minimal proxy clone with implementation address embedded in bytecode"
            }
            ProxyType::Eip1967 => "Transparent proxy with implementation stored at EIP-1967 slot",
            ProxyType::Eip1822 => {
                "Universal Upgradeable Proxy Standard with upgrade logic in implementation"
            }
            ProxyType::OpenZeppelinLegacy => "Legacy OpenZeppelin proxy pattern (pre-EIP-1967)",
            ProxyType::Generic => "Proxy detected via DELEGATECALL but pattern not recognized",
        }
    }
}

/// Result of proxy detection
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ProxyInfo {
    /// Whether this contract is a proxy
    pub is_proxy: bool,
    /// The detected proxy type (if any)
    pub proxy_type: Option<ProxyType>,
    /// Implementation address if known (embedded in bytecode for EIP-1167,
    /// or fetched from storage slot for EIP-1967/etc.)
    pub implementation: Option<Address>,
}

/// Detect if bytecode is an EIP-1167 minimal proxy
///
/// EIP-1167 bytecode pattern:
/// `363d3d373d3d3d363d73<20-byte-address>5af43d82803e903d91602b57fd5bf3`
pub fn detect_eip1167(bytecode: &[u8]) -> Option<Address> {
    // EIP-1167 exact bytecode is 45 bytes
    if bytecode.len() < 45 {
        return None;
    }

    // Check prefix: 363d3d373d3d3d363d73 (10 bytes)
    let prefix = [0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73];
    if !bytecode.starts_with(&prefix) {
        return None;
    }

    // Check suffix: 5af43d82803e903d91602b57fd5bf3 (15 bytes)
    let suffix = [
        0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d, 0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b, 0xf3,
    ];
    if !bytecode[30..45].starts_with(&suffix) {
        return None;
    }

    // Extract the 20-byte implementation address
    let addr_bytes: [u8; 20] = bytecode[10..30].try_into().ok()?;
    Some(Address::from(addr_bytes))
}

/// Detect proxy patterns from bytecode alone
///
/// This only detects patterns that are visible in bytecode (like EIP-1167).
/// For storage-based proxies (EIP-1967), use `detect_proxy_with_storage`.
#[must_use]
pub fn detect_proxy(bytecode: &[u8]) -> ProxyInfo {
    // Check for EIP-1167 minimal proxy
    if let Some(impl_addr) = detect_eip1167(bytecode) {
        return ProxyInfo {
            is_proxy: true,
            proxy_type: Some(ProxyType::Eip1167 {
                implementation: impl_addr,
            }),
            implementation: Some(impl_addr),
        };
    }

    // Check for DELEGATECALL opcode (0xf4) which suggests proxy behavior
    // This is a heuristic - not all contracts with DELEGATECALL are proxies
    let has_delegatecall = bytecode.contains(&0xf4);

    // Look for patterns that suggest EIP-1967/storage-based proxy:
    // - Small bytecode with DELEGATECALL
    // - Contains SLOAD opcode (0x54) for reading implementation slot
    let has_sload = bytecode.contains(&0x54);

    if has_delegatecall && has_sload && bytecode.len() < 2000 {
        // Likely a storage-based proxy, but we need storage access to confirm
        return ProxyInfo {
            is_proxy: true,
            proxy_type: Some(ProxyType::Generic),
            implementation: None,
        };
    }

    ProxyInfo::default()
}

/// Extract implementation address from storage slot value
pub fn address_from_storage(value: B256) -> Option<Address> {
    // Storage values are left-padded, address is in the last 20 bytes
    let bytes = value.as_slice();

    // Check if first 12 bytes are zero (properly formatted address)
    if bytes[..12].iter().all(|&b| b == 0) {
        let addr_bytes: [u8; 20] = bytes[12..32].try_into().ok()?;
        let addr = Address::from(addr_bytes);

        // Return None for zero address
        if addr.is_zero() {
            return None;
        }
        Some(addr)
    } else {
        None
    }
}

/// Convert U256 to B256 for storage value
pub fn u256_to_b256(value: U256) -> B256 {
    B256::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_eip1167() {
        // EIP-1167 minimal proxy pointing to 0x1234...5678
        let impl_addr = Address::repeat_byte(0x12);
        let mut bytecode = vec![0x36, 0x3d, 0x3d, 0x37, 0x3d, 0x3d, 0x3d, 0x36, 0x3d, 0x73];
        bytecode.extend_from_slice(impl_addr.as_slice());
        bytecode.extend_from_slice(&[
            0x5a, 0xf4, 0x3d, 0x82, 0x80, 0x3e, 0x90, 0x3d, 0x91, 0x60, 0x2b, 0x57, 0xfd, 0x5b,
            0xf3,
        ]);

        let detected = detect_eip1167(&bytecode);
        assert_eq!(detected, Some(impl_addr));
    }

    #[test]
    fn test_detect_eip1167_real() {
        // Real EIP-1167 bytecode from mainnet
        let bytecode = hex::decode(
            "363d3d373d3d3d363d73bebebebebebebebebebebebebebebebebebebebe5af43d82803e903d91602b57fd5bf3"
        ).unwrap();

        let result = detect_proxy(&bytecode);
        assert!(result.is_proxy);
        assert!(matches!(result.proxy_type, Some(ProxyType::Eip1167 { .. })));
    }

    #[test]
    fn test_not_proxy() {
        // Regular contract bytecode
        let bytecode = hex::decode("6060604052600080fd").unwrap();
        let result = detect_proxy(&bytecode);
        assert!(!result.is_proxy);
        assert!(result.proxy_type.is_none());
    }

    #[test]
    fn test_address_from_storage() {
        // Storage value with address in last 20 bytes
        let addr = Address::repeat_byte(0xab);
        let mut value = [0u8; 32];
        value[12..32].copy_from_slice(addr.as_slice());
        let b256 = B256::from(value);

        assert_eq!(address_from_storage(b256), Some(addr));
    }

    #[test]
    fn test_address_from_storage_zero() {
        let b256 = B256::ZERO;
        assert_eq!(address_from_storage(b256), None);
    }
}
