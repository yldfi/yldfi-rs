//! Account-related commands
//!
//! Query balances, transactions, and token transfers for addresses

use super::OutputFormat;
use crate::config::Chain;
use crate::etherscan::Client;
use crate::rpc::get_rpc_endpoint;
use crate::rpc::multicall::{selectors, MulticallBuilder, MULTICALL3_ADDRESS};
use crate::utils::address::{is_ens_name, resolve_from_book};
use crate::utils::format::format_wei_to_eth;
use alloy::primitives::Address;
use alloy::providers::Provider;
use clap::Subcommand;
use std::io::Write;
use std::str::FromStr;

/// Resolve an address from label, ENS name, or raw address
/// Checks address book first, then tries ENS resolution
async fn resolve_address(
    input: &str,
    chain: Chain,
    quiet: bool,
) -> anyhow::Result<(Address, Option<String>)> {
    // Try sync resolution first (hex address or address book)
    match resolve_from_book(input) {
        Ok(result) => return Ok(result),
        Err(_) if is_ens_name(input) => {
            // Fall through to ENS resolution
        }
        Err(e) => return Err(e),
    }

    // Try ENS resolution if it looks like an ENS name
    // Only works on Ethereum mainnet
    if chain != Chain::Ethereum {
        return Err(anyhow::anyhow!("ENS is only available on Ethereum mainnet"));
    }

    if !quiet {
        eprintln!("Resolving ENS name {}...", input);
        let _ = std::io::stderr().flush();
    }

    // Get RPC endpoint for ENS resolution (smart selection)
    let endpoint = get_rpc_endpoint(chain)?;
    let provider = endpoint.provider();

    let address = crate::cli::ens::resolve_name(&provider, input).await?;
    Ok((address, Some(input.to_string())))
}

#[derive(Subcommand)]
pub enum AccountCommands {
    /// Get comprehensive account information (balance, recent txs, tokens)
    Info {
        /// Address to query
        address: String,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get native token balance for address(es)
    #[command(visible_alias = "bal")]
    Balance {
        /// Address(es) to query
        addresses: Vec<String>,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get transaction history for an address
    Txs {
        /// Address to query
        address: String,

        /// Page number (1-indexed)
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Sort order (asc, desc)
        #[arg(long, default_value = "desc")]
        sort: String,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get internal transactions for an address
    InternalTxs {
        /// Address to query
        address: String,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get ERC20 token transfers for an address
    Erc20 {
        /// Address to query
        address: String,

        /// Filter by token contract address
        #[arg(long)]
        token: Option<String>,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get ERC721 (NFT) transfers for an address
    Erc721 {
        /// Address to query
        address: String,

        /// Filter by token contract address
        #[arg(long)]
        token: Option<String>,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get ERC1155 transfers for an address
    Erc1155 {
        /// Address to query
        address: String,

        /// Filter by token contract address
        #[arg(long)]
        token: Option<String>,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },

    /// Get blocks mined/validated by an address
    MinedBlocks {
        /// Address to query
        address: String,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,

        /// Number of results per page
        #[arg(long, default_value = "50")]
        limit: u64,

        /// Output format (json, table/pretty)
        #[arg(long, short, value_enum, default_value = "table")]
        output: OutputFormat,
    },
}

pub async fn handle(
    action: &AccountCommands,
    chain: Chain,
    api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = Client::new(chain, api_key)?;

    match action {
        AccountCommands::Info { address, output } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching account info for {}...", address);
                let _ = std::io::stderr().flush();
            }

            // Fetch balance
            let balance = client.get_ether_balance_single(&addr, None).await?;
            let balance_eth = format_wei_to_eth(&balance.balance.to_string());

            // Fetch recent transactions (last 10)
            let tx_params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: 99999999,
                page: 1,
                offset: 10,
                sort: foundry_block_explorers::account::Sort::Desc,
            };
            let txs = client.get_transactions(&addr, Some(tx_params)).await.ok();

            // Fetch ERC20 transfers to identify token holdings
            let token_params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: 99999999,
                page: 1,
                offset: 20,
                sort: foundry_block_explorers::account::Sort::Desc,
            };
            let token_query = foundry_block_explorers::account::TokenQueryOption::ByAddress(addr);
            let token_transfers = client
                .get_erc20_token_transfer_events(token_query, Some(token_params))
                .await
                .ok();

            if output.is_json() {
                let json = serde_json::json!({
                    "address": format!("{:#x}", addr),
                    "balance": balance_eth,
                    "balanceWei": balance.balance.to_string(),
                    "symbol": chain.native_symbol(),
                    "recentTransactions": txs.as_ref().map(|txs| txs.iter().take(10).map(|tx| {
                        serde_json::json!({
                            "hash": format!("{:#x}", tx.hash.value().unwrap_or_default()),
                            "blockNumber": tx.block_number.as_number().map(|n| n.to::<u64>()),
                            "from": tx.from.value().map(|a| format!("{:#x}", a)),
                            "to": tx.to.map(|a| format!("{:#x}", a)),
                            "value": tx.value.to_string(),
                            "valueEth": format_wei_to_eth(&tx.value.to_string()),
                        })
                    }).collect::<Vec<_>>()),
                    "recentTokenTransfers": token_transfers.as_ref().map(|transfers| transfers.iter().take(10).map(|tx| {
                        serde_json::json!({
                            "token": &tx.token_symbol,
                            "tokenAddress": format!("{:#x}", tx.contract_address),
                            "from": format!("{:#x}", tx.from),
                            "to": tx.to.as_ref().map(|a| format!("{:#x}", a)),
                            "value": &tx.value,
                        })
                    }).collect::<Vec<_>>()),
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                println!("Account: {}", address);
                println!("{}", "═".repeat(60));
                println!("Balance: {} {}", balance_eth, chain.native_symbol());

                if let Some(ref txs) = txs {
                    println!("\nRecent Transactions ({}):", txs.len().min(10));
                    println!("{}", "─".repeat(60));
                    for tx in txs.iter().take(5) {
                        let hash = format!("{:#x}", tx.hash.value().unwrap_or_default());
                        let value_eth = format_wei_to_eth(&tx.value.to_string());
                        let to_addr = tx
                            .to
                            .map(|a| format!("{:#x}", a))
                            .unwrap_or_else(|| "contract".to_string());
                        let status = if tx.is_error == "0" { "✓" } else { "✗" };
                        println!(
                            "  {} {} {} {} → {}",
                            status,
                            &hash[..12],
                            value_eth,
                            chain.native_symbol(),
                            truncate_addr(&to_addr)
                        );
                    }
                    if txs.len() > 5 {
                        println!("  ... and {} more", txs.len() - 5);
                    }
                }

                if let Some(ref transfers) = token_transfers {
                    if !transfers.is_empty() {
                        println!("\nRecent Token Activity:");
                        println!("{}", "─".repeat(60));
                        // Show unique tokens
                        let mut seen_tokens: std::collections::HashSet<String> =
                            std::collections::HashSet::new();
                        for tx in transfers.iter() {
                            let key = format!(
                                "{} ({})",
                                tx.token_symbol,
                                truncate_addr(&format!("{:#x}", tx.contract_address))
                            );
                            if seen_tokens.insert(key.clone()) && seen_tokens.len() <= 5 {
                                println!("  • {}", key);
                            }
                        }
                        if seen_tokens.len() > 5 {
                            println!("  ... and {} more tokens", seen_tokens.len() - 5);
                        }
                    }
                }

                // Explorer link
                if let Some(explorer) = chain.explorer_url() {
                    println!("\nExplorer: {}/address/{}", explorer, address);
                }
            }
        }

        AccountCommands::Balance { addresses, output } => {
            if addresses.is_empty() {
                return Err(anyhow::anyhow!("At least one address is required"));
            }

            // Resolve ENS names and parse addresses
            let mut resolved: Vec<(Address, Option<String>)> = Vec::with_capacity(addresses.len());
            for addr_str in addresses {
                let (addr, ens_name) = resolve_address(addr_str, chain, quiet).await?;
                resolved.push((addr, ens_name));
            }

            if !quiet {
                eprintln!(
                    "Fetching balance for {} address(es) on {}...",
                    resolved.len(),
                    chain.display_name()
                );
                let _ = std::io::stderr().flush();
            }

            // Use RPC endpoint instead of Etherscan API
            let endpoint = get_rpc_endpoint(chain)?;
            let provider = endpoint.provider();

            if resolved.len() == 1 {
                // Single address - direct RPC call
                let (addr, label) = &resolved[0];
                let balance = provider
                    .get_balance(*addr)
                    .await
                    .map_err(|e| anyhow::anyhow!("Balance failed: {}", e))?;
                let balance_eth = format_wei_to_eth(&balance.to_string());

                if output.is_json() {
                    let mut json = serde_json::json!({
                        "address": format!("{:#x}", addr),
                        "balance_wei": balance.to_string(),
                        "balance": balance_eth,
                        "symbol": chain.native_symbol()
                    });
                    if let Some(name) = label {
                        json["label"] = serde_json::json!(name);
                    }
                    println!("{}", json);
                } else {
                    // Show label if resolved
                    if let Some(name) = label {
                        println!("{} ({})", name, truncate_addr(&format!("{:#x}", addr)));
                    }
                    println!("{} {}", balance_eth, chain.native_symbol());
                    // Add explorer link
                    if let Some(explorer) = chain.explorer_url() {
                        println!("\nExplorer: {}/address/{:#x}", explorer, addr);
                    }
                }
            } else {
                // Multiple addresses - use Multicall3 for single RPC call
                let mut multicall = MulticallBuilder::new();
                for (addr, _) in &resolved {
                    multicall = multicall.add_call_allow_failure(
                        MULTICALL3_ADDRESS,
                        selectors::get_eth_balance(*addr),
                    );
                }

                // Execute with retry (up to 3 retries with exponential backoff)
                let multicall_results = multicall.execute_with_retry(provider, 3).await?;

                let mut results = Vec::new();
                for (i, (addr, label)) in resolved.iter().enumerate() {
                    let balance = multicall_results
                        .get(i)
                        .and_then(|r| r.decode_uint256())
                        .ok_or_else(|| anyhow::anyhow!("Balance failed for {}", addr))?;
                    results.push((*addr, label.clone(), balance));
                }

                if output.is_json() {
                    let json_results: Vec<serde_json::Value> = results
                        .iter()
                        .map(|(addr, label, bal)| {
                            let mut json = serde_json::json!({
                                "address": format!("{:#x}", addr),
                                "balance_wei": bal.to_string(),
                                "balance": format_wei_to_eth(&bal.to_string()),
                                "symbol": chain.native_symbol()
                            });
                            if let Some(name) = label {
                                json["label"] = serde_json::json!(name);
                            }
                            json
                        })
                        .collect();
                    println!("{}", serde_json::to_string_pretty(&json_results)?);
                } else {
                    for (addr, label, bal) in &results {
                        let eth = format_wei_to_eth(&bal.to_string());
                        if let Some(name) = label {
                            println!(
                                "{} ({}): {} {}",
                                name,
                                truncate_addr(&format!("{:#x}", addr)),
                                eth,
                                chain.native_symbol()
                            );
                        } else {
                            println!("{:#x}: {} {}", addr, eth, chain.native_symbol());
                        }
                    }
                }
            }
        }

        AccountCommands::Txs {
            address,
            page,
            limit,
            sort,
            output,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching transactions for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: 99999999,
                page: *page,
                offset: *limit,
                sort: if sort == "asc" {
                    foundry_block_explorers::account::Sort::Asc
                } else {
                    foundry_block_explorers::account::Sort::Desc
                },
            };

            let txs = client.get_transactions(&addr, Some(params)).await?;

            if output.is_json() {
                // Manually construct JSON to avoid GenesisOption serialization issues
                let json_txs: Vec<serde_json::Value> = txs
                    .iter()
                    .map(|tx| {
                        serde_json::json!({
                            "hash": format!("{:#x}", tx.hash.value().unwrap_or_default()),
                            "blockNumber": tx.block_number.as_number().map(|n| n.to::<u64>()),
                            "timeStamp": tx.time_stamp,
                            "dateTime": format_timestamp(&tx.time_stamp),
                            "from": tx.from.value().map(|a| format!("{:#x}", a)),
                            "to": tx.to.map(|a| format!("{:#x}", a)),
                            "value": tx.value.to_string(),
                            "valueEth": format_wei_to_eth(&tx.value.to_string()),
                            "gas": tx.gas.to_string(),
                            "gasPrice": tx.gas_price.map(|p| p.to_string()),
                            "gasPriceGwei": tx.gas_price.map(|p| format_wei_to_gwei(&p.to_string())),
                            "gasUsed": tx.gas_used.to_string(),
                            "input": &tx.input,
                            "methodId": tx.method_id.as_ref(),
                            "functionName": tx.function_name.as_ref(),
                            "contractAddress": tx.contract_address.map(|a| format!("{:#x}", a)),
                            "isError": &tx.is_error,
                            "txreceipt_status": &tx.tx_receipt_status,
                            "confirmations": tx.confirmations,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&json_txs)?);
            } else {
                println!("Transactions for {}", address);
                println!("{}", "─".repeat(80));
                for tx in txs.iter().take(20) {
                    let value_eth = format_wei_to_eth(&tx.value.to_string());
                    let status = if tx.is_error == "0" { "✓" } else { "✗" };
                    let to_addr = tx
                        .to
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "contract creation".to_string());
                    let hash_str = format!("{:#x}", tx.hash.value().unwrap_or_default());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "{} {} | Block {} | {} {} → {}",
                        status,
                        &hash_str[..hash_str.len().min(12)],
                        block_num,
                        value_eth,
                        chain.native_symbol(),
                        truncate_addr(&to_addr)
                    );
                }
                if txs.len() > 20 {
                    println!("... and {} more", txs.len() - 20);
                }
                // Add explorer link
                if let Some(explorer) = chain.explorer_url() {
                    println!("\nExplorer: {}/address/{}", explorer, address);
                }
            }
        }

        AccountCommands::InternalTxs {
            address,
            page,
            limit,
            output,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching internal transactions for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let query = foundry_block_explorers::account::InternalTxQueryOption::ByAddress(addr);
            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: 99999999,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };
            let txs = client
                .get_internal_transactions(query, Some(params))
                .await?;

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&txs)?);
            } else {
                println!("Internal Transactions for {}", address);
                println!("{}", "─".repeat(80));
                for tx in txs.iter().take(20) {
                    let value_eth = format_wei_to_eth(&tx.value.to_string());
                    let to_addr = tx
                        .to
                        .value()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | {} {} | {} → {}",
                        block_num,
                        value_eth,
                        chain.native_symbol(),
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if txs.len() > 20 {
                    println!("... and {} more", txs.len() - 20);
                }
            }
        }

        AccountCommands::Erc20 {
            address,
            token,
            page,
            limit,
            output,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching ERC20 transfers for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let query = if let Some(tok) = token {
                let tok_addr = Address::from_str(tok)
                    .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;
                foundry_block_explorers::account::TokenQueryOption::ByAddressAndContract(
                    addr, tok_addr,
                )
            } else {
                foundry_block_explorers::account::TokenQueryOption::ByAddress(addr)
            };

            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: 99999999,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };

            let transfers = client
                .get_erc20_token_transfer_events(query, Some(params))
                .await?;

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&transfers)?);
            } else {
                println!("ERC20 Transfers for {}", address);
                println!("{}", "─".repeat(80));
                for tx in transfers.iter().take(20) {
                    let symbol = &tx.token_symbol;
                    let to_addr = tx
                        .to
                        .as_ref()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | {} {} | {} → {}",
                        block_num,
                        tx.value,
                        symbol,
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if transfers.len() > 20 {
                    println!("... and {} more", transfers.len() - 20);
                }
            }
        }

        AccountCommands::Erc721 {
            address,
            token,
            page,
            limit,
            output,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching ERC721 transfers for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let query = if let Some(tok) = token {
                let tok_addr = Address::from_str(tok)
                    .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;
                foundry_block_explorers::account::TokenQueryOption::ByAddressAndContract(
                    addr, tok_addr,
                )
            } else {
                foundry_block_explorers::account::TokenQueryOption::ByAddress(addr)
            };

            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: 99999999,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };

            let transfers = client
                .get_erc721_token_transfer_events(query, Some(params))
                .await?;

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&transfers)?);
            } else {
                println!("ERC721 Transfers for {}", address);
                println!("{}", "─".repeat(80));
                for tx in transfers.iter().take(20) {
                    let name = &tx.token_name;
                    let to_addr = tx
                        .to
                        .as_ref()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | Token ID {} | {} | {} → {}",
                        block_num,
                        tx.token_id,
                        name,
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if transfers.len() > 20 {
                    println!("... and {} more", transfers.len() - 20);
                }
            }
        }

        AccountCommands::Erc1155 {
            address,
            token,
            page,
            limit,
            output,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching ERC1155 transfers for {}...", address);
                let _ = std::io::stderr().flush();
            }

            let query = if let Some(tok) = token {
                let tok_addr = Address::from_str(tok)
                    .map_err(|e| anyhow::anyhow!("Invalid token address: {}", e))?;
                foundry_block_explorers::account::TokenQueryOption::ByAddressAndContract(
                    addr, tok_addr,
                )
            } else {
                foundry_block_explorers::account::TokenQueryOption::ByAddress(addr)
            };

            let params = foundry_block_explorers::account::TxListParams {
                start_block: 0,
                end_block: 99999999,
                page: *page,
                offset: *limit,
                sort: foundry_block_explorers::account::Sort::Desc,
            };

            let transfers = client
                .get_erc1155_token_transfer_events(query, Some(params))
                .await?;

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&transfers)?);
            } else {
                println!("ERC1155 Transfers for {}", address);
                println!("{}", "─".repeat(80));
                for tx in transfers.iter().take(20) {
                    let to_addr = tx
                        .to
                        .as_ref()
                        .map(|a| format!("{:#x}", a))
                        .unwrap_or_else(|| "n/a".to_string());
                    let block_num = tx
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | Token ID {} | Qty {} | {} → {}",
                        block_num,
                        tx.token_id,
                        tx.token_value,
                        truncate_addr(&format!("{:#x}", tx.from)),
                        truncate_addr(&to_addr)
                    );
                }
                if transfers.len() > 20 {
                    println!("... and {} more", transfers.len() - 20);
                }
            }
        }

        AccountCommands::MinedBlocks {
            address,
            page: _page,
            limit: _limit,
            output,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching mined blocks for {}...", address);
                let _ = std::io::stderr().flush();
            }

            // Note: get_mined_blocks API expects BlockType, not page/limit
            // Using None for pagination to get default results
            let blocks = client.get_mined_blocks(&addr, None, None).await?;

            if output.is_json() {
                println!("{}", serde_json::to_string_pretty(&blocks)?);
            } else {
                println!("Blocks Mined by {}", address);
                println!("{}", "─".repeat(60));
                for block in blocks.iter().take(20) {
                    let reward_eth = format_wei_to_eth(&block.block_reward.to_string());
                    let block_num = block
                        .block_number
                        .as_number()
                        .map(|n| n.to::<u64>())
                        .unwrap_or(0);
                    println!(
                        "Block {} | Reward: {} {}",
                        block_num,
                        reward_eth,
                        chain.native_symbol()
                    );
                }
                if blocks.len() > 20 {
                    println!("... and {} more", blocks.len() - 20);
                }
            }
        }
    }

    Ok(())
}

/// Truncate address for display
fn truncate_addr(addr: &str) -> String {
    if addr.len() > 12 {
        format!("{}...{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

/// Format wei to gwei string (shows up to 2 decimal places)
fn format_wei_to_gwei(wei: &str) -> String {
    // Parse wei and divide by 1e9 to get gwei
    wei.parse::<u128>()
        .map(|w| {
            let gwei = w / 1_000_000_000;
            let remainder = (w % 1_000_000_000) / 10_000_000; // 2 decimal places
            if remainder == 0 {
                format!("{}", gwei)
            } else {
                format!("{}.{:02}", gwei, remainder)
            }
        })
        .unwrap_or_else(|_| wei.to_string())
}

/// Format unix timestamp to human-readable date (simple implementation)
fn format_timestamp(ts: &str) -> String {
    ts.parse::<i64>()
        .ok()
        .map(|secs| {
            // Simple date calculation from unix timestamp
            let days_since_epoch = secs / 86400;
            let time_of_day = secs % 86400;
            let hours = time_of_day / 3600;
            let minutes = (time_of_day % 3600) / 60;
            let seconds = time_of_day % 60;

            // Calculate year, month, day from days since 1970-01-01
            let (year, month, day) = days_to_ymd(days_since_epoch);
            format!(
                "{:02}/{:02}/{} {:02}:{:02}:{:02} UTC",
                day, month, year, hours, minutes, seconds
            )
        })
        .unwrap_or_else(|| ts.to_string())
}

/// Convert days since epoch to year, month, day
fn days_to_ymd(days: i64) -> (i64, u32, u32) {
    // Simplified algorithm for dates after 1970
    let mut remaining = days;
    let mut year = 1970i64;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let days_in_months: [i64; 12] = if leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &days_in_month in &days_in_months {
        if remaining < days_in_month {
            break;
        }
        remaining -= days_in_month;
        month += 1;
    }

    (year, month, (remaining + 1) as u32)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}
