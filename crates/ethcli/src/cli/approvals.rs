//! `account approvals`: open token approvals and the amount each puts at risk
//!
//! Candidate (token, spender) pairs come from the owner's past approval events,
//! read through the explorer's logs API. Every candidate is then checked
//! against current chain state with Multicall3, so a pair that was revoked or
//! spent down to zero is dropped and the amounts shown are today's.

use super::account::{fetch_all_pages, AllPages, ALL_PAGE_SIZE, PAGE_DELAY};
use crate::config::Chain;
use crate::etherscan::Client;
use crate::rpc::multicall::{selectors, MulticallBuilder, MulticallResult};
use crate::utils::format::format_u256_with_decimals;
use alloy::primitives::{address, Address, Bytes, B256, U256};
use alloy::providers::Provider;
use alloy::sol;
use alloy::sol_types::{SolCall, SolEvent};
use std::collections::{BTreeMap, BTreeSet, HashMap};

/// Uniswap's Permit2, at the same address on every chain it is deployed to
pub const PERMIT2: Address = address!("000000000022D473030F116dDEE9F6B43aC78BA3");

/// Calls per Multicall3 `eth_call`
const CALLS_PER_BATCH: usize = 300;
/// Most tokens priced in one run
const MAX_PRICED_TOKENS: usize = 40;
/// Tokens per DefiLlama price request
const PRICE_BATCH: usize = 50;
/// Most spender contracts checked for verified source in one run
const MAX_SOURCE_CHECKS: usize = 60;

sol! {
    event Approval(address indexed owner, address indexed spender, uint256 value);
    event ApprovalForAll(address indexed owner, address indexed operator, bool approved);
    function allowance(address owner, address spender) external view returns (uint256);
    function isApprovedForAll(address owner, address operator) external view returns (bool);
    function getCurrentBlockTimestamp() external view returns (uint256);
}

/// Permit2's own events and allowance getter
mod permit2 {
    alloy::sol! {
        event Approval(address indexed owner, address indexed token, address indexed spender, uint160 amount, uint48 expiration);
        event Permit(address indexed owner, address indexed token, address indexed spender, uint160 amount, uint48 expiration, uint48 nonce);
        function allowance(address owner, address token, address spender) external view returns (uint160 amount, uint48 expiration, uint48 nonce);
    }
}

/// Longest symbol kept in the output
const MAX_SYMBOL_CHARS: usize = 32;
/// Longest contract name kept in the output
const MAX_NAME_CHARS: usize = 64;

/// Token-supplied or explorer-supplied text, made safe to print and to pass
/// on: hidden and control characters escaped, cut to `max` characters
pub(crate) fn clean_text(text: &str, max: usize) -> String {
    let safe = crate::abi::revert::safe_text(text.trim());
    if safe.chars().count() <= max {
        safe
    } else {
        let mut cut: String = safe.chars().take(max.saturating_sub(1)).collect();
        cut.push('…');
        cut
    }
}

/// Whether a symbol looks like a lure rather than a ticker: characters
/// outside letters, digits and `-_.+$/`, or link/claim wording
pub(crate) fn symbol_suspicious(symbol: &str) -> bool {
    let lower = symbol.to_lowercase();
    let odd_chars = symbol
        .chars()
        .any(|c| !(c.is_alphanumeric() || "-_.+$/".contains(c)));
    let lure = [
        "http", "www", "://", ".com", ".io", ".org", ".xyz", "visit", "claim", "airdrop", "reward",
    ]
    .iter()
    .any(|w| lower.contains(w));
    symbol.is_empty() || odd_chars || lure
}

/// One log row from the explorer
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LogRow {
    pub address: Address,
    pub topics: Vec<B256>,
    pub block: u64,
}

fn parse_hex_u64(text: &str) -> Option<u64> {
    match text.strip_prefix("0x") {
        Some("") => Some(0),
        Some(hex) => u64::from_str_radix(hex, 16).ok(),
        None => text.parse().ok(),
    }
}

/// Parse one row of the explorer's `logs/getLogs` result
pub(crate) fn parse_log_row(value: &serde_json::Value) -> Option<LogRow> {
    let address = value["address"].as_str()?.parse().ok()?;
    let topics = value["topics"]
        .as_array()?
        .iter()
        .filter_map(|t| t.as_str())
        .filter(|t| !t.is_empty())
        .map(|t| t.parse::<B256>().ok())
        .collect::<Option<Vec<_>>>()?;
    let block = parse_hex_u64(value["blockNumber"].as_str()?)?;
    Some(LogRow {
        address,
        topics,
        block,
    })
}

fn topic_address(topic: &B256) -> Address {
    Address::from_slice(&topic[12..])
}

/// The delegate of an EIP-7702 delegated account (`0xef0100 || address`)
pub(crate) fn eip7702_delegate(code: &[u8]) -> Option<Address> {
    (code.len() == 23 && code.starts_with(&[0xef, 0x01, 0x00]))
        .then(|| Address::from_slice(&code[3..]))
}

/// Amount at risk for an allowance: the spender can take no more than the
/// owner holds
pub(crate) fn exposure(allowance: U256, balance: U256) -> U256 {
    allowance.min(balance)
}

/// Whether an allowance is one of the "infinite" values wallets and dapps set
pub(crate) fn is_unlimited(allowance: U256) -> bool {
    let max_of = |bits: usize| (U256::from(1u8) << bits) - U256::from(1u8);
    allowance >= U256::from(1u8) << 255 || allowance == max_of(160) || allowance == max_of(96)
}

/// A Permit2 allowance is usable while `now <= expiration`
pub(crate) fn permit2_expired(expiration: u64, now: u64) -> bool {
    now > expiration
}

/// `raw` as a float in whole tokens, for sorting and USD values
pub(crate) fn to_float(raw: U256, decimals: u8) -> f64 {
    alloy::primitives::utils::format_units(raw, decimals)
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0)
}

/// Candidate pairs found in the owner's events
#[derive(Debug, Default)]
pub(crate) struct Candidates {
    /// (token, spender) → last block with an Approval event
    pub erc20: BTreeMap<(Address, Address), u64>,
    /// (collection, operator) → last block with an ApprovalForAll event
    pub operators: BTreeMap<(Address, Address), u64>,
    /// (token, spender) → last block with a Permit2 Approval or Permit event
    pub permit2: BTreeMap<(Address, Address), u64>,
}

impl Candidates {
    pub(crate) fn add_approval_logs(&mut self, rows: &[LogRow]) {
        for row in rows {
            // ERC-721 Approval has the token id as a fourth topic: a
            // single-token approval, cleared on transfer, not tracked here
            if row.topics.len() == 3 && row.topics[0] == Approval::SIGNATURE_HASH {
                bump(
                    &mut self.erc20,
                    (row.address, topic_address(&row.topics[2])),
                    row.block,
                );
            }
        }
    }

    pub(crate) fn add_operator_logs(&mut self, rows: &[LogRow]) {
        for row in rows {
            if row.topics.len() == 3 && row.topics[0] == ApprovalForAll::SIGNATURE_HASH {
                bump(
                    &mut self.operators,
                    (row.address, topic_address(&row.topics[2])),
                    row.block,
                );
            }
        }
    }

    pub(crate) fn add_permit2_logs(&mut self, rows: &[LogRow]) {
        for row in rows {
            let is_permit2_event = row.topics[0] == permit2::Approval::SIGNATURE_HASH
                || row.topics[0] == permit2::Permit::SIGNATURE_HASH;
            if row.address == PERMIT2 && row.topics.len() == 4 && is_permit2_event {
                bump(
                    &mut self.permit2,
                    (topic_address(&row.topics[2]), topic_address(&row.topics[3])),
                    row.block,
                );
            }
        }
    }
}

fn bump(map: &mut BTreeMap<(Address, Address), u64>, key: (Address, Address), block: u64) {
    let entry = map.entry(key).or_insert(block);
    *entry = (*entry).max(block);
}

/// One explorer log query and how far it got
struct Scan {
    event: &'static str,
    rows: Vec<LogRow>,
    report: serde_json::Value,
}

#[allow(clippy::too_many_arguments)]
async fn scan_logs(
    client: &Client,
    event: &'static str,
    contract: Option<Address>,
    topic0: B256,
    owner: Address,
    from_block: u64,
    to_block: u64,
    max_pages: u64,
) -> anyhow::Result<Scan> {
    let owner_topic = format!("{:#x}", B256::left_padding_from(owner.as_slice()));
    let fetched: AllPages<LogRow> = fetch_all_pages(
        from_block,
        to_block,
        true,
        max_pages,
        PAGE_DELAY,
        |page, start, end| {
            let mut params = vec![
                ("module", "logs".to_string()),
                ("action", "getLogs".to_string()),
                ("fromBlock", start.to_string()),
                ("toBlock", end.to_string()),
                ("topic0", format!("{topic0:#x}")),
                ("topic0_1_opr", "and".to_string()),
                ("topic1", owner_topic.clone()),
                ("page", page.to_string()),
                ("offset", ALL_PAGE_SIZE.to_string()),
            ];
            if let Some(contract) = contract {
                params.push(("address", format!("{contract:#x}")));
            }
            async move {
                let result = client
                    .explorer_get(&params)
                    .await
                    .map_err(|e| anyhow::anyhow!("explorer getLogs ({event}): {e}"))?;
                Ok(result
                    .as_array()
                    .map(|rows| rows.iter().filter_map(parse_log_row).collect())
                    .unwrap_or_default())
            }
        },
        |row: &LogRow| row.block,
    )
    .await?;
    let scanned_to = if fetched.complete {
        to_block
    } else {
        fetched.rows.last().map_or(from_block, |r| r.block)
    };
    let mut report = serde_json::json!({
        "event": event,
        "from_block": from_block,
        "scanned_to_block": scanned_to,
        "logs": fetched.rows.len(),
        "requests": fetched.requests,
        "complete": fetched.complete,
    });
    if let Some(stopped) = &fetched.stopped {
        report["stopped"] = serde_json::json!(stopped);
    }
    Ok(Scan {
        event,
        rows: fetched.rows,
        report,
    })
}

/// Run `calls` through Multicall3 in batches, keeping the order
///
/// Candidate tokens are attacker-chosen, and one call that burns all the gas
/// makes a whole `aggregate3` fail. A failed batch is split in half and each
/// half retried, down to single calls; a single call that still fails comes
/// back unread (`None`). A call that fails alone with an execution error
/// (revert, out of gas, node execution timeout) is the target's fault and its
/// address goes in `reverted`; any other error (transport, rate limit) is
/// retried with backoff and, if it persists, goes in `transport_failed`.
async fn multicall_all<P: Provider>(
    provider: &P,
    calls: Vec<(Address, Bytes)>,
) -> anyhow::Result<SplitRead> {
    run_split(
        calls,
        MAX_SPLIT_REQUESTS,
        |batch: Vec<(Address, Bytes)>, retries: u32| async move {
            let mut builder = MulticallBuilder::new();
            for (target, data) in batch {
                builder = builder.add_call_allow_failure(target, data);
            }
            builder.execute_with_retry(provider, retries).await
        },
    )
    .await
}

/// What a read that came back without a usable value means
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Resolved {
    /// The value was read
    Known(U256),
    /// The call ran but the address has no code: nothing can be moved, so
    /// the exposure is a known zero
    NoCode,
    /// The address has code but did not answer like a token (reverted or
    /// returned data that does not decode)
    NonStandard,
    /// The call was never read (RPC failure, request cap), or the code
    /// check failed
    Unknown,
}

impl Resolved {
    pub(crate) fn value(self) -> Option<U256> {
        match self {
            Resolved::Known(v) => Some(v),
            Resolved::NoCode => Some(U256::ZERO),
            Resolved::NonStandard | Resolved::Unknown => None,
        }
    }
}

/// Classify a read: `value` is the decoded value if any, `unread` whether a
/// call it needs was never answered, `has_code` the target's code check
pub(crate) fn resolve_read(value: Option<U256>, unread: bool, has_code: Option<bool>) -> Resolved {
    match (value, unread, has_code) {
        (Some(v), _, _) => Resolved::Known(v),
        (None, true, _) | (None, false, None) => Resolved::Unknown,
        (None, false, Some(false)) => Resolved::NoCode,
        (None, false, Some(true)) => Resolved::NonStandard,
    }
}

/// Most `eth_call`s one state read may make, splitting included
const MAX_SPLIT_REQUESTS: usize = 200;
/// Retries (with backoff) of a single call that failed without an
/// execution error
const SINGLE_CALL_RETRIES: u32 = 2;

/// Results of [`run_split`]
#[derive(Debug, Default)]
pub(crate) struct SplitRead {
    /// One entry per call, in order; `None` when the call was never read
    pub results: Vec<Option<MulticallResult>>,
    /// Targets whose call failed alone with an execution error
    pub reverted: BTreeSet<Address>,
    /// Targets whose call failed alone for another reason, after retries
    pub transport_failed: BTreeSet<Address>,
    /// `eth_call`s made
    pub requests: usize,
    /// Whether the request cap stopped the splitting
    pub capped: bool,
}

/// Whether an `eth_call` error came from executing the call (the target's
/// doing) rather than from reaching the node
pub(crate) fn is_execution_error(error: &anyhow::Error) -> bool {
    let text = format!("{error:#}").to_ascii_lowercase();
    [
        "revert",
        "out of gas",
        "gas required exceeds",
        "execution aborted",
        "execution timeout",
        "invalid opcode",
        "stack underflow",
        "stack overflow",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

/// The batching and splitting behind [`multicall_all`], with the call that
/// runs one batch passed in (`exec(calls, retries)`) so it can be tested.
/// Fails only when no call could be read and the node itself failed.
async fn run_split<F, Fut>(
    calls: Vec<(Address, Bytes)>,
    max_requests: usize,
    exec: F,
) -> anyhow::Result<SplitRead>
where
    F: Fn(Vec<(Address, Bytes)>, u32) -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<Vec<MulticallResult>>>,
{
    let mut out = SplitRead {
        results: vec![None; calls.len()],
        ..Default::default()
    };
    let mut last_transport_error = None;
    let mut pending: Vec<(usize, usize)> = (0..calls.len())
        .step_by(CALLS_PER_BATCH)
        .map(|start| (start, (start + CALLS_PER_BATCH).min(calls.len())))
        .rev()
        .collect();
    while let Some((start, end)) = pending.pop() {
        if out.requests >= max_requests {
            out.capped = true;
            break;
        }
        out.requests += 1;
        let single = end - start == 1;
        let retries = if single { SINGLE_CALL_RETRIES } else { 1 };
        match exec(calls[start..end].to_vec(), retries).await {
            Ok(batch) if batch.len() == end - start => {
                for (i, r) in batch.into_iter().enumerate() {
                    out.results[start + i] = Some(r);
                }
            }
            outcome => {
                if !single {
                    let mid = start + (end - start) / 2;
                    pending.push((mid, end));
                    pending.push((start, mid));
                } else {
                    match outcome {
                        Err(e) if is_execution_error(&e) => {
                            out.reverted.insert(calls[start].0);
                        }
                        Err(e) => {
                            out.transport_failed.insert(calls[start].0);
                            last_transport_error = Some(e);
                        }
                        // A wrong-length answer for one call: not the target's fault
                        Ok(_) => {
                            out.transport_failed.insert(calls[start].0);
                        }
                    }
                }
            }
        }
    }
    if !calls.is_empty() && out.results.iter().all(Option::is_none) {
        if let Some(e) = last_transport_error {
            return Err(e.context("could not read any approval state"));
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Default)]
struct TokenInfo {
    symbol: Option<String>,
    decimals: Option<u8>,
    balance: Option<U256>,
}

/// What is known about a spender or operator
#[derive(Debug, Clone, Default)]
struct SpenderInfo {
    label: Option<String>,
    kind: &'static str,
    verified: Option<bool>,
    contract_name: Option<String>,
}

impl SpenderInfo {
    fn json(&self, address: Address) -> serde_json::Value {
        let mut out = serde_json::json!({
            "address": address.to_checksum(None),
            "kind": self.kind,
        });
        if let Some(label) = &self.label {
            out["label"] = serde_json::json!(clean_text(label, MAX_NAME_CHARS));
        }
        if let Some(verified) = self.verified {
            out["verified"] = serde_json::json!(verified);
        }
        if let Some(name) = &self.contract_name {
            out["contract_name"] = serde_json::json!(name);
        }
        out
    }

    fn flags(&self) -> Vec<&'static str> {
        let mut flags = Vec::new();
        if self.kind == "eoa" || self.kind == "eoa_7702_delegated" {
            flags.push("eoa_spender");
        }
        if self.kind == "eoa_7702_delegated" {
            // Key-controlled, whatever code it delegates to
            flags.push("delegated_eoa");
        }
        if self.verified == Some(false) {
            flags.push("unverified_spender");
        }
        flags
    }
}

async fn describe_spenders<P: Provider>(
    client: &Client,
    provider: &P,
    spenders: &BTreeSet<Address>,
    source_check: &BTreeSet<Address>,
) -> HashMap<Address, SpenderInfo> {
    let book = crate::config::AddressBook::load_default();
    let book_entries = book.list(None);
    use futures::StreamExt;
    let codes: Vec<_> = futures::stream::iter(spenders.iter())
        .map(|s| async move { provider.get_code_at(*s).await })
        .buffered(8)
        .collect()
        .await;
    let mut out = HashMap::new();
    let mut first_request = true;
    for (spender, code) in spenders.iter().zip(codes) {
        let lower = format!("{spender:#x}");
        let label = crate::tx::addresses::get_label(spender)
            .map(str::to_string)
            .or_else(|| {
                book_entries
                    .iter()
                    .find(|(_, e)| e.address.eq_ignore_ascii_case(&lower))
                    .map(|(label, _)| label.to_string())
            })
            .or_else(|| (*spender == PERMIT2).then(|| "Uniswap Permit2".to_string()));
        let kind = match &code {
            Ok(code) if code.is_empty() => "eoa",
            Ok(code) if eip7702_delegate(code).is_some() => "eoa_7702_delegated",
            Ok(_) => "contract",
            Err(_) => "unknown",
        };
        let mut info = SpenderInfo {
            label,
            kind,
            ..Default::default()
        };
        if source_check.contains(spender) && kind == "contract" {
            if !first_request {
                tokio::time::sleep(PAGE_DELAY).await;
            }
            first_request = false;
            if let Ok(result) = client
                .explorer_get(&[
                    ("module", "contract".to_string()),
                    ("action", "getsourcecode".to_string()),
                    ("address", lower.clone()),
                ])
                .await
            {
                let entry = &result[0];
                let source = entry["SourceCode"].as_str().unwrap_or_default();
                info.verified = Some(!source.is_empty());
                info.contract_name = entry["ContractName"]
                    .as_str()
                    .filter(|n| !n.is_empty())
                    .map(|n| clean_text(n, MAX_NAME_CHARS));
            }
        }
        out.insert(*spender, info);
    }
    out
}

/// A DefiLlama price and its confidence (1.0 when the source gives none)
#[derive(Debug, Clone, Copy)]
pub(crate) struct Price {
    pub usd: f64,
    pub confidence: f64,
}

/// Prices below this confidence are shown per row but left out of the total
pub(crate) const MIN_TOTAL_CONFIDENCE: f64 = 0.9;

/// USD totals over the rows' `exposure_usd`: (counted, excluded as spam,
/// excluded for low confidence)
///
/// One balance approved to several spenders (or to Permit2 and its spenders)
/// can be taken only once, so each token counts with its largest exposure.
pub(crate) fn usd_totals<'a>(rows: impl Iterator<Item = &'a serde_json::Value>) -> (f64, f64, f64) {
    let mut per_token: BTreeMap<String, (f64, &'a serde_json::Value)> = BTreeMap::new();
    for row in rows {
        let Some(usd) = row["exposure_usd"].as_f64() else {
            continue;
        };
        let key = row["token"]["address"]
            .as_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let entry = per_token.entry(key).or_insert((usd, row));
        if usd > entry.0 {
            *entry = (usd, row);
        }
    }
    let (mut counted, mut spam, mut low) = (0.0, 0.0, 0.0);
    for (usd, row) in per_token.into_values() {
        if row["token"]["spam"] == true {
            spam += usd;
        } else if row["price_confidence"].as_f64().unwrap_or(0.0) >= MIN_TOTAL_CONFIDENCE {
            counted += usd;
        } else {
            low += usd;
        }
    }
    (counted, spam, low)
}

/// The `n` largest token contributions to `total_exposure_usd`: per token
/// the row with the largest exposure, counted (not spam, confident price)
pub(crate) fn top_exposures<'a>(
    rows: impl Iterator<Item = &'a serde_json::Value>,
    n: usize,
) -> Vec<serde_json::Value> {
    let mut per_token: BTreeMap<String, (f64, &'a serde_json::Value)> = BTreeMap::new();
    for row in rows {
        let Some(usd) = row["exposure_usd"].as_f64() else {
            continue;
        };
        if row["token"]["spam"] == true
            || row["price_confidence"].as_f64().unwrap_or(0.0) < MIN_TOTAL_CONFIDENCE
        {
            continue;
        }
        let key = row["token"]["address"]
            .as_str()
            .unwrap_or_default()
            .to_ascii_lowercase();
        let entry = per_token.entry(key).or_insert((usd, row));
        if usd > entry.0 {
            *entry = (usd, row);
        }
    }
    let mut top: Vec<(f64, &serde_json::Value)> = per_token.into_values().collect();
    top.sort_by(|a, b| b.0.total_cmp(&a.0));
    top.into_iter()
        .take(n)
        .map(|(usd, row)| {
            serde_json::json!({
                "token": row["token"]["address"],
                "symbol": row["token"]["symbol"],
                "exposure_formatted": row["exposure_formatted"],
                "exposure_usd": usd,
                "price_usd": row["price_usd"],
                "price_source": row["price_source"],
                "price_confidence": row["price_confidence"],
            })
        })
        .collect()
}

/// Sum of `exposure_usd` over every row, counting a balance once per spender
pub(crate) fn per_approval_sum<'a>(rows: impl Iterator<Item = &'a serde_json::Value>) -> f64 {
    rows.filter_map(|r| r["exposure_usd"].as_f64()).sum()
}

/// Spam reason for a token: on the local blacklist, or (unless a known
/// protocol token) unverified, a honeypot or a sell tax over 10% per GoPlus
async fn spam_check(chain: Chain, token: Address) -> Option<String> {
    let text = format!("{token:#x}");
    if crate::config::TokenBlacklist::load_default().is_blacklisted(&text) {
        return Some("on the local token blacklist".to_string());
    }
    if crate::cli::blacklist::is_known_safe_protocol(chain.chain_id(), &text) {
        return None;
    }
    let security = crate::cli::blacklist::check_security(chain.chain_id(), &text).await?;
    if security.is_honeypot {
        Some("GoPlus: honeypot".to_string())
    } else if security.has_high_sell_tax() {
        Some("GoPlus: sell tax over 10%".to_string())
    } else if !security.is_open_source {
        Some("GoPlus: source not verified".to_string())
    } else {
        None
    }
}

async fn spam_checks(chain: Chain, tokens: &[Address]) -> HashMap<Address, String> {
    use futures::StreamExt;
    futures::stream::iter(tokens.iter().take(MAX_PRICED_TOKENS).copied())
        .map(|t| async move { spam_check(chain, t).await.map(|r| (t, r)) })
        .buffered(4)
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}

/// A token ethcli knows by address: built-in labels, the price module's
/// symbol table, or the blacklist's known protocol tokens
fn is_known_token(chain: Chain, token: Address) -> bool {
    let text = format!("{token:#x}");
    crate::tx::addresses::get_label(&token).is_some()
        || (chain == Chain::Ethereum
            && crate::aggregator::price::eth_address_to_symbol(&text).is_some())
        || crate::cli::blacklist::is_known_safe_protocol(chain.chain_id(), &text)
}

/// DefiLlama prices for `tokens`, [`PRICE_BATCH`] per request
async fn usd_prices(chain: Chain, tokens: &[Address]) -> HashMap<Address, Price> {
    let Ok(client) = crate::aggregator::price::get_llama_client() else {
        return HashMap::new();
    };
    let llama_chain =
        crate::aggregator::chain_map::normalize_chain_for_source("llama", chain.name());
    let mut out = HashMap::new();
    for chunk in tokens.chunks(PRICE_BATCH) {
        let ids: Vec<dllma::coins::Token> = chunk
            .iter()
            .map(|t| dllma::coins::Token::new(llama_chain.clone(), format!("{t:#x}")))
            .collect();
        let Ok(response) = client.coins().current(&ids).await else {
            continue;
        };
        for (token, id) in chunk.iter().zip(&ids) {
            if let Some(coin) = response.coins.get(&id.format()) {
                if coin.price.is_finite() && coin.price > 0.0 {
                    out.insert(
                        *token,
                        Price {
                            usd: coin.price,
                            confidence: coin.confidence.unwrap_or(1.0),
                        },
                    );
                }
            }
        }
    }
    out
}

/// Options for [`approvals_report`]
pub struct ApprovalsOptions {
    pub from_block: u64,
    pub max_pages: u64,
    pub price: bool,
    pub source_check: bool,
    /// Leave out rows whose exposure is zero (counted in `hidden_zero_exposure`)
    pub exposed_only: bool,
    pub quiet: bool,
}

/// Build the full approvals report for `owner`
pub async fn approvals_report(
    client: &Client,
    chain: Chain,
    owner: Address,
    opts: &ApprovalsOptions,
) -> anyhow::Result<serde_json::Value> {
    let endpoint = crate::rpc::get_rpc_endpoint(chain)?;
    let provider = endpoint.provider();
    let latest = provider
        .get_block_number()
        .await
        .map_err(|e| anyhow::anyhow!("Could not read the latest block: {e}"))?;

    let queries: [(&'static str, Option<Address>, B256); 4] = [
        ("Approval", None, Approval::SIGNATURE_HASH),
        ("ApprovalForAll", None, ApprovalForAll::SIGNATURE_HASH),
        (
            "Permit2.Approval",
            Some(PERMIT2),
            permit2::Approval::SIGNATURE_HASH,
        ),
        (
            "Permit2.Permit",
            Some(PERMIT2),
            permit2::Permit::SIGNATURE_HASH,
        ),
    ];
    let mut candidates = Candidates::default();
    let mut scans = Vec::new();
    for (i, (event, contract, topic0)) in queries.into_iter().enumerate() {
        if i > 0 {
            tokio::time::sleep(PAGE_DELAY).await;
        }
        if !opts.quiet {
            eprintln!("Scanning {event} events...");
        }
        let scan = scan_logs(
            client,
            event,
            contract,
            topic0,
            owner,
            opts.from_block,
            latest,
            opts.max_pages,
        )
        .await?;
        match scan.event {
            "Approval" => candidates.add_approval_logs(&scan.rows),
            "ApprovalForAll" => candidates.add_operator_logs(&scan.rows),
            _ => candidates.add_permit2_logs(&scan.rows),
        }
        scans.push(scan.report);
    }

    // Current state, one batch of calls
    let tokens: BTreeSet<Address> = candidates
        .erc20
        .keys()
        .chain(candidates.permit2.keys())
        .map(|(token, _)| *token)
        .collect();
    let collections: BTreeSet<Address> = candidates.operators.keys().map(|(c, _)| *c).collect();
    let mut calls: Vec<(Address, Bytes)> = vec![(
        crate::rpc::multicall::MULTICALL3_ADDRESS,
        getCurrentBlockTimestampCall {}.abi_encode().into(),
    )];
    for token in &tokens {
        calls.push((*token, selectors::symbol()));
        calls.push((*token, selectors::decimals()));
        calls.push((*token, selectors::balance_of(owner)));
    }
    for (token, spender) in candidates.erc20.keys() {
        let data = allowanceCall {
            owner,
            spender: *spender,
        }
        .abi_encode();
        calls.push((*token, data.into()));
    }
    for collection in &collections {
        calls.push((*collection, selectors::balance_of(owner)));
    }
    for (collection, operator) in candidates.operators.keys() {
        let data = isApprovedForAllCall {
            owner,
            operator: *operator,
        }
        .abi_encode();
        calls.push((*collection, data.into()));
    }
    for (token, spender) in candidates.permit2.keys() {
        let data = permit2::allowanceCall {
            owner,
            token: *token,
            spender: *spender,
        }
        .abi_encode();
        calls.push((PERMIT2, data.into()));
    }
    // The token's own allowance to Permit2 caps every Permit2 allowance. Read
    // it on chain: its Approval event may be outside the scanned range.
    let permit2_tokens: BTreeSet<Address> =
        candidates.permit2.keys().map(|(token, _)| *token).collect();
    for token in &permit2_tokens {
        let data = allowanceCall {
            owner,
            spender: PERMIT2,
        }
        .abi_encode();
        calls.push((*token, data.into()));
    }
    if !opts.quiet {
        eprintln!(
            "Reading current state for {} candidate approvals...",
            candidates.erc20.len() + candidates.operators.len() + candidates.permit2.len()
        );
    }
    let targets: Vec<Address> = calls.iter().map(|c| c.0).collect();
    let read = multicall_all(provider, calls).await?;
    let unread_targets: BTreeSet<Address> = targets
        .iter()
        .zip(&read.results)
        .filter(|(_, r)| r.is_none())
        .map(|(t, _)| *t)
        .collect();
    let unknown_reads = read.results.iter().filter(|r| r.is_none()).count();
    let mut results = read.results.clone().into_iter();
    // A call that was not read, or reverted inside the batch, is unknown:
    // never a zero
    let mut next = || results.next().flatten().filter(|r| r.success);

    let now = next()
        .and_then(|r| r.decode_uint256())
        .map(|t| t.saturating_to::<u64>())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        });
    let mut token_info: HashMap<Address, TokenInfo> = HashMap::new();
    for token in &tokens {
        let symbol = next().and_then(|r| r.decode_string());
        let decimals = next().and_then(|r| r.decode_uint8());
        let balance = next().and_then(|r| r.decode_uint256());
        token_info.insert(
            *token,
            TokenInfo {
                symbol,
                decimals,
                balance,
            },
        );
    }
    let erc20_state: Vec<((Address, Address), u64, Option<U256>)> = candidates
        .erc20
        .iter()
        .map(|(pair, block)| (*pair, *block, next().and_then(|r| r.decode_uint256())))
        .collect();
    let collection_balance: HashMap<Address, Option<U256>> = collections
        .iter()
        .map(|c| (*c, next().and_then(|r| r.decode_uint256())))
        .collect();
    let operator_state: Vec<((Address, Address), u64, Option<bool>)> = candidates
        .operators
        .iter()
        .map(|(pair, block)| {
            let approved = next()
                .and_then(|r| r.decode_uint256())
                .map(|v| !v.is_zero());
            (*pair, *block, approved)
        })
        .collect();
    #[allow(clippy::type_complexity)]
    let permit2_state: Vec<((Address, Address), u64, Option<(U256, u64, u64)>)> = candidates
        .permit2
        .iter()
        .map(|(pair, block)| {
            let decoded = next()
                .and_then(|r| permit2::allowanceCall::abi_decode_returns(&r.data).ok())
                .map(|out| {
                    (
                        U256::from(out.amount),
                        out.expiration.to::<u64>(),
                        out.nonce.to::<u64>(),
                    )
                });
            (*pair, *block, decoded)
        })
        .collect();

    let permit2_erc20_allowance: HashMap<Address, Option<U256>> = permit2_tokens
        .iter()
        .map(|t| (*t, next().and_then(|r| r.decode_uint256())))
        .collect();

    // Keep the open ones, and every one whose state is unknown
    let open_erc20: Vec<_> = erc20_state
        .into_iter()
        .filter(|(_, _, allowance)| allowance.is_none_or(|a| !a.is_zero()))
        .collect();
    let open_operators: Vec<_> = operator_state
        .into_iter()
        .filter(|(_, _, approved)| *approved != Some(false))
        .collect();
    let open_permit2: Vec<_> = permit2_state
        .into_iter()
        .filter(|(_, _, state)| state.is_none_or(|(amount, _, _)| !amount.is_zero()))
        .collect();

    // Amount at risk, or None when a value it needs is unknown
    let erc20_at_risk = |token: &Address, allowance: Option<U256>| -> Option<U256> {
        let balance = token_info.get(token).and_then(|t| t.balance)?;
        Some(exposure(allowance?, balance))
    };
    let permit2_at_risk = |token: &Address, state: Option<(U256, u64, u64)>| -> Option<U256> {
        let (amount, expiration, _) = state?;
        if permit2_expired(expiration, now) {
            return Some(U256::ZERO);
        }
        let cap = permit2_erc20_allowance.get(token).copied().flatten()?;
        let balance = token_info.get(token).and_then(|t| t.balance)?;
        Some(exposure(amount.min(cap), balance))
    };

    // Contracts that answered without a usable value: check for code, so an
    // address with no code reads as a known zero, not as unknown state
    let answered_badly: BTreeSet<Address> = open_erc20
        .iter()
        .filter(|((t, _), _, a)| erc20_at_risk(t, *a).is_none() && !unread_targets.contains(t))
        .map(|((t, _), _, _)| *t)
        .chain(
            open_permit2
                .iter()
                .filter(|((t, _), _, st)| {
                    permit2_at_risk(t, *st).is_none()
                        && !unread_targets.contains(t)
                        && !unread_targets.contains(&PERMIT2)
                })
                .map(|((t, _), _, _)| *t),
        )
        .chain(
            open_operators
                .iter()
                .filter(|((c, _), _, ap)| ap.is_none() && !unread_targets.contains(c))
                .map(|((c, _), _, _)| *c),
        )
        .collect();
    let has_code: HashMap<Address, Option<bool>> = {
        use futures::StreamExt;
        futures::stream::iter(answered_badly.iter().copied())
            .map(|a| async move { (a, provider.get_code_at(a).await.ok().map(|c| !c.is_empty())) })
            .buffered(8)
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect()
    };
    let code_of = |a: &Address| has_code.get(a).copied().flatten();

    // Owner account: EIP-7702 delegation
    let owner_code = provider.get_code_at(owner).await.ok();
    let delegation = owner_code.as_ref().and_then(|c| eip7702_delegate(c));

    // Spenders and prices
    let mut source_notes: Vec<String> = Vec::new();
    let spenders: BTreeSet<Address> = open_erc20
        .iter()
        .map(|((_, s), _, _)| *s)
        .chain(open_operators.iter().map(|((_, o), _, _)| *o))
        .chain(open_permit2.iter().map(|((_, s), _, _)| *s))
        .chain(delegation)
        .collect();
    if !opts.quiet && !spenders.is_empty() {
        eprintln!("Describing {} spenders...", spenders.len());
    }
    // Source checks cost one explorer request each: only for spenders that
    // can take something now (or the delegate)
    let mut to_check: BTreeSet<Address> = BTreeSet::new();
    if opts.source_check {
        for ((token, spender), _, allowance) in &open_erc20 {
            if erc20_at_risk(token, *allowance).is_none_or(|e| !e.is_zero()) {
                to_check.insert(*spender);
            }
        }
        to_check.extend(open_operators.iter().map(|((_, o), _, _)| *o));
        to_check.extend(open_permit2.iter().map(|((_, s), _, _)| *s));
        to_check.extend(delegation);
        let all = to_check.len();
        to_check = to_check.into_iter().take(MAX_SOURCE_CHECKS).collect();
        if all > MAX_SOURCE_CHECKS {
            source_notes.push(format!(
                "Verification was checked for {MAX_SOURCE_CHECKS} of {all} spenders with exposure"
            ));
        }
    }
    let spender_info = describe_spenders(client, provider, &spenders, &to_check).await;
    let exposed_tokens: Vec<Address> = {
        let mut set = BTreeSet::new();
        for ((token, _), _, allowance) in &open_erc20 {
            if erc20_at_risk(token, *allowance).is_some_and(|e| !e.is_zero()) {
                set.insert(*token);
            }
        }
        for ((token, _), _, state) in &open_permit2 {
            if permit2_at_risk(token, *state).is_some_and(|e| !e.is_zero()) {
                set.insert(*token);
            }
        }
        // Known tokens first, so the spam-check cap never skips them
        let mut tokens: Vec<Address> = set.into_iter().collect();
        tokens.sort_by_key(|t| !is_known_token(chain, *t));
        tokens
    };
    let prices = if opts.price && !exposed_tokens.is_empty() {
        if !opts.quiet {
            eprintln!("Pricing {} tokens...", exposed_tokens.len());
        }
        usd_prices(chain, &exposed_tokens).await
    } else {
        HashMap::new()
    };
    let spam: HashMap<Address, String> = if opts.price && !exposed_tokens.is_empty() {
        if !opts.quiet {
            eprintln!("Checking {} tokens for spam...", exposed_tokens.len());
        }
        // Known tokens first, then the largest priced holdings, so the cap
        // never skips what dominates the total
        let held_usd = |t: &Address| {
            let info = token_info.get(t);
            match (prices.get(t), info.and_then(|i| i.balance)) {
                (Some(p), Some(b)) => {
                    to_float(b, info.and_then(|i| i.decimals).unwrap_or(0)) * p.usd
                }
                _ => 0.0,
            }
        };
        let mut order = exposed_tokens.clone();
        order.sort_by(|a, b| {
            is_known_token(chain, *b)
                .cmp(&is_known_token(chain, *a))
                .then(held_usd(b).total_cmp(&held_usd(a)))
        });
        spam_checks(chain, &order).await
    } else {
        HashMap::new()
    };

    let token_json = |token: Address| {
        let info = token_info.get(&token).cloned().unwrap_or_default();
        serde_json::json!({
            "address": token.to_checksum(None),
            "symbol": info.symbol.as_deref().map(|s| clean_text(s, MAX_SYMBOL_CHARS)),
            "symbol_untrusted": !prices.contains_key(&token)
                || info.symbol.as_deref().is_none_or(symbol_suspicious),
            "decimals": info.decimals,
            "spam": spam.contains_key(&token),
            "spam_reason": spam.get(&token),
        })
    };
    let amount =
        |raw: U256, decimals: Option<u8>| decimals.map(|d| format_u256_with_decimals(&raw, d));
    let spender_entry = |s: Address| spender_info.get(&s).cloned().unwrap_or_default().json(s);
    let spender_flags = |s: Address| spender_info.get(&s).map(|i| i.flags()).unwrap_or_default();

    // ERC-20 approvals, sorted by exposure
    let mut erc20_rows: Vec<(f64, f64, serde_json::Value)> = open_erc20
        .iter()
        .map(|((token, spender), block, allowance)| {
            let info = token_info.get(token).cloned().unwrap_or_default();
            let resolved = resolve_read(
                erc20_at_risk(token, *allowance),
                unread_targets.contains(token),
                code_of(token),
            );
            let at_risk = resolved.value();
            let whole = at_risk.map_or(0.0, |r| to_float(r, info.decimals.unwrap_or(0)));
            let price = prices.get(token).copied();
            let usd = at_risk
                .and(price)
                .map(|p| whole * p.usd)
                .filter(|u| u.is_finite());
            let unlimited = allowance.is_some_and(is_unlimited);
            let mut flags = spender_flags(*spender);
            if unlimited {
                flags.insert(0, "unlimited");
            }
            let row = serde_json::json!({
                "token": token_json(*token),
                "spender": spender_entry(*spender),
                "allowance": allowance.map(|a| a.to_string()),
                "allowance_formatted": if unlimited { Some("unlimited".to_string()) } else { allowance.and_then(|a| amount(a, info.decimals)) },
                "unlimited": unlimited,
                "balance": info.balance.map(|b| b.to_string()),
                "balance_formatted": info.balance.and_then(|b| amount(b, info.decimals)),
                "exposure": at_risk.map(|r| r.to_string()),
                "exposure_formatted": at_risk.and_then(|r| amount(r, info.decimals)),
                "exposure_usd": usd,
                "state_unknown": resolved == Resolved::Unknown,
                "token_kind": (resolved == Resolved::NoCode).then_some("no_code"),
                "non_standard_token": resolved == Resolved::NonStandard,
                "price_usd": price.map(|p| p.usd),
                "price_confidence": price.map(|p| p.confidence),
                "price_source": price.map(|_| "llama"),
                "last_approval_block": block,
                "flags": flags,
            });
            (usd.unwrap_or(-1.0), whole, row)
        })
        .collect();
    erc20_rows.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Permit2 allowances: limited by the token's allowance to Permit2 too
    let mut permit2_rows: Vec<(f64, f64, serde_json::Value)> = open_permit2
        .iter()
        .map(|((token, spender), block, state)| {
            let info = token_info.get(token).cloned().unwrap_or_default();
            let resolved = resolve_read(
                permit2_at_risk(token, *state),
                unread_targets.contains(token) || unread_targets.contains(&PERMIT2),
                code_of(token),
            );
            let at_risk = resolved.value();
            let whole = at_risk.map_or(0.0, |r| to_float(r, info.decimals.unwrap_or(0)));
            let price = prices.get(token).copied();
            let usd = at_risk
                .and(price)
                .map(|p| whole * p.usd)
                .filter(|u| u.is_finite());
            let amt = state.map(|s| s.0);
            let expiration = state.map(|s| s.1);
            let expired = expiration.map(|e| permit2_expired(e, now));
            let cap = permit2_erc20_allowance.get(token).copied().flatten();
            let unlimited = amt.is_some_and(is_unlimited);
            let mut flags = spender_flags(*spender);
            if unlimited {
                flags.insert(0, "unlimited");
            }
            if expired == Some(true) {
                flags.push("expired");
            }
            let row = serde_json::json!({
                "token": token_json(*token),
                "spender": spender_entry(*spender),
                "amount": amt.map(|a| a.to_string()),
                "amount_formatted": if unlimited { Some("unlimited".to_string()) } else { amt.and_then(|a| amount(a, info.decimals)) },
                "unlimited": unlimited,
                "expiration": expiration,
                "expiration_time": expiration.map(crate::rpc::block_time::format_timestamp),
                "expired": expired,
                "nonce": state.map(|s| s.2),
                "token_allowance_to_permit2": cap.map(|c| c.to_string()),
                "exposure": at_risk.map(|r| r.to_string()),
                "exposure_formatted": at_risk.and_then(|r| amount(r, info.decimals)),
                "exposure_usd": usd,
                "state_unknown": resolved == Resolved::Unknown,
                "token_kind": (resolved == Resolved::NoCode).then_some("no_code"),
                "non_standard_token": resolved == Resolved::NonStandard,
                "price_usd": price.map(|p| p.usd),
                "price_confidence": price.map(|p| p.confidence),
                "price_source": price.map(|_| "llama"),
                "last_event_block": block,
                "flags": flags,
            });
            (usd.unwrap_or(-1.0), whole, row)
        })
        .collect();
    permit2_rows.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal))
    });

    // Operator (setApprovalForAll) approvals, most items held first
    let mut operator_rows: Vec<(U256, serde_json::Value)> = open_operators
        .iter()
        .map(|((collection, operator), block, approved)| {
            let resolved = resolve_read(
                approved.map(|a| U256::from(a as u8)),
                unread_targets.contains(collection),
                code_of(collection),
            );
            let held = if resolved == Resolved::NoCode {
                Some(U256::ZERO)
            } else {
                collection_balance.get(collection).copied().flatten()
            };
            let mut flags = vec!["all_items"];
            flags.extend(spender_flags(*operator));
            (
                held.unwrap_or_default(),
                serde_json::json!({
                    "collection": collection.to_checksum(None),
                    "operator": spender_entry(*operator),
                    "items_held": held.map(|h| h.to_string()),
                    "state_unknown": resolved == Resolved::Unknown,
                    "token_kind": (resolved == Resolved::NoCode).then_some("no_code"),
                    "non_standard_token": resolved == Resolved::NonStandard,
                    "last_approval_block": block,
                    "flags": flags,
                }),
            )
        })
        .collect();
    operator_rows.sort_by_key(|r| std::cmp::Reverse(r.0));

    let spenders_delegated = spender_info
        .values()
        .any(|i| i.kind == "eoa_7702_delegated");
    let (total_usd, spam_usd, low_confidence_usd) =
        usd_totals(erc20_rows.iter().chain(permit2_rows.iter()).map(|r| &r.2));
    let top = top_exposures(
        erc20_rows.iter().chain(permit2_rows.iter()).map(|r| &r.2),
        5,
    );
    let approvals_sum_usd =
        per_approval_sum(erc20_rows.iter().chain(permit2_rows.iter()).map(|r| &r.2));
    let mut hidden = 0usize;
    if opts.exposed_only {
        for rows in [&mut erc20_rows, &mut permit2_rows] {
            let before = rows.len();
            rows.retain(|r| r.2["exposure"] != "0");
            hidden += before - rows.len();
        }
        let before = operator_rows.len();
        operator_rows.retain(|r| r.1["items_held"] != "0");
        hidden += before - operator_rows.len();
    }
    let state_unknown = erc20_rows
        .iter()
        .chain(permit2_rows.iter())
        .map(|r| &r.2)
        .chain(operator_rows.iter().map(|r| &r.1))
        .filter(|r| r["state_unknown"] == true)
        .count();
    let scans_complete = scans.iter().all(|s| s["complete"] == true);
    let complete = scans_complete && state_unknown == 0 && !read.capped;
    let mut notes = vec![
        format!(
            "total_exposure_usd leaves out tokens flagged spam (excluded_spam_usd) and prices \
             with confidence under {MIN_TOTAL_CONFIDENCE} (excluded_low_confidence_usd)"
        ),
        "Pairs come from the owner's Approval, ApprovalForAll and Permit2 events; an \
         approval that emitted no event is not found"
            .to_string(),
        "ERC-721 single-token approvals (Approval with a token id) are not listed".to_string(),
    ];
    notes.extend(source_notes);
    if !scans_complete {
        notes.push(
            "The event scan was cut short (see scan); later approvals may be missing. \
             Raise --max-pages or set --from-block"
                .to_string(),
        );
    }
    if opts.price && exposed_tokens.len() > MAX_PRICED_TOKENS {
        notes.push(format!(
            "Spam checks ran for {MAX_PRICED_TOKENS} of {} exposed tokens: known tokens first, then the largest priced holdings",
            exposed_tokens.len()
        ));
    }
    if opts.from_block > 0 {
        notes.push(format!(
            "Events before block {} were not scanned; older approvals are not listed",
            opts.from_block
        ));
    }
    if state_unknown > 0 {
        notes.push(format!(
            "{state_unknown} approvals have state_unknown: a value they need could not be \
             read, so their exposure is null; they are kept even with --exposed-only"
        ));
    }
    let non_standard = erc20_rows
        .iter()
        .chain(permit2_rows.iter())
        .map(|r| &r.2)
        .chain(operator_rows.iter().map(|r| &r.1))
        .filter(|r| r["non_standard_token"] == true)
        .count();
    if non_standard > 0 {
        notes.push(format!(
            "{non_standard} approvals are on contracts that have code but did not answer like a \
             token (non_standard_token): exposure is null and they are kept visible, but they \
             do not make the report incomplete"
        ));
    }
    if !read.reverted.is_empty() {
        notes.push(format!(
            "{} contracts failed when called on their own (revert or gas exhaustion): see \
             unreadable_contracts",
            read.reverted.len()
        ));
    }
    if !read.transport_failed.is_empty() {
        notes.push(format!(
            "{} contracts could not be read because of RPC errors that persisted after \
             retries: see rpc_failed_contracts",
            read.transport_failed.len()
        ));
    }
    if read.capped {
        notes.push(format!(
            "State reads stopped at {MAX_SPLIT_REQUESTS} requests while isolating failing \
             calls; {unknown_reads} calls were not read"
        ));
    }
    if spenders_delegated {
        notes.push(
            "A spender flagged delegated_eoa is an EIP-7702 delegated EOA: its key can move \
             approved tokens whatever code it delegates to"
                .to_string(),
        );
    }

    Ok(serde_json::json!({
        "owner": owner.to_checksum(None),
        "chain": chain.name(),
        "block": latest,
        "timestamp": now,
        "complete": complete,
        "scan": scans,
        "eip7702_delegation": delegation.map(spender_entry),
        "candidates": {
            "erc20": candidates.erc20.len(),
            "operators": candidates.operators.len(),
            "permit2": candidates.permit2.len(),
        },
        "total_exposure_usd": total_usd,
        "top_exposures": top,
        "per_approval_sum_usd": approvals_sum_usd,
        "excluded_spam_usd": spam_usd,
        "excluded_low_confidence_usd": low_confidence_usd,
        "hidden_zero_exposure": hidden,
        "state_unknown": state_unknown,
        "state_reads": {
            "requests": read.requests,
            "capped": read.capped,
            "unread_calls": unknown_reads,
        },
        "unreadable_contracts": read
            .reverted
            .iter()
            .map(|a| a.to_checksum(None))
            .collect::<Vec<_>>(),
        "rpc_failed_contracts": read
            .transport_failed
            .iter()
            .map(|a| a.to_checksum(None))
            .collect::<Vec<_>>(),
        "approvals": erc20_rows.into_iter().map(|r| r.2).collect::<Vec<_>>(),
        "permit2": permit2_rows.into_iter().map(|r| r.2).collect::<Vec<_>>(),
        "operator_approvals": operator_rows.into_iter().map(|r| r.1).collect::<Vec<_>>(),
        "notes": notes,
    }))
}

/// Human-readable report
pub fn format_approvals(report: &serde_json::Value) -> String {
    use crate::abi::revert::safe_text;
    use std::fmt::Write;
    let text = |v: &serde_json::Value| v.as_str().map(safe_text).unwrap_or_default();
    let spender = |s: &serde_json::Value| {
        let address = text(&s["address"]);
        let name = s["label"].as_str().or(s["contract_name"].as_str());
        match name {
            Some(name) => format!("{} ({address})", safe_text(name)),
            None => address,
        }
    };
    let flags = |row: &serde_json::Value| {
        row["flags"]
            .as_array()
            .map(|f| {
                f.iter()
                    .filter_map(|x| x.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    };
    let usd = |v: &serde_json::Value| {
        v.as_f64()
            .map(crate::utils::table::format_usd)
            .unwrap_or_else(|| "-".to_string())
    };
    let symbol = |row: &serde_json::Value| {
        let name = row["token"]["symbol"]
            .as_str()
            .map(safe_text)
            .unwrap_or_else(|| text(&row["token"]["address"]));
        if row["token"]["spam"] == true {
            format!("{name} (spam)")
        } else {
            name
        }
    };

    let mut out = String::new();
    let _ = writeln!(
        out,
        "Approvals for {} on {} (block {})",
        text(&report["owner"]),
        text(&report["chain"]),
        report["block"]
    );
    if let Some(d) = report["eip7702_delegation"].as_object() {
        let d = serde_json::Value::Object(d.clone());
        let _ = writeln!(out, "EIP-7702: delegated to {}", spender(&d));
    }
    let _ = writeln!(out, "{}", "═".repeat(70));

    let erc20 = report["approvals"].as_array().cloned().unwrap_or_default();
    let _ = writeln!(out, "\nERC-20 approvals ({}):", erc20.len());
    for row in &erc20 {
        let _ = writeln!(
            out,
            "  {} → {}\n    allowance {}  balance {}  at risk {} {} ({}){}",
            symbol(row),
            spender(&row["spender"]),
            text(&row["allowance_formatted"]),
            text(&row["balance_formatted"]),
            text(&row["exposure_formatted"]),
            symbol(row),
            usd(&row["exposure_usd"]),
            match flags(row) {
                f if f.is_empty() => String::new(),
                f => format!("  [{f}]"),
            }
        );
    }

    let p2 = report["permit2"].as_array().cloned().unwrap_or_default();
    if !p2.is_empty() {
        let _ = writeln!(out, "\nPermit2 allowances ({}):", p2.len());
        for row in &p2 {
            let _ = writeln!(
                out,
                "  {} → {}\n    amount {}  expires {}{}  at risk {} ({}){}",
                symbol(row),
                spender(&row["spender"]),
                text(&row["amount_formatted"]),
                text(&row["expiration_time"]),
                if row["expired"] == true {
                    " (expired)"
                } else {
                    ""
                },
                text(&row["exposure_formatted"]),
                usd(&row["exposure_usd"]),
                match flags(row) {
                    f if f.is_empty() => String::new(),
                    f => format!("  [{f}]"),
                }
            );
        }
    }

    let ops = report["operator_approvals"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    if !ops.is_empty() {
        let _ = writeln!(out, "\nNFT operator approvals ({}):", ops.len());
        for row in &ops {
            let _ = writeln!(
                out,
                "  {} → {}  items held {}  [{}]",
                text(&row["collection"]),
                spender(&row["operator"]),
                row["items_held"].as_str().unwrap_or("?"),
                flags(row)
            );
        }
    }

    let _ = writeln!(
        out,
        "\nTotal priced exposure: {} (excluded: spam {}, low-confidence prices {})",
        usd(&report["total_exposure_usd"]),
        usd(&report["excluded_spam_usd"]),
        usd(&report["excluded_low_confidence_usd"])
    );
    let _ = writeln!(out, "\nScan:");
    for scan in report["scan"].as_array().into_iter().flatten() {
        let _ = writeln!(
            out,
            "  {:<17} blocks {}..{}  {} logs  {}",
            text(&scan["event"]),
            scan["from_block"],
            scan["scanned_to_block"],
            scan["logs"],
            if scan["complete"] == true {
                "complete".to_string()
            } else {
                format!("TRUNCATED ({})", text(&scan["stopped"]))
            }
        );
    }
    for note in report["notes"].as_array().into_iter().flatten() {
        let _ = writeln!(out, "Note: {}", text(note));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn owner_topic(a: Address) -> String {
        format!("{:#x}", B256::left_padding_from(a.as_slice()))
    }

    #[test]
    fn parses_explorer_log_rows() {
        let owner = Address::repeat_byte(0x11);
        let spender = Address::repeat_byte(0x22);
        let row = serde_json::json!({
            "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
            "topics": [format!("{:#x}", Approval::SIGNATURE_HASH), owner_topic(owner), owner_topic(spender), ""],
            "data": "0x01",
            "blockNumber": "0x10",
        });
        let parsed = parse_log_row(&row).unwrap();
        assert_eq!(parsed.block, 16);
        assert_eq!(parsed.topics.len(), 3);
        assert_eq!(topic_address(&parsed.topics[2]), spender);
        assert!(parse_log_row(&serde_json::json!({"address": "nope"})).is_none());
    }

    #[test]
    fn candidates_keep_erc20_pairs_and_skip_nft_token_approvals() {
        let token = Address::repeat_byte(0xaa);
        let owner = Address::repeat_byte(0x11);
        let spender = Address::repeat_byte(0x22);
        let t = |a: Address| B256::left_padding_from(a.as_slice());
        let rows = vec![
            LogRow {
                address: token,
                topics: vec![Approval::SIGNATURE_HASH, t(owner), t(spender)],
                block: 5,
            },
            LogRow {
                address: token,
                topics: vec![Approval::SIGNATURE_HASH, t(owner), t(spender)],
                block: 9,
            },
            // ERC-721 single-token approval: four topics
            LogRow {
                address: Address::repeat_byte(0xbb),
                topics: vec![Approval::SIGNATURE_HASH, t(owner), t(spender), B256::ZERO],
                block: 7,
            },
        ];
        let mut c = Candidates::default();
        c.add_approval_logs(&rows);
        assert_eq!(c.erc20.len(), 1);
        assert_eq!(c.erc20[&(token, spender)], 9);

        let p2 = vec![LogRow {
            address: PERMIT2,
            topics: vec![
                permit2::Permit::SIGNATURE_HASH,
                t(owner),
                t(token),
                t(spender),
            ],
            block: 3,
        }];
        c.add_permit2_logs(&p2);
        assert_eq!(c.permit2[&(token, spender)], 3);
    }

    #[test]
    fn exposure_is_capped_by_balance_and_allowance() {
        assert_eq!(exposure(U256::MAX, U256::from(5)), U256::from(5));
        assert_eq!(exposure(U256::from(3), U256::from(5)), U256::from(3));
        assert_eq!(exposure(U256::from(3), U256::ZERO), U256::ZERO);
    }

    #[test]
    fn unlimited_values() {
        let max_of = |bits: usize| (U256::from(1u8) << bits) - U256::from(1u8);
        assert!(is_unlimited(U256::MAX));
        assert!(is_unlimited(max_of(160)));
        assert!(is_unlimited(max_of(96)));
        assert!(is_unlimited(U256::from(1u8) << 255));
        assert!(!is_unlimited(U256::from(10u64.pow(18))));
    }

    #[test]
    fn permit2_expiry_and_7702() {
        assert!(!permit2_expired(100, 100));
        assert!(permit2_expired(99, 100));
        let target = Address::repeat_byte(0x42);
        let mut code = vec![0xef, 0x01, 0x00];
        code.extend_from_slice(target.as_slice());
        assert_eq!(eip7702_delegate(&code), Some(target));
        assert_eq!(eip7702_delegate(&code[..22]), None);
        assert_eq!(eip7702_delegate(&[0x60, 0x80]), None);
    }

    #[test]
    fn totals_leave_out_spam_and_low_confidence_prices() {
        let t = |a: &str| serde_json::json!({"address": a, "spam": false});
        let rows = [
            serde_json::json!({"token": t("0xa"), "exposure_usd": 100.0, "price_confidence": 0.99}),
            serde_json::json!({"token": {"address": "0xb", "spam": true}, "exposure_usd": 5000.0, "price_confidence": 0.99}),
            serde_json::json!({"token": t("0xc"), "exposure_usd": 70.0, "price_confidence": 0.5}),
            serde_json::json!({"token": t("0xd"), "exposure_usd": null}),
            // The same balance approved to other spenders: counted once
            serde_json::json!({"token": t("0xA"), "exposure_usd": 100.0, "price_confidence": 0.99}),
            serde_json::json!({"token": t("0xa"), "exposure_usd": 40.0, "price_confidence": 0.99}),
        ];
        assert_eq!(usd_totals(rows.iter()), (100.0, 5000.0, 70.0));
        assert_eq!(per_approval_sum(rows.iter()), 5310.0);
    }

    #[test]
    fn lure_symbols_are_flagged_and_cut() {
        let lure = "Visit https://x.example to claim";
        assert!(symbol_suspicious(lure));
        assert!(symbol_suspicious("USDC\u{202e}"));
        for ok in ["USDC", "UNI-V2", "stETH", "yvCurve-3pool", "шайлушай"] {
            assert!(!symbol_suspicious(ok), "{ok}");
        }
        let long = "A".repeat(100);
        let cut = clean_text(&long, MAX_SYMBOL_CHARS);
        assert_eq!(cut.chars().count(), MAX_SYMBOL_CHARS);
        assert!(cut.ends_with('…'));
        assert_eq!(clean_text("ab\ncd", 32), "ab\\u{a}cd");
        assert_eq!(clean_text(lure, MAX_SYMBOL_CHARS), lure);
    }

    #[tokio::test]
    async fn a_failing_call_is_isolated_and_the_rest_are_read() {
        let bad = Address::repeat_byte(0xbd);
        let calls: Vec<(Address, Bytes)> = (0..10u8)
            .map(|i| {
                let target = if i == 6 {
                    bad
                } else {
                    Address::repeat_byte(i + 1)
                };
                (target, Bytes::from(vec![i]))
            })
            .collect();
        let attempts = std::sync::atomic::AtomicUsize::new(0);
        // Like a node where one gas-burning call sinks the whole aggregate3
        let exec = |batch: Vec<(Address, Bytes)>, _retries: u32| {
            attempts.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            async move {
                if batch.iter().any(|(t, _)| *t == bad) {
                    anyhow::bail!("out of gas");
                }
                Ok(batch
                    .into_iter()
                    .map(|(_, data)| MulticallResult {
                        success: true,
                        data,
                    })
                    .collect())
            }
        };
        let read = run_split(calls, MAX_SPLIT_REQUESTS, exec).await.unwrap();
        assert_eq!(read.results.len(), 10);
        assert_eq!(read.reverted.iter().copied().collect::<Vec<_>>(), vec![bad]);
        assert!(read.transport_failed.is_empty());
        assert!(!read.capped);
        for (i, r) in read.results.iter().enumerate() {
            if i == 6 {
                assert!(r.is_none(), "the bad call is unread, not a zero");
            } else {
                let r = r.as_ref().unwrap();
                assert!(r.success);
                assert_eq!(r.data.as_ref(), &[i as u8]);
            }
        }
        // Halving, not one call per candidate
        assert!(attempts.load(std::sync::atomic::Ordering::Relaxed) < 10);
    }

    #[tokio::test]
    async fn an_all_failing_node_is_an_error() {
        let calls = vec![
            (Address::repeat_byte(1), Bytes::new()),
            (Address::repeat_byte(2), Bytes::new()),
        ];
        let exec = |_: Vec<(Address, Bytes)>, _: u32| async { anyhow::bail!("connection reset") };
        assert!(run_split(calls, MAX_SPLIT_REQUESTS, exec).await.is_err());
    }

    #[tokio::test]
    async fn rpc_errors_are_kept_apart_from_reverts_and_retried() {
        let flaky = Address::repeat_byte(0xee);
        let calls: Vec<(Address, Bytes)> = (0..4u8)
            .map(|i| {
                let target = if i == 1 {
                    flaky
                } else {
                    Address::repeat_byte(i + 1)
                };
                (target, Bytes::from(vec![i]))
            })
            .collect();
        let single_retries = std::sync::Mutex::new(Vec::new());
        let exec = |batch: Vec<(Address, Bytes)>, retries: u32| {
            if batch.len() == 1 {
                single_retries.lock().unwrap().push(retries);
            }
            async move {
                if batch.iter().any(|(t, _)| *t == flaky) {
                    anyhow::bail!("429 Too Many Requests");
                }
                Ok(batch
                    .into_iter()
                    .map(|(_, data)| MulticallResult {
                        success: true,
                        data,
                    })
                    .collect())
            }
        };
        let read = run_split(calls, MAX_SPLIT_REQUESTS, exec).await.unwrap();
        assert!(read.reverted.is_empty());
        assert_eq!(
            read.transport_failed.iter().copied().collect::<Vec<_>>(),
            vec![flaky]
        );
        assert!(read.results[1].is_none());
        assert!(read.results[0].is_some() && read.results[2].is_some());
        assert!(single_retries
            .lock()
            .unwrap()
            .iter()
            .all(|r| *r == SINGLE_CALL_RETRIES));
    }

    #[tokio::test]
    async fn splitting_stops_at_the_request_cap() {
        let calls: Vec<(Address, Bytes)> = (0..64u8)
            .map(|i| (Address::repeat_byte(i + 1), Bytes::new()))
            .collect();
        // Every batch and every call fails as a revert: the worst case
        let exec = |_: Vec<(Address, Bytes)>, _: u32| async { anyhow::bail!("execution reverted") };
        let read = run_split(calls, 10, exec).await.unwrap();
        assert!(read.capped);
        assert_eq!(read.requests, 10);
        assert!(read.results.iter().all(Option::is_none));
    }

    #[test]
    fn execution_errors_are_told_apart_from_rpc_errors() {
        for text in [
            "execution reverted",
            "out of gas",
            "execution aborted (timeout = 5s)",
        ] {
            assert!(
                is_execution_error(&anyhow::anyhow!("Multicall failed: {text}")),
                "{text}"
            );
        }
        for text in [
            "429 Too Many Requests",
            "connection reset",
            "request timed out",
        ] {
            assert!(
                !is_execution_error(&anyhow::anyhow!("Multicall failed: {text}")),
                "{text}"
            );
        }
    }

    #[test]
    fn top_exposures_lists_counted_tokens_largest_first() {
        let row = |a: &str, usd: f64, conf: f64, spam: bool| {
            serde_json::json!({"token": {"address": a, "symbol": a, "spam": spam},
                "exposure_usd": usd, "price_confidence": conf, "price_source": "llama"})
        };
        let rows = [
            row("0xa", 10.0, 0.99, false),
            row("0xa", 30.0, 0.99, false),
            row("0xb", 500.0, 0.99, true),
            row("0xc", 400.0, 0.5, false),
            row("0xd", 20.0, 0.95, false),
        ];
        let top = top_exposures(rows.iter(), 5);
        let got: Vec<(&str, f64)> = top
            .iter()
            .map(|t| {
                (
                    t["token"].as_str().unwrap(),
                    t["exposure_usd"].as_f64().unwrap(),
                )
            })
            .collect();
        assert_eq!(got, vec![("0xa", 30.0), ("0xd", 20.0)]);
    }

    #[test]
    fn empty_answers_are_classified_by_code() {
        let v = U256::from(7u8);
        assert_eq!(resolve_read(Some(v), false, None), Resolved::Known(v));
        // Answered, no code: nothing to move, a known zero
        assert_eq!(resolve_read(None, false, Some(false)), Resolved::NoCode);
        assert_eq!(Resolved::NoCode.value(), Some(U256::ZERO));
        // Answered, has code, not a token: visible, value unknown
        assert_eq!(resolve_read(None, false, Some(true)), Resolved::NonStandard);
        assert_eq!(Resolved::NonStandard.value(), None);
        // Never read, or the code check failed: unknown
        assert_eq!(resolve_read(None, true, Some(false)), Resolved::Unknown);
        assert_eq!(resolve_read(None, false, None), Resolved::Unknown);
    }

    #[test]
    fn delegated_eoa_spenders_count_as_eoas() {
        let info = SpenderInfo {
            kind: "eoa_7702_delegated",
            ..Default::default()
        };
        assert_eq!(info.flags(), vec!["eoa_spender", "delegated_eoa"]);
    }

    #[test]
    fn float_amounts() {
        assert_eq!(to_float(U256::from(1_500_000u64), 6), 1.5);
        assert_eq!(to_float(U256::ZERO, 18), 0.0);
        // Large amounts have no thousands separators to trip the parse
        assert_eq!(
            to_float(U256::from(30_000_000_000u64) * U256::from(10u64.pow(9)), 9),
            3e10
        );
    }
}
