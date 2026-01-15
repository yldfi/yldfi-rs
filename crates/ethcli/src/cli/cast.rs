//! Cast utilities - type conversions, hashing, and encoding
//!
//! Similar to Foundry's cast utility

use alloy::primitives::{keccak256, utils::parse_units, Address, B256, U256};
use clap::Subcommand;
use std::str::FromStr;

#[derive(Subcommand)]
pub enum CastCommands {
    /// Convert to wei (e.g., "1.5 eth" -> wei)
    ToWei {
        /// Amount to convert
        value: String,
        /// Unit (wei, gwei, eth). Default: eth
        #[arg(default_value = "eth")]
        unit: String,
    },

    /// Convert from wei to unit (e.g., wei -> "1.5 eth")
    FromWei {
        /// Amount in wei
        value: String,
        /// Unit to convert to (wei, gwei, eth). Default: eth
        #[arg(default_value = "eth")]
        unit: String,
    },

    /// Convert decimal to hex
    ToHex {
        /// Decimal number
        value: String,
    },

    /// Convert hex to decimal
    ToDec {
        /// Hex number (with or without 0x prefix)
        value: String,
    },

    /// Compute keccak256 hash
    Keccak {
        /// Data to hash (string or hex with 0x prefix)
        data: String,
    },

    /// Get 4-byte function selector from signature
    Sig {
        /// Function signature (e.g., "transfer(address,uint256)")
        signature: String,
    },

    /// Get event topic0 from signature
    Topic {
        /// Event signature (e.g., "Transfer(address,address,uint256)")
        signature: String,
    },

    /// Checksum an address (EIP-55)
    Checksum {
        /// Address to checksum
        address: String,
    },

    /// Compute CREATE address
    ComputeAddress {
        /// Deployer address
        deployer: String,
        /// Nonce
        nonce: u64,
    },

    /// Compute CREATE2 address
    Create2 {
        /// Deployer/factory address
        deployer: String,
        /// Salt (32 bytes hex)
        salt: String,
        /// Init code hash (32 bytes hex)
        init_code_hash: String,
    },

    /// Concatenate hex strings
    Concat {
        /// Hex strings to concatenate
        #[arg(required = true, num_args = 1..)]
        values: Vec<String>,
    },

    /// Left-pad hex to 32 bytes
    ToBytes32 {
        /// Hex value to pad
        value: String,
    },

    /// ABI-encode function call
    AbiEncode {
        /// Function signature (e.g., "transfer(address,uint256)")
        signature: String,
        /// Arguments
        #[arg(required = false, num_args = 0..)]
        args: Vec<String>,
    },

    /// ABI-decode data
    AbiDecode {
        /// Type signature (e.g., "(address,uint256)" or "transfer(address,uint256)")
        signature: String,
        /// Hex data to decode
        data: String,
    },
}

pub fn handle(action: &CastCommands) -> anyhow::Result<()> {
    match action {
        CastCommands::ToWei { value, unit } => {
            let result = to_wei(value, unit)?;
            println!("{}", result);
        }

        CastCommands::FromWei { value, unit } => {
            let result = from_wei(value, unit)?;
            println!("{}", result);
        }

        CastCommands::ToHex { value } => {
            let num =
                U256::from_str(value).map_err(|e| anyhow::anyhow!("Invalid number: {}", e))?;
            println!("{:#x}", num);
        }

        CastCommands::ToDec { value } => {
            let hex = value.strip_prefix("0x").unwrap_or(value);
            let num =
                U256::from_str_radix(hex, 16).map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
            println!("{}", num);
        }

        CastCommands::Keccak { data } => {
            let bytes = if data.starts_with("0x") {
                hex::decode(data.strip_prefix("0x").unwrap())
                    .map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?
            } else {
                data.as_bytes().to_vec()
            };
            let hash = keccak256(&bytes);
            println!("{:#x}", hash);
        }

        CastCommands::Sig { signature } => {
            let hash = keccak256(signature.as_bytes());
            let selector = &hash[..4];
            println!("0x{}", hex::encode(selector));
        }

        CastCommands::Topic { signature } => {
            let hash = keccak256(signature.as_bytes());
            println!("{:#x}", hash);
        }

        CastCommands::Checksum { address } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
            println!("{}", addr.to_checksum(None));
        }

        CastCommands::ComputeAddress { deployer, nonce } => {
            let deployer_addr = Address::from_str(deployer)
                .map_err(|e| anyhow::anyhow!("Invalid deployer address: {}", e))?;
            let computed = deployer_addr.create(*nonce);
            println!("{}", computed.to_checksum(None));
        }

        CastCommands::Create2 {
            deployer,
            salt,
            init_code_hash,
        } => {
            let deployer_addr = Address::from_str(deployer)
                .map_err(|e| anyhow::anyhow!("Invalid deployer address: {}", e))?;
            let salt_bytes = B256::from_str(salt)
                .map_err(|e| anyhow::anyhow!("Invalid salt (need 32 bytes hex): {}", e))?;
            let init_hash = B256::from_str(init_code_hash)
                .map_err(|e| anyhow::anyhow!("Invalid init code hash: {}", e))?;
            let computed = deployer_addr.create2(salt_bytes, init_hash);
            println!("{}", computed.to_checksum(None));
        }

        CastCommands::Concat { values } => {
            let mut result = String::from("0x");
            for v in values {
                let hex = v.strip_prefix("0x").unwrap_or(v);
                result.push_str(hex);
            }
            println!("{}", result);
        }

        CastCommands::ToBytes32 { value } => {
            let hex = value.strip_prefix("0x").unwrap_or(value);
            let bytes = hex::decode(hex).map_err(|e| anyhow::anyhow!("Invalid hex: {}", e))?;
            if bytes.len() > 32 {
                return Err(anyhow::anyhow!("Value exceeds 32 bytes"));
            }
            let mut padded = [0u8; 32];
            padded[32 - bytes.len()..].copy_from_slice(&bytes);
            println!("0x{}", hex::encode(padded));
        }

        CastCommands::AbiEncode { signature, args } => {
            let result = abi_encode(signature, args)?;
            println!("{}", result);
        }

        CastCommands::AbiDecode { signature, data } => {
            let result = abi_decode(signature, data)?;
            println!("{}", result);
        }
    }

    Ok(())
}

fn to_wei(value: &str, unit: &str) -> anyhow::Result<String> {
    // Use alloy's parse_units for robust decimal handling
    let unit_str = match unit.to_lowercase().as_str() {
        "wei" => "wei",
        "gwei" => "gwei",
        "eth" | "ether" => "ether",
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown unit: {}. Use wei, gwei, or eth",
                unit
            ))
        }
    };

    let parsed = parse_units(value, unit_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse '{}' as {}: {}", value, unit, e))?;

    // Get the absolute value as U256
    let wei: U256 = parsed.get_absolute();
    Ok(wei.to_string())
}

fn from_wei(value: &str, unit: &str) -> anyhow::Result<String> {
    let wei = U256::from_str(value).map_err(|e| anyhow::anyhow!("Invalid wei value: {}", e))?;

    let (divisor, decimals) = match unit.to_lowercase().as_str() {
        "wei" => (U256::from(1), 0),
        "gwei" => (U256::from(1_000_000_000u64), 9),
        "eth" | "ether" => (U256::from(1_000_000_000_000_000_000u64), 18),
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown unit: {}. Use wei, gwei, or eth",
                unit
            ))
        }
    };

    if decimals == 0 {
        return Ok(wei.to_string());
    }

    let integer_part = wei / divisor;
    let remainder = wei % divisor;

    if remainder.is_zero() {
        Ok(format!("{}.0", integer_part))
    } else {
        let remainder_str = format!("{:0>width$}", remainder, width = decimals);
        let trimmed = remainder_str.trim_end_matches('0');
        Ok(format!("{}.{}", integer_part, trimmed))
    }
}

/// Encode a function call with signature and arguments
pub fn abi_encode(signature: &str, args: &[String]) -> anyhow::Result<String> {
    use alloy::dyn_abi::{DynSolType, DynSolValue};

    // Parse signature to get types
    let sig = signature.trim();
    let types_start = sig
        .find('(')
        .ok_or_else(|| anyhow::anyhow!("Invalid signature: missing '('"))?;
    let types_end = sig
        .rfind(')')
        .ok_or_else(|| anyhow::anyhow!("Invalid signature: missing ')'"))?;

    let types_str = &sig[types_start + 1..types_end];

    // Get function selector
    let selector = keccak256(sig.as_bytes());
    let selector_hex = hex::encode(&selector[..4]);

    if types_str.is_empty() {
        return Ok(format!("0x{}", selector_hex));
    }

    // Parse types and encode values
    let type_strs: Vec<&str> = split_types(types_str);

    if type_strs.len() != args.len() {
        return Err(anyhow::anyhow!(
            "Expected {} arguments, got {}",
            type_strs.len(),
            args.len()
        ));
    }

    let mut values = Vec::new();
    for (type_str, arg) in type_strs.iter().zip(args.iter()) {
        let ty = DynSolType::parse(type_str)
            .map_err(|e| anyhow::anyhow!("Invalid type '{}': {}", type_str, e))?;
        let val = ty.coerce_str(arg).map_err(|e| {
            anyhow::anyhow!("Invalid value '{}' for type '{}': {}", arg, type_str, e)
        })?;
        values.push(val);
    }

    // Wrap values in a tuple and encode as parameters
    let tuple = DynSolValue::Tuple(values);
    let encoded = tuple.abi_encode_params();
    Ok(format!("0x{}{}", selector_hex, hex::encode(encoded)))
}

fn abi_decode(signature: &str, data: &str) -> anyhow::Result<String> {
    use alloy::dyn_abi::DynSolType;

    let data_hex = data.strip_prefix("0x").unwrap_or(data);
    let data_bytes =
        hex::decode(data_hex).map_err(|e| anyhow::anyhow!("Invalid hex data: {}", e))?;

    // Check if this is a function signature or just types
    let sig = signature.trim();
    let (types_str, skip_selector) = if sig.contains('(') && !sig.starts_with('(') {
        // Function signature like "transfer(address,uint256)"
        let types_start = sig
            .find('(')
            .ok_or_else(|| anyhow::anyhow!("Invalid signature: missing '('"))?;
        let types_end = sig
            .rfind(')')
            .ok_or_else(|| anyhow::anyhow!("Invalid signature: missing ')'"))?;
        (&sig[types_start + 1..types_end], true)
    } else if sig.starts_with('(') {
        // Tuple type like "(address,uint256)"
        let types_end = sig
            .rfind(')')
            .ok_or_else(|| anyhow::anyhow!("Invalid tuple type: missing closing ')'"))?;
        (&sig[1..types_end], false)
    } else {
        // Single type or comma-separated types
        (sig, false)
    };

    let data_to_decode = if skip_selector && data_bytes.len() > 4 {
        &data_bytes[4..]
    } else {
        &data_bytes
    };

    if types_str.is_empty() {
        return Ok("()".to_string());
    }

    let type_strs: Vec<&str> = split_types(types_str);
    let tuple_type = format!("({})", type_strs.join(","));

    let ty = DynSolType::parse(&tuple_type)
        .map_err(|e| anyhow::anyhow!("Invalid type signature: {}", e))?;

    let decoded = ty
        .abi_decode(data_to_decode)
        .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?;

    Ok(format!("{:?}", decoded))
}

/// Split comma-separated types, handling nested parentheses
fn split_types(types_str: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in types_str.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let t = types_str[start..i].trim();
                if !t.is_empty() {
                    result.push(t);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    let last = types_str[start..].trim();
    if !last.is_empty() {
        result.push(last);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== to_wei tests ====================

    #[test]
    fn test_to_wei_eth_whole() {
        let result = to_wei("1", "eth").unwrap();
        assert_eq!(result, "1000000000000000000");
    }

    #[test]
    fn test_to_wei_eth_decimal() {
        let result = to_wei("1.5", "eth").unwrap();
        assert_eq!(result, "1500000000000000000");
    }

    #[test]
    fn test_to_wei_eth_small_decimal() {
        let result = to_wei("0.001", "eth").unwrap();
        assert_eq!(result, "1000000000000000");
    }

    #[test]
    fn test_to_wei_gwei() {
        let result = to_wei("1", "gwei").unwrap();
        assert_eq!(result, "1000000000");
    }

    #[test]
    fn test_to_wei_gwei_decimal() {
        let result = to_wei("1.5", "gwei").unwrap();
        assert_eq!(result, "1500000000");
    }

    #[test]
    fn test_to_wei_wei() {
        let result = to_wei("1000", "wei").unwrap();
        assert_eq!(result, "1000");
    }

    #[test]
    fn test_to_wei_zero() {
        let result = to_wei("0", "eth").unwrap();
        assert_eq!(result, "0");
    }

    #[test]
    fn test_to_wei_ether_alias() {
        let result = to_wei("1", "ether").unwrap();
        assert_eq!(result, "1000000000000000000");
    }

    #[test]
    fn test_to_wei_invalid_unit() {
        let result = to_wei("1", "invalid");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown unit"));
    }

    #[test]
    fn test_to_wei_invalid_value() {
        let result = to_wei("abc", "eth");
        assert!(result.is_err());
    }

    // ==================== from_wei tests ====================

    #[test]
    fn test_from_wei_to_eth_whole() {
        let result = from_wei("1000000000000000000", "eth").unwrap();
        assert_eq!(result, "1.0");
    }

    #[test]
    fn test_from_wei_to_eth_fractional() {
        let result = from_wei("1500000000000000000", "eth").unwrap();
        assert_eq!(result, "1.5");
    }

    #[test]
    fn test_from_wei_to_eth_small() {
        let result = from_wei("1000000000000000", "eth").unwrap();
        assert_eq!(result, "0.001");
    }

    #[test]
    fn test_from_wei_to_gwei() {
        let result = from_wei("1000000000", "gwei").unwrap();
        assert_eq!(result, "1.0");
    }

    #[test]
    fn test_from_wei_to_gwei_fractional() {
        let result = from_wei("1500000000", "gwei").unwrap();
        assert_eq!(result, "1.5");
    }

    #[test]
    fn test_from_wei_to_wei() {
        let result = from_wei("12345", "wei").unwrap();
        assert_eq!(result, "12345");
    }

    #[test]
    fn test_from_wei_zero() {
        let result = from_wei("0", "eth").unwrap();
        assert_eq!(result, "0.0");
    }

    #[test]
    fn test_from_wei_invalid_unit() {
        let result = from_wei("1000", "invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_from_wei_invalid_value() {
        let result = from_wei("not_a_number", "eth");
        assert!(result.is_err());
    }

    // ==================== split_types tests ====================

    #[test]
    fn test_split_types_simple() {
        let result = split_types("address,uint256");
        assert_eq!(result, vec!["address", "uint256"]);
    }

    #[test]
    fn test_split_types_single() {
        let result = split_types("uint256");
        assert_eq!(result, vec!["uint256"]);
    }

    #[test]
    fn test_split_types_empty() {
        let result = split_types("");
        assert!(result.is_empty());
    }

    #[test]
    fn test_split_types_nested_tuple() {
        let result = split_types("(address,uint256),bytes");
        assert_eq!(result, vec!["(address,uint256)", "bytes"]);
    }

    #[test]
    fn test_split_types_deeply_nested() {
        let result = split_types("address,(uint256,(bool,bytes32)),string");
        assert_eq!(
            result,
            vec!["address", "(uint256,(bool,bytes32))", "string"]
        );
    }

    #[test]
    fn test_split_types_with_spaces() {
        let result = split_types("address, uint256, bytes32");
        assert_eq!(result, vec!["address", "uint256", "bytes32"]);
    }

    // ==================== abi_encode tests ====================

    #[test]
    fn test_abi_encode_no_args() {
        let result = abi_encode("totalSupply()", &[]).unwrap();
        // totalSupply() selector = 0x18160ddd
        assert!(result.starts_with("0x18160ddd"));
        assert_eq!(result.len(), 10); // 0x + 8 hex chars
    }

    #[test]
    fn test_abi_encode_transfer() {
        let args = vec![
            "0x0000000000000000000000000000000000000001".to_string(),
            "1000".to_string(),
        ];
        let result = abi_encode("transfer(address,uint256)", &args).unwrap();
        // transfer(address,uint256) selector = 0xa9059cbb
        assert!(result.starts_with("0xa9059cbb"));
    }

    #[test]
    fn test_abi_encode_wrong_arg_count() {
        let args = vec!["0x0000000000000000000000000000000000000001".to_string()];
        let result = abi_encode("transfer(address,uint256)", &args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Expected 2"));
    }

    // ==================== abi_decode tests ====================

    #[test]
    fn test_abi_decode_uint256() {
        // Encoded value of 1000 as uint256
        let data = "0x00000000000000000000000000000000000000000000000000000000000003e8";
        let result = abi_decode("(uint256)", data).unwrap();
        assert!(result.contains("1000"));
    }

    #[test]
    fn test_abi_decode_empty_types() {
        let result = abi_decode("()", "0x").unwrap();
        assert_eq!(result, "()");
    }

    // ==================== function selector tests ====================

    #[test]
    fn test_transfer_selector() {
        let hash = keccak256("transfer(address,uint256)".as_bytes());
        let selector = hex::encode(&hash[..4]);
        assert_eq!(selector, "a9059cbb");
    }

    #[test]
    fn test_approve_selector() {
        let hash = keccak256("approve(address,uint256)".as_bytes());
        let selector = hex::encode(&hash[..4]);
        assert_eq!(selector, "095ea7b3");
    }

    #[test]
    fn test_balance_of_selector() {
        let hash = keccak256("balanceOf(address)".as_bytes());
        let selector = hex::encode(&hash[..4]);
        assert_eq!(selector, "70a08231");
    }

    // ==================== event topic tests ====================

    #[test]
    fn test_transfer_event_topic() {
        let hash = keccak256("Transfer(address,address,uint256)".as_bytes());
        assert_eq!(
            format!("{:#x}", hash),
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
        );
    }

    #[test]
    fn test_approval_event_topic() {
        let hash = keccak256("Approval(address,address,uint256)".as_bytes());
        assert_eq!(
            format!("{:#x}", hash),
            "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925"
        );
    }
}
