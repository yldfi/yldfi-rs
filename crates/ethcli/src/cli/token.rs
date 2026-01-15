//! Token-related commands
//!
//! Get token info, holders, and balances

use super::OutputFormat;
use crate::config::{AddressBook, Chain};
use crate::etherscan::TokenMetadataCache;
use crate::rpc::get_rpc_endpoint;
use crate::rpc::multicall::{selectors, MulticallBuilder};
use crate::utils::address::resolve_from_book;
use crate::utils::format::format_token_amount;
use alloy::primitives::Address;
use alloy::providers::Provider;
use clap::Subcommand;
use std::str::FromStr;
use std::sync::OnceLock;

/// Global token metadata cache (lazy initialized)
static TOKEN_CACHE: OnceLock<TokenMetadataCache> = OnceLock::new();

fn get_token_cache() -> &'static TokenMetadataCache {
    TOKEN_CACHE.get_or_init(TokenMetadataCache::new)
}

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Get token info (name, symbol, decimals, supply)
    Info {
        /// Token contract address
        address: String,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get top token holders (requires Etherscan Pro plan)
    Holders {
        /// Token contract address
        address: String,

        /// Number of holders to return
        #[arg(long, default_value = "100")]
        limit: u32,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get token balance for holder(s)
    Balance {
        /// Token contract address(es) or labels. Use "eth" for native ETH balance.
        #[arg(required = true)]
        tokens: Vec<String>,

        /// Holder address(es) - can specify multiple
        #[arg(long, num_args = 1..)]
        holder: Vec<String>,

        /// Get balances for all addresses with this tag (can combine with --holder)
        #[arg(long)]
        tag: Option<String>,

        /// Show zero balances (hidden by default when multiple holders)
        #[arg(long)]
        show_zero: bool,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },
}

pub async fn handle(
    action: &TokenCommands,
    chain: Chain,
    _api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TokenCommands::Info { address, output } => {
            let (token_addr, label) = resolve_from_book(address)?;
            let addr_str = format!("{:#x}", token_addr);
            let display = label.as_ref().unwrap_or(&addr_str);

            let cache = get_token_cache();
            let chain_name = chain.name();

            // Check cache first
            if let Some(cached) = cache.get(chain_name, &addr_str) {
                if !quiet {
                    eprintln!("Using cached token info for {}...", display);
                }

                let formatted_supply = match (cached.total_supply.as_ref(), cached.decimals) {
                    (Some(supply), Some(dec)) => format_token_amount(supply, dec),
                    (Some(supply), None) => supply.clone(),
                    _ => "(unknown)".to_string(),
                };

                if output.is_json() {
                    println!(
                        "{}",
                        serde_json::json!({
                            "address": addr_str,
                            "label": label,
                            "name": cached.name,
                            "symbol": cached.symbol,
                            "decimals": cached.decimals,
                            "totalSupply": cached.total_supply,
                            "totalSupplyFormatted": formatted_supply,
                            "cached": true
                        })
                    );
                } else {
                    println!("Token Info (cached)");
                    println!("{}", "─".repeat(40));
                    if let Some(lbl) = &label {
                        println!("Label:    {}", lbl);
                    }
                    println!("Address:  {}", addr_str);
                    println!(
                        "Name:     {}",
                        cached.name.as_deref().unwrap_or("(unknown)")
                    );
                    println!(
                        "Symbol:   {}",
                        cached.symbol.as_deref().unwrap_or("(unknown)")
                    );
                    println!(
                        "Decimals: {}",
                        cached
                            .decimals
                            .map(|d| d.to_string())
                            .unwrap_or_else(|| "(unknown)".to_string())
                    );
                    println!("Supply:   {}", formatted_supply);

                    if let Some(explorer) = chain.explorer_url() {
                        println!("\nExplorer: {}/token/{}", explorer, addr_str);
                    }
                }
                return Ok(());
            }

            if !quiet {
                eprintln!("Fetching token info for {}...", display);
            }

            // Use RPC with Multicall3 for single request
            let endpoint = get_rpc_endpoint(chain)?;
            let provider = endpoint.provider();

            let multicall = MulticallBuilder::new()
                .add_call_allow_failure(token_addr, selectors::name())
                .add_call_allow_failure(token_addr, selectors::symbol())
                .add_call_allow_failure(token_addr, selectors::decimals())
                .add_call_allow_failure(token_addr, selectors::total_supply());

            // Execute with retry (up to 3 retries with exponential backoff)
            let results = multicall.execute_with_retry(provider, 3).await?;

            let name = results.first().and_then(|r| r.decode_string());
            let symbol = results.get(1).and_then(|r| r.decode_string());
            let decimals = results.get(2).and_then(|r| r.decode_uint8());
            let total_supply = results.get(3).and_then(|r| r.decode_uint256());

            // Cache the result (token metadata is immutable)
            cache.set(
                chain_name,
                &addr_str,
                name.clone(),
                symbol.clone(),
                decimals,
                total_supply.map(|s| s.to_string()),
            );

            let formatted_supply = match (total_supply, decimals) {
                (Some(supply), Some(dec)) => format_token_amount(&supply.to_string(), dec),
                (Some(supply), None) => supply.to_string(),
                _ => "(unknown)".to_string(),
            };

            if output.is_json() {
                println!(
                    "{}",
                    serde_json::json!({
                        "address": addr_str,
                        "label": label,
                        "name": name,
                        "symbol": symbol,
                        "decimals": decimals,
                        "totalSupply": total_supply.map(|s| s.to_string()),
                        "totalSupplyFormatted": formatted_supply
                    })
                );
            } else {
                println!("Token Info");
                println!("{}", "─".repeat(40));
                if let Some(lbl) = &label {
                    println!("Label:    {}", lbl);
                }
                println!("Address:  {}", addr_str);
                println!("Name:     {}", name.as_deref().unwrap_or("(unknown)"));
                println!("Symbol:   {}", symbol.as_deref().unwrap_or("(unknown)"));
                println!(
                    "Decimals: {}",
                    decimals
                        .map(|d| d.to_string())
                        .unwrap_or_else(|| "(unknown)".to_string())
                );
                println!("Supply:   {}", formatted_supply);

                if let Some(explorer) = chain.explorer_url() {
                    println!("\nExplorer: {}/token/{}", explorer, addr_str);
                }
            }
        }

        TokenCommands::Holders { .. } => {
            return Err(anyhow::anyhow!(
                "Token holders endpoint requires an Etherscan Pro plan.\n\
                 See: https://docs.etherscan.io/resources/pro-endpoints\n\
                 Alternative: View holders on Etherscan website directly."
            ));
        }

        TokenCommands::Balance {
            tokens,
            holder,
            tag,
            show_zero,
            output,
        } => {
            // Build list of holders from --holder args and --tag
            let mut holders: Vec<(Address, Option<String>)> = Vec::new();

            // Add explicit holders
            for h in holder {
                holders.push(resolve_from_book(h)?);
            }

            // Add tagged addresses
            if let Some(t) = tag {
                let book = AddressBook::load_default();
                let entries = book.list(Some(t.as_str()));
                if entries.is_empty() && holders.is_empty() {
                    return Err(anyhow::anyhow!("No addresses found with tag '{}'", t));
                }
                for (label, entry) in entries {
                    if let Ok(addr) = Address::from_str(&entry.address) {
                        // Avoid duplicates
                        if !holders.iter().any(|(a, _)| *a == addr) {
                            holders.push((addr, Some(label.to_string())));
                        }
                    }
                }
            }

            if holders.is_empty() {
                return Err(anyhow::anyhow!("Must provide --holder or --tag"));
            }

            // Get RPC endpoint
            let endpoint = get_rpc_endpoint(chain)?;
            let provider = endpoint.provider();

            // Collect results for JSON output
            let mut json_results = Vec::new();
            let multiple_tokens = tokens.len() > 1;
            let multiple_holders = holders.len() > 1;

            // Process each token
            for (token_idx, token) in tokens.iter().enumerate() {
                let is_eth = token.eq_ignore_ascii_case("eth");

                // Add separator between tokens in pretty output
                if multiple_tokens && token_idx > 0 && output.is_table() {
                    println!();
                }

                if is_eth {
                    // Native ETH balance
                    let symbol = "ETH";
                    let decimals = 18u8;

                    if !quiet {
                        eprintln!("Fetching ETH balances for {} holder(s)...", holders.len());
                    }

                    // Print header for multiple tokens
                    if multiple_tokens && output.is_table() {
                        println!("═══ {} ═══", symbol);
                    }

                    for (holder_addr, holder_label) in &holders {
                        let holder_str = format!("{:#x}", holder_addr);
                        let holder_display = holder_label.as_ref().unwrap_or(&holder_str);

                        let balance = provider.get_balance(*holder_addr).await?;
                        let formatted = format_token_amount(&balance.to_string(), decimals);

                        if output.is_json() {
                            json_results.push(serde_json::json!({
                                "token": "eth",
                                "tokenLabel": "ETH",
                                "holder": holder_str,
                                "holderLabel": holder_label,
                                "balance": balance.to_string(),
                                "balanceFormatted": formatted,
                                "decimals": decimals,
                                "symbol": symbol
                            }));
                        } else {
                            let should_show = *show_zero || !balance.is_zero() || !multiple_holders;
                            if should_show {
                                println!("{:<20} {:>15} {}", holder_display, formatted, symbol);
                            }
                        }
                    }
                } else {
                    // ERC20 token
                    let (token_addr, token_label) = resolve_from_book(token)?;
                    let token_str = format!("{:#x}", token_addr);
                    let token_display = token_label.as_ref().unwrap_or(&token_str);

                    // Get token decimals and symbol
                    let meta_multicall = MulticallBuilder::new()
                        .add_call_allow_failure(token_addr, selectors::decimals())
                        .add_call_allow_failure(token_addr, selectors::symbol());
                    let meta_results = meta_multicall.execute_with_retry(&provider, 3).await?;
                    let decimals = meta_results
                        .first()
                        .and_then(|r| r.decode_uint8())
                        .unwrap_or(18);
                    let symbol = meta_results
                        .get(1)
                        .and_then(|r| r.decode_string())
                        .unwrap_or_else(|| "???".to_string());

                    // Build multicall for all holder balances at once
                    let mut balance_multicall = MulticallBuilder::new();
                    for (holder_addr, _) in &holders {
                        balance_multicall = balance_multicall.add_call_allow_failure(
                            token_addr,
                            selectors::balance_of(*holder_addr),
                        );
                    }

                    if !quiet {
                        eprintln!(
                            "Fetching {} balances for {} holder(s)...",
                            token_display,
                            holders.len()
                        );
                    }

                    // Print header for multiple tokens
                    if multiple_tokens && output.is_table() {
                        println!("═══ {} ═══", symbol);
                    }

                    let balance_results =
                        balance_multicall.execute_with_retry(&provider, 3).await?;

                    for (i, (holder_addr, holder_label)) in holders.iter().enumerate() {
                        let holder_str = format!("{:#x}", holder_addr);
                        let holder_display = holder_label.as_ref().unwrap_or(&holder_str);

                        let balance = balance_results
                            .get(i)
                            .and_then(|r| r.decode_uint256())
                            .unwrap_or_default();

                        let formatted = format_token_amount(&balance.to_string(), decimals);

                        if output.is_json() {
                            json_results.push(serde_json::json!({
                                "token": token_str,
                                "tokenLabel": token_label,
                                "holder": holder_str,
                                "holderLabel": holder_label,
                                "balance": balance.to_string(),
                                "balanceFormatted": formatted,
                                "decimals": decimals,
                                "symbol": symbol
                            }));
                        } else {
                            let should_show = *show_zero || !balance.is_zero() || !multiple_holders;
                            if should_show {
                                println!("{:<20} {:>15} {}", holder_display, formatted, symbol);
                            }
                        }
                    }
                }
            }

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&json_results)?);
            }
        }
    }

    Ok(())
}
