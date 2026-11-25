//! RPC commands - direct Ethereum RPC calls
//!
//! Commands for reading blockchain state

use crate::cli::OutputFormat;
use crate::config::{Chain, EndpointConfig};
use crate::rpc::Endpoint;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use clap::Subcommand;
use std::str::FromStr;

/// Help text shared by every `--block` that also accepts a time
const BLOCK_OR_TIME_HELP: &str = "Block number, tag (\"latest\"), or a time: a date \
    (2024-01-31, 2024-01-31T12:00:00Z), a Unix timestamp (@1706700000) or an age (7d, 12h). \
    A time selects the latest block at or before it. A plain date is 00:00 UTC at the start of \
    that day, so it reads the state before anything on that date; for an end-of-day figure use \
    the next day's date or a full time (2024-01-31T23:59:59Z)";

#[derive(Subcommand, Clone)]
#[command(after_help = r#"Examples:
  # Call a contract with function signature
  ethcli rpc call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 -s "totalSupply()" -d uint256

  # Call with arguments and decode output
  ethcli rpc call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 -s "balanceOf(address)" -a 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 -d uint256

  # Get block info
  ethcli rpc block latest
  ethcli rpc block 21000000 --full --format json

  # Read storage slot
  ethcli rpc storage 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 0

  # Get transaction receipt
  ethcli rpc receipt 0x123abc...

  # Get a transaction by hash
  ethcli rpc tx 0x123abc... -o json

  # Get native balance at a block
  ethcli rpc balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --block 21000000 -o json

  # A date, a Unix timestamp or an age works wherever --block is accepted
  ethcli rpc balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --block 2024-01-01
  ethcli rpc call 0x... -s "totalSupply()" -d uint256 --block 7d

  # Find the block for a time
  ethcli rpc block-at 2024-01-01T12:00:00Z"#)]
pub enum RpcCommands {
    /// Call a contract (eth_call) - read-only, no transaction
    ///
    /// Examples:
    ///   ethcli rpc call 0x... 0xa9059cbb...              # Raw calldata
    ///   ethcli rpc call 0x... -s "balanceOf(address)" -a 0xabc...  # With signature
    ///   ethcli rpc call 0x... -s "totalSupply()" -d uint256        # With decode
    #[command(alias = "eth_call")]
    Call {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        to: String,

        /// Calldata (hex encoded) - use this OR --sig
        #[arg(value_name = "DATA")]
        data: Option<String>,

        /// Function signature, e.g. "balanceOf(address)". The form with
        /// return types, "balanceOf(address)(uint256)", also decodes the
        /// result (as --decode would)
        #[arg(long, short, conflicts_with = "data", value_name = "SIG")]
        sig: Option<String>,

        /// Function arguments (use with --sig). Split on spaces: use
        /// --args-json for an argument that holds a space
        #[arg(long, short = 'a', num_args = 1.., value_delimiter = ' ', value_name = "ARG")]
        args: Option<Vec<String>>,

        /// Function arguments as a JSON array of strings (use with --sig), for
        /// arguments that contain spaces: '["hello world", "1"]'
        #[arg(long, conflicts_with = "args", requires = "sig", value_name = "JSON")]
        args_json: Option<String>,

        /// Block number, "latest", or a time (2024-01-31, @1706700000, 7d)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK", long_help = BLOCK_OR_TIME_HELP)]
        block: String,

        /// Decode output as type (e.g., "uint256", "(address,uint256)")
        #[arg(long, short, value_name = "TYPE")]
        decode: Option<String>,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get block information
    #[command(alias = "eth_getBlockByNumber", alias = "getBlockByNumber")]
    Block {
        /// Block number, hash, or "latest"
        #[arg(value_name = "BLOCK")]
        block: String,

        /// Show full transactions
        #[arg(long, short)]
        full: bool,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Read storage slot
    #[command(alias = "eth_getStorageAt")]
    Storage {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Storage slot (hex or decimal)
        #[arg(value_name = "SLOT")]
        slot: String,

        /// Block number, "latest", or a time (2024-01-31, @1706700000, 7d)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK", long_help = BLOCK_OR_TIME_HELP)]
        block: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get contract bytecode
    #[command(alias = "eth_getCode")]
    Code {
        /// Contract address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Block number, "latest", or a time (2024-01-31, @1706700000, 7d)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK", long_help = BLOCK_OR_TIME_HELP)]
        block: String,

        /// Print only the size and code hash, not the bytecode
        ///
        /// Answers "is this a contract" and "did the code change" without
        /// returning the bytecode itself.
        #[arg(long, short)]
        summary: bool,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get account nonce
    #[command(alias = "eth_getTransactionCount")]
    Nonce {
        /// Account address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Block number, "latest", or a time (2024-01-31, @1706700000, 7d)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK", long_help = BLOCK_OR_TIME_HELP)]
        block: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get transaction receipt
    #[command(alias = "eth_getTransactionReceipt")]
    Receipt {
        /// Transaction hash
        #[arg(value_name = "TX_HASH")]
        hash: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Run many read-only calls at one block in a single request (Multicall3)
    ///
    /// Each call is one argument or one line: `<address> <signature> [args...]`.
    /// Give the return types in the signature to have results decoded, e.g.
    /// `balanceOf(address)(uint256)`. Arguments are split on whitespace.
    ///
    /// "latest" is resolved to a block number once, so every call reads the
    /// same state. With several blocks (`--block a,b,c` or `--block-range`),
    /// the same calls are repeated at each block.
    ///
    /// Calls go through the Multicall3 contract, so `msg.sender` is that
    /// contract. Where Multicall3 is not available (a block before it was
    /// deployed, or a chain without it) each call is sent separately.
    ///
    /// Examples:
    ///   ethcli rpc multicall "0xA0b8... totalSupply()(uint256)" "0xA0b8... balanceOf(address)(uint256) 0xd8dA..."
    ///   ethcli rpc multicall -f calls.txt --block-range 21000000:21070000:7200 -o json
    #[command(visible_alias = "call-many", alias = "batch")]
    Multicall {
        /// Calls, each as "<address> <signature> [args...]"
        #[arg(value_name = "CALL")]
        calls: Vec<String>,

        /// Read calls from a file, one per line ("-" for stdin; # starts a comment)
        #[arg(long, short = 'f', value_name = "FILE")]
        file: Option<String>,

        /// Block number, tag, or a comma-separated list of block numbers
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
        block: String,

        /// Repeat at every STEP blocks from FROM to TO inclusive (FROM:TO:STEP)
        #[arg(long, value_name = "FROM:TO:STEP", conflicts_with = "block")]
        block_range: Option<String>,

        /// Print CSV: one row per call and block, with each decoded output in
        /// its own column (`output_0`, `output_1`, ...)
        #[arg(long, conflicts_with = "format")]
        csv: bool,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Find the block for a time: the latest block at or before it
    ///
    /// TIME is a date (2024-01-31 or 2024-01-31T12:00:00Z), a Unix timestamp
    /// written as @1706700000, or an age such as 7d, 12h or 30m. The block is
    /// found by binary search over block headers, so it is exact.
    ///
    /// A plain date is 00:00 UTC at the start of that day, so 2024-01-31
    /// gives the last block of 30 January. For the end of a day, pass the
    /// next day's date or a full time such as 2024-01-31T23:59:59Z.
    #[command(visible_alias = "find-block")]
    BlockAt {
        /// Date, @timestamp or age
        #[arg(value_name = "TIME")]
        time: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Find the block where a value changed, by binary search over blocks
    ///
    /// The target is one of: a contract call "<address> <signature> [args...]"
    /// (as in `rpc multicall`; give return types to have values decoded),
    /// --balance, --storage, --code or --nonce. The value is read at --from,
    /// and the first block in (--from, --to] where it differs is found in
    /// about log2(to - from) reads. Needs historical state (an archive
    /// endpoint for old blocks).
    ///
    /// Assumes one change in the range. With several, it finds one boundary,
    /// not always the first: narrow the range or use --all. --all repeats the
    /// search after each change; it misses changes that are undone inside a
    /// segment (a value that goes up and back down).
    ///
    /// Examples:
    ///   ethcli rpc find-change --code 0xA0b8... --from 6000000
    ///   ethcli rpc find-change "0xA0b8... balanceOf(address)(uint256) 0xd8dA..." --from 2024-01-01 --all
    ///   ethcli rpc find-change --nonce 0xd8dA... --from 30d --json
    #[command(visible_alias = "bisect")]
    FindChange {
        /// Contract call, "<address> <signature> [args...]"
        #[arg(value_name = "CALL")]
        call: Option<String>,

        /// Native balance of this address
        #[arg(long, value_name = "ADDRESS")]
        balance: Option<String>,

        /// Storage slot of a contract
        #[arg(long, num_args = 2, value_names = ["ADDRESS", "SLOT"])]
        storage: Vec<String>,

        /// Code at this address (when it was deployed or changed)
        #[arg(long, value_name = "ADDRESS")]
        code: Option<String>,

        /// Nonce of this address
        #[arg(long, value_name = "ADDRESS")]
        nonce: Option<String>,

        /// Start: block number, or a time (2024-01-31, @1706700000, 7d)
        #[arg(long, value_name = "BLOCK")]
        from: String,

        /// End: block number, a time, or "latest"
        #[arg(long, default_value = "latest", value_name = "BLOCK")]
        to: String,

        /// Report every change (up to 100), searching again after each one
        #[arg(long)]
        all: bool,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get a transaction by hash (eth_getTransactionByHash)
    #[command(visible_alias = "transaction", alias = "eth_getTransactionByHash")]
    Tx {
        /// Transaction hash
        #[arg(value_name = "TX_HASH")]
        hash: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get native token balance (eth_getBalance)
    ///
    /// Prints the balance in wei. With `-o json` the output also carries the
    /// balance in ether and the block that was queried.
    #[command(visible_alias = "bal", alias = "eth_getBalance")]
    Balance {
        /// Account address
        #[arg(value_name = "ADDRESS")]
        address: String,

        /// Block number, "latest", or a time (2024-01-31, @1706700000, 7d)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK", long_help = BLOCK_OR_TIME_HELP)]
        block: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get current chain ID
    #[command(alias = "eth_chainId")]
    ChainId {
        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get current block number
    #[command(alias = "eth_blockNumber")]
    BlockNumber {
        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },

    /// Get current gas price
    #[command(alias = "eth_gasPrice")]
    GasPrice {
        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },
}

pub async fn handle(
    action: &RpcCommands,
    chain: Chain,
    rpc_url: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    if let RpcCommands::FindChange {
        call,
        balance,
        storage,
        code,
        nonce,
        from,
        to,
        all,
        format,
    } = action
    {
        let target = super::find_change::Target::from_args(
            call.as_deref(),
            balance.as_deref(),
            storage,
            code.as_deref(),
            nonce.as_deref(),
        )?;
        let args = super::find_change::Args {
            target,
            from,
            to,
            all: *all,
            json: !format.is_table(),
        };
        return super::find_change::run(args, chain, rpc_url, quiet).await;
    }

    // A --block written as a time is turned into a block number first, so
    // endpoint selection and the command itself both see a plain number
    let resolved_action;
    let action = match action_block(action) {
        Some(block) if crate::rpc::block_time::parse_time_spec(block, u64::MAX).is_some() => {
            let endpoint = match &rpc_url {
                Some(url) => Endpoint::new(EndpointConfig::new(url.clone()), 30, None)?,
                None => crate::rpc::get_rpc_endpoint(chain)?,
            };
            let resolved = crate::rpc::block_time::resolve_time_spec(endpoint.provider(), block)
                .await?
                .ok_or_else(|| anyhow::anyhow!("'{block}' could not be resolved to a block"))?;
            if !quiet {
                eprintln!(
                    "Resolved --block {block} to block {} ({})",
                    resolved.number,
                    resolved.iso_time()
                );
            }
            let mut owned = action.clone();
            set_action_block(&mut owned, resolved.number.to_string());
            resolved_action = owned;
            &resolved_action
        }
        _ => action,
    };

    // Explicit --rpc-url: use it alone. Otherwise try ranked configured
    // endpoints, failing over on rate limits / transport errors.
    let candidates = if let Some(url) = rpc_url {
        vec![Endpoint::new(EndpointConfig::new(url), 30, None)?]
    } else {
        // A historical block number prefers archive endpoints
        let options = match action_block_number(action) {
            Some(n) => crate::rpc::SelectionOptions::for_block(n),
            None => crate::rpc::SelectionOptions::default(),
        };
        crate::rpc::candidate_endpoints(chain, &options, crate::rpc::MAX_FAILOVER_ATTEMPTS)?
    };

    // Like with_failover, but "not found" also moves on: a pruned or lagging
    // node may lack a block or receipt another endpoint has
    let total = candidates.len();
    let mut last_err = None;
    for (i, endpoint) in candidates.into_iter().enumerate() {
        match run_action(action, &endpoint, chain, quiet).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                let retry = !e.is::<Reverted>()
                    && (e.is::<NotFound>() || crate::rpc::is_failover_error(&format!("{e:#}")));
                if i + 1 < total && retry {
                    last_err = Some(e);
                    continue;
                }
                return Err(e);
            }
        }
    }
    Err(last_err.unwrap_or_else(|| anyhow::anyhow!("No RPC endpoints available")))
}

/// The contract reverted; the text says why
#[derive(Debug)]
struct Reverted(String);

impl std::fmt::Display for Reverted {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Call reverted: {}", self.0)
    }
}

impl std::error::Error for Reverted {}

/// Nothing at that block/hash on this endpoint
#[derive(Debug)]
struct NotFound(&'static str);

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotFound {}

/// The `--block` value of an action that takes a single block
fn action_block(action: &RpcCommands) -> Option<&str> {
    match action {
        RpcCommands::Call { block, .. }
        | RpcCommands::Storage { block, .. }
        | RpcCommands::Code { block, .. }
        | RpcCommands::Nonce { block, .. }
        | RpcCommands::Balance { block, .. }
        | RpcCommands::Block { block, .. } => Some(block),
        // A comma-separated list of block numbers is not a time
        RpcCommands::Multicall {
            block,
            block_range: None,
            ..
        } if !block.contains(',') => Some(block),
        _ => None,
    }
}

/// Replace the `--block` value of an action (see [`action_block`])
fn set_action_block(action: &mut RpcCommands, value: String) {
    match action {
        RpcCommands::Call { block, .. }
        | RpcCommands::Storage { block, .. }
        | RpcCommands::Code { block, .. }
        | RpcCommands::Nonce { block, .. }
        | RpcCommands::Balance { block, .. }
        | RpcCommands::Block { block, .. }
        | RpcCommands::Multicall { block, .. } => *block = value,
        _ => {}
    }
}

/// The numeric block an action reads at, if any (tags: None)
fn action_block_number(action: &RpcCommands) -> Option<u64> {
    if let RpcCommands::Multicall {
        block, block_range, ..
    } = action
    {
        // The earliest block decides whether an archive node is needed
        return multicall_blocks(block, block_range.as_deref())
            .ok()?
            .into_iter()
            .filter_map(|b| crate::utils::parse_block_number(&b).ok())
            .min();
    }
    let block = match action {
        RpcCommands::Call { block, .. }
        | RpcCommands::Block { block, .. }
        | RpcCommands::Storage { block, .. }
        | RpcCommands::Code { block, .. }
        | RpcCommands::Nonce { block, .. }
        | RpcCommands::Balance { block, .. } => block,
        _ => return None,
    };
    crate::utils::parse_block_number(block).ok()
}

/// Execute a (read-only) RPC command against a single endpoint
async fn run_action(
    action: &RpcCommands,
    endpoint: &Endpoint,
    chain: Chain,
    quiet: bool,
) -> anyhow::Result<()> {
    let provider = endpoint.provider();

    match action {
        RpcCommands::Call {
            to,
            data,
            sig,
            args,
            args_json,
            block,
            decode,
            format,
        } => {
            let to_addr =
                Address::from_str(to).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            // "name(args)(returns)": encode as multicall does and decode the
            // result with the declared return types
            let json_args: Option<Vec<String>> = args_json
                .as_deref()
                .map(|j| {
                    serde_json::from_str(j).map_err(|e| {
                        anyhow::anyhow!("--args-json must be a JSON array of strings: {e}")
                    })
                })
                .transpose()?;
            let func_args: &[String] = json_args.as_deref().or(args.as_deref()).unwrap_or(&[]);
            let mut returns_fn: Option<alloy::json_abi::Function> = None;
            let calldata = if let Some(signature) =
                sig.as_deref().filter(|s| return_types_of(s).is_some())
            {
                let (function, calldata) =
                    super::contract::encode_signature_call(signature, func_args)?;
                if decode.is_none() && !function.outputs.is_empty() {
                    returns_fn = Some(function);
                }
                calldata
            } else if let Some(signature) = sig {
                // Encode from function signature
                let encoded = super::cast::abi_encode(signature, func_args)?;
                let hex_str = encoded.strip_prefix("0x").unwrap_or(&encoded);
                hex::decode(hex_str).map_err(|e| anyhow::anyhow!("Encoding error: {}", e))?
            } else if let Some(data) = data {
                // Use raw calldata
                let data_hex = data.strip_prefix("0x").unwrap_or(data);
                hex::decode(data_hex).map_err(|e| anyhow::anyhow!("Invalid calldata: {}", e))?
            } else {
                return Err(anyhow::anyhow!(
                    "Calldata or --sig required. Use: rpc call <addr> <calldata> OR rpc call <addr> -s \"func()\""
                ));
            };

            let block_id = parse_block_id(block)?;

            let tx = alloy::rpc::types::TransactionRequest::default()
                .to(to_addr)
                .input(calldata.into());

            let result = match provider.call(tx).block(block_id).await {
                Ok(result) => result,
                Err(e) => {
                    // A revert is an answer from the contract, not a failed
                    // request: say why, and do not try another endpoint
                    let Some(data) = crate::abi::revert::revert_data_of(&e) else {
                        return Err(anyhow::anyhow!("Call failed: {}", e));
                    };
                    let revert = crate::abi::revert::describe(&data, None, chain).await;
                    if !format.is_table() {
                        let out = serde_json::json!({
                            "to": to_addr.to_checksum(None),
                            "block": block,
                            "reverted": true,
                            "revert": revert,
                        });
                        print_json(&out, *format)?;
                    }
                    return Err(Reverted(revert.message).into());
                }
            };

            if let Some(function) = &returns_fn {
                let values = decode_returns_strict(function, &result)?;
                if format.is_table() {
                    for v in &values {
                        println!("{}", super::contract::format_value(v));
                    }
                } else {
                    let outputs: Vec<serde_json::Value> = values
                        .iter()
                        .map(super::contract::dyn_value_to_json)
                        .collect();
                    let out = serde_json::json!({
                        "to": to_addr.to_checksum(None),
                        "block": block,
                        "raw": format!("0x{}", hex::encode(&result)),
                        "returns": return_signature(function),
                        "decoded": if outputs.len() == 1 { outputs[0].clone() } else { serde_json::Value::Array(outputs.clone()) },
                        "outputs": outputs,
                    });
                    print_json(&out, *format)?;
                }
            } else if !format.is_table() {
                let mut out = serde_json::json!({
                    "to": to_addr.to_checksum(None),
                    "block": block,
                    "raw": format!("0x{}", hex::encode(&result)),
                });
                if let Some(type_sig) = decode.as_deref() {
                    out["type"] = serde_json::json!(type_sig);
                    out["decoded"] = decode_output_json(&result, type_sig)?;
                }
                print_json(&out, *format)?;
            } else if let Some(type_sig) = decode.as_deref() {
                let decoded = decode_output(&result, type_sig)?;
                println!("{}", decoded);
            } else {
                println!("0x{}", hex::encode(&result));
            }
        }

        RpcCommands::Block {
            block,
            full,
            format,
        } => {
            let block_id = parse_block_id(block)?;

            let block_data = if *full {
                provider
                    .get_block(block_id)
                    .full()
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get block: {}", e))?
            } else {
                provider
                    .get_block(block_id)
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to get block: {}", e))?
            };

            match block_data {
                Some(b) => {
                    if matches!(format, OutputFormat::Json) {
                        println!("{}", serde_json::to_string_pretty(&b)?);
                    } else if matches!(format, OutputFormat::Ndjson) {
                        println!("{}", serde_json::to_string(&b)?);
                    } else {
                        println!("Block {}", b.header.number);
                        println!("{}", "─".repeat(50));
                        println!("Hash:        {:#x}", b.header.hash);
                        println!("Parent:      {:#x}", b.header.parent_hash);
                        println!("Timestamp:   {}", b.header.timestamp);
                        println!("Gas Used:    {}", b.header.gas_used);
                        println!("Gas Limit:   {}", b.header.gas_limit);
                        // Show base fee with decimal precision
                        let base_fee = b.header.base_fee_per_gas.unwrap_or(0);
                        let gwei_div = 1_000_000_000u64;
                        let base_gwei = base_fee / gwei_div;
                        let base_frac = (base_fee % gwei_div) * 1000 / gwei_div;
                        println!("Base Fee:    {}.{:03} gwei", base_gwei, base_frac);
                        println!("Txs:         {}", b.transactions.len());
                        println!("Miner:       {}", b.header.beneficiary.to_checksum(None));
                    }
                }
                None => return Err(NotFound("Block not found").into()),
            }
        }

        RpcCommands::Storage {
            address,
            slot,
            block,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let slot_val = if slot.starts_with("0x") {
                U256::from_str(slot).map_err(|e| anyhow::anyhow!("Invalid slot: {}", e))?
            } else {
                U256::from_str(slot).map_err(|e| anyhow::anyhow!("Invalid slot: {}", e))?
            };

            let block_id = parse_block_id(block)?;

            let value = provider
                .get_storage_at(addr, slot_val)
                .block_id(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to read storage: {}", e))?;

            // The whole 32-byte word, leading zeros kept, so an address
            // stored in it reads the same as everywhere else
            let as_address = storage_word_address(value);
            if format.is_table() {
                println!("{:#066x}", value);
                if let (Some(a), false) = (as_address, quiet) {
                    eprintln!("as an address (low 20 bytes): {}", a.to_checksum(None));
                }
            } else {
                let out = serde_json::json!({
                    "address": addr.to_checksum(None),
                    "slot": format!("{slot_val:#x}"),
                    "block": block,
                    "value": format!("{:#066x}", value),
                    "decimal": value.to_string(),
                    "value_as_address": as_address.map(|a| a.to_checksum(None)),
                });
                print_json(&out, *format)?;
            }
        }

        RpcCommands::Code {
            address,
            block,
            summary,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let block_id = parse_block_id(block)?;

            let code = provider
                .get_code_at(addr)
                .block_id(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get code: {}", e))?;

            let code_hash = alloy::primitives::keccak256(&code);
            if !format.is_table() {
                let mut out = serde_json::json!({
                    "address": addr.to_checksum(None),
                    "block": block,
                    "has_code": !code.is_empty(),
                    "bytes": code.len(),
                    "code_hash": format!("{code_hash:#x}"),
                });
                if let Some(delegate) = eip7702_delegate(&code) {
                    out["eip7702_delegate"] = serde_json::json!(delegate.to_checksum(None));
                }
                if !*summary {
                    out["code"] = serde_json::json!(format!("0x{}", hex::encode(&code)));
                }
                print_json(&out, *format)?;
            } else if *summary {
                if code.is_empty() {
                    println!("no code (EOA or empty contract)");
                } else if let Some(delegate) = eip7702_delegate(&code) {
                    println!(
                        "EOA delegated to {} (EIP-7702), code hash {code_hash:#x}",
                        delegate.to_checksum(None)
                    );
                } else {
                    println!("{} bytes, code hash {code_hash:#x}", code.len());
                }
            } else if code.is_empty() {
                println!("0x (no code - EOA or empty contract)");
            } else {
                println!("0x{}", hex::encode(&code));
                if !quiet {
                    eprintln!("\n({} bytes)", code.len());
                }
            }
        }

        RpcCommands::Nonce {
            address,
            block,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let block_id = parse_block_id(block)?;

            let nonce = provider
                .get_transaction_count(addr)
                .block_id(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get nonce: {}", e))?;

            if format.is_table() {
                println!("{}", nonce);
            } else {
                let out = serde_json::json!({
                    "address": addr.to_checksum(None),
                    "block": block,
                    "nonce": nonce,
                });
                print_json(&out, *format)?;
            }
        }

        RpcCommands::Receipt { hash, format } => {
            let tx_hash =
                B256::from_str(hash).map_err(|e| anyhow::anyhow!("Invalid tx hash: {}", e))?;

            let receipt = provider
                .get_transaction_receipt(tx_hash)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get receipt: {}", e))?;

            match receipt {
                Some(r) => {
                    // The raw receipt keeps chain-specific fields (l1Fee,
                    // gasUsedForL1, ...) that the typed one drops
                    let raw: Option<serde_json::Value> = provider
                        .raw_request("eth_getTransactionReceipt".into(), (tx_hash,))
                        .await
                        .ok()
                        .flatten();
                    let breakdown = raw.as_ref().and_then(|v| {
                        crate::tx::l2_fee::FeeBreakdown::from_receipt_json(chain.chain_id(), v)
                    });
                    if !format.is_table() {
                        let mut json = match raw {
                            Some(v) if v.is_object() => v,
                            _ => serde_json::to_value(&r)?,
                        };
                        if let Some(b) = &breakdown {
                            json["feeBreakdown"] = serde_json::to_value(b)?;
                        }
                        if matches!(format, OutputFormat::Ndjson) {
                            println!("{}", serde_json::to_string(&json)?);
                        } else {
                            println!("{}", serde_json::to_string_pretty(&json)?);
                        }
                    } else {
                        println!("Transaction Receipt");
                        println!("{}", "─".repeat(50));
                        println!("Hash:        {:#x}", r.transaction_hash);
                        println!("Block:       {}", r.block_number.unwrap_or(0));
                        println!(
                            "Status:      {}",
                            if r.status() { "Success" } else { "Failed" }
                        );
                        println!("Gas Used:    {}", r.gas_used);
                        if let Some(to) = r.to {
                            println!("To:          {}", to.to_checksum(None));
                        }
                        if let Some(addr) = r.contract_address {
                            println!("Created:     {}", addr.to_checksum(None));
                        }
                        println!("Logs:        {}", r.inner.logs().len());
                        if let Some(b) = &breakdown {
                            println!("Fee:         {} {}", b.total_fee, b.native_symbol);
                            for line in b.lines(&b.native_symbol) {
                                println!("  {}", line.trim_start());
                            }
                        }
                    }
                }
                None => return Err(NotFound("Receipt not found (tx may be pending)").into()),
            }
        }

        RpcCommands::Multicall {
            calls,
            file,
            block,
            block_range,
            csv,
            format,
        } => {
            let specs = multicall_specs(calls, file.as_deref())?;
            let mut blocks = multicall_blocks(block, block_range.as_deref())?;
            // Pin a moving tag to a number so every call reads the same state
            for b in &mut blocks {
                if matches!(b.to_lowercase().as_str(), "latest" | "safe" | "finalized") {
                    let id = parse_block_id(b)?;
                    let header = provider
                        .get_block(id)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to resolve block '{b}': {e}"))?
                        .ok_or(NotFound("Block not found"))?;
                    *b = header.header.number.to_string();
                }
            }

            let mut reports = Vec::with_capacity(blocks.len());
            for b in &blocks {
                reports.push(run_multicall(provider, &specs, b, chain).await?);
            }

            if *csv {
                print!(
                    "{}",
                    crate::utils::csv_rows::json_rows_to_csv(
                        &multicall_csv_rows(&reports),
                        MULTICALL_CSV_COLUMNS
                    )?
                );
            } else if format.is_table() {
                for (i, report) in reports.iter().enumerate() {
                    if i > 0 {
                        println!();
                    }
                    print!("{}", format_multicall_table(report));
                }
            } else if format.is_ndjson() {
                for report in &reports {
                    println!("{}", serde_json::to_string(report)?);
                }
            } else if reports.len() == 1 {
                println!("{}", serde_json::to_string_pretty(&reports[0])?);
            } else {
                println!("{}", serde_json::to_string_pretty(&reports)?);
            }
        }

        RpcCommands::BlockAt { time, format } => {
            let resolved = crate::rpc::block_time::resolve_time_spec(provider, time)
                .await?
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "'{time}' is not a time. Use a date (2024-01-31), @<unix timestamp> or an age (7d)"
                    )
                })?;
            if format.is_table() {
                println!("{}", resolved.number);
                if !quiet {
                    eprintln!("Block time: {}", resolved.iso_time());
                }
            } else {
                let out = serde_json::json!({
                    "requested": time,
                    "block": resolved.number,
                    "timestamp": resolved.timestamp,
                    "time": resolved.iso_time(),
                });
                print_json(&out, *format)?;
            }
        }

        RpcCommands::Tx { hash, format } => {
            let tx_hash =
                B256::from_str(hash).map_err(|e| anyhow::anyhow!("Invalid tx hash: {}", e))?;

            let tx = provider
                .get_transaction_by_hash(tx_hash)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get transaction: {}", e))?;

            match tx {
                Some(t) => {
                    if matches!(format, OutputFormat::Json) {
                        println!("{}", serde_json::to_string_pretty(&t)?);
                    } else if matches!(format, OutputFormat::Ndjson) {
                        println!("{}", serde_json::to_string(&t)?);
                    } else {
                        print!("{}", format_tx_table(&serde_json::to_value(&t)?));
                    }
                }
                None => return Err(NotFound("Transaction not found").into()),
            }
        }

        RpcCommands::Balance {
            address,
            block,
            format,
        } => {
            let addr = Address::from_str(address)
                .map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            let block_id = parse_block_id(block)?;

            let balance = provider
                .get_balance(addr)
                .block_id(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get balance: {}", e))?;

            if matches!(format, OutputFormat::Table) {
                println!("{}", balance);
            } else {
                // Amounts are strings: a wei balance does not fit a JSON number
                let out = serde_json::json!({
                    "address": addr.to_checksum(None),
                    "block": block,
                    "wei": balance.to_string(),
                    "ether": super::cast::from_wei(&balance.to_string(), "eth")?,
                });
                if matches!(format, OutputFormat::Ndjson) {
                    println!("{}", serde_json::to_string(&out)?);
                } else {
                    println!("{}", serde_json::to_string_pretty(&out)?);
                }
            }
        }

        RpcCommands::ChainId { format } => {
            let chain_id = provider
                .get_chain_id()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get chain ID: {}", e))?;
            if format.is_table() {
                println!("{}", chain_id);
            } else {
                print_json(&serde_json::json!({ "chain_id": chain_id }), *format)?;
            }
        }

        RpcCommands::BlockNumber { format } => {
            let block_num = provider
                .get_block_number()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get block number: {}", e))?;
            if format.is_table() {
                println!("{}", block_num);
            } else {
                print_json(&serde_json::json!({ "block_number": block_num }), *format)?;
            }
        }

        RpcCommands::FindChange { .. } => {
            anyhow::bail!("find-change is handled before endpoint selection")
        }

        RpcCommands::GasPrice { format } => {
            let gas_price = provider
                .get_gas_price()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get gas price: {}", e))?;
            if format.is_table() {
                println!("{} gwei", format_gwei(gas_price));
            } else {
                let out = serde_json::json!({
                    "wei": gas_price.to_string(),
                    "gwei": format_gwei(gas_price),
                });
                print_json(&out, *format)?;
            }
        }
    }

    Ok(())
}

pub fn parse_block_id(block: &str) -> anyhow::Result<alloy::eips::BlockId> {
    use alloy::eips::{BlockId, BlockNumberOrTag};

    match block.to_lowercase().as_str() {
        "latest" => Ok(BlockId::Number(BlockNumberOrTag::Latest)),
        "pending" => Ok(BlockId::Number(BlockNumberOrTag::Pending)),
        "earliest" => Ok(BlockId::Number(BlockNumberOrTag::Earliest)),
        "finalized" => Ok(BlockId::Number(BlockNumberOrTag::Finalized)),
        "safe" => Ok(BlockId::Number(BlockNumberOrTag::Safe)),
        _ => {
            if block.starts_with("0x") && block.len() == 66 {
                // Could be a block hash (66 chars = 0x + 64 hex digits) or a padded hex number.
                // Try parsing as a number first - if it's a reasonable block number, use that.
                // Block numbers are currently ~20M, so anything that fits in u64 and is < 2^48
                // is likely a number, not a hash.
                if let Ok(num) = crate::utils::parse_block_number(block) {
                    // 2^48 = 281 trillion - far beyond any realistic block number
                    if num < (1u64 << 48) {
                        return Ok(BlockId::Number(BlockNumberOrTag::Number(num)));
                    }
                }
                // Otherwise treat as hash
                let hash = B256::from_str(block)
                    .map_err(|e| anyhow::anyhow!("Invalid block hash: {}", e))?;
                Ok(BlockId::Hash(hash.into()))
            } else {
                // Block number (decimal or hex)
                let num = crate::utils::parse_block_number(block)
                    .map_err(|e| anyhow::anyhow!("Invalid block number '{}': {}", block, e))?;
                Ok(BlockId::Number(BlockNumberOrTag::Number(num)))
            }
        }
    }
}

/// The declared return types of a function as one list, `(t1,t2)`
fn return_signature(function: &alloy::json_abi::Function) -> String {
    let types: Vec<String> = function
        .outputs
        .iter()
        .map(|p| p.selector_type().into_owned())
        .collect();
    format!("({})", types.join(","))
}

/// Decode return data with a function's declared outputs, strictly: the
/// values must re-encode to exactly the returned bytes, so declared types
/// that do not fit the data (a `bool` where a `uint8` came back, a tuple
/// where a struct came back) are an error, not a silent guess
pub(super) fn decode_returns_strict(
    function: &alloy::json_abi::Function,
    data: &[u8],
) -> anyhow::Result<Vec<alloy::dyn_abi::DynSolValue>> {
    use alloy::dyn_abi::FunctionExt;
    let sig = return_signature(function);
    let values = function.abi_decode_output(data).map_err(|e| {
        anyhow::anyhow!("Return data does not decode as {sig}: {e}. Check the declared return types, or call without them for the raw data")
    })?;
    let again = function
        .abi_encode_output(&values)
        .map_err(|e| anyhow::anyhow!("Cannot re-encode the decoded values as {sig}: {e}"))?;
    if again != data {
        anyhow::bail!(
            "Return data ({} bytes) is not the canonical encoding of {sig} (re-encoded: {} bytes): the declared return types are probably wrong. Call without them for the raw data",
            data.len(),
            again.len()
        );
    }
    Ok(values)
}

/// The return types of a `"name(args)(returns)"` signature as one type
/// string for `--decode`: the bare type for one return value, a tuple for
/// several, empty for `()`. `None` when the signature has no return part
pub(super) fn return_types_of(signature: &str) -> Option<String> {
    let sig = signature.trim();
    let open = sig.find('(')?;
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
    let rest = sig[close? + 1..].trim();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?.trim();
    // One top-level type stays bare; several make a tuple
    let mut depth = 0i32;
    let top_level_comma = inner.chars().any(|c| {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            ',' if depth == 0 => return true,
            _ => {}
        }
        false
    });
    Some(if top_level_comma {
        format!("({inner})")
    } else {
        inner.to_string()
    })
}

/// The address an EOA delegates to under EIP-7702, read from its code
/// (`0xef0100` followed by the 20-byte address)
fn eip7702_delegate(code: &[u8]) -> Option<Address> {
    (code.len() == 23 && code[..3] == [0xef, 0x01, 0x00]).then(|| Address::from_slice(&code[3..]))
}

/// Most calls sent in one Multicall3 request
const MULTICALL_CHUNK: usize = 200;
/// Most blocks one `rpc multicall` will visit
const MULTICALL_MAX_BLOCKS: usize = 1000;

/// One parsed `<address> <signature> [args...]` line
pub(super) struct CallSpec {
    pub(super) address: Address,
    args: Vec<String>,
    pub(super) function: alloy::json_abi::Function,
    pub(super) calldata: Vec<u8>,
}

pub(super) fn parse_call_spec(line: &str) -> anyhow::Result<CallSpec> {
    let mut parts = line.split_whitespace();
    let (Some(address), Some(signature)) = (parts.next(), parts.next()) else {
        anyhow::bail!("Call '{line}' must be \"<address> <signature> [args...]\"");
    };
    let address = Address::from_str(address)
        .map_err(|e| anyhow::anyhow!("Invalid address in call '{line}': {e}"))?;
    let args: Vec<String> = parts.map(str::to_string).collect();
    let (function, calldata) = super::contract::encode_signature_call(signature, &args)
        .map_err(|e| anyhow::anyhow!("Call '{line}': {e}"))?;
    Ok(CallSpec {
        address,
        args,
        function,
        calldata,
    })
}

/// Calls from the command line followed by those in `--file`
fn multicall_specs(calls: &[String], file: Option<&str>) -> anyhow::Result<Vec<CallSpec>> {
    let mut lines: Vec<String> = calls.to_vec();
    if let Some(path) = file {
        let text = if path == "-" {
            std::io::read_to_string(std::io::stdin())?
        } else {
            std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!("Cannot read {path}: {e}"))?
        };
        lines.extend(
            text.lines()
                .map(|l| l.split('#').next().unwrap_or("").trim().to_string())
                .filter(|l| !l.is_empty()),
        );
    }
    if lines.is_empty() {
        anyhow::bail!("No calls given. Pass \"<address> <signature> [args...]\" or --file");
    }
    lines.iter().map(|l| parse_call_spec(l)).collect()
}

/// The blocks to read at: `--block` (one, or a comma-separated list) or
/// `--block-range FROM:TO:STEP`
fn multicall_blocks(block: &str, range: Option<&str>) -> anyhow::Result<Vec<String>> {
    let blocks: Vec<String> = match range {
        Some(range) => {
            let parts: Vec<&str> = range.split(':').collect();
            let [from, to, step] = parts.as_slice() else {
                anyhow::bail!("--block-range must be FROM:TO:STEP, got '{range}'");
            };
            let parse = |s: &str| {
                crate::utils::parse_block_number(s)
                    .map_err(|e| anyhow::anyhow!("Invalid block '{s}' in --block-range: {e}"))
            };
            let (from, to, step) = (parse(from)?, parse(to)?, parse(step)?);
            if step == 0 || from > to {
                anyhow::bail!("--block-range needs FROM <= TO and STEP > 0, got '{range}'");
            }
            // Check the count before building the list: the range is caller
            // input and may span the whole chain
            let count = ((to - from) / step).saturating_add(1);
            if count > MULTICALL_MAX_BLOCKS as u64 {
                anyhow::bail!(
                    "{count} blocks requested; the limit is {MULTICALL_MAX_BLOCKS}. Use a larger step"
                );
            }
            (0..count).map(|i| (from + i * step).to_string()).collect()
        }
        None => block
            .split(',')
            .map(|b| b.trim().to_string())
            .filter(|b| !b.is_empty())
            .take(MULTICALL_MAX_BLOCKS + 1)
            .collect(),
    };
    if blocks.is_empty() {
        anyhow::bail!("No block given");
    }
    if blocks.len() > MULTICALL_MAX_BLOCKS {
        anyhow::bail!(
            "{} blocks requested; the limit is {MULTICALL_MAX_BLOCKS}. Use a larger step",
            blocks.len()
        );
    }
    Ok(blocks)
}

/// Text of a standard `Error(string)` revert, if the data is one
fn revert_reason(data: &[u8]) -> Option<String> {
    use alloy::dyn_abi::DynSolType;
    if data.len() < 4 || data[..4] != [0x08, 0xc3, 0x79, 0xa0] {
        return None;
    }
    match DynSolType::String.abi_decode(&data[4..]).ok()? {
        alloy::dyn_abi::DynSolValue::String(s) => Some(s),
        _ => None,
    }
}

/// Run every call at one block and describe the results as JSON
async fn run_multicall<P: Provider>(
    provider: &P,
    specs: &[CallSpec],
    block: &str,
    chain: Chain,
) -> anyhow::Result<serde_json::Value> {
    use crate::rpc::{MulticallBuilder, MulticallResult};
    use alloy::dyn_abi::FunctionExt;

    let block_id = parse_block_id(block)?;

    // Multicall3 first; a failure there (not deployed at this block, or on
    // this chain) falls back to one eth_call per entry
    let mut results: Vec<MulticallResult> = Vec::with_capacity(specs.len());
    let mut via = "multicall3";
    for chunk in specs.chunks(MULTICALL_CHUNK) {
        let builder = chunk.iter().fold(MulticallBuilder::new(), |b, spec| {
            b.add_call(spec.address, spec.calldata.clone().into(), true)
        });
        match builder.execute_at(provider, block_id).await {
            Ok(mut chunk_results) if chunk_results.len() == chunk.len() => {
                results.append(&mut chunk_results)
            }
            _ => {
                via = "eth_call";
                results.clear();
                break;
            }
        }
    }
    if via == "eth_call" {
        let calls = specs.iter().map(|spec| async move {
            let tx = alloy::rpc::types::TransactionRequest::default()
                .to(spec.address)
                .input(spec.calldata.clone().into());
            match provider.call(tx).block(block_id).await {
                Ok(data) => Ok(MulticallResult {
                    success: true,
                    data,
                }),
                // A revert is a result; a transport failure is not
                Err(e) => match crate::abi::revert::revert_data_of(&e) {
                    Some(data) => Ok(MulticallResult {
                        success: false,
                        data,
                    }),
                    None => Err(anyhow::anyhow!("Call failed: {e}")),
                },
            }
        });
        results = futures::future::try_join_all(calls).await?;
    }

    let mut entries: Vec<serde_json::Value> = specs
        .iter()
        .zip(&results)
        .enumerate()
        .map(|(index, (spec, result))| {
            let mut entry = serde_json::json!({
                "index": index,
                "address": spec.address.to_checksum(None),
                "function": spec.function.signature(),
                "args": spec.args,
                "success": result.success,
                "raw": format!("0x{}", hex::encode(&result.data)),
            });
            if !result.success {
                if let Some(reason) = revert_reason(&result.data) {
                    entry["revert_reason"] = serde_json::json!(reason);
                }
            } else if !spec.function.outputs.is_empty() {
                match spec.function.abi_decode_output(&result.data) {
                    Ok(values) => {
                        entry["outputs"] = serde_json::Value::Array(
                            values
                                .iter()
                                .map(super::contract::dyn_value_to_json)
                                .collect(),
                        );
                    }
                    Err(e) => {
                        // No code at the address returns empty data, which
                        // cannot be the declared types
                        entry["success"] = serde_json::json!(false);
                        entry["decode_error"] = serde_json::json!(e.to_string());
                    }
                }
            }
            entry
        })
        .collect();

    // Name the errors of the calls that reverted
    for (entry, result) in entries.iter_mut().zip(&results) {
        if !result.success {
            let revert = crate::abi::revert::describe(&result.data, None, chain).await;
            entry["revert"] = serde_json::to_value(&revert)?;
        }
    }

    Ok(serde_json::json!({
        "block": block,
        "via": via,
        "results": entries,
    }))
}

/// Column order for `rpc multicall --csv` (outputs follow, then the rest)
const MULTICALL_CSV_COLUMNS: &[&str] = &[
    "block", "index", "address", "function", "args", "success", "output_0", "output_1", "output_2",
    "output_3", "output_4", "output_5", "output_6", "output_7", "output_8", "output_9",
];

/// One flat row per call and block, for CSV: the decoded outputs each get a
/// column, the revert reason replaces the nested revert object
fn multicall_csv_rows(reports: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let mut rows = Vec::new();
    for report in reports {
        for entry in report["results"].as_array().into_iter().flatten() {
            let args: Vec<String> = entry["args"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|a| {
                    a.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| a.to_string())
                })
                .collect();
            let mut row = serde_json::json!({
                "block": report["block"],
                "index": entry["index"],
                "address": entry["address"],
                "function": entry["function"],
                "args": args.join(" "),
                "success": entry["success"],
            });
            for (i, output) in entry["outputs"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                row[format!("output_{i}")] = match output {
                    serde_json::Value::String(text) => serde_json::json!(text),
                    other => serde_json::json!(other.to_string()),
                };
            }
            if let Some(message) = entry["revert"]["message"].as_str() {
                row["revert"] = serde_json::json!(message);
            }
            if let Some(error) = entry["decode_error"].as_str() {
                row["decode_error"] = serde_json::json!(error);
            }
            row["raw"] = entry["raw"].clone();
            rows.push(row);
        }
    }
    rows
}

/// One line per call: `[index] address function(args) = values`
fn format_multicall_table(report: &serde_json::Value) -> String {
    use std::fmt::Write;

    let mut out = String::new();
    let _ = writeln!(
        out,
        "Block {}",
        report["block"].as_str().unwrap_or_default()
    );
    for entry in report["results"].as_array().into_iter().flatten() {
        let args: Vec<&str> = entry["args"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|a| a.as_str())
            .collect();
        let name = entry["function"]
            .as_str()
            .unwrap_or_default()
            .split('(')
            .next()
            .unwrap_or_default();
        let value = if entry["success"].as_bool() != Some(true) {
            match (
                entry["revert"]["message"].as_str(),
                entry["decode_error"].as_str(),
            ) {
                (Some(reason), _) => format!("REVERT: {reason}"),
                (None, Some(_)) => "FAILED: no data returned for the declared types".to_string(),
                (None, None) => "REVERT".to_string(),
            }
        } else if let Some(outputs) = entry["outputs"].as_array() {
            outputs
                .iter()
                .map(|v| match v {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            entry["raw"].as_str().unwrap_or_default().to_string()
        };
        let _ = writeln!(
            out,
            "[{}] {} {}({}) = {}",
            entry["index"],
            entry["address"].as_str().unwrap_or_default(),
            name,
            args.join(", "),
            value
        );
    }
    out
}

/// A hex quantity from a JSON-RPC object as a decimal string
fn hex_field_to_dec(value: &serde_json::Value, key: &str) -> Option<String> {
    let raw = value.get(key)?.as_str()?;
    U256::from_str(raw).ok().map(|n| n.to_string())
}

/// Human-readable summary of a transaction object as returned by the node
fn format_tx_table(tx: &serde_json::Value) -> String {
    use std::fmt::Write;

    let text = |key: &str| tx.get(key).and_then(|v| v.as_str()).map(str::to_string);
    let mut out = String::new();
    let _ = writeln!(out, "Transaction");
    let _ = writeln!(out, "{}", "─".repeat(50));
    let _ = writeln!(out, "Hash:        {}", text("hash").unwrap_or_default());
    match hex_field_to_dec(tx, "blockNumber") {
        Some(block) => {
            let _ = writeln!(out, "Block:       {block}");
        }
        None => {
            let _ = writeln!(out, "Block:       pending");
        }
    }
    if let Some(index) = hex_field_to_dec(tx, "transactionIndex") {
        let _ = writeln!(out, "Index:       {index}");
    }
    let _ = writeln!(out, "From:        {}", text("from").unwrap_or_default());
    match text("to") {
        Some(to) => {
            let _ = writeln!(out, "To:          {to}");
        }
        None => {
            let _ = writeln!(out, "To:          (contract creation)");
        }
    }
    if let Some(wei) = hex_field_to_dec(tx, "value") {
        let ether = super::cast::from_wei(&wei, "eth").unwrap_or_default();
        let _ = writeln!(out, "Value:       {wei} wei ({ether} ETH)");
    }
    if let Some(nonce) = hex_field_to_dec(tx, "nonce") {
        let _ = writeln!(out, "Nonce:       {nonce}");
    }
    if let Some(gas) = hex_field_to_dec(tx, "gas") {
        let _ = writeln!(out, "Gas Limit:   {gas}");
    }
    let input = text("input").unwrap_or_default();
    let bytes = input.len().saturating_sub(2) / 2;
    if bytes >= 4 {
        let _ = writeln!(out, "Input:       {} ({bytes} bytes)", &input[..10]);
    } else {
        let _ = writeln!(out, "Input:       {input} ({bytes} bytes)");
    }
    out
}

/// Print a JSON value pretty (json) or on one line (ndjson)
/// A storage word read as an address: when it fits in 20 bytes and is
/// wider than 12 bytes (a counter or flag is not shown as one). It may
/// still not be an address
fn storage_word_address(value: U256) -> Option<Address> {
    (value.bit_len() > 96 && value.bit_len() <= 160)
        .then(|| Address::from_slice(&value.to_be_bytes::<32>()[12..]))
}

fn print_json(value: &serde_json::Value, format: OutputFormat) -> anyhow::Result<()> {
    if format.is_ndjson() {
        println!("{}", serde_json::to_string(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}

/// Decode return data as JSON: a tuple type gives an array of its members
fn decode_output_json(data: &[u8], type_sig: &str) -> anyhow::Result<serde_json::Value> {
    use alloy::dyn_abi::DynSolType;

    let ty = DynSolType::parse(type_sig).map_err(|e| anyhow::anyhow!("Invalid type: {}", e))?;
    let decoded = ty
        .abi_decode_params(data)
        .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?;
    Ok(super::contract::dyn_value_to_json(&decoded))
}

fn decode_output(data: &[u8], type_sig: &str) -> anyhow::Result<String> {
    use alloy::dyn_abi::DynSolType;

    let ty = DynSolType::parse(type_sig).map_err(|e| anyhow::anyhow!("Invalid type: {}", e))?;

    // A tuple is the return values themselves, not one value wrapped in an offset
    let decoded = ty
        .abi_decode_params(data)
        .map_err(|e| anyhow::anyhow!("Failed to decode: {}", e))?;

    Ok(super::contract::format_value(&decoded))
}

/// Format a wei amount as gwei with exact decimals (trailing zeros trimmed).
///
/// `180_000_000` -> `"0.18"`, `25_000_000_000` -> `"25"`.
fn format_gwei(wei: u128) -> String {
    const GWEI: u128 = 1_000_000_000;
    let whole = wei / GWEI;
    let frac = wei % GWEI;
    if frac == 0 {
        whole.to_string()
    } else {
        let frac = format!("{frac:09}");
        format!("{whole}.{}", frac.trim_end_matches('0'))
    }
}

#[cfg(test)]
mod tests {
    /// A word holding an address with a leading zero nibble keeps all 64
    /// hex digits, and reads as that address
    #[test]
    fn storage_word_keeps_leading_zeros() {
        use alloy::primitives::{Address, U256};
        use std::str::FromStr;
        let v = U256::from_str("0x0f17d856ecc7c8ce9bd7d5b3e52a47eca5e8955a").unwrap();
        assert_eq!(
            format!("{v:#066x}"),
            "0x0000000000000000000000000f17d856ecc7c8ce9bd7d5b3e52a47eca5e8955a"
        );
        assert_eq!(
            super::storage_word_address(v),
            Some(Address::from_str("0x0f17d856ecc7c8ce9bd7d5b3e52a47eca5e8955a").unwrap())
        );
        assert_eq!(super::storage_word_address(U256::from(5u8)), None);
        assert_eq!(super::storage_word_address(U256::MAX), None);
    }

    #[test]
    fn decode_output_handles_dynamic_values() {
        use alloy::dyn_abi::DynSolValue;
        use alloy::primitives::U256;

        let tuple = DynSolValue::Tuple(vec![
            DynSolValue::Uint(U256::from(7), 256),
            DynSolValue::String("hello".to_string()),
        ])
        .abi_encode_params();
        let out = decode_output(&tuple, "(uint256,string)").unwrap();
        assert!(out.contains("hello"), "{out}");

        let single = DynSolValue::String("hello".to_string()).abi_encode();
        assert!(decode_output(&single, "string").unwrap().contains("hello"));
    }

    use super::*;
    use alloy::eips::{BlockId, BlockNumberOrTag};

    fn returns_of(sig: &str) -> alloy::json_abi::Function {
        super::super::contract::encode_signature_call(sig, &[])
            .unwrap()
            .0
    }

    #[test]
    fn return_decoding_is_strict_and_keeps_structs() {
        use alloy::dyn_abi::DynSolValue as V;
        use alloy::primitives::U256;
        let u = |n: u64| V::Uint(U256::from(n), 256);
        // One struct (string, uint256): a single dynamic tuple on the wire
        let one_struct =
            V::Tuple(vec![V::Tuple(vec![V::String("ab".into()), u(7)])]).abi_encode_params();
        let v = decode_returns_strict(&returns_of("f()((string,uint256))"), &one_struct).unwrap();
        assert_eq!(v.len(), 1);
        assert!(matches!(&v[0], V::Tuple(t) if t.len() == 2));
        // The same bytes are not two return values
        assert!(decode_returns_strict(&returns_of("f()(string,uint256)"), &one_struct).is_err());
        // Two return values (string, uint256)
        let two = V::Tuple(vec![V::String("ab".into()), u(7)]).abi_encode_params();
        assert_eq!(
            decode_returns_strict(&returns_of("f()(string,uint256)"), &two)
                .unwrap()
                .len(),
            2
        );
        assert!(decode_returns_strict(&returns_of("f()((string,uint256))"), &two).is_err());
        // Nested arrays
        let nested = V::Tuple(vec![V::Array(vec![
            V::Array(vec![u(1), u(2)]),
            V::Array(vec![]),
        ])])
        .abi_encode_params();
        let v = decode_returns_strict(&returns_of("f()(uint256[][])"), &nested).unwrap();
        assert!(matches!(&v[0], V::Array(a) if a.len() == 2));
        // bytes and string
        let bs = V::Tuple(vec![V::Bytes(vec![1, 2, 3]), V::String("x".into())]).abi_encode_params();
        assert_eq!(
            decode_returns_strict(&returns_of("f()(bytes,string)"), &bs)
                .unwrap()
                .len(),
            2
        );
        // Wrong declared types are errors: name() returns one string
        let name = V::Tuple(vec![V::String("USD Coin".into())]).abi_encode_params();
        assert!(decode_returns_strict(&returns_of("name()(string,uint256)"), &name).is_err());
        // decimals() returns 6, which is not a bool
        let six = V::Tuple(vec![u(6)]).abi_encode_params();
        assert!(decode_returns_strict(&returns_of("decimals()(bool)"), &six).is_err());
        assert!(decode_returns_strict(&returns_of("decimals()(uint8)"), &six).is_ok());
        // Extra bytes after the values are not accepted silently
        let mut longer = six.clone();
        longer.extend([0u8; 32]);
        assert!(decode_returns_strict(&returns_of("decimals()(uint8)"), &longer).is_err());
    }

    #[test]
    fn return_types_from_signature() {
        assert_eq!(return_types_of("balanceOf(address)"), None);
        assert_eq!(
            return_types_of("balanceOf(address)(uint256)").as_deref(),
            Some("uint256")
        );
        assert_eq!(
            return_types_of("getReserves()(uint112,uint112,uint32)").as_deref(),
            Some("(uint112,uint112,uint32)")
        );
        assert_eq!(
            return_types_of("f((uint256,address))((uint256,address))").as_deref(),
            Some("(uint256,address)")
        );
        assert_eq!(return_types_of("f()()").as_deref(), Some(""));
        // The return form encodes the same calldata as the plain one
        let (_, plain) = super::super::contract::encode_signature_call(
            "balanceOf(address)(uint256)",
            &["0x0000000000000000000000000000000000000001".to_string()],
        )
        .unwrap();
        assert_eq!(hex::encode(&plain[..4]), "70a08231");
    }

    #[test]
    fn decode_output_json_keeps_big_integers_as_strings() {
        use alloy::dyn_abi::DynSolValue;
        use alloy::primitives::U256;

        let data = DynSolValue::Tuple(vec![
            DynSolValue::Uint(U256::MAX, 256),
            DynSolValue::Bool(true),
            DynSolValue::String("hi".to_string()),
        ])
        .abi_encode_params();
        let out = decode_output_json(&data, "(uint256,bool,string)").unwrap();
        assert_eq!(out, serde_json::json!([U256::MAX.to_string(), true, "hi"]));

        let one = DynSolValue::Uint(U256::from(7), 256).abi_encode();
        assert_eq!(
            decode_output_json(&one, "uint256").unwrap(),
            serde_json::json!("7")
        );
    }

    #[test]
    fn multicall_blocks_from_list_and_range() {
        assert_eq!(multicall_blocks("latest", None).unwrap(), ["latest"]);
        assert_eq!(
            multicall_blocks("10, 20,30", None).unwrap(),
            ["10", "20", "30"]
        );
        assert_eq!(
            multicall_blocks("latest", Some("100:130:10")).unwrap(),
            ["100", "110", "120", "130"]
        );
        assert!(multicall_blocks("latest", Some("100:90:10")).is_err());
        assert!(multicall_blocks("latest", Some("100:200:0")).is_err());
        assert!(multicall_blocks("latest", Some("100:200")).is_err());
        assert!(multicall_blocks("latest", Some("0:1000000:1")).is_err());
        // Rejected from the count alone, without building the list
        assert!(multicall_blocks("latest", Some("0:18446744073709551615:1")).is_err());
        assert_eq!(
            multicall_blocks("latest", Some("0:999:1")).unwrap().len(),
            1000
        );
        let long_list = vec!["1"; 1001].join(",");
        assert!(multicall_blocks(&long_list, None).is_err());
    }

    #[test]
    fn call_spec_parses_address_signature_and_args() {
        let spec = parse_call_spec(
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 balanceOf(address)(uint256) 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        )
        .unwrap();
        assert_eq!(hex::encode(&spec.calldata[..4]), "70a08231");
        assert_eq!(spec.calldata.len(), 36);
        assert_eq!(spec.function.outputs.len(), 1);
        assert_eq!(spec.args.len(), 1);

        assert!(parse_call_spec("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48").is_err());
        assert!(parse_call_spec("nope totalSupply()(uint256)").is_err());
        // A bare name has no types to encode with
        assert!(parse_call_spec("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 totalSupply").is_err());
    }

    #[test]
    fn revert_reason_and_delegate_decoding() {
        use alloy::dyn_abi::DynSolValue;
        let mut data = vec![0x08, 0xc3, 0x79, 0xa0];
        data.extend(DynSolValue::String("nope".into()).abi_encode());
        assert_eq!(revert_reason(&data).as_deref(), Some("nope"));
        assert_eq!(revert_reason(&[1, 2, 3, 4, 5]), None);

        let mut code = vec![0xef, 0x01, 0x00];
        code.extend([0x11u8; 20]);
        assert!(eip7702_delegate(&code).is_some());
        assert!(eip7702_delegate(&[0x60, 0x80]).is_none());
    }

    #[test]
    fn multicall_csv_has_one_column_per_output() {
        let reports = vec![serde_json::json!({
            "block": "100",
            "via": "multicall3",
            "results": [
                {"index": 0, "address": "0xA", "function": "getReserves()", "args": [],
                 "success": true, "raw": "0x01", "outputs": ["5", "7", "123"]},
                {"index": 1, "address": "0xA", "function": "f(uint256)", "args": ["9"],
                 "success": false, "raw": "0x", "revert": {"message": "\"nope\""}}
            ]
        })];
        let csv = crate::utils::csv_rows::json_rows_to_csv(
            &multicall_csv_rows(&reports),
            MULTICALL_CSV_COLUMNS,
        )
        .unwrap();
        let lines: Vec<&str> = csv.lines().collect();
        assert_eq!(
            lines[0],
            "block,index,address,function,args,success,output_0,output_1,output_2,raw,revert"
        );
        assert_eq!(lines[1], "100,0,0xA,getReserves(),,true,5,7,123,0x01,");
        assert_eq!(
            lines[2],
            "100,1,0xA,f(uint256),9,false,,,,0x,\"\"\"nope\"\"\""
        );
    }

    #[test]
    fn multicall_table_lines() {
        let report = serde_json::json!({
            "block": "100",
            "via": "multicall3",
            "results": [
                {"index": 0, "address": "0xA", "function": "balanceOf(address)", "args": ["0xB"],
                 "success": true, "raw": "0x01", "outputs": ["1", true]},
                {"index": 1, "address": "0xA", "function": "f()", "args": [],
                 "success": false, "raw": "0x", "revert": {"message": "\"nope\""}}
            ]
        });
        let out = format_multicall_table(&report);
        assert!(out.contains("[0] 0xA balanceOf(0xB) = 1, true"), "{out}");
        assert!(out.contains("[1] 0xA f() = REVERT: \"nope\""), "{out}");
    }

    #[test]
    fn time_given_as_block_is_recognised_and_replaced() {
        use clap::Parser;
        let cli = crate::cli::Cli::try_parse_from([
            "ethcli",
            "rpc",
            "balance",
            "0x00",
            "--block",
            "2024-01-01",
        ])
        .unwrap();
        let crate::cli::Commands::Rpc { mut action, .. } = cli.command else {
            panic!("expected rpc");
        };
        assert_eq!(action_block(&action), Some("2024-01-01"));
        set_action_block(&mut action, "18908895".to_string());
        assert_eq!(action_block(&action), Some("18908895"));
        assert_eq!(action_block_number(&action), Some(18_908_895));

        // A list of blocks and a block range are left alone
        let cli = crate::cli::Cli::try_parse_from([
            "ethcli",
            "rpc",
            "multicall",
            "0x00 f()",
            "--block",
            "1,2",
        ])
        .unwrap();
        let crate::cli::Commands::Rpc { action, .. } = cli.command else {
            panic!("expected rpc");
        };
        assert_eq!(action_block(&action), None);
        assert!(crate::cli::Cli::try_parse_from(["ethcli", "rpc", "block-at", "7d"]).is_ok());
        assert!(crate::cli::Cli::try_parse_from(["ethcli", "rpc", "find-block", "@1"]).is_ok());
    }

    #[test]
    fn test_format_tx_table() {
        let tx = serde_json::json!({
            "hash": "0xabc",
            "blockNumber": "0x10",
            "transactionIndex": "0x2",
            "from": "0x1111111111111111111111111111111111111111",
            "to": null,
            "value": "0xde0b6b3a7640000",
            "nonce": "0x5",
            "gas": "0x5208",
            "input": "0xa9059cbb00"
        });
        let out = format_tx_table(&tx);
        assert!(out.contains("Block:       16"), "{out}");
        assert!(out.contains("(contract creation)"), "{out}");
        assert!(out.contains("1000000000000000000 wei"), "{out}");
        assert!(out.contains("Nonce:       5"), "{out}");
        assert!(out.contains("0xa9059cbb (5 bytes)"), "{out}");

        let pending = serde_json::json!({"hash": "0xabc", "input": "0x"});
        assert!(format_tx_table(&pending).contains("pending"));
    }

    #[test]
    fn test_method_name_aliases_parse() {
        use clap::Parser;
        for args in [
            vec!["ethcli", "rpc", "tx", "0x00"],
            vec!["ethcli", "rpc", "transaction", "0x00"],
            vec!["ethcli", "rpc", "eth_getTransactionByHash", "0x00"],
            vec![
                "ethcli", "rpc", "balance", "0x00", "--block", "5", "-o", "json",
            ],
            vec!["ethcli", "rpc", "eth_getTransactionReceipt", "0x00"],
            vec!["ethcli", "rpc", "eth_blockNumber"],
        ] {
            assert!(
                crate::cli::Cli::try_parse_from(&args).is_ok(),
                "failed to parse {args:?}"
            );
        }
    }

    #[test]
    fn test_format_gwei() {
        assert_eq!(format_gwei(180_000_000), "0.18");
        assert_eq!(format_gwei(25_000_000_000), "25");
        assert_eq!(format_gwei(1), "0.000000001");
        assert_eq!(format_gwei(1_500_000_000), "1.5");
        assert_eq!(format_gwei(0), "0");
    }

    // ==================== parse_block_id tests ====================

    #[test]
    fn test_parse_block_latest() {
        let result = parse_block_id("latest").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Latest));
    }

    #[test]
    fn test_parse_block_latest_uppercase() {
        let result = parse_block_id("LATEST").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Latest));
    }

    #[test]
    fn test_parse_block_pending() {
        let result = parse_block_id("pending").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Pending));
    }

    #[test]
    fn test_parse_block_earliest() {
        let result = parse_block_id("earliest").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Earliest));
    }

    #[test]
    fn test_parse_block_finalized() {
        let result = parse_block_id("finalized").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Finalized));
    }

    #[test]
    fn test_parse_block_safe() {
        let result = parse_block_id("safe").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Safe));
    }

    #[test]
    fn test_parse_block_number() {
        let result = parse_block_id("12345").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Number(12345)));
    }

    #[test]
    fn test_parse_block_number_zero() {
        let result = parse_block_id("0").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Number(0)));
    }

    #[test]
    fn test_parse_block_number_large() {
        let result = parse_block_id("21000000").unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Number(21000000)));
    }

    #[test]
    fn test_parse_block_hash() {
        let hash_str = "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef";
        let result = parse_block_id(hash_str).unwrap();
        let expected_hash = B256::from_str(hash_str).unwrap();
        assert_eq!(result, BlockId::Hash(expected_hash.into()));
    }

    #[test]
    fn test_parse_block_invalid_number() {
        let result = parse_block_id("not_a_number");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_block_invalid_hash_length() {
        // Hash that's too short (not 66 chars)
        let result = parse_block_id("0x1234");
        // This should try to parse as a number and succeed (0x1234 = 4660)
        assert_eq!(
            result.unwrap(),
            BlockId::Number(BlockNumberOrTag::Number(0x1234))
        );
    }

    #[test]
    fn test_parse_block_padded_hex_number() {
        // 64-char hex that represents a small block number should be treated as number, not hash
        // Block 17382257 = 0x1093b71 padded to 64 hex digits
        let padded = "0x0000000000000000000000000000000000000000000000000000000001093b71";
        assert_eq!(padded.len(), 66); // 0x + 64 hex chars
        let result = parse_block_id(padded).unwrap();
        assert_eq!(result, BlockId::Number(BlockNumberOrTag::Number(17382257)));
    }

    #[test]
    fn test_parse_block_real_hash() {
        // A real block hash has high entropy and will have a large numeric value (> 2^48)
        // This is a real mainnet block hash
        let hash = "0xd4e56740f876aef8c010b86a40d5f56745a118d0906a34e69aec8c0db1cb8fa3";
        let result = parse_block_id(hash).unwrap();
        // Should be parsed as hash, not number
        assert!(matches!(result, BlockId::Hash(_)));
    }

    // ==================== decode_output tests ====================

    #[test]
    fn test_decode_output_uint256() {
        // 1000 encoded as uint256
        let data = hex::decode("00000000000000000000000000000000000000000000000000000000000003e8")
            .unwrap();
        let result = decode_output(&data, "uint256").unwrap();
        assert_eq!(result, "1000");
    }

    #[test]
    fn test_decode_output_bool_true() {
        // true encoded as bool
        let data = hex::decode("0000000000000000000000000000000000000000000000000000000000000001")
            .unwrap();
        let result = decode_output(&data, "bool").unwrap();
        assert!(result.contains("true"));
    }

    #[test]
    fn test_decode_output_bool_false() {
        // false encoded as bool
        let data = hex::decode("0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap();
        let result = decode_output(&data, "bool").unwrap();
        assert!(result.contains("false"));
    }

    #[test]
    fn test_decode_output_address() {
        // Address encoded
        let data = hex::decode("000000000000000000000000a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
            .unwrap();
        let result = decode_output(&data, "address").unwrap();
        // Should contain the address (case-insensitive check)
        assert!(result
            .to_lowercase()
            .contains("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"));
    }

    #[test]
    fn test_decode_output_invalid_type() {
        let data = vec![0u8; 32];
        let result = decode_output(&data, "invalid_type");
        assert!(result.is_err());
    }
}
