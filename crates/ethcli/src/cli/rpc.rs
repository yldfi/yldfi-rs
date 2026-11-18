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

#[derive(Subcommand)]
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
  ethcli rpc balance 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045 --block 21000000 -o json"#)]
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

        /// Function signature (e.g., "balanceOf(address)")
        #[arg(long, short, conflicts_with = "data", value_name = "SIG")]
        sig: Option<String>,

        /// Function arguments (use with --sig)
        #[arg(long, short = 'a', num_args = 1.., value_delimiter = ' ', value_name = "ARG")]
        args: Option<Vec<String>>,

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
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

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
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

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
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

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
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

        /// Block number or "latest" (default: latest)
        #[arg(long, short, default_value = "latest", value_name = "BLOCK")]
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
        match run_action(action, &endpoint, quiet).await {
            Ok(()) => return Ok(()),
            Err(e) => {
                let retry = e.is::<NotFound>() || crate::rpc::is_failover_error(&format!("{e:#}"));
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

/// Nothing at that block/hash on this endpoint
#[derive(Debug)]
struct NotFound(&'static str);

impl std::fmt::Display for NotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for NotFound {}

/// The numeric block an action reads at, if any (tags: None)
fn action_block_number(action: &RpcCommands) -> Option<u64> {
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
async fn run_action(action: &RpcCommands, endpoint: &Endpoint, quiet: bool) -> anyhow::Result<()> {
    let provider = endpoint.provider();

    match action {
        RpcCommands::Call {
            to,
            data,
            sig,
            args,
            block,
            decode,
            format,
        } => {
            let to_addr =
                Address::from_str(to).map_err(|e| anyhow::anyhow!("Invalid address: {}", e))?;

            // Build calldata from either raw data or signature + args
            let calldata = if let Some(signature) = sig {
                // Encode from function signature
                let func_args = args.as_deref().unwrap_or(&[]);
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

            let result = provider
                .call(tx)
                .block(block_id)
                .await
                .map_err(|e| anyhow::anyhow!("Call failed: {}", e))?;

            if !format.is_table() {
                let mut out = serde_json::json!({
                    "to": to_addr.to_checksum(None),
                    "block": block,
                    "raw": format!("0x{}", hex::encode(&result)),
                });
                if let Some(type_sig) = decode {
                    out["type"] = serde_json::json!(type_sig);
                    out["decoded"] = decode_output_json(&result, type_sig)?;
                }
                print_json(&out, *format)?;
            } else if let Some(type_sig) = decode {
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

            if format.is_table() {
                println!("{:#x}", value);
            } else {
                let out = serde_json::json!({
                    "address": addr.to_checksum(None),
                    "slot": format!("{slot_val:#x}"),
                    "block": block,
                    "value": format!("{:#066x}", value),
                    "decimal": value.to_string(),
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
                    if matches!(format, OutputFormat::Json) {
                        println!("{}", serde_json::to_string_pretty(&r)?);
                    } else if matches!(format, OutputFormat::Ndjson) {
                        println!("{}", serde_json::to_string(&r)?);
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
                    }
                }
                None => return Err(NotFound("Receipt not found (tx may be pending)").into()),
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

/// The address an EOA delegates to under EIP-7702, read from its code
/// (`0xef0100` followed by the 20-byte address)
fn eip7702_delegate(code: &[u8]) -> Option<Address> {
    (code.len() == 23 && code[..3] == [0xef, 0x01, 0x00]).then(|| Address::from_slice(&code[3..]))
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
