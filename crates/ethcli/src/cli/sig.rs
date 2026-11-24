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

/// Canonical form, selector and topic of a signature, computed locally. A
/// `"name(args)(returns)"` return part is cut off; parameter names and
/// spaces are dropped, and `uint`/`int` become `uint256`/`int256`
pub fn hash_signature(signature: &str) -> anyhow::Result<serde_json::Value> {
    let sig = signature.trim();
    let sig = sig
        .strip_prefix("function ")
        .or_else(|| sig.strip_prefix("event "))
        .or_else(|| sig.strip_prefix("error "))
        .unwrap_or(sig)
        .trim();
    let open = sig
        .find('(')
        .ok_or_else(|| anyhow::anyhow!("'{signature}' is not a signature: no '('"))?;
    let mut depth = 0usize;
    let mut close = None;
    for (i, ch) in sig[open..].char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + i);
                    break;
                }
            }
            _ => {}
        }
    }
    let close = close.ok_or_else(|| anyhow::anyhow!("'{signature}': unbalanced parentheses"))?;
    let head = &sig[..=close];
    let function = alloy::json_abi::Function::parse(head)
        .map_err(|e| anyhow::anyhow!("Cannot parse '{head}': {e}"))?;
    let canonical = function.signature();
    let hash = alloy::primitives::keccak256(canonical.as_bytes());
    Ok(serde_json::json!({
        "signature": canonical,
        "selector": format!("0x{}", hex::encode(&hash[..4])),
        "topic": format!("{hash:#x}"),
        "returns_ignored": sig[close + 1..].trim().starts_with('('),
    }))
}

#[cfg(test)]
mod hash_tests {
    use super::hash_signature;

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
        let h = hash_signature("transfer(address to, uint amount)").unwrap();
        assert_eq!(h["signature"], "transfer(address,uint256)");
        assert!(hash_signature("nope").is_err());
    }
}
