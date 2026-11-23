//! `rpc find-change`: the block where a value changed, by binary search
//!
//! The value is read at `--from`, then the search narrows `(from, to]` to the
//! first block whose value differs. That costs about log2(to - from) reads.
//! It assumes the value changed once in the range; with several changes it
//! finds one boundary (not always the first). `--all` repeats the search
//! after each boundary, which finds every change only when no value returns
//! to an earlier one inside a searched segment.

use crate::config::{Chain, EndpointConfig};
use crate::rpc::Endpoint;
use alloy::eips::{BlockId, BlockNumberOrTag};
use alloy::primitives::{Address, U256};
use alloy::providers::Provider;
use std::future::Future;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};

/// Most boundaries `--all` will report
const MAX_CHANGES: usize = 100;

/// What is read at each block
pub(super) enum Target {
    Call(super::rpc::CallSpec),
    Balance(Address),
    Storage(Address, U256),
    Code(Address),
    Nonce(Address),
}

impl Target {
    /// Build the target from the command's arguments (exactly one is set)
    pub fn from_args(
        call: Option<&str>,
        balance: Option<&str>,
        storage: &[String],
        code: Option<&str>,
        nonce: Option<&str>,
    ) -> anyhow::Result<Self> {
        let addr = |s: &str| {
            Address::from_str(s).map_err(|e| anyhow::anyhow!("Invalid address '{s}': {e}"))
        };
        let given = [
            call.is_some(),
            balance.is_some(),
            !storage.is_empty(),
            code.is_some(),
            nonce.is_some(),
        ]
        .iter()
        .filter(|b| **b)
        .count();
        if given != 1 {
            anyhow::bail!(
                "Give exactly one target: a call \"<address> <signature> [args...]\", --balance, --storage, --code or --nonce"
            );
        }
        Ok(if let Some(c) = call {
            Target::Call(super::rpc::parse_call_spec(c)?)
        } else if let Some(a) = balance {
            Target::Balance(addr(a)?)
        } else if let [a, slot] = storage {
            let slot = if let Some(h) = slot.strip_prefix("0x") {
                U256::from_str_radix(h, 16)
            } else {
                U256::from_str_radix(slot, 10)
            }
            .map_err(|e| anyhow::anyhow!("Invalid slot '{slot}': {e}"))?;
            Target::Storage(addr(a)?, slot)
        } else if let Some(a) = code {
            Target::Code(addr(a)?)
        } else if let Some(a) = nonce {
            Target::Nonce(addr(a)?)
        } else {
            anyhow::bail!("--storage takes <ADDRESS> <SLOT>")
        })
    }

    fn address(&self) -> Address {
        match self {
            Target::Call(spec) => spec.address,
            Target::Balance(a) | Target::Storage(a, _) | Target::Code(a) | Target::Nonce(a) => *a,
        }
    }

    fn kind(&self) -> &'static str {
        match self {
            Target::Call(_) => "call",
            Target::Balance(_) => "balance",
            Target::Storage(..) => "storage",
            Target::Code(_) => "code",
            Target::Nonce(_) => "nonce",
        }
    }

    /// The value at `block`, as text that compares equal exactly when the
    /// values are equal
    async fn read<P: Provider>(&self, provider: &P, block: u64) -> anyhow::Result<String> {
        use alloy::dyn_abi::FunctionExt;
        let id = BlockId::Number(BlockNumberOrTag::Number(block));
        Ok(match self {
            Target::Balance(a) => provider.get_balance(*a).block_id(id).await?.to_string(),
            Target::Nonce(a) => provider
                .get_transaction_count(*a)
                .block_id(id)
                .await?
                .to_string(),
            Target::Storage(a, slot) => {
                let v = provider.get_storage_at(*a, *slot).block_id(id).await?;
                format!("{:#066x}", v)
            }
            Target::Code(a) => {
                let code = provider.get_code_at(*a).block_id(id).await?;
                if code.is_empty() {
                    "none".to_string()
                } else {
                    format!(
                        "{} bytes, keccak {}",
                        code.len(),
                        alloy::primitives::keccak256(&code)
                    )
                }
            }
            Target::Call(spec) => {
                let tx = alloy::rpc::types::TransactionRequest::default()
                    .to(spec.address)
                    .input(spec.calldata.clone().into());
                match provider.call(tx).block(id).await {
                    Ok(data) => {
                        match (spec.function.outputs.is_empty(), data.is_empty()) {
                            // No code yet returns empty data
                            (_, true) => "0x (empty)".to_string(),
                            (true, false) => format!("0x{}", hex::encode(&data)),
                            (false, false) => match spec.function.abi_decode_output(&data) {
                                Ok(values) => {
                                    let json: Vec<_> = values
                                        .iter()
                                        .map(super::contract::dyn_value_to_json)
                                        .collect();
                                    let parts: Vec<String> = json
                                        .iter()
                                        .map(|v| match v {
                                            serde_json::Value::String(s) => s.clone(),
                                            other => other.to_string(),
                                        })
                                        .collect();
                                    parts.join(", ")
                                }
                                Err(_) => format!("0x{}", hex::encode(&data)),
                            },
                        }
                    }
                    Err(e) => match crate::abi::revert::revert_data_of(&e) {
                        Some(data) => format!("reverted 0x{}", hex::encode(&data)),
                        None => return Err(anyhow::anyhow!("Call failed: {e}")),
                    },
                }
            }
        })
    }
}

/// One block where the value differs from the block before it
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change<V> {
    pub block: u64,
    pub old: V,
    pub new: V,
}

/// First block in `(low, high]` whose value differs from `base`, the value
/// at `low`, assuming the value changes at most once in the range. `None` if
/// the value at `high` equals `base`. `high_value` is the value at `high`
/// when already known.
pub async fn bisect<V, F, Fut>(
    mut low: u64,
    mut high: u64,
    base: &V,
    high_value: Option<V>,
    read: &F,
) -> anyhow::Result<Option<Change<V>>>
where
    V: PartialEq + Clone,
    F: Fn(u64) -> Fut,
    Fut: Future<Output = anyhow::Result<V>>,
{
    if high <= low {
        return Ok(None);
    }
    let mut high_value = match high_value {
        Some(v) => v,
        None => read(high).await?,
    };
    if &high_value == base {
        return Ok(None);
    }
    // Invariant: value(low) == base, value(high) != base
    while high - low > 1 {
        let mid = low + (high - low) / 2;
        let v = read(mid).await?;
        if &v == base {
            low = mid;
        } else {
            high = mid;
            high_value = v;
        }
    }
    Ok(Some(Change {
        block: high,
        old: base.clone(),
        new: high_value,
    }))
}

/// Every boundary found by repeating [`bisect`] after each change, up to
/// `max`. Exact when the value never returns to an earlier one inside a
/// segment; otherwise changes between equal endpoints are missed.
pub async fn bisect_all<V, F, Fut>(
    low: u64,
    high: u64,
    base: V,
    read: &F,
    max: usize,
) -> anyhow::Result<Vec<Change<V>>>
where
    V: PartialEq + Clone,
    F: Fn(u64) -> Fut,
    Fut: Future<Output = anyhow::Result<V>>,
{
    let mut changes = Vec::new();
    if high <= low {
        return Ok(changes);
    }
    let end_value = read(high).await?;
    let (mut low, mut base) = (low, base);
    while changes.len() < max {
        match bisect(low, high, &base, Some(end_value.clone()), read).await? {
            Some(change) => {
                low = change.block;
                base = change.new.clone();
                changes.push(change);
            }
            None => break,
        }
    }
    Ok(changes)
}

/// Resolve a `--from`/`--to` value: a block number, "latest", or a time
async fn resolve_block<P: Provider>(provider: &P, spec: &str) -> anyhow::Result<u64> {
    if spec.eq_ignore_ascii_case("latest") {
        return Ok(provider.get_block_number().await?);
    }
    if let Some(r) = crate::rpc::block_time::resolve_time_spec(provider, spec).await? {
        return Ok(r.number);
    }
    crate::utils::parse_block_number(spec)
        .map_err(|e| anyhow::anyhow!("'{spec}' is not a block, 'latest', a date or an age: {e}"))
}

/// Transactions in `block` sent from or to `address` (top level only)
async fn touching_txs<P: Provider>(
    provider: &P,
    block: u64,
    address: Address,
) -> anyhow::Result<Vec<String>> {
    let b = provider
        .get_block(BlockId::Number(BlockNumberOrTag::Number(block)))
        .full()
        .await?
        .ok_or_else(|| anyhow::anyhow!("Block {block} not found"))?;
    let hashes: Vec<String> = b
        .transactions
        .txns()
        .filter(|tx| {
            alloy::network::TransactionResponse::from(*tx) == address
                || alloy::consensus::Transaction::to(*tx) == Some(address)
        })
        .map(|tx| alloy::network::TransactionResponse::tx_hash(tx).to_string())
        .collect();
    Ok(hashes)
}

pub(super) struct Args<'a> {
    pub(super) target: Target,
    pub from: &'a str,
    pub to: &'a str,
    pub all: bool,
    pub json: bool,
}

pub(super) async fn run(
    args: Args<'_>,
    chain: Chain,
    rpc_url: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    // Resolve the range on any endpoint, then search on one that can read
    // state at --from (an archive endpoint for old blocks)
    let first = match &rpc_url {
        Some(url) => Endpoint::new(EndpointConfig::new(url.clone()), 30, None)?,
        None => crate::rpc::get_rpc_endpoint(chain)?,
    };
    let from = resolve_block(first.provider(), args.from).await?;
    let to = resolve_block(first.provider(), args.to).await?;
    if to <= from {
        anyhow::bail!("--to ({to}) must be after --from ({from})");
    }

    let candidates = match rpc_url {
        Some(_) => vec![first],
        None => crate::rpc::candidate_endpoints(
            chain,
            &crate::rpc::SelectionOptions::for_block(from),
            crate::rpc::MAX_FAILOVER_ATTEMPTS,
        )?,
    };
    let calls = AtomicU64::new(0);
    let mut chosen = None;
    let mut errors = Vec::new();
    for endpoint in candidates {
        calls.fetch_add(1, Ordering::Relaxed);
        match args.target.read(endpoint.provider(), from).await {
            Ok(base) => {
                chosen = Some((endpoint, base));
                break;
            }
            Err(e) => errors.push(format!("{e:#}")),
        }
    }
    let Some((endpoint, base)) = chosen else {
        anyhow::bail!(
            "Could not read the value at block {from}: {}. Searching needs historical state; \
             add an archive endpoint (ethcli endpoints add <url> --node-type archive) or pass --rpc-url",
            errors.join("; ")
        );
    };
    let provider = endpoint.provider();
    let read = |block: u64| {
        calls.fetch_add(1, Ordering::Relaxed);
        let target = &args.target;
        async move {
            target
                .read(provider, block)
                .await
                .map_err(|e| anyhow::anyhow!("Reading block {block}: {e:#}"))
        }
    };

    let changes = if args.all {
        bisect_all(from, to, base.clone(), &read, MAX_CHANGES).await?
    } else {
        bisect(from, to, &base, None, &read)
            .await?
            .into_iter()
            .collect()
    };
    let truncated = args.all && changes.len() >= MAX_CHANGES;

    // Block time and touching transactions for each change (2 calls each)
    let address = args.target.address();
    let mut rows = Vec::with_capacity(changes.len());
    for c in &changes {
        calls.fetch_add(2, Ordering::Relaxed);
        let header = provider
            .get_block(BlockId::Number(BlockNumberOrTag::Number(c.block)))
            .await?
            .ok_or_else(|| anyhow::anyhow!("Block {} not found", c.block))?;
        let txs = touching_txs(provider, c.block, address)
            .await
            .unwrap_or_default();
        rows.push(serde_json::json!({
            "block": c.block,
            "timestamp": header.header.timestamp,
            "time": crate::rpc::block_time::format_timestamp(header.header.timestamp),
            "old": c.old,
            "new": c.new,
            "transactions": txs,
        }));
    }
    let rpc_calls = calls.load(Ordering::Relaxed);

    let several = args.all && rows.len() > 1;
    if several && !quiet {
        eprintln!("Warning: {SEVERAL_CHANGES_WARNING}");
    }
    if args.json {
        let out = result_json(
            args.target.kind(),
            &address.to_checksum(None),
            (from, to),
            &base,
            rows,
            args.all,
            truncated,
            rpc_calls,
        );
        println!("{}", serde_json::to_string_pretty(&out)?);
    } else {
        if rows.is_empty() {
            println!("No change between blocks {from} and {to} (value: {base})");
        }
        for r in &rows {
            println!(
                "Block {} ({})",
                r["block"],
                r["time"].as_str().unwrap_or("")
            );
            println!("  old: {}", r["old"].as_str().unwrap_or(""));
            println!("  new: {}", r["new"].as_str().unwrap_or(""));
            for tx in r["transactions"].as_array().into_iter().flatten() {
                println!("  tx:  {}", tx.as_str().unwrap_or(""));
            }
        }
        if truncated {
            println!("Stopped after {MAX_CHANGES} changes");
        }
        if !args.all && !rows.is_empty() {
            println!("Note: {SINGLE_CHANGE_NOTE}");
        }
        if !quiet {
            eprintln!("{rpc_calls} RPC calls");
        }
    }
    Ok(())
}

/// Printed with a plain (non `--all`) result
const SINGLE_CHANGE_NOTE: &str =
    "binary search finds a block where the value changed, not necessarily \
the first change; use --all when the first change matters";

/// Given when `--all` finds more than one change
const SEVERAL_CHANGES_WARNING: &str = "the value changed several times in this range; a search \
without --all may not have returned the first change";

/// The JSON result. `rows` are the changes found, in block order.
#[allow(clippy::too_many_arguments)]
fn result_json(
    kind: &str,
    address: &str,
    (from, to): (u64, u64),
    base: &str,
    rows: Vec<serde_json::Value>,
    all: bool,
    truncated: bool,
    rpc_calls: u64,
) -> serde_json::Value {
    let mut out = serde_json::json!({
        "target": kind,
        "address": address,
        "from": from,
        "to": to,
        "value_at_from": base,
        "changed": !rows.is_empty(),
        "single_change_assumed": !all,
        "rpc_calls": rpc_calls,
    });
    if all {
        if rows.len() > 1 {
            out["warning"] = serde_json::json!(SEVERAL_CHANGES_WARNING);
        }
        out["changes"] = serde_json::Value::Array(rows);
        out["truncated"] = serde_json::json!(truncated);
    } else {
        if !rows.is_empty() {
            out["note"] = serde_json::json!(SINGLE_CHANGE_NOTE);
        }
        out["change"] = rows.into_iter().next().unwrap_or(serde_json::Value::Null);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    /// A value that steps up at each of `steps`
    fn stepped(steps: &'static [u64]) -> impl Fn(u64) -> std::future::Ready<anyhow::Result<u64>> {
        move |b| std::future::ready(Ok(steps.iter().filter(|s| **s <= b).count() as u64))
    }

    #[tokio::test]
    async fn finds_single_change() {
        let f = stepped(&[1234]);
        let c = bisect(0, 10_000, &0, None, &f).await.unwrap().unwrap();
        assert_eq!(
            c,
            Change {
                block: 1234,
                old: 0,
                new: 1
            }
        );
        // Edges: change right after low, and at high
        let c = bisect(1233, 10_000, &0, None, &f).await.unwrap().unwrap();
        assert_eq!(c.block, 1234);
        let c = bisect(0, 1234, &0, None, &f).await.unwrap().unwrap();
        assert_eq!(c.block, 1234);
    }

    #[tokio::test]
    async fn none_without_change() {
        let f = stepped(&[50]);
        assert!(bisect(60, 100, &1, None, &f).await.unwrap().is_none());
        assert!(bisect(100, 100, &1, None, &f).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn uses_logarithmic_reads() {
        let n = Cell::new(0u32);
        let f = |b: u64| {
            n.set(n.get() + 1);
            std::future::ready(Ok::<_, anyhow::Error>(b >= 777_777))
        };
        let c = bisect(0, 1 << 20, &false, None, &f).await.unwrap().unwrap();
        assert_eq!(c.block, 777_777);
        assert!(n.get() <= 21, "{} reads", n.get());
    }

    #[tokio::test]
    async fn all_finds_every_monotonic_change() {
        let f = stepped(&[10, 11, 500, 9_999]);
        let changes = bisect_all(0, 10_000, 0, &f, 100).await.unwrap();
        let blocks: Vec<u64> = changes.iter().map(|c| c.block).collect();
        assert_eq!(blocks, vec![10, 11, 500, 9_999]);
        assert_eq!(changes[3].old, 3);
        assert_eq!(changes[3].new, 4);
        let capped = bisect_all(0, 10_000, 0, &f, 2).await.unwrap();
        assert_eq!(capped.len(), 2);
    }

    #[test]
    fn json_states_the_single_change_assumption() {
        let row = |b: u64| serde_json::json!({ "block": b });
        let single = result_json("nonce", "0x", (1, 9), "0", vec![row(5)], false, false, 4);
        assert_eq!(single["single_change_assumed"], true);
        assert!(single["note"].is_string());
        assert!(single.get("warning").is_none());
        assert_eq!(single["change"]["block"], 5);

        let none = result_json("nonce", "0x", (1, 9), "0", vec![], false, false, 2);
        assert_eq!(none["single_change_assumed"], true);
        assert!(none.get("note").is_none());

        let one = result_json("nonce", "0x", (1, 9), "0", vec![row(5)], true, false, 6);
        assert_eq!(one["single_change_assumed"], false);
        assert!(one.get("warning").is_none());

        let many = result_json(
            "nonce",
            "0x",
            (1, 9),
            "0",
            vec![row(3), row(5)],
            true,
            false,
            9,
        );
        assert_eq!(many["single_change_assumed"], false);
        assert!(many["warning"].as_str().unwrap().contains("several"));
        assert_eq!(many["changes"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn target_needs_exactly_one() {
        let a = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
        assert!(Target::from_args(None, None, &[], None, None).is_err());
        assert!(Target::from_args(None, Some(a), &[], Some(a), None).is_err());
        assert!(matches!(
            Target::from_args(None, None, &[a.into(), "0x2".into()], None, None).unwrap(),
            Target::Storage(_, s) if s == U256::from(2)
        ));
        assert!(matches!(
            Target::from_args(
                Some(&format!("{a} balanceOf(address)(uint256) {a}")),
                None,
                &[],
                None,
                None
            )
            .unwrap(),
            Target::Call(_)
        ));
    }
}
