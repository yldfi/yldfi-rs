//! Contract-related commands
//!
//! Fetch ABI, source code, creation info, and bytecode analysis for contracts

use super::OutputFormat;
use crate::bytecode::{
    analyze_bytecode, analyze_handler_checks, disassemble_bytecode, extract_selectors,
    infer_dispatcher, opcode_stats, BytecodeAnalysis, RiskLevel, MAX_BYTECODE_SIZE,
};
use crate::config::{Chain, EndpointConfig};
use crate::etherscan::Client;
use crate::rpc::Endpoint;
use crate::utils::format::with_thousands_sep;
use alloy::dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt};
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use clap::Subcommand;
use std::path::PathBuf;
use std::str::FromStr;
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
    /// Return types from `name(args)(returns)`, used when there is no ABI
    explicit_returns: Option<Vec<String>>,
    original: String,
}

/// Byte offset of the `)` that closes the `(` at `open`
fn matching_paren(text: &str, open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (i, c) in text[open..].char_indices() {
        match c {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + i);
                }
            }
            _ => {}
        }
    }
    None
}

fn parse_type_list(types: &str) -> anyhow::Result<Vec<String>> {
    split_type_list(types)
        .into_iter()
        .map(canonicalize_type)
        .collect()
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
            explicit_returns: None,
            original,
        });
    };

    let types_end = matching_paren(&original, types_start)
        .ok_or_else(|| anyhow::anyhow!("Invalid function signature '{}': missing ')'", original))?;

    // An optional return spec, as Foundry writes it: `f(uint256)(address)`, or
    // `f(uint256) returns (address)`
    let rest = original[types_end + 1..].trim();
    let rest = rest.strip_prefix("returns").map_or(rest, str::trim_start);
    let explicit_returns = if rest.is_empty() {
        None
    } else {
        let closes_at_end =
            rest.starts_with('(') && matching_paren(rest, 0) == Some(rest.len() - 1);
        if !closes_at_end {
            return Err(anyhow::anyhow!(
                "Invalid function signature '{}': unexpected trailing characters",
                original
            ));
        }
        Some(parse_type_list(&rest[1..rest.len() - 1])?)
    };

    let name = original[..types_start].trim();
    if name.is_empty() {
        return Err(anyhow::anyhow!(
            "Invalid function signature '{}': missing function name",
            original
        ));
    }

    let raw_types = &original[types_start + 1..types_end];
    let explicit_types = parse_type_list(raw_types)?;

    Ok(FunctionQuery {
        name: name.to_string(),
        explicit_types: Some(explicit_types),
        explicit_returns,
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

/// A function built from a full signature, for a contract with no usable ABI.
/// `None` for a bare name, which can't be called without one.
fn function_from_query(query: &FunctionQuery) -> anyhow::Result<Option<alloy::json_abi::Function>> {
    let Some(inputs) = &query.explicit_types else {
        return Ok(None);
    };
    let returns = query.explicit_returns.as_deref().unwrap_or_default();
    let signature = format!(
        "{}({})({})",
        query.name,
        inputs.join(","),
        returns.join(",")
    );
    alloy::json_abi::Function::parse(&signature)
        .map(Some)
        .map_err(|e| anyhow::anyhow!("Invalid function signature '{}': {}", query.original, e))
}

/// Build a call from a full signature such as `balanceOf(address)(uint256)`
/// and its arguments, with no ABI lookup. Returns the function (with its
/// return types, if the signature gave any) and the encoded calldata.
pub(crate) fn encode_signature_call(
    signature: &str,
    args: &[String],
) -> anyhow::Result<(alloy::json_abi::Function, Vec<u8>)> {
    let query = parse_function_query(signature)?;
    let function = function_from_query(&query)?.ok_or_else(|| {
        anyhow::anyhow!(
            "'{signature}' needs its argument types, e.g. \"balanceOf(address)(uint256)\""
        )
    })?;
    let (func, values) = select_contract_function(&query, std::slice::from_ref(&function), args)?;
    let calldata = func
        .abi_encode_input(&values)
        .map_err(|e| anyhow::anyhow!("Failed to encode arguments: {}", e))?;
    Ok((function.clone(), calldata))
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
                if let Some(impl_addr) = address_from_word(&result) {
                    return Some(impl_addr);
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
    address_from_word(&result)
}

/// An address returned as a single ABI word: exactly 32 bytes, left-padded
/// with zeros. Anything else (a struct, a string, two values) is not an
/// `implementation()` answer.
fn address_from_word(result: &[u8]) -> Option<Address> {
    if result.len() != 32 || result[..12].iter().any(|b| *b != 0) {
        return None;
    }
    let addr = Address::from_slice(&result[12..]);
    (!addr.is_zero()).then_some(addr)
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

        /// List the source files with their line counts instead of printing
        /// the source
        #[arg(long, conflicts_with_all = ["output", "file", "function", "grep"])]
        list: bool,

        /// Print only this file: its path, or a file name such as `Vault.sol`
        #[arg(long, value_name = "PATH", conflicts_with = "output")]
        file: Option<String>,

        /// Print only this function, modifier or constructor, with its
        /// NatSpec comment and file and line (every definition with that name)
        #[arg(long, value_name = "NAME", conflicts_with_all = ["output", "grep"])]
        function: Option<String>,

        /// Print only the lines matching this regular expression, as
        /// `path:line: text`
        #[arg(long, value_name = "REGEX", conflicts_with = "output")]
        grep: Option<String>,

        /// Output format for --list, --file, --function and --grep
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
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

        /// Function name or signature (e.g., "totalSupply", "balanceOf(address)",
        /// "balanceOf(address)(uint256)"). A full signature also works with no ABI
        #[arg(value_name = "FUNCTION")]
        function: String,

        /// Function arguments
        #[arg(value_name = "ARG", allow_negative_numbers = true)]
        args: Vec<String>,

        /// Block number, "latest", or a time (2024-01-31, @1706700000, 7d).
        /// A time selects the latest block at or before it. A plain date is
        /// 00:00 UTC at the start of that day; for an end-of-day figure use
        /// the next day's date or a full time (2024-01-31T23:59:59Z)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
        block: String,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

        /// Format output for human readability (commas in numbers, token decimals)
        #[arg(long, short = 'H')]
        human: bool,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Everything about an address in one call
    ///
    /// Reports whether the address is an EOA, a contract or a delegated EOA
    /// (EIP-7702); for a contract, its proxy kind and implementation, its
    /// verified name and creator, token details if it is a token, Safe owners
    /// if it is a Safe, and its functions. Functions come from the verified
    /// ABI (of the implementation, for a proxy); when the contract is not
    /// verified they are recovered from bytecode and named from the signature
    /// database, which is a guess.
    ///
    /// Replaces the usual chain of `rpc code`, `contract selectors`,
    /// `contract abi`, `contract creation` and `token info`. Parts that cannot
    /// be fetched are listed under "notes" instead of failing the command.
    ///
    /// Examples:
    ///   ethcli contract inspect 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48
    ///   ethcli contract inspect 0x... --json
    #[command(visible_alias = "overview")]
    Inspect {
        /// Address to inspect
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
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
    // Built only by the subcommands that use Etherscan (ABI, source,
    // creation, call): selectors/disassemble/opcodes/analyze are RPC-only and
    // must work on chains Etherscan doesn't cover.
    let etherscan = || -> anyhow::Result<Client> {
        let client = Client::new(chain, api_key.clone())?;
        client.ensure_api_supported()?;
        Ok(client)
    };
    // 4byte.directory lookups are chain-independent
    let signature_client =
        || Client::new(chain, api_key.clone()).or_else(|_| Client::new(Chain::Ethereum, None));

    match action {
        ContractCommands::Abi { address, output } => {
            let client = etherscan()?;
            Address::from_str(address).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            // Check cache first
            let abi = if let Some(abi) = client.cached_abi(address) {
                if !quiet {
                    eprintln!("Using cached ABI for {}...", address);
                }
                abi
            } else {
                if !quiet {
                    eprintln!("Fetching ABI for {}...", address);
                }

                let (abi, origin) = client
                    .fetch_abi_with_origin(address)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to fetch ABI: {}", e))?;
                if !quiet && origin != crate::etherscan::fallback::Origin::Etherscan {
                    eprintln!("Not available from Etherscan; ABI from {origin}");
                }
                abi
            };
            let json = serde_json::to_string_pretty(&abi)?;

            if let Some(path) = output {
                std::fs::write(path, &json)?;
                if !quiet {
                    eprintln!("ABI saved to {}", path.display());
                }
            } else {
                println!("{}", json);
            }
        }

        ContractCommands::Source {
            address,
            output,
            list,
            file,
            function,
            grep,
            format,
        } => {
            let client = etherscan()?;
            Address::from_str(address).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching source code for {}...", address);
            }

            let (metadata, origin) = client
                .source_code_with_origin(address)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to fetch source: {}", e))?;
            if !quiet && origin != crate::etherscan::fallback::Origin::Etherscan {
                eprintln!("Not available from Etherscan; source from {origin}");
            }

            if *list || file.is_some() || function.is_some() || grep.is_some() {
                let files = source_files(&metadata.items);
                if files.is_empty() {
                    return Err(anyhow::anyhow!(
                        "No source code found (contract may not be verified)"
                    ));
                }
                return print_source_slice(
                    &files,
                    *list,
                    file.as_deref(),
                    function.as_deref(),
                    grep.as_deref(),
                    *format,
                );
            }

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
            let client = etherscan()?;
            Address::from_str(address).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            if !quiet {
                eprintln!("Fetching creation info for {}...", address);
            }

            let creation = client
                .creation_data(address)
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
            format,
        } => {
            let client = etherscan()?;
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            // A --block written as a time (a date, @timestamp or age) is
            // turned into a block number first
            let resolved_block;
            let block: &String = if crate::rpc::block_time::parse_time_spec(block, u64::MAX)
                .is_some()
            {
                let probe = match rpc_url {
                    Some(url) => Endpoint::new(EndpointConfig::new(url.clone()), 30, None)?,
                    None => crate::rpc::get_rpc_endpoint(chain)?,
                };
                let resolved = crate::rpc::block_time::resolve_time_spec(probe.provider(), block)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("'{block}' could not be resolved to a block"))?;
                if !quiet {
                    eprintln!(
                        "Resolved --block {block} to block {} ({})",
                        resolved.number,
                        resolved.iso_time()
                    );
                }
                resolved_block = resolved.number.to_string();
                &resolved_block
            } else {
                block
            };

            // Get RPC endpoint first (needed for proxy detection). A historical
            // --block needs an archive-capable endpoint.
            let endpoint = if let Some(url) = rpc_url {
                Endpoint::new(EndpointConfig::new(url.clone()), 30, None)?
            } else {
                let target_block = crate::utils::parse_block_number(block).ok();
                crate::rpc::selector::get_rpc_endpoint_at(chain, target_block)?
            };

            let provider = endpoint.provider();

            let query = parse_function_query(function)?;

            // The target's own ABI first: a contract that merely has an
            // implementation() getter isn't necessarily a proxy
            if !quiet {
                eprintln!("Fetching ABI for {}...", address);
            }
            let own_key = format!("{:#x}", addr);
            let own_abi = match client.cached_abi(&own_key) {
                Some(abi) => Ok(abi),
                None => client.fetch_abi(&own_key).await,
            };
            let has_function =
                |abi: &alloy::json_abi::JsonAbi| abi.functions.contains_key(query.name.as_str());
            let abi_result: anyhow::Result<alloy::json_abi::JsonAbi> = async {
                Ok(match own_abi {
                Ok(abi) if has_function(&abi) => abi,
                own => match detect_proxy_implementation(provider, addr).await {
                    Some(impl_addr) => {
                        if !quiet {
                            eprintln!(
                                "Detected proxy contract, fetching implementation ABI from {:#x}...",
                                impl_addr
                            );
                        }
                        let impl_key = format!("{:#x}", impl_addr);
                        match client.cached_abi(&impl_key) {
                            Some(abi) => abi,
                            None => client
                                .fetch_abi(&impl_key)
                                .await
                                .map_err(|e| anyhow::anyhow!("Failed to fetch ABI: {}", e))?,
                        }
                    }
                    None => own.map_err(|e| anyhow::anyhow!("Failed to fetch ABI: {}", e))?,
                },
            })
            }
            .await;

            // Find the function - handle overloaded functions by matching arg count and types.
            // A full signature can be called without an ABI, for an unverified contract.
            let signature_function = function_from_query(&query)?;
            let abi_functions = abi_result
                .as_ref()
                .ok()
                .and_then(|abi| abi.functions.get(query.name.as_str()));
            let funcs: &[alloy::json_abi::Function] = match (abi_functions, &signature_function) {
                (Some(funcs), _) => funcs,
                (None, Some(function)) => {
                    if !quiet {
                        eprintln!("No ABI for this function; using the signature given");
                    }
                    std::slice::from_ref(function)
                }
                (None, None) => {
                    return Err(match abi_result {
                        Err(e) => e,
                        Ok(_) => anyhow::anyhow!("Function '{}' not found in ABI", query.original),
                    })
                }
            };

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

            let result = match provider.call(tx).block(block_id).await {
                Ok(result) => result,
                Err(e) => {
                    // A revert is an answer from the contract: say why, using
                    // the contract's own error definitions when we have them
                    let Some(data) = crate::abi::revert::revert_data_of(&e) else {
                        return Err(anyhow::anyhow!("Call failed: {}", e));
                    };
                    let revert =
                        crate::abi::revert::describe(&data, abi_result.as_ref().ok(), chain).await;
                    if !format.is_table() {
                        let out = serde_json::json!({
                            "address": addr.to_checksum(None),
                            "function": func.signature(),
                            "block": block,
                            "reverted": true,
                            "revert": revert,
                        });
                        if format.is_ndjson() {
                            println!("{}", serde_json::to_string(&out)?);
                        } else {
                            println!("{}", serde_json::to_string_pretty(&out)?);
                        }
                    }
                    return Err(anyhow::anyhow!("Call reverted: {}", revert.message));
                }
            };

            // Decode the result
            if !format.is_table() {
                let decoded = if func.outputs.is_empty() {
                    None
                } else {
                    Some(
                        func.abi_decode_output(&result)
                            .map_err(|e| anyhow::anyhow!("Failed to decode output: {}", e))?,
                    )
                };
                let mut out = serde_json::json!({
                    "address": addr.to_checksum(None),
                    "function": func.signature(),
                    "block": block,
                    "raw": result.to_string(),
                });
                if let Some(decoded) = &decoded {
                    let outputs: Vec<serde_json::Value> = func
                        .outputs
                        .iter()
                        .zip(decoded)
                        .map(|(output, value)| {
                            serde_json::json!({
                                "name": output.name,
                                "type": output.ty,
                                "value": dyn_value_to_json(value),
                            })
                        })
                        .collect();
                    out["outputs"] = serde_json::Value::Array(outputs);
                    // Token amounts also come scaled, so the caller need not
                    // fetch decimals and divide
                    if decoded.len() == 1 && is_amount_getter(&func.name) {
                        if let (DynSolValue::Uint(amount, _), Some(decimals)) =
                            (&decoded[0], get_token_decimals(provider, addr).await)
                        {
                            out["decimals"] = serde_json::json!(decimals);
                            out["formatted"] = serde_json::json!(
                                alloy::primitives::utils::format_units(*amount, decimals)
                                    .map_err(|e| anyhow::anyhow!("Failed to scale amount: {e}"))?
                            );
                        }
                    }
                }
                if format.is_ndjson() {
                    println!("{}", serde_json::to_string(&out)?);
                } else {
                    println!("{}", serde_json::to_string_pretty(&out)?);
                }
            } else if func.outputs.is_empty() {
                // With no ABI and no return spec, the types are unknown: show the data
                if result.is_empty() {
                    println!("(no return value)");
                } else {
                    println!("{result}");
                }
            } else {
                let decoded = func
                    .abi_decode_output(&result)
                    .map_err(|e| anyhow::anyhow!("Failed to decode output: {}", e))?;

                // For human mode, scale by the token's decimals only for getters
                // that return token amounts (decimals(), nonces, timestamps... stay raw)
                let token_decimals = if *human && is_amount_getter(&func.name) {
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

        ContractCommands::Inspect {
            address,
            rpc_url,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
            if !quiet {
                eprintln!("Inspecting {}...", address);
            }
            let report = inspect_address(chain, api_key.clone(), addr, rpc_url.as_deref()).await?;
            if format.is_table() {
                print!("{}", format_inspect_table(&report));
            } else if format.is_ndjson() {
                println!("{}", serde_json::to_string(&report)?);
            } else {
                println!("{}", serde_json::to_string_pretty(&report)?);
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
                lookup_signatures(&signature_client()?, &mut functions).await;
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
            // With --follow-proxy, always check the implementation slots: the
            // bytecode heuristic misses some storage proxies
            let (analysis_address, analysis_bytecode, proxy_detected) = if proxy_info.is_proxy
                || *follow_proxy
            {
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
                    } else if proxy_info.is_proxy {
                        if !quiet {
                            eprintln!(
                                "Warning: Could not determine implementation address, analyzing proxy bytecode"
                            );
                        }
                        (address.clone(), bytecode, Some(proxy_info))
                    } else {
                        // Not a proxy after all
                        (address.clone(), bytecode, None)
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
                lookup_signatures(&signature_client()?, &mut analysis.functions).await;
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

/// Candidate endpoints for a read: the explicit `--rpc-url` if given,
/// otherwise the ranked configured endpoints (for failover).
fn read_candidates(chain: &Chain, rpc_url: Option<&str>) -> anyhow::Result<Vec<Endpoint>> {
    if let Some(url) = rpc_url {
        Ok(vec![Endpoint::new(
            EndpointConfig::new(url.to_string()),
            30,
            None,
        )?])
    } else {
        crate::rpc::candidate_endpoints(
            *chain,
            &crate::rpc::SelectionOptions::default(),
            crate::rpc::MAX_FAILOVER_ATTEMPTS,
        )
    }
}

/// Get bytecode for an address via RPC
async fn get_bytecode(
    chain: &Chain,
    rpc_url: Option<&str>,
    address: Address,
) -> anyhow::Result<Vec<u8>> {
    let candidates = read_candidates(chain, rpc_url)?;
    crate::rpc::with_failover(candidates, |endpoint| async move {
        let code = endpoint
            .provider()
            .get_code_at(address)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch bytecode: {}", e))?;
        Ok(code.to_vec())
    })
    .await
}

/// Try to get implementation address from EIP-1967 storage slot
///
/// RPC errors fail over to the next endpoint; if every endpoint fails the
/// error is reported as a warning (instead of being silently treated as
/// "not a proxy").
async fn get_implementation_from_storage(
    chain: &Chain,
    rpc_url: Option<&str>,
    proxy_address: Address,
) -> Option<Address> {
    let candidates = match read_candidates(chain, rpc_url) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: cannot check proxy implementation slots: {e}");
            return None;
        }
    };
    match crate::rpc::with_failover(candidates, |endpoint| async move {
        implementation_from_storage_on(&endpoint, proxy_address).await
    })
    .await
    {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!(
                "Warning: failed to read proxy implementation slots: {}",
                crate::utils::url::redact_urls_in_text(&format!("{e:#}"))
            );
            None
        }
    }
}

/// Read the known implementation slots on a single endpoint, propagating
/// RPC errors so the caller can fail over.
async fn implementation_from_storage_on(
    endpoint: &Endpoint,
    proxy_address: Address,
) -> anyhow::Result<Option<Address>> {
    use crate::bytecode::{address_from_storage, proxy_slots, u256_to_b256};

    let provider = endpoint.provider();

    let slots = [
        // EIP-1967 implementation slot
        proxy_slots::EIP1967_IMPLEMENTATION.into(),
        // OpenZeppelin legacy slot
        proxy_slots::OZ_LEGACY_IMPLEMENTATION.into(),
        // OpenZeppelin AdminUpgradeabilityProxy slot used by older proxies such as USDC.
        oz_impl_slot().into(),
    ];
    for slot in slots {
        let value = provider
            .get_storage_at(proxy_address, slot)
            .await
            .map_err(|e| anyhow::anyhow!("eth_getStorageAt failed: {}", e))?;
        if let Some(addr) = address_from_storage(u256_to_b256(value)) {
            return Ok(Some(addr));
        }
    }

    Ok(call_implementation_function(provider, proxy_address).await)
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
            let sig_display = crate::utils::format::truncate_str(&sig, 33);
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
            let sig_display = crate::utils::format::truncate_str(sig, 33);
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
/// EIP-1967 admin slot: keccak256("eip1967.proxy.admin") - 1
fn eip1967_admin_slot() -> B256 {
    static SLOT: OnceLock<B256> = OnceLock::new();
    *SLOT.get_or_init(|| {
        "0xb53127684a568b3173ae13b9f8a6016e243e63b6e8ee1178d6a717850b5d6103"
            .parse()
            .expect("valid slot")
    })
}

/// The address held in a storage slot, if the slot is not empty
pub(crate) async fn storage_address<P: Provider>(
    provider: &P,
    address: Address,
    slot: B256,
    block: alloy::eips::BlockId,
) -> Option<Address> {
    let word = provider
        .get_storage_at(address, slot.into())
        .block_id(block)
        .await
        .ok()?;
    let found = Address::from_slice(&word.to_be_bytes::<32>()[12..]);
    (!found.is_zero()).then_some(found)
}

/// How a proxy's implementation was found
async fn find_implementation<P: Provider>(
    provider: &P,
    address: Address,
    code: &[u8],
    block: alloy::eips::BlockId,
) -> Option<(&'static str, Address)> {
    if let Some(found) = crate::bytecode::detect_proxy(code).implementation {
        return Some(("EIP-1167 minimal proxy", found));
    }
    if let Some(found) = storage_address(provider, address, eip1967_impl_slot(), block).await {
        return Some(("EIP-1967 proxy", found));
    }
    if let Some(found) = storage_address(provider, address, oz_impl_slot(), block).await {
        return Some(("OpenZeppelin legacy proxy", found));
    }
    if let Some(beacon) = storage_address(provider, address, eip1967_beacon_slot(), block).await {
        if let Some(found) = call_implementation_function(provider, beacon).await {
            return Some(("EIP-1967 beacon proxy", found));
        }
    }
    // Safe proxies keep their singleton in slot 0 and answer masterCopy()
    // themselves (selector 0xa619486e)
    let master_copy = alloy::rpc::types::TransactionRequest::default()
        .to(address)
        .input(vec![0xa6, 0x19, 0x48, 0x6e].into());
    if let Ok(result) = provider.call(master_copy).block(block).await {
        if let Some(found) = address_from_word(&result).filter(|a| !a.is_zero()) {
            return Some(("Safe proxy", found));
        }
    }
    // A contract can expose implementation() without delegating to it
    call_implementation_function(provider, address)
        .await
        .map(|found| ("implementation() getter (may not be a proxy)", found))
}

/// Everything `contract inspect` reports about an address, as JSON
async fn inspect_address(
    chain: Chain,
    api_key: Option<String>,
    address: Address,
    rpc_url: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    use crate::rpc::MulticallBuilder;
    use alloy::primitives::keccak256;

    let endpoint = match rpc_url {
        Some(url) => Endpoint::new(EndpointConfig::new(url.to_string()), 30, None)?,
        None => crate::rpc::get_rpc_endpoint(chain)?,
    };
    let provider = endpoint.provider();
    let mut notes: Vec<String> = Vec::new();

    // Pin one block so every part describes the same state
    let block_number = provider
        .get_block_number()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get the latest block: {}", e))?;
    let block: alloy::eips::BlockId = block_number.into();

    let code = provider
        .get_code_at(address)
        .block_id(block)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get code: {}", e))?;
    let balance = provider
        .get_balance(address)
        .block_id(block)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))?;

    let mut out = serde_json::json!({
        "address": address.to_checksum(None),
        "chain": chain.display_name(),
        "block": block_number,
        "balance": {
            "wei": balance.to_string(),
            "formatted": super::cast::from_wei(&balance.to_string(), "eth")?,
            "symbol": chain.native_symbol(),
        },
    });

    let book = crate::config::AddressBook::load_default();
    let wanted = format!("{address:#x}");
    if let Some((label, entry)) = book
        .list(None)
        .into_iter()
        .find(|(_, entry)| entry.address.eq_ignore_ascii_case(&wanted))
    {
        out["label"] = serde_json::json!(label);
        if let Some(description) = &entry.description {
            out["label_description"] = serde_json::json!(description);
        }
    }

    // Not a contract
    let delegate = (code.len() == 23 && code[..3] == [0xef, 0x01, 0x00])
        .then(|| Address::from_slice(&code[3..]));
    if code.is_empty() || delegate.is_some() {
        let nonce = provider
            .get_transaction_count(address)
            .block_id(block)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get nonce: {}", e))?;
        out["nonce"] = serde_json::json!(nonce);
        match delegate {
            Some(delegate) => {
                out["kind"] = serde_json::json!("eoa_delegated");
                out["eip7702_delegate"] = serde_json::json!(delegate.to_checksum(None));
                notes.push(
                    "An EOA that delegates its code under EIP-7702; inspect the delegate for its functions"
                        .to_string(),
                );
            }
            None => out["kind"] = serde_json::json!("eoa"),
        }
        out["notes"] = serde_json::json!(notes);
        return Ok(out);
    }

    out["kind"] = serde_json::json!("contract");
    out["code"] = serde_json::json!({
        "bytes": code.len(),
        "hash": format!("{:#x}", keccak256(&code)),
    });

    // Proxy
    let implementation = find_implementation(provider, address, &code, block).await;
    if let Some((kind, target)) = implementation {
        let mut proxy = serde_json::json!({
            "kind": kind,
            "implementation": target.to_checksum(None),
        });
        if let Some(admin) = storage_address(provider, address, eip1967_admin_slot(), block).await {
            proxy["admin"] = serde_json::json!(admin.to_checksum(None));
        }
        if let Some(beacon) = storage_address(provider, address, eip1967_beacon_slot(), block).await
        {
            proxy["beacon"] = serde_json::json!(beacon.to_checksum(None));
        }
        out["proxy"] = proxy;
    }
    let interface_target = implementation.map(|(_, target)| target).unwrap_or(address);

    // Standard getters, read together at the pinned block. Each may fail.
    let probe = |signature: &str| keccak256(signature.as_bytes())[..4].to_vec();
    let probes = [
        "name()",
        "symbol()",
        "decimals()",
        "totalSupply()",
        "owner()",
        "getOwners()",
        "getThreshold()",
        "asset()",
    ];
    let batch = probes.iter().fold(MulticallBuilder::new(), |b, signature| {
        b.add_call(address, probe(signature).into(), true)
    });
    match batch.execute_at(provider, block).await {
        Ok(results) if results.len() == probes.len() => {
            let ok = |i: usize| results[i].success && !results[i].data.is_empty();
            let text = |i: usize| ok(i).then(|| results[i].decode_string()).flatten();
            let number = |i: usize| ok(i).then(|| results[i].decode_uint256()).flatten();
            let account = |i: usize| ok(i).then(|| address_from_word(&results[i].data)).flatten();

            let decimals = ok(2).then(|| results[2].decode_uint8()).flatten();
            if let (Some(symbol), Some(decimals)) = (text(1), decimals) {
                let mut token = serde_json::json!({
                    "symbol": symbol,
                    "decimals": decimals,
                });
                if let Some(name) = text(0) {
                    token["name"] = serde_json::json!(name);
                }
                if let Some(supply) = number(3) {
                    token["total_supply"] = serde_json::json!(supply.to_string());
                    if let Ok(formatted) = alloy::primitives::utils::format_units(supply, decimals)
                    {
                        token["total_supply_formatted"] = serde_json::json!(formatted);
                    }
                }
                out["token"] = token;
            }
            if let Some(owner) = account(4) {
                out["owner"] = serde_json::json!(owner.to_checksum(None));
            }
            let owners = ok(5)
                .then(|| {
                    DynSolType::Array(Box::new(DynSolType::Address))
                        .abi_decode(&results[5].data)
                        .ok()
                })
                .flatten();
            if let (Some(DynSolValue::Array(owners)), Some(threshold)) = (owners, number(6)) {
                out["safe"] = serde_json::json!({
                    "threshold": threshold.to_string(),
                    "owners": owners.iter().map(dyn_value_to_json).collect::<Vec<_>>(),
                });
            }
            if let Some(asset) = account(7) {
                out["erc4626_asset"] = serde_json::json!(asset.to_checksum(None));
            }
        }
        _ => notes.push(
            "Standard getters (token, owner, Safe) were not read: Multicall3 is not available here"
                .to_string(),
        ),
    }

    // Explorer: verified name, creator, and the ABI of the code that runs
    let address_text = format!("{address:#x}");
    let target_text = format!("{interface_target:#x}");
    let explorer = Client::new(chain, api_key.clone()).and_then(|client| {
        client.ensure_api_supported()?;
        Ok(client)
    });
    let mut abi: Option<alloy::json_abi::JsonAbi> = None;
    match &explorer {
        Ok(client) => {
            let metadata = async {
                crate::abi::AbiFetcher::new(api_key.clone())?
                    .get_contract_metadata(chain, &address_text)
                    .await
            };
            let fetched_abi = async {
                match client.cached_abi(&target_text) {
                    Some(abi) => Ok(abi),
                    None => client.fetch_abi(&target_text).await,
                }
            };
            // One at a time: explorers rate-limit to a few calls a second
            let metadata = metadata.await;
            let creation = client.creation_data(&address_text).await;
            let fetched_abi = fetched_abi.await;
            match metadata {
                Ok(metadata) => {
                    out["verified"] = serde_json::json!(metadata.is_verified);
                    if let Some(name) = metadata.name {
                        out["name"] = serde_json::json!(name);
                    }
                }
                Err(e) => notes.push(format!("Verification status not fetched: {e}")),
            }
            match creation {
                Ok(creation) => {
                    out["creation"] = serde_json::json!({
                        "creator": creation.contract_creator.to_checksum(None),
                        "tx_hash": format!("{:#x}", creation.transaction_hash),
                    });
                }
                Err(e) => notes.push(format!("Creation info not fetched: {e}")),
            }
            // A proxy's own ABI can verify with no functions at all; that
            // tells the reader nothing, so fall back to the bytecode
            abi = fetched_abi
                .ok()
                .filter(|abi| abi.functions().next().is_some());
        }
        Err(e) => notes.push(format!("No block explorer for this chain: {e}")),
    }

    // Interface: the verified ABI when there is one, bytecode otherwise
    match abi {
        Some(abi) => {
            let mut functions: Vec<serde_json::Value> = abi
                .functions()
                .map(|f| {
                    serde_json::json!({
                        "signature": f.signature(),
                        "mutability": format!("{:?}", f.state_mutability).to_lowercase(),
                    })
                })
                .collect();
            functions.sort_by(|a, b| a["signature"].as_str().cmp(&b["signature"].as_str()));
            let mut events: Vec<String> = abi.events().map(|e| e.signature()).collect();
            let mut errors: Vec<String> = abi.errors().map(|e| e.signature()).collect();
            events.sort();
            errors.sort();
            out["interface"] = serde_json::json!({
                "source": "abi",
                "of": interface_target.to_checksum(None),
                "functions": functions,
                "events": events,
                "errors": errors,
            });
        }
        None => {
            let target_code = if interface_target == address {
                code.clone()
            } else {
                provider
                    .get_code_at(interface_target)
                    .block_id(block)
                    .await
                    .unwrap_or_default()
            };
            let mut functions = extract_selectors(&target_code);
            if let Ok(client) =
                Client::new(chain, api_key.clone()).or_else(|_| Client::new(Chain::Ethereum, None))
            {
                lookup_signatures(&client, &mut functions).await;
            }
            out["interface"] = serde_json::json!({
                "source": "bytecode",
                "of": interface_target.to_checksum(None),
                "functions": functions,
            });
            notes.push(
                "No verified ABI: selectors were recovered from bytecode, and any function names \
                 are guesses from the signature database"
                    .to_string(),
            );
        }
    }

    out["notes"] = serde_json::json!(notes);
    Ok(out)
}

/// Human-readable form of an `inspect_address` report
fn format_inspect_table(report: &serde_json::Value) -> String {
    use crate::abi::revert::safe_text;
    use std::fmt::Write;

    let text = |value: &serde_json::Value| value.as_str().map(safe_text);
    let mut out = String::new();
    let line = |out: &mut String, key: &str, value: String| {
        let _ = writeln!(out, "{:<16}{}", format!("{key}:"), value);
    };

    line(
        &mut out,
        "Address",
        text(&report["address"]).unwrap_or_default(),
    );
    if let Some(label) = text(&report["label"]) {
        line(&mut out, "Label", label);
    }
    let kind = match report["kind"].as_str() {
        Some("eoa") => "EOA (no code)".to_string(),
        Some("eoa_delegated") => format!(
            "EOA delegated to {} (EIP-7702)",
            text(&report["eip7702_delegate"]).unwrap_or_default()
        ),
        _ => match (text(&report["name"]), report["verified"].as_bool()) {
            (Some(name), _) => format!("contract {name} (verified)"),
            (None, Some(false)) => "contract (not verified)".to_string(),
            _ => "contract".to_string(),
        },
    };
    line(&mut out, "Kind", kind);
    line(
        &mut out,
        "Chain",
        format!(
            "{} at block {}",
            text(&report["chain"]).unwrap_or_default(),
            report["block"]
        ),
    );
    line(
        &mut out,
        "Balance",
        format!(
            "{} {}",
            text(&report["balance"]["formatted"]).unwrap_or_default(),
            text(&report["balance"]["symbol"]).unwrap_or_default()
        ),
    );
    if let Some(nonce) = report.get("nonce") {
        line(&mut out, "Nonce", nonce.to_string());
    }
    if let Some(bytes) = report["code"]["bytes"].as_u64() {
        line(
            &mut out,
            "Code",
            format!(
                "{bytes} bytes, hash {}",
                text(&report["code"]["hash"]).unwrap_or_default()
            ),
        );
    }
    if let Some(implementation) = text(&report["proxy"]["implementation"]) {
        line(
            &mut out,
            "Proxy",
            format!(
                "{} -> {implementation}",
                text(&report["proxy"]["kind"]).unwrap_or_default()
            ),
        );
        if let Some(admin) = text(&report["proxy"]["admin"]) {
            line(&mut out, "Proxy admin", admin);
        }
    }
    if let Some(creator) = text(&report["creation"]["creator"]) {
        line(
            &mut out,
            "Created by",
            format!(
                "{creator} in {}",
                text(&report["creation"]["tx_hash"]).unwrap_or_default()
            ),
        );
    }
    if let Some(symbol) = text(&report["token"]["symbol"]) {
        let name = text(&report["token"]["name"]).unwrap_or_default();
        let supply = text(&report["token"]["total_supply_formatted"])
            .map(|s| format!(", total supply {s}"))
            .unwrap_or_default();
        line(
            &mut out,
            "Token",
            format!(
                "{name} ({symbol}), {} decimals{supply}",
                report["token"]["decimals"]
            ),
        );
    }
    if let Some(asset) = text(&report["erc4626_asset"]) {
        line(&mut out, "Vault asset", asset);
    }
    if let Some(owner) = text(&report["owner"]) {
        line(&mut out, "Owner", owner);
    }
    if let Some(owners) = report["safe"]["owners"].as_array() {
        line(
            &mut out,
            "Safe",
            format!(
                "{} of {} owners",
                text(&report["safe"]["threshold"]).unwrap_or_default(),
                owners.len()
            ),
        );
        for owner in owners {
            let _ = writeln!(out, "{:<16}{}", "", text(owner).unwrap_or_default());
        }
    }
    if let Some(functions) = report["interface"]["functions"].as_array() {
        let from_abi = report["interface"]["source"].as_str() == Some("abi");
        line(
            &mut out,
            "Functions",
            format!(
                "{} ({} {}{})",
                functions.len(),
                if from_abi {
                    "verified ABI of"
                } else {
                    "recovered from the bytecode of"
                },
                text(&report["interface"]["of"]).unwrap_or_default(),
                if from_abi { "" } else { "; names are guesses" }
            ),
        );
        for function in functions {
            let (name, mutability) = if from_abi {
                (
                    text(&function["signature"]).unwrap_or_default(),
                    text(&function["mutability"]).unwrap_or_default(),
                )
            } else {
                (
                    text(&function["signature"])
                        .or_else(|| text(&function["selector"]))
                        .unwrap_or_default(),
                    text(&function["state_mutability"]).unwrap_or_default(),
                )
            };
            let _ = writeln!(out, "  {name}  [{mutability}]");
        }
    }
    for note in report["notes"].as_array().into_iter().flatten() {
        let _ = writeln!(out, "Note: {}", text(note).unwrap_or_default());
    }
    out
}

/// A decoded ABI value as JSON. Integers are decimal strings (a uint256 does
/// not fit a JSON number), addresses are checksummed, byte values are 0x hex,
/// and arrays and tuples become JSON arrays.
pub(crate) fn dyn_value_to_json(value: &DynSolValue) -> serde_json::Value {
    use serde_json::Value;
    match value {
        DynSolValue::Bool(b) => Value::Bool(*b),
        DynSolValue::Int(i, _) => Value::String(i.to_string()),
        DynSolValue::Uint(u, _) => Value::String(u.to_string()),
        DynSolValue::Address(a) => Value::String(a.to_checksum(None)),
        DynSolValue::String(s) => Value::String(s.clone()),
        DynSolValue::Bytes(b) => Value::String(format!("0x{}", hex::encode(b))),
        DynSolValue::FixedBytes(word, size) => {
            Value::String(format!("0x{}", hex::encode(&word[..*size])))
        }
        DynSolValue::Function(f) => Value::String(format!("0x{}", hex::encode(f.as_slice()))),
        DynSolValue::Array(items) | DynSolValue::FixedArray(items) | DynSolValue::Tuple(items) => {
            Value::Array(items.iter().map(dyn_value_to_json).collect())
        }
    }
}

pub(crate) fn format_value(value: &DynSolValue) -> String {
    format_value_internal(value, false, None)
}

/// Fill in signatures from 4byte.directory, a few lookups at a time instead
/// of one after another
async fn lookup_signatures(
    client: &crate::etherscan::Client,
    functions: &mut [crate::bytecode::ExtractedFunction],
) {
    use futures::StreamExt;
    const CONCURRENT_LOOKUPS: usize = 8;
    let selectors: Vec<String> = functions
        .iter()
        .map(|f| f.selector.trim_start_matches("0x").to_string())
        .collect();
    let signatures: Vec<Option<String>> = futures::stream::iter(selectors)
        .map(|selector| async move { client.lookup_selector(&selector).await })
        .buffered(CONCURRENT_LOOKUPS)
        .collect()
        .await;
    for (func, sig) in functions.iter_mut().zip(signatures) {
        if sig.is_some() {
            func.signature = sig;
        }
    }
}

/// Whether a function returns a token amount that `--human` should scale by
/// the token's decimals (ERC-20 / ERC-4626 amount getters)
fn is_amount_getter(name: &str) -> bool {
    const AMOUNT_GETTERS: &[&str] = &[
        "totalsupply",
        "balanceof",
        "allowance",
        "totalassets",
        "converttoassets",
        "converttoshares",
        "previewdeposit",
        "previewmint",
        "previewwithdraw",
        "previewredeem",
        "maxdeposit",
        "maxmint",
        "maxwithdraw",
        "maxredeem",
        "circulatingsupply",
        "maxsupply",
        "cap",
    ];
    AMOUNT_GETTERS.contains(&name.to_ascii_lowercase().as_str())
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

/// The verified source as separate files, sorted by path. A single-file
/// verification is named after the contract.
fn source_files(
    items: &[foundry_block_explorers::contract::Metadata],
) -> Vec<super::source_view::SourceFile> {
    use foundry_block_explorers::contract::SourceCodeMetadata;
    let mut files = Vec::new();
    for item in items {
        match &item.source_code {
            SourceCodeMetadata::SourceCode(text) => {
                let extension = if item.compiler_version.to_ascii_lowercase().contains("vyper") {
                    "vy"
                } else {
                    "sol"
                };
                files.push(super::source_view::SourceFile {
                    path: format!("{}.{extension}", item.contract_name),
                    content: text.clone(),
                });
            }
            other => {
                for (path, entry) in other.sources() {
                    files.push(super::source_view::SourceFile {
                        path,
                        content: entry.content,
                    });
                }
            }
        }
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files.dedup_by(|a, b| a.path == b.path);
    super::source_view::strip_common_absolute_prefix(&mut files);
    files.sort_by(|a, b| a.path.cmp(&b.path));
    files
}

/// Print the part of the source that `--list`, `--file`, `--function` or
/// `--grep` asked for. `--file` narrows `--function` and `--grep` to the
/// matching files.
fn print_source_slice(
    files: &[super::source_view::SourceFile],
    list: bool,
    file: Option<&str>,
    function: Option<&str>,
    grep: Option<&str>,
    format: OutputFormat,
) -> anyhow::Result<()> {
    use super::source_view;

    if list {
        let listed = source_view::list(files);
        if format.is_table() {
            for entry in &listed {
                println!(
                    "{:>6} lines  {}",
                    entry["lines"].as_u64().unwrap_or(0),
                    entry["path"].as_str().unwrap_or("")
                );
            }
        } else {
            println!("{}", serde_json::to_string_pretty(&listed)?);
        }
        return Ok(());
    }

    let selected: Vec<source_view::SourceFile> = match file {
        Some(name) => {
            let found = source_view::find_files(files, name);
            if found.is_empty() {
                return Err(anyhow::anyhow!(
                    "No source file matches '{name}' ({} files; see them with --list)",
                    files.len()
                ));
            }
            found.into_iter().cloned().collect()
        }
        None => files.to_vec(),
    };

    if let Some(name) = function {
        let found = source_view::functions(&selected, name)?;
        if found.is_empty() {
            return Err(anyhow::anyhow!(
                "No function, modifier or constructor named '{name}' in the verified source \
                 (search it with --grep)"
            ));
        }
        if format.is_table() {
            for (i, f) in found.iter().enumerate() {
                if i > 0 {
                    println!();
                }
                println!("// {}:{}", f.path, f.line);
                println!("{}", f.text);
            }
        } else {
            println!("{}", serde_json::to_string_pretty(&found)?);
        }
        return Ok(());
    }

    if let Some(pattern) = grep {
        let hits = source_view::grep(&selected, pattern)?;
        if format.is_table() {
            for hit in &hits {
                println!("{}:{}: {}", hit.path, hit.line, hit.text);
            }
            if hits.is_empty() {
                eprintln!("No lines match '{pattern}'");
            }
        } else {
            println!("{}", serde_json::to_string_pretty(&hits)?);
        }
        return Ok(());
    }

    // --file alone
    if format.is_table() {
        for (i, f) in selected.iter().enumerate() {
            if selected.len() > 1 {
                if i > 0 {
                    println!();
                }
                println!("// File: {}", f.path);
            }
            println!("{}", f.content);
        }
    } else {
        let out: Vec<serde_json::Value> = selected
            .iter()
            .map(|f| serde_json::json!({"path": f.path, "content": f.content}))
            .collect();
        println!("{}", serde_json::to_string_pretty(&out)?);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_table_shows_each_part_and_escapes_contract_text() {
        let report = serde_json::json!({
            "address": "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "chain": "Ethereum",
            "block": 100,
            "kind": "contract",
            "verified": true,
            "name": "FiatTokenProxy",
            "balance": {"wei": "0", "formatted": "0.0", "symbol": "ETH"},
            "code": {"bytes": 2186, "hash": "0xabc"},
            "proxy": {"kind": "EIP-1967 proxy", "implementation": "0x1111", "admin": "0x2222"},
            "creation": {"creator": "0x3333", "tx_hash": "0x4444"},
            "token": {"name": "Evil\u{1b}[31m", "symbol": "E\u{202e}VIL", "decimals": 6,
                      "total_supply_formatted": "1.5"},
            "owner": "0x5555",
            "safe": {"threshold": "2", "owners": ["0x6666", "0x7777"]},
            "interface": {"source": "abi", "of": "0x1111",
                          "functions": [{"signature": "transfer(address,uint256)", "mutability": "nonpayable"}]},
            "notes": ["something was skipped"]
        });
        let out = format_inspect_table(&report);
        for expected in [
            "Kind:           contract FiatTokenProxy (verified)",
            "Chain:          Ethereum at block 100",
            "Proxy:          EIP-1967 proxy -> 0x1111",
            "Proxy admin:    0x2222",
            "Created by:     0x3333 in 0x4444",
            "6 decimals, total supply 1.5",
            "Safe:           2 of 2 owners",
            "Functions:      1 (verified ABI of 0x1111)",
            "  transfer(address,uint256)  [nonpayable]",
            "Note: something was skipped",
        ] {
            assert!(out.contains(expected), "missing {expected:?} in:\n{out}");
        }
        // The token's own name and symbol cannot carry control characters
        assert!(out.contains("Evil\\u{1b}[31m (E\\u{202e}VIL)"), "{out}");
        assert!(!out.chars().any(|c| c == '\u{1b}' || c == '\u{202e}'));

        let eoa = serde_json::json!({
            "address": "0x01", "chain": "Ethereum", "block": 1, "kind": "eoa", "nonce": 7,
            "balance": {"wei": "1", "formatted": "0.000000000000000001", "symbol": "ETH"},
            "notes": []
        });
        let out = format_inspect_table(&eoa);
        assert!(out.contains("Kind:           EOA (no code)"), "{out}");
        assert!(out.contains("Nonce:          7"), "{out}");

        let unverified = serde_json::json!({
            "address": "0x02", "chain": "Ethereum", "block": 1, "kind": "contract", "verified": false,
            "balance": {"wei": "0", "formatted": "0.0", "symbol": "ETH"},
            "interface": {"source": "bytecode", "of": "0x02",
                          "functions": [{"selector": "0xa9059cbb", "state_mutability": "nonpayable"}]},
            "notes": []
        });
        let out = format_inspect_table(&unverified);
        assert!(out.contains("contract (not verified)"), "{out}");
        assert!(
            out.contains("1 (recovered from the bytecode of 0x02; names are guesses)"),
            "{out}"
        );
        assert!(out.contains("  0xa9059cbb  [nonpayable]"), "{out}");
    }

    #[test]
    fn inspect_parses_with_its_alias_and_json_flag() {
        use clap::Parser;
        for args in [
            vec!["ethcli", "contract", "inspect", "0x00"],
            vec!["ethcli", "contract", "overview", "0x00", "-o", "json"],
        ] {
            assert!(crate::cli::Cli::try_parse_from(&args).is_ok(), "{args:?}");
        }
        let expanded = crate::cli::expand_json_flag(
            ["ethcli", "contract", "inspect", "0x00", "--json"]
                .iter()
                .map(|s| s.to_string())
                .collect(),
        );
        assert!(
            crate::cli::Cli::try_parse_from(&expanded).is_ok(),
            "{expanded:?}"
        );
    }
    use alloy::json_abi::JsonAbi;

    #[test]
    fn function_query_accepts_a_return_spec() {
        for text in [
            "decimals()(uint8)",
            "decimals() returns (uint8)",
            " decimals() ( uint8 ) ",
        ] {
            let query = parse_function_query(text).unwrap();
            assert_eq!(query.name, "decimals", "{text}");
            assert_eq!(query.explicit_types, Some(vec![]), "{text}");
            assert_eq!(
                query.explicit_returns,
                Some(vec!["uint8".to_string()]),
                "{text}"
            );
        }
        let query =
            parse_function_query("f((address,uint256),uint256)((uint8,bool),address)").unwrap();
        assert_eq!(
            query.explicit_types,
            Some(vec!["(address,uint256)".to_string(), "uint256".to_string()])
        );
        assert_eq!(
            query.explicit_returns,
            Some(vec!["(uint8,bool)".to_string(), "address".to_string()])
        );
        assert_eq!(
            parse_function_query("f(uint256)").unwrap().explicit_returns,
            None
        );
        assert_eq!(parse_function_query("f").unwrap().explicit_returns, None);
    }

    #[test]
    fn function_query_rejects_malformed_signatures() {
        for text in [
            "f()x",
            "f()(uint8)x",
            "f()(uint8",
            "f(uint8",
            "f()(notatype)",
            "f() returns uint8",
        ] {
            assert!(parse_function_query(text).is_err(), "{text}");
        }
    }

    #[test]
    fn full_signatures_build_a_function_without_an_abi() {
        let query = parse_function_query("balanceOf(address)(uint256)").unwrap();
        let function = function_from_query(&query).unwrap().unwrap();
        assert_eq!(function.selector().to_string(), "0x70a08231");
        assert_eq!(function.outputs.len(), 1);

        let query = parse_function_query("f((uint8,bool),string)((address,uint256))").unwrap();
        let function = function_from_query(&query).unwrap().unwrap();
        assert_eq!(function.inputs.len(), 2);
        assert_eq!(function.outputs.len(), 1);

        let query = parse_function_query("totalSupply").unwrap();
        assert!(function_from_query(&query).unwrap().is_none());
    }

    fn parse_call(argv: &[&str]) -> (Vec<String>, bool, String) {
        use clap::Parser;
        let cli = crate::cli::Cli::try_parse_from(argv).unwrap();
        match cli.command {
            crate::cli::Commands::Contract {
                action:
                    ContractCommands::Call {
                        args, human, block, ..
                    },
            } => (args, human, block),
            _ => panic!("expected contract call"),
        }
    }

    #[test]
    fn test_call_trailing_flags_not_swallowed() {
        let (args, human, block) = parse_call(&[
            "ethcli",
            "contract",
            "call",
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
            "balanceOf",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            "-H",
            "--block",
            "123",
        ]);
        assert_eq!(args, vec!["0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"]);
        assert!(human);
        assert_eq!(block, "123");
    }

    #[test]
    fn test_call_negative_number_args() {
        let (args, human, _) =
            parse_call(&["ethcli", "contract", "call", "0xabc", "foo", "-5", "7"]);
        assert_eq!(args, vec!["-5", "7"]);
        assert!(!human);
    }

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

    #[test]
    fn human_scaling_applies_only_to_amount_getters() {
        assert!(is_amount_getter("totalSupply"));
        assert!(is_amount_getter("balanceOf"));
        assert!(is_amount_getter("convertToAssets"));
        assert!(!is_amount_getter("decimals"));
        assert!(!is_amount_getter("nonces"));
        assert!(!is_amount_getter("getRoundData"));
    }

    #[test]
    fn implementation_answer_must_be_one_address_word() {
        let mut word = [0u8; 32];
        word[31] = 1;
        assert!(address_from_word(&word).is_some());
        assert!(address_from_word(&[word, word].concat()).is_none());
        let mut dirty = word;
        dirty[0] = 1;
        assert!(address_from_word(&dirty).is_none());
        assert!(address_from_word(&[0u8; 32]).is_none());
    }
}
