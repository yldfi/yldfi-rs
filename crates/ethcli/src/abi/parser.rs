//! Event signature parsing
//!
//! Parses event signatures like "Transfer(address indexed from, address indexed to, uint256 value)"
//! into structured form for topic matching and log decoding.

use crate::error::{AbiError, Result};
use alloy::primitives::{keccak256, B256};

/// A parsed event parameter
#[derive(Debug, Clone)]
pub struct ParsedParam {
    /// Parameter name (may be empty)
    pub name: String,
    /// Solidity type (e.g., "address", "uint256", "bytes32")
    pub ty: String,
    /// Whether this parameter is indexed (appears in topics)
    pub indexed: bool,
}

/// A parsed event signature
#[derive(Debug, Clone)]
pub struct EventSignature {
    /// Event name
    pub name: String,
    /// Parameters
    pub params: Vec<ParsedParam>,
    /// Event topic (keccak256 of canonical signature)
    pub topic: B256,
    /// Canonical signature (for hashing)
    pub canonical: String,
}

impl EventSignature {
    /// Parse an event signature string
    ///
    /// Accepts formats like:
    /// - "Transfer(address,address,uint256)"
    /// - "Transfer(address indexed from, address indexed to, uint256 value)"
    /// - "Transfer(address indexed, address indexed, uint256)"
    pub fn parse(signature: &str) -> Result<Self> {
        let sig = signature.trim();

        // Find the opening parenthesis
        let paren_pos = sig
            .find('(')
            .ok_or_else(|| AbiError::InvalidEventSignature(format!("Missing '(' in: {}", sig)))?;

        // Extract event name
        let name = sig[..paren_pos].trim().to_string();
        if name.is_empty() {
            return Err(AbiError::InvalidEventSignature("Empty event name".to_string()).into());
        }

        // Ensure it ends with ')'
        if !sig.ends_with(')') {
            return Err(AbiError::InvalidEventSignature(format!("Missing ')' in: {}", sig)).into());
        }

        // Extract parameters string
        let params_str = &sig[paren_pos + 1..sig.len() - 1];

        // Parse parameters
        let params = Self::parse_params(params_str)?;

        // Build canonical signature (types only, no names or indexed)
        let canonical = Self::build_canonical(&name, &params);

        // Calculate topic
        let topic = keccak256(canonical.as_bytes());

        Ok(Self {
            name,
            params,
            topic,
            canonical,
        })
    }

    /// Parse parameter list
    fn parse_params(params_str: &str) -> Result<Vec<ParsedParam>> {
        if params_str.trim().is_empty() {
            return Ok(Vec::new());
        }

        let mut params = Vec::new();
        let mut depth = 0;
        let mut current = String::new();

        // Handle nested types like tuple(uint256,address)
        for ch in params_str.chars() {
            match ch {
                '(' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    params.push(Self::parse_single_param(current.trim())?);
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        // Don't forget the last parameter
        if !current.trim().is_empty() {
            params.push(Self::parse_single_param(current.trim())?);
        }

        Ok(params)
    }

    /// Parse a single parameter
    fn parse_single_param(param_str: &str) -> Result<ParsedParam> {
        let parts: Vec<&str> = param_str.split_whitespace().collect();

        if parts.is_empty() {
            return Err(AbiError::InvalidEventSignature("Empty parameter".to_string()).into());
        }

        // Determine type, indexed, and name
        let (ty, indexed, name) = if parts.len() == 1 {
            // Just type: "address" or "uint256"
            (parts[0].to_string(), false, String::new())
        } else if parts.len() == 2 {
            if parts[1] == "indexed" {
                // "address indexed"
                (parts[0].to_string(), true, String::new())
            } else {
                // "address from" (type + name)
                (parts[0].to_string(), false, parts[1].to_string())
            }
        } else if parts.len() == 3 {
            if parts[1] == "indexed" {
                // "address indexed from"
                (parts[0].to_string(), true, parts[2].to_string())
            } else {
                return Err(AbiError::InvalidEventSignature(format!(
                    "Invalid parameter format: {}",
                    param_str
                ))
                .into());
            }
        } else {
            return Err(AbiError::InvalidEventSignature(format!(
                "Too many parts in parameter: {}",
                param_str
            ))
            .into());
        };

        // Validate type
        Self::validate_type(&ty)?;

        Ok(ParsedParam { name, ty, indexed })
    }

    /// Validate a Solidity type
    fn validate_type(ty: &str) -> Result<()> {
        // Basic type validation
        let valid_base_types = ["address", "bool", "string", "bytes", "int", "uint"];

        let ty_lower = ty.to_lowercase();

        // Check for arrays
        let base_ty = if ty_lower.contains('[') {
            ty_lower.split('[').next().unwrap_or(&ty_lower)
        } else {
            &ty_lower
        };

        // Check base type
        let is_valid = valid_base_types.iter().any(|t| base_ty.starts_with(t))
            || base_ty.starts_with("bytes")
            || base_ty.starts_with("tuple");

        if !is_valid {
            return Err(AbiError::InvalidEventSignature(format!("Invalid type: {}", ty)).into());
        }

        Ok(())
    }

    /// Build canonical signature for hashing
    fn build_canonical(name: &str, params: &[ParsedParam]) -> String {
        let param_types: Vec<&str> = params.iter().map(|p| p.ty.as_str()).collect();
        format!("{}({})", name, param_types.join(","))
    }

    /// Get indexed parameters
    pub fn indexed_params(&self) -> Vec<&ParsedParam> {
        self.params.iter().filter(|p| p.indexed).collect()
    }

    /// Get non-indexed parameters
    pub fn non_indexed_params(&self) -> Vec<&ParsedParam> {
        self.params.iter().filter(|p| !p.indexed).collect()
    }

    /// Get parameter count
    pub fn param_count(&self) -> usize {
        self.params.len()
    }

    /// Check if matches a log's topic0
    pub fn matches_topic(&self, topic0: &B256) -> bool {
        &self.topic == topic0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_signature() {
        let sig = EventSignature::parse("Transfer(address,address,uint256)").unwrap();

        assert_eq!(sig.name, "Transfer");
        assert_eq!(sig.params.len(), 3);
        assert_eq!(sig.params[0].ty, "address");
        assert_eq!(sig.params[1].ty, "address");
        assert_eq!(sig.params[2].ty, "uint256");
        assert_eq!(sig.canonical, "Transfer(address,address,uint256)");
    }

    #[test]
    fn test_parse_indexed_signature() {
        let sig = EventSignature::parse(
            "Transfer(address indexed from, address indexed to, uint256 value)",
        )
        .unwrap();

        assert_eq!(sig.name, "Transfer");
        assert_eq!(sig.params.len(), 3);
        assert!(sig.params[0].indexed);
        assert!(sig.params[1].indexed);
        assert!(!sig.params[2].indexed);
        assert_eq!(sig.params[0].name, "from");
        assert_eq!(sig.params[1].name, "to");
        assert_eq!(sig.params[2].name, "value");
    }

    #[test]
    fn test_topic_calculation() {
        // Known topic for Transfer(address,address,uint256)
        let sig = EventSignature::parse("Transfer(address,address,uint256)").unwrap();

        // ERC20 Transfer topic
        let expected = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
        assert_eq!(format!("{:#x}", sig.topic), expected);
    }

    #[test]
    fn test_approval_topic() {
        let sig = EventSignature::parse("Approval(address,address,uint256)").unwrap();

        // ERC20 Approval topic
        let expected = "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925";
        assert_eq!(format!("{:#x}", sig.topic), expected);
    }

    #[test]
    fn test_indexed_count() {
        let sig = EventSignature::parse(
            "Transfer(address indexed from, address indexed to, uint256 value)",
        )
        .unwrap();

        assert_eq!(sig.indexed_params().len(), 2);
        assert_eq!(sig.non_indexed_params().len(), 1);
    }

    #[test]
    fn test_empty_params() {
        let sig = EventSignature::parse("Paused()").unwrap();

        assert_eq!(sig.name, "Paused");
        assert_eq!(sig.params.len(), 0);
    }

    #[test]
    fn test_invalid_signature() {
        assert!(EventSignature::parse("NoParens").is_err());
        assert!(EventSignature::parse("(address)").is_err());
        assert!(EventSignature::parse("Test(address").is_err());
    }
}
