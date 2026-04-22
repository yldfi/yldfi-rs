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
/// - Address mask (0xffff...ffff used for address masking operations)
fn is_known_address(addr: &[u8]) -> bool {
    if addr.len() != 20 {
        return false;
    }

    // Zero address
    if addr.iter().all(|&b| b == 0) {
        return true;
    }

    // Address mask (0xffffffffffffffffffffffffffffffffffffffff)
    // Used for masking operations like `addr & 0xfff...fff`
    if addr.iter().all(|&b| b == 0xff) {
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
    // Note: 0xee and 0xEE are the same byte value
    if addr.iter().all(|&b| b == 0xee) {
        return true;
    }

    false
}

/// Detect security issues from pre-disassembled operations
///
/// This is the optimized version that takes pre-disassembled operations
/// to avoid redundant disassembly when used with analyze_bytecode.
///
/// Note: We stop scanning after encountering INVALID opcodes followed by
/// non-executable patterns, as this typically marks the start of the
/// contract metadata section (CBOR-encoded compiler info, etc.)
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

    // Track when we've entered the metadata section (after runtime code ends)
    // Heuristic: After INVALID opcode, if next opcode is not JUMPDEST, we're in metadata
    // (INVALID followed by JUMPDEST is a normal revert handler pattern)
    let mut saw_invalid = false;
    let mut in_metadata_section = false;

    for op in operations {
        // Check if we've entered metadata section
        if saw_invalid {
            // INVALID followed by JUMPDEST is a revert handler - still in code
            // INVALID followed by anything else likely means metadata section
            if op.opcode != Opcode::JUMPDEST {
                in_metadata_section = true;
            }
            saw_invalid = false;
        }

        if op.opcode == Opcode::INVALID {
            saw_invalid = true;
            continue;
        }

        // Skip scanning if we're in the metadata section
        if in_metadata_section {
            continue;
        }

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
            Opcode::PUSH20 if op.input.len() == 20 => {
                // Skip zero address and well-known addresses
                if !is_known_address(&op.input) {
                    hardcoded_addresses += 1;
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

        // Address mask (0xfff...fff) - used for masking operations
        let mask = [0xff; 20];
        assert!(is_known_address(&mask));

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

    #[test]
    fn test_metadata_section_filtering() {
        // Simulate bytecode with ORIGIN in metadata section
        // Normal code: PUSH1 0x60, PUSH1 0x40, MSTORE
        // End of code: INVALID followed by non-JUMPDEST (metadata starts)
        // Metadata (should be ignored): GASLIMIT, NUMBER, ORIGIN, PUSH20
        //
        // Heuristic: INVALID followed by anything other than JUMPDEST = metadata
        let operations = vec![
            Operation {
                opcode: Opcode::PUSH1,
                offset: 0,
                input: vec![0x60],
            },
            Operation {
                opcode: Opcode::PUSH1,
                offset: 2,
                input: vec![0x40],
            },
            Operation {
                opcode: Opcode::MSTORE,
                offset: 4,
                input: vec![],
            },
            // End of runtime code marker - INVALID followed by non-JUMPDEST
            Operation {
                opcode: Opcode::INVALID,
                offset: 5,
                input: vec![],
            },
            // Metadata section starts here (GASLIMIT after INVALID, not JUMPDEST)
            Operation {
                opcode: Opcode::GASLIMIT,
                offset: 6,
                input: vec![],
            },
            Operation {
                opcode: Opcode::NUMBER,
                offset: 7,
                input: vec![],
            },
            // ORIGIN in metadata - should be ignored
            Operation {
                opcode: Opcode::ORIGIN,
                offset: 8,
                input: vec![],
            },
            Operation {
                opcode: Opcode::PUSH20,
                offset: 9,
                input: vec![
                    0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78, 0x9a,
                    0xbc, 0xde, 0xf0, 0x12, 0x34, 0x56, 0x78,
                ],
            },
        ];

        let analysis = detect_security_issues_from_operations(&operations);

        // ORIGIN in metadata should NOT be flagged
        assert!(
            !analysis.issues.iter().any(|i| i.pattern == "ORIGIN"),
            "ORIGIN in metadata section should not be flagged"
        );

        // PUSH20 in metadata should NOT be counted
        assert_eq!(
            analysis.hardcoded_address_count, 0,
            "Addresses in metadata section should not be counted"
        );
    }

    #[test]
    fn test_origin_in_runtime_code_is_flagged() {
        // ORIGIN in actual runtime code (before any INVALID) should be flagged
        let operations = vec![
            Operation {
                opcode: Opcode::PUSH1,
                offset: 0,
                input: vec![0x60],
            },
            Operation {
                opcode: Opcode::ORIGIN,
                offset: 2,
                input: vec![],
            },
            Operation {
                opcode: Opcode::PUSH1,
                offset: 3,
                input: vec![0x40],
            },
        ];

        let analysis = detect_security_issues_from_operations(&operations);

        // ORIGIN in runtime code SHOULD be flagged
        assert!(
            analysis.issues.iter().any(|i| i.pattern == "ORIGIN"),
            "ORIGIN in runtime code should be flagged"
        );
    }

    #[test]
    fn test_invalid_followed_by_jumpdest_is_not_metadata() {
        // INVALID followed by JUMPDEST is a normal revert handler pattern
        // Code after this should still be scanned
        let operations = vec![
            Operation {
                opcode: Opcode::PUSH1,
                offset: 0,
                input: vec![0x60],
            },
            // Revert handler pattern
            Operation {
                opcode: Opcode::INVALID,
                offset: 2,
                input: vec![],
            },
            Operation {
                opcode: Opcode::JUMPDEST,
                offset: 3,
                input: vec![],
            },
            // This ORIGIN should still be flagged (it's after INVALID+JUMPDEST, still in code)
            Operation {
                opcode: Opcode::ORIGIN,
                offset: 4,
                input: vec![],
            },
        ];

        let analysis = detect_security_issues_from_operations(&operations);

        // ORIGIN after INVALID+JUMPDEST should still be flagged
        assert!(
            analysis.issues.iter().any(|i| i.pattern == "ORIGIN"),
            "ORIGIN after INVALID+JUMPDEST (revert handler) should still be flagged"
        );
    }
}
