//! Bytecode analysis module
//!
//! Provides bytecode analysis capabilities using evmole and evm-disassembler:
//! - Function selector extraction
//! - Opcode disassembly
//! - Security pattern detection
//!
//! ## Single-Pass Analysis
//!
//! For best performance when you need multiple types of analysis (security, stats,
//! disassembly), use `analyze_bytecode` which performs a single disassembly pass
//! and reuses the result. For individual operations, use the standalone functions.
//!
//! ## Example
//!
//! ```ignore
//! use ethcli::bytecode::{analyze_bytecode, detect_security_issues};
//!
//! // Full analysis (single-pass internally)
//! let analysis = analyze_bytecode("0x1234...", &bytecode, true, true, None);
//!
//! // Or individual operations
//! let security = detect_security_issues(&bytecode);
//! ```

mod analyze;
mod disassemble;
mod proxy;
mod security;
mod selectors;
mod types;

pub use analyze::{analyze_bytecode, AnalysisError};
pub use disassemble::{
    disassemble_bytecode, disassemble_raw, opcode_stats, opcode_stats_from_operations,
    DisassemblyError,
};
pub use proxy::{
    address_from_storage, detect_eip1167, detect_proxy, slots as proxy_slots, u256_to_b256,
    ProxyInfo, ProxyType,
};
pub use security::{
    detect_security_issues, detect_security_issues_from_operations, SecurityPattern,
};
pub use selectors::extract_selectors;
pub use types::{
    BytecodeAnalysis, DisassembledOp, ExtractedFunction, OpcodeStats, RiskLevel, SecurityAnalysis,
    SecurityIssue, StateMutability, MAX_BYTECODE_SIZE,
};
