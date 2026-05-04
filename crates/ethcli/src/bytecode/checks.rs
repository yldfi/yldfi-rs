//! Selector dispatcher and handler guard heuristics.

use super::types::{
    BytecodeCheckSummary, CheckFinding, DispatcherEntry, ExtractedFunction, FunctionCheckAnalysis,
    RiskLevel, StateMutability,
};
use super::{disassemble_bytecode, DisassemblyError};
use std::collections::HashMap;

/// Infer selector dispatcher entries from Solidity-style dispatcher bytecode.
pub fn infer_dispatcher(
    bytecode: &[u8],
    functions: &[ExtractedFunction],
) -> Result<Vec<DispatcherEntry>, DisassemblyError> {
    let ops = disassemble_bytecode(bytecode)?;
    let signatures = selector_signatures(functions);
    let known_selectors: std::collections::HashSet<String> = functions
        .iter()
        .map(|f| normalize_selector(&f.selector))
        .collect();

    let mut entries = Vec::new();

    for (idx, op) in ops.iter().enumerate() {
        if op.opcode != "PUSH4" {
            continue;
        }

        let Some(selector) = op.operand.as_deref().map(normalize_selector) else {
            continue;
        };

        if !known_selectors.is_empty() && !known_selectors.contains(&selector) {
            continue;
        }

        let window_end = std::cmp::min(idx + 12, ops.len().saturating_sub(1));
        let mut saw_eq = false;
        let mut jump_dest = None;

        for j in idx + 1..=window_end {
            if ops[j].opcode == "EQ" {
                saw_eq = true;
            }

            if ops[j].opcode == "JUMPI" && saw_eq {
                for k in (idx + 1..j).rev() {
                    if is_push_dest(&ops[k].opcode) {
                        jump_dest = ops[k].operand.as_deref().and_then(parse_hex_usize);
                        break;
                    }
                }
                break;
            }
        }

        if let Some(handler_offset) = jump_dest {
            entries.push(DispatcherEntry {
                selector: selector.clone(),
                signature: signatures.get(&selector).cloned().flatten(),
                selector_offset: op.offset,
                handler_offset,
            });
        }
    }

    entries.sort_by(|a, b| {
        a.handler_offset
            .cmp(&b.handler_offset)
            .then(a.selector.cmp(&b.selector))
    });
    entries.dedup_by(|a, b| a.selector == b.selector && a.handler_offset == b.handler_offset);

    Ok(entries)
}

/// Analyze inferred handlers for common auth, hash, state, and external-call checks.
pub fn analyze_handler_checks(
    bytecode: &[u8],
    functions: &[ExtractedFunction],
    dispatcher: &[DispatcherEntry],
) -> Result<BytecodeCheckSummary, DisassemblyError> {
    let ops = disassemble_bytecode(bytecode)?;
    let function_map: HashMap<String, &ExtractedFunction> = functions
        .iter()
        .map(|f| (normalize_selector(&f.selector), f))
        .collect();

    let mut sorted_handlers = dispatcher.to_vec();
    sorted_handlers.sort_by_key(|entry| entry.handler_offset);

    let resolved_bodies: Vec<_> = sorted_handlers
        .iter()
        .enumerate()
        .map(|(idx, entry)| {
            let wrapper_end = sorted_handlers
                .iter()
                .skip(idx + 1)
                .find(|next| next.handler_offset > entry.handler_offset)
                .map(|next| next.handler_offset)
                .unwrap_or(bytecode.len());
            let body_offset =
                resolve_body_offset(&ops, entry.handler_offset, wrapper_end, bytecode.len());
            (entry, wrapper_end, body_offset)
        })
        .collect();
    let mut body_offsets: Vec<_> = resolved_bodies
        .iter()
        .map(|(_, _, body_offset)| *body_offset)
        .collect();
    body_offsets.sort_unstable();
    body_offsets.dedup();

    let mut analyses = Vec::new();

    for (entry, wrapper_end, body_offset) in resolved_bodies {
        let body_scan_end = body_offsets
            .iter()
            .copied()
            .find(|next| *next > body_offset)
            .unwrap_or(bytecode.len());
        let range_ops: Vec<_> = ops
            .iter()
            .filter(|op| {
                (op.offset >= entry.handler_offset && op.offset < wrapper_end)
                    || (op.offset >= body_offset && op.offset < body_scan_end)
            })
            .collect();

        let function = function_map
            .get(&normalize_selector(&entry.selector))
            .copied();
        let state_mutability = function
            .map(|f| f.state_mutability)
            .unwrap_or(StateMutability::Unknown);
        let arguments = function.map(|f| f.arguments.as_slice()).unwrap_or(&[]);

        let uses_caller = range_ops.iter().any(|op| op.opcode == "CALLER");
        let has_eq = range_ops.iter().any(|op| op.opcode == "EQ");
        let has_branch = range_ops.iter().any(|op| op.opcode == "JUMPI");
        let caller_gated = uses_caller && has_eq && has_branch;
        let calldata_hash_check = range_ops.iter().any(|op| op.opcode == "KECCAK256")
            && range_ops
                .iter()
                .any(|op| matches!(op.opcode.as_str(), "CALLDATALOAD" | "CALLDATACOPY"));
        let storage_reads = count_ops(&range_ops, "SLOAD");
        let storage_writes = count_ops(&range_ops, "SSTORE");
        let external_calls = range_ops
            .iter()
            .filter(|op| {
                matches!(
                    op.opcode.as_str(),
                    "CALL" | "STATICCALL" | "DELEGATECALL" | "CALLCODE"
                )
            })
            .count();
        let reverts = count_ops(&range_ops, "REVERT");
        let storage_status_check = storage_reads > 0
            && has_branch
            && range_ops
                .iter()
                .any(|op| matches!(op.opcode.as_str(), "EQ" | "GT" | "LT" | "ISZERO"));

        let findings = build_findings(
            state_mutability,
            arguments,
            caller_gated,
            calldata_hash_check,
            storage_writes,
            external_calls,
        );

        analyses.push(FunctionCheckAnalysis {
            selector: entry.selector.clone(),
            signature: entry.signature.clone(),
            handler_offset: entry.handler_offset,
            body_offset: (body_offset != entry.handler_offset).then_some(body_offset),
            scan_end_offset: body_scan_end,
            state_mutability,
            uses_caller,
            caller_gated,
            calldata_hash_check,
            storage_status_check,
            storage_reads,
            storage_writes,
            external_calls,
            reverts,
            findings,
        });
    }

    let finding_count = analyses.iter().map(|a| a.findings.len()).sum();
    let risk_level = analyses
        .iter()
        .flat_map(|a| a.findings.iter().map(|f| f.risk))
        .max_by_key(risk_rank)
        .unwrap_or(RiskLevel::Low);

    Ok(BytecodeCheckSummary {
        risk_level,
        function_count: analyses.len(),
        finding_count,
        functions: analyses,
    })
}

fn build_findings(
    state_mutability: StateMutability,
    arguments: &[String],
    caller_gated: bool,
    calldata_hash_check: bool,
    storage_writes: usize,
    external_calls: usize,
) -> Vec<CheckFinding> {
    let mut findings = Vec::new();
    let mutating = matches!(
        state_mutability,
        StateMutability::NonPayable | StateMutability::Payable
    ) && (storage_writes > 0 || external_calls > 0);
    let payable = state_mutability == StateMutability::Payable;
    let complex_args = arguments.iter().any(|arg| {
        arg.contains('(') || arg.contains('[') || arg == "bytes" || arg.starts_with("bytes")
    });

    if mutating && !caller_gated {
        findings.push(CheckFinding {
            id: "missing_caller_gate".to_string(),
            risk: RiskLevel::Medium,
            description:
                "Mutating handler has no obvious CALLER/EQ/JUMPI authorization gate in its entry range"
                    .to_string(),
        });
    }

    if mutating && complex_args && !calldata_hash_check {
        findings.push(CheckFinding {
            id: "caller_supplied_complex_data_without_hash_check".to_string(),
            risk: RiskLevel::High,
            description:
                "Mutating handler accepts complex caller-supplied calldata but no calldata hash check was inferred"
                    .to_string(),
        });
    }

    if external_calls > 0 && !caller_gated {
        findings.push(CheckFinding {
            id: "external_calls_without_caller_gate".to_string(),
            risk: RiskLevel::Medium,
            description:
                "Handler performs external calls without an obvious caller authorization gate"
                    .to_string(),
        });
    }

    if payable && complex_args && mutating && !caller_gated && !calldata_hash_check {
        findings.push(CheckFinding {
            id: "payable_complex_entrypoint_open_settlement_shape".to_string(),
            risk: RiskLevel::High,
            description:
                "Payable complex entrypoint resembles an open settlement/fill surface; review caller auth and canonical data validation"
                    .to_string(),
        });
    }

    findings
}

fn selector_signatures(functions: &[ExtractedFunction]) -> HashMap<String, Option<String>> {
    functions
        .iter()
        .map(|f| (normalize_selector(&f.selector), f.signature.clone()))
        .collect()
}

fn normalize_selector(selector: &str) -> String {
    let stripped = selector.trim_start_matches("0x").trim_start_matches("0X");
    format!("0x{}", stripped.to_ascii_lowercase())
}

fn parse_hex_usize(value: &str) -> Option<usize> {
    usize::from_str_radix(value.trim_start_matches("0x").trim_start_matches("0X"), 16).ok()
}

fn is_push_dest(opcode: &str) -> bool {
    matches!(opcode, "PUSH1" | "PUSH2" | "PUSH3" | "PUSH4")
}

fn resolve_body_offset(
    ops: &[super::types::DisassembledOp],
    handler_offset: usize,
    wrapper_end: usize,
    bytecode_len: usize,
) -> usize {
    let mut saw_local_jumpdest = false;

    for (idx, op) in ops.iter().enumerate() {
        if op.offset < handler_offset || op.offset >= wrapper_end {
            continue;
        }

        if op.opcode == "JUMPDEST" && op.offset > handler_offset {
            saw_local_jumpdest = true;
            continue;
        }

        if op.opcode == "JUMP" && saw_local_jumpdest {
            if let Some(dest) = previous_push_dest(ops, idx) {
                if dest > handler_offset && dest < bytecode_len {
                    return dest;
                }
            }
        }
    }

    handler_offset
}

fn previous_push_dest(ops: &[super::types::DisassembledOp], jump_idx: usize) -> Option<usize> {
    ops[..jump_idx]
        .iter()
        .rev()
        .find(|op| is_push_dest(&op.opcode))
        .and_then(|op| op.operand.as_deref())
        .and_then(parse_hex_usize)
}

fn count_ops(ops: &[&super::types::DisassembledOp], opcode: &str) -> usize {
    ops.iter().filter(|op| op.opcode == opcode).count()
}

fn risk_rank(risk: &RiskLevel) -> u8 {
    match risk {
        RiskLevel::Low => 0,
        RiskLevel::Medium => 1,
        RiskLevel::High => 2,
        RiskLevel::Critical => 3,
        RiskLevel::Unknown => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn function(selector: &str, state_mutability: StateMutability) -> ExtractedFunction {
        ExtractedFunction {
            selector: selector.to_string(),
            arguments: vec!["(uint256,address)".to_string(), "uint256".to_string()],
            state_mutability,
            signature: None,
        }
    }

    #[test]
    fn infers_simple_solidity_dispatcher() {
        let bytecode = hex::decode("63aabbccdd14610010575b005b600080fd").unwrap();
        let functions = vec![function("0xaabbccdd", StateMutability::NonPayable)];

        let dispatcher = infer_dispatcher(&bytecode, &functions).unwrap();

        assert_eq!(dispatcher.len(), 1);
        assert_eq!(dispatcher[0].selector, "0xaabbccdd");
        assert_eq!(dispatcher[0].handler_offset, 0x10);
    }

    #[test]
    fn flags_complex_mutating_handler_without_hash_check() {
        let bytecode = hex::decode("63aabbccdd14610010575b005b546000f100").unwrap();
        let functions = vec![function("0xaabbccdd", StateMutability::Payable)];
        let dispatcher = infer_dispatcher(&bytecode, &functions).unwrap();

        let checks = analyze_handler_checks(&bytecode, &functions, &dispatcher).unwrap();

        assert_eq!(checks.function_count, 1);
        assert!(checks.finding_count > 0);
        assert_eq!(checks.risk_level, RiskLevel::High);
    }

    #[test]
    fn scans_solidity_wrapper_body_for_checks() {
        let mut bytecode = hex::decode("63aabbccdd1461001057").unwrap();
        bytecode.resize(0x10, 0);
        bytecode.extend(hex::decode("5b61003061001836610040565b61003856").unwrap());
        bytecode.resize(0x38, 0);
        bytecode.extend(hex::decode("5b331461004057352000").unwrap());
        let functions = vec![function("0xaabbccdd", StateMutability::Payable)];
        let dispatcher = infer_dispatcher(&bytecode, &functions).unwrap();

        let checks = analyze_handler_checks(&bytecode, &functions, &dispatcher).unwrap();

        assert_eq!(checks.function_count, 1);
        assert_eq!(checks.functions[0].body_offset, Some(0x38));
        assert!(checks.functions[0].caller_gated);
        assert!(checks.functions[0].calldata_hash_check);
        assert_eq!(checks.finding_count, 0);
    }
}
