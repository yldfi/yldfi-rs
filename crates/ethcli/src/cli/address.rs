//! Address book management commands
//!
//! Add, remove, list, and lookup saved addresses.

use super::OutputFormat;
use crate::config::{AddressBook, AddressEntry};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AddressCommands {
    /// Add an address to the address book
    Add {
        /// Label for the address (e.g., "vitalik", "usdc")
        label: String,

        /// Ethereum address (0x...)
        address: String,

        /// Description or notes
        #[arg(long, short)]
        description: Option<String>,

        /// Tags for categorization (can be repeated)
        #[arg(long, short)]
        tag: Vec<String>,

        /// Mark address as chain-specific (e.g., "polygon", "arbitrum")
        #[arg(long = "for-chain")]
        for_chain: Option<String>,
    },

    /// Remove an address from the address book
    Remove {
        /// Label to remove
        label: String,
    },

    /// List all addresses in the address book
    List {
        /// Filter by tag
        #[arg(long, short)]
        tag: Option<String>,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get address by label
    Get {
        /// Label to look up
        label: String,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Search addresses by partial match
    Search {
        /// Search query (matches label, address, description, or tags)
        query: String,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Import addresses from a JSON file
    Import {
        /// Path to JSON file
        file: String,

        /// Overwrite existing entries with same label
        #[arg(long)]
        overwrite: bool,
    },

    /// Export addresses to JSON
    Export {
        /// Output file (stdout if not specified)
        #[arg(long, short)]
        output: Option<String>,
    },
}

pub fn handle(action: &AddressCommands, quiet: bool) -> anyhow::Result<()> {
    match action {
        AddressCommands::Add {
            label,
            address,
            description,
            tag,
            for_chain,
        } => {
            let mut book = AddressBook::load_default();

            // Check if label already exists
            if book.get(label).is_some() {
                return Err(anyhow::anyhow!(
                    "Label '{}' already exists. Use 'ethcli address remove {}' first.",
                    label,
                    label
                ));
            }

            book.add(
                label,
                address,
                description.clone(),
                tag.clone(),
                for_chain.clone(),
            )
            .map_err(|e| anyhow::anyhow!(e))?;

            if !quiet {
                println!("Added '{}' -> {}", label, address);
            }
        }

        AddressCommands::Remove { label } => {
            let mut book = AddressBook::load_default();

            if book.remove(label).map_err(|e| anyhow::anyhow!(e))? {
                if !quiet {
                    println!("Removed '{}'", label);
                }
            } else {
                return Err(anyhow::anyhow!("Label '{}' not found", label));
            }
        }

        AddressCommands::List { tag, output } => {
            let book = AddressBook::load_default();
            let entries = book.list(tag.as_deref());

            if entries.is_empty() {
                if !quiet {
                    if tag.is_some() {
                        println!("No addresses found with tag '{}'", tag.as_ref().unwrap());
                    } else {
                        println!("Address book is empty. Add addresses with 'ethcli address add'");
                    }
                }
                return Ok(());
            }

            if output.is_json() {
                let json_entries: Vec<_> = entries
                    .iter()
                    .map(|(label, entry)| {
                        serde_json::json!({
                            "label": label,
                            "address": entry.address,
                            "description": entry.description,
                            "tags": entry.tags,
                            "chain": entry.chain,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_entries)?);
            } else {
                println!("Address Book");
                println!("{}", "─".repeat(60));

                for (label, entry) in entries {
                    print_entry(label, entry);
                }
            }
        }

        AddressCommands::Get { label, output } => {
            let book = AddressBook::load_default();

            if let Some(entry) = book.get(label) {
                if output.is_json() {
                    println!(
                        "{}",
                        serde_json::json!({
                            "label": label,
                            "address": entry.address,
                            "description": entry.description,
                            "tags": entry.tags,
                            "chain": entry.chain,
                        })
                    );
                } else {
                    print_entry(label, entry);
                }
            } else {
                return Err(anyhow::anyhow!("Label '{}' not found", label));
            }
        }

        AddressCommands::Search { query, output } => {
            let book = AddressBook::load_default();
            let entries = book.search(query);

            if entries.is_empty() {
                if !quiet {
                    println!("No matches found for '{}'", query);
                }
                return Ok(());
            }

            if output.is_json() {
                let json_entries: Vec<_> = entries
                    .iter()
                    .map(|(label, entry)| {
                        serde_json::json!({
                            "label": label,
                            "address": entry.address,
                            "description": entry.description,
                            "tags": entry.tags,
                            "chain": entry.chain,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_entries)?);
            } else {
                println!("Search Results for '{}'", query);
                println!("{}", "─".repeat(60));

                for (label, entry) in entries {
                    print_entry(label, entry);
                }
            }
        }

        AddressCommands::Import { file, overwrite } => {
            let content = std::fs::read_to_string(file)
                .map_err(|e| anyhow::anyhow!("Failed to read file: {}", e))?;

            let entries: Vec<ImportEntry> = serde_json::from_str(&content)
                .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;

            let mut book = AddressBook::load_default();
            let mut added = 0;
            let mut skipped = 0;

            for entry in entries {
                if book.get(&entry.label).is_some() && !overwrite {
                    skipped += 1;
                    continue;
                }

                book.entries.insert(
                    entry.label.to_lowercase(),
                    AddressEntry {
                        address: entry.address,
                        description: entry.description,
                        tags: entry.tags.unwrap_or_default(),
                        chain: entry.chain,
                    },
                );
                added += 1;
            }

            book.save_default().map_err(|e| anyhow::anyhow!(e))?;

            if !quiet {
                println!("Imported {} addresses ({} skipped)", added, skipped);
            }
        }

        AddressCommands::Export { output } => {
            let book = AddressBook::load_default();
            let entries: Vec<_> = book
                .entries
                .iter()
                .map(|(label, entry)| {
                    serde_json::json!({
                        "label": label,
                        "address": entry.address,
                        "description": entry.description,
                        "tags": entry.tags,
                        "chain": entry.chain,
                    })
                })
                .collect();

            let json = serde_json::to_string_pretty(&entries)?;

            if let Some(path) = output {
                std::fs::write(path, &json)?;
                if !quiet {
                    println!("Exported {} addresses to {}", entries.len(), path);
                }
            } else {
                println!("{}", json);
            }
        }
    }

    Ok(())
}

fn print_entry(label: &str, entry: &AddressEntry) {
    println!("{:<15} {}", label, entry.address);

    if let Some(desc) = &entry.description {
        println!("{:<15} {}", "", desc);
    }

    if !entry.tags.is_empty() {
        println!("{:<15} [{}]", "", entry.tags.join(", "));
    }

    if let Some(chain) = &entry.chain {
        println!("{:<15} chain: {}", "", chain);
    }

    println!();
}

#[derive(serde::Deserialize)]
struct ImportEntry {
    label: String,
    address: String,
    description: Option<String>,
    tags: Option<Vec<String>>,
    chain: Option<String>,
}
