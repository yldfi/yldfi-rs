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
            builder = builder.arg("--topic").arg(&topic);
        }
    }

    builder.execute().await.map_err(ToolError::from)
}

// =============================================================================
// TX (Transaction Analysis)
// =============================================================================

pub async fn tx_analyze(hash: &str, chain: Option<&str>, trace: bool) -> Result<String, ToolError> {
    ArgsBuilder::new("tx")
        .arg(hash)
        .chain(chain)
        .opt_flag("--trace", trace)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ACCOUNT (8 subcommands)
// =============================================================================

pub async fn account_info(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("info")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_balance(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("balance")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_txs(
    address: &str,
    chain: Option<&str>,
    limit: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("account")
        .subcommand("txs")
        .arg(address)
        .chain(chain);

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn account_internal_txs(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("internal-txs")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_erc20(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("erc20")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_erc721(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("erc721")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_erc1155(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("erc1155")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn account_mined_blocks(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("account")
        .subcommand("mined-blocks")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// ADDRESS (7 subcommands) - Address book management
// =============================================================================

pub async fn address_add(name: &str, address: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("add")
        .arg(name)
        .arg(address)
        .execute()
        .await
        .map_err(ToolError::from)
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

pub async fn address_import(file: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("import")
        .arg(file)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn address_export(file: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("address")
        .subcommand("export")
        .arg(file)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// BLACKLIST (7 subcommands) - Sanctions/blacklist checking
// =============================================================================

pub async fn blacklist_add(address: &str, reason: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("add")
        .arg(address)
        .opt("--reason", reason)
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

pub async fn blacklist_path(
    from: &str,
    to: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("blacklist")
        .subcommand("path")
        .arg(from)
        .arg(to)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// CONTRACT (4 subcommands)
// =============================================================================

pub async fn contract_abi(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("abi")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn contract_source(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("source")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn contract_creation(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("contract")
        .subcommand("creation")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn contract_call(
    address: &str,
    sig: &str,
    args: Vec<String>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("contract")
        .subcommand("call")
        .arg(address)
        .opt("--sig", Some(sig))
        .chain(chain);

    for arg in args {
        builder = builder.arg(&arg);
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
        .chain(chain);

    if let Some(l) = limit {
        builder = builder.opt("--limit", Some(&l.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn token_balance(
    token: &str,
    holder: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("token")
        .subcommand("balance")
        .arg(token)
        .opt("--holder", Some(holder))
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// GAS (2 subcommands)
// =============================================================================

pub async fn gas_oracle(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("gas")
        .subcommand("oracle")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn gas_estimate(
    to: &str,
    value: Option<&str>,
    data: Option<&str>,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("gas")
        .subcommand("estimate")
        .arg(to)
        .opt("--value", value)
        .opt("--data", data)
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
        .opt("--nonce", Some(nonce))
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
        .opt("--salt", Some(salt))
        .opt("--init-code-hash", Some(init_code_hash))
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

pub async fn rpc_block(block: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("rpc")
        .subcommand("block")
        .arg(block)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
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

pub async fn simulate_call(
    contract: &str,
    sig: &str,
    args: Vec<String>,
    chain: Option<&str>,
    via: Option<&str>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("simulate")
        .subcommand("call")
        .arg(contract)
        .opt("--sig", Some(sig))
        .chain(chain)
        .opt("--via", via);

    for arg in args {
        builder = builder.arg(&arg);
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn simulate_tx(hash: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("simulate")
        .subcommand("tx")
        .arg(hash)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn simulate_bundle(txs: Vec<String>, chain: Option<&str>) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("simulate")
        .subcommand("bundle")
        .chain(chain);

    for tx in txs {
        builder = builder.arg(&tx);
    }

    builder.execute().await.map_err(ToolError::from)
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
        .arg(contract)
        .arg(data)
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

pub async fn portfolio(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("portfolio")
        .arg(address)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
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
        .opt("--protocol", protocol)
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

pub async fn alchemy_nft(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("nft")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_token(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("token")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_transfers(
    address: &str,
    chain: Option<&str>,
    category: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("transfers")
        .arg(address)
        .chain(chain)
        .opt("--category", category)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_portfolio(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("portfolio")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_prices(tokens: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("prices")
        .arg(tokens)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn alchemy_debug(hash: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("alchemy")
        .subcommand("debug")
        .arg(hash)
        .chain(chain)
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
        .opt("--vs-currencies", vs)
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
        .subcommand("list")
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
        .arg(addresses)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_yields(
    chain: Option<&str>,
    protocol: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("yields")
        .chain(chain)
        .opt("--project", protocol)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_volumes(protocol: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("volumes")
        .opt("--protocol", protocol)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn llama_fees(protocol: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("llama")
        .subcommand("fees")
        .opt("--protocol", protocol)
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
// MORALIS (5 subcommands)
// =============================================================================

pub async fn moralis_wallet(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("wallet")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_token(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("token")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_nft(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("nft")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_resolve(domain: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("resolve")
        .arg(domain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn moralis_market() -> Result<String, ToolError> {
    ArgsBuilder::new("moralis")
        .subcommand("market")
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

pub async fn dsim_balances(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("balances")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_collectibles(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("collectibles")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_activity(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("activity")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_token(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("token")
        .arg(address)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_holders(token: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("holders")
        .arg(token)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dsim_defi(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("dsim")
        .subcommand("defi")
        .arg(address)
        .chain(chain)
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
        .arg(query_id)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_sql(sql: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("sql")
        .arg(sql)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn dune_execution(execution_id: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("dune")
        .subcommand("execution")
        .arg(execution_id)
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
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_tokens(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("tokens")
        .chain(chain)
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
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_ohlc(pool: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("ohlc")
        .arg(pool)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn curve_trades(pool: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("curve")
        .subcommand("trades")
        .arg(pool)
        .chain(chain)
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

// =============================================================================
// QUOTE (3 subcommands)
// =============================================================================

pub async fn quote_best(
    from_token: &str,
    to_token: &str,
    amount: &str,
    chain: Option<&str>,
    slippage: Option<u32>,
) -> Result<String, ToolError> {
    let mut builder = ArgsBuilder::new("quote")
        .subcommand("best")
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .chain(chain);

    if let Some(s) = slippage {
        builder = builder.opt("--slippage", Some(&s.to_string()));
    }

    builder.execute().await.map_err(ToolError::from)
}

pub async fn quote_from(
    aggregator: &str,
    from_token: &str,
    to_token: &str,
    amount: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("quote")
        .subcommand("from")
        .arg(aggregator)
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn quote_compare(
    from_token: &str,
    to_token: &str,
    amount: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("quote")
        .subcommand("compare")
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
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

pub async fn chainlink_streams() -> Result<String, ToolError> {
    ArgsBuilder::new("chainlink")
        .subcommand("streams")
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

pub async fn ccxt_compare(symbol: &str, exchanges: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("ccxt")
        .subcommand("compare")
        .arg(symbol)
        .opt("--exchanges", exchanges)
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
    address: &str,
    token: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("uniswap")
        .subcommand("balance")
        .arg(address)
        .arg(token)
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

pub async fn kong_vaults(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("vaults")
        .subcommand("list")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_strategies(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("strategies")
        .subcommand("list")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_prices(tokens: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("prices")
        .arg(tokens)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_tvl(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("tvl")
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn kong_reports(vault: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("kong")
        .subcommand("reports")
        .arg(vault)
        .chain(chain)
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
) -> Result<String, ToolError> {
    ArgsBuilder::new("open-ocean")
        .subcommand("quote")
        .arg(in_token)
        .arg(out_token)
        .arg(amount)
        .chain(chain)
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
    route_summary: &str,
    sender: &str,
    recipient: &str,
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("kyber-swap")
        .subcommand("build")
        .arg(route_summary)
        .arg(sender)
        .arg(recipient)
        .chain(chain)
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
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("0x")
        .subcommand("price")
        .arg(sell_token)
        .arg(buy_token)
        .arg(sell_amount)
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

pub async fn lifi_tokens(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("tokens")
        .chain(chain)
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

pub async fn lifi_connections(from_chain: &str, to_chain: &str) -> Result<String, ToolError> {
    ArgsBuilder::new("lifi")
        .subcommand("connections")
        .arg(from_chain)
        .arg(to_chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

// =============================================================================
// VELORA (3 subcommands)
// =============================================================================

pub async fn velora_price(token: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("velora")
        .subcommand("price")
        .arg(token)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn velora_transaction(hash: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("velora")
        .subcommand("transaction")
        .arg(hash)
        .chain(chain)
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
    chain: Option<&str>,
) -> Result<String, ToolError> {
    ArgsBuilder::new("enso")
        .subcommand("route")
        .arg(from_token)
        .arg(to_token)
        .arg(amount)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn enso_price(token: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("enso")
        .subcommand("price")
        .arg(token)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn enso_balances(address: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("enso")
        .subcommand("balances")
        .arg(address)
        .chain(chain)
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
        .arg(account)
        .arg(project)
        .arg(key)
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
        .arg(client_id)
        .arg(client_secret)
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

pub async fn config_add_debug_rpc(url: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("add-debug-rpc")
        .arg(url)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn config_remove_debug_rpc(url: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("config")
        .subcommand("remove-debug-rpc")
        .arg(url)
        .chain(chain)
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

pub async fn endpoints_add(url: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("add")
        .arg(url)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_remove(url: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("remove")
        .arg(url)
        .chain(chain)
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

pub async fn endpoints_test(url: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("test")
        .arg(url)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_enable(url: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("enable")
        .arg(url)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_disable(url: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("disable")
        .arg(url)
        .chain(chain)
        .execute()
        .await
        .map_err(ToolError::from)
}

pub async fn endpoints_health(chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("endpoints")
        .subcommand("health")
        .chain(chain)
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
    status.push_str(&format!("\nMetrics:\n"));
    status.push_str(&format!("  commands_total: {}\n", m.commands_total));
    status.push_str(&format!("  commands_success: {}\n", m.commands_success));
    status.push_str(&format!("  commands_failed: {}\n", m.commands_failed));
    status.push_str(&format!("  rate_limited: {}\n", m.rate_limited));
    status.push_str(&format!("  timeouts: {}\n", m.timeouts));

    let elapsed = start.elapsed();
    status.push_str(&format!("\nHealth check completed in {:?}\n", elapsed));

    status
}
