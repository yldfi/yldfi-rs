//! `contract decompile`, `contract fingerprint` and `contract match`: tools
//! for contracts without verified source.
//!
//! `decompile` runs the optional external decompiler evmdec on bytecode that
//! ethcli fetched itself, and puts a cross-check against ethcli's own facts
//! before evmdec's heuristic output. `fingerprint` reads what the runtime code
//! says about its compiler. `match` compares a compiled artifact with the
//! deployed code.

use super::OutputFormat;
use crate::bytecode::matcher::{assembly_share, match_code, match_creation, parse_artifact};
use crate::bytecode::metadata::{fingerprint, sensitive_census, OpcodeCensus};
use crate::bytecode::{extract_selectors, ExtractedFunction};
use crate::config::{Chain, EndpointConfig};
use crate::rpc::Endpoint;
use alloy::primitives::{Address, B256};
use alloy::providers::Provider;
use clap::Args;
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

    /// Decompiler backend. Only evmdec is supported
    #[arg(long, default_value = "evmdec", value_parser = ["evmdec"])]
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

/// Does this code delegate (a reachable DELEGATECALL)?
/// Does this code act as the proxy `kind` says? A slot alone proves
/// nothing (any contract can write to it), and a DELEGATECALL alone proves
/// nothing either (a contract can delegate to an argument). The code must
/// delegate and read the slot it names: the slot constant appears as a
/// PUSH32. EIP-1167 clones are recognised from their exact code; Safe
/// proxies are small and read slot 0
fn acts_as_proxy(kind: &str, code: &[u8]) -> bool {
    const EIP1967_IMPL: &str = "360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc";
    const EIP1967_BEACON: &str = "a3f0ad74e5423aebfd80d3ef4346578335a9a72aeaee59ff6cb3582b35133d50";
    const OZ_LEGACY: &str = "7050c9e0f4ca769c69bd3a8ef740bc37934f8e2c036e5a723fd8ee048ed3f8c3";
    if kind.starts_with("EIP-1167") {
        return true;
    }
    if !code_delegates(code) {
        return false;
    }
    let pushes = |slot: &str| {
        let Ok(slot) = hex::decode(slot) else {
            return false;
        };
        crate::bytecode::metadata::linear_sweep(code)
            .iter()
            .any(|(pc, op)| *op == 0x7f && code.get(pc + 1..pc + 33) == Some(slot.as_slice()))
    };
    match kind {
        "EIP-1967 proxy" => pushes(EIP1967_IMPL),
        "EIP-1967 beacon proxy" => pushes(EIP1967_BEACON),
        "OpenZeppelin legacy proxy" => pushes(OZ_LEGACY),
        "Safe proxy" => code.len() <= 1024,
        _ => false,
    }
}

fn code_delegates(code: &[u8]) -> bool {
    let census = sensitive_census(code);
    census.positions.contains_key("DELEGATECALL")
        || census.inside_metadata.contains_key("DELEGATECALL")
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
        if let Some((kind, implementation)) =
            super::contract::find_implementation(provider, address, &code, block_id).await
        {
            if kind.starts_with("implementation() getter") {
                notes.push(format!(
                    "answers implementation() with {}, but that alone does not make it a proxy; using its own code",
                    implementation.to_checksum(None)
                ));
            } else if !acts_as_proxy(kind, &code) {
                let warning = format!(
                    "{kind} slot points to {}, but this address's own code does not delegate through that slot (no reachable DELEGATECALL with the slot read), so it does not run that code; analysing its own code",
                    implementation.to_checksum(None)
                );
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

/// Find the evmdec binary: EVMDEC_PATH, then PATH
fn find_evmdec() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("EVMDEC_PATH").filter(|p| !p.is_empty()) {
        let p = PathBuf::from(p);
        return p.is_file().then_some(p);
    }
    let path = std::env::var_os("PATH")?;
    std::env::split_paths(&path)
        .map(|dir| {
            dir.join(if cfg!(windows) {
                "evmdec.exe"
            } else {
                "evmdec"
            })
        })
        .find(|p| p.is_file())
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
    let mut cmd = tokio::process::Command::new(bin);
    cmd.args(args)
        .env_clear()
        .env("PATH", MINIMAL_PATH)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);
    if let Some(dir) = std::env::temp_dir().to_str() {
        cmd.current_dir(dir);
    }
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

/// Occurrences of `token` as a whole word; with `call`, only when followed
/// by `(` or `{` (so `createPair` or `create2` never count as `create`)
fn count_word(source: &str, token: &str, call: bool) -> usize {
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
    let source = evmdec["source"].as_str().unwrap_or("");
    let mut warnings: Vec<String> = Vec::new();

    let stats = &evmdec["stats"];
    if let (Some(covered), Some(total)) = (
        stats["covered_blocks"].as_u64(),
        stats["code_blocks"].as_u64(),
    ) {
        if covered < total {
            warnings.push(format!(
                "evmdec covered {covered} of {total} code blocks; code in the other {} is not in the output",
                total - covered
            ));
        }
    }
    if stats["unresolved_jumps"].as_u64().unwrap_or(0) > 0 {
        warnings.push(format!(
            "evmdec left {} jumps unresolved; control flow there is missing",
            stats["unresolved_jumps"]
        ));
    }
    if stats["budget_exhausted"].as_bool() == Some(true) {
        warnings.push("evmdec ran out of its work budget; output is partial".into());
    }

    warnings.extend(census.warnings.iter().cloned());
    // Executable "metadata" counts as code for the comparison
    let mut all = census.positions.clone();
    for (k, v) in &census.inside_metadata {
        all.entry(k.clone()).or_default().extend(v);
    }
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
        let effect_missing = effect.is_some_and(|e| !effect_set(e));
        if shown < at.len() || effect_missing {
            warnings.push(format!(
                "{} {name} site{} at pc {}, but evmdec's text shows {} ({}){}; {what}",
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
    if let Some(sstores) = all.get("SSTORE") {
        let writes = fns
            .iter()
            .any(|f| f["effects"]["writes_storage"].as_bool() == Some(true));
        if !writes {
            warnings.push(format!(
                "{} SSTORE at pc {} but evmdec reports no function writing storage; state changes are missing",
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
                    "evmdec": format!("({})", dec.join(",")),
                    "evmdec_name": d["name"],
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
            "{disagreements} function(s) where evmole and evmdec disagree on argument types; see type_comparison"
        ));
    }

    // Absence claims, reported as what evmdec did not find
    let mut not_found: Vec<String> = Vec::new();
    for s in evmdec["storage"].as_array().into_iter().flatten() {
        if s["writes"].as_u64() == Some(0) {
            not_found.push(format!(
                "writes to slot {} ({}): not found by evmdec",
                s["slot"].as_str().unwrap_or("?"),
                s["name"].as_str().unwrap_or("?")
            ));
        }
    }

    json!({
        "warnings": warnings,
        "type_comparison": types,
        "selectors_only_in_ethcli": only_ethcli,
        "selectors_only_in_evmdec": only_evmdec,
        "not_found_by_evmdec": not_found,
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
    for (key, label) in [
        ("positions", "sensitive opcodes (EVM instruction boundaries, reachable by fallthrough)"),
        ("unreachable_by_fallthrough", "after a halt, before the next valid JUMPDEST (usually data; reachable only if something jumps there)"),
        ("inside_metadata", "inside executable metadata-looking bytes (treat as real code)"),
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

    let Some(bin) = find_evmdec() else {
        out["evmdec"] = json!({"installed": false, "install": EVMDEC_INSTALL});
        if args.target.format.is_table() {
            print_target_table(&out);
            println!();
            println!("{EVMDEC_INSTALL}");
            println!();
            print_facts_table(&out["facts"]);
            println!();
            println!(
                "Dedaub (sign-in needed): {}",
                out["dedaub"]["url"].as_str().unwrap_or("")
            );
        } else {
            print_json(&args.target.format, &out)?;
        }
        return Ok(());
    };

    let version = run_sandboxed(&bin, &["--version".as_ref()], Duration::from_secs(5))
        .await
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    out["evmdec"] = json!({
        "installed": true,
        "path": bin.display().to_string(),
        "version": version,
        "binary_sha256": sha256_file(&bin),
        "commit": null,
        "commit_note": "evmdec does not report its commit; the sha256 identifies the binary",
    });

    let file = HexFile::create(&target.code)?;
    if !quiet {
        eprintln!("Running evmdec (timeout {} s)...", args.timeout);
    }
    let run = run_sandboxed(
        &bin,
        &["decompile".as_ref(), file.0.as_os_str(), "--json".as_ref()],
        Duration::from_secs(args.timeout),
    )
    .await;
    drop(file);

    let parsed: Result<Value, String> = match run {
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
    };

    match parsed {
        Err(e) => {
            out["evmdec"]["error"] = json!(e);
            if args.target.format.is_table() {
                print_target_table(&out);
                println!("evmdec failed: {e}");
                println!();
                print_facts_table(&out["facts"]);
                println!();
                println!(
                    "Dedaub (sign-in needed): {}",
                    out["dedaub"]["url"].as_str().unwrap_or("")
                );
            } else {
                print_json(&args.target.format, &out)?;
            }
        }
        Ok(dec) => {
            out["cross_check"] = cross_check(&functions, &census, &dec);
            out["heuristic"] = json!({
                "label": "HEURISTIC: evmdec's reconstruction, not source. Names come from its built-in signature dictionary; types, storage names and control flow are guesses",
                "compiler": dec["compiler"],
                "stats": dec["stats"],
                "functions": dec["functions"].as_array().map(|fs| fs.iter().map(|f| json!({
                    "selector": f["selector"], "name": f["name"], "params": f["params"],
                    "returns": f["returns"], "mutability": f["mutability"], "effects": f["effects"],
                })).collect::<Vec<_>>()),
                "storage": dec["storage"],
                "call_targets": dec["call_targets"],
                "source": dec["source"],
            });
            if args.target.format.is_table() {
                print_decompile_table(&out);
            } else {
                print_json(&args.target.format, &out)?;
            }
        }
    }
    Ok(())
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
    let e = &out["evmdec"];
    println!(
        "evmdec: {} (sha256 {})",
        e["version"].as_str().unwrap_or("version unknown"),
        e["binary_sha256"]
            .as_str()
            .map(|s| &s[..16.min(s.len())])
            .unwrap_or("?")
    );
    println!();
    println!("== CROSS-CHECK (ethcli facts vs evmdec) ==");
    let cc = &out["cross_check"];
    let warnings = cc["warnings"].as_array().cloned().unwrap_or_default();
    if warnings.is_empty() {
        println!("No disagreement found between ethcli's facts and evmdec's output.");
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
        println!("{:<12} {:<40} evmdec", "selector", "evmole");
        for t in disagree {
            println!(
                "{:<12} {:<40} {}",
                t["selector"].as_str().unwrap_or(""),
                t["evmole"].as_str().unwrap_or(""),
                t["evmdec"].as_str().unwrap_or("")
            );
        }
    }
    for key in ["selectors_only_in_ethcli", "selectors_only_in_evmdec"] {
        let list = cc[key].as_array().cloned().unwrap_or_default();
        if !list.is_empty() {
            let items: Vec<&str> = list.iter().filter_map(Value::as_str).collect();
            println!("{key}: {}", items.join(", "));
        }
    }
    for n in cc["not_found_by_evmdec"].as_array().into_iter().flatten() {
        println!(
            "Not found by evmdec (not a fact): {}",
            n.as_str().unwrap_or("")
        );
    }
    println!();
    print_facts_table(&out["facts"]);
    println!();
    println!("== HEURISTIC PSEUDO-SOLIDITY (evmdec; not source, may be wrong or incomplete) ==");
    print!("{}", out["heuristic"]["source"].as_str().unwrap_or(""));
    println!();
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
    let fp = fingerprint(&target.code);
    let out = json!({
        "target": target_json(chain, &target),
        "notes": target.notes,
        "fingerprint": fp,
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
    let Some(dir) = artifact_path
        .parent()
        .and_then(Path::parent)
        .map(|out| out.join("build-info"))
    else {
        return Ok(None);
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(None);
    };
    let mut candidates: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("json"))
        .collect();
    candidates.sort();
    for path in candidates.into_iter().take(64) {
        let Ok(json) = read_artifact(&path) else {
            continue;
        };
        if artifact.build_info_matches(&json) && artifact.attach_build_info(&json).is_ok() {
            return Ok(name(&path));
        }
    }
    Ok(None)
}

pub async fn match_cmd(chain: Chain, args: &MatchArgs, quiet: bool) -> anyhow::Result<()> {
    let json_artifact = read_artifact(&args.artifact)?;
    let mut artifact = parse_artifact(&json_artifact, args.contract.as_deref())?;
    if artifact.runtime.is_empty() {
        anyhow::bail!("the artifact has no deployed bytecode (an interface or abstract contract?)");
    }
    let build_info_used =
        attach_build_info(&mut artifact, &args.artifact, args.build_info.as_deref())?;
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
            method: "linear sweep",
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
        assert_eq!(cc["type_comparison"][0]["evmdec"], "(uint256,uint256)");
        assert_eq!(
            cc["not_found_by_evmdec"][0],
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
    fn delegatecall_without_the_slot_is_not_a_proxy() {
        // DELEGATECALL present but no EIP-1967 slot constant
        let code = hex::decode("6000600060006000355af400").unwrap();
        assert!(code_delegates(&code));
        assert!(!acts_as_proxy("EIP-1967 proxy", &code));
        // PUSH32 <slot> SLOAD ... DELEGATECALL
        let code = hex::decode("7f360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc546000600060006000855af400").unwrap();
        assert!(acts_as_proxy("EIP-1967 proxy", &code));
    }

    #[test]
    fn slot_alone_does_not_make_a_proxy() {
        // CALLER ORIGIN EQ ... no DELEGATECALL: not a proxy, whatever the slot says
        assert!(!code_delegates(&hex::decode("333214600057").unwrap()));
        // EIP-1167-like body with a DELEGATECALL
        assert!(code_delegates(&hex::decode("363d3d373d3d3d363d73bebebebebebebebebebebebebebebebebebebebe5af43d82803e903d91602b57fd5bf3").unwrap()));
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
}
