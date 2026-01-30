//! Security pattern detection in bytecode

use super::types::{RiskLevel, SecurityAnalysis, SecurityIssue, MAX_BYTECODE_SIZE};
use evm_disassembler::{Opcode, Operation};

/// Security patterns to detect
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecurityPattern {
    /// SELFDESTRUCT opcode - contract can be destroyed
    SelfDestruct,
    /// DELEGATECALL opcode - arbitrary code execution risk
    DelegateCall,
    /// CALLCODE opcode - legacy dangerous call (deprecated)
    CallCode,
    /// ORIGIN opcode - tx.origin auth (honeypot indicator)
    Origin,
    /// CREATE opcode - dynamic contract creation
    Create,
    /// CREATE2 opcode - deterministic contract creation
    Create2,
}

impl SecurityPattern {
    pub fn name(&self) -> &'static str {
        match self {
            SecurityPattern::SelfDestruct => "SELFDESTRUCT",
            SecurityPattern::DelegateCall => "DELEGATECALL",
            SecurityPattern::CallCode => "CALLCODE",
            SecurityPattern::Origin => "ORIGIN",
            SecurityPattern::Create => "CREATE",
            SecurityPattern::Create2 => "CREATE2",
        }
    }

    pub fn risk_level(&self) -> RiskLevel {
        match self {
            SecurityPattern::SelfDestruct => RiskLevel::Critical,
            SecurityPattern::DelegateCall => RiskLevel::High,
            SecurityPattern::CallCode => RiskLevel::High,
            SecurityPattern::Origin => RiskLevel::Medium,
            SecurityPattern::Create => RiskLevel::Medium,
            SecurityPattern::Create2 => RiskLevel::Medium,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SecurityPattern::SelfDestruct => {
                "Contract can be destroyed, potentially locking user funds (note: EIP-6780 limits this post-Cancun)"
            }
            SecurityPattern::DelegateCall => {
                "Arbitrary code execution - can modify contract state via external code"
            }
            SecurityPattern::CallCode => {
                "Deprecated opcode with similar risks to DELEGATECALL"
            }
            SecurityPattern::Origin => {
                "Uses tx.origin for authorization - vulnerable to phishing attacks, honeypot indicator"
            }
            SecurityPattern::Create => "Dynamically creates contracts - review carefully",
            SecurityPattern::Create2 => {
                "Deterministic contract creation - can be used for address grinding"
            }
        }
    }
}

/// Helper function to create a security issue from a pattern and offsets
fn create_issue(pattern: SecurityPattern, offsets: Vec<usize>) -> Option<SecurityIssue> {
    if offsets.is_empty() {
        return None;
    }
    Some(SecurityIssue {
        pattern: pattern.name().to_string(),
        risk: pattern.risk_level(),
        description: pattern.description().to_string(),
        count: offsets.len(),
        offsets,
    })
}

/// Check if an address is a well-known address that shouldn't be flagged
///
/// This includes:
/// - Zero address (0x0000...0000)
/// - Precompiles (0x01-0x09)
/// - Dead/burn addresses (0xdead, 0x000...dead)
/// - ETH indicator address (0xEeee...eeee used by some protocols for native ETH)
fn is_known_address(addr: &[u8]) -> bool {
    if addr.len() != 20 {
        return false;
    }

    // Zero address
    if addr.iter().all(|&b| b == 0) {
        return true;
    }

    // Precompiles (0x01 through 0x09) - first 19 bytes are zero, last byte is 1-9
    if addr[..19].iter().all(|&b| b == 0) && addr[19] <= 9 {
        return true;
    }

    // Dead address variations
    // 0x000000000000000000000000000000000000dEaD
    if addr[..18].iter().all(|&b| b == 0) && addr[18] == 0xde && addr[19] == 0xad {
        return true;
    }

    // 0xdEaD000000000000000000000000000000000000
    if addr[0] == 0xde && addr[1] == 0xad && addr[2..].iter().all(|&b| b == 0) {
        return true;
    }

    // ETH indicator (0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE)
    if addr.iter().all(|&b| b == 0xee || b == 0xEE) {
        return true;
    }

    false
}

/// Detect security issues from pre-disassembled operations
///
/// This is the optimized version that takes pre-disassembled operations
/// to avoid redundant disassembly when used with analyze_bytecode.
#[must_use]
pub fn detect_security_issues_from_operations(operations: &[Operation]) -> SecurityAnalysis {
    let mut dangerous_count = 0;
    let mut hardcoded_addresses = 0;

    // Track findings by pattern
    let mut selfdestruct_offsets = Vec::new();
    let mut delegatecall_offsets = Vec::new();
    let mut callcode_offsets = Vec::new();
    let mut origin_offsets = Vec::new();
    let mut create_offsets = Vec::new();
    let mut create2_offsets = Vec::new();

    for op in operations {
        match op.opcode {
            Opcode::SELFDESTRUCT => {
                selfdestruct_offsets.push(op.offset as usize);
                dangerous_count += 1;
            }
            Opcode::DELEGATECALL => {
                delegatecall_offsets.push(op.offset as usize);
                dangerous_count += 1;
            }
            Opcode::CALLCODE => {
                callcode_offsets.push(op.offset as usize);
                dangerous_count += 1;
            }
            Opcode::ORIGIN => {
                origin_offsets.push(op.offset as usize);
            }
            Opcode::CREATE => {
                create_offsets.push(op.offset as usize);
            }
            Opcode::CREATE2 => {
                create2_offsets.push(op.offset as usize);
            }
            // Check for PUSH20 which often contains hardcoded addresses
            Opcode::PUSH20 => {
                if op.input.len() == 20 {
                    // Skip zero address and well-known addresses
                    if !is_known_address(&op.input) {
                        hardcoded_addresses += 1;
                    }
                }
            }
            _ => {}
        }
    }

    // Create issues using the helper function
    let mut issues: Vec<SecurityIssue> = Vec::new();

    if let Some(issue) = create_issue(SecurityPattern::SelfDestruct, selfdestruct_offsets) {
        issues.push(issue);
    }
    if let Some(issue) = create_issue(SecurityPattern::DelegateCall, delegatecall_offsets) {
        issues.push(issue);
    }
    if let Some(issue) = create_issue(SecurityPattern::CallCode, callcode_offsets) {
        issues.push(issue);
    }
    if let Some(issue) = create_issue(SecurityPattern::Origin, origin_offsets) {
        issues.push(issue);
    }
    if let Some(issue) = create_issue(SecurityPattern::Create, create_offsets) {
        issues.push(issue);
    }
    if let Some(issue) = create_issue(SecurityPattern::Create2, create2_offsets) {
        issues.push(issue);
    }

    // Calculate overall risk level
    let risk_level = calculate_risk_level(&issues);

    SecurityAnalysis {
        risk_level,
        dangerous_opcode_count: dangerous_count,
        hardcoded_address_count: hardcoded_addresses,
        issues,
    }
}

/// Detect security issues in bytecode
///
/// For better performance when also needing disassembly or stats,
/// use `disassemble_raw` and pass the result to `detect_security_issues_from_operations`.
#[must_use]
pub fn detect_security_issues(bytecode: &[u8]) -> SecurityAnalysis {
    if bytecode.is_empty() {
        return SecurityAnalysis::default();
    }

    if bytecode.len() > MAX_BYTECODE_SIZE {
        // Return Unknown risk for oversized bytecode
        return SecurityAnalysis {
            risk_level: RiskLevel::Unknown,
            dangerous_opcode_count: 0,
            hardcoded_address_count: 0,
            issues: Vec::new(),
        };
    }

    let operations = match evm_disassembler::disassemble_bytes(bytecode.to_vec()) {
        Ok(ops) => ops,
        Err(e) => {
            // Log the error but return Unknown risk instead of clean
            eprintln!(
                "Warning: Failed to parse bytecode for security analysis: {}",
                e
            );
            return SecurityAnalysis::parse_failed();
        }
    };

    detect_security_issues_from_operations(&operations)
}

/// Calculate overall risk level based on issues found
fn calculate_risk_level(issues: &[SecurityIssue]) -> RiskLevel {
    let mut max_risk = RiskLevel::Low;

    for issue in issues {
        match issue.risk {
            RiskLevel::Critical => return RiskLevel::Critical,
            RiskLevel::High if max_risk != RiskLevel::Critical => {
                max_risk = RiskLevel::High;
            }
            RiskLevel::Medium if max_risk != RiskLevel::Critical && max_risk != RiskLevel::High => {
                max_risk = RiskLevel::Medium;
            }
            _ => {}
        }
    }

    max_risk
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_bytecode() {
        // Simple bytecode with no dangerous opcodes
        let bytecode = hex::decode("6060604052600080fd").unwrap();
        let analysis = detect_security_issues(&bytecode);

        assert_eq!(analysis.risk_level, RiskLevel::Low);
        assert_eq!(analysis.dangerous_opcode_count, 0);
        assert!(analysis.issues.is_empty());
    }

    #[test]
    fn test_selfdestruct_detection() {
        // Bytecode containing SELFDESTRUCT (0xff)
        let bytecode = hex::decode("6060604052ff").unwrap();
        let analysis = detect_security_issues(&bytecode);

        assert_eq!(analysis.risk_level, RiskLevel::Critical);
        assert_eq!(analysis.dangerous_opcode_count, 1);
        assert!(analysis.issues.iter().any(|i| i.pattern == "SELFDESTRUCT"));
    }

    #[test]
    fn test_delegatecall_detection() {
        // Bytecode containing DELEGATECALL (0xf4)
        let bytecode = hex::decode("6060604052f4").unwrap();
        let analysis = detect_security_issues(&bytecode);

        assert_eq!(analysis.risk_level, RiskLevel::High);
        assert!(analysis.issues.iter().any(|i| i.pattern == "DELEGATECALL"));
    }

    #[test]
    fn test_origin_detection() {
        // Bytecode containing ORIGIN (0x32)
        let bytecode = hex::decode("606060405232").unwrap();
        let analysis = detect_security_issues(&bytecode);

        assert_eq!(analysis.risk_level, RiskLevel::Medium);
        assert!(analysis.issues.iter().any(|i| i.pattern == "ORIGIN"));
    }

    #[test]
    fn test_empty_bytecode() {
        let analysis = detect_security_issues(&[]);

        assert_eq!(analysis.risk_level, RiskLevel::Low);
        assert!(analysis.issues.is_empty());
    }

    #[test]
    fn test_oversized_bytecode() {
        let large_bytecode = vec![0u8; MAX_BYTECODE_SIZE + 1];
        let analysis = detect_security_issues(&large_bytecode);

        assert_eq!(analysis.risk_level, RiskLevel::Unknown);
    }

    #[test]
    fn test_known_addresses() {
        // Zero address
        let zero = [0u8; 20];
        assert!(is_known_address(&zero));

        // Precompile 0x01 (ecrecover)
        let mut precompile = [0u8; 20];
        precompile[19] = 1;
        assert!(is_known_address(&precompile));

        // Precompile 0x09 (blake2f)
        let mut precompile9 = [0u8; 20];
        precompile9[19] = 9;
        assert!(is_known_address(&precompile9));

        // Dead address (0x...dEaD)
        let mut dead = [0u8; 20];
        dead[18] = 0xde;
        dead[19] = 0xad;
        assert!(is_known_address(&dead));

        // ETH indicator (0xEeee...eeee)
        let eth_indicator = [0xee; 20];
        assert!(is_known_address(&eth_indicator));

        // Random address should NOT be known
        let random = [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc,
            0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
        ];
        assert!(!is_known_address(&random));
    }
}
