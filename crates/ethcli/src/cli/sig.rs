//! Signature lookup commands
//!
//! Look up function selectors and event topics from 4byte.directory

use crate::config::Chain;
use crate::etherscan::Client;
use clap::Subcommand;
use std::io::Write;

#[derive(Subcommand)]
pub enum SigCommands {
    /// Lookup function signature by 4-byte selector
    #[command(name = "fn")]
    Function {
        /// 4-byte selector (e.g., 0xa9059cbb)
        #[arg(value_name = "SELECTOR")]
        selector: String,
    },

    /// Lookup event signature by topic0 hash
    Event {
        /// Topic0 hash (e.g., 0xddf252ad...)
        #[arg(value_name = "TOPIC")]
        topic: String,
    },

    /// Lookup a selector or topic, picking function or event by its length
    ///
    /// A 4-byte value is looked up as a function selector and a 32-byte value
    /// as an event topic.
    Lookup {
        /// 4-byte selector or 32-byte topic0 hash
        #[arg(value_name = "HEX")]
        value: String,
    },

    /// Decode revert data: Error(string), Panic(uint256) or a custom error
    ///
    /// Pass the data a reverted call returned. A custom error's selector is
    /// looked up in the signature database and its arguments are decoded.
    /// With a bare 4-byte selector, the matching signatures are listed.
    #[command(visible_alias = "decode-error")]
    Error {
        /// Revert data (0x hex), or a 4-byte error selector
        #[arg(value_name = "DATA")]
        data: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: crate::cli::OutputFormat,
    },

    /// Hash a signature locally to its 4-byte selector and 32-byte topic
    ///
    /// No network. Accepts "transfer(address,uint256)", the return form
    /// "balanceOf(address)(uint256)" (the return part is not hashed), and
    /// parameter names, which are dropped
    Hash {
        /// Function, event or error signature
        #[arg(value_name = "SIGNATURE")]
        signature: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: crate::cli::OutputFormat,
    },

    /// Show signature cache statistics
    CacheStats,

    /// Clear the signature cache
    CacheClear,
}

pub async fn handle(
    action: &SigCommands,
    chain: Chain,
    api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    let resolved;
    let action = if let SigCommands::Lookup { value } = action {
        resolved = classify_lookup(value)?;
        &resolved
    } else {
        action
    };

    match action {
        SigCommands::Lookup { .. } => unreachable!("lookup is resolved to fn or event above"),

        SigCommands::Hash { signature, format } => {
            let h = hash_signature(signature)?;
            if format.is_table() {
                println!("signature {}", h["signature"].as_str().unwrap_or(""));
                println!("selector  {}", h["selector"].as_str().unwrap_or(""));
                println!("topic     {}", h["topic"].as_str().unwrap_or(""));
            } else if format.is_ndjson() {
                println!("{}", serde_json::to_string(&h)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&h)?);
            }
        }

        SigCommands::Error { data, format } => {
            let bytes = hex::decode(data.strip_prefix("0x").unwrap_or(data))
                .map_err(|e| anyhow::anyhow!("Invalid hex data: {e}"))?;
            let mut revert = crate::abi::revert::describe(&bytes, None, chain).await;

            // A bare selector has no arguments to decode against: list the
            // signatures it could be
            let mut candidates = Vec::new();
            if bytes.len() == 4 && revert.kind == crate::abi::revert::RevertKind::Unknown {
                let client = Client::new(chain, api_key)?;
                candidates = client.lookup_selector_all(data).await.unwrap_or_default();
                if !candidates.is_empty() {
                    revert.message = format!("selector of {}", candidates.join(" or "));
                }
            }

            if format.is_table() {
                println!("{}", revert.message);
                if let Some(signature) = &revert.signature {
                    if !quiet {
                        eprintln!("Signature: {signature}");
                    }
                }
            } else {
                let mut out = serde_json::to_value(&revert)?;
                if !candidates.is_empty() {
                    out["candidates"] = serde_json::json!(candidates);
                }
                if format.is_ndjson() {
                    println!("{}", serde_json::to_string(&out)?);
                } else {
                    println!("{}", serde_json::to_string_pretty(&out)?);
                }
            }
        }

        SigCommands::Function { selector } => {
            let client = Client::new(chain, api_key)?;

            if !quiet {
                eprintln!("Looking up selector {}...", selector);
                let _ = std::io::stderr().flush();
            }

            match client.lookup_selector_all(selector).await {
                Some(signatures) => {
                    if signatures.len() == 1 {
                        println!("{}", signatures[0]);
                    } else {
                        // Multiple matches - show all with ranking
                        println!("Found {} matching signatures:", signatures.len());
                        for (i, sig) in signatures.iter().enumerate() {
                            let marker = if i == 0 { " (most common)" } else { "" };
                            println!("  {}. {}{}", i + 1, sig, marker);
                        }
                    }
                }
                None => {
                    eprintln!("No signature found for selector {}", selector);
                    std::process::exit(1);
                }
            }
        }

        SigCommands::Event { topic } => {
            let client = Client::new(chain, api_key)?;

            if !quiet {
                eprintln!("Looking up event topic {}...", topic);
                let _ = std::io::stderr().flush();
            }

            match client.lookup_event_all(topic).await {
                Some(signatures) => {
                    if signatures.len() == 1 {
                        println!("{}", signatures[0]);
                    } else {
                        // Multiple matches - show all with ranking
                        println!("Found {} matching signatures:", signatures.len());
                        for (i, sig) in signatures.iter().enumerate() {
                            let marker = if i == 0 { " (most common)" } else { "" };
                            println!("  {}. {}{}", i + 1, sig, marker);
                        }
                    }
                }
                None => {
                    eprintln!("No signature found for topic {}", topic);
                    std::process::exit(1);
                }
            }
        }

        SigCommands::CacheStats => {
            let client = Client::new(chain, api_key)?;
            let stats = client.cache_stats();

            println!("Signature Cache");
            println!("{}", "─".repeat(40));
            println!(
                "Functions: {} cached ({} valid)",
                stats.total_functions, stats.valid_functions
            );
            println!(
                "Events:    {} cached ({} valid)",
                stats.total_events, stats.valid_events
            );
            println!(
                "ABIs:      {} cached ({} valid)",
                stats.total_abis, stats.valid_abis
            );
            println!("Path:      {}", stats.cache_path.display());
        }

        SigCommands::CacheClear => {
            let client = Client::new(chain, api_key)?;
            client.cache().clear();
            println!("Signature cache cleared");
        }
    }

    Ok(())
}

/// Turn `sig lookup <hex>` into the function or event lookup its length implies
fn classify_lookup(value: &str) -> anyhow::Result<SigCommands> {
    let hex_part = value.strip_prefix("0x").unwrap_or(value);
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!("'{value}' is not hex. Pass a 4-byte selector or a 32-byte topic0 hash");
    }
    match hex_part.len() {
        8 => Ok(SigCommands::Function {
            selector: value.to_string(),
        }),
        64 => Ok(SigCommands::Event {
            topic: value.to_string(),
        }),
        n => anyhow::bail!(
            "'{value}' is {} bytes. A function selector is 4 bytes and an event topic is 32 bytes",
            n.div_ceil(2)
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lookup_picks_function_or_event_by_length() {
        assert!(matches!(
            classify_lookup("0xa9059cbb").unwrap(),
            SigCommands::Function { .. }
        ));
        assert!(matches!(
            classify_lookup("a9059cbb").unwrap(),
            SigCommands::Function { .. }
        ));
        let topic = "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef";
        assert!(matches!(
            classify_lookup(topic).unwrap(),
            SigCommands::Event { .. }
        ));
        assert!(classify_lookup("0x1234").is_err());
        assert!(classify_lookup("transfer").is_err());
    }
}

/// Canonical form, selector and topic of a signature, computed locally.
///
/// Every parameter type is checked: elementary types are validated by
/// alloy's `DynSolType` (`uint`/`int` become `uint256`/`int256`, `byte`
/// becomes `bytes1`), tuples and arrays recursively. Names, data locations
/// (`memory`, `calldata`, `storage`), `indexed` and `address payable` are
/// accepted and dropped. Qualified or user-defined types (`IPool.Params`,
/// `MyStruct`, `IERC20`) are rejected: the canonical ABI type has to be given
/// (contract or interface -> address, enum -> uint8, struct -> a tuple).
/// A return part, `(returns)` or `returns (returns)`, is checked and not
/// hashed; anything else after the parameters is an error.
pub fn hash_signature(signature: &str) -> anyhow::Result<serde_json::Value> {
    let mut sig = signature.trim();
    for kw in ["function", "event", "error"] {
        if let Some(rest) = sig.strip_prefix(kw) {
            if rest.starts_with(char::is_whitespace) {
                sig = rest.trim_start();
                break;
            }
        }
    }
    let open = sig
        .find('(')
        .ok_or_else(|| anyhow::anyhow!("'{signature}' is not a signature: no '('"))?;
    let name = sig[..open].trim();
    if !is_identifier(name) {
        anyhow::bail!("'{signature}': '{name}' is not a valid function, event or error name");
    }
    let close = matching_paren(sig, open)
        .ok_or_else(|| anyhow::anyhow!("'{signature}': unbalanced parentheses"))?;
    let params = canonical_params(&sig[open + 1..close], signature)?;

    // What may follow: nothing, or a return list
    let rest = sig[close + 1..].trim();
    let rest = rest
        .strip_prefix("returns")
        .filter(|r| r.trim_start().starts_with('('))
        .map(str::trim_start)
        .unwrap_or(rest);
    let returns_ignored = !rest.is_empty();
    if returns_ignored {
        let end = if rest.starts_with('(') {
            matching_paren(rest, 0)
        } else {
            None
        }
        .ok_or_else(|| {
            anyhow::anyhow!("'{signature}': unexpected text after the parameters: '{rest}'")
        })?;
        canonical_params(&rest[1..end], signature)?;
        let tail = rest[end + 1..].trim();
        if !tail.is_empty() {
            anyhow::bail!("'{signature}': unexpected text after the return types: '{tail}'");
        }
    }

    let canonical = format!("{name}({})", params.join(","));
    let hash = alloy::primitives::keccak256(canonical.as_bytes());
    Ok(serde_json::json!({
        "signature": canonical,
        "selector": format!("0x{}", hex::encode(&hash[..4])),
        "topic": format!("{hash:#x}"),
        "returns_ignored": returns_ignored,
    }))
}

fn is_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_' || c == '$')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '$')
}

/// Index of the `)` closing the `(` at `open`
fn matching_paren(s: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, ch) in s[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open + i);
                }
            }
            _ => {}
        }
    }
    None
}

/// Canonical types of a comma-separated parameter list
fn canonical_params(list: &str, signature: &str) -> anyhow::Result<Vec<String>> {
    if list.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut parts = Vec::new();
    let (mut depth, mut start) = (0i32, 0usize);
    for (i, ch) in list.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(&list[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    parts.push(&list[start..]);
    parts
        .into_iter()
        .map(|p| canonical_param(p.trim(), signature))
        .collect()
}

/// One parameter: a type, then optional `payable`, data location, `indexed`
/// and a name
fn canonical_param(param: &str, signature: &str) -> anyhow::Result<String> {
    if param.is_empty() {
        anyhow::bail!("'{signature}': empty parameter (a stray comma?)");
    }
    // The type: a tuple `( ... )` or a word, then array suffixes
    let (mut ty, mut rest) = if param.starts_with('(') {
        let close = matching_paren(param, 0)
            .ok_or_else(|| anyhow::anyhow!("'{signature}': unbalanced parentheses"))?;
        let inner = canonical_params(&param[1..close], signature)?;
        if inner.is_empty() {
            anyhow::bail!("'{signature}': an empty tuple '()' is not an ABI type");
        }
        (format!("({})", inner.join(",")), &param[close + 1..])
    } else if let Some(rest) = param.strip_prefix("tuple(") {
        let close = matching_paren(param, 5)
            .ok_or_else(|| anyhow::anyhow!("'{signature}': unbalanced parentheses"))?;
        let inner = canonical_params(&rest[..close - 6], signature)?;
        (format!("({})", inner.join(",")), &param[close + 1..])
    } else {
        let end = param
            .find(|c: char| c.is_whitespace() || c == '[')
            .unwrap_or(param.len());
        let word = &param[..end];
        (elementary(word, signature)?, &param[end..])
    };
    // Array suffixes
    loop {
        let r = rest.trim_start();
        let Some(after) = r.strip_prefix('[') else {
            rest = r;
            break;
        };
        let close = after
            .find(']')
            .ok_or_else(|| anyhow::anyhow!("'{signature}': unclosed '['"))?;
        let size = after[..close].trim();
        if !size.is_empty() && !matches!(size.parse::<u64>(), Ok(n) if n > 0) {
            anyhow::bail!("'{signature}': invalid array size '[{size}]'");
        }
        ty = format!("{ty}[{size}]");
        rest = &after[close + 1..];
    }
    // Modifiers and an optional name
    let mut words = rest.split_whitespace().peekable();
    if ty == "address" && words.peek() == Some(&"payable") {
        words.next();
    }
    let mut name_seen = false;
    for w in words {
        match w {
            "memory" | "calldata" | "storage" | "indexed" if !name_seen => {}
            w if !name_seen && is_identifier(w) => name_seen = true,
            w => anyhow::bail!("'{signature}': unexpected '{w}' in parameter '{param}'"),
        }
    }
    Ok(ty)
}

/// An elementary type, checked and made canonical
fn elementary(word: &str, signature: &str) -> anyhow::Result<String> {
    use alloy::dyn_abi::DynSolType;
    let word = match word {
        "uint" => "uint256",
        "int" => "int256",
        "byte" => "bytes1",
        w => w,
    };
    let user_type = || {
        anyhow::anyhow!(
            "'{signature}': '{word}' is not an ABI type. Give the canonical type instead: a contract or interface is address, an enum is uint8, a struct is a tuple of its member types, e.g. (address,uint256)"
        )
    };
    if word.is_empty() {
        anyhow::bail!("'{signature}': missing parameter type");
    }
    if word.contains('.') || word.starts_with(|c: char| c.is_ascii_uppercase()) {
        return Err(user_type());
    }
    if word == "function" {
        return Ok("function".into());
    }
    if word.starts_with("fixed") || word.starts_with("ufixed") {
        anyhow::bail!("'{signature}': fixed-point types ('{word}') are not supported");
    }
    match DynSolType::parse(word) {
        Ok(
            t @ (DynSolType::Address
            | DynSolType::Bool
            | DynSolType::String
            | DynSolType::Bytes
            | DynSolType::FixedBytes(_)
            | DynSolType::Uint(_)
            | DynSolType::Int(_)),
        ) => {
            let s = t.sol_type_name().into_owned();
            // DynSolType accepts names it then renames; only exact forms pass
            if s == word {
                Ok(s)
            } else {
                anyhow::bail!("'{signature}': '{word}' is not a valid ABI type")
            }
        }
        Ok(_) | Err(_) if is_identifier(word) && !looks_elementary(word) => Err(user_type()),
        _ => anyhow::bail!("'{signature}': '{word}' is not a valid ABI type"),
    }
}

/// Names that are meant as elementary types (so a bad size is reported as
/// invalid, not as a user type)
fn looks_elementary(word: &str) -> bool {
    ["uint", "int", "bytes", "address", "bool", "string"]
        .iter()
        .any(|p| word.starts_with(p))
}

#[cfg(test)]
mod hash_tests {
    use super::hash_signature;

    fn canon(s: &str) -> String {
        hash_signature(s).unwrap()["signature"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn err(s: &str) -> String {
        hash_signature(s).unwrap_err().to_string()
    }

    #[test]
    fn hashes_signatures_locally() {
        let h = hash_signature("transfer(address,uint256)").unwrap();
        assert_eq!(h["selector"], "0xa9059cbb");
        let h = hash_signature("balanceOf(address)(uint256)").unwrap();
        assert_eq!(h["signature"], "balanceOf(address)");
        assert_eq!(h["selector"], "0x70a08231");
        assert_eq!(h["returns_ignored"], true);
        let h = hash_signature(
            "event Transfer(address indexed from, address indexed to, uint256 value)",
        )
        .unwrap();
        assert_eq!(
            h["topic"],
            "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"
        );
    }

    #[test]
    fn accepts_documented_forms() {
        assert_eq!(
            canon("transfer(address to, uint amount)"),
            "transfer(address,uint256)"
        );
        assert_eq!(canon("f(address payable to)"), "f(address)");
        assert_eq!(canon("f ( uint256 a )"), "f(uint256)");
        assert_eq!(canon("f(byte b, int x)"), "f(bytes1,int256)");
        assert_eq!(
            canon("f(string memory s, bytes calldata d)"),
            "f(string,bytes)"
        );
        assert_eq!(
            canon("f((address to, (uint256 a, bytes32[2] b)[] inner) p)"),
            "f((address,(uint256,bytes32[2])[]))"
        );
        assert_eq!(canon("f(uint256[][3])"), "f(uint256[][3])");
        assert_eq!(canon("function f() returns (uint256)"), "f()");
        assert_eq!(canon("f()"), "f()");
    }

    #[test]
    fn rejects_user_types_and_invalid_types() {
        for user in ["f(IPool.Params p)", "f(MyStruct s)", "f(IERC20 token)"] {
            assert!(
                err(user).contains("canonical type"),
                "{user}: {}",
                err(user)
            );
        }
        for bad in [
            "f(unit256)",
            "f(uint7)",
            "f(bytes33)",
            "f(fixed)",
            "f(uint0)",
        ] {
            assert!(hash_signature(bad).is_err(), "{bad} accepted");
        }
    }

    #[test]
    fn rejects_malformed_signatures() {
        for bad in [
            "nope",
            "f(uint256,)",
            "f(,uint256)",
            "f(uint256",
            "f(uint256))",
            "f(uint256) view",
            "f(uint256)(uint256) extra",
            "f(uint256 a b)",
            "1f(uint256)",
            "f(uint256[0])",
            "f(())",
        ] {
            assert!(hash_signature(bad).is_err(), "{bad} accepted");
        }
    }
}
