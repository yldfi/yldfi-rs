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
async fn multicall_all<P: Provider>(
    provider: &P,
    calls: Vec<(Address, Bytes)>,
) -> anyhow::Result<Vec<MulticallResult>> {
    let mut results = Vec::with_capacity(calls.len());
    for chunk in calls.chunks(CALLS_PER_BATCH) {
        let mut builder = MulticallBuilder::new();
        for (target, data) in chunk {
            builder = builder.add_call_allow_failure(*target, data.clone());
        }
        let batch = builder.execute_with_retry(provider, 3).await?;
        anyhow::ensure!(
            batch.len() == chunk.len(),
            "Multicall returned {} results for {} calls",
            batch.len(),
            chunk.len()
        );
        results.extend(batch);
    }
    Ok(results)
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
        if self.kind == "eoa" {
            flags.push("eoa_spender");
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
pub(crate) fn usd_totals<'a>(rows: impl Iterator<Item = &'a serde_json::Value>) -> (f64, f64, f64) {
    let (mut counted, mut spam, mut low) = (0.0, 0.0, 0.0);
    for row in rows {
        let Some(usd) = row["exposure_usd"].as_f64() else {
            continue;
        };
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

async fn usd_prices(chain: Chain, tokens: &[Address]) -> HashMap<Address, Price> {
    use crate::aggregator::price::fetch_prices_parallel;
    use crate::aggregator::PriceSource;
    let lookups = tokens
        .iter()
        .take(MAX_PRICED_TOKENS)
        .map(|token| async move {
            let text = format!("{token:#x}");
            let result = fetch_prices_parallel(&text, chain.name(), &[PriceSource::Llama]).await;
            let price = result
                .sources
                .iter()
                .filter_map(|s| s.data.as_ref())
                .find(|p| p.usd.is_finite() && p.usd > 0.0)
                .map(|p| Price {
                    usd: p.usd,
                    confidence: p.confidence.unwrap_or(1.0),
                });
            price.map(|p| (*token, p))
        });
    futures::future::join_all(lookups)
        .await
        .into_iter()
        .flatten()
        .collect()
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
    if !opts.quiet {
        eprintln!(
            "Reading current state for {} candidate approvals...",
            candidates.erc20.len() + candidates.operators.len() + candidates.permit2.len()
        );
    }
    let results = multicall_all(provider, calls).await?;
    let mut results = results.into_iter();
    let mut next = || {
        results.next().unwrap_or(MulticallResult {
            success: false,
            data: Bytes::new(),
        })
    };

    let now = next()
        .decode_uint256()
        .map(|t| t.saturating_to::<u64>())
        .unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_secs())
        });
    let mut token_info: HashMap<Address, TokenInfo> = HashMap::new();
    for token in &tokens {
        let symbol = next().decode_string();
        let decimals = next().decode_uint8();
        let balance = next().decode_uint256();
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
        .map(|(pair, block)| (*pair, *block, next().decode_uint256()))
        .collect();
    let collection_balance: HashMap<Address, Option<U256>> = collections
        .iter()
        .map(|c| (*c, next().decode_uint256()))
        .collect();
    let operator_state: Vec<((Address, Address), u64, Option<bool>)> = candidates
        .operators
        .iter()
        .map(|(pair, block)| {
            let r = next();
            (*pair, *block, r.decode_uint256().map(|v| !v.is_zero()))
        })
        .collect();
    #[allow(clippy::type_complexity)]
    let permit2_state: Vec<((Address, Address), u64, Option<(U256, u64, u64)>)> = candidates
        .permit2
        .iter()
        .map(|(pair, block)| {
            let r = next();
            let decoded = r
                .success
                .then(|| permit2::allowanceCall::abi_decode_returns(&r.data).ok())
                .flatten()
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

    // Keep the open ones
    let open_erc20: Vec<_> = erc20_state
        .into_iter()
        .filter_map(|(pair, block, allowance)| {
            allowance.filter(|a| !a.is_zero()).map(|a| (pair, block, a))
        })
        .collect();
    let open_operators: Vec<_> = operator_state
        .into_iter()
        .filter(|(_, _, approved)| *approved == Some(true))
        .collect();
    let open_permit2: Vec<_> = permit2_state
        .into_iter()
        .filter_map(|(pair, block, state)| {
            state
                .filter(|(amount, _, _)| !amount.is_zero())
                .map(|s| (pair, block, s))
        })
        .collect();
    let permit2_erc20_allowance: HashMap<Address, U256> = open_erc20
        .iter()
        .filter(|((_, spender), _, _)| *spender == PERMIT2)
        .map(|((token, _), _, a)| (*token, *a))
        .collect();

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
            let balance = token_info.get(token).and_then(|t| t.balance);
            if balance.is_some_and(|b| !exposure(*allowance, b).is_zero()) {
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
            let balance = token_info.get(token).and_then(|t| t.balance);
            if balance.is_some_and(|b| !exposure(*allowance, b).is_zero()) {
                set.insert(*token);
            }
        }
        set.into_iter().collect()
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
        spam_checks(chain, &exposed_tokens).await
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
            let balance = info.balance.unwrap_or_default();
            let at_risk = exposure(*allowance, balance);
            let decimals = info.decimals.unwrap_or(0);
            let whole = to_float(at_risk, decimals);
            let price = prices.get(token).copied();
            let usd = price.map(|p| whole * p.usd).filter(|u| u.is_finite());
            let mut flags = spender_flags(*spender);
            if is_unlimited(*allowance) {
                flags.insert(0, "unlimited");
            }
            let row = serde_json::json!({
                "token": token_json(*token),
                "spender": spender_entry(*spender),
                "allowance": allowance.to_string(),
                "allowance_formatted": if is_unlimited(*allowance) { Some("unlimited".to_string()) } else { amount(*allowance, info.decimals) },
                "unlimited": is_unlimited(*allowance),
                "balance": info.balance.map(|b| b.to_string()),
                "balance_formatted": info.balance.and_then(|b| amount(b, info.decimals)),
                "exposure": at_risk.to_string(),
                "exposure_formatted": amount(at_risk, info.decimals),
                "exposure_usd": usd,
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
        .map(|((token, spender), block, (amt, expiration, nonce))| {
            let info = token_info.get(token).cloned().unwrap_or_default();
            let balance = info.balance.unwrap_or_default();
            let expired = permit2_expired(*expiration, now);
            let erc20_to_permit2 = permit2_erc20_allowance.get(token).copied().unwrap_or_default();
            let at_risk = if expired {
                U256::ZERO
            } else {
                exposure((*amt).min(erc20_to_permit2), balance)
            };
            let decimals = info.decimals.unwrap_or(0);
            let whole = to_float(at_risk, decimals);
            let price = prices.get(token).copied();
            let usd = price.map(|p| whole * p.usd).filter(|u| u.is_finite());
            let mut flags = spender_flags(*spender);
            if is_unlimited(*amt) {
                flags.insert(0, "unlimited");
            }
            if expired {
                flags.push("expired");
            }
            let row = serde_json::json!({
                "token": token_json(*token),
                "spender": spender_entry(*spender),
                "amount": amt.to_string(),
                "amount_formatted": if is_unlimited(*amt) { Some("unlimited".to_string()) } else { amount(*amt, info.decimals) },
                "unlimited": is_unlimited(*amt),
                "expiration": expiration,
                "expiration_time": crate::rpc::block_time::format_timestamp(*expiration),
                "expired": expired,
                "nonce": nonce,
                "token_allowance_to_permit2": erc20_to_permit2.to_string(),
                "exposure": at_risk.to_string(),
                "exposure_formatted": amount(at_risk, info.decimals),
                "exposure_usd": usd,
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
        .map(|((collection, operator), block, _)| {
            let held = collection_balance.get(collection).copied().flatten();
            let mut flags = vec!["all_items"];
            flags.extend(spender_flags(*operator));
            (
                held.unwrap_or_default(),
                serde_json::json!({
                    "collection": collection.to_checksum(None),
                    "operator": spender_entry(*operator),
                    "items_held": held.map(|h| h.to_string()),
                    "last_approval_block": block,
                    "flags": flags,
                }),
            )
        })
        .collect();
    operator_rows.sort_by_key(|r| std::cmp::Reverse(r.0));

    let (total_usd, spam_usd, low_confidence_usd) =
        usd_totals(erc20_rows.iter().chain(permit2_rows.iter()).map(|r| &r.2));
    let mut hidden = 0usize;
    if opts.exposed_only {
        for rows in [&mut erc20_rows, &mut permit2_rows] {
            let before = rows.len();
            rows.retain(|r| r.2["exposure"] != "0");
            hidden += before - rows.len();
        }
    }
    let complete = scans.iter().all(|s| s["complete"] == true);
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
    if !complete {
        notes.push(
            "The event scan was cut short (see scan); later approvals may be missing. \
             Raise --max-pages or set --from-block"
                .to_string(),
        );
    }
    if opts.price && exposed_tokens.len() > MAX_PRICED_TOKENS {
        notes.push(format!(
            "Only the first {MAX_PRICED_TOKENS} exposed tokens were priced"
        ));
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
        "excluded_spam_usd": spam_usd,
        "excluded_low_confidence_usd": low_confidence_usd,
        "hidden_zero_exposure": hidden,
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
        let rows = [
            serde_json::json!({"token": {"spam": false}, "exposure_usd": 100.0, "price_confidence": 0.99}),
            serde_json::json!({"token": {"spam": true}, "exposure_usd": 5000.0, "price_confidence": 0.99}),
            serde_json::json!({"token": {"spam": false}, "exposure_usd": 70.0, "price_confidence": 0.5}),
            serde_json::json!({"token": {"spam": false}, "exposure_usd": null}),
        ];
        assert_eq!(usd_totals(rows.iter()), (100.0, 5000.0, 70.0));
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
