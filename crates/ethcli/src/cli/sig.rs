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
