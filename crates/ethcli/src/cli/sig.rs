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
        selector: String,
    },

    /// Lookup event signature by topic0 hash
    Event {
        /// Topic0 hash (e.g., 0xddf252ad...)
        topic: String,
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
    match action {
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
