//! `contract decompile`, `contract fingerprint` and `contract match`: tools
//! for contracts without verified source.
//!
//! `decompile` runs the optional external decompiler evmdec on bytecode that
//! ethcli fetched itself, and puts a cross-check against ethcli's own facts
//! before evmdec's heuristic output. `fingerprint` reads what the runtime code
//! says about its compiler. `match` compares a compiled artifact with the
//! deployed code.

use super::OutputFormat;
use crate::bytecode::calls::{analyze_calls, CallsBudget, CallsReport};
use crate::bytecode::matcher::{assembly_share, match_code, match_creation, parse_artifact};
use crate::bytecode::metadata::fingerprint;
use crate::bytecode::{extract_selectors, ExtractedFunction};
use crate::config::{Chain, EndpointConfig};
use crate::rpc::Endpoint;
use alloy::primitives::{Address, B256, U256};
use alloy::providers::Provider;
use clap::Args;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

/// Pinned evmdec commit the documentation and install hint refer to
pub const EVMDEC_PINNED_COMMIT: &str = "5d8b52c1bcdfe40abd5c7c63e75b22e51e5916f9";

const EVMDEC_INSTALL: &str =
    "evmdec is not installed. It is optional. To install, build from source at a pinned commit:
  git clone https://github.com/rink3y/Evmdec evmdec && cd evmdec
  git checkout 5d8b52c1bcdfe40abd5c7c63e75b22e51e5916f9
  cargo build --release -p evmdec-cli   # or: cargo build --release
then put target/release/evmdec on PATH, or set EVMDEC_PATH to it.
ethcli only ever runs `evmdec decompile <file> --json` on code it fetched itself.";

/// Where to read the code
#[derive(Args, Debug, Clone)]
pub struct TargetArgs {
    /// Contract address
    #[arg(value_name = "ADDRESS")]
    pub address: String,

    /// Block to read the code at: a number, a tag or a time (as on `rpc code`)
    #[arg(long, short = 'b', value_name = "BLOCK")]
    pub block: Option<String>,

    /// Use the address's own code even when it is a proxy
    #[arg(long)]
    pub no_follow_proxy: bool,

    /// Custom RPC URL (overrides config)
    #[arg(long, value_name = "URL")]
    pub rpc_url: Option<String>,

    /// Output format
    #[arg(
        long,
        short = 'o',
        visible_alias = "output",
        value_enum,
        default_value = "table"
    )]
    pub format: OutputFormat,
}

/// `contract decompile`
#[derive(Args, Debug, Clone)]
pub struct DecompileArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// Decompiler backend: evmdec (default), heimdall, or all (every
    /// installed backend, each cross-checked)
    #[arg(long, default_value = "evmdec", value_parser = ["evmdec", "heimdall", "all"])]
    pub backend: String,

    /// Seconds before the decompiler is killed
    #[arg(long, default_value_t = 60, value_parser = clap::value_parser!(u64).range(1..=600))]
    pub timeout: u64,
}

/// `contract fingerprint`
#[derive(Args, Debug, Clone)]
pub struct FingerprintArgs {
    #[command(flatten)]
    pub target: TargetArgs,
}

/// `contract match`
#[derive(Args, Debug, Clone)]
pub struct MatchArgs {
    #[command(flatten)]
    pub target: TargetArgs,

    /// Compiled artifact: forge `out/<File>.sol/<Name>.json` or a solc
    /// standard-JSON output file
    #[arg(long, value_name = "PATH")]
    pub artifact: PathBuf,

    /// Contract to use from a standard-JSON output (Name or file:Name)
    #[arg(long, value_name = "NAME")]
    pub contract: Option<String>,

    /// Deployment transaction: also compare creation code and report the
    /// constructor arguments (top-level deployments only)
    #[arg(long, value_name = "TX_HASH")]
    pub creation_tx: Option<String>,

    /// forge build-info (out/build-info/<id>.json, from `forge build
    /// --build-info`) or standard-JSON output holding every source's AST, so
    /// code from base contracts and other files is attributed. Without it,
    /// a matching file next to a forge artifact is used when present
    #[arg(long, value_name = "PATH")]
    pub build_info: Option<PathBuf>,

    /// Do not look for a build-info file next to the artifact
    #[arg(long)]
    pub no_build_info_discovery: bool,
}

/// Code fetched for one address
struct Target {
    address: Address,
    code_address: Address,
    resolved_via: Option<String>,
    block: u64,
    code: Vec<u8>,
    /// The address's own code, when another code was substituted for it
    /// (proxy or EIP-7702 delegate); it runs on every call too
    own_code: Option<Vec<u8>>,
    notes: Vec<String>,
    endpoint: Endpoint,
}

/// Parse a resolved `--block` value: a decimal or hex number, or a tag
fn parse_block(value: &str) -> anyhow::Result<alloy::eips::BlockNumberOrTag> {
    use alloy::eips::BlockNumberOrTag;
    if let Ok(n) = value.parse::<u64>() {
        return Ok(BlockNumberOrTag::Number(n));
    }
    BlockNumberOrTag::from_str(value).map_err(|_| {
        anyhow::anyhow!(
            "--block '{value}' is not a block number, a tag (latest, earliest, safe, finalized, pending) or a time"
        )
    })
}

const EIP1967_IMPL_SLOT: &str =
    "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
const EIP1967_BEACON_SLOT: &str =
    "0xa3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50";
const OZ_LEGACY_SLOT: &str = "0x7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3";

fn same_word(a: &str, b: &str) -> bool {
    match (U256::from_str(a), U256::from_str(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => false,
    }
}

/// Does this code act as the proxy `kind` says?
#[cfg(test)]
fn acts_as_proxy(kind: &str, code: &[u8], implementation: Address) -> bool {
    proxy_check(kind, code, implementation, None).ok
}

/// What ethcli's call analysis says about a proxy claim
#[derive(Debug, Default)]
struct ProxyCheck {
    /// The code delegates every unmatched call as `kind` says
    ok: bool,
    /// The analysis covered every path, so a negative answer is definite
    complete: bool,
    /// Selectors under which the code delegates as `kind` says, when it
    /// does so only inside specific functions
    only_under: Vec<String>,
}

/// A slot alone proves nothing (any contract can write to it), and so does
/// a DELEGATECALL alone (a contract can delegate to an argument). A
/// DELEGATECALL reached on the dispatcher path (the one every call whose
/// selector the dispatcher does not match runs) must take its target from:
/// - the EIP-1967 or OpenZeppelin legacy implementation slot,
/// - the return of a call to the beacon: read from the EIP-1967 beacon
///   slot, or a code constant equal to the beacon (an immutable beacon, as
///   in OpenZeppelin 5.x BeaconProxy),
/// - a code constant equal to the implementation (EIP-1167 clones).
///
/// Safe proxies and Vyper forwarders are recognised by their exact code.
fn proxy_check(
    kind: &str,
    code: &[u8],
    implementation: Address,
    beacon: Option<Address>,
) -> ProxyCheck {
    if kind == "Safe proxy" {
        return ProxyCheck {
            ok: is_known_safe_proxy(code),
            complete: true,
            ..Default::default()
        };
    }
    if kind == "Vyper forwarder proxy" {
        return ProxyCheck {
            ok: vyper_forwarder(code) == Some(implementation),
            complete: true,
            ..Default::default()
        };
    }
    let report = analyze_calls(code, CallsBudget::default());
    proxy_check_report(kind, &report, implementation, beacon)
}

fn is_constant(p: &crate::bytecode::Provenance, addr: Address) -> bool {
    matches!(p.kind.as_str(), "constant" | "code_constant")
        && p.value
            .as_deref()
            .and_then(|v| U256::from_str(v).ok())
            .is_some_and(|v| v == U256::from_be_slice(addr.as_slice()))
}

/// Is this call exactly `implementation()` with no arguments, by
/// STATICCALL, or by CALL with a value that is a constant zero?
fn is_implementation_call(c: &crate::bytecode::CallSite) -> bool {
    let const_eq = |p: &crate::bytecode::Provenance, want: U256| {
        p.kind == "constant"
            && p.value
                .as_deref()
                .and_then(|v| U256::from_str(v).ok())
                .is_some_and(|v| v == want)
    };
    let kind_ok = match c.kind.as_str() {
        "STATICCALL" => true,
        "CALL" => !c.value.is_empty() && c.value.iter().all(|v| const_eq(v, U256::ZERO)),
        _ => false,
    };
    kind_ok
        && !c.selector.is_empty()
        && c.selector
            .iter()
            .all(|s| s.kind == "constant" && s.value.as_deref() == Some("0x5c60da1b"))
        && !c.args_len.is_empty()
        && c.args_len.iter().all(|l| const_eq(l, U256::from(4u8)))
}

fn proxy_check_report(
    kind: &str,
    report: &CallsReport,
    implementation: Address,
    beacon: Option<Address>,
) -> ProxyCheck {
    let from_slot = |p: &crate::bytecode::Provenance, slot: &str| {
        p.kind == "storage" && p.slot.as_deref().is_some_and(|s| same_word(s, slot))
    };
    let mut check = ProxyCheck {
        complete: report.coverage.complete,
        ..Default::default()
    };
    for site in report
        .call_sites
        .iter()
        .filter(|s| s.kind == "DELEGATECALL")
    {
        let matches = site.target.iter().any(|t| match kind {
            "EIP-1967 proxy" => from_slot(t, EIP1967_IMPL_SLOT),
            "OpenZeppelin legacy proxy" => from_slot(t, OZ_LEGACY_SLOT),
            "EIP-1967 beacon proxy" | "beacon proxy (immutable beacon)" => {
                t.kind == "call_return"
                    && t.call_pc.is_some_and(|pc| {
                        report.call_sites.iter().any(|c| {
                            c.pc == pc
                                && is_implementation_call(c)
                                && c.target.iter().any(|b| {
                                    from_slot(b, EIP1967_BEACON_SLOT)
                                        || beacon.is_some_and(|addr| is_constant(b, addr))
                                })
                        })
                    })
            }
            k if k.starts_with("EIP-1167") => is_constant(t, implementation),
            _ => false,
        });
        if !matches {
            continue;
        }
        if site.functions.iter().any(|f| f == "dispatcher") {
            check.ok = true;
        } else {
            check.only_under.extend(site.functions.iter().cloned());
        }
    }
    check.only_under.sort();
    check.only_under.dedup();
    check
}

/// A Vyper `create_forwarder_to` proxy (Uniswap V1 exchanges, Curve
/// factory pools): copy calldata, DELEGATECALL a code constant, return.
/// The two known byte patterns, with the target address
fn vyper_forwarder(code: &[u8]) -> Option<Address> {
    const PREFIX: &str = "366000600037611000600036600073";
    const SUFFIXES: &[&str] = &["5af4602c57600080fd5b6110006000f3", "5af41558576110006000f3"];
    let hex_code = hex::encode(code);
    let rest = hex_code.strip_prefix(PREFIX)?;
    let (addr, tail) = rest.split_at_checked(40)?;
    SUFFIXES
        .contains(&tail)
        .then(|| Address::from_str(addr).ok())
        .flatten()
}

/// A beacon held in the code (immutable): a DELEGATECALL on the dispatcher
/// path whose target is the return of a call to a code constant. Returns
/// that constant
fn immutable_beacon(code: &[u8]) -> Option<Address> {
    let report = analyze_calls(code, CallsBudget::default());
    report
        .call_sites
        .iter()
        .filter(|s| s.kind == "DELEGATECALL" && s.functions.iter().any(|f| f == "dispatcher"))
        .flat_map(|s| s.target.iter())
        .filter(|t| t.kind == "call_return")
        .find_map(|t| {
            let call = report
                .call_sites
                .iter()
                .find(|c| Some(c.pc) == t.call_pc && is_implementation_call(c))?;
            call.target.iter().find_map(|b| {
                if matches!(b.kind.as_str(), "constant" | "code_constant") {
                    b.value.as_deref().and_then(|v| Address::from_str(v).ok())
                } else {
                    None
                }
            })
        })
}

/// Safe proxy runtime code, exactly as the canonical proxy factories of
/// safe-global/safe-deployments deploy it (Safe 1.0.0, 1.1.1, 1.3.0, 1.4.1,
/// 1.5.0; the runtime does not depend on the singleton, which is in
/// storage). zkSync deployments run EraVM code and are not covered.
const SAFE_PROXY_RUNTIMES: &[&str] = &[
    // 1.0.0
    "608060405273ffffffffffffffffffffffffffffffffffffffff600054163660008037600080366000845af43d6000803e6000811415603d573d6000fd5b3d6000f3fea165627a7a723058201e7d648b83cfac072cbccefc2ffc62a6999d4a050ee87a721942de1da9670db80029",
    // 1.1.1
    "608060405273ffffffffffffffffffffffffffffffffffffffff600054167fa619486e0000000000000000000000000000000000000000000000000000000060003514156050578060005260206000f35b3660008037600080366000845af43d6000803e60008114156070573d6000fd5b3d6000f3fea265627a7a72315820d8a00dc4fe6bf675a9d7416fc2d00bb3433362aa8186b750f76c4027269667ff64736f6c634300050e0032",
    // 1.3.0
    "608060405273ffffffffffffffffffffffffffffffffffffffff600054167fa619486e0000000000000000000000000000000000000000000000000000000060003514156050578060005260206000f35b3660008037600080366000845af43d6000803e60008114156070573d6000fd5b3d6000f3fea2646970667358221220d1429297349653a4918076d650332de1a1068c5f3e07c5c82360c277770b955264736f6c63430007060033",
    // 1.4.1
    "608060405273ffffffffffffffffffffffffffffffffffffffff600054167fa619486e0000000000000000000000000000000000000000000000000000000060003514156050578060005260206000f35b3660008037600080366000845af43d6000803e60008114156070573d6000fd5b3d6000f3fea264697066735822122003d1488ee65e08fa41e58e888a9865554c535f2c77126a82cb4c0f917f31441364736f6c63430007060033",
    // 1.5.0
    "608060405260005463a619486e60003560e01c14156024578060601b606c5260206060f35b3660008037600080366000845af43d6000803e806040573d6000fd5b3d6000f3fea2646970667358221220e61834ebd2d8cd909d362bf67c47ef58fd665df38e6dd036ce65611101d072e964736f6c63430007060033",
];

fn is_known_safe_proxy(code: &[u8]) -> bool {
    let hex_code = hex::encode(code);
    SAFE_PROXY_RUNTIMES.contains(&hex_code.as_str())
}

async fn fetch_target(chain: Chain, args: &TargetArgs, quiet: bool) -> anyhow::Result<Target> {
    let address =
        Address::from_str(&args.address).map_err(|e| anyhow::anyhow!("Invalid address: {e}"))?;
    let block_arg = match &args.block {
        Some(b) => Some(
            super::contract::resolve_block_arg(chain, args.rpc_url.as_deref(), b, quiet).await?,
        ),
        None => None,
    };
    let tag = block_arg.as_deref().map(parse_block).transpose()?;
    let block_number_arg = tag.and_then(|t| t.as_number());
    let endpoint = match &args.rpc_url {
        Some(url) => Endpoint::new(EndpointConfig::new(url.to_string()), 30, None)?,
        None => crate::rpc::selector::get_rpc_endpoint_at(chain, block_number_arg)?,
    };
    let provider = endpoint.provider();
    use alloy::eips::BlockNumberOrTag as Tag;
    let block = match tag {
        Some(Tag::Number(n)) => n,
        Some(Tag::Earliest) => 0,
        None | Some(Tag::Latest) | Some(Tag::Pending) => provider
            .get_block_number()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get the latest block: {e}"))?,
        Some(t) => {
            provider
                .get_block_by_number(t)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to resolve --block {t}: {e}"))?
                .ok_or_else(|| anyhow::anyhow!("--block {t}: no such block"))?
                .header
                .number
        }
    };
    let block_id: alloy::eips::BlockId = block.into();
    let get_code = |a: Address| async move {
        provider
            .get_code_at(a)
            .block_id(block_id)
            .await
            .map(|c| c.to_vec())
            .map_err(|e| anyhow::anyhow!("Failed to fetch code: {e}"))
    };
    let mut code = get_code(address).await?;
    let mut code_address = address;
    let mut resolved_via = None;
    let mut own_code = None;
    let mut notes = Vec::new();

    if code.len() == 23 && code[..3] == [0xef, 0x01, 0x00] {
        let delegate = Address::from_slice(&code[3..]);
        notes.push(format!(
            "{} is an EOA delegating under EIP-7702 to {}; using the delegate's code",
            address.to_checksum(None),
            delegate.to_checksum(None)
        ));
        let delegate_code = get_code(delegate).await?;
        own_code = Some(std::mem::replace(&mut code, delegate_code));
        code_address = delegate;
        resolved_via = Some("EIP-7702 delegation".to_string());
    } else if !code.is_empty() && !args.no_follow_proxy {
        // A canonical Safe proxy keeps its singleton in slot 0; Safe 1.0.0
        // proxies do not answer masterCopy(), so read the slot directly
        let safe_singleton = if is_known_safe_proxy(&code) {
            super::contract::storage_address(provider, address, B256::ZERO, block_id)
                .await
                .map(|a| ("Safe proxy", a))
        } else {
            None
        };
        let mut found = match safe_singleton {
            Some(found) => Some(found),
            None => super::contract::find_implementation(provider, address, &code, block_id).await,
        };
        let mut beacon = super::contract::storage_address(
            provider,
            address,
            B256::from_str(EIP1967_BEACON_SLOT).expect("valid slot"),
            block_id,
        )
        .await;
        if found.is_none() {
            if let Some(target) = vyper_forwarder(&code) {
                found = Some(("Vyper forwarder proxy", target));
            } else if let Some(b) = immutable_beacon(&code) {
                if let Some(imp) =
                    super::contract::call_implementation_function_at(provider, b, block_id).await
                {
                    beacon = Some(b);
                    found = Some(("beacon proxy (immutable beacon)", imp));
                }
            }
        }
        if let Some((kind, implementation)) = found {
            let check = if kind.starts_with("implementation() getter") {
                ProxyCheck::default()
            } else {
                proxy_check(kind, &code, implementation, beacon)
            };
            if kind.starts_with("implementation() getter") {
                notes.push(format!(
                    "answers implementation() with {}, but that alone does not make it a proxy; using its own code",
                    implementation.to_checksum(None)
                ));
            } else if !check.ok {
                let warning = if !check.only_under.is_empty() {
                    format!(
                        "{kind}: points to {}, but this address delegates to it only under selector {}, not for every call; analysing its own code",
                        implementation.to_checksum(None),
                        check.only_under.join(", ")
                    )
                } else if check.complete {
                    format!(
                        "{kind}: points to {}, but ethcli's call analysis (complete) finds no DELEGATECALL in this address's own code whose target comes from that slot, beacon or constant, so it does not run that code; analysing its own code",
                        implementation.to_checksum(None)
                    )
                } else {
                    format!(
                        "{kind}: points to {}, but ethcli's call analysis could not confirm that this address delegates to it (coverage incomplete); analysing its own code",
                        implementation.to_checksum(None)
                    )
                };
                if !quiet {
                    eprintln!("Warning: {warning}");
                }
                notes.push(warning);
            } else {
                let impl_code = get_code(implementation).await?;
                if impl_code.is_empty() {
                    notes.push(format!(
                        "{kind} implementation {} has no code; using the proxy's own code",
                        implementation.to_checksum(None)
                    ));
                } else {
                    if !quiet {
                        eprintln!(
                            "{kind}: using implementation {}",
                            implementation.to_checksum(None)
                        );
                    }
                    own_code = Some(std::mem::replace(&mut code, impl_code));
                    code_address = implementation;
                    resolved_via = Some(kind.to_string());
                }
            }
        }
    }
    if code.is_empty() {
        anyhow::bail!(
            "No code at {} at block {block} (an EOA, or not deployed yet)",
            code_address.to_checksum(None)
        );
    }
    Ok(Target {
        address,
        code_address,
        resolved_via,
        block,
        code,
        own_code,
        notes,
        endpoint,
    })
}

/// Facts about the address's own code when another code was analysed: a
/// proxy's own code runs on every call and can hold its own checks
fn own_code_json(t: &Target) -> Value {
    match &t.own_code {
        Some(code) => json!({
            "address": t.address.to_checksum(None),
            "code_size": code.len(),
            "note": "the address's own code, which runs on every call before delegating",
            "sensitive_opcodes": sensitive_census(code),
        }),
        None => Value::Null,
    }
}

fn print_own_code(out: &Value) {
    let own = &out["own_code"];
    if own.is_null() {
        return;
    }
    let positions = own["sensitive_opcodes"]["positions"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    let list: Vec<String> = positions
        .iter()
        .map(|(k, v)| format!("{k} x{}", v.as_array().map(Vec::len).unwrap_or(0)))
        .collect();
    println!(
        "Address's own code ({} bytes, runs on every call): {}",
        own["code_size"],
        if list.is_empty() {
            "no sensitive opcodes".to_string()
        } else {
            list.join(", ")
        }
    );
}

fn target_json(chain: Chain, t: &Target) -> Value {
    let mut v = target_json_inner(chain, t);
    v["own_code"] = own_code_json(t);
    v
}

fn target_json_inner(chain: Chain, t: &Target) -> Value {
    json!({
        "address": t.address.to_checksum(None),
        "code_address": t.code_address.to_checksum(None),
        "resolved_via": t.resolved_via,
        "chain": chain.name(),
        "block": t.block,
        "code_size": t.code.len(),
        "code_hash": format!("{:#x}", alloy::primitives::keccak256(&t.code)),
    })
}

fn print_json(format: &OutputFormat, value: &Value) -> anyhow::Result<()> {
    if format.is_ndjson() {
        println!("{}", serde_json::to_string(value)?);
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
    }
    Ok(())
}

/// Dedaub's decompiler page for an address (needs sign-in)
fn dedaub_link(chain: Chain, address: Address) -> String {
    let name = match chain.name() {
        "ethereum" => "ethereum",
        "bsc" | "binance" => "binance",
        other => other,
    };
    format!(
        "https://app.dedaub.com/{name}/address/{:#x}/decompiled",
        address
    )
}

// ---------------------------------------------------------------------------
// evmdec
// ---------------------------------------------------------------------------

/// Find a backend binary: `<NAME>_PATH`, then PATH
fn find_binary(name: &str, env_var: &str) -> Option<PathBuf> {
    if let Some(p) = std::env::var_os(env_var).filter(|p| !p.is_empty()) {
        let p = PathBuf::from(p);
        return p.is_file().then_some(p);
    }
    let path = std::env::var_os("PATH")?;
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    std::env::split_paths(&path)
        .map(|dir| dir.join(&file))
        .find(|p| p.is_file())
}

const HEIMDALL_INSTALL: &str = "heimdall is not installed. It is optional; install it yourself (see github.com/Jon-Becker/heimdall-rs) and put it on PATH or set HEIMDALL_PATH. ethcli never installs or downloads it, and runs only `heimdall decompile <bytecode> --include-sol --skip-resolving` offline.";

/// A temporary directory, removed on drop
struct TempDir(PathBuf);

impl TempDir {
    fn create(tag: &str) -> anyhow::Result<Self> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let path =
            std::env::temp_dir().join(format!("ethcli-{tag}-{}-{nanos}", std::process::id()));
        std::fs::create_dir(&path)?;
        Ok(Self(path))
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// A proxy address that refuses connections: heimdall checks GitHub for a
/// newer release on every run; this makes that request fail at once
const DEAD_PROXY: &str = "http://127.0.0.1:9";

/// Run heimdall on runtime bytecode only. Its config file
/// (`$HOME/.bifrost/config.toml`, which can hold an RPC URL and API keys) is
/// never read: HOME points at an empty temporary directory. All HTTP goes to
/// a dead proxy. Arguments are fixed: no address, RPC, LLM or explorer key.
async fn run_heimdall(bin: &Path, code: &[u8], timeout: Duration) -> Result<Value, String> {
    let dir = TempDir::create("heimdall").map_err(|e| format!("temp dir: {e}"))?;
    let home = dir.0.join("home");
    let out = dir.0.join("out");
    std::fs::create_dir_all(&home).map_err(|e| format!("temp dir: {e}"))?;
    std::fs::create_dir_all(&out).map_err(|e| format!("temp dir: {e}"))?;
    let bytecode = format!("0x{}", hex::encode(code));
    let args: [&std::ffi::OsStr; 7] = [
        "decompile".as_ref(),
        bytecode.as_ref(),
        "--include-sol".as_ref(),
        "--skip-resolving".as_ref(),
        "--default".as_ref(),
        "--output".as_ref(),
        out.as_os_str(),
    ];
    let env = heimdall_env(&home);
    let run = run_sandboxed_in(bin, &args, timeout, &env, &dir.0)
        .await
        .map_err(|e| format!("{e:#}"))?;
    let read = |name: &str| -> Option<String> {
        let path = out.join(name);
        let meta = std::fs::metadata(&path).ok()?;
        (meta.is_file() && meta.len() <= 32 * 1024 * 1024)
            .then(|| std::fs::read_to_string(&path).ok())
            .flatten()
    };
    let source = read("decompiled.sol");
    if source.is_none() {
        return Err(format!(
            "heimdall exited with {:?} and wrote no source: {}",
            run.status,
            String::from_utf8_lossy(&run.stderr)
                .chars()
                .take(600)
                .collect::<String>()
        ));
    }
    let abi: Value = read("abi.json")
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null);
    Ok(heimdall_as_decompiler_json(
        source.unwrap_or_default(),
        &abi,
    ))
}

/// heimdall's environment besides PATH: an empty HOME (so its config file
/// with any RPC URL or API keys is not read) and a dead proxy for all HTTP
fn heimdall_env(home: &Path) -> [(&'static str, &std::ffi::OsStr); 7] {
    let proxy: &std::ffi::OsStr = DEAD_PROXY.as_ref();
    [
        ("HOME", home.as_os_str()),
        ("HTTP_PROXY", proxy),
        ("HTTPS_PROXY", proxy),
        ("ALL_PROXY", proxy),
        ("http_proxy", proxy),
        ("https_proxy", proxy),
        ("all_proxy", proxy),
    ]
}

/// Put heimdall's output in the shape the cross-check reads. With
/// `--skip-resolving` its functions are named `Unresolved_<selector>`
pub fn heimdall_as_decompiler_json(source: String, abi: &Value) -> Value {
    let functions: Vec<Value> = abi
        .as_array()
        .into_iter()
        .flatten()
        .filter(|e| e["type"] == "function")
        .map(|e| {
            let name = e["name"].as_str().unwrap_or("");
            let selector = name
                .strip_prefix("Unresolved_")
                .filter(|s| s.len() == 8 && s.chars().all(|c| c.is_ascii_hexdigit()))
                .map(|s| format!("0x{}", s.to_lowercase()));
            let params: Vec<Value> = e["inputs"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|i| json!({"ty": i["type"]}))
                .collect();
            json!({"selector": selector, "name": name, "params": params, "mutability": e["stateMutability"]})
        })
        .collect();
    json!({"source": source, "functions": functions, "storage": [], "stats": {}})
}

/// The only environment evmdec gets: a PATH for the system tools, nothing
/// else (no RPC URLs, no API keys)
const MINIMAL_PATH: &str = if cfg!(windows) {
    "C:\\Windows\\System32"
} else {
    "/usr/bin:/bin"
};

#[derive(Debug)]
struct RunOutput {
    status: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Run a program with a cleared environment and a hard timeout; the child is
/// killed when the timeout fires
async fn run_sandboxed(
    bin: &Path,
    args: &[&std::ffi::OsStr],
    timeout: Duration,
) -> anyhow::Result<RunOutput> {
    run_sandboxed_in(bin, args, timeout, &[], &std::env::temp_dir()).await
}

/// `run_sandboxed` with a few extra environment variables and a working
/// directory
async fn run_sandboxed_in(
    bin: &Path,
    args: &[&std::ffi::OsStr],
    timeout: Duration,
    extra_env: &[(&str, &std::ffi::OsStr)],
    cwd: &Path,
) -> anyhow::Result<RunOutput> {
    let mut cmd = tokio::process::Command::new(bin);
    cmd.args(args)
        .env_clear()
        .env("PATH", MINIMAL_PATH)
        .envs(extra_env.iter().map(|(k, v)| (*k, *v)))
        .current_dir(cwd)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    let mut child = cmd
        .spawn()
        .map_err(|e| anyhow::anyhow!("could not start {}: {e}", bin.display()))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let read = |pipe: Option<tokio::process::ChildStdout>| async move {
        let mut buf = Vec::new();
        if let Some(mut p) = pipe {
            use tokio::io::AsyncReadExt;
            let _ = p.read_to_end(&mut buf).await;
        }
        buf
    };
    let read_err = |pipe: Option<tokio::process::ChildStderr>| async move {
        let mut buf = Vec::new();
        if let Some(mut p) = pipe {
            use tokio::io::AsyncReadExt;
            let _ = p.read_to_end(&mut buf).await;
        }
        buf
    };
    let work = async {
        let (out, err, status) = tokio::join!(read(stdout), read_err(stderr), child.wait());
        (out, err, status)
    };
    match tokio::time::timeout(timeout, work).await {
        Ok((stdout, stderr, status)) => Ok(RunOutput {
            status: status.ok().and_then(|s| s.code()),
            stdout,
            stderr,
        }),
        Err(_) => {
            let _ = child.kill().await;
            anyhow::bail!(
                "{} did not finish within {} s and was killed",
                bin.display(),
                timeout.as_secs()
            )
        }
    }
}

/// A temporary file holding the code as hex, removed on drop
struct HexFile(PathBuf);

impl HexFile {
    fn create(code: &[u8]) -> anyhow::Result<Self> {
        use std::io::Write;
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or_default();
        let path = std::env::temp_dir().join(format!(
            "ethcli-decompile-{}-{nanos}.hex",
            std::process::id()
        ));
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        // Hex text: evmdec reads a file as hex when it is all hex digits
        f.write_all(format!("0x{}", hex::encode(code)).as_bytes())?;
        Ok(Self(path))
    }
}

impl Drop for HexFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn sha256_file(path: &Path) -> Option<String> {
    use sha2::Digest;
    let bytes = std::fs::read(path).ok()?;
    Some(hex::encode(sha2::Sha256::digest(&bytes)))
}

/// Sensitive opcodes from ethcli's call analysis (`contract calls`):
/// sites executed by an analysed path, sites no path reached, and sites in
/// trailing metadata bytes without a JUMPDEST (data). SSTOREs come from the
/// per-function facts.
#[derive(Debug, Clone, Serialize, Default)]
pub struct OpcodeCensus {
    pub method: &'static str,
    /// opcode -> pcs executed by an analysed path
    pub positions: std::collections::BTreeMap<String, Vec<usize>>,
    /// opcode -> pcs no analysed path reached (dead code, data, or beyond
    /// the analysis)
    pub not_reached: std::collections::BTreeMap<String, Vec<usize>>,
    /// opcode -> pcs in trailing metadata bytes that hold no JUMPDEST
    pub in_metadata: std::collections::BTreeMap<String, Vec<usize>>,
    /// Guards on msg.sender / tx.origin found by the analysis
    pub guards: Vec<String>,
    pub analysis_complete: bool,
    pub warnings: Vec<String>,
}

fn census_from(report: &CallsReport) -> OpcodeCensus {
    let mut c = OpcodeCensus {
        method: "ethcli call analysis (bytecode::calls): symbolic paths over the CFG, plus a linear sweep for unreached sites",
        analysis_complete: report.coverage.complete,
        ..Default::default()
    };
    let mut reached_in_metadata: Vec<String> = Vec::new();
    for site in &report.opcode_sites {
        // Executed code is code, wherever it sits
        let map = if site.reached {
            if site.in_metadata {
                reached_in_metadata.push(format!("{} at {:#x}", site.opcode, site.pc));
            }
            &mut c.positions
        } else if site.in_metadata {
            &mut c.in_metadata
        } else {
            &mut c.not_reached
        };
        map.entry(site.opcode.clone()).or_default().push(site.pc);
    }
    for f in &report.functions {
        for s in f.sstores.iter().filter(|s| !s.transient) {
            c.positions.entry("SSTORE".into()).or_default().push(s.pc);
        }
        for g in &f.guards {
            c.guards.push(format!(
                "{} at {:#x}: {} {} {}",
                f.function,
                g.pc,
                g.subject,
                g.check,
                g.against.describe()
            ));
        }
    }
    for v in c.positions.values_mut() {
        v.sort_unstable();
        v.dedup();
    }
    if !reached_in_metadata.is_empty() {
        c.warnings.push(format!(
            "reached code inside metadata-looking bytes: {}",
            reached_in_metadata.join(", ")
        ));
    }
    if !c.analysis_complete {
        c.warnings.push(
            "ethcli's own call analysis did not cover every path (coverage incomplete); facts below may miss code"
                .into(),
        );
    }
    c
}

fn sensitive_census(code: &[u8]) -> OpcodeCensus {
    census_from(&analyze_calls(code, CallsBudget::default()))
}

/// ethcli's own facts about the code
fn own_facts(code: &[u8]) -> (Vec<ExtractedFunction>, OpcodeCensus) {
    (extract_selectors(code), sensitive_census(code))
}

/// How each sensitive opcode shows in evmdec's output
/// How each sensitive opcode shows in evmdec's output: the text forms to
/// count, whether they must be called (`name(` / `name{`), and the
/// structured `effects` flag that also shows it
type Evidence = (
    &'static str,
    &'static [&'static str],
    bool,
    Option<&'static str>,
    &'static str,
);
const TEXT_EVIDENCE: &[Evidence] = &[
    (
        "ORIGIN",
        &["tx.origin"],
        false,
        None,
        "tx.origin checks may be missing; treat access-control conclusions as incomplete",
    ),
    (
        "CALLER",
        &["msg.sender"],
        false,
        None,
        "msg.sender checks may be missing; treat access-control conclusions as incomplete",
    ),
    (
        "DELEGATECALL",
        &["delegatecall"],
        true,
        Some("delegatecall"),
        "delegatecalls may be missing",
    ),
    (
        "CALLCODE",
        &["callcode"],
        true,
        None,
        "callcodes may be missing",
    ),
    (
        "SELFDESTRUCT",
        &["selfdestruct"],
        true,
        Some("selfdestruct"),
        "selfdestruct may be missing",
    ),
    (
        "CREATE",
        &["create"],
        true,
        Some("creates"),
        "contract creations may be missing",
    ),
    (
        "CREATE2",
        &["create2"],
        true,
        Some("creates"),
        "CREATE2 deployments may be missing",
    ),
];

/// The text with `//` and `/* */` comments and quoted string literals
/// replaced by spaces, so a revert string or comment that mentions
/// `tx.origin` does not count as a check
fn strip_comments_and_strings(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let next = chars.get(i + 1).copied();
        if c == '/' && next == Some('/') {
            while i < chars.len() && chars[i] != '\n' {
                out.push(' ');
                i += 1;
            }
        } else if c == '/' && next == Some('*') {
            i += 2;
            out.push_str("  ");
            while i < chars.len() && !(chars[i] == '*' && chars.get(i + 1) == Some(&'/')) {
                out.push(if chars[i] == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            i = (i + 2).min(chars.len());
            out.push_str("  ");
        } else if c == '"' || c == '\'' {
            let quote = c;
            out.push(' ');
            i += 1;
            while i < chars.len() && chars[i] != quote && chars[i] != '\n' {
                if chars[i] == '\\' {
                    i += 1;
                    out.push(' ');
                }
                out.push(' ');
                i += 1;
            }
            out.push(' ');
            i += 1;
        } else {
            out.push(c);
            i += 1;
        }
    }
    out
}

/// Occurrences of `token` as a whole word; with `call`, only when followed
/// by `(` or `{` (so `createPair` or `create2` never count as `create`)
fn count_word(source: &str, token: &str, call: bool) -> usize {
    let stripped = strip_comments_and_strings(source);
    let source = stripped.as_str();
    let bytes = source.as_bytes();
    let word = |c: u8| c.is_ascii_alphanumeric() || c == b'_';
    source
        .match_indices(token)
        .filter(|(i, _)| {
            let before_ok = *i == 0 || !word(bytes[i - 1]);
            let next = bytes.get(i + token.len()).copied();
            let after_ok = if call {
                matches!(next, Some(b'(') | Some(b'{'))
            } else {
                next.is_none_or(|c| !word(c))
            };
            before_ok && after_ok
        })
        .count()
}

fn pcs(list: &[usize]) -> String {
    let shown: Vec<String> = list.iter().take(8).map(|p| format!("{p:#x}")).collect();
    let more = if list.len() > 8 {
        format!(" and {} more", list.len() - 8)
    } else {
        String::new()
    };
    format!("{}{more}", shown.join(", "))
}

/// Compare evmdec's output with ethcli's facts
pub fn cross_check(
    functions: &[ExtractedFunction],
    census: &OpcodeCensus,
    evmdec: &Value,
) -> Value {
    cross_check_for("evmdec", functions, census, evmdec)
}

/// Compare a decompiler's output with ethcli's facts. `evmdec` is the
/// decompiler's output in evmdec's JSON shape (`source`, `functions` with
/// `selector`/`params`/optional `effects`, `storage`, `stats`)
pub fn cross_check_for(
    backend: &str,
    functions: &[ExtractedFunction],
    census: &OpcodeCensus,
    evmdec: &Value,
) -> Value {
    let source = evmdec["source"].as_str().unwrap_or("");
    let mut warnings: Vec<String> = Vec::new();

    let stats = &evmdec["stats"];
    if let (Some(covered), Some(total)) = (
        stats["covered_blocks"].as_u64(),
        stats["code_blocks"].as_u64(),
    ) {
        if covered < total {
            warnings.push(format!(
                "{backend} covered {covered} of {total} code blocks; code in the other {} is not in the output",
                total - covered
            ));
        }
    }
    if stats["unresolved_jumps"].as_u64().unwrap_or(0) > 0 {
        warnings.push(format!(
            "{backend} left {} jumps unresolved; control flow there is missing",
            stats["unresolved_jumps"]
        ));
    }
    if stats["budget_exhausted"].as_bool() == Some(true) {
        warnings.push(format!(
            "{backend} ran out of its work budget; output is partial"
        ));
    }

    warnings.extend(census.warnings.iter().cloned());
    // Sites an analysed path executed. Unreached sites are mostly data
    // after a halt; with incomplete coverage some may be real code, so they
    // are summarised in one warning instead of compared one by one
    let all = census.positions.clone();
    if !census.analysis_complete {
        let unreached: Vec<String> = census
            .not_reached
            .iter()
            .map(|(k, v)| format!("{k} x{}", v.len()))
            .collect();
        if !unreached.is_empty() {
            warnings.push(format!(
                "sites ethcli's analysis did not reach (data, dead code or unanalysed paths): {}",
                unreached.join(", ")
            ));
        }
    }
    // Guard and control opcodes the analysis did not reach are still named,
    // one by one, when the decompiler's text shows fewer than exist: a
    // check behind a function pointer or other unresolved jump can sit there
    for (name, tokens, call, _, _) in TEXT_EVIDENCE {
        if !matches!(*name, "ORIGIN" | "CALLER" | "DELEGATECALL" | "SELFDESTRUCT") {
            continue;
        }
        let Some(unreached) = census.not_reached.get(*name) else {
            continue;
        };
        let reached = census.positions.get(*name).map_or(0, Vec::len);
        let shown: usize = tokens.iter().map(|t| count_word(source, t, *call)).sum();
        if shown >= reached + unreached.len() {
            continue;
        }
        for pc in unreached.iter().take(5) {
            warnings.push(format!(
                "{name} at pc {pc:#x} not reached by ethcli's analysis and not shown by the decompiler: possible dropped guard"
            ));
        }
        if unreached.len() > 5 {
            warnings.push(format!(
                "and {} more unreached {name} site(s) not shown by the decompiler",
                unreached.len() - 5
            ));
        }
    }
    // Only evmdec reports per-function effects; for other backends the
    // text alone is compared
    let has_effects = evmdec["functions"]
        .as_array()
        .into_iter()
        .flatten()
        .any(|f| f["effects"].is_object());
    let effect_set = |flag: &str| {
        evmdec["functions"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|f| f["effects"][flag].as_bool() == Some(true))
    };
    for (name, tokens, call, effect, what) in TEXT_EVIDENCE {
        let Some(at) = all.get(*name) else {
            continue;
        };
        let shown: usize = tokens.iter().map(|t| count_word(source, t, *call)).sum();
        let effect_missing = has_effects && effect.is_some_and(|e| !effect_set(e));
        if shown < at.len() || effect_missing {
            warnings.push(format!(
                "{} {name} site{} at pc {}, but {backend}'s text shows {} ({}){}; {what}",
                at.len(),
                if at.len() == 1 { "" } else { "s" },
                pcs(at),
                shown,
                tokens.join("/"),
                if effect_missing {
                    format!(" and no function has the `{}` effect", effect.unwrap_or(""))
                } else {
                    String::new()
                }
            ));
        }
    }
    let fns = evmdec["functions"].as_array().cloned().unwrap_or_default();
    if let Some(sstores) = all.get("SSTORE").filter(|_| has_effects) {
        let writes = fns
            .iter()
            .any(|f| f["effects"]["writes_storage"].as_bool() == Some(true));
        if !writes {
            warnings.push(format!(
                "{} SSTORE at pc {} but {backend} reports no function writing storage; state changes are missing",
                sstores.len(),
                pcs(sstores)
            ));
        }
    }

    // Types: evmole vs evmdec, by selector
    let mut types = Vec::new();
    let mut only_ethcli = Vec::new();
    for f in functions {
        let sel = f.selector.to_lowercase();
        match fns
            .iter()
            .find(|d| d["selector"].as_str().map(str::to_lowercase) == Some(sel.clone()))
        {
            Some(d) => {
                let dec: Vec<String> = d["params"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|p| p["ty"].as_str().map(String::from))
                    .collect();
                types.push(json!({
                    "selector": sel,
                    "evmole": format!("({})", f.arguments.join(",")),
                    "decompiler": format!("({})", dec.join(",")),
                    "decompiler_name": d["name"],
                    "agree": f.arguments == dec,
                }));
            }
            None => only_ethcli.push(sel),
        }
    }
    let only_evmdec: Vec<String> = fns
        .iter()
        .filter_map(|d| d["selector"].as_str().map(str::to_lowercase))
        .filter(|s| !functions.iter().any(|f| f.selector.to_lowercase() == *s))
        .collect();
    let disagreements = types.iter().filter(|t| t["agree"] == false).count();
    if disagreements > 0 {
        warnings.push(format!(
            "{disagreements} function(s) where evmole and {backend} disagree on argument types; see type_comparison"
        ));
    }

    // Absence claims, reported as what evmdec did not find
    let mut not_found: Vec<String> = Vec::new();
    for s in evmdec["storage"].as_array().into_iter().flatten() {
        if s["writes"].as_u64() == Some(0) {
            not_found.push(format!(
                "writes to slot {} ({}): not found by {backend}",
                s["slot"].as_str().unwrap_or("?"),
                s["name"].as_str().unwrap_or("?")
            ));
        }
    }

    json!({
        "warnings": warnings,
        "type_comparison": types,
        "selectors_only_in_ethcli": only_ethcli,
        "selectors_only_in_decompiler": only_evmdec,
        "not_found_by_decompiler": not_found,
    })
}

fn facts_json(functions: &[ExtractedFunction], census: &OpcodeCensus) -> Value {
    json!({
        "selectors_source": "evmole (ethcli)",
        "functions": functions,
        "sensitive_opcodes": census,
    })
}

fn print_facts_table(facts: &Value) {
    println!("ethcli facts (from the bytecode itself)");
    let fns = facts["functions"].as_array().cloned().unwrap_or_default();
    println!("  functions (evmole): {}", fns.len());
    for f in &fns {
        let args: Vec<String> = f["arguments"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|a| a.as_str().map(String::from))
            .collect();
        println!(
            "    {} ({}) {}",
            f["selector"].as_str().unwrap_or(""),
            args.join(","),
            f["state_mutability"].as_str().unwrap_or("")
        );
    }
    let census = &facts["sensitive_opcodes"];
    for g in census["guards"].as_array().into_iter().flatten() {
        println!("  guard: {}", g.as_str().unwrap_or(""));
    }
    for (key, label) in [
        (
            "positions",
            "sensitive opcodes executed by ethcli's call analysis",
        ),
        (
            "not_reached",
            "sensitive opcodes no analysed path reached (dead code, data, or beyond the analysis)",
        ),
        (
            "in_metadata",
            "in trailing metadata bytes with no JUMPDEST (data)",
        ),
    ] {
        let entries = census[key].as_object().cloned().unwrap_or_default();
        if entries.is_empty() && key != "positions" {
            continue;
        }
        println!("  {label}:");
        if entries.is_empty() {
            println!("    none");
        }
        for (name, list) in entries {
            let list: Vec<usize> = list
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|v| v.as_u64().map(|n| n as usize))
                .collect();
            println!("    {name:<13} x{:<4} at {}", list.len(), pcs(&list));
        }
    }
}

pub async fn decompile(chain: Chain, args: &DecompileArgs, quiet: bool) -> anyhow::Result<()> {
    let target = fetch_target(chain, &args.target, quiet).await?;
    let (functions, census) = own_facts(&target.code);
    let facts = facts_json(&functions, &census);
    let mut out = json!({
        "target": target_json(chain, &target),
        "notes": target.notes,
        "facts": facts,
        "dedaub": {
            "url": dedaub_link(chain, target.code_address),
            "note": "Dedaub's decompiler; needs sign-in",
        },
    });
    let backends: &[&str] = match args.backend.as_str() {
        "heimdall" => &["heimdall"],
        "all" => &["evmdec", "heimdall"],
        _ => &["evmdec"],
    };
    let timeout = Duration::from_secs(args.timeout);
    let mut results = Vec::new();
    for backend in backends {
        let result = run_backend(backend, &target.code, timeout, quiet).await;
        let mut entry = result.meta;
        match result.output {
            Some(Ok(dec)) => {
                entry["cross_check"] = cross_check_for(backend, &functions, &census, &dec);
                entry["heuristic"] = heuristic_json(backend, &dec);
            }
            Some(Err(e)) => entry["error"] = json!(e),
            None => {}
        }
        // Everything a decompiler returns can carry strings recovered from
        // untrusted bytecode: escape control and direction characters
        sanitise_json(&mut entry);
        results.push(entry);
    }
    out["decompilers"] = json!(results);
    if args.target.format.is_table() {
        print_decompile_table(&out);
        Ok(())
    } else {
        print_json(&args.target.format, &out)
    }
}

/// Untrusted multi-line text made safe to print: control, invisible and
/// direction-changing characters are escaped, except newline and tab
pub fn safe_multiline(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            line.split('\t')
                .map(crate::abi::revert::safe_text)
                .collect::<Vec<_>>()
                .join("\t")
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Apply `safe_multiline` to every string in a JSON value
fn sanitise_json(value: &mut Value) {
    match value {
        // Single-line fields: every control character escaped, newlines too
        Value::String(s) => *s = crate::abi::revert::safe_text(s),
        Value::Array(items) => items.iter_mut().for_each(sanitise_json),
        Value::Object(map) => {
            for (key, v) in map.iter_mut() {
                match (key.as_str(), v) {
                    // Decompiled source keeps its own line breaks, but not
                    // those inside string literals (which come from the
                    // bytecode and could fake lines of output)
                    ("source", Value::String(s)) => {
                        *s = safe_multiline(&escape_newlines_in_literals(s))
                    }
                    (_, v) => sanitise_json(v),
                }
            }
        }
        _ => {}
    }
}

/// Escape newlines and carriage returns that sit inside quoted literals
fn escape_newlines_in_literals(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for c in text.chars() {
        match quote {
            Some(q) => {
                if c == '\n' {
                    out.push_str("\\n");
                    continue;
                }
                if c == '\r' {
                    out.push_str("\\r");
                    continue;
                }
                if escaped {
                    escaped = false;
                } else if c == '\\' {
                    escaped = true;
                } else if c == q {
                    quote = None;
                }
                out.push(c);
            }
            None => {
                if c == '"' {
                    quote = Some(c);
                }
                out.push(c);
            }
        }
    }
    out
}

struct BackendRun {
    meta: Value,
    /// None when the backend is not installed
    output: Option<Result<Value, String>>,
}

async fn run_backend(backend: &str, code: &[u8], timeout: Duration, quiet: bool) -> BackendRun {
    let (env_var, install) = match backend {
        "heimdall" => ("HEIMDALL_PATH", HEIMDALL_INSTALL),
        _ => ("EVMDEC_PATH", EVMDEC_INSTALL),
    };
    let Some(bin) = find_binary(backend, env_var) else {
        return BackendRun {
            meta: json!({"backend": backend, "installed": false, "install": install}),
            output: None,
        };
    };
    let version = backend_version(backend, &bin).await.map(|o| {
        String::from_utf8_lossy(&o.stdout)
            .trim()
            .chars()
            .take(80)
            .collect::<String>()
    });
    let meta = json!({
        "backend": backend,
        "installed": true,
        "path": bin.display().to_string(),
        "version": version,
        "binary_sha256": sha256_file(&bin),
    });
    if !quiet {
        eprintln!("Running {backend} (timeout {} s)...", timeout.as_secs());
    }
    let output = match backend {
        "heimdall" => run_heimdall(&bin, code, timeout).await,
        _ => run_evmdec(&bin, code, timeout).await,
    };
    BackendRun {
        meta,
        output: Some(output),
    }
}

/// `<backend> --version`, under the same sandbox as the real run
async fn backend_version(backend: &str, bin: &Path) -> Option<RunOutput> {
    let timeout = Duration::from_secs(5);
    if backend != "heimdall" {
        return run_sandboxed(bin, &["--version".as_ref()], timeout)
            .await
            .ok();
    }
    let dir = TempDir::create("heimdall").ok()?;
    let home = dir.0.join("home");
    std::fs::create_dir_all(&home).ok()?;
    let env = heimdall_env(&home);
    run_sandboxed_in(bin, &["--version".as_ref()], timeout, &env, &dir.0)
        .await
        .ok()
}

async fn run_evmdec(bin: &Path, code: &[u8], timeout: Duration) -> Result<Value, String> {
    let file = HexFile::create(code).map_err(|e| format!("temp file: {e}"))?;
    let run = run_sandboxed(
        bin,
        &["decompile".as_ref(), file.0.as_os_str(), "--json".as_ref()],
        timeout,
    )
    .await;
    drop(file);
    match run {
        Err(e) => Err(format!("{e:#}")),
        Ok(o) if o.status != Some(0) => Err(format!(
            "evmdec exited with {:?}: {}",
            o.status,
            String::from_utf8_lossy(&o.stderr)
                .chars()
                .take(600)
                .collect::<String>()
        )),
        Ok(o) => {
            serde_json::from_slice(&o.stdout).map_err(|e| format!("evmdec output is not JSON: {e}"))
        }
    }
}

fn heuristic_json(backend: &str, dec: &Value) -> Value {
    let label = match backend {
        "heimdall" => "HEURISTIC: heimdall's reconstruction (symbolic execution), not source. Selectors are not resolved; parameter types and counts, storage names and some conditions are guesses",
        _ => "HEURISTIC: evmdec's reconstruction, not source. Names come from its built-in signature dictionary; types, storage names and control flow are guesses",
    };
    json!({
        "label": label,
        "compiler": dec["compiler"],
        "stats": dec["stats"],
        "functions": dec["functions"].as_array().map(|fs| fs.iter().map(|f| json!({
            "selector": f["selector"], "name": f["name"], "params": f["params"],
            "returns": f["returns"], "mutability": f["mutability"], "effects": f["effects"],
        })).collect::<Vec<_>>()),
        "storage": dec["storage"],
        "call_targets": dec["call_targets"],
        "source": dec["source"],
    })
}

fn print_target_table(out: &Value) {
    let t = &out["target"];
    println!(
        "Target: {} on {} at block {}",
        t["address"].as_str().unwrap_or(""),
        t["chain"].as_str().unwrap_or(""),
        t["block"]
    );
    if let Some(via) = t["resolved_via"].as_str() {
        println!(
            "Decompiled code: {} ({via})",
            t["code_address"].as_str().unwrap_or("")
        );
    }
    println!(
        "Code: {} bytes, hash {}",
        t["code_size"],
        t["code_hash"].as_str().unwrap_or("")
    );
    for n in out["notes"].as_array().into_iter().flatten() {
        println!("Note: {}", n.as_str().unwrap_or(""));
    }
    let wrapped = json!({"own_code": out["target"]["own_code"].clone()});
    print_own_code(&wrapped);
}

fn print_decompile_table(out: &Value) {
    print_target_table(out);
    let decs = out["decompilers"].as_array().cloned().unwrap_or_default();
    for d in &decs {
        let backend = d["backend"].as_str().unwrap_or("?");
        if d["installed"] == false {
            println!();
            println!("{}", d["install"].as_str().unwrap_or(""));
            continue;
        }
        println!(
            "{backend}: {} (sha256 {})",
            d["version"].as_str().unwrap_or("version unknown"),
            d["binary_sha256"]
                .as_str()
                .map(|s| &s[..16.min(s.len())])
                .unwrap_or("?")
        );
    }
    // Cross-checks first, for every backend
    for d in &decs {
        let backend = d["backend"].as_str().unwrap_or("?");
        if let Some(e) = d["error"].as_str() {
            println!();
            println!("{backend} failed: {e}");
            continue;
        }
        let cc = &d["cross_check"];
        if cc.is_null() {
            continue;
        }
        println!();
        println!("== CROSS-CHECK (ethcli facts vs {backend}) ==");
        let warnings = cc["warnings"].as_array().cloned().unwrap_or_default();
        if warnings.is_empty() {
            println!("No disagreement found between ethcli's facts and {backend}'s output.");
        }
        for w in &warnings {
            println!("WARNING: {}", w.as_str().unwrap_or(""));
        }
        let types = cc["type_comparison"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let disagree: Vec<&Value> = types.iter().filter(|t| t["agree"] == false).collect();
        if !disagree.is_empty() {
            println!();
            println!("{:<12} {:<40} {backend}", "selector", "evmole");
            for t in disagree {
                println!(
                    "{:<12} {:<40} {}",
                    t["selector"].as_str().unwrap_or(""),
                    t["evmole"].as_str().unwrap_or(""),
                    t["decompiler"].as_str().unwrap_or("")
                );
            }
        }
        for (key, label) in [
            ("selectors_only_in_ethcli", "selectors only in ethcli"),
            (
                "selectors_only_in_decompiler",
                "selectors only in the decompiler",
            ),
        ] {
            let list = cc[key].as_array().cloned().unwrap_or_default();
            if !list.is_empty() {
                let items: Vec<&str> = list.iter().filter_map(Value::as_str).collect();
                println!("{label}: {}", items.join(", "));
            }
        }
        for n in cc["not_found_by_decompiler"]
            .as_array()
            .into_iter()
            .flatten()
        {
            println!("Not a fact: {}", n.as_str().unwrap_or(""));
        }
    }
    println!();
    print_facts_table(&out["facts"]);
    for d in &decs {
        let Some(src) = d["heuristic"]["source"].as_str() else {
            continue;
        };
        let backend = d["backend"].as_str().unwrap_or("?");
        println!();
        println!(
            "== HEURISTIC PSEUDO-SOLIDITY ({backend}; not source, may be wrong or incomplete) =="
        );
        // Every decompiler line carries a fixed prefix, so nothing in its
        // output can pass for one of ethcli's own lines
        for line in src.lines() {
            println!("| {line}");
        }
        println!();
    }
    println!(
        "Second opinion (sign-in needed): {}",
        out["dedaub"]["url"].as_str().unwrap_or("")
    );
}

// ---------------------------------------------------------------------------
// fingerprint
// ---------------------------------------------------------------------------

pub async fn fingerprint_cmd(
    chain: Chain,
    args: &FingerprintArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let target = fetch_target(chain, &args.target, quiet).await?;
    let analysis = analyze_calls(&target.code, CallsBudget::default());
    let fp = fingerprint(&target.code, &analysis.visited);
    let out = json!({
        "target": target_json(chain, &target),
        "notes": target.notes,
        "fingerprint": fp,
        // The same trailer as read by evmole (contract analyze's inferred
        // compiler metadata), for cross-reference
        "compiler_metadata": crate::bytecode::compiler_metadata(&target.code).map(|m| {
            let mut v = json!(m);
            sanitise_json(&mut v);
            v
        }),
        "analysis_complete": analysis.coverage.complete,
    });
    if !args.target.format.is_table() {
        return print_json(&args.target.format, &out);
    }
    print_target_table(&out);
    let f = &out["fingerprint"];
    match f["metadata"].as_object() {
        Some(m) => println!(
            "Compiler: {} {} (metadata hash: {}{})",
            m["compiler"].as_str().unwrap_or("?"),
            m.get("version")
                .and_then(Value::as_str)
                .unwrap_or("(version not recorded)"),
            m["hash_type"].as_str().unwrap_or("?"),
            if m.get("experimental").and_then(Value::as_bool) == Some(true) {
                ", experimental"
            } else {
                ""
            }
        ),
        None => println!("Compiler: unknown (no metadata at the end of the code)"),
    }
    let embedded = f["embedded_metadata"].as_array().map(Vec::len).unwrap_or(0);
    if embedded > 0 {
        println!("Embedded metadata blocks (contracts created by this one): {embedded}");
    }
    println!(
        "EVM version: {} ({})",
        f["min_evm_version"].as_str().unwrap_or("?"),
        f["evm_evidence"]
            .as_array()
            .map(|e| e
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join(", "))
            .unwrap_or_default()
    );
    println!(
        "Pipeline (guess): {} - {}",
        f["pipeline_guess"].as_str().unwrap_or("?"),
        f["pipeline_evidence"]
            .as_array()
            .map(|e| e
                .iter()
                .filter_map(Value::as_str)
                .collect::<Vec<_>>()
                .join("; "))
            .unwrap_or_default()
    );
    println!(
        "Address-shaped PUSH32 operands (immutables or address constants, upper bound): {}",
        f["address_like_push32"]
    );
    for n in f["notes"].as_array().into_iter().flatten() {
        println!("Note: {}", n.as_str().unwrap_or(""));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// match
// ---------------------------------------------------------------------------

/// Largest artifact `contract match` reads
pub const MAX_ARTIFACT_BYTES: u64 = 50 * 1024 * 1024;

/// Read an artifact file: a regular file under the size cap, parsed as JSON.
/// Errors never include the file's contents.
fn read_artifact(path: &Path) -> anyhow::Result<Value> {
    let meta = std::fs::metadata(path)
        .map_err(|_| anyhow::anyhow!("artifact not found or not readable"))?;
    if !meta.is_file() {
        anyhow::bail!("artifact is not a regular file");
    }
    if meta.len() > MAX_ARTIFACT_BYTES {
        anyhow::bail!(
            "artifact is larger than {} MB",
            MAX_ARTIFACT_BYTES / 1024 / 1024
        );
    }
    let bytes = std::fs::read(path).map_err(|_| anyhow::anyhow!("artifact could not be read"))?;
    serde_json::from_slice(&bytes).map_err(|_| {
        anyhow::anyhow!(
            "artifact is not valid JSON (expected a forge artifact or solc standard-JSON output)"
        )
    })
}

/// Load all source ASTs from a build-info file: the one given, or a forge
/// build-info next to the artifact (`out/build-info/*.json`) that lists the
/// same contract. Returns the file name used.
fn attach_build_info(
    artifact: &mut crate::bytecode::matcher::Artifact,
    artifact_path: &Path,
    given: Option<&Path>,
    discover: bool,
) -> anyhow::Result<Option<String>> {
    if artifact.format != "forge" && given.is_none() {
        return Ok(None);
    }
    let name = |p: &Path| p.file_name().map(|f| f.to_string_lossy().to_string());
    if let Some(path) = given {
        let json = read_artifact(path)?;
        if !artifact.build_info_matches(&json) {
            anyhow::bail!("--build-info does not contain this artifact's contract");
        }
        artifact.attach_build_info(&json)?;
        return Ok(name(path));
    }
    if !discover {
        return Ok(None);
    }
    // Confine discovery: the build-info directory and every file in it must
    // resolve inside the artifact's own out/ directory (and inside
    // ETHCLI_MCP_ARTIFACT_ROOT when that is set); symlinks are skipped
    let Some(out_dir) = artifact_path
        .canonicalize()
        .ok()
        .and_then(|p| p.parent().and_then(Path::parent).map(Path::to_path_buf))
    else {
        return Ok(None);
    };
    let root = std::env::var_os("ETHCLI_MCP_ARTIFACT_ROOT")
        .filter(|v| !v.is_empty())
        .and_then(|r| PathBuf::from(r).canonicalize().ok());
    let inside =
        |p: &Path| p.starts_with(&out_dir) && root.as_ref().is_none_or(|r| p.starts_with(r));
    let raw_dir = out_dir.join("build-info");
    if std::fs::symlink_metadata(&raw_dir)
        .map(|m| m.file_type().is_symlink())
        .unwrap_or(true)
    {
        return Ok(None);
    }
    let Ok(dir) = raw_dir.canonicalize() else {
        return Ok(None);
    };
    if !inside(&dir) {
        return Ok(None);
    }
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(None);
    };
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .filter_map(|p| p.canonicalize().ok())
        .filter(|p| inside(p))
        .collect();
    candidates.sort();
    // Only a build-info holding this artifact's exact deployed bytecode is
    // used; if several do (and differ), the choice is left to the user
    let mut matches: Vec<(PathBuf, Value)> = Vec::new();
    for path in candidates.into_iter().take(64) {
        let Ok(json) = read_artifact(&path) else {
            continue;
        };
        if artifact.build_info_matches(&json) {
            matches.push((path, json));
        }
    }
    match matches.len() {
        0 => Ok(None),
        1 => {
            let (path, json) = matches.remove(0);
            artifact.attach_build_info(&json)?;
            Ok(name(&path))
        }
        _ => anyhow::bail!(
            "several build-info files hold this artifact's bytecode ({}); pass one with --build-info",
            matches
                .iter()
                .filter_map(|(p, _)| name(p))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

pub async fn match_cmd(chain: Chain, args: &MatchArgs, quiet: bool) -> anyhow::Result<()> {
    let json_artifact = read_artifact(&args.artifact)?;
    let mut artifact = parse_artifact(&json_artifact, args.contract.as_deref())?;
    if artifact.runtime.is_empty() {
        anyhow::bail!("the artifact has no deployed bytecode (an interface or abstract contract?)");
    }
    let build_info_used = attach_build_info(
        &mut artifact,
        &args.artifact,
        args.build_info.as_deref(),
        !args.no_build_info_discovery,
    )?;
    let target = fetch_target(chain, &args.target, quiet).await?;
    let report = match_code(&artifact, &target.code);

    // Selectors: artifact methodIdentifiers vs the deployed code
    let deployed: Vec<String> = extract_selectors(&target.code)
        .into_iter()
        .map(|f| f.selector.trim_start_matches("0x").to_lowercase())
        .collect();
    let missing: Vec<String> = artifact
        .methods
        .iter()
        .filter(|(s, _)| !deployed.contains(s))
        .map(|(s, sig)| format!("0x{s} {sig}"))
        .collect();
    let extra: Vec<String> = deployed
        .iter()
        .filter(|s| !artifact.methods.contains_key(*s))
        .map(|s| format!("0x{s}"))
        .collect();

    let creation = match &args.creation_tx {
        Some(hash) => {
            let hash =
                B256::from_str(hash).map_err(|e| anyhow::anyhow!("invalid --creation-tx: {e}"))?;
            let tx = target
                .endpoint
                .get_transaction(hash)
                .await
                .map_err(|e| anyhow::anyhow!("cannot fetch the creation transaction: {e}"))?
                .ok_or_else(|| anyhow::anyhow!("creation transaction not found"))?;
            use alloy::consensus::Transaction as _;
            if tx.inner.to().is_some() {
                anyhow::bail!(
                    "--creation-tx is a call, not a top-level deployment; contracts created by a factory are not supported"
                );
            }
            let receipt = target
                .endpoint
                .get_transaction_receipt(hash)
                .await
                .map_err(|e| anyhow::anyhow!("cannot fetch the creation receipt: {e}"))?
                .ok_or_else(|| anyhow::anyhow!("creation receipt not found"))?;
            match receipt.contract_address {
                Some(created) if created == target.code_address => {}
                Some(created) => anyhow::bail!(
                    "--creation-tx created {}, not {}",
                    created.to_checksum(None),
                    target.code_address.to_checksum(None)
                ),
                None => anyhow::bail!("--creation-tx created no contract"),
            }
            Some(
                match_creation(&artifact, tx.inner.input())
                    .ok_or_else(|| anyhow::anyhow!("the artifact has no creation bytecode"))?,
            )
        }
        None => None,
    };

    let asm = assembly_share(&artifact);
    let mut guidance = vec![
        "Only an exact match (metadata included) ties the code to these sources; a partial match must be reported as \"matches ignoring metadata\", never as verified".to_string(),
    ];
    match &asm {
        Some(a) if a.blocks > 0 => guidance.push(format!(
            "{} inline assembly block(s), {}% of function source: assembly can copy bytecode without explaining it; a rebuild that matches through assembly proves little",
            a.blocks, a.percent
        )),
        Some(_) => guidance.push("no inline assembly in the sources with an AST".into()),
        None => guidance.push("inline assembly share unknown: the artifact has no AST (build with `forge build --ast`)".into()),
    }

    let out = json!({
        "target": target_json(chain, &target),
        "notes": target.notes,
        "artifact": {
            "path": args.artifact.file_name().map(|f| f.to_string_lossy().to_string()),
            "format": artifact.format,
            "contract": artifact.contract_name,
            "source": artifact.source_path,
            "compiler": artifact.compiler_version,
            "has_ast": !artifact.asts.is_empty(),
            "build_info": build_info_used,
            "has_source_map": artifact.source_map.is_some(),
        },
        "match": report,
        "selectors": {"in_artifact_not_deployed": missing, "deployed_not_in_artifact": extra},
        "creation": creation,
        "inline_assembly": asm,
        "guidance": guidance,
    });
    if !args.target.format.is_table() {
        return print_json(&args.target.format, &out);
    }
    print_match_table(&out);
    Ok(())
}

fn print_match_table(out: &Value) {
    print_target_table(out);
    let a = &out["artifact"];
    println!(
        "Artifact: {} {} ({}, {})",
        a["contract"].as_str().unwrap_or("?"),
        a["source"].as_str().unwrap_or(""),
        a["format"].as_str().unwrap_or(""),
        a["compiler"].as_str().unwrap_or("compiler unknown")
    );
    let m = &out["match"];
    println!();
    println!(
        "VERDICT: {}",
        m["verdict"].as_str().unwrap_or("").to_uppercase()
    );
    println!("{}", m["summary"].as_str().unwrap_or(""));
    println!(
        "Size: artifact {} bytes, deployed {} bytes (executable {} vs {})",
        m["artifact_size"],
        m["onchain_size"],
        m["executable_size_artifact"],
        m["executable_size_onchain"]
    );
    println!("Metadata equal: {}", m["metadata_equal"]);
    for (label, key) in [("Immutable", "immutables"), ("Library", "libraries")] {
        for v in m[key].as_array().into_iter().flatten() {
            println!(
                "{label} {}: {} ({} position(s){})",
                v["id"].as_str().unwrap_or(""),
                v["values"]
                    .as_array()
                    .map(|vs| vs
                        .iter()
                        .filter_map(Value::as_str)
                        .collect::<Vec<_>>()
                        .join(", "))
                    .unwrap_or_default(),
                v["positions"],
                if v["consistent"] == false {
                    ", INCONSISTENT"
                } else {
                    ""
                }
            );
        }
    }
    if let Some(mm) = m["mismatch"].as_object() {
        println!();
        println!(
            "First difference at {:#x}, in {}",
            mm["first_difference_offset"].as_u64().unwrap_or(0),
            mm["function"].as_str().unwrap_or("?")
        );
        println!(
            "  artifact: {}",
            mm["artifact_bytes"].as_str().unwrap_or("")
        );
        println!("  deployed: {}", mm["onchain_bytes"].as_str().unwrap_or(""));
        println!(
            "  (artifact from {:#x}, deployed around {:#x}); size difference {} bytes",
            mm["context_start"].as_u64().unwrap_or(0),
            mm["onchain_offset"].as_u64().unwrap_or(0),
            mm["size_difference"]
        );
        if let (Some(a), Some(d)) = (
            mm.get("artifact_text").and_then(Value::as_str),
            mm.get("onchain_text").and_then(Value::as_str),
        ) {
            println!("  text: artifact {a:?} vs deployed {d:?}");
        }
        if let Some(h) = mm.get("hint").and_then(Value::as_str) {
            println!("  hint: {h}");
        }
        println!(
            "Functions ({}; mapping: {}):",
            mm["table_method"].as_str().unwrap_or(""),
            m["function_mapping"].as_str().unwrap_or("")
        );
        for f in mm["functions"].as_array().into_iter().flatten() {
            println!(
                "  {:<10} {:>6} instr  {}",
                f["status"].as_str().unwrap_or(""),
                f["instructions"],
                f["function"].as_str().unwrap_or("")
            );
        }
    }
    let s = &out["selectors"];
    for (label, key) in [
        ("In artifact, not deployed", "in_artifact_not_deployed"),
        ("Deployed, not in artifact", "deployed_not_in_artifact"),
    ] {
        let list = s[key].as_array().cloned().unwrap_or_default();
        if !list.is_empty() {
            println!(
                "{label}: {}",
                list.iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
    }
    if let Some(c) = out["creation"].as_object() {
        println!(
            "Creation code: {} ; constructor args {}",
            c["verdict"].as_str().unwrap_or(""),
            c["constructor_args"].as_str().unwrap_or("")
        );
    }
    for w in m["warnings"].as_array().into_iter().flatten() {
        println!("WARNING: {}", w.as_str().unwrap_or(""));
    }
    for g in out["guidance"].as_array().into_iter().flatten() {
        println!("Note: {}", g.as_str().unwrap_or(""));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::StateMutability;

    fn census_with(name: &str, at: Vec<usize>) -> OpcodeCensus {
        let mut c = OpcodeCensus {
            method: "test",
            analysis_complete: true,
            ..Default::default()
        };
        c.positions.insert(name.into(), at);
        c
    }

    #[test]
    fn warns_on_dropped_origin_and_partial_coverage() {
        let census = census_with("ORIGIN", vec![0x3430, 0x350e, 0x359b]);
        let dec = json!({
            "source": "fallback() { require(msg.sender == x); }",
            "stats": {"covered_blocks": 120, "code_blocks": 163, "unresolved_jumps": 0},
            "functions": [], "storage": [],
        });
        let cc = cross_check(&[], &census, &dec);
        let w: Vec<String> = cc["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert!(w
            .iter()
            .any(|s| s.starts_with("3 ORIGIN sites at pc 0x3430, 0x350e, 0x359b")));
        assert!(w.iter().any(|s| s.contains("120 of 163")));
    }

    #[test]
    fn types_side_by_side_and_absence_wording() {
        let f = ExtractedFunction {
            selector: "0xa9059cbb".into(),
            arguments: vec!["address".into(), "uint256".into()],
            state_mutability: StateMutability::NonPayable,
            signature: None,
        };
        let dec = json!({
            "source": "",
            "functions": [{"selector": "0xa9059cbb", "name": "transfer", "params": [{"ty": "uint256"}, {"ty": "uint256"}], "effects": {"writes_storage": true}}],
            "storage": [{"slot": "0x5", "name": "tickBitmap", "writes": 0}],
            "stats": {},
        });
        let census = census_with("SSTORE", vec![10]);
        let cc = cross_check(&[f], &census, &dec);
        assert_eq!(cc["type_comparison"][0]["agree"], false);
        assert_eq!(cc["type_comparison"][0]["decompiler"], "(uint256,uint256)");
        assert_eq!(
            cc["not_found_by_decompiler"][0],
            "writes to slot 0x5 (tickBitmap): not found by evmdec"
        );
        // writes_storage is true, so no SSTORE warning
        assert!(!cc["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("SSTORE")));
    }

    #[cfg(unix)]
    fn script(dir: &Path, body: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join("fake-evmdec");
        std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
        p
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn sandbox_clears_env_and_kills_on_timeout() {
        let dir = tempfile::tempdir().unwrap();
        let env_dump = script(dir.path(), "env");
        let out = run_sandboxed(&env_dump, &[], Duration::from_secs(10))
            .await
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        let names: Vec<&str> = text
            .lines()
            .filter_map(|l| l.split('=').next())
            .filter(|n| !n.is_empty())
            .collect();
        // sh itself may add PWD, SHLVL or _; nothing from the parent may leak
        assert!(
            names
                .iter()
                .all(|n| matches!(*n, "PATH" | "PWD" | "SHLVL" | "_" | "OLDPWD")),
            "{names:?}"
        );

        let sleeper = script(dir.path(), "sleep 30");
        let started = std::time::Instant::now();
        let err = run_sandboxed(&sleeper, &[], Duration::from_millis(300))
            .await
            .unwrap_err();
        assert!(err.to_string().contains("killed"));
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn dedaub_link_shape() {
        let a = Address::from_str("0x6b75d8AF000000e20B7a7DDf000Ba900b4009A80").unwrap();
        assert_eq!(
            dedaub_link(Chain::Ethereum, a),
            "https://app.dedaub.com/ethereum/address/0x6b75d8af000000e20b7a7ddf000ba900b4009a80/decompiled"
        );
    }

    #[test]
    fn read_artifact_refuses_dirs_and_hides_contents() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_artifact(dir.path())
            .unwrap_err()
            .to_string()
            .contains("not a regular file"));
        let p = dir.path().join("a.json");
        std::fs::write(&p, "top secret, not json").unwrap();
        let err = read_artifact(&p).unwrap_err().to_string();
        assert!(!err.contains("secret"), "{err}");
        assert!(read_artifact(&dir.path().join("missing.json")).is_err());
    }

    #[test]
    fn block_values_parse_or_error() {
        use alloy::eips::BlockNumberOrTag as T;
        assert_eq!(parse_block("123").unwrap(), T::Number(123));
        assert_eq!(parse_block("0x10").unwrap(), T::Number(16));
        assert_eq!(parse_block("earliest").unwrap(), T::Earliest);
        assert_eq!(parse_block("finalized").unwrap(), T::Finalized);
        assert!(parse_block("foo").is_err());
    }

    #[test]
    fn counts_sites_not_token_presence() {
        let census = census_with("CREATE", vec![1, 2]);
        // `createPair` must not count, and one create( is fewer than two sites
        let dec = json!({"source": "createPair(a, b); x = create(0, 0, 0);", "functions": [{"effects": {"creates": true}}], "storage": [], "stats": {}});
        let cc = cross_check(&[], &census, &dec);
        assert!(cc["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().starts_with("2 CREATE sites")));
        let census = census_with("ORIGIN", vec![1, 2, 3]);
        let dec = json!({"source": "require(tx.origin == a);", "functions": [], "storage": [], "stats": {}});
        let cc = cross_check(&[], &census, &dec);
        assert!(cc["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|w| w.as_str().unwrap().contains("shows 1")));
    }

    #[test]
    fn heimdall_output_is_cross_checked() {
        // Canned heimdall output: abi.json and decompiled.sol with --skip-resolving
        let abi = json!([
            {"type": "function", "name": "Unresolved_a9059cbb", "inputs": [{"name": "arg0", "type": "address"}, {"name": "arg1", "type": "uint256"}], "outputs": [], "stateMutability": "nonpayable"},
            {"type": "event", "name": "Event_1", "inputs": []}
        ]);
        let source = "function Unresolved_a9059cbb(address arg0, uint256 arg1) public {\n    require(tx.origin == address(0xae2f));\n    require(msg.sender == address(0x1));\n}\n".to_string();
        let dec = heimdall_as_decompiler_json(source, &abi);
        assert_eq!(dec["functions"][0]["selector"], "0xa9059cbb");
        let f = ExtractedFunction {
            selector: "0xa9059cbb".into(),
            arguments: vec!["address".into(), "uint256".into()],
            state_mutability: StateMutability::NonPayable,
            signature: None,
        };
        let mut census = census_with("ORIGIN", vec![1, 2, 3]);
        census.positions.insert("SSTORE".into(), vec![9]);
        let cc = cross_check_for("heimdall", &[f], &census, &dec);
        let w: Vec<String> = cc["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        // 3 ORIGIN sites, 1 tx.origin in the text
        assert!(
            w.iter().any(|s| s.contains("heimdall's text shows 1")),
            "{w:?}"
        );
        // No effects in heimdall output: no SSTORE warning from missing effects
        assert!(!w.iter().any(|s| s.contains("SSTORE")), "{w:?}");
        assert_eq!(cc["type_comparison"][0]["agree"], true);
    }

    #[cfg(unix)]
    #[test]
    fn build_info_discovery_skips_symlinks() {
        use crate::bytecode::matcher::parse_artifact;
        let outside = tempfile::tempdir().unwrap();
        let proj = tempfile::tempdir().unwrap();
        let out = proj.path().join("out");
        std::fs::create_dir_all(out.join("A.sol")).unwrap();
        let artifact = serde_json::json!({
            "deployedBytecode": {"object": "0x6000"},
            "bytecode": {"object": "0x6000"},
            "metadata": {"settings": {"compilationTarget": {"src/A.sol": "A"}}},
        });
        let path = out.join("A.sol/A.json");
        std::fs::write(&path, artifact.to_string()).unwrap();
        let info = serde_json::json!({"output": {"contracts": {"src/A.sol": {"A": {"evm": {"deployedBytecode": {"object": "6000"}}}}}, "sources": {"src/A.sol": {"id": 0, "ast": {"nodeType": "SourceUnit"}}}}});
        std::fs::write(outside.path().join("x.json"), info.to_string()).unwrap();
        // build-info dir that is a symlink out of the project
        std::os::unix::fs::symlink(outside.path(), out.join("build-info")).unwrap();
        let mut art = parse_artifact(&artifact, None).unwrap();
        assert_eq!(
            attach_build_info(&mut art, &path, None, true).unwrap(),
            None
        );
        // a real dir holding a symlinked file is skipped too
        std::fs::remove_file(out.join("build-info")).unwrap();
        std::fs::create_dir(out.join("build-info")).unwrap();
        std::os::unix::fs::symlink(outside.path().join("x.json"), out.join("build-info/x.json"))
            .unwrap();
        assert_eq!(
            attach_build_info(&mut art, &path, None, true).unwrap(),
            None
        );
        // a regular file inside is used, unless discovery is off
        std::fs::remove_file(out.join("build-info/x.json")).unwrap();
        std::fs::write(out.join("build-info/y.json"), info.to_string()).unwrap();
        assert_eq!(
            attach_build_info(&mut art, &path, None, false).unwrap(),
            None
        );
        assert_eq!(
            attach_build_info(&mut art, &path, None, true)
                .unwrap()
                .as_deref(),
            Some("y.json")
        );
    }

    #[test]
    fn decompiler_text_is_sanitised() {
        let source = "require(x, \"ok\u{1b}[31mFAKE\u{202e}gnp.exe\");\n\tdone".to_string();
        let mut entry = json!({"heuristic": {"source": source}, "error": "bad\u{1b}]0;title\u{7}\nWARNING: fake"});
        sanitise_json(&mut entry);
        let text = entry.to_string();
        let src = entry["heuristic"]["source"].as_str().unwrap();
        assert!(
            !src.contains('\u{1b}') && !src.contains('\u{202e}'),
            "{text}"
        );
        assert!(src.contains("\\u{1b}") && src.contains("\\u{202e}"));
        // newline and tab survive
        assert!(src.contains("\n\tdone"));
        assert!(!entry["error"].as_str().unwrap().contains('\u{7}'));
    }

    #[test]
    fn injected_lines_stay_inside_their_string() {
        let source = "require(x > 1, \"a\n== CROSS-CHECK (ethcli facts vs evmdec) ==\nNo disagreement found\");\nreturn;".to_string();
        let mut entry = json!({"heuristic": {"source": source}, "error": "e\nWARNING: fake"});
        sanitise_json(&mut entry);
        let src = entry["heuristic"]["source"].as_str().unwrap();
        // Only the structural newline survives
        assert_eq!(src.lines().count(), 2, "{src}");
        assert!(!entry["error"].as_str().unwrap().contains('\n'));
    }

    #[test]
    fn comments_and_strings_do_not_count_as_checks() {
        let src = "// tx.origin is checked\nrequire(a, \"tx.origin\");\n/* tx.origin */ x = 1;";
        assert_eq!(count_word(src, "tx.origin", false), 0);
        assert_eq!(
            count_word("require(tx.origin == a);", "tx.origin", false),
            1
        );
    }

    #[test]
    fn safe_1_0_0_proxy_is_known_by_code() {
        assert!(is_known_safe_proxy(
            &hex::decode(SAFE_PROXY_RUNTIMES[0]).unwrap()
        ));
    }

    const SLOT: &str = "360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";

    #[test]
    fn proxy_needs_a_delegatecall_target_from_the_slot() {
        let imp = Address::repeat_byte(0x11);
        // retLen retOff argsLen argsOff | addr = SLOAD(slot) | GAS DELEGATECALL STOP
        let real = hex::decode(format!("600060003660007f{SLOT}545af400")).unwrap();
        assert!(acts_as_proxy("EIP-1967 proxy", &real, imp));
        // The same, delegating to calldata: not a proxy
        let to_calldata = hex::decode("6000600036600060003560601c5af400").unwrap();
        assert!(!acts_as_proxy("EIP-1967 proxy", &to_calldata, imp));
        // Reads the slot (then drops it) and delegates to calldata
        let reads_slot =
            hex::decode(format!("7f{SLOT}54506000600036600060003560601c5af400")).unwrap();
        assert!(!acts_as_proxy("EIP-1967 proxy", &reads_slot, imp));
        // No DELEGATECALL at all
        assert!(!acts_as_proxy(
            "EIP-1967 proxy",
            &hex::decode("333214600057").unwrap(),
            imp
        ));
    }

    #[test]
    fn eip1167_needs_the_same_constant() {
        let clone = hex::decode("363d3d373d3d3d363d73bebebebebebebebebebebebebebebebebebebebe5af43d82803e903d91602b57fd5bf3").unwrap();
        assert!(acts_as_proxy(
            "EIP-1167 minimal proxy",
            &clone,
            Address::repeat_byte(0xbe)
        ));
        assert!(!acts_as_proxy(
            "EIP-1167 minimal proxy",
            &clone,
            Address::repeat_byte(0x11)
        ));
    }

    #[test]
    fn safe_proxies_are_recognised_by_exact_code() {
        for runtime in SAFE_PROXY_RUNTIMES {
            assert!(acts_as_proxy(
                "Safe proxy",
                &hex::decode(runtime).unwrap(),
                Address::ZERO
            ));
        }
        assert!(!acts_as_proxy(
            "Safe proxy",
            &hex::decode("6000600060006000355af400").unwrap(),
            Address::ZERO
        ));
    }

    #[test]
    fn reached_sites_in_a_trailer_count_as_code() {
        use crate::bytecode::calls::{Coverage, OpcodeSite};
        let site = |pc, reached, in_metadata| OpcodeSite {
            opcode: "SELFDESTRUCT".into(),
            pc,
            reached,
            in_metadata,
        };
        let report = CallsReport {
            source: "inferred",
            bytecode_size: 0,
            call_sites: vec![],
            functions: vec![],
            opcode_sites: vec![
                site(10, true, true),
                site(20, false, true),
                site(30, false, false),
            ],
            jump_tables: vec![],
            coverage: Coverage {
                complete: true,
                ..Default::default()
            },
            notes: vec![],
            visited: vec![],
        };
        let c = census_from(&report);
        assert_eq!(c.positions["SELFDESTRUCT"], vec![10]);
        assert_eq!(c.in_metadata["SELFDESTRUCT"], vec![20]);
        assert_eq!(c.not_reached["SELFDESTRUCT"], vec![30]);
        assert!(c
            .warnings
            .iter()
            .any(|w| w.contains("reached code inside metadata-looking bytes")));
    }

    fn prov(
        kind: &str,
        value: Option<&str>,
        slot: Option<&str>,
        call_pc: Option<usize>,
    ) -> crate::bytecode::Provenance {
        crate::bytecode::Provenance {
            kind: kind.into(),
            value: value.map(String::from),
            slot: slot.map(String::from),
            offset: None,
            call_pc,
            note: None,
        }
    }

    fn site(
        pc: usize,
        kind: &str,
        function: &str,
        target: Vec<crate::bytecode::Provenance>,
    ) -> crate::bytecode::CallSite {
        crate::bytecode::CallSite {
            pc,
            kind: kind.into(),
            functions: vec![function.into()],
            target,
            selector: vec![],
            value: vec![],
            args: vec![],
            args_len: vec![],
            resolved: true,
        }
    }

    fn report_with(sites: Vec<crate::bytecode::CallSite>) -> CallsReport {
        CallsReport {
            source: "inferred",
            bytecode_size: 0,
            call_sites: sites,
            functions: vec![],
            opcode_sites: vec![],
            jump_tables: vec![],
            coverage: crate::bytecode::calls::Coverage {
                complete: true,
                ..Default::default()
            },
            notes: vec![],
            visited: vec![],
        }
    }

    #[test]
    fn immutable_beacon_is_followed() {
        let beacon = Address::repeat_byte(0xbe);
        let imp = Address::repeat_byte(0x11);
        let beacon_call = |kind: &str, selector: &str, len: &str, value: Option<&str>| {
            let mut c = site(
                10,
                kind,
                "dispatcher",
                vec![prov(
                    "code_constant",
                    Some(&format!("{beacon:#x}")),
                    None,
                    None,
                )],
            );
            c.selector = vec![prov("constant", Some(selector), None, None)];
            c.args_len = vec![prov("constant", Some(len), None, None)];
            c.value = value
                .map(|v| vec![prov("constant", Some(v), None, None)])
                .unwrap_or_default();
            c
        };
        let delegate = || {
            site(
                20,
                "DELEGATECALL",
                "dispatcher",
                vec![prov("call_return", None, None, Some(10))],
            )
        };
        let ok = |c, b| {
            proxy_check_report(
                "EIP-1967 beacon proxy",
                &report_with(vec![c, delegate()]),
                imp,
                Some(b),
            )
            .ok
        };
        assert!(ok(
            beacon_call("STATICCALL", "0x5c60da1b", "0x4", None),
            beacon
        ));
        assert!(ok(
            beacon_call("CALL", "0x5c60da1b", "0x4", Some("0x0")),
            beacon
        ));
        // Wrong beacon, another selector (getTarget()), extra arguments, a value
        assert!(!ok(
            beacon_call("STATICCALL", "0x5c60da1b", "0x4", None),
            Address::repeat_byte(0x22)
        ));
        assert!(!ok(
            beacon_call("STATICCALL", "0xf00e6a2a", "0x4", None),
            beacon
        ));
        assert!(!ok(
            beacon_call("STATICCALL", "0x5c60da1b", "0x24", None),
            beacon
        ));
        assert!(!ok(
            beacon_call("CALL", "0x5c60da1b", "0x4", Some("0x1")),
            beacon
        ));
    }

    #[test]
    fn slot_delegatecall_under_one_selector_is_not_a_proxy() {
        let slot = format!("0x{SLOT}");
        let report = report_with(vec![site(
            20,
            "DELEGATECALL",
            "0xa3ecc898",
            vec![prov("storage", None, Some(&slot), None)],
        )]);
        let check = proxy_check_report("EIP-1967 proxy", &report, Address::ZERO, None);
        assert!(!check.ok);
        assert_eq!(check.only_under, vec!["0xa3ecc898".to_string()]);
        let report = report_with(vec![site(
            20,
            "DELEGATECALL",
            "dispatcher",
            vec![prov("storage", None, Some(&slot), None)],
        )]);
        assert!(proxy_check_report("EIP-1967 proxy", &report, Address::ZERO, None).ok);
    }

    #[test]
    fn vyper_forwarders_are_recognised() {
        let curve = hex::decode("3660006000376110006000366000735f890841f657d90e081babdb532a05996af79fe65af4602c57600080fd5b6110006000f3").unwrap();
        assert_eq!(
            vyper_forwarder(&curve),
            Some(Address::from_str("0x5f890841f657d90e081babdb532a05996af79fe6").unwrap())
        );
        assert!(vyper_forwarder(&hex::decode("6000").unwrap()).is_none());
    }

    #[test]
    fn unreached_origin_is_named_when_not_shown() {
        let mut census = census_with("SSTORE", vec![1]);
        census.not_reached.insert("ORIGIN".into(), vec![0xc8]);
        let dec = json!({"source": "x = 1;", "functions": [], "storage": [], "stats": {}});
        let cc = cross_check_for("heimdall", &[], &census, &dec);
        assert!(cc["warnings"].as_array().unwrap().iter().any(|w| w
            .as_str()
            .unwrap()
            .starts_with("ORIGIN at pc 0xc8 not reached")));
    }
}
