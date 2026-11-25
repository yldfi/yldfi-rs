//! Contract-related commands
//!
//! Fetch ABI, source code, creation info, and bytecode analysis for contracts

use super::OutputFormat;
use crate::bytecode::{
    analyze_bytecode, analyze_handler_checks, disassemble_bytecode, extract_selectors,
    infer_dispatcher, opcode_stats, BytecodeAnalysis, CallsBudget, RiskLevel, MAX_BYTECODE_SIZE,
};
use crate::config::{Chain, EndpointConfig};
use crate::etherscan::Client;
use crate::rpc::Endpoint;
use crate::utils::format::with_thousands_sep;
use alloy::dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt};
use alloy::primitives::{Address, B256, U256};
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

/// The implementation `resolve_proxy` follows for this address when asked
/// to follow (as `--follow-proxy` does): the same decision as fingerprint.
/// `None` when it is not a proxy, or when nothing can be followed without a
/// choice (a msg.sender-dependent proxy with several user-side targets; the
/// reason goes to stderr)
pub(crate) async fn followed_implementation<P: Provider>(
    provider: &P,
    address: Address,
    follow: bool,
) -> Option<Address> {
    let block: alloy::eips::BlockId = alloy::eips::BlockNumberOrTag::Latest.into();
    let code = provider.get_code_at(address).block_id(block).await.ok()?;
    let opts = super::decompile::FollowOpts {
        follow_proxy: follow,
        target: None,
    };
    match super::decompile::resolve_proxy(provider, address, &code, block, &opts, true).await {
        Ok(d) => {
            if d.followed.is_none() {
                if let Some(r) = d.proxy_slot.as_ref().and_then(|v| v["reason"].as_str()) {
                    eprintln!("Note: not following the proxy: {r}");
                }
            }
            d.followed.map(|(a, _, _)| a)
        }
        Err(e) => {
            eprintln!("Note: not following the proxy: {e}");
            None
        }
    }
}

/// Try to detect if a contract is a proxy and return the implementation
/// address, by the shared proxy decision (followed as with --follow-proxy)
async fn detect_proxy_implementation<P: Provider>(
    provider: &P,
    address: Address,
) -> Option<Address> {
    followed_implementation(provider, address, true).await
}

/// `implementation()` at a given block
pub(crate) async fn call_implementation_function_at<P: Provider>(
    provider: &P,
    address: Address,
    block: alloy::eips::BlockId,
) -> Option<Address> {
    let tx = alloy::rpc::types::TransactionRequest::default()
        .to(address)
        .input(vec![0x5c, 0x60, 0xda, 0x1b].into());
    let result = provider.call(tx).block(block).await.ok()?;
    address_from_word(&result)
}

/// An address returned as a single ABI word: exactly 32 bytes, left-padded
/// with zeros. Anything else (a struct, a string, two values) is not an
/// `implementation()` answer.
pub(crate) fn address_from_word(result: &[u8]) -> Option<Address> {
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

    /// Read a storage variable by name, decoded, using the verified storage layout
    ///
    /// The layout (variable names, slots, offsets and types) comes from
    /// Sourcify. Mapping keys, array indexes and struct members follow the
    /// variable, either as extra arguments or in the path itself. For a proxy
    /// the implementation's layout is used and the proxy's storage is read.
    ///
    /// Examples:
    ///   ethcli contract storage 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --list
    ///   ethcli contract storage <usdc> totalSupply_
    ///   ethcli contract storage <usdc> balanceAndBlacklistStates <holder>
    ///   ethcli contract storage <usdc> allowed <owner> <spender> --block 2025-01-01
    ///   ethcli contract storage <addr> "positions[3].liquidity" --json
    Storage {
        /// Contract address (a proxy is followed to its implementation's layout)
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Variable name, optionally with a path: `name`, `name.member`, `name[key][index]`
        #[arg(value_name = "VARIABLE", required_unless_present = "list")]
        variable: Option<String>,

        /// Mapping keys, array indexes and struct member names, in order
        #[arg(value_name = "KEY", allow_negative_numbers = true)]
        keys: Vec<String>,

        /// List the storage layout (name, slot, offset, type) instead of reading
        #[arg(long)]
        list: bool,

        /// Block number, tag, date, @timestamp or age (e.g. 7d)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
        block: String,

        /// Custom RPC URL (overrides config)
        #[arg(long, value_name = "URL")]
        rpc_url: Option<String>,

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

    /// Decompile with the optional external evmdec, cross-checked first
    ///
    /// ethcli fetches the runtime code (following proxies like `contract
    /// inspect`) and runs only `evmdec decompile <file> --json`, with an empty
    /// environment and a hard timeout. Output starts with a cross-check against
    /// ethcli's own facts (evmole selectors and types, sensitive opcode
    /// positions); evmdec's pseudo-Solidity follows, labelled heuristic.
    /// Without evmdec it prints install steps, ethcli's facts and a Dedaub link.
    ///
    /// Examples:
    ///   ethcli contract decompile 0x6b75d8AF000000e20B7a7DDf000Ba900b4009A80
    ///   ethcli contract decompile 0x... --timeout 120 --json
    Decompile(super::decompile::DecompileArgs),

    /// Compiler facts from runtime code: version, metadata hash, EVM version
    ///
    /// Reads the CBOR metadata (compiler and version, ipfs/bzzr hash), the
    /// lowest EVM version the opcodes need (PUSH0, MCOPY, TLOAD, ...), a
    /// legacy/via-IR guess from the code's shape, and an upper bound on
    /// address-shaped immutables.
    ///
    /// Examples:
    ///   ethcli contract fingerprint 0xdb5889e35e379ef0498aae126fc2cce1fbd23216
    #[command(visible_alias = "fp")]
    Fingerprint(super::decompile::FingerprintArgs),

    /// Compare a compiled artifact with the deployed code
    ///
    /// Takes a forge artifact (out/<File>.sol/<Name>.json) or a solc
    /// standard-JSON output. Immutables and library addresses are masked
    /// (their deployed values reported), metadata is compared separately.
    /// Verdict: exact, partial (equal ignoring metadata; NOT a verification),
    /// partial_unverified_metadata (as partial, but the deployed metadata
    /// bytes hold a JUMPDEST and effectful opcodes that might be reachable)
    /// or mismatch, with the first differing offset, the function it falls in
    /// and a per-function table. Build with `forge build --ast` for
    /// source-level function mapping and the inline-assembly share.
    ///
    /// Examples:
    ///   ethcli contract match 0x... --artifact out/Router.sol/Router.json
    ///   ethcli contract match 0x... --artifact out.json --contract Router --creation-tx 0x...
    Match(super::decompile::MatchArgs),

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

        /// Leave out the call-site summary and evmole's inferred storage
        /// layout (faster on very large contracts)
        #[arg(long)]
        no_inference: bool,

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

    /// Call sites and access control recovered from bytecode (inferred)
    ///
    /// For every CALL, DELEGATECALL, STATICCALL, CALLCODE, CREATE and CREATE2:
    /// the pc, the function (dispatcher branch) it runs in, and where the
    /// target, outgoing selector, value and first arguments come from
    /// (constant, storage slot, calldata offset, msg.sender, return of an
    /// earlier call, or unresolved). Per function: ORIGIN and CALLER uses,
    /// guards (msg.sender / tx.origin / address(this) compared with a
    /// constant, slot or mapping), DELEGATECALL, SELFDESTRUCT, CREATE and
    /// SSTORE sites. A coverage section says what was analysed and what was
    /// not; nothing unresolved is guessed.
    ///
    /// Examples:
    ///   ethcli contract calls 0x88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640
    ///   ethcli contract calls 0x... --lookup --json
    ///   ethcli contract calls 0x... --block 21000000 --follow-proxy
    Calls {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Block number (default: latest)
        #[arg(long, short)]
        block: Option<String>,

        /// If proxy detected, analyze the implementation contract instead
        #[arg(long)]
        follow_proxy: bool,

        /// Name selectors from the signature database (guesses)
        #[arg(long, short)]
        lookup: bool,

        /// Opcode step budget over all paths
        #[arg(long, value_name = "N", default_value_t = 4_000_000)]
        max_steps: u64,

        /// Time budget in seconds (results are partial when it is hit)
        #[arg(long, value_name = "SECS", default_value_t = 15)]
        timeout: u64,

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

    /// Verified contracts with the same (or similar) runtime code
    ///
    /// Asks Etherscan (similar match), Blockscout (verified twin) and
    /// Sourcify (verification of the address itself), then reads each
    /// candidate's code and says whether it is byte-for-byte the same.
    /// Sourcify has no public lookup by code hash.
    ///
    /// --submit-similarity asks Sourcify to verify the address against
    /// similar verified contracts. That is a write: on success Sourcify
    /// publishes a verified match for the address.
    ///
    /// Examples:
    ///   ethcli contract twins 0x51c72848c68a965f66fa7a88855f9f7784502a7f
    ///   ethcli contract twins 0x... --json
    Twins {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Submit a Sourcify similarity verification (publishes a match on
        /// Sourcify; not read-only)
        #[arg(long)]
        submit_similarity: bool,

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
            if !quiet {
                if origin != crate::etherscan::fallback::Origin::Etherscan {
                    eprintln!("Not available from Etherscan; source from {origin}");
                }
                eprintln!(
                    "Verification match: {} (exact = code and metadata hash match; \
                     partial = code matches, metadata differs)",
                    origin.match_type()
                );
            }
            // Etherscan can serve the source of another address whose code is
            // similar; say so, since it was not verified for this address
            // Also in stdout (and JSON), so MCP callers see it
            let mut source_note: Option<String> = None;
            let mut verified_for: Option<String> = None;
            if origin == crate::etherscan::fallback::Origin::Etherscan {
                match etherscan_similar_match(&client, address).await {
                    Ok(Some(similar)) => {
                        source_note = Some(format!(
                            "Etherscan serves this source through a similar match: it was \
                             verified for {similar}, not for {address}. Check `ethcli contract \
                             twins {address}` for whether the code is identical."
                        ));
                        verified_for = Some(similar);
                    }
                    Ok(None) => {}
                    Err(e) => {
                        source_note = Some(format!(
                            "Could not tell whether Etherscan's source was verified for this \
                             address or borrowed from a similar contract ({e})"
                        ));
                    }
                }
                if let (Some(note), false) = (&source_note, quiet) {
                    eprintln!("Note: {note}");
                }
            }

            if *list || file.is_some() || function.is_some() || grep.is_some() {
                if let (Some(note), true) = (&source_note, format.is_table()) {
                    println!("// Note: {note}");
                }
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
                    verified_for.as_deref(),
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
                // Etherscan gives viaIR only inside standard-JSON settings;
                // when it is missing, ask Sourcify / Blockscout for theirs
                let mut extra_settings: Option<(serde_json::Value, String)> = None;
                if items
                    .iter()
                    .any(|i| compiler_settings_json(i, None)["via_ir"].is_null())
                {
                    let checksum = Address::from_str(address)?.to_checksum(None);
                    if let crate::etherscan::fallback::Fallback::Found(v) =
                        crate::etherscan::fallback::find_verified(
                            &reqwest::Client::new(),
                            client.chain().chain_id(),
                            &checksum,
                        )
                        .await
                    {
                        if let Some(s) = v.settings.clone() {
                            extra_settings = Some((s, v.origin.to_string()));
                        }
                    }
                }
                let extra = extra_settings.as_ref().map(|(s, o)| (s, o.as_str()));

                if !format.is_table() {
                    let contracts: Vec<serde_json::Value> = items
                        .iter()
                        .map(|item| {
                            let files: Vec<serde_json::Value> =
                                source_files(std::slice::from_ref(item))
                                    .into_iter()
                                    .map(|f| serde_json::json!({"path": f.path, "content": f.content}))
                                    .collect();
                            serde_json::json!({
                                "name": item.contract_name,
                                "compiler": compiler_settings_json(item, extra),
                                "license": (!item.license_type.is_empty()).then_some(&item.license_type),
                                "files": files,
                            })
                        })
                        .collect();
                    let out = serde_json::json!({
                        "address": address,
                        "origin": origin.to_string(),
                        "verification_match": origin.match_type(),
                        "verified_for": verified_for,
                        "note": source_note,
                        "contracts": contracts,
                    });
                    println!("{}", serde_json::to_string_pretty(&out)?);
                    return Ok(());
                }

                if let Some(note) = &source_note {
                    println!("// Note: {note}");
                }
                for item in items {
                    let c = compiler_settings_json(&item, extra);
                    println!("// Contract: {}", item.contract_name);
                    println!("// Compiler: {}", item.compiler_version);
                    println!(
                        "// Optimization: {} (runs: {})",
                        if c["optimizer"]["enabled"].as_bool() == Some(true) {
                            "enabled"
                        } else {
                            "disabled"
                        },
                        c["optimizer"]["runs"]
                    );
                    println!(
                        "// EVM version: {}; viaIR: {}",
                        c["evm_version"].as_str().unwrap_or("not reported"),
                        match c["via_ir"].as_bool() {
                            Some(v) => v.to_string(),
                            None => "not reported".to_string(),
                        }
                    );
                    if let Some(libs) = c["libraries"].as_object().filter(|l| !l.is_empty()) {
                        let list: Vec<String> = libs
                            .iter()
                            .map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or("")))
                            .collect();
                        println!("// Libraries: {}", list.join(", "));
                    }
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

        ContractCommands::Storage {
            address,
            variable,
            keys,
            list,
            block,
            rpc_url,
            format,
        } => {
            storage_command(
                chain,
                address,
                variable.as_deref(),
                keys,
                *list,
                block,
                rpc_url.as_deref(),
                format,
                quiet,
            )
            .await?;
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
            let resolved_block = resolve_block_arg(chain, rpc_url.as_deref(), block, quiet).await?;
            let block: &String = &resolved_block;

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

        ContractCommands::Decompile(args) => {
            super::decompile::decompile(chain, args, quiet).await?;
        }
        ContractCommands::Fingerprint(args) => {
            super::decompile::fingerprint_cmd(chain, args, quiet).await?;
        }
        ContractCommands::Match(args) => {
            super::decompile::match_cmd(chain, args, quiet).await?;
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
            no_inference,
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

            // evmole's inferred facts (storage layout only when asked for
            // inference) and the call-site summary
            let with_storage = !*no_inference;
            let code = analysis_bytecode.clone();
            let facts =
                run_blocking(move || crate::bytecode::infer_facts(&code, with_storage)).await?;
            analysis.inferred = Some(serde_json::to_value(&facts)?);
            if !*no_inference {
                if !quiet {
                    eprintln!("Tracing call sites...");
                }
                let report = calls_report(
                    analysis_bytecode.clone(),
                    CallsBudget {
                        max_time: std::time::Duration::from_secs(10),
                        ..CallsBudget::default()
                    },
                )
                .await?;
                analysis.calls = Some(report.summary());
            }

            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&analysis)?);
            } else {
                print_analysis_table(&analysis);
            }
        }

        ContractCommands::Calls {
            address,
            block,
            follow_proxy,
            lookup,
            max_steps,
            timeout,
            rpc_url,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
            let block_id = match block {
                Some(b) => {
                    let b = resolve_block_arg(chain, rpc_url.as_deref(), b, quiet).await?;
                    Some(super::rpc::parse_block_id(&b)?)
                }
                None => None,
            };
            let mut target = addr;
            let mut code = get_bytecode_at(&chain, rpc_url.as_deref(), addr, block_id).await?;
            if code.is_empty() {
                return Err(anyhow::anyhow!(
                    "No bytecode found at {address} (not a contract or empty)"
                ));
            }
            let proxy = crate::bytecode::detect_proxy(&code);
            if *follow_proxy {
                let implementation = match proxy.implementation {
                    Some(i) => Some(i),
                    None => get_implementation_from_storage(&chain, rpc_url.as_deref(), addr).await,
                };
                if let Some(i) = implementation {
                    if !quiet {
                        eprintln!("Following proxy to implementation: {i:#x}");
                    }
                    let impl_code =
                        get_bytecode_at(&chain, rpc_url.as_deref(), i, block_id).await?;
                    if !impl_code.is_empty() {
                        target = i;
                        code = impl_code;
                    }
                }
            } else if proxy.is_proxy && !quiet {
                eprintln!("Detected a proxy; use --follow-proxy to analyse the implementation");
            }
            if !quiet {
                eprintln!("Tracing call sites in {} bytes of code...", code.len());
            }
            let budget = CallsBudget {
                max_steps: *max_steps,
                max_time: std::time::Duration::from_secs(*timeout),
                ..CallsBudget::default()
            };
            let mut report = calls_report(code, budget).await?;
            let mut names = serde_json::Map::new();
            if *lookup {
                let selectors: std::collections::BTreeSet<String> = report
                    .functions
                    .iter()
                    .map(|f| f.function.clone())
                    .chain(
                        report
                            .call_sites
                            .iter()
                            .flat_map(|c| c.selector.iter())
                            .filter(|p| p.kind == "constant")
                            .filter_map(|p| p.value.clone()),
                    )
                    .filter(|s| s.len() == 10 && s.starts_with("0x"))
                    .collect();
                let client = signature_client()?;
                for selector in selectors {
                    if let Some(sig) = client.lookup_selector(&selector[2..]).await {
                        names.insert(selector, serde_json::json!(sig));
                    }
                }
                for f in &mut report.functions {
                    f.signature = names
                        .get(&f.function)
                        .and_then(|v| v.as_str())
                        .map(str::to_string);
                }
            }
            let mut out = serde_json::to_value(&report)?;
            out["address"] = serde_json::json!(addr.to_checksum(None));
            if target != addr {
                out["analysed"] = serde_json::json!(target.to_checksum(None));
            }
            if *lookup {
                out["selector_names"] = serde_json::json!(names);
                out["selector_names_note"] = serde_json::json!(
                    "Names come from the signature database: guesses, not verified"
                );
            }
            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{}", format_calls_table(&report, &names));
            }
        }

        ContractCommands::Twins {
            address,
            submit_similarity,
            rpc_url,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
            let code = get_bytecode_at(&chain, rpc_url.as_deref(), addr, None).await?;
            if code.is_empty() {
                return Err(anyhow::anyhow!(
                    "No bytecode found at {address} (not a contract or empty)"
                ));
            }
            let mut out = find_twins(chain, api_key.clone(), addr, &code, rpc_url.as_deref()).await;
            if *submit_similarity {
                eprintln!(
                    "WARNING: --submit-similarity asks Sourcify to verify {} against similar \
                     verified contracts. If it succeeds, Sourcify PUBLISHES a verified match \
                     for this address. This is not a read-only operation.",
                    addr.to_checksum(None)
                );
                let http = reqwest::Client::new();
                let result = crate::etherscan::fallback::sourcify_submit_similarity(
                    &http,
                    chain.chain_id(),
                    &addr.to_checksum(None),
                    std::time::Duration::from_secs(60),
                )
                .await;
                out["similarity_submission"] = match result {
                    Ok(job) => serde_json::json!({ "submitted": true, "job": job }),
                    Err(e) => serde_json::json!({ "submitted": false, "error": e }),
                };
            }
            if format.is_json() {
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                print!("{}", format_twins_table(&out));
            }
        }
    }

    Ok(())
}

/// Run CPU-bound analysis off the async runtime
async fn run_blocking<T: Send + 'static>(
    f: impl FnOnce() -> T + Send + 'static,
) -> anyhow::Result<T> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| anyhow::anyhow!("analysis task failed: {e}"))
}

/// Run the call-site analysis on a blocking thread. evmole's own passes have
/// no budget, so the whole job gets a deadline a little past the
/// interpreter's.
async fn calls_report(
    code: Vec<u8>,
    budget: CallsBudget,
) -> anyhow::Result<crate::bytecode::CallsReport> {
    let deadline = budget.max_time + std::time::Duration::from_secs(10);
    tokio::time::timeout(
        deadline,
        // Own thread with a large stack: values are depth-capped, so this is
        // defence in depth against recursion on hostile code
        run_blocking(move || {
            std::thread::Builder::new()
                .name("calls-analysis".into())
                .stack_size(64 * 1024 * 1024)
                .spawn(move || crate::bytecode::analyze_calls(&code, budget))
                .map_err(|e| anyhow::anyhow!("could not start the analysis thread: {e}"))?
                .join()
                .map_err(|_| anyhow::anyhow!("bytecode analysis panicked"))
        }),
    )
    .await
    .map_err(|_| {
        anyhow::anyhow!(
            "bytecode analysis did not finish within {} s",
            deadline.as_secs()
        )
    })??
}

/// Runtime code at a block (latest when `None`)
async fn get_bytecode_at(
    chain: &Chain,
    rpc_url: Option<&str>,
    address: Address,
    block: Option<alloy::eips::BlockId>,
) -> anyhow::Result<Vec<u8>> {
    let candidates = read_candidates(chain, rpc_url)?;
    crate::rpc::with_failover(candidates, |endpoint| async move {
        let call = endpoint.provider().get_code_at(address);
        let call = match block {
            Some(b) => call.block_id(b),
            None => call,
        };
        let code = call
            .await
            .map_err(|e| anyhow::anyhow!("Failed to fetch bytecode: {}", e))?;
        Ok(code.to_vec())
    })
    .await
}

/// Verified contracts with the same or similar runtime code, and how the
/// address itself is verified
async fn find_twins(
    chain: Chain,
    api_key: Option<String>,
    address: Address,
    code: &[u8],
    rpc_url: Option<&str>,
) -> serde_json::Value {
    use crate::etherscan::fallback;
    use alloy::primitives::keccak256;
    let checksum = address.to_checksum(None);
    let code_hash = keccak256(code);
    let http = reqwest::Client::new();
    let mut lookups = Vec::new();
    let mut verification = Vec::new();
    // (candidate, where it came from)
    let mut candidates: Vec<(Address, String)> = Vec::new();

    // Etherscan: getsourcecode carries a SimilarMatch address for contracts
    // it verified through a similar contract
    match Client::new(chain, api_key) {
        Ok(client) => {
            let result = client
                .explorer_get(&[
                    ("module", "contract".to_string()),
                    ("action", "getsourcecode".to_string()),
                    ("address", checksum.clone()),
                ])
                .await;
            match result {
                Ok(v) => {
                    let first = v.get(0).cloned().unwrap_or_default();
                    let similar = first["SimilarMatch"]
                        .as_str()
                        .unwrap_or_default()
                        .to_string();
                    let verified = first["SourceCode"].as_str().is_some_and(|s| !s.is_empty());
                    lookups.push(serde_json::json!({
                        "source": "etherscan",
                        "method": "getsourcecode SimilarMatch",
                        // Etherscan serves source either verified for this
                        // address or borrowed from a similar one
                        "has_source": verified,
                        "verified_for_this_address": verified && similar.is_empty(),
                        "similar_match": (!similar.is_empty()).then_some(&similar),
                        "field_present": first.get("SimilarMatch").is_some(),
                    }));
                    if let Ok(a) = Address::from_str(&similar) {
                        candidates.push((a, "etherscan SimilarMatch".into()));
                    }
                }
                Err(e) => lookups.push(serde_json::json!({
                    "source": "etherscan", "error": e,
                })),
            }
        }
        Err(e) => lookups.push(serde_json::json!({
            "source": "etherscan", "error": e.to_string(),
        })),
    }

    // Blockscout: verified_twin_address_hash and its own match flags
    match fallback::blockscout_lookup(&http, chain.chain_id(), &checksum).await {
        Some((base, value)) => {
            let m = fallback::blockscout_match_type(&value);
            let twin = value["verified_twin_address_hash"]
                .as_str()
                .map(str::to_string);
            lookups.push(serde_json::json!({
                "source": "blockscout",
                "method": "smart-contracts verified_twin_address_hash",
                "host": base,
                "verified_twin": twin,
            }));
            if let Some(a) = twin.as_deref().and_then(|t| Address::from_str(t).ok()) {
                candidates.push((a, "blockscout verified twin".into()));
            }
            verification.push(m);
        }
        None => lookups.push(serde_json::json!({
            "source": "blockscout",
            "note": "no public Blockscout API for this chain, or it did not answer",
        })),
    }

    // Sourcify: the address itself. There is no public lookup by code hash.
    match fallback::sourcify_lookup(&http, chain.chain_id(), &checksum).await {
        Some(m) => verification.push(m),
        None => lookups.push(serde_json::json!({
            "source": "sourcify", "error": "Sourcify did not answer",
        })),
    }
    lookups.push(serde_json::json!({
        "source": "sourcify",
        "method": "none",
        "note": "Sourcify v2 has no public lookup by runtime code hash; \
                 `--submit-similarity` asks it to verify against similar code (a write)",
    }));

    // Compare each candidate's code with ours
    candidates.sort();
    candidates.dedup_by(|a, b| a.0 == b.0);
    let mut twins = Vec::new();
    for (candidate, from) in candidates {
        if candidate == address {
            continue;
        }
        let twin_code = get_bytecode_at(&chain, rpc_url, candidate, None).await;
        let basis = match &twin_code {
            Ok(c) if keccak256(c) == code_hash => "exact_runtime_code",
            Ok(c) if c.is_empty() => "no_code",
            Ok(_) => "similar_runtime_code",
            Err(_) => "code_not_read",
        };
        twins.push(serde_json::json!({
            "address": candidate.to_checksum(None),
            "found_by": from,
            "basis": basis,
            "code_hash": twin_code.ok().map(|c| format!("{:#x}", keccak256(&c))),
        }));
    }

    serde_json::json!({
        "address": checksum,
        "chain": chain.display_name(),
        "code_hash": format!("{code_hash:#x}"),
        "code_size": code.len(),
        "verification": verification,
        "twins": twins,
        "lookups": lookups,
    })
}

fn format_twins_table(out: &serde_json::Value) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "Address:   {}", out["address"].as_str().unwrap_or("?"));
    let _ = writeln!(s, "Code hash: {}", out["code_hash"].as_str().unwrap_or("?"));
    for m in out["verification"].as_array().into_iter().flatten() {
        let _ = writeln!(
            s,
            "Verified on {}: {}",
            m["source"].as_str().unwrap_or("?"),
            m["match_type"].as_str().unwrap_or("?")
        );
    }
    let twins = out["twins"].as_array().cloned().unwrap_or_default();
    if twins.is_empty() {
        let _ = writeln!(s, "Twins:     none found");
    }
    for t in twins {
        let _ = writeln!(
            s,
            "Twin:      {} ({}, found by {})",
            t["address"].as_str().unwrap_or("?"),
            t["basis"].as_str().unwrap_or("?"),
            t["found_by"].as_str().unwrap_or("?")
        );
    }
    if let Some(sub) = out.get("similarity_submission") {
        let _ = writeln!(s, "Sourcify similarity submission: {sub}");
    }
    s
}

fn format_calls_table(
    report: &crate::bytecode::CallsReport,
    names: &serde_json::Map<String, serde_json::Value>,
) -> String {
    use std::fmt::Write;
    let name = |sel: &str| -> String {
        match names.get(sel).and_then(|v| v.as_str()) {
            Some(n) => format!("{sel} {n}?"),
            None => sel.to_string(),
        }
    };
    let list = |ps: &[crate::bytecode::Provenance]| -> String {
        if ps.is_empty() {
            return "-".into();
        }
        ps.iter()
            .map(|p| p.describe())
            .collect::<Vec<_>>()
            .join(" | ")
    };
    let mut s = String::new();
    let _ = writeln!(
        s,
        "Inferred from bytecode; unresolved means unknown, not safe.\n"
    );
    let _ = writeln!(s, "CALL SITES ({})", report.call_sites.len());
    for c in &report.call_sites {
        let fns: Vec<String> = c.functions.iter().map(|f| name(f)).collect();
        let _ = writeln!(s, "  pc {:>6} {:<12} in {}", c.pc, c.kind, fns.join(", "));
        if !c.target.is_empty() {
            let _ = writeln!(s, "      target:   {}", list(&c.target));
        }
        if !c.selector.is_empty() {
            let sels: Vec<String> = c
                .selector
                .iter()
                .map(|p| match (&p.kind[..], &p.value) {
                    ("constant", Some(v)) => name(v),
                    _ => p.describe(),
                })
                .collect();
            let _ = writeln!(s, "      selector: {}", sels.join(" | "));
        }
        if !c.value.is_empty() {
            let _ = writeln!(s, "      value:    {}", list(&c.value));
        }
        for (i, a) in c.args.iter().enumerate() {
            let _ = writeln!(s, "      arg{i}:     {}", list(a));
        }
    }
    let _ = writeln!(s, "\nFUNCTIONS");
    for f in &report.functions {
        let _ = writeln!(s, "  {}", name(&f.function));
        let pcs = |label: &str, v: &[usize], s: &mut String| {
            if !v.is_empty() {
                let _ = writeln!(s, "      {label:<13} {v:?}");
            }
        };
        pcs("ORIGIN", &f.origin_pcs, &mut s);
        pcs("CALLER", &f.caller_pcs, &mut s);
        pcs("calls", &f.call_pcs, &mut s);
        pcs("DELEGATECALL", &f.delegatecall_pcs, &mut s);
        pcs("SELFDESTRUCT", &f.selfdestruct_pcs, &mut s);
        pcs("CREATE", &f.create_pcs, &mut s);
        for g in &f.guards {
            let _ = writeln!(
                s,
                "      guard pc {}: {} {} {}",
                g.pc,
                g.subject,
                g.check,
                g.against.describe()
            );
        }
        for st in &f.sstores {
            let _ = writeln!(s, "      SSTORE pc {}: {}", st.pc, st.slot.describe());
        }
    }
    let ue: Vec<String> = report
        .opcode_sites
        .iter()
        .filter(|o| !o.reached && !o.in_metadata)
        .map(|o| format!("{}@{}", o.opcode, o.pc))
        .collect();
    // Sites in the trailing metadata (which holds no JUMPDEST) are data, not
    // code: they are counted by a note, not listed as unreached code
    let c = &report.coverage;
    let _ = writeln!(s, "\nCOVERAGE");
    let _ = writeln!(
        s,
        "  CFG blocks analysed {}/{} (+{} outside evmole's CFG), unresolved jumps {}, unresolved call sites {}",
        c.cfg_blocks_analysed,
        c.cfg_reachable_blocks,
        c.extra_blocks_analysed,
        c.unresolved_jumps.len(),
        c.unresolved_call_sites
    );
    let _ = writeln!(
        s,
        "  steps {}/{}, {} ms, states pruned {}, budget hit: {}, complete: {}",
        c.steps,
        c.step_limit,
        c.elapsed_ms,
        c.states_pruned,
        c.step_budget_hit || c.time_budget_hit,
        c.complete
    );
    if !ue.is_empty() {
        let _ = writeln!(s, "  sensitive opcodes not reached: {}", ue.join(", "));
    }
    for n in &report.notes {
        let _ = writeln!(s, "  note: {n}");
    }
    s
}

/// Turn a `--block` given as a time (a date, @timestamp or age) into a block
/// number; anything else is returned unchanged
pub(crate) async fn resolve_block_arg(
    chain: Chain,
    rpc_url: Option<&str>,
    block: &str,
    quiet: bool,
) -> anyhow::Result<String> {
    if crate::rpc::block_time::parse_time_spec(block, u64::MAX).is_none() {
        return Ok(block.to_string());
    }
    let probe = match rpc_url {
        Some(url) => Endpoint::new(EndpointConfig::new(url.to_string()), 30, None)?,
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
    Ok(resolved.number.to_string())
}

/// Storage reads at one block, cached so packed slots are fetched once
struct RpcSlotReader<'a, P> {
    provider: &'a P,
    address: Address,
    block: alloy::eips::BlockId,
    cache: std::sync::Mutex<std::collections::HashMap<U256, U256>>,
}

impl<P: Provider + Sync> crate::storage_layout::SlotReader for RpcSlotReader<'_, P> {
    fn read(
        &self,
        slot: U256,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = anyhow::Result<U256>> + Send + '_>>
    {
        Box::pin(async move {
            if let Some(v) = self.cache.lock().expect("cache lock").get(&slot) {
                return Ok(*v);
            }
            let v = self
                .provider
                .get_storage_at(self.address, slot)
                .block_id(self.block)
                .await
                .map_err(|e| anyhow::anyhow!("eth_getStorageAt failed: {e}"))?;
            self.cache.lock().expect("cache lock").insert(slot, v);
            Ok(v)
        })
    }
}

const NO_LAYOUT_HINT: &str =
    "Read the slot directly instead: `ethcli rpc storage <address> <slot>`, \
with `ethcli cast index <key_type> <key> <slot>` for a mapping entry's slot";

#[allow(clippy::too_many_arguments)]
async fn storage_command(
    chain: Chain,
    address: &str,
    variable: Option<&str>,
    keys: &[String],
    list: bool,
    block: &str,
    rpc_url: Option<&str>,
    format: &OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    use crate::etherscan::fallback::{storage_layout, LayoutLookup};
    use crate::storage_layout::Layout;
    use alloy::eips::{BlockId, BlockNumberOrTag};

    let addr = Address::from_str(address).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;
    let block = resolve_block_arg(chain, rpc_url, block, quiet).await?;
    let endpoint = if let Some(url) = rpc_url {
        Endpoint::new(EndpointConfig::new(url.to_string()), 30, None)?
    } else {
        let target_block = crate::utils::parse_block_number(&block).ok();
        crate::rpc::selector::get_rpc_endpoint_at(chain, target_block)?
    };
    let provider = endpoint.provider();

    // Pin one block for the proxy check and every slot read
    let block_number = match super::rpc::parse_block_id(&block)? {
        BlockId::Number(BlockNumberOrTag::Number(n)) => n,
        other => {
            provider
                .get_block(other)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get block {block}: {e}"))?
                .ok_or_else(|| anyhow::anyhow!("block {block} not found"))?
                .header
                .number
        }
    };
    let block_id: BlockId = block_number.into();

    let code = provider
        .get_code_at(addr)
        .block_id(block_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch code: {e}"))?;
    if code.is_empty() {
        anyhow::bail!(
            "{} has no code at block {block_number}",
            addr.to_checksum(None)
        );
    }
    // The same proxy decision as fingerprint: a followed proxy's
    // implementation layout first; a held one's value only after the
    // address's own layout
    let decision = super::decompile::resolve_proxy(
        provider,
        addr,
        &code,
        block_id,
        &super::decompile::FollowOpts::default(),
        true,
    )
    .await?;
    let mut candidates: Vec<(Address, &str)> = Vec::new();
    match (&decision.followed, &decision.proxy_slot) {
        (Some((imp, kind, _)), _) => {
            if !quiet {
                eprintln!(
                    "{kind}; using the layout of implementation {}",
                    imp.to_checksum(None)
                );
            }
            candidates.push((*imp, "implementation"));
            candidates.push((addr, "address"));
        }
        (None, Some(slot)) => {
            candidates.push((addr, "address"));
            if let Some(imp) = slot["value"]
                .as_str()
                .and_then(|v| Address::from_str(v).ok())
            {
                candidates.push((imp, "implementation (not confirmed)"));
            }
        }
        (None, None) => candidates.push((addr, "address")),
    }

    let http = reqwest::Client::builder()
        .user_agent(concat!("ethcli/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let mut reasons = Vec::new();
    let mut found = None;
    for (candidate, role) in &candidates {
        let key = candidate.to_checksum(None);
        match storage_layout(&http, chain.chain_id(), &key).await {
            LayoutLookup::Found(v) => {
                found = Some((*candidate, *role, Layout::parse(&v)?));
                break;
            }
            LayoutLookup::NoLayout => reasons.push(format!(
                "{key} ({role}) is verified on Sourcify but has no storage layout (older compiler or Vyper)"
            )),
            LayoutLookup::NotVerified => {
                reasons.push(format!("{key} ({role}) is not verified on Sourcify"))
            }
            LayoutLookup::Unavailable(e) => {
                reasons.push(format!("Sourcify lookup for {key} ({role}) failed: {e}"))
            }
        }
    }
    let Some((layout_addr, layout_role, layout)) = found else {
        anyhow::bail!(
            "No storage layout available.\n  {}\n{NO_LAYOUT_HINT}",
            reasons.join("\n  ")
        );
    };
    if !quiet && layout_role == "address" && candidates.len() > 1 {
        eprintln!("Using the layout of the address itself");
    }

    if list {
        if format.is_table() {
            println!("{:<6} {:<6} {:<40} TYPE", "SLOT", "OFFSET", "NAME");
            for v in &layout.storage {
                println!(
                    "{:<6} {:<6} {:<40} {}",
                    v.slot.to_string(),
                    v.offset,
                    v.label,
                    layout.type_label(&v.type_id)
                );
            }
        } else {
            let rows: Vec<serde_json::Value> = layout
                .storage
                .iter()
                .map(|v| {
                    serde_json::json!({
                        "name": v.label,
                        "slot": v.slot.to_string(),
                        "offset": v.offset,
                        "type": layout.type_label(&v.type_id),
                        "contract": v.contract,
                    })
                })
                .collect();
            let out = serde_json::json!({
                "address": addr.to_checksum(None),
                "layout_from": layout_addr.to_checksum(None),
                "variables": rows,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        }
        return Ok(());
    }

    let variable = variable.ok_or_else(|| anyhow::anyhow!("give a variable name or --list"))?;
    let (name, mut steps) = crate::storage_layout::split_path(variable)?;
    steps.extend(keys.iter().cloned());
    let loc = layout.locate(&name, &steps)?;
    let reader = RpcSlotReader {
        provider,
        address: addr,
        block: block_id,
        cache: Default::default(),
    };
    layout.check_bounds(&reader, &loc).await?;
    let value = layout.decode(&reader, &loc).await?;
    let type_label = layout.type_label(&loc.type_id);
    let slot_hex = format!("{:#066x}", loc.slot);

    if format.is_table() {
        if !quiet {
            eprintln!(
                "{type_label} at slot {slot_hex} offset {}, block {block_number}",
                loc.offset
            );
        }
        match &value {
            serde_json::Value::String(s) => println!("{s}"),
            other => println!("{}", serde_json::to_string_pretty(other)?),
        }
    } else {
        let mut path = name.clone();
        for s in &steps {
            path.push_str(&format!("[{s}]"));
        }
        let out = serde_json::json!({
            "address": addr.to_checksum(None),
            "layout_from": layout_addr.to_checksum(None),
            "variable": path,
            "type": type_label,
            "slot": slot_hex,
            "offset": loc.offset,
            "block": block_number,
            "value": value,
        });
        println!("{}", serde_json::to_string_pretty(&out)?);
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

/// The implementation to follow for `--follow-proxy` on one endpoint, by
/// the shared proxy decision; RPC errors propagate so the caller can fail
/// over
async fn implementation_from_storage_on(
    endpoint: &Endpoint,
    proxy_address: Address,
) -> anyhow::Result<Option<Address>> {
    let provider = endpoint.provider();
    // Check the endpoint answers, so a dead one fails over instead of
    // reading as "not a proxy"
    provider
        .get_code_at(proxy_address)
        .await
        .map_err(|e| anyhow::anyhow!("eth_getCode failed: {}", e))?;
    Ok(followed_implementation(provider, proxy_address, true).await)
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
    if let Some(inferred) = &analysis.inferred {
        if let Some(compiler) = inferred["metadata"]["compiler"].as_str() {
            println!("  Compiler:       {compiler} (CBOR metadata)");
        }
        let cf = &inferred["control_flow"];
        println!(
            "  Control flow:   {} reachable / {} blocks, {} unresolved jumps (inferred)",
            cf["reachable_blocks"], cf["basic_blocks"], cf["unresolved_jumps"]
        );
        if let Some(slots) = inferred["storage"]["slots"].as_array() {
            println!("  Storage:        {} slots (inferred)", slots.len());
            for slot in slots.iter().take(12) {
                println!(
                    "    {} +{} {}",
                    slot["slot"].as_str().unwrap_or("?"),
                    slot["offset"],
                    slot["type"].as_str().unwrap_or("?")
                );
            }
        }
    }
    if let Some(calls) = &analysis.calls {
        println!(
            "  Call sites:     {} (calldata-controlled targets {}, msg.sender targets {}) (inferred)",
            calls["call_sites"], calls["calldata_controlled_targets"], calls["msg_sender_targets"]
        );
        if let Some(targets) = calls["hardcoded_targets"].as_array() {
            for t in targets.iter().take(10) {
                println!("    hard-coded target {}", t.as_str().unwrap_or("?"));
            }
        }
        for g in calls["guards"].as_array().into_iter().flatten().take(12) {
            println!(
                "    guard in {} at pc {}: {}",
                g["function"].as_str().unwrap_or("?"),
                g["pc"],
                g["check"].as_str().unwrap_or("?")
            );
        }
        println!(
            "    ORIGIN at pcs {}; coverage {}/{} CFG blocks, complete: {}",
            calls["origin_pcs"],
            calls["coverage"]["cfg_blocks_analysed"],
            calls["coverage"]["cfg_reachable_blocks"],
            calls["coverage"]["complete"]
        );
        println!("    (run `ethcli contract calls` for every site)");
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
pub(crate) async fn find_implementation<P: Provider>(
    provider: &P,
    address: Address,
    code: &[u8],
    block: alloy::eips::BlockId,
) -> Option<(&'static str, Address)> {
    if let Some(clone) = crate::bytecode::detect_immutable_args_clone(code) {
        if let Some(found) = clone.implementation {
            return Some((clone.kind, found));
        }
    }
    if let Some(found) = crate::bytecode::detect_proxy(code).implementation {
        return Some(("EIP-1167 minimal proxy", found));
    }
    let slot_kind = match storage_address(provider, address, eip1967_impl_slot(), block).await {
        Some(found) => Some(("EIP-1967 proxy", found)),
        None => storage_address(provider, address, oz_impl_slot(), block)
            .await
            .map(|found| ("OpenZeppelin legacy proxy", found)),
    };
    if let Some(found) = slot_kind {
        return Some(found);
    }
    if let Some(beacon) = storage_address(provider, address, eip1967_beacon_slot(), block).await {
        if let Some(found) = call_implementation_function_at(provider, beacon, block).await {
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
    call_implementation_function_at(provider, address, block)
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
    // The same proxy decision as fingerprint: a held proxy is reported, and
    // the functions are then the address's own
    let decision = match super::decompile::resolve_proxy(
        provider,
        address,
        &code,
        block,
        &super::decompile::FollowOpts::default(),
        true,
    )
    .await
    {
        Ok(d) => d,
        Err(e) => {
            notes.push(format!("proxy check failed: {e}"));
            Default::default()
        }
    };
    let implementation = decision.followed.as_ref().map(|(a, _, _)| *a);
    let proxy_json = match (&decision.followed, &decision.proxy_slot) {
        (Some((imp, kind, _)), slot) => {
            let mut proxy = serde_json::json!({
                "kind": kind,
                "implementation": imp.to_checksum(None),
            });
            if let Some(slot) = slot {
                proxy["detail"] = slot.clone();
            }
            Some(proxy)
        }
        (None, Some(slot)) => {
            notes.push(format!(
                "Proxy not followed: {}; the functions are the address's own",
                slot["reason"].as_str().unwrap_or("not confirmed")
            ));
            Some(serde_json::json!({
                "kind": slot["kind"],
                "implementation": null,
                "followed": false,
                "held_value": slot["value"],
                "reason": slot["reason"],
                "candidates": slot.get("candidates").cloned().unwrap_or(serde_json::Value::Null),
                "force_with": slot["force_with"],
            }))
        }
        (None, None) => None,
    };
    if let Some(mut proxy) = proxy_json {
        if let Some(admin) = storage_address(provider, address, eip1967_admin_slot(), block).await {
            proxy["admin"] = serde_json::json!(admin.to_checksum(None));
        }
        if let Some(beacon) = storage_address(provider, address, eip1967_beacon_slot(), block).await
        {
            proxy["beacon"] = serde_json::json!(beacon.to_checksum(None));
        }
        out["proxy"] = proxy;
    }
    // Data appended to a clone's code, read from the address's own code
    if let Some(clone) = crate::bytecode::detect_immutable_args_clone(&code) {
        if out["proxy"].is_null() {
            out["proxy"] = serde_json::json!({"kind": clone.kind, "implementation": null});
        }
        out["proxy"]["immutable_args"] = clone.to_json();
    }
    let interface_target = implementation.unwrap_or(address);

    // Bytecode facts evmole infers (of the implementation, for a proxy)
    let inferred_code = if interface_target == address {
        code.to_vec()
    } else {
        provider
            .get_code_at(interface_target)
            .block_id(block)
            .await
            .map(|c| c.to_vec())
            .unwrap_or_default()
    };
    if !inferred_code.is_empty() {
        let for_facts = inferred_code.clone();
        match run_blocking(move || crate::bytecode::infer_facts(&for_facts, true)).await {
            Ok(facts) => {
                let mut value = serde_json::to_value(&facts)?;
                value["of"] = serde_json::json!(interface_target.to_checksum(None));
                out["inferred"] = value;
            }
            Err(e) => notes.push(format!("Bytecode inference failed: {e}")),
        }
    }

    // How the code is verified, and verified twins
    let twins = match tokio::time::timeout(
        std::time::Duration::from_secs(12),
        find_twins(chain, api_key.clone(), address, &code, rpc_url),
    )
    .await
    {
        Ok(t) => t,
        Err(_) => {
            notes.push("Verification and twin lookups timed out".to_string());
            serde_json::json!({})
        }
    };
    out["verification"] = twins["verification"].clone();
    out["twins"] = twins["twins"].clone();

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
            // No dispatcher: say whether reached code still reads calldata
            // (a fallback-only or packed-calldata executor)
            let reads = if functions.is_empty() && !target_code.is_empty() {
                let report = crate::bytecode::analyze_calls(&target_code, CallsBudget::default());
                crate::bytecode::metadata::calldata_reads(&target_code, &report.visited)
            } else {
                Vec::new()
            };
            out["interface"] = serde_json::json!({
                "source": "bytecode",
                "of": interface_target.to_checksum(None),
                "functions": functions,
            });
            if reads.is_empty() {
                notes.push(
                    "No verified ABI: selectors were recovered from bytecode, and any function \
                     names are guesses from the signature database"
                        .to_string(),
                );
            } else {
                out["interface"]["dispatcher"] =
                    serde_json::json!(crate::bytecode::metadata::FALLBACK_ONLY);
                out["interface"]["calldata_reads"] = serde_json::json!(reads);
                notes.push(
                    "No verified ABI and no selector dispatcher: reached code reads calldata \
                     directly (interface.calldata_reads), so the input is a custom encoding, \
                     not ABI calls"
                        .to_string(),
                );
            }
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
    for m in report["verification"].as_array().into_iter().flatten() {
        if let (Some(source), Some(kind)) = (text(&m["source"]), text(&m["match_type"])) {
            line(&mut out, "Verification", format!("{source}: {kind}"));
        }
    }
    for t in report["twins"].as_array().into_iter().flatten() {
        line(
            &mut out,
            "Verified twin",
            format!(
                "{} ({}, found by {})",
                text(&t["address"]).unwrap_or_default(),
                text(&t["basis"]).unwrap_or_default(),
                text(&t["found_by"]).unwrap_or_default()
            ),
        );
    }
    let inferred = &report["inferred"];
    if let Some(compiler) = text(&inferred["metadata"]["compiler"]) {
        let hash = text(&inferred["metadata"]["ipfs"])
            .map(|h| format!(", ipfs {h}"))
            .or_else(|| text(&inferred["metadata"]["bzzr"]).map(|h| format!(", {h}")))
            .unwrap_or_default();
        line(
            &mut out,
            "Compiler",
            format!("{compiler}{hash} (from the code's CBOR metadata)"),
        );
    }
    if let Some(blocks) = inferred["control_flow"]["reachable_blocks"].as_u64() {
        line(
            &mut out,
            "Control flow",
            format!(
                "{blocks} reachable blocks of {}, {} unresolved jumps (inferred)",
                inferred["control_flow"]["basic_blocks"]
                    .as_u64()
                    .unwrap_or(0),
                inferred["control_flow"]["unresolved_jumps"]
                    .as_u64()
                    .unwrap_or(0)
            ),
        );
    }
    if let Some(slots) = inferred["storage"]["slots"].as_array() {
        line(
            &mut out,
            "Storage",
            format!(
                "{} slots inferred from bytecode (types are guesses)",
                slots.len()
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
    let args = &report["proxy"]["immutable_args"];
    if let Some(hex) = text(&args["args_hex"]) {
        line(
            &mut out,
            "Immutable args",
            format!(
                "{} bytes (raw data): {hex}",
                args["args_length"].as_u64().unwrap_or(0)
            ),
        );
        for a in args["address_like"]["packed"]
            .as_array()
            .into_iter()
            .flatten()
        {
            line(
                &mut out,
                "  address-like",
                format!(
                    "offset {}: {}",
                    a["offset"],
                    a["address"].as_str().unwrap_or_default()
                ),
            );
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
    if let Some(dispatcher) = text(&report["interface"]["dispatcher"]) {
        let reads: Vec<String> = report["interface"]["calldata_reads"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|p| p.as_u64().map(|p| format!("{p:#x}")))
            .collect();
        line(
            &mut out,
            "Functions",
            format!(
                "{dispatcher} (code of {}; calldata read at {})",
                text(&report["interface"]["of"]).unwrap_or_default(),
                reads.join(", ")
            ),
        );
    } else if let Some(functions) = report["interface"]["functions"].as_array() {
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
/// Compiler settings of a verified contract: what the explorer's record
/// says, filled from its standard-JSON `settings` when it has them, and from
/// `extra` (another verifier's settings, with its name) for what is still
/// missing. `via_ir` is null when no source reports it
fn compiler_settings_json(
    item: &foundry_block_explorers::contract::Metadata,
    extra: Option<(&serde_json::Value, &str)>,
) -> serde_json::Value {
    let own = item.source_code.settings();
    let pick = |key: &str| -> Option<(&serde_json::Value, &'static str)> {
        if let Some(v) = own.and_then(|s| s.get(key)).filter(|v| !v.is_null()) {
            return Some((v, "explorer"));
        }
        extra
            .and_then(|(s, _)| s.get(key))
            .filter(|v| !v.is_null())
            .map(|v| (v, "other_verifier"))
    };
    let explorer_evm = match item.evm_version.trim() {
        v if !v.is_empty() && !v.eq_ignore_ascii_case("default") => Some(v.to_ascii_lowercase()),
        _ => pick("evmVersion")
            .and_then(|(v, _)| v.as_str())
            .map(str::to_string),
    };
    // "default" means the compiler's own default target
    let (evm_version, evm_version_from) = match explorer_evm {
        Some(v) => (Some(v), "explorer"),
        None => match solc_default_evm(&item.compiler_version) {
            Some(v) => (Some(v.to_string()), "compiler default"),
            None => (None, "not reported"),
        },
    };
    let via_ir = pick("viaIR");
    let optimizer = own.and_then(|s| s.get("optimizer"));
    let enabled = optimizer
        .and_then(|o| o["enabled"].as_bool())
        .unwrap_or(item.optimization_used == 1);
    let runs = optimizer
        .and_then(|o| o["runs"].as_u64())
        .unwrap_or(item.runs);
    // One shape: {"<file>:<Name>" or "<Name>": "0x<checksummed address>"}.
    // Etherscan gives "Name:0xaddr;Name2:0xaddr" (names only, addresses
    // sometimes without 0x); standard JSON {file: {name: addr}}
    let norm_addr = |a: &str| -> serde_json::Value {
        let a = a.trim();
        let with_0x = if a.starts_with("0x") || a.starts_with("0X") {
            a.to_string()
        } else {
            format!("0x{a}")
        };
        match Address::from_str(&with_0x) {
            Ok(addr) => serde_json::json!(addr.to_checksum(None)),
            Err(_) => serde_json::json!(a),
        }
    };
    let mut libraries = serde_json::Map::new();
    for part in item.library.split(';').filter(|p| !p.trim().is_empty()) {
        if let Some((name, addr)) = part.split_once(':') {
            libraries.insert(name.trim().to_string(), norm_addr(addr));
        }
    }
    if libraries.is_empty() {
        if let Some((libs, _)) = pick("libraries") {
            for (file, names) in libs.as_object().into_iter().flatten() {
                match names.as_object() {
                    Some(m) => {
                        for (name, addr) in m {
                            libraries.insert(
                                format!("{file}:{name}"),
                                norm_addr(addr.as_str().unwrap_or_default()),
                            );
                        }
                    }
                    None => {
                        libraries
                            .insert(file.clone(), norm_addr(names.as_str().unwrap_or_default()));
                    }
                }
            }
        }
    }
    // solc has the via-IR pipeline from 0.7.2; Vyper never had the option
    let predates_via_ir = {
        let v = item.compiler_version.trim_start_matches('v');
        let mut it = v
            .split(|c: char| !c.is_ascii_digit())
            .filter(|s| !s.is_empty())
            .map(|s| s.parse::<u64>().unwrap_or(0));
        let ver = (
            it.next().unwrap_or(0),
            it.next().unwrap_or(0),
            it.next().unwrap_or(0),
        );
        !item.compiler_version.to_ascii_lowercase().contains("vyper")
            && ver != (0, 0, 0)
            && ver < (0, 7, 2)
    };
    let (via_ir_value, via_ir_from) = match via_ir {
        Some((v, "explorer")) => (v.as_bool(), Some("explorer".to_string())),
        Some((v, _)) => (v.as_bool(), extra.map(|(_, o)| o.to_string())),
        None if predates_via_ir => (
            Some(false),
            Some("compiler version (before 0.7.2)".to_string()),
        ),
        // The explorer gave the standard-JSON settings and they have no
        // viaIR key: solc's default, off
        None if own.is_some() => (
            Some(false),
            Some("explorer settings (no viaIR key: solc default, off)".to_string()),
        ),
        None => (None, None),
    };
    serde_json::json!({
        "compiler_version": item.compiler_version,
        "evm_version": evm_version.unwrap_or_else(|| "default".to_string()),
        "evm_version_from": evm_version_from,
        "via_ir": via_ir_value,
        "via_ir_from": via_ir_from,
        "optimizer": {"enabled": enabled, "runs": runs},
        "libraries": libraries,
    })
}

/// The EVM version solc targets by default, by compiler version (solc's
/// changelog); `None` for Vyper or an unreadable version
fn solc_default_evm(version: &str) -> Option<&'static str> {
    if version.to_ascii_lowercase().contains("vyper") {
        return None;
    }
    let v = version.trim_start_matches('v');
    let mut it = v
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u64>().ok());
    let ver = (it.next()??, it.next()??, it.next()??);
    const TABLE: &[((u64, u64, u64), &str)] = &[
        ((0, 8, 30), "prague"),
        ((0, 8, 25), "cancun"),
        ((0, 8, 20), "shanghai"),
        ((0, 8, 18), "paris"),
        ((0, 8, 7), "london"),
        ((0, 8, 5), "berlin"),
        ((0, 5, 14), "istanbul"),
        ((0, 5, 5), "petersburg"),
        ((0, 4, 21), "byzantium"),
    ];
    Some(
        TABLE
            .iter()
            .find(|(t, _)| ver >= *t)
            .map(|(_, e)| *e)
            .unwrap_or("spuriousDragon"),
    )
}

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
/// Mark every element of a JSON array with the address the source was
/// verified for, when Etherscan served it through a similar match
fn with_verified_for(
    mut value: serde_json::Value,
    verified_for: Option<&str>,
) -> serde_json::Value {
    if let (Some(addr), Some(items)) = (verified_for, value.as_array_mut()) {
        for item in items {
            if let Some(obj) = item.as_object_mut() {
                obj.insert("source_verified_for".into(), serde_json::json!(addr));
            }
        }
    }
    value
}

/// Etherscan's `SimilarMatch` for an address: `Some(other)` when the source
/// it serves was verified for another address. One retry.
async fn etherscan_similar_match(client: &Client, address: &str) -> Result<Option<String>, String> {
    let params = [
        ("module", "contract".to_string()),
        ("action", "getsourcecode".to_string()),
        ("address", address.to_string()),
    ];
    let mut last = String::new();
    for attempt in 0..2 {
        if attempt > 0 {
            tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        }
        match client.explorer_get(&params).await {
            Ok(v) => {
                return Ok(v
                    .get(0)
                    .and_then(|r| r["SimilarMatch"].as_str())
                    .filter(|s| !s.is_empty())
                    .map(str::to_string))
            }
            Err(e) => last = e,
        }
    }
    Err(last)
}

fn print_source_slice(
    files: &[super::source_view::SourceFile],
    list: bool,
    file: Option<&str>,
    function: Option<&str>,
    grep: Option<&str>,
    format: OutputFormat,
    verified_for: Option<&str>,
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
            println!(
                "{}",
                serde_json::to_string_pretty(&with_verified_for(
                    serde_json::to_value(&listed)?,
                    verified_for
                ))?
            );
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
            println!(
                "{}",
                serde_json::to_string_pretty(&with_verified_for(
                    serde_json::to_value(&found)?,
                    verified_for
                ))?
            );
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
            println!(
                "{}",
                serde_json::to_string_pretty(&with_verified_for(
                    serde_json::to_value(&hits)?,
                    verified_for
                ))?
            );
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
        println!(
            "{}",
            serde_json::to_string_pretty(&with_verified_for(
                serde_json::to_value(&out)?,
                verified_for
            ))?
        );
    }
    Ok(())
}

#[cfg(test)]
mod compiler_settings_tests {
    use super::compiler_settings_json;
    use foundry_block_explorers::contract::Metadata;
    use serde_json::json;

    fn etherscan_item(source_code: &str, version: &str, evm: &str, library: &str) -> Metadata {
        serde_json::from_value(json!({
            "SourceCode": source_code,
            "ABI": "[]",
            "ContractName": "C",
            "CompilerVersion": version,
            "OptimizationUsed": "1",
            "Runs": "200",
            "ConstructorArguments": "",
            "EVMVersion": evm,
            "Library": library,
            "LicenseType": "MIT",
            "Proxy": "0",
            "Implementation": "",
            "SwarmSource": ""
        }))
        .unwrap()
    }

    #[test]
    fn default_evm_by_compiler() {
        use super::solc_default_evm;
        assert_eq!(solc_default_evm("v0.8.17+commit.8df45f5f"), Some("london"));
        assert_eq!(
            solc_default_evm("v0.8.24+commit.e11b9ed9"),
            Some("shanghai")
        );
        assert_eq!(
            solc_default_evm("0.4.18+commit.9cf6e910"),
            Some("spuriousDragon")
        );
        assert_eq!(
            solc_default_evm("v0.5.16+commit.9c3226ce"),
            Some("istanbul")
        );
        assert_eq!(solc_default_evm("vyper:0.3.10"), None);
        let item = etherscan_item("contract C {}", "v0.8.17+commit.8df45f5f", "Default", "");
        let c = compiler_settings_json(&item, None);
        assert_eq!(c["evm_version"], json!("london"));
        assert_eq!(c["evm_version_from"], json!("compiler default"));
    }

    /// Standard-JSON input carries viaIR and the optimizer; Etherscan's
    /// "Default" EVM version falls back to the settings
    #[test]
    fn settings_from_standard_json() {
        let src = r#"{{"language":"Solidity","sources":{"a.sol":{"content":"contract C {}"}},"settings":{"viaIR":true,"evmVersion":"cancun","optimizer":{"enabled":true,"runs":999},"libraries":{"a.sol":{"L":"0x0000000000000000000000000000000000000001"}}}}}"#;
        let c = compiler_settings_json(
            &etherscan_item(src, "v0.8.24+commit.e11b9ed9", "Default", ""),
            None,
        );
        assert_eq!(c["via_ir"], json!(true));
        assert_eq!(c["via_ir_from"], json!("explorer"));
        assert_eq!(c["evm_version"], json!("cancun"));
        assert_eq!(c["optimizer"], json!({"enabled": true, "runs": 999}));
        assert_eq!(
            c["libraries"]["a.sol:L"],
            json!("0x0000000000000000000000000000000000000001")
        );
        // Settings without a viaIR key: solc's default, off
        let src = r#"{{"language":"Solidity","sources":{"a.sol":{"content":"contract C {}"}},"settings":{"optimizer":{"enabled":false}}}}"#;
        let c = compiler_settings_json(
            &etherscan_item(src, "v0.8.24+commit.e11b9ed9", "Default", ""),
            None,
        );
        assert_eq!(c["via_ir"], json!(false));
    }

    /// A flat source says nothing about viaIR: another verifier's settings
    /// fill it, a pre-0.7.2 compiler cannot have it, and otherwise it stays
    /// null. Etherscan's `Library` string is parsed
    #[test]
    fn via_ir_when_the_explorer_does_not_say() {
        let item = etherscan_item(
            "contract C {}",
            "v0.8.17+commit.8df45f5f",
            "london",
            "Lib:0xabc",
        );
        let c = compiler_settings_json(&item, None);
        assert!(c["via_ir"].is_null());
        assert_eq!(c["evm_version"], json!("london"));
        assert_eq!(c["libraries"]["Lib"], json!("0xabc"));
        let other = json!({"viaIR": false});
        let c = compiler_settings_json(&item, Some((&other, "Sourcify (match)")));
        assert_eq!(c["via_ir"], json!(false));
        assert_eq!(c["via_ir_from"], json!("Sourcify (match)"));
        let old = etherscan_item("contract C {}", "v0.4.24+commit.e67f0147", "byzantium", "");
        let c = compiler_settings_json(&old, None);
        assert_eq!(c["via_ir"], json!(false));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_table_does_not_list_metadata_bytes_as_unreached() {
        // CALLER (0x33) as the metadata length byte, and in dead code before
        // it (after a REVERT, no JUMPDEST): only the latter is listed
        let meta = "a26469706673582212208b5d96ae75ef7e28d88d1c3820279a801b63c4e1319cce2a5166f9a221abe79564736f6c63430008150033";
        let code = hex::decode(format!("6080604052600080fd33{meta}")).unwrap();
        let report = crate::bytecode::analyze_calls(&code, CallsBudget::default());
        assert!(report.opcode_sites.iter().any(|o| o.in_metadata));
        let table = format_calls_table(&report, &serde_json::Map::new());
        let line = table
            .lines()
            .find(|l| l.contains("sensitive opcodes not reached"))
            .expect("unreached line");
        assert!(line.contains("CALLER@9"), "{line}");
        assert_eq!(line.matches('@').count(), 1, "{line}");
        assert!(table.contains("trailing metadata region"), "{table}");
    }

    #[test]
    fn similar_match_source_is_marked_in_json() {
        let listed = serde_json::json!([{"path": "A.sol", "lines": 3}]);
        let marked = with_verified_for(listed.clone(), Some("0xb4e1"));
        assert_eq!(marked[0]["source_verified_for"], "0xb4e1");
        assert_eq!(with_verified_for(listed.clone(), None), listed);
    }

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

        let executor = serde_json::json!({
            "address": "0x03", "chain": "Ethereum", "block": 1, "kind": "contract", "verified": false,
            "balance": {"wei": "0", "formatted": "0.0", "symbol": "ETH"},
            "interface": {"source": "bytecode", "of": "0x03", "functions": [],
                          "dispatcher": crate::bytecode::metadata::FALLBACK_ONLY,
                          "calldata_reads": [55, 60]},
            "notes": []
        });
        let out = format_inspect_table(&executor);
        assert!(
            out.contains("no selector dispatcher: fallback-only / packed-calldata executor (code of 0x03; calldata read at 0x37, 0x3c)"),
            "{out}"
        );
        assert!(!out.contains("Functions:      0"), "{out}");
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
