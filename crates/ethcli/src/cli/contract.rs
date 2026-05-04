//! Contract-related commands
//!
//! Fetch ABI, source code, creation info, and bytecode analysis for contracts

use super::OutputFormat;
use crate::bytecode::{
    analyze_bytecode, analyze_handler_checks, disassemble_bytecode, extract_selectors,
    infer_dispatcher, opcode_stats, BytecodeAnalysis, RiskLevel, MAX_BYTECODE_SIZE,
};
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct FunctionQuery {
    name: String,
    explicit_types: Option<Vec<String>>,
    original: String,
}

fn split_type_list(types_str: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;

    for (i, c) in types_str.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => {
                let ty = types_str[start..i].trim();
                if !ty.is_empty() {
                    result.push(ty);
                }
                start = i + 1;
            }
            _ => {}
        }
    }

    let tail = types_str[start..].trim();
    if !tail.is_empty() {
        result.push(tail);
    }

    result
}

fn canonicalize_type(type_str: &str) -> anyhow::Result<String> {
    DynSolType::parse(type_str.trim())
        .map(|ty| ty.to_string())
        .map_err(|e| anyhow::anyhow!("Invalid type '{}': {}", type_str, e))
}

fn parse_function_query(function: &str) -> anyhow::Result<FunctionQuery> {
    let original = function.trim().to_string();
    if original.is_empty() {
        return Err(anyhow::anyhow!("Function name cannot be empty"));
    }

    let Some(types_start) = original.find('(') else {
        return Ok(FunctionQuery {
            name: original.clone(),
            explicit_types: None,
            original,
        });
    };

    let types_end = original
        .rfind(')')
        .ok_or_else(|| anyhow::anyhow!("Invalid function signature '{}': missing ')'", original))?;
    if types_end != original.len() - 1 {
        return Err(anyhow::anyhow!(
            "Invalid function signature '{}': unexpected trailing characters",
            original
        ));
    }

    let name = original[..types_start].trim();
    if name.is_empty() {
        return Err(anyhow::anyhow!(
            "Invalid function signature '{}': missing function name",
            original
        ));
    }

    let raw_types = &original[types_start + 1..types_end];
    let explicit_types = if raw_types.trim().is_empty() {
        Vec::new()
    } else {
        split_type_list(raw_types)
            .into_iter()
            .map(canonicalize_type)
            .collect::<anyhow::Result<Vec<_>>>()?
    };

    Ok(FunctionQuery {
        name: name.to_string(),
        explicit_types: Some(explicit_types),
        original,
    })
}

fn format_contract_signature(name: &str, function: &alloy::json_abi::Function) -> String {
    format!(
        "{}({})",
        name,
        function
            .inputs
            .iter()
            .map(|input| input.ty.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
}

fn coerce_call_args(
    function_name: &str,
    function: &alloy::json_abi::Function,
    args: &[String],
) -> anyhow::Result<Vec<DynSolValue>> {
    if function.inputs.len() != args.len() {
        return Err(anyhow::anyhow!(
            "Function '{}' expects {} arguments, got {}",
            function_name,
            function.inputs.len(),
            args.len()
        ));
    }

    function
        .inputs
        .iter()
        .zip(args.iter())
        .map(|(input, arg)| {
            let ty = DynSolType::parse(&input.ty.to_string())
                .map_err(|e| anyhow::anyhow!("Invalid type '{}': {}", input.ty, e))?;
            ty.coerce_str(arg).map_err(|e| {
                anyhow::anyhow!("Invalid value '{}' for type '{}': {}", arg, input.ty, e)
            })
        })
        .collect()
}

fn select_contract_function<'a>(
    query: &FunctionQuery,
    funcs: &'a [alloy::json_abi::Function],
    args: &[String],
) -> anyhow::Result<(&'a alloy::json_abi::Function, Vec<DynSolValue>)> {
    if let Some(explicit_types) = &query.explicit_types {
        let matching_funcs: Vec<_> = funcs
            .iter()
            .filter(|function| {
                function.inputs.len() == explicit_types.len()
                    && function
                        .inputs
                        .iter()
                        .map(|input| input.ty.to_string())
                        .eq(explicit_types.iter().cloned())
            })
            .collect();

        return match matching_funcs.as_slice() {
            [] => {
                let overloads: Vec<String> = funcs
                    .iter()
                    .map(|function| format_contract_signature(&query.name, function))
                    .collect();
                Err(anyhow::anyhow!(
                    "Function '{}' not found in ABI.\nAvailable overloads:\n  {}",
                    query.original,
                    overloads.join("\n  ")
                ))
            }
            [function] => Ok((function, coerce_call_args(&query.name, function, args)?)),
            _ => {
                let overloads: Vec<String> = matching_funcs
                    .iter()
                    .map(|function| format_contract_signature(&query.name, function))
                    .collect();
                Err(anyhow::anyhow!(
                    "Ambiguous explicit signature '{}': multiple ABI entries match:\n  {}",
                    query.original,
                    overloads.join("\n  ")
                ))
            }
        };
    }

    let try_coerce = |function: &'a alloy::json_abi::Function| -> Option<Vec<DynSolValue>> {
        if function.inputs.len() != args.len() {
            return None;
        }

        let mut values = Vec::new();
        for (input, arg) in function.inputs.iter().zip(args.iter()) {
            let ty = DynSolType::parse(&input.ty.to_string()).ok()?;
            let val = ty.coerce_str(arg).ok()?;
            values.push(val);
        }
        Some(values)
    };

    if funcs.len() == 1 {
        let function = &funcs[0];
        return Ok((function, coerce_call_args(&query.name, function, args)?));
    }

    let matches: Vec<_> = funcs
        .iter()
        .filter_map(|function| try_coerce(function).map(|values| (function, values)))
        .collect();

    match matches.len() {
        0 => {
            let overloads: Vec<String> = funcs
                .iter()
                .map(|function| format_contract_signature(&query.name, function))
                .collect();
            Err(anyhow::anyhow!(
                "Function '{}' has {} overloads, none match the provided arguments:\n  {}\n\nProvided: {} args [{}]",
                query.name,
                funcs.len(),
                overloads.join("\n  "),
                args.len(),
                args.join(", ")
            ))
        }
        1 => Ok(matches.into_iter().next().unwrap()),
        _ => {
            let matching_sigs: Vec<String> = matches
                .iter()
                .map(|(function, _)| format_contract_signature(&query.name, function))
                .collect();
            Err(anyhow::anyhow!(
                "Ambiguous call: {} overloads match the provided arguments:\n  {}\n\nUse an explicit signature like {}.",
                matches.len(),
                matching_sigs.join("\n  "),
                matching_sigs[0]
            ))
        }
    }
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

    // Some legacy proxies expose implementation() directly instead of using
    // standardized EIP-1967 storage slots.
    if let Some(impl_addr) = call_implementation_function(provider, address).await {
        return Some(impl_addr);
    }

    None
}

async fn call_implementation_function<P: Provider>(
    provider: &P,
    address: Address,
) -> Option<Address> {
    // implementation() selector = 0x5c60da1b
    let calldata = hex::decode("5c60da1b").ok()?;
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(address)
        .input(calldata.into());

    let result = provider.call(tx).await.ok()?;
    if result.len() >= 32 {
        let impl_addr = Address::from_slice(&result[result.len() - 20..]);
        if !impl_addr.is_zero() {
            return Some(impl_addr);
        }
    }

    None
}

#[derive(Subcommand)]
pub enum ContractCommands {
    /// Get verified contract ABI
    Abi {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Save to file instead of stdout
        #[arg(long, short, value_name = "FILE")]
        output: Option<PathBuf>,
    },

    /// Get verified source code
    #[command(visible_alias = "src")]
    Source {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Save to directory instead of stdout
        #[arg(long, short, value_name = "DIR")]
        output: Option<PathBuf>,
    },

    /// Get contract creation info (deployer, tx hash)
    #[command(visible_alias = "info")]
    Creation {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "table"
        )]
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
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Function name (e.g., "totalSupply", "balanceOf")
        #[arg(value_name = "FUNCTION")]
        function: String,

        /// Function arguments
        #[arg(trailing_var_arg = true, value_name = "ARG")]
        args: Vec<String>,

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
        block: String,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

        /// Format output for human readability (commas in numbers, token decimals)
        #[arg(long, short = 'H')]
        human: bool,
    },

    /// Extract function selectors from bytecode (uses evmole)
    ///
    /// Extracts function selectors, arguments, and state mutability
    /// from contract bytecode without needing source code or ABI.
    ///
    /// Examples:
    ///   ethcli contract selectors 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    ///   ethcli contract sel 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --lookup
    #[command(visible_alias = "sel")]
    Selectors {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Lookup function signatures from 4byte.directory
        #[arg(long, short)]
        lookup: bool,

        /// If proxy detected, extract selectors from the implementation contract instead
        #[arg(long)]
        follow_proxy: bool,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "table"
        )]
        format: OutputFormat,
    },

    /// Disassemble contract bytecode into opcodes
    ///
    /// Shows the raw EVM opcodes in the contract bytecode.
    ///
    /// Examples:
    ///   ethcli contract disassemble 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    ///   ethcli contract dis 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --limit 50
    #[command(visible_alias = "dis")]
    Disassemble {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Maximum number of opcodes to show (default: unlimited)
        #[arg(long, short, value_name = "N")]
        limit: Option<usize>,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "table"
        )]
        format: OutputFormat,
    },

    /// Show opcode frequency statistics
    ///
    /// Analyzes bytecode and shows frequency of each opcode type.
    ///
    /// Examples:
    ///   ethcli contract opcodes 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    ///   ethcli contract ops 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    #[command(visible_alias = "ops")]
    Opcodes {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "table"
        )]
        format: OutputFormat,
    },

    /// Comprehensive bytecode security analysis
    ///
    /// Combines function selector extraction, security pattern detection,
    /// and opcode analysis to identify potential risks in contract bytecode.
    ///
    /// Detects dangerous patterns like:
    ///   - SELFDESTRUCT (contract can be destroyed)
    ///   - DELEGATECALL (arbitrary code execution)
    ///   - ORIGIN (tx.origin auth, honeypot indicator)
    ///   - CREATE/CREATE2 (dynamic contract creation)
    ///
    /// Examples:
    ///   ethcli contract analyze 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    ///   ethcli contract az 0x... --include-disassembly --limit 100
    #[command(visible_aliases = ["az", "an"])]
    Analyze {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Include full opcode disassembly in output
        #[arg(long)]
        include_disassembly: bool,

        /// Limit number of opcodes in disassembly (requires --include-disassembly)
        #[arg(long, value_name = "N", requires = "include_disassembly")]
        limit: Option<usize>,

        /// Lookup function signatures from 4byte.directory
        #[arg(long, short)]
        lookup: bool,

        /// If proxy detected, analyze the implementation contract instead
        #[arg(long)]
        follow_proxy: bool,

        /// Include selector -> handler offset dispatcher mapping
        #[arg(long)]
        dispatcher: bool,

        /// Include handler guard/check heuristics
        #[arg(long)]
        checks: bool,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

        /// Output format (json, table/pretty)
        #[arg(
            long,
            short = 'o',
            visible_alias = "output",
            value_enum,
            default_value = "table"
        )]
        format: OutputFormat,
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

            let query = parse_function_query(function)?;

            // Find the function - handle overloaded functions by matching arg count and types
            let funcs = json_abi
                .functions
                .get(query.name.as_str())
                .ok_or_else(|| anyhow::anyhow!("Function '{}' not found in ABI", query.original))?;

            let (func, values) = select_contract_function(&query, funcs, args)?;

            // Encode the call
            let calldata = func
                .abi_encode_input(&values)
                .map_err(|e| anyhow::anyhow!("Failed to encode arguments: {}", e))?;

            if !quiet {
                eprintln!("Calling {}({})...", query.name, args.join(", "));
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

        ContractCommands::Selectors {
            address,
            lookup,
            follow_proxy,
            rpc_url,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching bytecode for {}...", address);
            }

            let bytecode = get_bytecode(&chain, rpc_url.as_deref(), addr).await?;

            if bytecode.is_empty() {
                return Err(anyhow::anyhow!(
                    "No bytecode found at {} (not a contract or empty)",
                    address
                ));
            }

            let proxy_info = crate::bytecode::detect_proxy(&bytecode);
            let (selector_address, selector_bytecode) = if *follow_proxy {
                let impl_addr = if let Some(impl_addr) = proxy_info.implementation {
                    Some(impl_addr)
                } else {
                    get_implementation_from_storage(&chain, rpc_url.as_deref(), addr).await
                };

                if let Some(impl_addr) = impl_addr {
                    if !quiet {
                        eprintln!("Following proxy to implementation: {:#x}", impl_addr);
                    }
                    let impl_bytecode = get_bytecode(&chain, rpc_url.as_deref(), impl_addr).await?;
                    if impl_bytecode.is_empty() {
                        if !quiet {
                            eprintln!(
                                "Warning: Implementation at {:#x} has no bytecode; using proxy bytecode",
                                impl_addr
                            );
                        }
                        (addr, bytecode)
                    } else {
                        (impl_addr, impl_bytecode)
                    }
                } else {
                    if !quiet {
                        eprintln!(
                            "Warning: Proxy detected but implementation address could not be resolved"
                        );
                    }
                    (addr, bytecode)
                }
            } else {
                (addr, bytecode)
            };

            let mut functions = extract_selectors(&selector_bytecode);

            // Optionally lookup signatures from 4byte.directory
            if *lookup {
                if !quiet {
                    eprintln!("Looking up function signatures from 4byte.directory...");
                }
                // Use Client to fetch from 4byte.directory (with cache fallback)
                for func in &mut functions {
                    // Strip 0x prefix for lookup
                    let selector = func.selector.trim_start_matches("0x");
                    if let Some(sig) = client.lookup_selector(selector).await {
                        func.signature = Some(sig);
                    }
                }
            }

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&functions)?);
            } else {
                println!("Function Selectors: {:#x}", selector_address);
                println!("{}", "═".repeat(70));
                println!();
                println!(
                    "{:<12} {:<40} {:<12}",
                    "Selector", "Signature/Arguments", "Mutability"
                );
                println!("{}", "─".repeat(70));

                for func in &functions {
                    let sig_or_args = func.signature.as_ref().cloned().unwrap_or_else(|| {
                        if func.arguments.is_empty() {
                            "()".to_string()
                        } else {
                            format!("({})", func.arguments.join(", "))
                        }
                    });
                    println!(
                        "{:<12} {:<40} {:<12}",
                        func.selector, sig_or_args, func.state_mutability
                    );
                }

                println!();
                println!("Total: {} functions", functions.len());
            }
        }

        ContractCommands::Disassemble {
            address,
            limit,
            rpc_url,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching bytecode for {}...", address);
            }

            let bytecode = get_bytecode(&chain, rpc_url.as_deref(), addr).await?;

            if bytecode.is_empty() {
                return Err(anyhow::anyhow!(
                    "No bytecode found at {} (not a contract or empty)",
                    address
                ));
            }

            // Check bytecode size
            if bytecode.len() > MAX_BYTECODE_SIZE {
                return Err(anyhow::anyhow!(
                    "Bytecode too large: {} bytes (max: {} bytes)",
                    bytecode.len(),
                    MAX_BYTECODE_SIZE
                ));
            }

            let mut ops = disassemble_bytecode(&bytecode)
                .map_err(|e| anyhow::anyhow!("Failed to disassemble bytecode: {}", e))?;
            if let Some(n) = limit {
                ops.truncate(*n);
            }

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&ops)?);
            } else {
                println!("Disassembly: {}", address);
                println!("{}", "═".repeat(60));
                println!();
                println!("{:<10} {:<15} Operand", "Offset", "Opcode");
                println!("{}", "─".repeat(60));

                for op in &ops {
                    let operand = op.operand.as_deref().unwrap_or("");
                    println!("{:08x}   {:<15} {}", op.offset, op.opcode, operand);
                }

                println!();
                println!("Total: {} opcodes shown", ops.len());
                if limit.is_some() {
                    println!("(limited output, bytecode size: {} bytes)", bytecode.len());
                }
            }
        }

        ContractCommands::Opcodes {
            address,
            rpc_url,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching bytecode for {}...", address);
            }

            let bytecode = get_bytecode(&chain, rpc_url.as_deref(), addr).await?;

            if bytecode.is_empty() {
                return Err(anyhow::anyhow!(
                    "No bytecode found at {} (not a contract or empty)",
                    address
                ));
            }

            // Check bytecode size
            if bytecode.len() > MAX_BYTECODE_SIZE {
                return Err(anyhow::anyhow!(
                    "Bytecode too large: {} bytes (max: {} bytes)",
                    bytecode.len(),
                    MAX_BYTECODE_SIZE
                ));
            }

            let stats = opcode_stats(&bytecode)
                .map_err(|e| anyhow::anyhow!("Failed to analyze opcodes: {}", e))?;

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                println!("Opcode Statistics: {}", address);
                println!("{}", "═".repeat(50));
                println!();
                println!(
                    "Bytecode Size:    {} bytes",
                    with_thousands_sep(&stats.bytecode_size.to_string())
                );
                println!(
                    "Total Opcodes:    {}",
                    with_thousands_sep(&stats.total_opcodes.to_string())
                );
                println!();
                println!("Category Summary");
                println!("{}", "─".repeat(50));
                println!("  PUSH operations:    {}", stats.push_count);
                println!("  JUMP operations:    {}", stats.jump_count);
                println!("  CALL operations:    {}", stats.call_count);
                println!("  Storage ops:        {}", stats.storage_count);
                println!();
                println!("Top Opcodes by Frequency");
                println!("{}", "─".repeat(50));

                // Sort by frequency descending
                let mut freq_vec: Vec<_> = stats.frequencies.iter().collect();
                freq_vec.sort_by(|a, b| b.1.cmp(a.1));

                for (opcode, count) in freq_vec.iter().take(15) {
                    // Guard against division by zero
                    let pct = if stats.total_opcodes > 0 {
                        **count as f64 / stats.total_opcodes as f64 * 100.0
                    } else {
                        0.0
                    };
                    println!("  {:<15} {:>6} ({:>5.1}%)", opcode, count, pct);
                }

                if freq_vec.len() > 15 {
                    println!("  ... and {} more", freq_vec.len() - 15);
                }
            }
        }

        ContractCommands::Analyze {
            address,
            include_disassembly,
            limit,
            lookup,
            follow_proxy,
            dispatcher,
            checks,
            rpc_url,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching bytecode for {}...", address);
            }

            let bytecode = get_bytecode(&chain, rpc_url.as_deref(), addr).await?;

            if bytecode.is_empty() {
                return Err(anyhow::anyhow!(
                    "No bytecode found at {} (not a contract or empty)",
                    address
                ));
            }

            // Check for proxy patterns
            let proxy_info = crate::bytecode::detect_proxy(&bytecode);
            let (analysis_address, analysis_bytecode, proxy_detected) = if proxy_info.is_proxy {
                if !quiet {
                    if let Some(ref proxy_type) = proxy_info.proxy_type {
                        eprintln!("Detected: {} contract", proxy_type.name());
                    }
                }

                if *follow_proxy {
                    // Try to get implementation address
                    let impl_addr = if let Some(impl_addr) = proxy_info.implementation {
                        // EIP-1167: implementation embedded in bytecode
                        Some(impl_addr)
                    } else {
                        // Storage-based proxy: try EIP-1967 slot
                        get_implementation_from_storage(&chain, rpc_url.as_deref(), addr).await
                    };

                    if let Some(impl_addr) = impl_addr {
                        if !quiet {
                            eprintln!("Following proxy to implementation: {:#x}", impl_addr);
                        }
                        let impl_bytecode =
                            get_bytecode(&chain, rpc_url.as_deref(), impl_addr).await?;
                        if impl_bytecode.is_empty() {
                            if !quiet {
                                eprintln!(
                                    "Warning: Implementation at {:#x} has no bytecode",
                                    impl_addr
                                );
                            }
                            (address.clone(), bytecode, Some(proxy_info))
                        } else {
                            (format!("{:#x}", impl_addr), impl_bytecode, Some(proxy_info))
                        }
                    } else {
                        if !quiet {
                            eprintln!(
                                "Warning: Could not determine implementation address, analyzing proxy bytecode"
                            );
                        }
                        (address.clone(), bytecode, Some(proxy_info))
                    }
                } else {
                    if !quiet && proxy_info.implementation.is_some() {
                        eprintln!(
                            "Hint: Use --follow-proxy to analyze the implementation contract"
                        );
                    }
                    (address.clone(), bytecode, Some(proxy_info))
                }
            } else {
                (address.clone(), bytecode, None)
            };

            if !quiet {
                eprintln!("Analyzing bytecode...");
            }

            let mut analysis = analyze_bytecode(
                &analysis_address,
                &analysis_bytecode,
                true, // include stats
                *include_disassembly,
                *limit,
            );

            // Add proxy info to analysis output
            if let Some(ref proxy) = proxy_detected {
                analysis.proxy_info = Some(proxy.clone());
            }

            // Optionally lookup signatures from 4byte.directory
            if *lookup {
                if !quiet {
                    eprintln!("Looking up function signatures from 4byte.directory...");
                }
                // Use Client to fetch from 4byte.directory (with cache fallback)
                for func in &mut analysis.functions {
                    let selector = func.selector.trim_start_matches("0x");
                    if let Some(sig) = client.lookup_selector(selector).await {
                        func.signature = Some(sig);
                    }
                }
            }

            if *dispatcher || *checks {
                if !quiet {
                    eprintln!("Inferring selector dispatcher...");
                }
                let inferred_dispatcher = infer_dispatcher(&analysis_bytecode, &analysis.functions)
                    .map_err(|e| anyhow::anyhow!("Failed to infer dispatcher: {}", e))?;

                if *checks {
                    if !quiet {
                        eprintln!("Analyzing handler guards and checks...");
                    }
                    let check_summary = analyze_handler_checks(
                        &analysis_bytecode,
                        &analysis.functions,
                        &inferred_dispatcher,
                    )
                    .map_err(|e| anyhow::anyhow!("Failed to analyze handler checks: {}", e))?;
                    analysis.checks = Some(check_summary);
                }

                if *dispatcher {
                    analysis.dispatcher = Some(inferred_dispatcher);
                }
            }

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&analysis)?);
            } else {
                print_analysis_table(&analysis);
            }
        }
    }

    Ok(())
}

/// Get bytecode for an address via RPC
async fn get_bytecode(
    chain: &Chain,
    rpc_url: Option<&str>,
    address: Address,
) -> anyhow::Result<Vec<u8>> {
    let endpoint = if let Some(url) = rpc_url {
        Endpoint::new(EndpointConfig::new(url.to_string()), 30, None)?
    } else {
        let config = ConfigFile::load_default()
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
            .unwrap_or_default();

        let chain_endpoints: Vec<_> = config
            .endpoints
            .into_iter()
            .filter(|e| e.enabled && e.chain == *chain)
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
    let code = provider
        .get_code_at(address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch bytecode: {}", e))?;

    Ok(code.to_vec())
}

/// Try to get implementation address from EIP-1967 storage slot
async fn get_implementation_from_storage(
    chain: &Chain,
    rpc_url: Option<&str>,
    proxy_address: Address,
) -> Option<Address> {
    use crate::bytecode::{address_from_storage, proxy_slots, u256_to_b256};

    let endpoint = if let Some(url) = rpc_url {
        Endpoint::new(EndpointConfig::new(url.to_string()), 30, None).ok()?
    } else {
        crate::rpc::get_rpc_endpoint(*chain).ok()?
    };

    let provider = endpoint.provider();

    // Try EIP-1967 implementation slot first
    if let Ok(value) = provider
        .get_storage_at(proxy_address, proxy_slots::EIP1967_IMPLEMENTATION.into())
        .await
    {
        if let Some(addr) = address_from_storage(u256_to_b256(value)) {
            return Some(addr);
        }
    }

    // Try OpenZeppelin legacy slot
    if let Ok(value) = provider
        .get_storage_at(proxy_address, proxy_slots::OZ_LEGACY_IMPLEMENTATION.into())
        .await
    {
        if let Some(addr) = address_from_storage(u256_to_b256(value)) {
            return Some(addr);
        }
    }

    // Try OpenZeppelin AdminUpgradeabilityProxy slot used by older proxies such as USDC.
    if let Ok(value) = provider
        .get_storage_at(proxy_address, oz_impl_slot().into())
        .await
    {
        if let Some(addr) = address_from_storage(u256_to_b256(value)) {
            return Some(addr);
        }
    }

    if let Some(addr) = call_implementation_function(&provider, proxy_address).await {
        return Some(addr);
    }

    None
}

/// Print a nicely formatted analysis table
fn print_analysis_table(analysis: &BytecodeAnalysis) {
    println!();
    println!("Bytecode Analysis: {}", analysis.address);
    println!("{}", "═".repeat(65));
    println!();

    // Contract info
    println!("Contract Info");
    println!("{}", "─".repeat(65));
    println!(
        "  Bytecode Size:  {} bytes",
        with_thousands_sep(&analysis.bytecode_size.to_string())
    );
    println!("  Functions:      {}", analysis.function_count);

    // Proxy info (if detected)
    if let Some(ref proxy) = analysis.proxy_info {
        if proxy.is_proxy {
            if let Some(ref proxy_type) = proxy.proxy_type {
                println!("  Proxy Type:     {}", proxy_type.name());
            }
            if let Some(impl_addr) = proxy.implementation {
                println!("  Implementation: {:#x}", impl_addr);
            }
        }
    }
    println!();

    // Functions (top 10)
    if !analysis.functions.is_empty() {
        println!(
            "Functions (top {})",
            std::cmp::min(10, analysis.functions.len())
        );
        println!("{}", "─".repeat(65));
        println!(
            "  {:<12} {:<35} {:<12}",
            "Selector", "Signature", "Mutability"
        );

        for func in analysis.functions.iter().take(10) {
            let sig = func.signature.as_ref().cloned().unwrap_or_else(|| {
                if func.arguments.is_empty() {
                    "()".to_string()
                } else {
                    format!("({})", func.arguments.join(", "))
                }
            });
            // Truncate signature if too long
            let sig_display = if sig.len() > 33 {
                format!("{}...", &sig[..30])
            } else {
                sig
            };
            println!(
                "  {:<12} {:<35} {:<12}",
                func.selector, sig_display, func.state_mutability
            );
        }
        if analysis.functions.len() > 10 {
            println!("  ... and {} more functions", analysis.functions.len() - 10);
        }
        println!();
    }

    if let Some(ref dispatcher) = analysis.dispatcher {
        println!("Dispatcher");
        println!("{}", "─".repeat(65));
        println!("  {:<12} {:<12} {:<35}", "Selector", "Handler", "Signature");
        for entry in dispatcher.iter().take(10) {
            let sig = entry.signature.as_deref().unwrap_or("");
            let sig_display = if sig.len() > 33 {
                format!("{}...", &sig[..30])
            } else {
                sig.to_string()
            };
            println!(
                "  {:<12} 0x{:<10x} {:<35}",
                entry.selector, entry.handler_offset, sig_display
            );
        }
        if dispatcher.len() > 10 {
            println!(
                "  ... and {} more dispatcher entries",
                dispatcher.len() - 10
            );
        }
        println!();
    }

    if let Some(ref checks) = analysis.checks {
        println!("Handler Checks");
        println!("{}", "─".repeat(65));
        println!("  Risk Level: {}", checks.risk_level);
        println!("  Functions Scanned: {}", checks.function_count);
        println!("  Findings: {}", checks.finding_count);
        if checks.finding_count > 0 {
            println!();
            for function in checks
                .functions
                .iter()
                .filter(|f| !f.findings.is_empty())
                .take(8)
            {
                let name = function
                    .signature
                    .as_deref()
                    .unwrap_or(function.selector.as_str());
                println!(
                    "  {} @ 0x{:x} ({})",
                    name, function.handler_offset, function.state_mutability
                );
                for finding in &function.findings {
                    println!("    [{}] {}", finding.risk, finding.id);
                    println!("       {}", finding.description);
                }
            }
        }
        println!();
    }

    // Security analysis
    println!("Security Analysis");
    println!("{}", "─".repeat(65));

    let risk_indicator = match analysis.security.risk_level {
        RiskLevel::Low => "✓ LOW",
        RiskLevel::Medium => "⚠ MEDIUM",
        RiskLevel::High => "⚠ HIGH",
        RiskLevel::Critical => "✗ CRITICAL",
        RiskLevel::Unknown => "? UNKNOWN (parse failed)",
    };
    println!("  Risk Level: {}", risk_indicator);
    println!();
    println!(
        "  Dangerous Opcodes:    {}",
        analysis.security.dangerous_opcode_count
    );
    println!(
        "  Hardcoded Addresses:  {}",
        analysis.security.hardcoded_address_count
    );

    if analysis.security.issues.is_empty() {
        println!();
        println!("  ✓ No security issues detected");
    } else {
        println!();
        println!("  Issues Found:");
        for issue in &analysis.security.issues {
            let risk_str = match issue.risk {
                RiskLevel::Critical => "[CRITICAL]",
                RiskLevel::High => "[HIGH]",
                RiskLevel::Medium => "[MEDIUM]",
                RiskLevel::Low => "[LOW]",
                RiskLevel::Unknown => "[UNKNOWN]",
            };
            println!("    {} {} (×{})", risk_str, issue.pattern, issue.count);
            println!("       {}", issue.description);
        }
    }

    // Opcode stats summary
    if let Some(ref stats) = analysis.opcode_stats {
        println!();
        println!("Opcode Summary");
        println!("{}", "─".repeat(65));
        println!(
            "  Total Opcodes:      {}",
            with_thousands_sep(&stats.total_opcodes.to_string())
        );
        println!("  PUSH operations:    {}", stats.push_count);
        println!("  JUMP operations:    {}", stats.jump_count);
        println!("  CALL operations:    {}", stats.call_count);
        println!("  Storage ops:        {}", stats.storage_count);
    }

    println!();
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::json_abi::JsonAbi;

    fn parse_abi(json: &str) -> JsonAbi {
        serde_json::from_str(json).expect("valid ABI json")
    }

    #[test]
    fn test_parse_function_query_name_only() {
        let query = parse_function_query("balanceOf").unwrap();
        assert_eq!(query.name, "balanceOf");
        assert!(query.explicit_types.is_none());
    }

    #[test]
    fn test_parse_function_query_full_signature() {
        let query = parse_function_query(" foo(uint, (address,uint256)) ").unwrap();
        assert_eq!(query.name, "foo");
        assert_eq!(
            query.explicit_types,
            Some(vec!["uint256".to_string(), "(address,uint256)".to_string()])
        );
    }

    #[test]
    fn test_select_contract_function_supports_explicit_overload() {
        let abi = parse_abi(
            r#"[
              {"type":"function","name":"foo","inputs":[{"name":"x","type":"uint8"}],"outputs":[],"stateMutability":"view"},
              {"type":"function","name":"foo","inputs":[{"name":"x","type":"uint256"}],"outputs":[],"stateMutability":"view"}
            ]"#,
        );

        let funcs = abi.functions.get("foo").unwrap();
        let query = parse_function_query("foo(uint8)").unwrap();
        let args = vec!["1".to_string()];

        let (function, values) = select_contract_function(&query, funcs, &args).unwrap();
        assert_eq!(function.inputs[0].ty.to_string(), "uint8");
        assert_eq!(values.len(), 1);
    }

    #[test]
    fn test_select_contract_function_reports_ambiguous_name_only() {
        let abi = parse_abi(
            r#"[
              {"type":"function","name":"foo","inputs":[{"name":"x","type":"uint8"}],"outputs":[],"stateMutability":"view"},
              {"type":"function","name":"foo","inputs":[{"name":"x","type":"uint256"}],"outputs":[],"stateMutability":"view"}
            ]"#,
        );

        let funcs = abi.functions.get("foo").unwrap();
        let query = parse_function_query("foo").unwrap();
        let args = vec!["1".to_string()];

        let err = select_contract_function(&query, funcs, &args).unwrap_err();
        assert!(err
            .to_string()
            .contains("Use an explicit signature like foo("));
    }
}
