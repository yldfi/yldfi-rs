//! Bytecode disassembly using evm-disassembler

use super::types::{DisassembledOp, OpcodeStats, MAX_BYTECODE_SIZE};
use evm_disassembler::{disassemble_bytes, Opcode, Operation};
use std::collections::HashMap;
use thiserror::Error;

/// Errors that can occur during disassembly
#[derive(Debug, Error)]
pub enum DisassemblyError {
    #[error("Bytecode too large: {0} bytes (max: {1})")]
    BytecodeTooLarge(usize, usize),
    #[error("Failed to disassemble bytecode: {0}")]
    ParseError(String),
}

/// Disassemble bytecode into individual opcodes
///
/// Returns a list of operations with offset, opcode name, and operand
#[must_use]
pub fn disassemble_bytecode(bytecode: &[u8]) -> Result<Vec<DisassembledOp>, DisassemblyError> {
    if bytecode.is_empty() {
        return Ok(Vec::new());
    }

    if bytecode.len() > MAX_BYTECODE_SIZE {
        return Err(DisassemblyError::BytecodeTooLarge(
            bytecode.len(),
            MAX_BYTECODE_SIZE,
        ));
    }

    let operations = disassemble_bytes(bytecode.to_vec())
        .map_err(|e| DisassemblyError::ParseError(e.to_string()))?;

    Ok(operations_to_disassembled_ops(&operations))
}

/// Convert raw operations to DisassembledOp structs
///
/// This is public to allow reuse in analyze.rs for single-pass disassembly.
pub fn operations_to_disassembled_ops(operations: &[Operation]) -> Vec<DisassembledOp> {
    operations
        .iter()
        .map(|op| {
            let opcode_name = opcode_to_static_str(op.opcode);

            // Extract operand for PUSH operations
            let operand = if !op.input.is_empty() {
                Some(format!("0x{}", hex::encode(&op.input)))
            } else {
                None
            };

            DisassembledOp {
                offset: op.offset as usize,
                opcode: opcode_name.to_string(),
                operand,
            }
        })
        .collect()
}

/// Get a static string for an opcode (avoids format! allocation)
fn opcode_to_static_str(opcode: Opcode) -> &'static str {
    match opcode {
        Opcode::STOP => "STOP",
        Opcode::ADD => "ADD",
        Opcode::MUL => "MUL",
        Opcode::SUB => "SUB",
        Opcode::DIV => "DIV",
        Opcode::SDIV => "SDIV",
        Opcode::MOD => "MOD",
        Opcode::SMOD => "SMOD",
        Opcode::ADDMOD => "ADDMOD",
        Opcode::MULMOD => "MULMOD",
        Opcode::EXP => "EXP",
        Opcode::SIGNEXTEND => "SIGNEXTEND",
        Opcode::LT => "LT",
        Opcode::GT => "GT",
        Opcode::SLT => "SLT",
        Opcode::SGT => "SGT",
        Opcode::EQ => "EQ",
        Opcode::ISZERO => "ISZERO",
        Opcode::AND => "AND",
        Opcode::OR => "OR",
        Opcode::XOR => "XOR",
        Opcode::NOT => "NOT",
        Opcode::BYTE => "BYTE",
        Opcode::SHL => "SHL",
        Opcode::SHR => "SHR",
        Opcode::SAR => "SAR",
        Opcode::SHA3 => "KECCAK256",
        Opcode::ADDRESS => "ADDRESS",
        Opcode::BALANCE => "BALANCE",
        Opcode::ORIGIN => "ORIGIN",
        Opcode::CALLER => "CALLER",
        Opcode::CALLVALUE => "CALLVALUE",
        Opcode::CALLDATALOAD => "CALLDATALOAD",
        Opcode::CALLDATASIZE => "CALLDATASIZE",
        Opcode::CALLDATACOPY => "CALLDATACOPY",
        Opcode::CODESIZE => "CODESIZE",
        Opcode::CODECOPY => "CODECOPY",
        Opcode::GASPRICE => "GASPRICE",
        Opcode::EXTCODESIZE => "EXTCODESIZE",
        Opcode::EXTCODECOPY => "EXTCODECOPY",
        Opcode::RETURNDATASIZE => "RETURNDATASIZE",
        Opcode::RETURNDATACOPY => "RETURNDATACOPY",
        Opcode::EXTCODEHASH => "EXTCODEHASH",
        Opcode::BLOCKHASH => "BLOCKHASH",
        Opcode::COINBASE => "COINBASE",
        Opcode::TIMESTAMP => "TIMESTAMP",
        Opcode::NUMBER => "NUMBER",
        Opcode::DIFFICULTY => "DIFFICULTY",
        Opcode::GASLIMIT => "GASLIMIT",
        Opcode::CHAINID => "CHAINID",
        Opcode::SELFBALANCE => "SELFBALANCE",
        Opcode::BASEFEE => "BASEFEE",
        Opcode::POP => "POP",
        Opcode::MLOAD => "MLOAD",
        Opcode::MSTORE => "MSTORE",
        Opcode::MSTORE8 => "MSTORE8",
        Opcode::SLOAD => "SLOAD",
        Opcode::SSTORE => "SSTORE",
        Opcode::JUMP => "JUMP",
        Opcode::JUMPI => "JUMPI",
        Opcode::PC => "PC",
        Opcode::MSIZE => "MSIZE",
        Opcode::GAS => "GAS",
        Opcode::JUMPDEST => "JUMPDEST",
        Opcode::PUSH0 => "PUSH0",
        Opcode::PUSH1 => "PUSH1",
        Opcode::PUSH2 => "PUSH2",
        Opcode::PUSH3 => "PUSH3",
        Opcode::PUSH4 => "PUSH4",
        Opcode::PUSH5 => "PUSH5",
        Opcode::PUSH6 => "PUSH6",
        Opcode::PUSH7 => "PUSH7",
        Opcode::PUSH8 => "PUSH8",
        Opcode::PUSH9 => "PUSH9",
        Opcode::PUSH10 => "PUSH10",
        Opcode::PUSH11 => "PUSH11",
        Opcode::PUSH12 => "PUSH12",
        Opcode::PUSH13 => "PUSH13",
        Opcode::PUSH14 => "PUSH14",
        Opcode::PUSH15 => "PUSH15",
        Opcode::PUSH16 => "PUSH16",
        Opcode::PUSH17 => "PUSH17",
        Opcode::PUSH18 => "PUSH18",
        Opcode::PUSH19 => "PUSH19",
        Opcode::PUSH20 => "PUSH20",
        Opcode::PUSH21 => "PUSH21",
        Opcode::PUSH22 => "PUSH22",
        Opcode::PUSH23 => "PUSH23",
        Opcode::PUSH24 => "PUSH24",
        Opcode::PUSH25 => "PUSH25",
        Opcode::PUSH26 => "PUSH26",
        Opcode::PUSH27 => "PUSH27",
        Opcode::PUSH28 => "PUSH28",
        Opcode::PUSH29 => "PUSH29",
        Opcode::PUSH30 => "PUSH30",
        Opcode::PUSH31 => "PUSH31",
        Opcode::PUSH32 => "PUSH32",
        Opcode::DUP1 => "DUP1",
        Opcode::DUP2 => "DUP2",
        Opcode::DUP3 => "DUP3",
        Opcode::DUP4 => "DUP4",
        Opcode::DUP5 => "DUP5",
        Opcode::DUP6 => "DUP6",
        Opcode::DUP7 => "DUP7",
        Opcode::DUP8 => "DUP8",
        Opcode::DUP9 => "DUP9",
        Opcode::DUP10 => "DUP10",
        Opcode::DUP11 => "DUP11",
        Opcode::DUP12 => "DUP12",
        Opcode::DUP13 => "DUP13",
        Opcode::DUP14 => "DUP14",
        Opcode::DUP15 => "DUP15",
        Opcode::DUP16 => "DUP16",
        Opcode::SWAP1 => "SWAP1",
        Opcode::SWAP2 => "SWAP2",
        Opcode::SWAP3 => "SWAP3",
        Opcode::SWAP4 => "SWAP4",
        Opcode::SWAP5 => "SWAP5",
        Opcode::SWAP6 => "SWAP6",
        Opcode::SWAP7 => "SWAP7",
        Opcode::SWAP8 => "SWAP8",
        Opcode::SWAP9 => "SWAP9",
        Opcode::SWAP10 => "SWAP10",
        Opcode::SWAP11 => "SWAP11",
        Opcode::SWAP12 => "SWAP12",
        Opcode::SWAP13 => "SWAP13",
        Opcode::SWAP14 => "SWAP14",
        Opcode::SWAP15 => "SWAP15",
        Opcode::SWAP16 => "SWAP16",
        Opcode::LOG0 => "LOG0",
        Opcode::LOG1 => "LOG1",
        Opcode::LOG2 => "LOG2",
        Opcode::LOG3 => "LOG3",
        Opcode::LOG4 => "LOG4",
        Opcode::CREATE => "CREATE",
        Opcode::CALL => "CALL",
        Opcode::CALLCODE => "CALLCODE",
        Opcode::RETURN => "RETURN",
        Opcode::DELEGATECALL => "DELEGATECALL",
        Opcode::CREATE2 => "CREATE2",
        Opcode::STATICCALL => "STATICCALL",
        Opcode::REVERT => "REVERT",
        Opcode::INVALID => "INVALID",
        Opcode::SELFDESTRUCT => "SELFDESTRUCT",
        _ => "UNKNOWN",
    }
}

/// Check if an opcode is a PUSH instruction
fn is_push_opcode(opcode: Opcode) -> bool {
    matches!(
        opcode,
        Opcode::PUSH0
            | Opcode::PUSH1
            | Opcode::PUSH2
            | Opcode::PUSH3
            | Opcode::PUSH4
            | Opcode::PUSH5
            | Opcode::PUSH6
            | Opcode::PUSH7
            | Opcode::PUSH8
            | Opcode::PUSH9
            | Opcode::PUSH10
            | Opcode::PUSH11
            | Opcode::PUSH12
            | Opcode::PUSH13
            | Opcode::PUSH14
            | Opcode::PUSH15
            | Opcode::PUSH16
            | Opcode::PUSH17
            | Opcode::PUSH18
            | Opcode::PUSH19
            | Opcode::PUSH20
            | Opcode::PUSH21
            | Opcode::PUSH22
            | Opcode::PUSH23
            | Opcode::PUSH24
            | Opcode::PUSH25
            | Opcode::PUSH26
            | Opcode::PUSH27
            | Opcode::PUSH28
            | Opcode::PUSH29
            | Opcode::PUSH30
            | Opcode::PUSH31
            | Opcode::PUSH32
    )
}

/// Calculate opcode frequency statistics from raw operations
///
/// This is the optimized version that takes pre-disassembled operations
/// to avoid redundant disassembly when used with analyze_bytecode.
#[must_use]
pub fn opcode_stats_from_operations(operations: &[Operation], bytecode_size: usize) -> OpcodeStats {
    let mut frequencies: HashMap<&'static str, usize> = HashMap::new();
    let mut push_count = 0;
    let mut jump_count = 0;
    let mut call_count = 0;
    let mut storage_count = 0;

    for op in operations {
        let opcode_name = opcode_to_static_str(op.opcode);
        *frequencies.entry(opcode_name).or_insert(0) += 1;

        // Categorize opcodes
        if is_push_opcode(op.opcode) {
            push_count += 1;
        }

        match op.opcode {
            // JUMP operations
            Opcode::JUMP | Opcode::JUMPI | Opcode::JUMPDEST => jump_count += 1,

            // CALL operations
            Opcode::CALL
            | Opcode::CALLCODE
            | Opcode::DELEGATECALL
            | Opcode::STATICCALL
            | Opcode::CREATE
            | Opcode::CREATE2 => call_count += 1,

            // Storage operations (including transient storage EIP-1153)
            Opcode::SLOAD | Opcode::SSTORE => storage_count += 1,

            _ => {}
        }
    }

    // Convert &'static str keys to String for serialization
    let frequencies: HashMap<String, usize> = frequencies
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();

    OpcodeStats {
        total_opcodes: operations.len(),
        bytecode_size,
        frequencies,
        push_count,
        jump_count,
        call_count,
        storage_count,
    }
}

/// Calculate opcode frequency statistics
///
/// For better performance when also needing disassembly or security analysis,
/// use `disassemble_raw` and pass the result to `opcode_stats_from_operations`.
#[must_use]
pub fn opcode_stats(bytecode: &[u8]) -> Result<OpcodeStats, DisassemblyError> {
    if bytecode.is_empty() {
        return Ok(OpcodeStats {
            total_opcodes: 0,
            bytecode_size: 0,
            frequencies: HashMap::new(),
            push_count: 0,
            jump_count: 0,
            call_count: 0,
            storage_count: 0,
        });
    }

    if bytecode.len() > MAX_BYTECODE_SIZE {
        return Err(DisassemblyError::BytecodeTooLarge(
            bytecode.len(),
            MAX_BYTECODE_SIZE,
        ));
    }

    let operations = disassemble_bytes(bytecode.to_vec())
        .map_err(|e| DisassemblyError::ParseError(e.to_string()))?;

    Ok(opcode_stats_from_operations(&operations, bytecode.len()))
}

/// Disassemble bytecode and return raw operations for further processing
///
/// This is useful when you need to perform multiple analyses on the same bytecode
/// (e.g., stats + security + disassembly) to avoid redundant parsing.
pub fn disassemble_raw(bytecode: &[u8]) -> Result<Vec<Operation>, DisassemblyError> {
    if bytecode.is_empty() {
        return Ok(Vec::new());
    }

    if bytecode.len() > MAX_BYTECODE_SIZE {
        return Err(DisassemblyError::BytecodeTooLarge(
            bytecode.len(),
            MAX_BYTECODE_SIZE,
        ));
    }

    disassemble_bytes(bytecode.to_vec()).map_err(|e| DisassemblyError::ParseError(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disassemble_simple() {
        // PUSH1 0x60, PUSH1 0x40, MSTORE
        let bytecode = hex::decode("6060604052").unwrap();
        let ops = disassemble_bytecode(&bytecode).unwrap();

        assert!(!ops.is_empty());
        assert_eq!(ops[0].opcode, "PUSH1");
        assert_eq!(ops[0].operand, Some("0x60".to_string()));
    }

    #[test]
    fn test_opcode_stats() {
        let bytecode = hex::decode("6060604052600080fd").unwrap();
        let stats = opcode_stats(&bytecode).unwrap();

        assert!(stats.total_opcodes > 0);
        assert!(stats.push_count > 0);
    }

    #[test]
    fn test_empty_bytecode() {
        let ops = disassemble_bytecode(&[]).unwrap();
        assert!(ops.is_empty());

        let stats = opcode_stats(&[]).unwrap();
        assert_eq!(stats.total_opcodes, 0);
    }

    #[test]
    fn test_bytecode_too_large() {
        let large_bytecode = vec![0u8; MAX_BYTECODE_SIZE + 1];
        let result = disassemble_bytecode(&large_bytecode);
        assert!(matches!(
            result,
            Err(DisassemblyError::BytecodeTooLarge(_, _))
        ));
    }
}
