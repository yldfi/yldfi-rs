//! Types for bytecode analysis results

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Maximum bytecode size to analyze (256KB - well above EIP-170 24KB limit)
pub const MAX_BYTECODE_SIZE: usize = 256 * 1024;

/// State mutability of a function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateMutability {
    Pure,
    View,
    NonPayable,
    Payable,
    #[default]
    Unknown,
}

impl StateMutability {
    pub fn as_str(&self) -> &'static str {
        match self {
            StateMutability::Pure => "pure",
            StateMutability::View => "view",
            StateMutability::NonPayable => "nonpayable",
            StateMutability::Payable => "payable",
            StateMutability::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for StateMutability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A function extracted from bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedFunction {
    /// 4-byte function selector (hex with 0x prefix)
    pub selector: String,
    /// Function arguments as Solidity types
    pub arguments: Vec<String>,
    /// State mutability (pure, view, nonpayable, payable)
    pub state_mutability: StateMutability,
    /// Resolved function signature (if lookup succeeded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// A disassembled opcode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisassembledOp {
    /// Byte offset in the bytecode
    pub offset: usize,
    /// Opcode mnemonic (PUSH1, ADD, etc.)
    pub opcode: String,
    /// Operand value for PUSH instructions (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operand: Option<String>,
}

/// Opcode frequency statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcodeStats {
    /// Total number of opcodes
    pub total_opcodes: usize,
    /// Bytecode size in bytes
    pub bytecode_size: usize,
    /// Frequency of each opcode (opcode -> count)
    pub frequencies: HashMap<String, usize>,
    /// Number of PUSH operations
    pub push_count: usize,
    /// Number of JUMP operations
    pub jump_count: usize,
    /// Number of CALL-family operations (CALL, STATICCALL, DELEGATECALL, CALLCODE)
    pub call_count: usize,
    /// Number of storage operations (SLOAD, SSTORE)
    pub storage_count: usize,
}

/// Security risk level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// No significant risks detected
    #[default]
    Low,
    /// Some potentially risky patterns
    Medium,
    /// Dangerous patterns detected
    High,
    /// Critical security issues
    Critical,
    /// Unable to analyze (parse failure)
    Unknown,
}

impl RiskLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
            RiskLevel::Unknown => "UNKNOWN",
        }
    }
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A detected security issue in bytecode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// Type of issue (e.g., "SELFDESTRUCT", "DELEGATECALL")
    pub pattern: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Human-readable description
    pub description: String,
    /// Number of occurrences
    pub count: usize,
    /// Byte offsets where the pattern was found
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub offsets: Vec<usize>,
}

/// Security analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAnalysis {
    /// Overall risk level
    pub risk_level: RiskLevel,
    /// Number of dangerous opcodes found
    pub dangerous_opcode_count: usize,
    /// Number of hardcoded addresses found
    pub hardcoded_address_count: usize,
    /// Detected security issues
    pub issues: Vec<SecurityIssue>,
}

impl Default for SecurityAnalysis {
    fn default() -> Self {
        Self {
            risk_level: RiskLevel::Low,
            dangerous_opcode_count: 0,
            hardcoded_address_count: 0,
            issues: Vec::new(),
        }
    }
}

impl SecurityAnalysis {
    /// Create a new security analysis with no issues (alias for Default)
    #[must_use]
    pub fn clean() -> Self {
        Self::default()
    }

    /// Create a security analysis indicating parse failure
    #[must_use]
    pub fn parse_failed() -> Self {
        Self {
            risk_level: RiskLevel::Unknown,
            dangerous_opcode_count: 0,
            hardcoded_address_count: 0,
            issues: Vec::new(),
        }
    }
}

/// Combined bytecode analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeAnalysis {
    /// Contract address
    pub address: String,
    /// Bytecode size in bytes
    pub bytecode_size: usize,
    /// Number of functions detected
    pub function_count: usize,
    /// Extracted functions
    pub functions: Vec<ExtractedFunction>,
    /// Security analysis
    pub security: SecurityAnalysis,
    /// Opcode statistics (optional, included when requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opcode_stats: Option<OpcodeStats>,
    /// Disassembly (optional, included when requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disassembly: Option<Vec<DisassembledOp>>,
    /// Proxy contract info (if detected)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy_info: Option<super::proxy::ProxyInfo>,
}
