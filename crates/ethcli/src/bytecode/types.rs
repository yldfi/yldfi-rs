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

/// Selector dispatcher entry inferred from bytecode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DispatcherEntry {
    /// 4-byte function selector (hex with 0x prefix)
    pub selector: String,
    /// Best-known signature, if selector lookup resolved one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Byte offset of the selector comparison in the dispatcher
    pub selector_offset: usize,
    /// Byte offset of the inferred handler JUMPDEST
    pub handler_offset: usize,
}

/// Per-function guard/check heuristics inferred from a handler range.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCheckAnalysis {
    /// 4-byte function selector (hex with 0x prefix)
    pub selector: String,
    /// Best-known signature, if selector lookup resolved one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    /// Inferred handler start byte offset
    pub handler_offset: usize,
    /// Inferred function body byte offset when the dispatcher handler is an ABI wrapper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body_offset: Option<usize>,
    /// Exclusive end byte offset used for the scan
    pub scan_end_offset: usize,
    /// Function mutability inferred by selector extraction
    pub state_mutability: StateMutability,
    /// True when the handler has a CALLER opcode
    pub uses_caller: bool,
    /// True when CALLER appears with EQ and JUMPI in the handler range
    pub caller_gated: bool,
    /// True when the handler hashes calldata or copied calldata
    pub calldata_hash_check: bool,
    /// True when the handler has storage reads plus a comparison branch
    pub storage_status_check: bool,
    /// Number of SLOAD opcodes in the handler range
    pub storage_reads: usize,
    /// Number of SSTORE opcodes in the handler range
    pub storage_writes: usize,
    /// Number of external call opcodes in the handler range
    pub external_calls: usize,
    /// Number of revert opcodes in the handler range
    pub reverts: usize,
    /// Heuristic findings for this handler
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<CheckFinding>,
}

/// A bytecode guard/check finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckFinding {
    /// Finding identifier
    pub id: String,
    /// Risk level
    pub risk: RiskLevel,
    /// Human-readable description
    pub description: String,
}

/// Aggregate guard/check analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BytecodeCheckSummary {
    /// Overall risk level for guard/check heuristics
    pub risk_level: RiskLevel,
    /// Number of functions scanned
    pub function_count: usize,
    /// Total findings across scanned functions
    pub finding_count: usize,
    /// Per-function guard/check results
    pub functions: Vec<FunctionCheckAnalysis>,
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
    /// Selector dispatcher mapping (optional, included when requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatcher: Option<Vec<DispatcherEntry>>,
    /// Handler guard/check heuristics (optional, included when requested)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checks: Option<BytecodeCheckSummary>,
}
