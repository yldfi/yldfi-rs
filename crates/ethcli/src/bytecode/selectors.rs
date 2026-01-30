//! Function selector extraction using evmole

use super::types::{ExtractedFunction, StateMutability as LocalStateMutability, MAX_BYTECODE_SIZE};
use evmole::{contract_info, ContractInfoArgs, StateMutability as EvmoleStateMutability};

/// Extract function selectors from bytecode
///
/// Uses evmole to extract:
/// - Function selectors (4-byte identifiers)
/// - Function arguments
/// - State mutability
///
/// Returns empty Vec for empty bytecode or bytecode exceeding MAX_BYTECODE_SIZE.
#[must_use]
pub fn extract_selectors(bytecode: &[u8]) -> Vec<ExtractedFunction> {
    // Return empty for empty or oversized bytecode
    if bytecode.is_empty() || bytecode.len() > MAX_BYTECODE_SIZE {
        return Vec::new();
    }

    let contract = contract_info(
        ContractInfoArgs::new(bytecode)
            .with_selectors()
            .with_arguments()
            .with_state_mutability(),
    );

    // functions is Option<Vec<Function>>
    let functions = match contract.functions {
        Some(funcs) => funcs,
        None => return Vec::new(),
    };

    functions
        .into_iter()
        .map(|func| {
            let selector = format!("0x{}", hex::encode(func.selector));

            let arguments: Vec<String> = func
                .arguments
                .map(|args| args.iter().map(|arg| arg.to_string()).collect())
                .unwrap_or_default();

            let state_mutability = func
                .state_mutability
                .map(|m| match m {
                    EvmoleStateMutability::Pure => LocalStateMutability::Pure,
                    EvmoleStateMutability::View => LocalStateMutability::View,
                    EvmoleStateMutability::NonPayable => LocalStateMutability::NonPayable,
                    EvmoleStateMutability::Payable => LocalStateMutability::Payable,
                })
                .unwrap_or(LocalStateMutability::Unknown);

            ExtractedFunction {
                selector,
                arguments,
                state_mutability,
                signature: None, // Will be filled in by signature lookup
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_selectors_empty_bytecode() {
        let bytecode = &[];
        let selectors = extract_selectors(bytecode);
        assert!(selectors.is_empty());
    }

    #[test]
    fn test_extract_selectors_oversized_bytecode() {
        use super::super::types::MAX_BYTECODE_SIZE;
        let large_bytecode = vec![0u8; MAX_BYTECODE_SIZE + 1];
        let selectors = extract_selectors(&large_bytecode);
        assert!(selectors.is_empty());
    }

    #[test]
    fn test_extract_selectors_simple() {
        // Simple contract bytecode with a few functions
        // This is a minimal ERC20-like contract
        let bytecode = hex::decode(
            "608060405234801561001057600080fd5b50600436106100365760003560e01c806370a0823114610\
             03b578063a9059cbb14610061575b600080fd5b61004e610049366004610123565b610091565b6040\
             519081526020015b60405180910390f35b610074610063366004610145565b60019392505050565b\
             604051901515815260200161005856",
        )
        .unwrap();

        let selectors = extract_selectors(&bytecode);

        // Should extract at least 2 function selectors
        assert!(selectors.len() >= 2);

        // Check that selectors are properly formatted
        for func in &selectors {
            assert!(func.selector.starts_with("0x"));
            assert_eq!(func.selector.len(), 10); // 0x + 8 hex chars
        }
    }
}
