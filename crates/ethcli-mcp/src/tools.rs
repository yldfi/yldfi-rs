//! MCP tool implementations for ethcli
//!
//! Each tool wraps an ethcli command and exposes it via MCP.
//! Tools use subprocess execution for full CLI compatibility.
//!
//! Total: 236 tools covering all ethcli subcommands.

use crate::executor::{ArgsBuilder, ExecutionError, ValidationError};

/// Error type for MCP tools
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    /// Command execution failed
    #[error("Command failed: {0}")]
    CommandFailed(String),

    /// Input validation failed
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Rate limited
    #[error("Rate limited: {0}")]
    RateLimited(String),

    /// Timeout
    #[error("Timeout: {0}")]
    Timeout(String),
}

impl From<String> for ToolError {
    fn from(s: String) -> Self {
        ToolError::CommandFailed(s)
    }
}

impl From<ExecutionError> for ToolError {
    fn from(e: ExecutionError) -> Self {
        match e {
            ExecutionError::Validation(v) => ToolError::InvalidInput(v.to_string()),
            ExecutionError::RateLimited => {
                ToolError::RateLimited("Too many concurrent requests".to_string())
            }
            ExecutionError::Timeout => ToolError::Timeout("Command timed out".to_string()),
            other => ToolError::CommandFailed(other.to_string()),
        }
    }
}

impl From<ValidationError> for ToolError {
    fn from(e: ValidationError) -> Self {
        ToolError::InvalidInput(e.to_string())
    }
}

/// Extension trait to convert tool results to MCP response strings
pub trait ToResponse {
    /// Convert a Result to a String response for MCP
    fn to_response(self) -> String;
}

impl<E: std::fmt::Display> ToResponse for Result<String, E> {
    fn to_response(self) -> String {
        match self {
            Ok(r) => r,
            Err(e) => format!("Error: {}", e),
        }
    }
}

// =============================================================================
// LOGS
// =============================================================================

pub async fn logs(
    contract: &str,
    event: Option<&str>,
    from_block: Option<&str>,
    to_block: Option<&str>,
    topics: Option<Vec<String>>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("logs")
        .opt("-c", Some(contract))
        .opt("-e", event)
        .opt("-f", from_block)
        .opt("-t", to_block)
        .chain(chain)
        .format_json();

    if let Some(t) = topics {
        for topic in t {
            builder = builder.opt("-e", Some(&topic));
        }
    }

    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// TX (Transaction Analysis)
// =============================================================================

pub async fn tx_analyze(
    hash: &str,
    chain: Option<&str>,
    block: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tx").arg(hash).chain(chain);

    if let Some(b) = block {
        builder = builder.opt("-b", Some(&b.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// ACCOUNT (8 subcommands)
// =============================================================================

pub async fn account_info(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("info")
        .arg(address)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_balance(
    addresses: &[String],
    chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("account").subcommand("balance");

    // Add addresses as positional arguments
    for addr in addresses {
        builder = builder.arg(addr);
    }

    builder
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_txs(
    address: &str,
    chain: Option<&str>,
    page: Option<u32>,
    limit: Option<u32>,
    sort: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("account")
        .subcommand("txs")
        .arg(address)
        .chain(chain)
        .format_json();

    if let Some(p) = page {
        builder = builder.opt("--page", Some(&p.to_string()));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }
    if let Some(s) = sort {
        builder = builder.opt("--sort", Some(s));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn account_internal_txs(
    address: &str,
    chain: Option<&str>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("account")
        .subcommand("internal-txs")
        .arg(address)
        .chain(chain)
        .format_json();

    if let Some(p) = page {
        builder = builder.opt("--page", Some(&p.to_string()));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn account_erc20(
    address: &str,
    chain: Option<&str>,
    token: Option<&str>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("account")
        .subcommand("erc20")
        .arg(address)
        .chain(chain)
        .opt("--token", token)
        .format_json();

    if let Some(p) = page {
        builder = builder.opt("--page", Some(&p.to_string()));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn account_erc721(
    address: &str,
    chain: Option<&str>,
    token: Option<&str>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("account")
        .subcommand("erc721")
        .arg(address)
        .chain(chain)
        .opt("--token", token)
        .format_json();

    if let Some(p) = page {
        builder = builder.opt("--page", Some(&p.to_string()));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn account_erc1155(
    address: &str,
    chain: Option<&str>,
    token: Option<&str>,
    page: Option<u32>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("account")
        .subcommand("erc1155")
        .arg(address)
        .chain(chain)
        .opt("--token", token)
        .format_json();

    if let Some(p) = page {
        builder = builder.opt("--page", Some(&p.to_string()));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn account_mined_blocks(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("mined-blocks")
        .arg(address)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ADDRESS (7 subcommands) - Address book management
// =============================================================================

pub async fn address_add(
    name: &str,
    address: &str,
    description: Option<&str>,
    tags: &[String],
    for_chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("address")
        .subcommand("add")
        .arg(name)
        .arg(address)
        .opt("-d", description)
        .opt("--for-chain", for_chain);

    for tag in tags {
        builder = builder.opt("-t", Some(tag.as_str()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn address_remove(name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("remove")
        .arg(name)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn address_list() -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("list")
        .opt("-o", Some("json"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn address_get(name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("get")
        .arg(name)
        .opt("-o", Some("json"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn address_search(query: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("search")
        .arg(query)
        .opt("-o", Some("json"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn address_import(file: &str, overwrite: bool) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("address").subcommand("import").arg(file);

    if overwrite {
        builder = builder.arg("--overwrite");
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn address_export(output: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("export")
        .opt("-o", output)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// BLACKLIST (7 subcommands) - Sanctions/blacklist checking
// =============================================================================

pub async fn blacklist_add(
    address: &str,
    symbol: Option<&str>,
    reason: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("add")
        .arg(address)
        .opt("-s", symbol)
        .opt("-r", reason)
        .opt("-c", chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn blacklist_remove(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("remove")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn blacklist_list() -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("list")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn blacklist_check(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("check")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn blacklist_scan(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("scan")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn blacklist_scan_portfolio(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("scan-portfolio")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn blacklist_path() -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("path")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// CONTRACT (4 subcommands)
// =============================================================================

pub async fn contract_abi(
    address: &str,
    chain: Option<&str>,
    output: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("abi")
        .arg(address)
        .chain(chain)
        .opt("-o", output)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn contract_source(
    address: &str,
    chain: Option<&str>,
    output: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("source")
        .arg(address)
        .chain(chain)
        .opt("-o", output)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn contract_creation(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("creation")
        .arg(address)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn contract_call(
    address: &str,
    sig: &str,
    args: Vec<String>,
    chain: Option<&str>,
    block: Option<&str>,
    rpc_url: Option<&str>,
    human: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("contract")
        .subcommand("call")
        .arg(address)
        .arg(sig)
        .chain(chain)
        .opt("-b", block)
        .opt("--rpc-url", rpc_url);

    if human {
        builder = builder.arg("-H");
    }

    for arg in args {
        builder = builder.arg(&arg);
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn contract_selectors(
    address: &str,
    chain: Option<&str>,
    lookup: bool,
    follow_proxy: bool,
) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("selectors")
        .arg(address)
        .chain(chain)
        .opt_flag("--lookup", lookup)
        .opt_flag("--follow-proxy", follow_proxy)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn contract_disassemble(
    address: &str,
    chain: Option<&str>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("contract")
        .subcommand("disassemble")
        .arg(address)
        .chain(chain)
        .format_json();

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn contract_opcodes(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("opcodes")
        .arg(address)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub struct ContractAnalyzeArgs<'a> {
    pub address: &'a str,
    pub chain: Option<&'a str>,
    pub include_disassembly: bool,
    pub limit: Option<u32>,
    pub lookup: bool,
    pub follow_proxy: bool,
    pub dispatcher: bool,
    pub checks: bool,
}

pub async fn contract_analyze(args: ContractAnalyzeArgs<'_>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("contract")
        .subcommand("analyze")
        .arg(args.address)
        .chain(args.chain)
        .opt_flag("--include-disassembly", args.include_disassembly)
        .opt_flag("--lookup", args.lookup)
        .opt_flag("--follow-proxy", args.follow_proxy)
        .opt_flag("--dispatcher", args.dispatcher)
        .opt_flag("--checks", args.checks)
        .format_json();

    if let Some(l) = args.limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// TOKEN (3 subcommands)
// =============================================================================

pub async fn token_info(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("token")
        .subcommand("info")
        .arg(address)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn token_holders(
    address: &str,
    chain: Option<&str>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("token")
        .subcommand("holders")
        .arg(address)
        .chain(chain)
        .format_json();

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn token_balance(
    tokens: &[String],
    holders: &[String],
    tag: Option<&str>,
    show_zero: bool,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("token").subcommand("balance");

    // Add tokens as positional arguments
    for token in tokens {
        builder = builder.arg(token);
    }

    // Add holders with --holder flag (can be specified multiple times)
    for holder in holders {
        builder = builder.opt("--holder", Some(holder));
    }

    builder = builder
        .opt("--tag", tag)
        .opt_flag("--show-zero", show_zero)
        .chain(chain)
        .format_json();

    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// GAS (2 subcommands)
// =============================================================================

pub async fn gas_oracle(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("gas")
        .subcommand("oracle")
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gas_estimate(gwei: u64, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("gas")
        .subcommand("estimate")
        .arg(&gwei.to_string())
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// SIG (4 subcommands) - Signature lookup
// =============================================================================

pub async fn sig_fn(selector: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("sig")
        .subcommand("fn")
        .arg(selector)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn sig_event(topic: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("sig")
        .subcommand("event")
        .arg(topic)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn sig_cache_stats() -> Result<String, ToolError> {
    ArgsBuilder::new("sig")
        .subcommand("cache-stats")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn sig_cache_clear() -> Result<String, ToolError> {
    ArgsBuilder::new("sig")
        .subcommand("cache-clear")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// CAST (14 subcommands) - Conversions
// =============================================================================

pub async fn cast_to_wei(amount: &str, unit: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("cast").subcommand("to-wei").arg(amount);
    if let Some(u) = unit {
        builder = builder.arg(u);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn cast_from_wei(wei: &str, unit: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("cast").subcommand("from-wei").arg(wei);
    if let Some(u) = unit {
        builder = builder.arg(u);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn cast_to_hex(value: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("to-hex")
        .arg(value)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_to_dec(value: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("to-dec")
        .arg(value)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_keccak(data: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("keccak")
        .arg(data)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_sig(signature: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("sig")
        .arg(signature)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_topic(signature: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("topic")
        .arg(signature)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_checksum(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("checksum")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_compute_address(deployer: &str, nonce: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("compute-address")
        .arg(deployer)
        .arg(nonce)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_create2(
    deployer: &str,
    salt: &str,
    init_code_hash: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("create2")
        .arg(deployer)
        .arg(salt)
        .arg(init_code_hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_concat(values: Vec<String>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("cast").subcommand("concat");
    for v in values {
        builder = builder.arg(&v);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn cast_to_bytes32(value: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("to-bytes32")
        .arg(value)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cast_abi_encode(sig: &str, args: Vec<String>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("cast").subcommand("abi-encode").arg(sig);
    for arg in args {
        builder = builder.arg(&arg);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn cast_abi_decode(sig: &str, data: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("cast")
        .subcommand("abi-decode")
        .arg(sig)
        .arg(data)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// RPC (9 subcommands)
// =============================================================================

pub async fn rpc_call(
    to: &str,
    data: &str,
    chain: Option<&str>,
    block: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("call")
        .arg(to)
        .arg(data)
        .chain(chain)
        .opt("--block", block)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn rpc_block(
    block: &str,
    chain: Option<&str>,
    full: bool,
    rpc_url: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("rpc")
        .subcommand("block")
        .arg(block)
        .chain(chain)
        .opt("--rpc-url", rpc_url)
        .format_json();

    if full {
        builder = builder.arg("-f");
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn rpc_storage(
    address: &str,
    slot: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("storage")
        .arg(address)
        .arg(slot)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn rpc_code(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("code")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn rpc_nonce(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("nonce")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn rpc_receipt(hash: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("receipt")
        .arg(hash)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn rpc_block_number(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("block-number")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn rpc_chain_id(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("chain-id")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn rpc_gas_price(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("gas-price")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ENS (4 subcommands)
// =============================================================================

pub async fn ens_resolve(name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ens")
        .subcommand("resolve")
        .arg(name)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn ens_lookup(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ens")
        .subcommand("lookup")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn ens_resolver(name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ens")
        .subcommand("resolver")
        .arg(name)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn ens_namehash(name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ens")
        .subcommand("namehash")
        .arg(name)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// SIMULATE (8 subcommands)
// =============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn simulate_call(
    contract: &str,
    sig: Option<&str>,
    data: Option<&str>,
    args: Vec<String>,
    chain: Option<&str>,
    // Transaction params
    from: Option<&str>,
    value: Option<&str>,
    block: Option<&str>,
    gas: Option<&str>,
    gas_price: Option<&str>,
    // State overrides
    balance_overrides: &[String],
    storage_overrides: &[String],
    code_overrides: &[String],
    nonce_overrides: &[String],
    // Block overrides
    block_timestamp: Option<u64>,
    block_number_override: Option<u64>,
    block_gas_limit: Option<&str>,
    block_coinbase: Option<&str>,
    block_difficulty: Option<&str>,
    block_base_fee: Option<&str>,
    transaction_index: Option<u32>,
    // L2 options
    l1_block_number: Option<u64>,
    l1_timestamp: Option<u64>,
    l1_message_sender: Option<&str>,
    deposit_tx: bool,
    system_tx: bool,
    // Simulation options
    via: Option<&str>,
    rpc_url: Option<&str>,
    trace: bool,
    decode_internal: bool,
    disable_labels: bool,
    labels: &[String],
    evm_version: Option<&str>,
    with_local_artifacts: bool,
    rpc_timeout: Option<u64>,
    rpc_headers: &[String],
    no_proxy: bool,
    fork_urls: &[String],
    fork_block_number: Option<&str>,
    fork_transaction_hash: Option<&str>,
    fork_chain_id: Option<u64>,
    fork_headers: &[String],
    hardfork: Option<&str>,
    network: Option<&str>,
    no_rate_limit: bool,
    no_storage_caching: bool,
    fork_timeout_ms: Option<u64>,
    fork_retries: Option<u32>,
    disable_block_gas_limit: bool,
    enable_tx_gas_limit: bool,
    estimate_gas: bool,
    generate_access_list: bool,
    access_list: Option<&str>,
    simulation_type: Option<&str>,
    network_id: Option<u64>,
    save: bool,
    // Tenderly options
    tenderly_key: Option<&str>,
    tenderly_account: Option<&str>,
    tenderly_project: Option<&str>,
    // Alchemy options
    alchemy_key: Option<&str>,
    alchemy_network: Option<&str>,
    // Debug options
    dry_run: Option<&str>,
    show_secrets: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("simulate")
        .subcommand("call")
        .arg(contract)
        .opt("--sig", sig)
        .opt("--data", data)
        .chain(chain)
        // Transaction params
        .opt("--from", from)
        .opt("--value", value)
        .opt("--block", block)
        .opt("--gas", gas)
        .opt("--gas-price", gas_price)
        // Block overrides
        .opt("--block-gas-limit", block_gas_limit)
        .opt("--block-coinbase", block_coinbase)
        .opt("--block-difficulty", block_difficulty)
        .opt("--block-base-fee", block_base_fee)
        // L2 options
        .opt("--l1-message-sender", l1_message_sender)
        .opt_flag("--deposit-tx", deposit_tx)
        .opt_flag("--system-tx", system_tx)
        // Simulation options
        .opt("--via", via)
        .opt("--rpc-url", rpc_url)
        .opt_flag("--trace", trace)
        .opt_flag("--decode-internal", decode_internal)
        .opt_flag("--disable-labels", disable_labels)
        .opt("--evm-version", evm_version)
        .opt_flag("--with-local-artifacts", with_local_artifacts)
        .opt_flag("--no-proxy", no_proxy)
        .opt("--fork-block-number", fork_block_number)
        .opt("--fork-transaction-hash", fork_transaction_hash)
        .opt("--hardfork", hardfork)
        .opt("--network", network)
        .opt_flag("--no-rate-limit", no_rate_limit)
        .opt_flag("--no-storage-caching", no_storage_caching)
        .opt_flag("--disable-block-gas-limit", disable_block_gas_limit)
        .opt_flag("--enable-tx-gas-limit", enable_tx_gas_limit)
        .opt_flag("--estimate-gas", estimate_gas)
        .opt_flag("--generate-access-list", generate_access_list)
        .opt("--access-list", access_list)
        .opt("--simulation-type", simulation_type)
        .opt_flag("--save", save)
        // Tenderly options
        .opt("--tenderly-key", tenderly_key)
        .opt("--tenderly-account", tenderly_account)
        .opt("--tenderly-project", tenderly_project)
        // Alchemy options
        .opt("--alchemy-key", alchemy_key)
        .opt("--alchemy-network", alchemy_network)
        // Debug options
        .opt("--dry-run", dry_run)
        .opt_flag("--show-secrets", show_secrets);

    // Numeric options
    if let Some(ts) = block_timestamp {
        builder = builder.opt("--block-timestamp", Some(&ts.to_string()));
    }
    if let Some(bn) = block_number_override {
        builder = builder.opt("--block-number-override", Some(&bn.to_string()));
    }
    if let Some(ti) = transaction_index {
        builder = builder.opt("--transaction-index", Some(&ti.to_string()));
    }
    if let Some(l1bn) = l1_block_number {
        builder = builder.opt("--l1-block-number", Some(&l1bn.to_string()));
    }
    if let Some(l1ts) = l1_timestamp {
        builder = builder.opt("--l1-timestamp", Some(&l1ts.to_string()));
    }
    if let Some(nid) = network_id {
        builder = builder.opt("--network-id", Some(&nid.to_string()));
    }
    if let Some(timeout) = rpc_timeout {
        builder = builder.opt("--rpc-timeout", Some(&timeout.to_string()));
    }
    if let Some(chain_id) = fork_chain_id {
        builder = builder.opt("--fork-chain-id", Some(&chain_id.to_string()));
    }
    if let Some(timeout_ms) = fork_timeout_ms {
        builder = builder.opt("--fork-timeout-ms", Some(&timeout_ms.to_string()));
    }
    if let Some(retries) = fork_retries {
        builder = builder.opt("--fork-retries", Some(&retries.to_string()));
    }

    // State overrides (repeatable)
    for bo in balance_overrides {
        builder = builder.opt("--balance-override", Some(bo));
    }
    for so in storage_overrides {
        builder = builder.opt("--storage-override", Some(so));
    }
    for co in code_overrides {
        builder = builder.opt("--code-override", Some(co));
    }
    for no in nonce_overrides {
        builder = builder.opt("--nonce-override", Some(no));
    }
    for label in labels {
        builder = builder.opt("--label", Some(label));
    }
    for header in rpc_headers {
        builder = builder.opt("--rpc-header", Some(header));
    }
    for fork_url in fork_urls {
        builder = builder.opt("--fork-url", Some(fork_url));
    }
    for fork_header in fork_headers {
        builder = builder.opt("--fork-header", Some(fork_header));
    }

    // Function arguments
    for arg in args {
        builder = builder.arg(&arg);
    }

    builder.execute().await.map_err(ToolError::from)
}

pub struct SimulateTxOptions<'a> {
    pub hash: &'a str,
    pub chain: Option<&'a str>,
    pub via: Option<&'a str>,
    pub rpc_url: Option<&'a str>,
    pub trace: bool,
    pub debug: bool,
    pub quick: bool,
    pub decode_internal: bool,
    pub trace_depth: Option<u32>,
    pub replay_system_txs: bool,
    pub disable_labels: bool,
    pub labels: &'a [String],
    pub evm_version: Option<&'a str>,
    pub with_local_artifacts: bool,
    pub no_proxy: bool,
    pub rpc_timeout: Option<u64>,
    pub rpc_headers: &'a [String],
    pub enable_tx_gas_limit: bool,
    pub disable_block_gas_limit: bool,
    pub etherscan_api_key: Option<&'a str>,
}

pub async fn simulate_tx(input: SimulateTxOptions<'_>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("simulate")
        .subcommand("tx")
        .arg(input.hash)
        .chain(input.chain)
        .opt("--via", input.via)
        .opt("--rpc-url", input.rpc_url)
        .opt_flag("--trace", input.trace)
        .opt_flag("--debug", input.debug)
        .opt_flag("--quick", input.quick)
        .opt_flag("--decode-internal", input.decode_internal)
        .opt_flag("--replay-system-txs", input.replay_system_txs)
        .opt_flag("--disable-labels", input.disable_labels)
        .opt("--evm-version", input.evm_version)
        .opt_flag("--with-local-artifacts", input.with_local_artifacts)
        .opt_flag("--no-proxy", input.no_proxy)
        .opt_flag("--enable-tx-gas-limit", input.enable_tx_gas_limit)
        .opt_flag("--disable-block-gas-limit", input.disable_block_gas_limit)
        .opt("--etherscan-api-key", input.etherscan_api_key);

    if let Some(depth) = input.trace_depth {
        builder = builder.opt("--trace-depth", Some(&depth.to_string()));
    }
    if let Some(timeout) = input.rpc_timeout {
        builder = builder.opt("--rpc-timeout", Some(&timeout.to_string()));
    }
    for label in input.labels {
        builder = builder.opt("--label", Some(label));
    }
    for header in input.rpc_headers {
        builder = builder.opt("--rpc-header", Some(header));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn simulate_bundle(txs: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("simulate")
        .subcommand("bundle")
        .opt("--txs", Some(txs))
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn simulate_list() -> Result<String, ToolError> {
    ArgsBuilder::new("simulate")
        .subcommand("list")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn simulate_get(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("simulate")
        .subcommand("get")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn simulate_info(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("simulate")
        .subcommand("info")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn simulate_share(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("simulate")
        .subcommand("share")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn simulate_unshare(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("simulate")
        .subcommand("unshare")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// TENDERLY (8 subcommands)
// =============================================================================

pub async fn tenderly_simulate(
    contract: &str,
    data: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("simulate")
        .subcommand("call")
        .arg(contract)
        .opt("--data", Some(data))
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets() -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_wallets() -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("wallets")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_contracts() -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("contracts")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_alerts() -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("alerts")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_actions() -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("actions")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_networks() -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("networks")
        .subcommand("list")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_channels() -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("channels")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// TENDERLY VNETS - Full CRUD + Simulate + Admin
// =============================================================================

pub async fn tenderly_vnets_create(
    slug: &str,
    name: &str,
    network_id: u64,
    block_number: Option<u64>,
    chain_id: Option<u64>,
    sync_state: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("create")
        .opt("--slug", Some(slug))
        .opt("--name", Some(name))
        .opt("--network-id", Some(&network_id.to_string()));
    if let Some(bn) = block_number {
        builder = builder.opt("--block-number", Some(&bn.to_string()));
    }
    if let Some(cid) = chain_id {
        builder = builder.opt("--chain-id", Some(&cid.to_string()));
    }
    if sync_state {
        builder = builder.arg("--sync-state");
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_get(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("get")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_delete(ids: &[String], all: bool) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("delete");
    if all {
        builder = builder.arg("--all");
    } else {
        for id in ids {
            builder = builder.arg(id);
        }
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_update(
    id: &str,
    name: Option<&str>,
    slug: Option<&str>,
    sync_state: Option<bool>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("update")
        .arg(id);
    builder = builder.opt("--name", name);
    builder = builder.opt("--slug", slug);
    if let Some(ss) = sync_state {
        builder = builder.opt("--sync-state", Some(&ss.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_fork(
    source: &str,
    slug: &str,
    name: &str,
    block_number: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("fork")
        .opt("--source", Some(source))
        .opt("--slug", Some(slug))
        .opt("--name", Some(name));
    if let Some(bn) = block_number {
        builder = builder.opt("--block-number", Some(&bn.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_rpc(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("rpc")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_transactions(
    id: &str,
    page: Option<u32>,
    per_page: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("transactions")
        .arg(id);
    if let Some(p) = page {
        builder = builder.opt("--page", Some(&p.to_string()));
    }
    if let Some(pp) = per_page {
        builder = builder.opt("--per-page", Some(&pp.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_get_transaction(vnet: &str, hash: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("get-transaction")
        .opt("--vnet", Some(vnet))
        .arg(hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_send(
    vnet: &str,
    from: &str,
    to: &str,
    data: Option<&str>,
    value: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("send")
        .opt("--vnet", Some(vnet))
        .opt("--from", Some(from))
        .opt("--to", Some(to));
    builder = builder.opt("--data", data);
    if let Some(v) = value {
        builder = builder.opt("--value", Some(v));
    }
    builder.execute().await.map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn tenderly_vnets_simulate(
    vnet: &str,
    from: &str,
    to: Option<&str>,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<u64>,
    gas_price: Option<&str>,
    max_fee_per_gas: Option<&str>,
    max_priority_fee_per_gas: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("simulate")
        .opt("--vnet", Some(vnet))
        .opt("--from", Some(from))
        .opt("--to", to);
    builder = builder.opt("--data", data);
    if let Some(v) = value {
        builder = builder.opt("--value", Some(v));
    }
    if let Some(g) = gas {
        builder = builder.opt("--gas", Some(&g.to_string()));
    }
    builder = builder.opt("--gas-price", gas_price);
    builder = builder.opt("--max-fee-per-gas", max_fee_per_gas);
    builder = builder.opt("--max-priority-fee-per-gas", max_priority_fee_per_gas);
    builder.execute().await.map_err(ToolError::from)
}

// --- VNet Admin operations ---

pub async fn tenderly_vnets_admin_set_balance(
    vnet: &str,
    address: &str,
    amount: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-balance")
        .arg(address)
        .arg(amount)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_add_balance(
    vnet: &str,
    address: &str,
    amount: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("add-balance")
        .arg(address)
        .arg(amount)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_set_erc20_balance(
    vnet: &str,
    token: &str,
    wallet: &str,
    amount: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-erc20-balance")
        .opt("--token", Some(token))
        .opt("--wallet", Some(wallet))
        .arg(amount)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_set_max_erc20_balance(
    vnet: &str,
    token: &str,
    wallet: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-max-erc20-balance")
        .opt("--token", Some(token))
        .opt("--wallet", Some(wallet))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_increase_time(
    vnet: &str,
    seconds: u64,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("increase-time")
        .arg(&seconds.to_string())
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_set_timestamp(
    vnet: &str,
    timestamp: u64,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-timestamp")
        .arg(&timestamp.to_string())
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_set_timestamp_no_mine(
    vnet: &str,
    timestamp: u64,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-timestamp-no-mine")
        .arg(&timestamp.to_string())
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_increase_blocks(
    vnet: &str,
    blocks: u64,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("increase-blocks")
        .arg(&blocks.to_string())
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_snapshot(vnet: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("snapshot")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_revert(
    vnet: &str,
    snapshot_id: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("revert")
        .arg(snapshot_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_set_storage(
    vnet: &str,
    address: &str,
    slot: &str,
    value: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-storage")
        .opt("--address", Some(address))
        .opt("--slot", Some(slot))
        .opt("--value", Some(value))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_set_code(
    vnet: &str,
    address: &str,
    code: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-code")
        .opt("--address", Some(address))
        .opt("--code", Some(code))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_send_tx(
    vnet: &str,
    from: &str,
    to: Option<&str>,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("send-tx")
        .opt("--from", Some(from));
    builder = builder.opt("--to", to);
    builder = builder.opt("--data", data);
    builder = builder.opt("--value", value);
    builder = builder.opt("--gas", gas);
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_get_latest(vnet: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("get-latest")
        .execute()
        .await
        .map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn tenderly_vnets_admin_simulate_tx(
    vnet: &str,
    from: &str,
    to: Option<&str>,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<&str>,
    block: &str,
    state_overrides: Option<&str>,
    block_overrides: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("simulate-tx")
        .opt("--from", Some(from));
    builder = builder.opt("--to", to);
    builder = builder.opt("--data", data);
    builder = builder.opt("--value", value);
    builder = builder.opt("--gas", gas);
    builder = builder.opt("--block", Some(block));
    builder = builder.opt("--state-overrides", state_overrides);
    builder = builder.opt("--block-overrides", block_overrides);
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_simulate_bundle(
    vnet: &str,
    txs: &str,
    state_overrides: Option<&str>,
    block_overrides: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("simulate-bundle")
        .arg(txs);
    builder = builder.opt("--state-overrides", state_overrides);
    builder = builder.opt("--block-overrides", block_overrides);
    builder.execute().await.map_err(ToolError::from)
}

// --- Tenderly Actions Batch ---

pub async fn tenderly_actions_stop_many(ids: &[String]) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("actions")
        .subcommand("stop-many");
    for id in ids {
        builder = builder.arg(id);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_actions_resume_many(ids: &[String]) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("actions")
        .subcommand("resume-many");
    for id in ids {
        builder = builder.arg(id);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_actions_get_call(id: &str, execution_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("actions")
        .subcommand("get-call")
        .arg(id)
        .arg(execution_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Tenderly Alerts Batch ---

pub async fn tenderly_alerts_update(
    id: &str,
    name: &str,
    alert_type: &str,
    network: Option<&str>,
    addresses: Option<Vec<String>>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("alerts")
        .subcommand("update")
        .arg(id);
    builder = builder.opt("--name", Some(name));
    builder = builder.opt("--alert-type", Some(alert_type));
    builder = builder.opt("--network", network);
    if let Some(addrs) = addresses {
        for addr in &addrs {
            builder = builder.opt("--address", Some(addr));
        }
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_alerts_add_destination(
    id: &str,
    destination_type: &str,
    destination_id: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("alerts")
        .subcommand("add-destination")
        .arg(id)
        .opt("--destination-type", Some(destination_type))
        .opt("--destination-id", Some(destination_id))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn tenderly_alerts_remove_destination(
    id: &str,
    destination_id: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("tenderly")
        .subcommand("alerts")
        .subcommand("remove-destination")
        .arg(id)
        .arg(destination_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Tenderly Contracts Batch ---

pub async fn tenderly_contracts_update(
    address: &str,
    network: Option<&str>,
    name: Option<&str>,
    tags: &[String],
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("contracts")
        .subcommand("update")
        .arg(address);
    builder = builder.opt("--network", network);
    builder = builder.opt("--name", name);
    for tag in tags {
        builder = builder.opt("--tag", Some(tag));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_contracts_remove_tag(
    address: &str,
    network: Option<&str>,
    tag: &str,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("contracts")
        .subcommand("remove-tag")
        .arg(address);
    builder = builder.opt("--network", network);
    builder = builder.arg(tag);
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_contracts_bulk_tag(
    tag: &str,
    contract_ids: &[String],
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("contracts")
        .subcommand("bulk-tag")
        .arg(tag);
    for id in contract_ids {
        builder = builder.arg(id);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_contracts_encode_state(
    network: Option<&str>,
    state_json: &str,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("contracts")
        .subcommand("encode-state");
    builder = builder.opt("--network", network);
    builder = builder.arg(state_json);
    builder.execute().await.map_err(ToolError::from)
}

// --- Tenderly VNets Admin Batch ---

pub async fn tenderly_vnets_admin_set_balances(
    vnet: &str,
    addresses: &[String],
    amount: &str,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("set-balances");
    for addr in addresses {
        builder = builder.arg(addr);
    }
    builder = builder.opt("--amount", Some(amount));
    builder.execute().await.map_err(ToolError::from)
}

pub async fn tenderly_vnets_admin_add_balances(
    vnet: &str,
    addresses: &[String],
    amount: &str,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("add-balances");
    for addr in addresses {
        builder = builder.arg(addr);
    }
    builder = builder.opt("--amount", Some(amount));
    builder.execute().await.map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn tenderly_vnets_admin_create_access_list(
    vnet: &str,
    from: &str,
    to: &str,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<&str>,
    block: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("tenderly")
        .subcommand("vnets")
        .subcommand("admin")
        .opt("--vnet", Some(vnet))
        .subcommand("create-access-list")
        .opt("--from", Some(from))
        .opt("--to", Some(to));
    builder = builder.opt("--data", data);
    builder = builder.opt("--value", value);
    builder = builder.opt("--gas", gas);
    builder = builder.opt("--block", block);
    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// PRICE, PORTFOLIO, NFTS, YIELDS, DOCTOR (standalone commands)
// =============================================================================

pub async fn price(token: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("price")
        .arg(token)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn portfolio(
    addresses: &[String],
    tag: Option<&str>,
    aggregate: bool,
    chain: Option<&str>,
    source: Option<&str>,
    min_value: Option<f64>,
    show_sources: bool,
    show_blacklisted: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("portfolio");

    // Add addresses as positional arguments
    for addr in addresses {
        builder = builder.arg(addr);
    }

    builder = builder
        .opt("--tag", tag)
        .opt_flag("--aggregate", aggregate)
        .chain(chain)
        .opt("--source", source)
        .opt_flag("--show-sources", show_sources)
        .opt_flag("--show-blacklisted", show_blacklisted)
        .format_json();

    if let Some(mv) = min_value {
        builder = builder.opt("--min-value", Some(&mv.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn nfts(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("nfts")
        .arg(address)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn yields(protocol: Option<&str>, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("yields")
        .opt("--project", protocol)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn doctor() -> Result<String, ToolError> {
    ArgsBuilder::new("doctor")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY (6 subcommands)
// =============================================================================

pub async fn alchemy_nft(address: &str, network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("get-nfts")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_token(address: &str, network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("token")
        .network(network)
        .subcommand("balances")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_transfers(address: &str, network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("transfers")
        .network(network)
        .subcommand("from")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_portfolio(address: &str, network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("portfolio")
        .network(network)
        .subcommand("tokens")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_prices(tokens: &str, network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("prices")
        .network(network)
        .subcommand("by-address")
        .arg(tokens)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_debug(hash: &str, network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .network(network)
        .subcommand("trace-tx")
        .arg(hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_metadata(
    contract: &str,
    token_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("metadata")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_floor_price(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("floor-price")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_owners(
    contract: &str,
    token_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("owners")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_is_holder(
    address: &str,
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("is-holder")
        .arg(address)
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_token_metadata(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("token")
        .network(network)
        .subcommand("metadata")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_token_allowances(
    contract: &str,
    owner: &str,
    spender: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("token")
        .network(network)
        .subcommand("allowances")
        .arg(contract)
        .arg(owner)
        .arg(spender)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_transfers_to(
    address: &str,
    from_block: Option<&str>,
    to_block: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("transfers")
        .network(network)
        .subcommand("to")
        .arg(address)
        .opt("--from-block", from_block)
        .opt("--to-block", to_block)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_prices_by_address(
    addresses: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("prices")
        .network(network)
        .subcommand("by-address")
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY NFT (additional)
// =============================================================================

pub async fn alchemy_nft_owners_for_contract(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("owners-for-contract")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_contracts_for_owner(
    address: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("contracts-for-owner")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_nfts_for_contract(
    contract: &str,
    start_token: Option<&str>,
    limit: Option<u32>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("nfts-for-contract")
        .arg(contract);
    if let Some(st) = start_token {
        builder = builder.opt("--start-token", Some(st));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn alchemy_nft_contract_metadata(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("contract-metadata")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_collection_metadata(
    slug: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("collection-metadata")
        .arg(slug)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_search_contract_metadata(
    query: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("search-contract-metadata")
        .arg(query)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_compute_rarity(
    contract: &str,
    token_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("compute-rarity")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_summarize_attributes(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("summarize-attributes")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_refresh_metadata(
    contract: &str,
    token_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("refresh-metadata")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_sales(
    contract: &str,
    token_id: Option<&str>,
    from_block: Option<u64>,
    to_block: Option<u64>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("sales")
        .arg(contract);
    if let Some(tid) = token_id {
        builder = builder.opt("--token-id", Some(tid));
    }
    if let Some(fb) = from_block {
        builder = builder.opt("--from-block", Some(&fb.to_string()));
    }
    if let Some(tb) = to_block {
        builder = builder.opt("--to-block", Some(&tb.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn alchemy_nft_spam_contracts(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("spam-contracts")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_is_spam(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("is-spam")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_is_airdrop(
    contract: &str,
    token_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("is-airdrop")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_report_spam(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("report-spam")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_nfts_for_collection(
    slug: &str,
    start_token: Option<&str>,
    limit: Option<u32>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("nfts-for-collection")
        .arg(slug);
    if let Some(st) = start_token {
        builder = builder.opt("--start-token", Some(st));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn alchemy_nft_collections_for_owner(
    address: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("collections-for-owner")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_nft_invalidate_contract(
    contract: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .network(network)
        .subcommand("invalidate-contract")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY TOKEN (additional)
// =============================================================================

pub async fn alchemy_token_balances_for_tokens(
    address: &str,
    tokens: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("token")
        .network(network)
        .subcommand("balances-for-tokens")
        .arg(address)
        .arg(tokens)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY TRANSFERS (additional)
// =============================================================================

pub async fn alchemy_transfers_all(
    address: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("transfers")
        .network(network)
        .subcommand("all")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY PORTFOLIO (additional)
// =============================================================================

pub async fn alchemy_portfolio_token_info(
    tokens: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("portfolio")
        .network(network)
        .subcommand("token-info")
        .arg(tokens)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_portfolio_nfts(
    address: &str,
    with_metadata: bool,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("portfolio")
        .network(network)
        .subcommand("nfts")
        .arg(address)
        .opt_flag("--with-metadata", with_metadata)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_portfolio_nft_contracts(
    address: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("portfolio")
        .network(network)
        .subcommand("nft-contracts")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY PRICES (additional)
// =============================================================================

pub async fn alchemy_prices_by_symbol(
    symbols: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("prices")
        .network(network)
        .subcommand("by-symbol")
        .arg(symbols)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_prices_historical_by_symbol(
    symbol: &str,
    start_time: &str,
    end_time: &str,
    interval: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("prices")
        .network(network)
        .subcommand("historical-by-symbol")
        .arg(symbol)
        .arg(start_time)
        .arg(end_time)
        .opt("--interval", interval)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_prices_historical_by_address(
    address: &str,
    start_time: &str,
    end_time: &str,
    interval: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("prices")
        .network(network)
        .subcommand("historical-by-address")
        .arg(address)
        .arg(start_time)
        .arg(end_time)
        .opt("--interval", interval)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY DEBUG (additional)
// =============================================================================

pub async fn alchemy_debug_trace_call(
    to: &str,
    block: Option<&str>,
    from: Option<&str>,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .network(network)
        .subcommand("trace-call")
        .arg(to)
        .opt("--block", block)
        .opt("--from", from)
        .opt("--data", data)
        .opt("--value", value)
        .opt("--gas", gas)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_debug_trace_block_by_hash(
    hash: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .network(network)
        .subcommand("trace-block-by-hash")
        .arg(hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_debug_trace_block_by_number(
    block: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .network(network)
        .subcommand("trace-block-by-number")
        .arg(block)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_debug_get_raw_block(
    block: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .network(network)
        .subcommand("get-raw-block")
        .arg(block)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_debug_get_raw_header(
    block: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .network(network)
        .subcommand("get-raw-header")
        .arg(block)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_debug_get_raw_receipts(
    block: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .network(network)
        .subcommand("get-raw-receipts")
        .arg(block)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY TRACE (Parity-style)
// =============================================================================

pub async fn alchemy_trace_block(block: &str, network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("block")
        .arg(block)
        .execute()
        .await
        .map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn alchemy_trace_call(
    to: &str,
    block: Option<&str>,
    from: Option<&str>,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<&str>,
    trace_types: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("call")
        .arg(to)
        .opt("--block", block)
        .opt("--from", from)
        .opt("--data", data)
        .opt("--value", value)
        .opt("--gas", gas)
        .opt("--trace-types", trace_types)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_trace_get(
    hash: &str,
    indices: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("get")
        .arg(hash)
        .arg(indices)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_trace_raw_transaction(
    raw_tx: &str,
    trace_types: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("raw-transaction")
        .arg(raw_tx)
        .opt("--trace-types", trace_types)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_trace_replay_block_transactions(
    block: &str,
    trace_types: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("replay-block-transactions")
        .arg(block)
        .opt("--trace-types", trace_types)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_trace_replay_transaction(
    hash: &str,
    trace_types: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("replay-transaction")
        .arg(hash)
        .opt("--trace-types", trace_types)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_trace_transaction(
    hash: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("transaction")
        .arg(hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_trace_filter(
    from_block: Option<&str>,
    to_block: Option<&str>,
    from_address: Option<&str>,
    to_address: Option<&str>,
    after: Option<u32>,
    count: Option<u32>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("alchemy")
        .subcommand("trace")
        .network(network)
        .subcommand("filter")
        .opt("--from-block", from_block)
        .opt("--to-block", to_block)
        .opt("--from-address", from_address)
        .opt("--to-address", to_address);
    if let Some(a) = after {
        builder = builder.opt("--after", Some(&a.to_string()));
    }
    if let Some(c) = count {
        builder = builder.opt("--count", Some(&c.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY SIMULATION
// =============================================================================

pub async fn alchemy_sim_asset_changes(
    to: &str,
    from: Option<&str>,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("simulation")
        .network(network)
        .subcommand("asset-changes")
        .arg(to)
        .opt("--from", from)
        .opt("--data", data)
        .opt("--value", value)
        .opt("--gas", gas)
        .execute()
        .await
        .map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn alchemy_sim_execution(
    to: &str,
    from: Option<&str>,
    data: Option<&str>,
    value: Option<&str>,
    gas: Option<&str>,
    block: Option<&str>,
    trace_format: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("simulation")
        .network(network)
        .subcommand("execution")
        .arg(to)
        .opt("--from", from)
        .opt("--data", data)
        .opt("--value", value)
        .opt("--gas", gas)
        .opt("--block", block)
        .opt("--trace-format", trace_format)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY BUNDLER (ERC-4337)
// =============================================================================

pub async fn alchemy_bundler_supported_entry_points(
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("bundler")
        .network(network)
        .subcommand("supported-entry-points")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_bundler_estimate_gas(
    user_op_json: &str,
    entry_point: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("bundler")
        .network(network)
        .subcommand("estimate-gas")
        .arg(user_op_json)
        .opt("--entry-point", entry_point)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_bundler_get_by_hash(
    hash: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("bundler")
        .network(network)
        .subcommand("get-by-hash")
        .arg(hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_bundler_get_receipt(
    hash: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("bundler")
        .network(network)
        .subcommand("get-receipt")
        .arg(hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_bundler_max_priority_fee(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("bundler")
        .network(network)
        .subcommand("max-priority-fee")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY GAS MANAGER
// =============================================================================

pub async fn alchemy_gas_manager_list_policies(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("gas-manager")
        .network(network)
        .subcommand("list-policies")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_gas_manager_get_policy(
    policy_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("gas-manager")
        .network(network)
        .subcommand("get-policy")
        .arg(policy_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_gas_manager_policy_stats(
    policy_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("gas-manager")
        .network(network)
        .subcommand("policy-stats")
        .arg(policy_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_gas_manager_list_sponsorships(
    policy_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("gas-manager")
        .network(network)
        .subcommand("list-sponsorships")
        .arg(policy_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY NOTIFY
// =============================================================================

pub async fn alchemy_notify_list_webhooks(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("notify")
        .network(network)
        .subcommand("list-webhooks")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_notify_list_addresses(
    webhook_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("notify")
        .network(network)
        .subcommand("list-addresses")
        .arg(webhook_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_notify_list_nft_filters(
    webhook_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("notify")
        .network(network)
        .subcommand("list-nft-filters")
        .arg(webhook_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY BEACON
// =============================================================================

pub async fn alchemy_beacon_genesis(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("genesis")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_fork_schedule(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("fork-schedule")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_deposit_contract(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("deposit-contract")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_spec(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("spec")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_headers(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("headers")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_header(
    block_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("header")
        .arg(block_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_block(
    block_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("block")
        .arg(block_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_block_root(
    block_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("block-root")
        .arg(block_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_block_attestations(
    block_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("block-attestations")
        .arg(block_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_blob_sidecars(
    block_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("blob-sidecars")
        .arg(block_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_state_root(
    state_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("state-root")
        .arg(state_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_state_fork(
    state_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("state-fork")
        .arg(state_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_finality_checkpoints(
    state_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("finality-checkpoints")
        .arg(state_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_validators(
    state_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("validators")
        .arg(state_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_validator(
    state_id: &str,
    validator_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("validator")
        .arg(state_id)
        .arg(validator_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_validator_balances(
    state_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("validator-balances")
        .arg(state_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_sync_committees(
    state_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("sync-committees")
        .arg(state_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_randao(
    state_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("randao")
        .arg(state_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_pool_attestations(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("pool-attestations")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_voluntary_exits(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("voluntary-exits")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_block_rewards(
    block_id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("block-rewards")
        .arg(block_id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_syncing(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("syncing")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_version(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("version")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_peers(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("peers")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_peer_count(network: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("peer-count")
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_attester_duties(
    epoch: &str,
    validators: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("attester-duties")
        .arg(epoch)
        .arg(validators)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_proposer_duties(
    epoch: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("proposer-duties")
        .arg(epoch)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_beacon_sync_duties(
    epoch: &str,
    validators: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("beacon")
        .network(network)
        .subcommand("sync-duties")
        .arg(epoch)
        .arg(validators)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ALCHEMY SOLANA
// =============================================================================

pub async fn alchemy_solana_get_asset(
    id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-asset")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_solana_get_assets(
    ids: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-assets")
        .arg(ids)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_solana_get_asset_proof(
    id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-asset-proof")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_solana_get_asset_proofs(
    ids: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-asset-proofs")
        .arg(ids)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_solana_get_assets_by_owner(
    owner: &str,
    page: Option<u32>,
    limit: Option<u32>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-assets-by-owner")
        .arg(owner);
    if let Some(p) = page {
        builder = builder.opt("--page", Some(&p.to_string()));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn alchemy_solana_get_assets_by_creator(
    creator: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-assets-by-creator")
        .arg(creator)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_solana_get_assets_by_authority(
    authority: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-assets-by-authority")
        .arg(authority)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_solana_get_assets_by_group(
    group_key: &str,
    group_value: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-assets-by-group")
        .arg(group_key)
        .arg(group_value)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_solana_get_token_accounts(
    owner: Option<&str>,
    mint: Option<&str>,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-token-accounts")
        .opt("--owner", owner)
        .opt("--mint", mint)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_solana_get_nft_editions(
    mint: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-nft-editions")
        .arg(mint)
        .execute()
        .await
        .map_err(ToolError::from)
}
pub async fn alchemy_solana_get_asset_signatures(
    id: &str,
    network: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("solana")
        .network(network)
        .subcommand("get-asset-signatures")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// GECKO (5 subcommands) - CoinGecko
// =============================================================================

pub async fn gecko_simple_price(ids: &str, vs: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("simple")
        .subcommand("price")
        .arg(ids)
        .opt("--vs", vs)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_info(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("get")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_global() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("global")
        .subcommand("data")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_nfts(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("nfts")
        .subcommand("get")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_onchain(network: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("token")
        .arg(network)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Gecko Simple ---

pub async fn gecko_simple_token_price(
    platform: &str,
    addresses: &str,
    vs: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("simple")
        .subcommand("token-price")
        .arg(platform)
        .arg(addresses)
        .opt("--vs", vs)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_simple_currencies() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("simple")
        .subcommand("currencies")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Gecko Coins ---

pub async fn gecko_coins_list(with_platforms: bool) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("list")
        .opt_flag("--with-platforms", with_platforms)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_markets(vs_currency: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("markets");
    if let Some(vs) = vs_currency {
        builder = builder.arg(vs);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn gecko_coins_tickers(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("tickers")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_chart(
    id: &str,
    vs: Option<&str>,
    days: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("chart")
        .arg(id)
        .opt("--vs", vs)
        .opt("--days", days)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_ohlc(
    id: &str,
    vs: Option<&str>,
    days: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("ohlc")
        .arg(id)
        .opt("--vs", vs)
        .opt("--days", days)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_history(id: &str, date: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("history")
        .arg(id)
        .arg(date)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_top_movers(
    vs: Option<&str>,
    duration: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("top-movers")
        .opt("--vs", vs)
        .opt("--duration", duration)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_new() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("new")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_coins_by_contract(platform: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("coins")
        .subcommand("by-contract")
        .arg(platform)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Gecko Global ---

pub async fn gecko_global_ping() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("global")
        .subcommand("ping")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_global_defi() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("global")
        .subcommand("defi")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_global_trending() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("global")
        .subcommand("trending")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_global_search(query: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("global")
        .subcommand("search")
        .arg(query)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_global_exchange_rates() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("global")
        .subcommand("exchange-rates")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_global_platforms() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("global")
        .subcommand("platforms")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Gecko NFTs ---

pub async fn gecko_nfts_list() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("nfts")
        .subcommand("list")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_nfts_by_contract(platform: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("nfts")
        .subcommand("by-contract")
        .arg(platform)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_nfts_markets() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("nfts")
        .subcommand("markets")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_nfts_tickers(id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("nfts")
        .subcommand("tickers")
        .arg(id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Gecko Onchain ---

pub async fn gecko_onchain_networks() -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("networks")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_onchain_dexes(network: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("dexes")
        .arg(network)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_onchain_trending_pools(network: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("trending-pools");
    if let Some(net) = network {
        builder = builder.arg(net);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn gecko_onchain_top_pools(network: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("top-pools");
    if let Some(net) = network {
        builder = builder.arg(net);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn gecko_onchain_new_pools(network: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("new-pools");
    if let Some(net) = network {
        builder = builder.arg(net);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn gecko_onchain_token_info(network: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("token")
        .arg(network)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_onchain_token_price(
    network: &str,
    addresses: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("token-price")
        .arg(network)
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_onchain_token_pools(network: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("token-pools")
        .arg(network)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_onchain_pool_ohlcv(
    network: &str,
    address: &str,
    timeframe: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("pool-ohlcv")
        .arg(network)
        .arg(address)
        .opt("--timeframe", timeframe)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gecko_onchain_search_pools(query: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("gecko")
        .subcommand("onchain")
        .subcommand("search-pools")
        .arg(query)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// GOPLUS (6 subcommands) - Security
// =============================================================================

pub async fn goplus_token(address: &str, chain_id: u64) -> Result<String, ToolError> {
    ArgsBuilder::new("goplus")
        .subcommand("token")
        .arg(address)
        .opt("--chain-id", Some(&chain_id.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn goplus_token_batch(addresses: &str, chain_id: u64) -> Result<String, ToolError> {
    ArgsBuilder::new("goplus")
        .subcommand("token-batch")
        .arg(addresses)
        .opt("--chain-id", Some(&chain_id.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn goplus_address(address: &str, chain_id: u64) -> Result<String, ToolError> {
    ArgsBuilder::new("goplus")
        .subcommand("address")
        .arg(address)
        .opt("--chain-id", Some(&chain_id.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn goplus_nft(address: &str, chain_id: u64) -> Result<String, ToolError> {
    ArgsBuilder::new("goplus")
        .subcommand("nft")
        .arg(address)
        .opt("--chain-id", Some(&chain_id.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn goplus_approval(address: &str, chain_id: u64) -> Result<String, ToolError> {
    ArgsBuilder::new("goplus")
        .subcommand("approval")
        .arg(address)
        .opt("--chain-id", Some(&chain_id.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn goplus_chains() -> Result<String, ToolError> {
    ArgsBuilder::new("goplus")
        .subcommand("chains")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// SOLODIT (5 subcommands) - Security DB
// =============================================================================

pub async fn solodit_search(
    query: &str,
    impact: Option<&str>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("solodit").subcommand("search").arg(query);

    if let Some(i) = impact {
        builder = builder.opt("--impact", Some(i));
    }
    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn solodit_get(slug: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("solodit")
        .subcommand("get")
        .arg(slug)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn solodit_rate_limit() -> Result<String, ToolError> {
    ArgsBuilder::new("solodit")
        .subcommand("rate-limit")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn solodit_tags() -> Result<String, ToolError> {
    ArgsBuilder::new("solodit")
        .subcommand("tags")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn solodit_firms() -> Result<String, ToolError> {
    ArgsBuilder::new("solodit")
        .subcommand("firms")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// LLAMA (6 subcommands) - DefiLlama
// =============================================================================

pub async fn llama_tvl(protocol: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("tvl")
        .subcommand("protocol")
        .arg(protocol)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_coins(addresses: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("coins")
        .subcommand("current")
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_yields() -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("yields")
        .subcommand("pools")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_volumes(protocol: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("volumes")
        .subcommand("dex-protocol")
        .arg(protocol)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_fees(protocol: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("fees")
        .subcommand("protocol")
        .arg(protocol)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_stablecoins() -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("stablecoins")
        .subcommand("list")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// MORALIS (12 subcommands)
// =============================================================================

// --- Moralis Token ---

pub async fn moralis_token_metadata(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("metadata")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_price(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("price")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_holders(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("holders")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_pairs(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("pairs")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_transfers(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("transfers")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Wallet ---

pub async fn moralis_wallet_balance(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("balance")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_tokens(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("tokens")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_history(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("history")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_net_worth(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("net-worth")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Resolve ---

pub async fn moralis_resolve_domain(
    domain: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("resolve")
        .opt("--chain", chain)
        .subcommand("domain")
        .arg(domain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_resolve_address(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("resolve")
        .opt("--chain", chain)
        .subcommand("address")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Market ---

pub async fn moralis_market_top_tokens(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("market")
        .opt("--chain", chain)
        .subcommand("top-tokens")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_market_top_movers(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("market")
        .opt("--chain", chain)
        .subcommand("top-movers")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_market_top_nfts(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("market")
        .opt("--chain", chain)
        .subcommand("top-nfts")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Wallet (additional) ---

pub async fn moralis_wallet_transactions(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("transactions")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_active_chains(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("active-chains")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_approvals(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("approvals")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_stats(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("stats")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_profitability(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("profitability")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Token (additional) ---

pub async fn moralis_token_swaps(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("swaps")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_stats(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("stats")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_search(query: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("search")
        .arg(query)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_trending(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("trending")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_pair_ohlcv(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("pair-ohlcv")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_pair_stats(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("pair-stats")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis NFT ---

pub async fn moralis_nft_list(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("list")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_metadata(
    contract: &str,
    token_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("metadata")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_transfers(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("transfers")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_collection(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("collection")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_collection_stats(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("collection-stats")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_owners(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("owners")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_trades(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("trades")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_floor_price(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("floor-price")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Wallet (new) ---

pub async fn moralis_wallet_profitability_tokens(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("profitability-tokens")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_wallet_multiple_balances(
    addresses: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .opt("--chain", chain)
        .subcommand("multiple-balances")
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Token (new) ---

pub async fn moralis_token_wallet_swaps(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("wallet-swaps")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_pair_swaps(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("pair-swaps")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_categories(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("categories")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_exchange_new_tokens(
    exchange: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("exchange-new-tokens")
        .arg(exchange)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_exchange_bonding_tokens(
    exchange: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("exchange-bonding-tokens")
        .arg(exchange)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_exchange_graduated_tokens(
    exchange: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("exchange-graduated-tokens")
        .arg(exchange)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_multiple_prices(
    addresses: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("multiple-prices")
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_by_symbols(
    symbols: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("by-symbols")
        .arg(symbols)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_contract_transfers(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("contract-transfers")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_holders_summary(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("holders-summary")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_holders_historical(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("holders-historical")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_pairs_stats(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("pairs-stats")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_top_gainers(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("top-gainers")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_pair_snipers(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("pair-snipers")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token_bonding_status(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .opt("--chain", chain)
        .subcommand("bonding-status")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis NFT (new) ---

pub async fn moralis_nft_wallet_collections(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("wallet-collections")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_contract_transfers(
    contract: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("contract-transfers")
        .arg(contract)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_token_transfers(
    contract: &str,
    token_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("token-transfers")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_token_owners(
    contract: &str,
    token_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("token-owners")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_token_floor_price(
    contract: &str,
    token_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("token-floor-price")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_token_trades(
    contract: &str,
    token_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("token-trades")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_wallet_trades(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("wallet-trades")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_collection_traits(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("collection-traits")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_collection_traits_paginated(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("collection-traits-paginated")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_unique_owners(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("unique-owners")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_resync_metadata(
    contract: &str,
    token_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("resync-metadata")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_multiple_nfts(
    tokens: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("multiple-nfts")
        .arg(tokens)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_floor_price_historical(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("floor-price-historical")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_sync_collection(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("sync-collection")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_contract_nfts(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("contract-nfts")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_multiple_collections(
    addresses: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("multiple-collections")
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_resync_traits(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("resync-traits")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_collection_prices(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("collection-prices")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft_token_prices(
    contract: &str,
    token_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .opt("--chain", chain)
        .subcommand("token-prices")
        .arg(contract)
        .arg(token_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Resolve (new) ---

pub async fn moralis_resolve_address_domains(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("resolve")
        .opt("--chain", chain)
        .subcommand("address-domains")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_resolve_ens_domain(
    domain: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("resolve")
        .opt("--chain", chain)
        .subcommand("ens-domain")
        .arg(domain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Market (new) ---

pub async fn moralis_market_hottest_nfts(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("market")
        .opt("--chain", chain)
        .subcommand("hottest-nfts")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_market_global_market_cap(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("market")
        .opt("--chain", chain)
        .subcommand("global-market-cap")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_market_global_volume(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("market")
        .opt("--chain", chain)
        .subcommand("global-volume")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Transaction ---

pub async fn moralis_transaction_get(
    tx_hash: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("transaction")
        .opt("--chain", chain)
        .subcommand("get")
        .arg(tx_hash)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_transaction_verbose(
    tx_hash: &str,
    include_internal: bool,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("transaction")
        .opt("--chain", chain)
        .subcommand("verbose")
        .arg(tx_hash)
        .opt_flag("--include-internal", include_internal)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_transaction_wallet(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("transaction")
        .opt("--chain", chain)
        .subcommand("wallet")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_transaction_wallet_verbose(
    address: &str,
    include_internal: bool,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("transaction")
        .opt("--chain", chain)
        .subcommand("wallet-verbose")
        .arg(address)
        .opt_flag("--include-internal", include_internal)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Block ---

pub async fn moralis_block_get(
    block_number_or_hash: &str,
    include_transactions: bool,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("block")
        .opt("--chain", chain)
        .subcommand("get")
        .arg(block_number_or_hash)
        .opt_flag("--include-transactions", include_transactions)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_block_latest(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("block")
        .opt("--chain", chain)
        .subcommand("latest")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_block_date_to_block(
    date: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("block")
        .opt("--chain", chain)
        .subcommand("date-to-block")
        .arg(date)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis DeFi ---

pub async fn moralis_defi_pair_price(
    token0: &str,
    token1: &str,
    exchange: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("defi")
        .opt("--chain", chain)
        .subcommand("pair-price")
        .arg(token0)
        .arg(token1)
        .opt("--exchange", exchange)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_defi_pair_reserves(
    pair_address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("defi")
        .opt("--chain", chain)
        .subcommand("pair-reserves")
        .arg(pair_address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_defi_pair_address(
    token0: &str,
    token1: &str,
    exchange: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("defi")
        .opt("--chain", chain)
        .subcommand("pair-address")
        .arg(token0)
        .arg(token1)
        .opt("--exchange", exchange)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_defi_wallet_summary(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("defi")
        .opt("--chain", chain)
        .subcommand("wallet-summary")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_defi_wallet_positions(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("defi")
        .opt("--chain", chain)
        .subcommand("wallet-positions")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_defi_protocol_positions(
    address: &str,
    protocol: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("defi")
        .opt("--chain", chain)
        .subcommand("protocol-positions")
        .arg(address)
        .arg(protocol)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Discovery ---

pub async fn moralis_discovery_rising_liquidity(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("rising-liquidity")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_buying_pressure(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("buying-pressure")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_solid_performers(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("solid-performers")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_experienced_buyers(
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("experienced-buyers")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_risky_bets(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("risky-bets")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_blue_chip(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("blue-chip")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_top_gainers(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("top-gainers")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_top_losers(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("top-losers")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_trending(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("trending")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_token_analytics(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("token-analytics")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_discovery_token_score(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("token-score")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn moralis_discovery_filter(
    min_market_cap: Option<f64>,
    max_market_cap: Option<f64>,
    min_liquidity: Option<f64>,
    max_liquidity: Option<f64>,
    min_volume_24h: Option<f64>,
    max_volume_24h: Option<f64>,
    min_holders: Option<i64>,
    min_security_score: Option<i32>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("filter");

    if let Some(v) = min_market_cap {
        builder = builder.arg("--min-market-cap").arg(&v.to_string());
    }
    if let Some(v) = max_market_cap {
        builder = builder.arg("--max-market-cap").arg(&v.to_string());
    }
    if let Some(v) = min_liquidity {
        builder = builder.arg("--min-liquidity").arg(&v.to_string());
    }
    if let Some(v) = max_liquidity {
        builder = builder.arg("--max-liquidity").arg(&v.to_string());
    }
    if let Some(v) = min_volume_24h {
        builder = builder.arg("--min-volume-24h").arg(&v.to_string());
    }
    if let Some(v) = max_volume_24h {
        builder = builder.arg("--max-volume-24h").arg(&v.to_string());
    }
    if let Some(v) = min_holders {
        builder = builder.arg("--min-holders").arg(&v.to_string());
    }
    if let Some(v) = min_security_score {
        builder = builder.arg("--min-security-score").arg(&v.to_string());
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn moralis_discovery_token(
    address: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("discovery")
        .opt("--chain", chain)
        .subcommand("token")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Analytics ---

pub async fn moralis_analytics_timeseries(
    addresses: &str,
    timeframe: Option<&str>,
    from_date: Option<&str>,
    to_date: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("analytics")
        .opt("--chain", chain)
        .subcommand("timeseries")
        .arg(addresses)
        .opt("--timeframe", timeframe)
        .opt("--from-date", from_date)
        .opt("--to-date", to_date)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_analytics_batch(
    addresses: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("analytics")
        .opt("--chain", chain)
        .subcommand("batch")
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Entities ---

pub async fn moralis_entities_search(
    query: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("entities")
        .opt("--chain", chain)
        .subcommand("search")
        .arg(query)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_entities_get(
    entity_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("entities")
        .opt("--chain", chain)
        .subcommand("get")
        .arg(entity_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_entities_categories(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("entities")
        .opt("--chain", chain)
        .subcommand("categories")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_entities_category_entities(
    category_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("entities")
        .opt("--chain", chain)
        .subcommand("category-entities")
        .arg(category_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Moralis Volume ---

pub async fn moralis_volume_chains(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("volume")
        .opt("--chain", chain)
        .subcommand("chains")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_volume_categories(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("volume")
        .opt("--chain", chain)
        .subcommand("categories")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_volume_timeseries(
    timeframe: Option<&str>,
    from_date: Option<&str>,
    to_date: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("volume")
        .opt("--chain", chain)
        .subcommand("timeseries")
        .opt("--timeframe", timeframe)
        .opt("--from-date", from_date)
        .opt("--to-date", to_date)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_volume_category_timeseries(
    category_id: &str,
    timeframe: Option<&str>,
    from_date: Option<&str>,
    to_date: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("volume")
        .opt("--chain", chain)
        .subcommand("category-timeseries")
        .arg(category_id)
        .opt("--timeframe", timeframe)
        .opt("--from-date", from_date)
        .opt("--to-date", to_date)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// DSIM (7 subcommands) - Dune Sim
// =============================================================================

pub async fn dsim_chains() -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("chains")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_balances(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("balances")
        .subcommand("get")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_collectibles(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("collectibles")
        .subcommand("get")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_activity(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("activity")
        .subcommand("get")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_token(address: &str, chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("token")
        .subcommand("info")
        .arg(address)
        .opt("--chain-id", chain_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_holders(token: &str, chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("holders")
        .subcommand("get")
        .arg(token)
        .opt("--chain-id", chain_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_defi(address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("defi")
        .subcommand("positions")
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// DUNE (3 subcommands)
// =============================================================================

pub async fn dune_query(query_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("query")
        .subcommand("execute")
        .arg(query_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_sql(sql: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("sql")
        .subcommand("execute")
        .arg(sql)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_execution(execution_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("execution")
        .subcommand("results")
        .arg(execution_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Dune Queries CRUD ---

pub async fn dune_queries_create(
    name: &str,
    sql: &str,
    description: Option<&str>,
    private: bool,
    tags: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("create")
        .opt("--name", Some(name))
        .opt("--sql", Some(sql))
        .opt("--description", description)
        .opt_flag("--private", private)
        .opt("--tags", tags)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_queries_get(query_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("get")
        .arg(query_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_queries_update(
    query_id: &str,
    name: Option<&str>,
    sql: Option<&str>,
    description: Option<&str>,
    tags: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("update")
        .arg(query_id)
        .opt("--name", name)
        .opt("--sql", sql)
        .opt("--description", description)
        .opt("--tags", tags)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_queries_list(
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("list");

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }
    if let Some(o) = offset {
        builder = builder.opt("--offset", Some(&o.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn dune_queries_archive(query_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("archive")
        .arg(query_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_queries_unarchive(query_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("unarchive")
        .arg(query_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_queries_make_private(query_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("make-private")
        .arg(query_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_queries_make_public(query_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("queries")
        .subcommand("make-public")
        .arg(query_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Dune Tables CRUD ---

pub async fn dune_tables_create(
    namespace: &str,
    table_name: &str,
    schema_json: &str,
    description: Option<&str>,
    private: bool,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("tables")
        .subcommand("create")
        .arg(namespace)
        .arg(table_name)
        .opt("--schema", Some(schema_json))
        .opt("--description", description)
        .opt_flag("--private", private)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_tables_upload_csv(
    table_name: &str,
    data: &str,
    description: Option<&str>,
    private: bool,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("tables")
        .subcommand("upload-csv")
        .arg(table_name)
        .arg(data)
        .opt("--description", description)
        .opt_flag("--private", private)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_tables_list(
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("dune")
        .subcommand("tables")
        .subcommand("list");

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }
    if let Some(o) = offset {
        builder = builder.opt("--offset", Some(&o.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn dune_tables_get(namespace: &str, table_name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("tables")
        .subcommand("get")
        .arg(namespace)
        .arg(table_name)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_tables_insert(
    namespace: &str,
    table_name: &str,
    data_json: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("tables")
        .subcommand("insert")
        .arg(namespace)
        .arg(table_name)
        .arg(data_json)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_tables_clear(namespace: &str, table_name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("tables")
        .subcommand("clear")
        .arg(namespace)
        .arg(table_name)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_tables_delete(namespace: &str, table_name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("tables")
        .subcommand("delete")
        .arg(namespace)
        .arg(table_name)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Dune Materialized Views ---

pub async fn dune_matviews_upsert(
    name: &str,
    query_id: &str,
    cron: Option<&str>,
    expires_at: Option<&str>,
    private: bool,
    performance: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("matviews")
        .subcommand("upsert")
        .arg(name)
        .opt("--query-id", Some(query_id))
        .opt("--cron", cron)
        .opt("--expires-at", expires_at)
        .opt_flag("--private", private)
        .opt("--performance", performance)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_matviews_get(name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("matviews")
        .subcommand("get")
        .arg(name)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_matviews_list(
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("dune")
        .subcommand("matviews")
        .subcommand("list");

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }
    if let Some(o) = offset {
        builder = builder.opt("--offset", Some(&o.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn dune_matviews_refresh(
    name: &str,
    performance: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("matviews")
        .subcommand("refresh")
        .arg(name)
        .opt("--performance", performance)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_matviews_delete(name: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("matviews")
        .subcommand("delete")
        .arg(name)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Dune Pipelines ---

pub async fn dune_pipelines_execute(
    pipeline_json: &str,
    params: Option<&str>,
    performance: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("pipelines")
        .subcommand("execute")
        .arg(pipeline_json)
        .opt("--params", params)
        .opt("--performance", performance)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_pipelines_status(execution_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("pipelines")
        .subcommand("status")
        .arg(execution_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Dune Usage ---

pub async fn dune_usage() -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("usage")
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// CURVE (10 subcommands)
// =============================================================================

pub async fn curve_router_route(
    from_token: &str,
    to_token: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("router")
        .subcommand("route")
        .arg(from_token)
        .arg(to_token)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_pools(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("list")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_volumes(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("volumes")
        .subcommand("total")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_lending(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("lending")
        .subcommand("all")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_tokens(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("tokens")
        .subcommand("list")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_crvusd() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("crvusd")
        .subcommand("total-supply")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_prices(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("prices")
        .subcommand("all")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_ohlc(chain: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("ohlc")
        .subcommand("pool")
        .arg(chain)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_trades(chain: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("trades")
        .subcommand("get")
        .arg(chain)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_dao() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("dao")
        .subcommand("gauges")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Router additional ---

pub async fn curve_router_encode(
    from: &str,
    to: &str,
    amount: &str,
    min_out: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("router")
        .subcommand("encode")
        .arg(from)
        .arg(to)
        .arg(amount)
        .arg(min_out)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_router_stats(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("router")
        .subcommand("stats")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_router_address(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("router")
        .subcommand("address")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Pools additional ---

pub async fn curve_pools_registry(chain: &str, registry: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("registry")
        .arg(chain)
        .arg(registry)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_pools_all() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("all")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_pools_big(chain: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("big");
    if let Some(c) = chain {
        builder = builder.arg(c);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn curve_pools_small(chain: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("small");
    if let Some(c) = chain {
        builder = builder.arg(c);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn curve_pools_empty(chain: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("empty");
    if let Some(c) = chain {
        builder = builder.arg(c);
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn curve_pools_addresses(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("addresses")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_pools_hidden() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("pools")
        .subcommand("hidden")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Volumes additional ---

pub async fn curve_volumes_gauges() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("volumes")
        .subcommand("gauges")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_volumes_total(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("volumes")
        .subcommand("total")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_volumes_apys(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("volumes")
        .subcommand("apys")
        .arg(chain.unwrap_or("ethereum"))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_volumes_crvusd() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("volumes")
        .subcommand("crvusd")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Lending additional ---

pub async fn curve_lending_all() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("lending")
        .subcommand("all")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_lending_registry(chain: &str, registry: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("lending")
        .subcommand("registry")
        .arg(chain)
        .arg(registry)
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- CrvUSD additional ---

pub async fn curve_crvusd_circulating_supply() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("crvusd")
        .subcommand("circulating-supply")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_crvusd_scrvusd_supply() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("crvusd")
        .subcommand("scrvusd-supply")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_crvusd_markets(chain: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("curve")
        .subcommand("crvusd")
        .subcommand("markets");
    if let Some(c) = chain {
        builder = builder.opt("--chain", Some(c));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn curve_crvusd_savings() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("crvusd")
        .subcommand("savings")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- Prices additional ---

pub async fn curve_prices_chains() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("prices")
        .subcommand("chains")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_prices_chain_stats() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("prices")
        .subcommand("chain-stats")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_prices_token(chain: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("prices")
        .subcommand("token")
        .arg(chain)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_prices_history(
    chain: &str,
    address: &str,
    start: Option<u64>,
    end: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("curve")
        .subcommand("prices")
        .subcommand("history")
        .arg(chain)
        .arg(address);
    if let Some(s) = start {
        builder = builder.opt("--start", Some(&s.to_string()));
    }
    if let Some(e) = end {
        builder = builder.opt("--end", Some(&e.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

pub async fn curve_prices_top_volume() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("prices")
        .subcommand("top-volume")
        .execute()
        .await
        .map_err(ToolError::from)
}

// --- OHLC additional ---

pub async fn curve_ohlc_lp_token(
    chain: &str,
    address: &str,
    start: Option<u64>,
    end: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("curve")
        .subcommand("ohlc")
        .subcommand("lp-token")
        .arg(chain)
        .arg(address);
    if let Some(s) = start {
        builder = builder.opt("--start", Some(&s.to_string()));
    }
    if let Some(e) = end {
        builder = builder.opt("--end", Some(&e.to_string()));
    }
    builder.execute().await.map_err(ToolError::from)
}

// --- DAO additional ---

pub async fn curve_dao_proposals() -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("dao")
        .subcommand("proposals")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_dao_lockers(top: Option<u32>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("curve")
        .subcommand("dao")
        .subcommand("lockers");
    if let Some(t) = top {
        builder = builder.arg(&t.to_string());
    }
    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// QUOTE (3 subcommands)
// =============================================================================

#[allow(clippy::too_many_arguments)]
pub async fn quote_best(
    from_token: &str,
    to_token: &str,
    amount: &str,
    chain: Option<&str>,
    decimals: Option<u8>,
    sender: Option<&str>,
    slippage: Option<u32>,
    show_tx: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("quote")
        .subcommand("best")
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .chain(chain)
        .opt("--sender", sender)
        .opt_flag("--show-tx", show_tx);

    if let Some(d) = decimals {
        builder = builder.opt("--decimals", Some(&d.to_string()));
    }
    if let Some(s) = slippage {
        builder = builder.opt("--slippage", Some(&s.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn quote_from(
    source: &str,
    from_token: &str,
    to_token: &str,
    amount: &str,
    chain: Option<&str>,
    decimals: Option<u8>,
    sender: Option<&str>,
    slippage: Option<u32>,
    show_tx: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("quote")
        .subcommand("from")
        .arg(source)
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .chain(chain)
        .opt("--sender", sender)
        .opt_flag("--show-tx", show_tx);

    if let Some(d) = decimals {
        builder = builder.opt("--decimals", Some(&d.to_string()));
    }
    if let Some(s) = slippage {
        builder = builder.opt("--slippage", Some(&s.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn quote_compare(
    from_token: &str,
    to_token: &str,
    amount: &str,
    chain: Option<&str>,
    decimals: Option<u8>,
    sender: Option<&str>,
    slippage: Option<u32>,
    show_tx: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("quote")
        .subcommand("compare")
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .chain(chain)
        .opt("--sender", sender)
        .opt_flag("--show-tx", show_tx);

    if let Some(d) = decimals {
        builder = builder.opt("--decimals", Some(&d.to_string()));
    }
    if let Some(s) = slippage {
        builder = builder.opt("--slippage", Some(&s.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// CHAINLINK (4 subcommands)
// =============================================================================

pub async fn chainlink_price(
    token: &str,
    chain: Option<&str>,
    block: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("price")
        .arg(token)
        .chain(chain)
        .opt("--block", block)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlink_feed(token: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("feed")
        .arg(token)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlink_oracles(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("oracles")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlink_streams_feeds() -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("streams")
        .subcommand("feeds")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlink_streams_latest(feed_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("streams")
        .subcommand("latest")
        .arg(feed_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlink_streams_report(feed_id: &str, timestamp: u64) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("streams")
        .subcommand("report")
        .arg(feed_id)
        .arg(&timestamp.to_string())
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlink_streams_bulk(feed_ids: &str, timestamp: u64) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("streams")
        .subcommand("bulk")
        .arg(feed_ids)
        .arg(&timestamp.to_string())
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlink_streams_history(
    feed_id: &str,
    timestamp: u64,
    limit: u32,
) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("streams")
        .subcommand("history")
        .arg(feed_id)
        .arg(&timestamp.to_string())
        .opt("--limit", Some(&limit.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// CCXT (7 subcommands) - Exchange data
// =============================================================================

pub async fn ccxt_ticker(exchange: &str, symbol: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ccxt")
        .subcommand("ticker")
        .arg(symbol)
        .opt("--exchange", Some(exchange))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn ccxt_tickers(exchange: &str, symbols: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("ccxt")
        .subcommand("tickers")
        .opt("--exchange", Some(exchange));

    if let Some(s) = symbols {
        builder = builder.arg(s);
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn ccxt_orderbook(
    exchange: &str,
    symbol: &str,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("ccxt")
        .subcommand("order-book")
        .arg(symbol)
        .opt("--exchange", Some(exchange));

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn ccxt_ohlcv(
    exchange: &str,
    symbol: &str,
    timeframe: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("ccxt")
        .subcommand("ohlcv")
        .arg(symbol)
        .opt("--exchange", Some(exchange))
        .opt("--timeframe", timeframe)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn ccxt_trades(exchange: &str, symbol: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ccxt")
        .subcommand("trades")
        .arg(symbol)
        .opt("--exchange", Some(exchange))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn ccxt_markets(exchange: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ccxt")
        .subcommand("markets")
        .opt("--exchange", Some(exchange))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn ccxt_compare(symbol: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("ccxt")
        .subcommand("compare")
        .arg(symbol)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// UNISWAP (9 subcommands)
// =============================================================================

pub async fn uniswap_pool(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("pool")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_liquidity(pool: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("liquidity")
        .arg(pool)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_eth_price(
    chain: Option<&str>,
    version: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("eth-price")
        .chain(chain)
        .opt("--version", version)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_top_pools(
    limit: Option<u32>,
    chain: Option<&str>,
    version: Option<&str>,
) -> Result<String, ToolError> {
    let l = limit.unwrap_or(10);
    ArgsBuilder::new("uniswap")
        .subcommand("top-pools")
        .arg(&l.to_string())
        .chain(chain)
        .opt("--version", version)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_swaps(pool: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("swaps")
        .arg(pool)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_day_data(pool: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("day-data")
        .arg(pool)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_positions(
    address: &str,
    chain: Option<&str>,
    version: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("positions")
        .arg(address)
        .chain(chain)
        .opt("--version", version)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_balance(
    token: &str,
    account: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("balance")
        .arg(token)
        .arg(account)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn uniswap_addresses(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("addresses")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// KONG (5 subcommands) - Yearn/Kong
// =============================================================================

pub async fn kong_vaults(chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("vaults")
        .subcommand("list")
        .opt("--chain-id", chain_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_strategies(chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("strategies")
        .subcommand("list")
        .opt("--chain-id", chain_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_prices(address: &str, chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("prices")
        .subcommand("current")
        .opt("--chain-id", chain_id)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_tvl(address: &str, chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("tvl")
        .subcommand("current")
        .opt("--chain-id", chain_id)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_reports(
    report_type: &str,
    address: &str,
    chain_id: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("reports")
        .subcommand(report_type)
        .opt("--chain-id", chain_id)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// 1INCH (7 subcommands)
// =============================================================================

pub async fn oneinch_quote(
    src: &str,
    dst: &str,
    amount: &str,
    chain_id: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("1inch")
        .subcommand("quote")
        .arg(src)
        .arg(dst)
        .arg(amount);

    if let Some(c) = chain_id {
        builder = builder.opt("--chain-id", Some(&c.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn oneinch_swap(
    src: &str,
    dst: &str,
    amount: &str,
    from: &str,
    chain_id: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("1inch")
        .subcommand("swap")
        .arg(src)
        .arg(dst)
        .arg(amount)
        .arg(from);

    if let Some(c) = chain_id {
        builder = builder.opt("--chain-id", Some(&c.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn oneinch_tokens(chain_id: Option<u64>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("1inch").subcommand("tokens");

    if let Some(c) = chain_id {
        builder = builder.opt("--chain-id", Some(&c.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn oneinch_sources(chain_id: Option<u64>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("1inch").subcommand("sources");

    if let Some(c) = chain_id {
        builder = builder.opt("--chain-id", Some(&c.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn oneinch_spender(chain_id: Option<u64>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("1inch").subcommand("spender");

    if let Some(c) = chain_id {
        builder = builder.opt("--chain-id", Some(&c.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn oneinch_allowance(
    token: &str,
    owner: &str,
    chain_id: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("1inch")
        .subcommand("allowance")
        .arg(token)
        .arg(owner);

    if let Some(c) = chain_id {
        builder = builder.opt("--chain-id", Some(&c.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn oneinch_approve(
    token: &str,
    amount: Option<&str>,
    chain_id: Option<u64>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("1inch")
        .subcommand("approve")
        .arg(token)
        .opt("--amount", amount);

    if let Some(c) = chain_id {
        builder = builder.opt("--chain-id", Some(&c.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// OPEN-OCEAN (5 subcommands)
// =============================================================================

pub async fn openocean_quote(
    in_token: &str,
    out_token: &str,
    amount: &str,
    chain: Option<&str>,
    gas_price: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("open-ocean")
        .subcommand("quote")
        .arg(in_token)
        .arg(out_token)
        .arg(amount)
        .chain(chain)
        .opt("--gas-price", gas_price)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn openocean_swap(
    in_token: &str,
    out_token: &str,
    amount: &str,
    account: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("open-ocean")
        .subcommand("swap")
        .arg(in_token)
        .arg(out_token)
        .arg(amount)
        .arg(account)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn openocean_reverse_quote(
    in_token: &str,
    out_token: &str,
    amount: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("open-ocean")
        .subcommand("reverse-quote")
        .arg(in_token)
        .arg(out_token)
        .arg(amount)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn openocean_tokens(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("open-ocean")
        .subcommand("tokens")
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn openocean_dexes(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("open-ocean")
        .subcommand("dexes")
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// KYBER-SWAP (3 subcommands)
// =============================================================================

pub async fn kyberswap_routes(
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("kyber-swap")
        .subcommand("routes")
        .arg(token_in)
        .arg(token_out)
        .arg(amount_in)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kyberswap_route_data(
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("kyber-swap")
        .subcommand("route-data")
        .arg(token_in)
        .arg(token_out)
        .arg(amount_in)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kyberswap_build(
    token_in: &str,
    token_out: &str,
    amount_in: &str,
    sender: &str,
    recipient: &str,
    route_summary: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("kyber-swap")
        .subcommand("build")
        .arg(token_in)
        .arg(token_out)
        .arg(amount_in)
        .arg(sender)
        .arg(recipient)
        .chain(chain)
        .opt("--route-summary", Some(route_summary))
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// 0X (3 subcommands)
// =============================================================================

pub async fn zerox_quote(
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
    taker: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("0x")
        .subcommand("quote")
        .arg(sell_token)
        .arg(buy_token)
        .arg(sell_amount)
        .arg(taker)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn zerox_price(
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
    taker: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("0x")
        .subcommand("price")
        .arg(sell_token)
        .arg(buy_token)
        .arg(sell_amount)
        .arg(taker)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn zerox_sources(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("0x")
        .subcommand("sources")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// COW-SWAP (8 subcommands)
// =============================================================================

pub async fn cowswap_quote(
    sell_token: &str,
    buy_token: &str,
    amount: &str,
    from: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("quote")
        .arg(sell_token)
        .arg(buy_token)
        .arg(amount)
        .arg(from)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_order(order_uid: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("order")
        .arg(order_uid)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_orders(owner: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("orders")
        .arg(owner)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_trades(owner: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("trades")
        .arg(owner)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_order_trades(
    order_uid: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("order-trades")
        .arg(order_uid)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_auction(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("auction")
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_competition(
    auction_id: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("competition")
        .arg(auction_id)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_native_price(token: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("native-price")
        .arg(token)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

#[allow(clippy::too_many_arguments)]
pub async fn cowswap_create_order(
    sell_token: &str,
    buy_token: &str,
    sell_amount: &str,
    buy_amount: &str,
    valid_to: u64,
    from: &str,
    receiver: &str,
    signature: &str,
    kind: &str,
    signing_scheme: &str,
    fee_amount: Option<&str>,
    app_data: Option<&str>,
    partially_fillable: bool,
    quote_id: Option<i64>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("cow-swap")
        .subcommand("create-order")
        .arg(sell_token)
        .arg(buy_token)
        .arg(sell_amount)
        .arg(buy_amount)
        .arg(&valid_to.to_string())
        .arg(from)
        .arg(receiver)
        .arg(signature)
        .opt("--kind", Some(kind))
        .opt("--signing-scheme", Some(signing_scheme));

    builder = builder.opt("--fee-amount", fee_amount);
    builder = builder.opt("--app-data", app_data);
    builder = builder.opt_flag("--partially-fillable", partially_fillable);
    if let Some(qid) = quote_id {
        builder = builder.opt("--quote-id", Some(&qid.to_string()));
    }
    builder = builder.chain(chain);

    builder
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn cowswap_cancel_order(
    uid: &str,
    signature: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("cow-swap")
        .subcommand("cancel-order")
        .arg(uid)
        .arg(signature)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// LIFI (12 subcommands)
// =============================================================================

pub async fn lifi_quote(
    from_chain: &str,
    from_token: &str,
    to_chain: &str,
    to_token: &str,
    amount: &str,
    from_address: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("quote")
        .arg(from_chain)
        .arg(from_token)
        .arg(to_chain)
        .arg(to_token)
        .arg(amount)
        .arg(from_address)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_routes(
    from_chain: &str,
    from_token: &str,
    to_chain: &str,
    to_token: &str,
    amount: &str,
    from_address: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("routes")
        .arg(from_chain)
        .arg(from_token)
        .arg(to_chain)
        .arg(to_token)
        .arg(amount)
        .arg(from_address)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_best_route(
    from_chain: &str,
    from_token: &str,
    to_chain: &str,
    to_token: &str,
    amount: &str,
    from_address: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("best-route")
        .arg(from_chain)
        .arg(from_token)
        .arg(to_chain)
        .arg(to_token)
        .arg(amount)
        .arg(from_address)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_status(tx_hash: &str, bridge: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("status")
        .arg(tx_hash)
        .opt("--bridge", bridge)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_chains() -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("chains")
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_chain(chain: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("chain")
        .arg(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_tokens(chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("tokens")
        .opt("--chain-id", chain_id)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_tools() -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("tools")
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_bridges() -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("bridges")
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_exchanges() -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("exchanges")
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_gas(chain_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("gas")
        .arg(chain_id)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_connections(
    from_chain: Option<&str>,
    to_chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("connections")
        .opt("--from-chain", from_chain)
        .opt("--to-chain", to_chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_get_transaction(
    from_chain: &str,
    from_token: &str,
    to_chain: &str,
    to_token: &str,
    amount: &str,
    from_address: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("get-transaction")
        .arg(from_chain)
        .arg(from_token)
        .arg(to_chain)
        .arg(to_token)
        .arg(amount)
        .arg(from_address)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn lifi_step_transaction(step_json: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("step-transaction")
        .arg(step_json)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// VELORA (3 subcommands)
// =============================================================================

pub async fn velora_price(
    src_token: &str,
    dest_token: &str,
    amount: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("velora")
        .subcommand("price")
        .arg(src_token)
        .arg(dest_token)
        .arg(amount)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn velora_transaction(
    user_address: &str,
    price_route: &str,
    chain: Option<&str>,
    slippage: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("velora")
        .subcommand("transaction")
        .arg(user_address)
        .chain(chain)
        .opt("--slippage", slippage)
        .opt("--price-route", Some(price_route))
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn velora_tokens(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("velora")
        .subcommand("tokens")
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ENSO (3 subcommands)
// =============================================================================

pub async fn enso_route(
    from_token: &str,
    to_token: &str,
    amount: &str,
    from_address: &str,
    chain_id: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("enso")
        .subcommand("route")
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .arg(from_address);
    if let Some(cid) = chain_id {
        builder = builder.opt("--chain-id", Some(cid));
    }
    builder
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn enso_price(token: &str, chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("enso")
        .subcommand("price")
        .arg(token)
        .opt("--chain-id", chain_id)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn enso_balances(address: &str, chain_id: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("enso")
        .subcommand("balances")
        .arg(address)
        .opt("--chain-id", chain_id)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn enso_bundle(
    from_address: &str,
    actions_json: &str,
    chain_id: Option<&str>,
    routing_strategy: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("enso")
        .subcommand("bundle")
        .arg(from_address)
        .arg(actions_json);
    if let Some(cid) = chain_id {
        builder = builder.opt("--chain-id", Some(cid));
    }
    if let Some(rs) = routing_strategy {
        builder = builder.opt("--routing-strategy", Some(rs));
    }
    builder
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// PYTH (4 subcommands)
// =============================================================================

pub async fn pyth_price(symbols: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("pyth")
        .subcommand("price")
        .arg(symbols)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn pyth_search(query: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("pyth")
        .subcommand("search")
        .arg(query)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn pyth_feeds(asset_type: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("pyth")
        .subcommand("feeds")
        .opt("--asset-type", asset_type)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn pyth_known_feeds() -> Result<String, ToolError> {
    ArgsBuilder::new("pyth")
        .subcommand("known-feeds")
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// CONFIG (16 subcommands)
// =============================================================================

pub async fn config_init() -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("init")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_path() -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("path")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_show() -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("show")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_validate() -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("validate")
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_etherscan_key(key: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-etherscan-key")
        .arg(key)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_tenderly(
    account: &str,
    project: &str,
    key: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-tenderly")
        .opt("--key", Some(key))
        .opt("--account", Some(account))
        .opt("--project", Some(project))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_alchemy(key: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-alchemy")
        .arg(key)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_moralis(key: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-moralis")
        .arg(key)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_chainlink(
    client_id: &str,
    client_secret: &str,
) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-chainlink")
        .opt("--key", Some(client_id))
        .opt("--secret", Some(client_secret))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_dune(key: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-dune")
        .arg(key)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_dune_sim(key: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-dune-sim")
        .arg(key)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_set_solodit(key: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("set-solodit")
        .arg(key)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_add_debug_rpc(url: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("add-debug-rpc")
        .arg(url)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_remove_debug_rpc(url: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("remove-debug-rpc")
        .arg(url)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ENDPOINTS (9 subcommands)
// =============================================================================

pub async fn endpoints_list(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("list")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_add(
    url: &str,
    chain: Option<&str>,
    no_optimize: bool,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("endpoints")
        .subcommand("add")
        .arg(url)
        .chain(chain);

    if no_optimize {
        builder = builder.arg("--no-optimize");
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn endpoints_remove(url: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("remove")
        .arg(url)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_optimize(
    url: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("endpoints")
        .subcommand("optimize")
        .chain(chain);

    if let Some(u) = url {
        builder = builder.arg(u);
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn endpoints_test(url: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("test")
        .arg(url)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_enable(url: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("enable")
        .arg(url)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_disable(url: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("disable")
        .arg(url)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_health(chain: Option<&str>, probes: u32) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("health")
        .chain(chain)
        .opt_flag("--json", true)
        .opt("--probes", Some(&probes.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// CHAINLIST (4 subcommands)
// =============================================================================

pub async fn chainlist_search(query: &str, testnets: bool) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlist")
        .subcommand("search")
        .arg(query)
        .opt_flag("--testnets", testnets)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlist_rpcs(chain: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlist")
        .subcommand("rpcs")
        .arg(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlist_add(chain: &str, max: usize) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlist")
        .subcommand("add")
        .arg(chain)
        .opt("--max", Some(&max.to_string()))
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn chainlist_list(testnets: bool) -> Result<String, ToolError> {
    ArgsBuilder::new("chainlist")
        .subcommand("list")
        .opt_flag("--testnets", testnets)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// HEALTH CHECK
// =============================================================================

/// Health check - verify ethcli-mcp and ethcli are working
pub async fn health() -> String {
    use crate::executor::metrics;
    use std::time::Instant;

    let start = Instant::now();
    let mut status = String::from("ethcli-mcp health check\n");
    status.push_str("========================\n\n");

    // Check ethcli binary
    let ethcli_check = ArgsBuilder::new("--version").execute().await;
    match ethcli_check {
        Ok(version) => {
            status.push_str(&format!("ethcli: OK ({})\n", version.trim()));
        }
        Err(e) => {
            status.push_str(&format!("ethcli: FAILED ({})\n", e));
        }
    }

    // Get metrics
    let m = metrics().snapshot();
    status.push_str("\nMetrics:\n");
    status.push_str(&format!("  commands_total: {}\n", m.commands_total));
    status.push_str(&format!("  commands_success: {}\n", m.commands_success));
    status.push_str(&format!("  commands_failed: {}\n", m.commands_failed));
    status.push_str(&format!("  rate_limited: {}\n", m.rate_limited));
    status.push_str(&format!("  timeouts: {}\n", m.timeouts));

    let elapsed = start.elapsed();
    status.push_str(&format!("\nHealth check completed in {:?}\n", elapsed));

    status
}
