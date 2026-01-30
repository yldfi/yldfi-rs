//! Combined bytecode analysis
//!
//! Provides single-pass bytecode analysis to avoid redundant disassembly.

use super::disassemble::{
    disassemble_raw, opcode_stats_from_operations, operations_to_disassembled_ops, DisassemblyError,
};
use super::security::detect_security_issues_from_operations;
use super::selectors::extract_selectors;
use super::types::{BytecodeAnalysis, SecurityAnalysis};

/// Perform a full bytecode analysis
///
/// Combines:
/// - Function selector extraction (evmole - separate pass, optimized internally)
/// - Security pattern detection
/// - Opcode statistics (optional)
/// - Disassembly (optional)
///
/// Uses single-pass disassembly when stats or security analysis is needed,
/// avoiding redundant parsing of bytecode.
#[must_use]
pub fn analyze_bytecode(
    address: &str,
    bytecode: &[u8],
    include_stats: bool,
    include_disassembly: bool,
    disassembly_limit: Option<usize>,
) -> BytecodeAnalysis {
    // Extract function selectors (evmole handles its own parsing efficiently)
    let functions = extract_selectors(bytecode);
    let function_count = functions.len();

    // Single-pass disassembly for security + stats + disassembly
    // This avoids parsing the bytecode 3 separate times
    let (security, stats, disassembly) = analyze_with_single_pass(
        bytecode,
        include_stats,
        include_disassembly,
        disassembly_limit,
    );

    BytecodeAnalysis {
        address: address.to_string(),
        bytecode_size: bytecode.len(),
        function_count,
        functions,
        security,
        opcode_stats: stats,
        disassembly,
        proxy_info: None, // Set by caller if proxy detected
    }
}

/// Perform analysis using a single disassembly pass
///
/// Returns (security_analysis, optional_stats, optional_disassembly)
fn analyze_with_single_pass(
    bytecode: &[u8],
    include_stats: bool,
    include_disassembly: bool,
    disassembly_limit: Option<usize>,
) -> (
    SecurityAnalysis,
    Option<super::types::OpcodeStats>,
    Option<Vec<super::types::DisassembledOp>>,
) {
    // Empty bytecode is a special case
    if bytecode.is_empty() {
        return (
            SecurityAnalysis::default(),
            if include_stats {
                Some(super::types::OpcodeStats {
                    total_opcodes: 0,
                    bytecode_size: 0,
                    frequencies: std::collections::HashMap::new(),
                    push_count: 0,
                    jump_count: 0,
                    call_count: 0,
                    storage_count: 0,
                })
            } else {
                None
            },
            if include_disassembly {
                Some(Vec::new())
            } else {
                None
            },
        );
    }

    // Try to disassemble once - all analysis uses this result
    match disassemble_raw(bytecode) {
        Ok(operations) => {
            // Security analysis from operations
            let security = detect_security_issues_from_operations(&operations);

            // Stats from operations (if requested)
            let stats = if include_stats {
                Some(opcode_stats_from_operations(&operations, bytecode.len()))
            } else {
                None
            };

            // Disassembly (if requested) - reuse already-parsed operations
            let disassembly = if include_disassembly {
                let mut ops = operations_to_disassembled_ops(&operations);
                if let Some(limit) = disassembly_limit {
                    ops.truncate(limit);
                }
                Some(ops)
            } else {
                None
            };

            (security, stats, disassembly)
        }
        Err(e) => {
            // Log the error and return parse-failed security analysis
            eprintln!(
                "Warning: Failed to disassemble bytecode for analysis: {}",
                e
            );
            (
                SecurityAnalysis::parse_failed(),
                None,
                if include_disassembly {
                    Some(Vec::new())
                } else {
                    None
                },
            )
        }
    }
}

/// Error type for bytecode analysis
#[derive(Debug, thiserror::Error)]
pub enum AnalysisError {
    #[error("Disassembly failed: {0}")]
    Disassembly(#[from] DisassemblyError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::types::RiskLevel;

    #[test]
    fn test_analyze_empty() {
        let analysis = analyze_bytecode("0x0", &[], false, false, None);

        assert_eq!(analysis.bytecode_size, 0);
        assert_eq!(analysis.function_count, 0);
        assert!(analysis.functions.is_empty());
        assert_eq!(analysis.security.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_with_stats() {
        let bytecode = hex::decode("6060604052600080fd").unwrap();
        let analysis = analyze_bytecode("0x1234", &bytecode, true, false, None);

        assert!(analysis.opcode_stats.is_some());
        assert!(analysis.disassembly.is_none());

        let stats = analysis.opcode_stats.unwrap();
        assert!(stats.total_opcodes > 0);
        assert!(stats.push_count > 0);
    }

    #[test]
    fn test_analyze_with_disassembly() {
        let bytecode = hex::decode("6060604052600080fd").unwrap();
        let analysis = analyze_bytecode("0x1234", &bytecode, false, true, None);

        assert!(analysis.opcode_stats.is_none());
        assert!(analysis.disassembly.is_some());
    }

    #[test]
    fn test_analyze_disassembly_limit() {
        let bytecode = hex::decode("6060604052600080fd5b5b5b5b5b5b").unwrap();
        let analysis = analyze_bytecode("0x1234", &bytecode, false, true, Some(3));

        let disasm = analysis.disassembly.unwrap();
        assert!(disasm.len() <= 3);
    }

    #[test]
    fn test_analyze_with_all_options() {
        let bytecode = hex::decode("6060604052600080fd").unwrap();
        let analysis = analyze_bytecode("0x1234", &bytecode, true, true, None);

        assert!(analysis.opcode_stats.is_some());
        assert!(analysis.disassembly.is_some());
        assert_eq!(analysis.security.risk_level, RiskLevel::Low);
    }

    #[test]
    fn test_analyze_security_detection() {
        // Bytecode with SELFDESTRUCT (0xff) - should be Critical risk
        let bytecode = hex::decode("6060604052ff").unwrap();
        let analysis = analyze_bytecode("0x1234", &bytecode, false, false, None);

        assert_eq!(analysis.security.risk_level, RiskLevel::Critical);
        assert!(analysis.security.dangerous_opcode_count > 0);
    }
}
