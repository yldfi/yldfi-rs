//! Contract-related commands
//!
//! Fetch ABI, source code, and creation info for contracts

use super::OutputFormat;
use crate::config::{Chain, ConfigFile, EndpointConfig};
use crate::etherscan::{Client, SignatureCache};
use crate::rpc::Endpoint;
use crate::utils::format::with_thousands_sep;
use alloy::dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt};
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use clap::Subcommand;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::sync::OnceLock;

/// Get EIP-1967 implementation slot (parsed once)
fn eip1967_impl_slot() -> B256 {
    static SLOT: OnceLock<B256> = OnceLock::new();
    *SLOT.get_or_init(|| {
        "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc"
            .parse()
            .expect("valid EIP-1967 implementation slot")
    })
}

/// Get EIP-1967 beacon slot (parsed once)
fn eip1967_beacon_slot() -> B256 {
    static SLOT: OnceLock<B256> = OnceLock::new();
    *SLOT.get_or_init(|| {
        "0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50"
            .parse()
            .expect("valid EIP-1967 beacon slot")
    })
}

/// Get OpenZeppelin AdminUpgradeabilityProxy slot (parsed once)
fn oz_impl_slot() -> B256 {
    static SLOT: OnceLock<B256> = OnceLock::new();
    *SLOT.get_or_init(|| {
        "0x7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3"
            .parse()
            .expect("valid OpenZeppelin implementation slot")
    })
}

/// Try to detect if a contract is a proxy and return the implementation address
async fn detect_proxy_implementation<P: Provider>(
    provider: &P,
    address: Address,
) -> Option<Address> {
    // Try EIP-1967 implementation slot first
    if let Ok(storage) = provider
        .get_storage_at(address, eip1967_impl_slot().into())
        .await
    {
        let impl_addr = Address::from_slice(&storage.to_be_bytes::<32>()[12..]);
        if !impl_addr.is_zero() {
            return Some(impl_addr);
        }
    }

    // Try OpenZeppelin AdminUpgradeabilityProxy slot (used by USDC, etc.)
    if let Ok(storage) = provider
        .get_storage_at(address, oz_impl_slot().into())
        .await
    {
        let impl_addr = Address::from_slice(&storage.to_be_bytes::<32>()[12..]);
        if !impl_addr.is_zero() {
            return Some(impl_addr);
        }
    }

    // Try EIP-1967 beacon slot
    if let Ok(storage) = provider
        .get_storage_at(address, eip1967_beacon_slot().into())
        .await
    {
        let beacon_addr = Address::from_slice(&storage.to_be_bytes::<32>()[12..]);
        if !beacon_addr.is_zero() {
            // Call beacon.implementation() to get the actual implementation
            // implementation() selector = 0x5c60da1b
            let calldata = hex::decode("5c60da1b").ok()?;
            let tx = alloy::rpc::types::TransactionRequest::default()
                .to(beacon_addr)
                .input(calldata.into());
            if let Ok(result) = provider.call(tx).await {
                if result.len() >= 32 {
                    let impl_addr = Address::from_slice(&result[12..32]);
                    if !impl_addr.is_zero() {
                        return Some(impl_addr);
                    }
                }
            }
        }
    }

    None
}

#[derive(Subcommand)]
pub enum ContractCommands {
    /// Get verified contract ABI
    #[command(visible_alias = "abi")]
    Abi {
        /// Contract address
        address: String,

        /// Save to file instead of stdout
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Get verified source code
    #[command(visible_alias = "src")]
    Source {
        /// Contract address
        address: String,

        /// Save to directory instead of stdout
        #[arg(long, short)]
        output: Option<PathBuf>,
    },

    /// Get contract creation info (deployer, tx hash)
    #[command(visible_alias = "info")]
    Creation {
        /// Contract address
        address: String,

        /// Output format (json, table/pretty)
        #[arg(long, short = 'f', value_enum, default_value = "table")]
        format: OutputFormat,
    },

    /// Call a contract function (auto-fetches ABI)
    ///
    /// Examples:
    ///   ethcli contract call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 totalSupply
    ///   ethcli contract call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 balanceOf 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
    ///   ethcli contract call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 totalSupply --human
    Call {
        /// Contract address
        address: String,

        /// Function name (e.g., "totalSupply", "balanceOf")
        function: String,

        /// Function arguments
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest")]
        block: String,

        /// Custom RPC URL (overrides config)
        #[arg(long)]
        rpc_url: Option<String>,

        /// Format output for human readability (commas in numbers, token decimals)
        #[arg(long, short = 'H')]
        human: bool,
    },
}

pub async fn handle(
    action: &ContractCommands,
    chain: Chain,
    api_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = Client::new(chain, api_key)?;

    match action {
        ContractCommands::Abi { address, output } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let cache = Arc::new(SignatureCache::new());
            let chain_id = chain.chain_id();

            // Check cache first
            let json = if let Some((cached_abi, _)) = cache.get_abi(chain_id, address) {
                if !quiet {
                    eprintln!("Using cached ABI for {}...", address);
                }
                cached_abi
            } else {
                if !quiet {
                    eprintln!("Fetching ABI for {}...", address);
                }

                let abi = client
                    .contract_abi(addr)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to fetch ABI: {}", e))?;

                let json = serde_json::to_string_pretty(&abi)?;

                // Cache the ABI
                cache.set_abi(chain_id, address, &json, None);

                json
            };

            if let Some(path) = output {
                std::fs::write(path, &json)?;
                if !quiet {
                    eprintln!("ABI saved to {}", path.display());
                }
            } else {
                println!("{}", json);
            }
        }

        ContractCommands::Source { address, output } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching source code for {}...", address);
            }

            let metadata = client
                .contract_source_code(addr)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch source: {}", e))?;

            if let Some(dir) = output {
                // Create directory and save files
                std::fs::create_dir_all(dir)?;

                // Canonicalize directory AFTER creation (so it exists and can be resolved)
                // This is critical for security - fail hard if we can't canonicalize
                let canonical_dir = dir.canonicalize().map_err(|e| {
                    anyhow::anyhow!("Failed to canonicalize output directory: {}", e)
                })?;

                // Get source items
                let items = metadata.items;
                if items.is_empty() {
                    return Err(anyhow::anyhow!(
                        "No source code found (contract may not be verified)"
                    ));
                }

                for item in &items {
                    // Sanitize contract name to prevent path traversal attacks
                    // Use ASCII only to ensure 1 byte per char (251 + ".sol" = 255 bytes max)
                    let safe_name: String = item
                        .contract_name
                        .chars()
                        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
                        .take(251) // Max 251 bytes + ".sol" = 255 (filesystem limit)
                        .collect();

                    if safe_name.is_empty() {
                        eprintln!(
                            "  Warning: Skipping contract with unsafe name: {}",
                            item.contract_name
                        );
                        continue;
                    }

                    // Check for Windows reserved filenames
                    const RESERVED_NAMES: &[&str] = &[
                        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6",
                        "COM7", "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6",
                        "LPT7", "LPT8", "LPT9",
                    ];
                    if RESERVED_NAMES.contains(&safe_name.to_uppercase().as_str()) {
                        eprintln!(
                            "  Warning: Skipping contract with reserved name: {}",
                            item.contract_name
                        );
                        continue;
                    }

                    // Build path from canonicalized directory
                    let filename = format!("{}.sol", safe_name);
                    let file_path = canonical_dir.join(&filename);

                    // Verify the constructed path is still within the target directory
                    // Note: We check the constructed path, not canonicalize it (file doesn't exist yet)
                    // The sanitization above should prevent ".." but this is defense in depth
                    if !file_path.starts_with(&canonical_dir) {
                        eprintln!(
                            "  Warning: Skipping file that would escape directory: {}",
                            item.contract_name
                        );
                        continue;
                    }

                    let source_code_str = item.source_code.source_code();
                    std::fs::write(&file_path, &source_code_str)?;
                    if !quiet {
                        eprintln!("  Saved: {}", file_path.display());
                    }
                }

                if !quiet {
                    eprintln!("Source code saved to {}", dir.display());
                }
            } else {
                // Print to stdout
                let items = metadata.items;
                if items.is_empty() {
                    return Err(anyhow::anyhow!(
                        "No source code found (contract may not be verified)"
                    ));
                }

                for item in items {
                    println!("// Contract: {}", item.contract_name);
                    println!("// Compiler: {}", item.compiler_version);
                    println!(
                        "// Optimization: {} (runs: {})",
                        if item.optimization_used == 1 {
                            "enabled"
                        } else {
                            "disabled"
                        },
                        item.runs
                    );
                    println!("\n{}", item.source_code.source_code());
                }
            }
        }

        ContractCommands::Creation { address, format } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching creation info for {}...", address);
            }

            let creation = client
                .contract_creation_data(addr)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch creation data: {}", e))?;

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&creation)?);
            } else {
                println!("Contract Creation Info");
                println!("{}", "─".repeat(50));
                println!("Contract:   {:#x}", creation.contract_address);
                println!("Creator:    {:#x}", creation.contract_creator);
                println!("Tx Hash:    {:#x}", creation.transaction_hash);

                // Add explorer link
                if let Some(explorer) = chain.explorer_url() {
                    println!(
                        "\nExplorer:   {}/tx/{:#x}",
                        explorer, creation.transaction_hash
                    );
                }
            }
        }

        ContractCommands::Call {
            address,
            function,
            args,
            block,
            rpc_url,
            human,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            // Get RPC endpoint first (needed for proxy detection)
            let endpoint = if let Some(url) = rpc_url {
                Endpoint::new(EndpointConfig::new(url.clone()), 30, None)?
            } else {
                let config = ConfigFile::load_default()
                    .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
                    .unwrap_or_default();

                let chain_endpoints: Vec<_> = config
                    .endpoints
                    .into_iter()
                    .filter(|e| e.enabled && e.chain == chain)
                    .collect();

                if chain_endpoints.is_empty() {
                    return Err(anyhow::anyhow!(
                        "No RPC endpoints configured for {}. Add one with: ethcli endpoints add <url>",
                        chain.display_name()
                    ));
                }
                Endpoint::new(chain_endpoints[0].clone(), 30, None)?
            };

            let provider = endpoint.provider();

            // Check if this is a proxy contract
            let abi_address =
                if let Some(impl_addr) = detect_proxy_implementation(provider, addr).await {
                    if !quiet {
                        eprintln!(
                            "Detected proxy contract, fetching implementation ABI from {:#x}...",
                            impl_addr
                        );
                    }
                    impl_addr
                } else {
                    if !quiet {
                        eprintln!("Fetching ABI for {}...", address);
                    }
                    addr
                };

            // Fetch ABI (from implementation address if proxy)
            let abi = client
                .contract_abi(abi_address)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch ABI: {}", e))?;

            // contract_abi returns JsonAbi directly
            let json_abi = abi;

            // Find the function - handle overloaded functions by matching arg count and types
            let funcs = json_abi
                .functions
                .get(function)
                .ok_or_else(|| anyhow::anyhow!("Function '{}' not found in ABI", function))?;

            // Helper to format function signature for error messages
            let format_sig = |f: &alloy::json_abi::Function| {
                format!(
                    "{}({})",
                    function,
                    f.inputs
                        .iter()
                        .map(|i| i.ty.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            };

            // Helper to try coercing arguments for a function
            let try_coerce = |f: &alloy::json_abi::Function| -> Option<Vec<DynSolValue>> {
                if f.inputs.len() != args.len() {
                    return None;
                }
                let mut values = Vec::new();
                for (input, arg) in f.inputs.iter().zip(args.iter()) {
                    let ty = DynSolType::parse(&input.ty.to_string()).ok()?;
                    let val = ty.coerce_str(arg).ok()?;
                    values.push(val);
                }
                Some(values)
            };

            // Try to find a matching overload
            let (func, values) = if funcs.len() == 1 {
                // Single function - try to coerce and give detailed error if it fails
                let f = &funcs[0];
                if f.inputs.len() != args.len() {
                    return Err(anyhow::anyhow!(
                        "Function '{}' expects {} arguments, got {}",
                        function,
                        f.inputs.len(),
                        args.len()
                    ));
                }
                let mut vals = Vec::new();
                for (input, arg) in f.inputs.iter().zip(args.iter()) {
                    let ty = DynSolType::parse(&input.ty.to_string())
                        .map_err(|e| anyhow::anyhow!("Invalid type '{}': {}", input.ty, e))?;
                    let val = ty.coerce_str(arg).map_err(|e| {
                        anyhow::anyhow!("Invalid value '{}' for type '{}': {}", arg, input.ty, e)
                    })?;
                    vals.push(val);
                }
                (f, vals)
            } else {
                // Multiple overloads - try each one and find matches
                let matches: Vec<_> = funcs
                    .iter()
                    .filter_map(|f| try_coerce(f).map(|v| (f, v)))
                    .collect();

                match matches.len() {
                    0 => {
                        // No matches - show all overloads
                        let overloads: Vec<String> = funcs.iter().map(format_sig).collect();
                        return Err(anyhow::anyhow!(
                            "Function '{}' has {} overloads, none match the provided arguments:\n  {}\n\nProvided: {} args [{}]",
                            function,
                            funcs.len(),
                            overloads.join("\n  "),
                            args.len(),
                            args.join(", ")
                        ));
                    }
                    1 => matches.into_iter().next().unwrap(),
                    _ => {
                        // Multiple matches - ambiguous
                        let matching_sigs: Vec<String> =
                            matches.iter().map(|(f, _)| format_sig(f)).collect();
                        return Err(anyhow::anyhow!(
                            "Ambiguous call: {} overloads match the provided arguments:\n  {}\n\nUse explicit types or specify the full signature.",
                            matches.len(),
                            matching_sigs.join("\n  ")
                        ));
                    }
                }
            };

            // Encode the call
            let calldata = func
                .abi_encode_input(&values)
                .map_err(|e| anyhow::anyhow!("Failed to encode arguments: {}", e))?;

            if !quiet {
                eprintln!("Calling {}({})...", function, args.join(", "));
            }

            // Parse block
            let block_id = super::rpc::parse_block_id(block)?;

            // Make the call
            let tx = alloy::rpc::types::TransactionRequest::default()
                .to(addr)
                .input(calldata.into());

            let result = provider
                .call(tx)
                .block(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Call failed: {}", e))?;

            // Decode the result
            if func.outputs.is_empty() {
                println!("(no return value)");
            } else {
                let decoded = func
                    .abi_decode_output(&result)
                    .map_err(|e| anyhow::anyhow!("Failed to decode output: {}", e))?;

                // For human mode, try to get decimals if this looks like a token
                let token_decimals = if *human {
                    // Try to detect if this is an ERC20 by calling decimals()
                    get_token_decimals(provider, addr).await
                } else {
                    None
                };

                // Format output nicely
                if decoded.len() == 1 {
                    if *human {
                        println!("{}", format_value_human(&decoded[0], token_decimals));
                    } else {
                        println!("{}", format_value(&decoded[0]));
                    }
                } else {
                    for (i, (output, value)) in func.outputs.iter().zip(decoded.iter()).enumerate()
                    {
                        let name = if output.name.is_empty() {
                            format!("[{}]", i)
                        } else {
                            output.name.clone()
                        };
                        if *human {
                            println!("{}: {}", name, format_value_human(value, token_decimals));
                        } else {
                            println!("{}: {}", name, format_value(value));
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// Try to get token decimals by calling decimals() on the contract
async fn get_token_decimals<P: Provider>(provider: &P, address: Address) -> Option<u8> {
    // decimals() selector = 0x313ce567
    let calldata = hex::decode("313ce567").ok()?;
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(address)
        .input(calldata.into());

    let result = provider.call(tx).await.ok()?;
    if result.len() >= 32 {
        // Last byte of a uint8 return value
        Some(result[31])
    } else {
        None
    }
}

/// Format a DynSolValue for display
fn format_value(value: &DynSolValue) -> String {
    format_value_internal(value, false, None)
}

/// Format a DynSolValue with human-readable formatting
fn format_value_human(value: &DynSolValue, decimals: Option<u8>) -> String {
    format_value_internal(value, true, decimals)
}

/// Internal formatting function
fn format_value_internal(value: &DynSolValue, human: bool, decimals: Option<u8>) -> String {
    match value {
        DynSolValue::Bool(b) => b.to_string(),
        DynSolValue::Int(i, _) => {
            if human {
                with_thousands_sep(&i.to_string())
            } else {
                i.to_string()
            }
        }
        DynSolValue::Uint(u, bits) => {
            if human {
                // Try to format with decimals if provided
                if let Some(dec) = decimals {
                    format_with_decimals(u, dec)
                } else if *bits <= 64 {
                    // Small numbers - use commas
                    with_thousands_sep(&u.to_string())
                } else {
                    // Large numbers (likely token amounts) - show with commas
                    with_thousands_sep(&u.to_string())
                }
            } else {
                u.to_string()
            }
        }
        DynSolValue::FixedBytes(b, _) => format!("0x{}", hex::encode(b)),
        DynSolValue::Address(a) => a.to_checksum(None),
        DynSolValue::Function(f) => format!("0x{}", hex::encode(f)),
        DynSolValue::Bytes(b) => format!("0x{}", hex::encode(b)),
        DynSolValue::String(s) => format!("\"{}\"", s),
        DynSolValue::Array(arr) => {
            let items: Vec<String> = arr
                .iter()
                .map(|v| format_value_internal(v, human, decimals))
                .collect();
            format!("[{}]", items.join(", "))
        }
        DynSolValue::FixedArray(arr) => {
            let items: Vec<String> = arr
                .iter()
                .map(|v| format_value_internal(v, human, decimals))
                .collect();
            format!("[{}]", items.join(", "))
        }
        DynSolValue::Tuple(tuple) => {
            let items: Vec<String> = tuple
                .iter()
                .map(|v| format_value_internal(v, human, decimals))
                .collect();
            format!("({})", items.join(", "))
        }
    }
}

/// Format a U256 with decimal places
fn format_with_decimals(value: &alloy::primitives::U256, decimals: u8) -> String {
    let s = value.to_string();
    let dec = decimals as usize;

    if dec == 0 {
        return with_thousands_sep(&s);
    }

    // Pad with leading zeros if needed
    let padded = if s.len() <= dec {
        format!("{:0>width$}", s, width = dec + 1)
    } else {
        s
    };

    let (integer, fraction) = padded.split_at(padded.len() - dec);
    let fraction_trimmed = fraction.trim_end_matches('0');

    if fraction_trimmed.is_empty() {
        with_thousands_sep(integer)
    } else {
        format!("{}.{}", with_thousands_sep(integer), fraction_trimmed)
    }
}
