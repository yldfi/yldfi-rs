//! Transaction analyzer
//!
//! Coordinates fetching and analyzing Ethereum transactions.

use crate::abi::AbiFetcher;
use crate::config::Chain;
use crate::error::Result;
use crate::rpc::RpcPool;
use crate::tx::addresses::{events, get_contract_info, get_label};
use crate::tx::flow::{
    native_transfers_from_trace, parse_transfers, parse_wraps, revert_data_from_trace,
    wrapped_native_token,
};
use crate::tx::types::{
    AnalyzedEvent, BalanceChange, ContractCategory, ContractInfo, EventParam, FunctionCall,
    FunctionParam, RawTxData, TransactionAnalysis,
};
use alloy::consensus::Transaction as TxTrait;
use alloy::primitives::U256;
use alloy::primitives::{Address, B256};
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use std::sync::Arc;

/// Convert U256 to f64 with ETH decimals (HIGH-001 fix)
///
/// Handles arbitrarily large values by:
/// 1. Using u128 conversion when possible for precision
/// 2. Falling back to string conversion for very large values
/// 3. Saturating at f64::MAX to prevent overflow/panic
fn u256_to_eth_f64(value: &U256) -> f64 {
    const ETH_SCALE: f64 = 1e18;

    // Try u128 first for better precision (covers values up to ~340 undecillion wei)
    if let Ok(as_u128) = TryInto::<u128>::try_into(*value) {
        return (as_u128 as f64) / ETH_SCALE;
    }

    // For values > u128::MAX, convert via string (loses precision but handles any size)
    let s = value.to_string();
    match s.parse::<f64>() {
        Ok(v) if v.is_finite() => v / ETH_SCALE,
        _ => f64::MAX, // Saturate on overflow or parse failure
    }
}

/// Format token amount with decimals (LOW-002 fix)
///
/// Safely formats large token amounts without panicking on edge cases.
/// Returns a human-readable string with 2 decimal places.
fn format_token_amount(amount_str: &str, decimals: usize) -> String {
    // Handle empty or non-numeric input
    if amount_str.is_empty() || !amount_str.chars().all(|c| c.is_ascii_digit()) {
        return "0.00".to_string();
    }

    if decimals == 0 {
        return amount_str.to_string();
    }

    if amount_str.len() > decimals {
        let int_part = &amount_str[..amount_str.len() - decimals];
        let frac_part = &amount_str[amount_str.len() - decimals..];
        // Take first 2 decimal places
        let frac_display = if frac_part.len() >= 2 {
            &frac_part[..2]
        } else {
            frac_part
        };
        format!("{}.{}", int_part, frac_display)
    } else {
        // Amount is less than 1 whole token
        let padding = "0".repeat(decimals - amount_str.len());
        let padded = format!("{}{}", padding, amount_str);
        // Dust amounts: show two significant digits instead of "0.00"
        let first_nonzero = padded.find(|c: char| c != '0');
        let frac_display = match first_nonzero {
            Some(i) if i >= 2 => padded[..(i + 2).min(padded.len())].trim_end_matches('0'),
            _ if padded.len() >= 2 => &padded[..2],
            _ => &padded,
        };
        format!("0.{}", frac_display)
    }
}

/// Decode an ABI-encoded `uint8` return value from `decimals()`.
///
/// Returns `None` for short/empty data or values that don't fit in `u8`
/// (non-ERC20 contracts sometimes return garbage).
fn decode_decimals(data: &[u8]) -> Option<u8> {
    if data.len() < 32 {
        return None;
    }
    let word = U256::from_be_slice(&data[..32]);
    if word > U256::from(77u8) {
        // No real token uses > 77 decimals (U256::MAX has 78 digits).
        return None;
    }
    Some(word.to::<u8>())
}

/// Render a token flow amount using its decimals, or raw units if unknown.
fn format_flow_amount(amount: &str, decimals: Option<u8>) -> String {
    match decimals {
        Some(d) => format_token_amount(amount, d as usize),
        None => format!("{amount} (raw units)"),
    }
}

/// Note attached when native-token figures were worked out without a trace
const PARTIAL_NATIVE_NOTE: &str =
    "balance_changes: native-token lines are marked partial. Without a \
    call trace, transfers made inside the call are known only for wrapped-native deposits and \
    withdrawals, so a contract or account that received the native token internally can be wrong. \
    Run with --full on an endpoint that supports tracing for exact figures";
/// Note attached when a call trace supplied the native-token transfers
const TRACED_NATIVE_NOTE: &str =
    "balance_changes: native-token lines include transfers made inside the call (from a call trace)";

/// Recompute `balance_changes` and its note from the analysis as it stands
fn refresh_balance_changes(analysis: &mut TransactionAnalysis) {
    analysis.balance_changes = net_balance_changes(analysis);
    analysis
        .notes
        .retain(|note| note != PARTIAL_NATIVE_NOTE && note != TRACED_NATIVE_NOTE);
    if analysis.native_transfers.is_some() {
        analysis.notes.push(TRACED_NATIVE_NOTE.to_string());
    } else if analysis.balance_changes.iter().any(|c| c.partial) {
        analysis.notes.push(PARTIAL_NATIVE_NOTE.to_string());
    }
}

/// Net change per address and token over a transaction.
///
/// Token changes come from the decoded Transfer events, plus deposits into
/// and withdrawals from the chain's wrapped native token, which mint and
/// burn it without a Transfer event.
///
/// Native-token changes come from a call trace when the analysis has one
/// (`native_transfers`), and are then exact. Without a trace they are built
/// from the top-level value and the native side of wrapped-native deposits
/// and withdrawals; other transfers made inside the call are unknown, so
/// those lines are marked `partial`. The fee is always charged to the sender.
///
/// NFT transfers and the zero address (mints and burns) are left out, as are
/// positions that net to zero.
pub fn net_balance_changes(analysis: &TransactionAnalysis) -> Vec<BalanceChange> {
    use std::collections::BTreeMap;

    // (address, token) -> (received, sent); None is the native token
    let mut totals: BTreeMap<(Address, Option<Address>), (U256, U256)> = BTreeMap::new();
    let mut token_info: HashMap<Address, (Option<String>, Option<u8>)> = HashMap::new();
    let mut flows: Vec<(Address, Address, Option<Address>, U256)> = Vec::new();

    for flow in &analysis.token_flows {
        if flow.token_id.is_some() {
            continue;
        }
        let Ok(amount) = U256::from_str(&flow.amount) else {
            continue;
        };
        let info = token_info.entry(flow.token).or_default();
        if info.0.is_none() {
            info.0 = flow.token_label.clone();
        }
        if info.1.is_none() {
            info.1 = flow.decimals;
        }
        flows.push((flow.from, flow.to, Some(flow.token), amount));
    }

    let traced = analysis.native_transfers.is_some();
    for wrap in &analysis.wraps {
        // A wrapped token that also logs the mint or burn as a Transfer has
        // already been counted above
        let logged_as_transfer = analysis.token_flows.iter().any(|flow| {
            flow.token == wrap.token
                && flow.amount == wrap.amount.to_string()
                && if wrap.deposit {
                    flow.from.is_zero() && flow.to == wrap.account
                } else {
                    flow.to.is_zero() && flow.from == wrap.account
                }
        });
        let info = token_info.entry(wrap.token).or_default();
        if info.0.is_none() {
            info.0 = get_label(&wrap.token).map(String::from);
        }
        if info.1.is_none() {
            info.1 = Some(18);
        }
        let (token_from, token_to) = if wrap.deposit {
            (Address::ZERO, wrap.account)
        } else {
            (wrap.account, Address::ZERO)
        };
        if !logged_as_transfer {
            flows.push((token_from, token_to, Some(wrap.token), wrap.amount));
        }
        // The native side: into the token contract on a deposit, out of it
        // on a withdrawal. A trace already contains these transfers.
        if !traced {
            let (native_from, native_to) = if wrap.deposit {
                (wrap.account, wrap.token)
            } else {
                (wrap.token, wrap.account)
            };
            flows.push((native_from, native_to, None, wrap.amount));
        }
    }

    match &analysis.native_transfers {
        Some(transfers) => {
            for (from, to, amount) in transfers {
                flows.push((*from, *to, None, *amount));
            }
        }
        None => {
            if analysis.status && !analysis.value.is_zero() {
                if let Some(to) = analysis.to {
                    flows.push((analysis.from, to, None, analysis.value));
                }
            }
        }
    }

    for (from, to, token, amount) in flows {
        let received = &mut totals.entry((to, token)).or_default().0;
        *received = received.saturating_add(amount);
        let sent = &mut totals.entry((from, token)).or_default().1;
        *sent = sent.saturating_add(amount);
    }
    if let Some(fee) = analysis
        .fee_wei
        .as_deref()
        .and_then(|f| U256::from_str(f).ok())
    {
        let sent = &mut totals.entry((analysis.from, None)).or_default().1;
        *sent = sent.saturating_add(fee);
    }

    totals
        .into_iter()
        .filter(|((address, _), (received, sent))| !address.is_zero() && received != sent)
        .map(|((address, token), (received, sent))| {
            let (sign, magnitude) = if received > sent {
                ("+", received - sent)
            } else {
                ("-", sent - received)
            };
            let (symbol, decimals) = match token {
                Some(token) => token_info.get(&token).cloned().unwrap_or_default(),
                None => (None, Some(18)),
            };
            let formatted = decimals.and_then(|d| {
                alloy::primitives::utils::format_units(magnitude, d)
                    .ok()
                    .map(|text| format!("{sign}{text}"))
            });
            BalanceChange {
                address,
                label: get_label(&address).map(String::from),
                token,
                symbol,
                decimals,
                raw: format!("{sign}{magnitude}"),
                formatted,
                partial: token.is_none() && !traced,
            }
        })
        .collect()
}

/// The value moved by a transaction as a Mermaid flowchart: one node per
/// address, one edge per transfer, labelled with the amount and token.
///
/// Edges are the token transfers, wrapped-native deposits and withdrawals,
/// and the native-token transfers: from the call trace when there is one,
/// otherwise only the top-level value. Labels and symbols are escaped, since
/// they can come from the contracts themselves.
pub fn flow_mermaid(analysis: &TransactionAnalysis) -> String {
    use crate::abi::revert::safe_text;
    use std::fmt::Write;

    // The sender is always the first node, so diagrams read from it
    let mut nodes: Vec<Address> = vec![analysis.from];
    let mut node = |address: Address| match nodes.iter().position(|a| *a == address) {
        Some(index) => index,
        None => {
            nodes.push(address);
            nodes.len() - 1
        }
    };
    let amount = |raw: &str, decimals: Option<u8>| match (raw.parse::<U256>(), decimals) {
        (Ok(value), Some(d)) => alloy::primitives::utils::format_units(value, d)
            .map(|text| {
                let trimmed = text.trim_end_matches('0').trim_end_matches('.');
                if trimmed.is_empty() {
                    "0".to_string()
                } else {
                    trimmed.to_string()
                }
            })
            .unwrap_or_else(|_| raw.to_string()),
        _ => raw.to_string(),
    };

    let mut edges: Vec<(usize, usize, String)> = Vec::new();
    for flow in &analysis.token_flows {
        let symbol = flow
            .token_label
            .as_deref()
            .map(safe_text)
            .unwrap_or_else(|| short_address(&flow.token));
        let label = match &flow.token_id {
            Some(id) => format!("{symbol} #{}", safe_text(id)),
            None => format!("{} {symbol}", amount(&flow.amount, flow.decimals)),
        };
        edges.push((node(flow.from), node(flow.to), label));
    }
    for wrap in &analysis.wraps {
        let symbol = get_label(&wrap.token)
            .map(String::from)
            .unwrap_or_else(|| short_address(&wrap.token));
        let value = amount(&wrap.amount.to_string(), Some(18));
        let (from, to, what) = if wrap.deposit {
            (
                wrap.account,
                wrap.token,
                format!("{value} native, wrapped to {symbol}"),
            )
        } else {
            (
                wrap.token,
                wrap.account,
                format!("{value} native, unwrapped from {symbol}"),
            )
        };
        // A trace already contains these native transfers
        if analysis.native_transfers.is_none() {
            edges.push((node(from), node(to), what));
        }
    }
    match &analysis.native_transfers {
        Some(transfers) => {
            for (from, to, value) in transfers {
                let label = format!("{} native", amount(&value.to_string(), Some(18)));
                edges.push((node(*from), node(*to), label));
            }
        }
        None => {
            if let Some(to) = analysis.to.filter(|_| !analysis.value.is_zero()) {
                let label = format!("{} native", amount(&analysis.value.to_string(), Some(18)));
                edges.push((node(analysis.from), node(to), label));
            }
        }
    }

    let mut out = String::from("flowchart LR\n");
    for (index, address) in nodes.iter().enumerate() {
        let mut name = get_label(address).map(String::from).unwrap_or_default();
        if *address == analysis.from {
            name = if name.is_empty() {
                "sender".to_string()
            } else {
                format!("sender, {name}")
            };
        }
        let text = if name.is_empty() {
            short_address(address)
        } else {
            format!("{name}<br/>{}", short_address(address))
        };
        let _ = writeln!(out, "  n{index}[\"{}\"]", text.replace('"', "#quot;"));
    }
    for (from, to, label) in edges {
        let _ = writeln!(
            out,
            "  n{from} -->|\"{}\"| n{to}",
            label.replace('"', "#quot;").replace('|', "#124;")
        );
    }
    if analysis.native_transfers.is_none() {
        out.push_str("  %% No call trace: native transfers made inside the call are missing\n");
    }
    out
}

/// `0x1234…abcd`
fn short_address(address: &Address) -> String {
    let full = format!("{address:#x}");
    format!("{}…{}", &full[..6], &full[full.len() - 4..])
}

/// Transaction analyzer
pub struct TxAnalyzer {
    /// RPC pool
    pool: RpcPool,
    /// Chain
    chain: Chain,
    /// ABI fetcher for Etherscan lookups (shared to avoid HTTP client recreation - PERF-002 fix)
    abi_fetcher: Arc<AbiFetcher>,
}

impl TxAnalyzer {
    /// Create a new analyzer
    pub fn new(pool: RpcPool, chain: Chain) -> Result<Self> {
        Ok(Self {
            pool,
            chain,
            abi_fetcher: Arc::new(AbiFetcher::new(None)?),
        })
    }

    /// Create a new analyzer with a shared AbiFetcher (PERF-002 fix)
    ///
    /// Use this when analyzing multiple transactions to avoid recreating
    /// HTTP clients for each transaction.
    pub fn with_fetcher(pool: RpcPool, chain: Chain, abi_fetcher: Arc<AbiFetcher>) -> Self {
        Self {
            pool,
            chain,
            abi_fetcher,
        }
    }

    /// Get a reference to the ABI fetcher for sharing with other analyzers
    pub fn abi_fetcher(&self) -> Arc<AbiFetcher> {
        Arc::clone(&self.abi_fetcher)
    }

    /// Analyze a transaction by hash (basic analysis)
    pub async fn analyze(&self, hash: &str) -> Result<TransactionAnalysis> {
        self.analyze_internal(hash, false, None).await
    }

    /// Analyze a transaction with enrichment from Etherscan (slower but more detailed)
    pub async fn analyze_enriched(&self, hash: &str) -> Result<TransactionAnalysis> {
        self.analyze_internal(hash, true, None).await
    }

    /// Analyze a transaction by hash with optional block number for partial hash resolution
    pub async fn analyze_with_block(
        &self,
        hash: &str,
        block_number: Option<u64>,
    ) -> Result<TransactionAnalysis> {
        self.analyze_internal(hash, false, block_number).await
    }

    /// Analyze a transaction with enrichment and optional block number for partial hash resolution
    pub async fn analyze_enriched_with_block(
        &self,
        hash: &str,
        block_number: Option<u64>,
    ) -> Result<TransactionAnalysis> {
        self.analyze_internal(hash, true, block_number).await
    }

    /// Resolve a partial transaction hash using a block number
    ///
    /// If the hash is less than 66 characters (full hash length with 0x prefix),
    /// and a block number is provided, fetch the block and find the transaction
    /// that matches the prefix.
    pub async fn resolve_partial_hash(
        &self,
        partial_hash: &str,
        block_number: u64,
    ) -> Result<B256> {
        let prefix = partial_hash.to_lowercase();
        let prefix = prefix.strip_prefix("0x").unwrap_or(&prefix);

        // Minimum 8 characters (4 bytes) for reasonable matching
        if prefix.len() < 8 {
            return Err(format!(
                "Partial hash '{}' is too short. Need at least 8 hex characters for reliable matching.",
                partial_hash
            ).into());
        }

        // Fetch block with full transactions
        let block = self
            .pool
            .get_block_with_txs(block_number)
            .await?
            .ok_or_else(|| format!("Block {} not found", block_number))?;

        // Search for matching transaction
        let mut matches = Vec::new();
        if let alloy::rpc::types::BlockTransactions::Full(txs) = block.transactions {
            for tx in txs {
                let tx_hash = format!("{:x}", tx.inner.tx_hash());
                if tx_hash.starts_with(prefix) {
                    matches.push(*tx.inner.tx_hash());
                }
            }
        }

        match matches.len() {
            0 => Err(format!(
                "No transaction in block {} matches prefix '{}'",
                block_number, partial_hash
            )
            .into()),
            1 => Ok(matches[0]),
            n => Err(format!(
                "Ambiguous: {} transactions in block {} match prefix '{}': {:?}",
                n,
                block_number,
                partial_hash,
                matches
                    .iter()
                    .map(|h| format!("{:#x}", h))
                    .collect::<Vec<_>>()
            )
            .into()),
        }
    }

    /// Add what needs extra requests: the block's time, and for a failed
    /// transaction the reason it reverted.
    ///
    /// The revert reason comes from replaying the call against the state at
    /// the end of the parent block. That is the state the block started
    /// from, not the exact state the transaction ran in, so a transaction
    /// that failed because of an earlier one in the same block may not
    /// reproduce; a note says so when that happens.
    pub async fn add_full_details(
        &self,
        analysis: &mut TransactionAnalysis,
        rpc_url: Option<&str>,
    ) {
        use alloy::providers::Provider;

        let endpoint = match rpc_url {
            Some(url) => crate::rpc::Endpoint::new(
                crate::config::EndpointConfig::new(url.to_string()),
                30,
                None,
            )
            .map_err(|e| e.to_string()),
            None => crate::rpc::get_rpc_endpoint_for_block(self.chain, analysis.block_number)
                .map_err(|e| e.to_string()),
        };
        let endpoint = match endpoint {
            Ok(endpoint) => endpoint,
            Err(e) => {
                analysis
                    .notes
                    .push(format!("Block time and revert reason not fetched: {e}"));
                return;
            }
        };
        let provider = endpoint.provider();

        match provider.get_block(analysis.block_number.into()).await {
            Ok(Some(block)) => {
                analysis.timestamp = Some(block.header.timestamp);
                analysis.time = Some(crate::rpc::block_time::format_timestamp(
                    block.header.timestamp,
                ));
            }
            Ok(None) => analysis
                .notes
                .push("Block time not fetched: block not found".to_string()),
            Err(e) => analysis.notes.push(format!("Block time not fetched: {e}")),
        }

        // Is a separate L1 fee really charged on top? Where the sender's
        // balance change equals value + gasUsed x price exactly, the l1Fee
        // was informational (already inside the gas price): fix the split
        let mut fee_fixed = false;
        if analysis.block_number > 0 {
            if let Some(b) = analysis.fee_breakdown.as_mut() {
                if b.l1_fee_wei.as_deref().is_some_and(|f| f != "0") && !b.l1_fee_included {
                    let from = analysis.from;
                    let at = |n: u64| provider.get_balance(from).block_id(n.into());
                    if let (Ok(before), Ok(after)) = (
                        at(analysis.block_number - 1).await,
                        at(analysis.block_number).await,
                    ) {
                        if let Some(spent) = before.checked_sub(after) {
                            let gas_times_price = alloy::primitives::U256::from(analysis.gas_used)
                                * b.gas_price_wei
                                    .parse::<alloy::primitives::U256>()
                                    .unwrap_or_default();
                            if spent == analysis.value + gas_times_price && b.mark_l1_fee_included()
                            {
                                analysis.fee_wei = Some(b.total_fee_wei.clone());
                                analysis.fee = Some(b.total_fee.clone());
                                analysis.notes.push(
                                    "The receipt's l1Fee is already part of gasUsed x price \
                                     (checked against the sender's balance change)"
                                        .to_string(),
                                );
                                fee_fixed = true;
                            }
                        }
                    }
                }
            }
        }

        if fee_fixed {
            refresh_balance_changes(analysis);
        }

        // The fee in USD at the block's time
        if let (Some(ts), Some(b)) = (analysis.timestamp, analysis.fee_breakdown.as_mut()) {
            match crate::tx::l2_fee::native_usd_price(self.chain, Some(ts)).await {
                Ok((price, source)) => b.set_usd(price, source),
                Err(e) => analysis.notes.push(format!("Fee in USD not computed: {e}")),
            }
        }

        // A call trace gives two things exactly: every native transfer made
        // inside the call, and for a failed transaction the revert data of
        // the transaction as it actually ran
        let mut traced_revert: Option<Vec<u8>> = None;
        match self.fetch_call_trace(analysis.hash, rpc_url).await {
            Ok(trace) => {
                let transfers = native_transfers_from_trace(&trace);
                analysis.native_transfer_list = transfers
                    .iter()
                    .map(|(from, to, value)| crate::tx::types::NativeTransfer {
                        from: *from,
                        to: *to,
                        value_wei: value.to_string(),
                        value: alloy::primitives::utils::format_units(*value, 18)
                            .unwrap_or_else(|_| value.to_string()),
                    })
                    .collect();
                analysis.native_transfers = Some(transfers);
                refresh_balance_changes(analysis);
                traced_revert = revert_data_from_trace(&trace);
            }
            Err(reason) => analysis.notes.push(format!(
                "No call trace: {reason}. Native-token balance changes stay partial"
            )),
        }

        if analysis.status || analysis.block_number == 0 {
            return;
        }
        // The target's ABI names its own errors with certainty
        let abi = match analysis.to {
            Some(to) => crate::etherscan::Client::new(self.chain, None)
                .ok()
                .and_then(|client| client.cached_abi(&format!("{to:#x}"))),
            None => None,
        };
        if let Some(data) = traced_revert {
            analysis.revert =
                Some(crate::abi::revert::describe(&data, abi.as_ref(), self.chain).await);
            analysis.revert_from = Some("trace".to_string());
            return;
        }

        // No trace: re-run the call against the state at the end of the
        // parent block. That is not the state the transaction ran in, so the
        // reason found this way can differ from the real one.
        let mut call = alloy::rpc::types::TransactionRequest::default()
            .from(analysis.from)
            .value(analysis.value)
            .gas_limit(analysis.gas_limit)
            .input(analysis.input.clone().into());
        if let Some(to) = analysis.to {
            call = call.to(to);
        }
        let parent: alloy::eips::BlockId = (analysis.block_number - 1).into();
        match provider.call(call).block(parent).await {
            Ok(_) => analysis.notes.push(
                "Revert reason not found: no call trace was available, and the call succeeds \
                 against the parent block's state, so the failure depended on an earlier \
                 transaction in the same block, on the block's time, or on gas"
                    .to_string(),
            ),
            Err(e) => match crate::abi::revert::revert_data_of(&e) {
                Some(data) => {
                    analysis.revert =
                        Some(crate::abi::revert::describe(&data, abi.as_ref(), self.chain).await);
                    analysis.revert_from = Some("replay".to_string());
                    analysis.notes.push(
                        "revert comes from re-running the call against the parent block's state, \
                         not from the transaction as it ran. If the failure depended on an earlier \
                         transaction in the same block or on the block's time, the real reason may \
                         differ. A tracing endpoint gives the actual reason"
                            .to_string(),
                    );
                }
                None => analysis
                    .notes
                    .push(format!("Revert reason not fetched: {e}")),
            },
        }
    }

    /// A transaction's `debug_traceTransaction` call trace (call tracer),
    /// tried on up to three debug-capable endpoints, or only on `rpc_url`
    /// when one is given.
    async fn fetch_call_trace(
        &self,
        hash: B256,
        rpc_url: Option<&str>,
    ) -> std::result::Result<serde_json::Value, String> {
        use alloy::providers::Provider;

        let configs: Vec<crate::config::EndpointConfig> = match rpc_url {
            Some(url) => vec![crate::config::EndpointConfig::new(url.to_string())],
            None => {
                let config = crate::config::ConfigFile::load_default()
                    .map_err(|e| e.to_string())?
                    .unwrap_or_default();
                crate::rpc::ranked_endpoint_configs(
                    &config,
                    self.chain,
                    &crate::rpc::SelectionOptions::default(),
                )
                .map_err(|e| e.to_string())?
                .into_iter()
                .filter(|endpoint| endpoint.has_debug)
                .take(3)
                .collect()
            }
        };
        if configs.is_empty() {
            return Err("no configured endpoint supports tracing".to_string());
        }

        let mut last_error = String::new();
        for config in configs {
            let endpoint = match crate::rpc::Endpoint::new(config, 60, None) {
                Ok(endpoint) => endpoint,
                Err(e) => {
                    last_error = e.to_string();
                    continue;
                }
            };
            let params = (
                format!("{hash:#x}"),
                serde_json::json!({ "tracer": "callTracer" }),
            );
            match endpoint
                .provider()
                .client()
                .request::<_, serde_json::Value>("debug_traceTransaction", params)
                .await
            {
                Ok(trace) => return Ok(trace),
                // Keep the reason short and free of the endpoint's address
                Err(e) => {
                    last_error = e
                        .to_string()
                        .split_whitespace()
                        .map(|word| {
                            if word.contains("://") {
                                "<endpoint>"
                            } else {
                                word
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                        .chars()
                        .take(160)
                        .collect();
                }
            }
        }
        Err(format!("the trace request failed ({last_error})"))
    }

    /// Internal analyze method
    async fn analyze_internal(
        &self,
        hash: &str,
        enrich: bool,
        block_number: Option<u64>,
    ) -> Result<TransactionAnalysis> {
        // Check if this is a partial hash that needs resolution
        let hash = if let (true, Some(block)) = (hash.len() < 66, block_number) {
            // Partial hash with block number - resolve it
            self.resolve_partial_hash(hash, block).await?
        } else {
            // Full hash or no block number - parse directly
            B256::from_str(hash)
                .map_err(|e| format!("Invalid transaction hash '{}': {}", hash, e))?
        };

        // Fetch raw data
        let raw = self.fetch_raw_data(hash).await?;

        // Build analysis
        let mut analysis = TransactionAnalysis::new(&raw);
        self.apply_fee_breakdown(&mut analysis, hash).await;

        // Analyze contracts involved
        analysis.contracts = self.analyze_contracts(&raw);

        // Analyze events
        analysis.events = self.analyze_events(&raw);

        // Parse token flows
        analysis.token_flows = parse_transfers(&raw.logs);
        if let Some(wrapped) = wrapped_native_token(self.chain.chain_id()) {
            analysis.wraps = parse_wraps(&raw.logs, wrapped);
        }

        // Label token flows
        for flow in &mut analysis.token_flows {
            if flow.token_label.is_none() {
                flow.token_label = get_label(&flow.token).map(String::from);
            }
            if flow.from_label.is_none() {
                flow.from_label = get_label(&flow.from).map(String::from);
            }
            if flow.to_label.is_none() {
                flow.to_label = get_label(&flow.to).map(String::from);
            }
        }

        // Resolve token decimals so amounts render correctly (USDC = 6, etc.)
        self.resolve_token_decimals(&mut analysis.token_flows).await;

        // Decode function call if there's input data
        if let Some(to) = analysis.to {
            let input = raw.tx.inner.input();
            if input.len() >= 4 {
                analysis.function_call = self.decode_function_call(&to, input, enrich).await;
            }
        }

        // Enrich unknown contracts if requested
        if enrich {
            self.enrich_contracts(&mut analysis.contracts).await;
            self.enrich_token_flows(&mut analysis.token_flows).await;
        }

        refresh_balance_changes(&mut analysis);

        Ok(analysis)
    }

    /// Decode function call from input data
    async fn decode_function_call(
        &self,
        to: &Address,
        input: &[u8],
        fetch_abi: bool,
    ) -> Option<FunctionCall> {
        if input.len() < 4 {
            return None;
        }

        let selector = format!("0x{}", hex::encode(&input[..4]));

        if fetch_abi {
            // Try to decode using Etherscan ABI
            if let Some(decoded) = self
                .abi_fetcher
                .decode_function_call(self.chain, &format!("{:#x}", to), input)
                .await
            {
                return Some(FunctionCall {
                    selector: decoded.selector,
                    name: decoded.name,
                    signature: decoded.signature,
                    params: decoded
                        .params
                        .into_iter()
                        .map(|(name, ty, value)| FunctionParam { name, ty, value })
                        .collect(),
                });
            }
        }

        // Try 4byte.directory lookup as fallback
        if let Some(sig) = self.abi_fetcher.lookup_selector(&selector).await {
            let name = sig.split('(').next().map(String::from);
            return Some(FunctionCall {
                selector,
                name,
                signature: Some(sig),
                params: Vec::new(),
            });
        }

        // Return unknown function with just selector
        Some(FunctionCall {
            selector,
            name: None,
            signature: None,
            params: Vec::new(),
        })
    }

    /// Enrich unknown contracts with Etherscan metadata
    async fn enrich_contracts(&self, contracts: &mut [ContractInfo]) {
        use futures::stream::{self, StreamExt};

        // MED-004 fix: Limit concurrency to prevent resource exhaustion
        const MAX_CONCURRENT_REQUESTS: usize = 10;

        // Collect contracts that need lookup
        let lookup_indices: Vec<usize> = contracts
            .iter()
            .enumerate()
            .filter(|(_, c)| c.label.is_none() && c.address != Address::ZERO)
            .map(|(i, _)| i)
            .collect();

        if lookup_indices.is_empty() {
            return;
        }

        // Clone what we need to avoid lifetime issues with async closures
        let abi_fetcher = Arc::clone(&self.abi_fetcher);
        let chain = self.chain;

        // Collect addresses before async to avoid borrowing contracts in closures
        let lookup_data: Vec<(usize, Address)> = lookup_indices
            .iter()
            .map(|&i| (i, contracts[i].address))
            .collect();

        // Fetch metadata with bounded concurrency
        let results: Vec<_> = stream::iter(lookup_data.into_iter().map(|(i, address)| {
            let fetcher = Arc::clone(&abi_fetcher);
            async move {
                let result = fetcher
                    .get_contract_metadata(chain, &format!("{:#x}", address))
                    .await;
                (i, result)
            }
        }))
        .buffer_unordered(MAX_CONCURRENT_REQUESTS)
        .collect()
        .await;

        // Apply results
        for (i, result) in results {
            if let Ok(metadata) = result {
                if let Some(name) = metadata.name {
                    contracts[i].label = Some(name);
                    // If verified, it's likely a contract (not EOA)
                    if metadata.is_verified && contracts[i].category == ContractCategory::Unknown {
                        contracts[i].category = ContractCategory::Protocol;
                    }
                }
            }
        }
    }

    /// Look up `decimals()` for every distinct token in `flows` via RPC.
    ///
    /// Failures leave `decimals = None`; the formatter then prints raw units
    /// rather than guessing 18.
    async fn resolve_token_decimals(&self, flows: &mut [crate::tx::types::TokenFlow]) {
        use futures::stream::{self, StreamExt};
        const MAX_CONCURRENT_REQUESTS: usize = 8;
        // decimals() selector
        const DECIMALS_SELECTOR: [u8; 4] = [0x31, 0x3c, 0xe5, 0x67];

        let tokens: HashSet<Address> = flows
            .iter()
            .filter(|f| f.decimals.is_none())
            .map(|f| f.token)
            .collect();
        if tokens.is_empty() {
            return;
        }

        let pool = &self.pool;
        let results: Vec<(Address, Option<u8>)> =
            stream::iter(tokens.into_iter().map(|token| async move {
                let out = pool
                    .call(
                        token,
                        alloy::primitives::Bytes::from_static(&DECIMALS_SELECTOR),
                    )
                    .await
                    .ok();
                (token, out.and_then(|b| decode_decimals(&b)))
            }))
            .buffer_unordered(MAX_CONCURRENT_REQUESTS)
            .collect()
            .await;

        let map: HashMap<Address, u8> = results
            .into_iter()
            .filter_map(|(t, d)| d.map(|d| (t, d)))
            .collect();
        for flow in flows.iter_mut() {
            if flow.decimals.is_none() {
                flow.decimals = map.get(&flow.token).copied();
            }
        }
    }

    /// Enrich token flows with token metadata
    async fn enrich_token_flows(&self, flows: &mut [crate::tx::types::TokenFlow]) {
        // Collect unique tokens that need lookup
        let mut tokens_to_lookup: HashSet<Address> = HashSet::new();
        for flow in flows.iter() {
            if flow.token_label.is_none() {
                tokens_to_lookup.insert(flow.token);
            }
        }

        if tokens_to_lookup.is_empty() {
            return;
        }

        // One Multicall3 call for every token (Etherscan only as a fallback)
        let tokens: Vec<Address> = tokens_to_lookup.into_iter().collect();
        let token_symbols: HashMap<Address, String> = self
            .abi_fetcher
            .get_tokens_metadata_rpc(self.chain, &tokens)
            .await
            .into_iter()
            .filter_map(|(token, metadata)| metadata.symbol.map(|symbol| (token, symbol)))
            .collect();

        // Apply to flows
        for flow in flows.iter_mut() {
            if flow.token_label.is_none() {
                if let Some(symbol) = token_symbols.get(&flow.token) {
                    flow.token_label = Some(symbol.clone());
                }
            }
        }
    }

    /// Fetch raw transaction data
    /// Split the fee into execution and L1 data parts. On rollups this needs
    /// the raw receipt (alloy drops the L1 fee fields), and the total
    /// replaces `fee_wei`/`fee`, since the L1 data fee is charged too.
    async fn apply_fee_breakdown(&self, analysis: &mut TransactionAnalysis, hash: B256) {
        use crate::tx::l2_fee::{FeeBreakdown, FeeModel};
        let chain_id = self.chain.chain_id();
        // Known L1 chains skip the raw receipt; anything else is read, since
        // the receipt's own l1Fee / gasUsedForL1 fields decide the split
        if FeeModel::known(chain_id) == Some(FeeModel::L1) {
            let (Some(price), Some(fee)) = (&analysis.effective_gas_price, &analysis.fee_wei)
            else {
                return;
            };
            let receipt = serde_json::json!({
                "gasUsed": analysis.gas_used.to_string(),
                "effectiveGasPrice": price,
            });
            analysis.fee_breakdown = FeeBreakdown::from_receipt_json(chain_id, &receipt)
                .filter(|b| &b.total_fee_wei == fee);
            return;
        }
        match self.pool.get_receipt_json(hash).await {
            Ok(Some(receipt)) => match FeeBreakdown::from_receipt_json(chain_id, &receipt) {
                Some(b) => {
                    analysis.fee_wei = Some(b.total_fee_wei.clone());
                    analysis.fee = Some(b.total_fee.clone());
                    analysis.fee_breakdown = Some(b);
                }
                None => analysis
                    .notes
                    .push("L1 data fee not read: receipt has no gas fields".to_string()),
            },
            Ok(None) => {}
            Err(e) => analysis.notes.push(format!(
                "L1 data fee not read ({e}); fee is L2 execution only"
            )),
        }
    }

    async fn fetch_raw_data(&self, hash: B256) -> Result<RawTxData> {
        // Fetch transaction and receipt in parallel
        let (tx_result, receipt_result) = tokio::join!(
            self.pool.get_transaction(hash),
            self.pool.get_transaction_receipt(hash)
        );

        let tx = tx_result?.ok_or_else(|| format!("Transaction not found: {:#x}", hash))?;

        let receipt = receipt_result?.ok_or_else(|| format!("Receipt not found: {:#x}", hash))?;

        let logs = receipt.inner.logs().to_vec();

        Ok(RawTxData { tx, receipt, logs })
    }

    /// Analyze contracts involved in the transaction
    fn analyze_contracts(&self, raw: &RawTxData) -> Vec<ContractInfo> {
        let mut addresses = HashSet::new();

        // Add from and to - access via Transaction trait from inner
        addresses.insert(raw.tx.inner.signer());
        if let Some(to) = raw.tx.inner.to() {
            addresses.insert(to);
        }

        // Add all addresses from logs
        for log in &raw.logs {
            addresses.insert(log.address());

            // Also extract addresses from indexed topics
            for topic in log.topics().iter().skip(1) {
                // Check if this looks like an address (first 12 bytes are zeros)
                if topic.0[..12] == [0u8; 12] {
                    let addr = Address::from_slice(&topic.0[12..]);
                    if addr != Address::ZERO {
                        addresses.insert(addr);
                    }
                }
            }
        }

        // Build contract info for each address
        let mut contracts: Vec<ContractInfo> = addresses
            .into_iter()
            .map(|addr| get_contract_info(&addr))
            .collect();

        // Sort by category then by label
        contracts.sort_by(|a, b| {
            // LOW-006: Explicit exhaustive match - adding new ContractCategory variants
            // will cause a compile error here, ensuring the sort order is updated.
            let cat_ord = |c: &ContractCategory| match c {
                ContractCategory::Token => 0,
                ContractCategory::Dex => 1,
                ContractCategory::Lending => 2,
                ContractCategory::Staking => 3,
                ContractCategory::Bridge => 4,
                ContractCategory::Nft => 5,
                ContractCategory::Protocol => 6,
                ContractCategory::Unknown => 7,
                ContractCategory::Eoa => 8,
            };

            let ord = cat_ord(&a.category).cmp(&cat_ord(&b.category));
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }

            // Then by label (known first)
            match (&a.label, &b.label) {
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (Some(a_l), Some(b_l)) => a_l.as_str().cmp(b_l.as_str()),
                (None, None) => a.address.cmp(&b.address),
            }
        });

        contracts
    }

    /// Analyze events in the transaction
    fn analyze_events(&self, raw: &RawTxData) -> Vec<AnalyzedEvent> {
        raw.logs
            .iter()
            .map(|log| {
                let topic0 = log.topics().first().copied().unwrap_or_default();
                let is_transfer = topic0 == events::TRANSFER;
                let event_name = if is_transfer && log.topics().len() == 4 {
                    Some("Transfer (ERC-721)")
                } else {
                    events::get_event_name(&topic0)
                };

                // Decode parameters based on known event signatures
                let params = self.decode_event_params(log, &topic0);

                AnalyzedEvent {
                    log_index: log.log_index.unwrap_or(0),
                    address: log.address(),
                    address_label: get_label(&log.address()).map(String::from),
                    name: event_name.map(String::from),
                    // The canonical signature, not the display name
                    signature: events::get_event_signature(&topic0).map(String::from),
                    params,
                    topic0,
                    is_transfer,
                }
            })
            .collect()
    }

    /// Decode event parameters
    fn decode_event_params(
        &self,
        log: &alloy::rpc::types::Log,
        topic0: &B256,
    ) -> HashMap<String, EventParam> {
        // PERF-010 fix: pre-allocate for typical event parameter count (3-4)
        let mut params = HashMap::with_capacity(4);
        let topics = log.topics();

        if *topic0 == events::TRANSFER && topics.len() >= 3 {
            // Transfer(address indexed from, address indexed to, uint256 value)
            let from = Address::from_slice(&topics[1].0[12..]);
            let to = Address::from_slice(&topics[2].0[12..]);

            params.insert(
                "from".to_string(),
                EventParam::Address(format!("{:#x}", from)),
            );
            params.insert("to".to_string(), EventParam::Address(format!("{:#x}", to)));

            let data = &log.data().data;
            if let Some(token_id) = topics.get(3) {
                // ERC-721: tokenId is the third indexed param
                let id = alloy::primitives::U256::from_be_bytes(token_id.0);
                params.insert("tokenId".to_string(), EventParam::Uint(id.to_string()));
            } else if data.len() >= 32 {
                let value = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("value".to_string(), EventParam::Uint(value.to_string()));
            } else {
                // LOW-001 fix: Log malformed event data for debugging
                tracing::debug!(
                    "Malformed Transfer event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        } else if *topic0 == events::APPROVAL && topics.len() >= 3 {
            // Approval(address indexed owner, address indexed spender, uint256 value)
            let owner = Address::from_slice(&topics[1].0[12..]);
            let spender = Address::from_slice(&topics[2].0[12..]);

            params.insert(
                "owner".to_string(),
                EventParam::Address(format!("{:#x}", owner)),
            );
            params.insert(
                "spender".to_string(),
                EventParam::Address(format!("{:#x}", spender)),
            );

            let data = &log.data().data;
            if data.len() >= 32 {
                let value = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("value".to_string(), EventParam::Uint(value.to_string()));
            } else {
                tracing::debug!(
                    "Malformed Approval event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        } else if *topic0 == events::DEPOSIT && topics.len() >= 2 {
            // Deposit(address indexed dst, uint256 wad)
            let dst = Address::from_slice(&topics[1].0[12..]);
            params.insert(
                "dst".to_string(),
                EventParam::Address(format!("{:#x}", dst)),
            );

            let data = &log.data().data;
            if data.len() >= 32 {
                let wad = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("wad".to_string(), EventParam::Uint(wad.to_string()));
            } else {
                tracing::debug!(
                    "Malformed Deposit event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        } else if *topic0 == events::WITHDRAWAL && topics.len() >= 2 {
            // Withdrawal(address indexed src, uint256 wad)
            let src = Address::from_slice(&topics[1].0[12..]);
            params.insert(
                "src".to_string(),
                EventParam::Address(format!("{:#x}", src)),
            );

            let data = &log.data().data;
            if data.len() >= 32 {
                let wad = alloy::primitives::U256::from_be_slice(&data[..32]);
                params.insert("wad".to_string(), EventParam::Uint(wad.to_string()));
            } else {
                tracing::debug!(
                    "Malformed Withdrawal event data: expected >= 32 bytes, got {} at {:?}",
                    data.len(),
                    log.address()
                );
            }
        }

        params
    }
}

/// Format transaction analysis for display
pub fn format_analysis(analysis: &TransactionAnalysis) -> String {
    // LOW-005 fix: Pre-allocate for typical output size
    let mut output = String::with_capacity(2048);

    // Header
    output.push_str(&format!("Transaction: {:#x}\n", analysis.hash));
    match &analysis.time {
        Some(time) => output.push_str(&format!("Block: {} ({})\n", analysis.block_number, time)),
        None => output.push_str(&format!("Block: {}\n", analysis.block_number)),
    }
    if let Some(index) = analysis.transaction_index {
        output.push_str(&format!("Index in block: {}\n", index));
    }
    output.push_str(&format!("From: {:#x}\n", analysis.from));

    if let Some(to) = analysis.to {
        let label = get_label(&to)
            .map(|l| format!(" ({})", l))
            .unwrap_or_default();
        output.push_str(&format!("To: {:#x}{}\n", to, label));
    } else {
        output.push_str("To: Contract Creation\n");
    }

    // HIGH-001 fix: Use safe U256 to f64 conversion
    output.push_str(&format!(
        "Value: {} wei ({:.6} ETH)\n",
        analysis.value,
        u256_to_eth_f64(&analysis.value)
    ));
    output.push_str(&format!("Gas Used: {}\n", analysis.gas_used));
    match &analysis.fee_breakdown {
        Some(b) if b.l1_fee_wei.is_some() => {
            output.push_str(&format!("Fee: {} ({})\n", b.total_fee, b.model));
            for line in b.lines(&b.native_symbol) {
                output.push_str(&line);
                output.push('\n');
            }
        }
        _ => {
            if let (Some(fee), Some(price)) = (&analysis.fee, &analysis.effective_gas_price) {
                output.push_str(&format!("Fee: {} (gas price {} wei)\n", fee, price));
                if let Some(u) = analysis.fee_breakdown.as_ref().and_then(|b| b.usd.as_ref()) {
                    output.push_str(&format!(
                        "  ${} at ${} ({})\n",
                        crate::tx::l2_fee::format_usd(u.total_usd),
                        crate::tx::l2_fee::format_usd(u.price_usd),
                        u.source
                    ));
                }
            }
        }
    }
    output.push_str(&format!(
        "Status: {}\n",
        if analysis.status { "Success" } else { "Failed" }
    ));
    if let Some(revert) = &analysis.revert {
        let how = match analysis.revert_from.as_deref() {
            Some("trace") => " (from the call trace)",
            Some("replay") => {
                " (from a replay on the parent block; may differ from the real reason)"
            }
            _ => "",
        };
        output.push_str(&format!("Reverted: {}{}\n", revert.message, how));
    }

    // Contracts
    output.push_str("\nContracts Involved:\n");
    for contract in &analysis.contracts {
        let label = contract
            .label
            .as_ref()
            .map(|l| format!(" {}", l))
            .unwrap_or_else(|| format!(" ({:?})", contract.category));
        output.push_str(&format!("  {:#x}{}\n", contract.address, label));
    }

    // Events summary
    output.push_str(&format!("\nEvents ({}):\n", analysis.events.len()));
    for (i, event) in analysis.events.iter().enumerate().take(20) {
        let name = event.name.as_deref().unwrap_or("Unknown");
        let contract = event
            .address_label
            .as_ref()
            .map(|l| format!("({})", l))
            .unwrap_or_default();
        output.push_str(&format!("  [{}] {} {}\n", i, name, contract));
    }
    if analysis.events.len() > 20 {
        output.push_str(&format!("  ... and {} more\n", analysis.events.len() - 20));
    }

    // Token flows
    if !analysis.token_flows.is_empty() {
        output.push_str(&format!(
            "\nToken Transfers ({}):\n",
            analysis.token_flows.len()
        ));
        for flow in analysis.token_flows.iter().take(15) {
            let token = flow.token_label.as_deref().unwrap_or("???");
            let from_short = format!("{:#x}", flow.from);
            let from_short = if from_short.len() > 12 {
                &from_short[..12]
            } else {
                &from_short
            };
            let to_short = format!("{:#x}", flow.to);
            let to_short = if to_short.len() > 12 {
                &to_short[..12]
            } else {
                &to_short
            };

            // LOW-002 fix: Use safe token amount formatting
            let amount_display = format_flow_amount(&flow.amount, flow.decimals);

            output.push_str(&format!(
                "  {} {}... → {}... {} {}\n",
                token, from_short, to_short, amount_display, token
            ));
        }
        if analysis.token_flows.len() > 15 {
            output.push_str(&format!(
                "  ... and {} more\n",
                analysis.token_flows.len() - 15
            ));
        }
    }

    // Net balance changes
    if !analysis.balance_changes.is_empty() {
        output.push_str(&format!(
            "\nNet Balance Changes ({}):\n",
            analysis.balance_changes.len()
        ));
        for change in analysis.balance_changes.iter().take(30) {
            let who = match &change.label {
                Some(label) => format!("{:#x} ({})", change.address, label),
                None => format!("{:#x}", change.address),
            };
            let amount = change
                .formatted
                .clone()
                .unwrap_or_else(|| format!("{} (raw units)", change.raw));
            let token = match (&change.symbol, change.token) {
                (Some(symbol), _) => crate::abi::revert::safe_text(symbol),
                (None, Some(token)) => format!("{token:#x}"),
                (None, None) => "native".to_string(),
            };
            let partial = if change.partial { " (partial)" } else { "" };
            output.push_str(&format!("  {who}  {amount} {token}{partial}\n"));
        }
        if analysis.balance_changes.len() > 30 {
            output.push_str(&format!(
                "  ... and {} more\n",
                analysis.balance_changes.len() - 30
            ));
        }
    }

    // Function call
    if let Some(func) = &analysis.function_call {
        output.push_str("\nFunction Call:\n");
        let name = func.name.as_deref().unwrap_or("Unknown");
        output.push_str(&format!("  {} ({})\n", name, func.selector));
        if let Some(sig) = &func.signature {
            output.push_str(&format!("  Signature: {}\n", sig));
        }
        if !func.params.is_empty() {
            output.push_str("  Parameters:\n");
            for param in func.params.iter().take(10) {
                output.push_str(&format!(
                    "    {} ({}): {}\n",
                    param.name, param.ty, param.value
                ));
            }
            if func.params.len() > 10 {
                output.push_str(&format!("    ... and {} more\n", func.params.len() - 10));
            }
        }
    }

    for note in &analysis.notes {
        output.push_str(&format!("\nNote: {note}\n"));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tx::types::{AnalyzedEvent, FunctionCall, FunctionParam};
    use alloy::primitives::{address, b256, B256, U256};
    use std::collections::HashMap;

    #[test]
    fn test_flow_amount_uses_token_decimals() {
        // 2.82 USDC (6 decimals) previously rendered as 0.00 with 18 decimals
        assert_eq!(format_flow_amount("2820000", Some(6)), "2.82");
        assert_eq!(format_flow_amount("1230000000000000000", Some(18)), "1.23");
        assert_eq!(format_flow_amount("5", Some(0)), "5");
        assert_eq!(format_flow_amount("2820000", None), "2820000 (raw units)");
        // Dust keeps two significant digits
        assert_eq!(format_flow_amount("28340421969401", Some(18)), "0.000028");
    }

    #[test]
    fn test_decode_decimals() {
        let mut word = [0u8; 32];
        word[31] = 6;
        assert_eq!(decode_decimals(&word), Some(6));
        assert_eq!(decode_decimals(&[]), None);
        assert_eq!(decode_decimals(&[0xff; 32]), None);
    }

    fn make_test_analysis() -> TransactionAnalysis {
        TransactionAnalysis {
            hash: b256!("0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
            block_number: 18_500_000,
            from: address!("d8da6bf26964af9d7eed9e03e53415d37aa96045"),
            to: Some(address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2")),
            value: U256::from(1_000_000_000_000_000_000u128), // 1 ETH
            gas_used: 21000,
            status: true,
            contracts: vec![ContractInfo {
                address: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
                label: Some("WETH".to_string()),
                category: ContractCategory::Token,
            }],
            events: vec![],
            token_flows: vec![],
            function_call: None,
            transaction_index: Some(3),
            effective_gas_price: Some("20000000000".to_string()),
            fee_wei: Some("420000000000000".to_string()),
            fee: Some("0.000420000000000000".to_string()),
            fee_breakdown: None,
            balance_changes: vec![],
            timestamp: None,
            time: None,
            revert: None,
            revert_from: None,
            notes: vec![],
            input: Default::default(),
            gas_limit: 21000,
            wraps: vec![],
            native_transfers: None,
            native_transfer_list: vec![],
        }
    }

    #[test]
    fn test_format_analysis_basic() {
        let analysis = make_test_analysis();
        let output = format_analysis(&analysis);

        assert!(output.contains("Transaction: 0x1234"));
        assert!(output.contains("Block: 18500000"));
        assert!(output.contains("Status: Success"));
        assert!(output.contains("WETH"));
    }

    #[test]
    fn net_balance_changes_sum_flows_value_and_fee() {
        use crate::tx::types::TokenFlow;
        let sender = address!("d8da6bf26964af9d7eed9e03e53415d37aa96045");
        let pool = address!("1111111111111111111111111111111111111111");
        let usdc = address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48");
        let flow = |from, to, amount: &str, log_index| TokenFlow {
            token: usdc,
            token_label: Some("USDC".to_string()),
            from,
            from_label: None,
            to,
            to_label: None,
            amount: amount.to_string(),
            decimals: Some(6),
            token_id: None,
            log_index,
        };
        let mut analysis = make_test_analysis();
        analysis.to = Some(pool);
        analysis.token_flows = vec![
            // The sender pays 5 USDC in and gets 2 back; a mint to the pool
            flow(sender, pool, "5000000", 0),
            flow(pool, sender, "2000000", 1),
            flow(Address::ZERO, pool, "1000000", 2),
        ];

        let changes = net_balance_changes(&analysis);
        let find = |address: Address, token: Option<Address>| {
            changes
                .iter()
                .find(|c| c.address == address && c.token == token)
                .unwrap_or_else(|| panic!("no change for {address} {token:?} in {changes:?}"))
        };
        // Sender: -3 USDC, and 1 ETH of value plus the 0.00042 ETH fee
        assert_eq!(find(sender, Some(usdc)).raw, "-3000000");
        assert_eq!(
            find(sender, Some(usdc)).formatted.as_deref(),
            Some("-3.000000")
        );
        assert_eq!(find(sender, None).raw, "-1000420000000000000");
        // Pool: +3 from the sender net, +1 minted, and the 1 ETH of value
        assert_eq!(find(pool, Some(usdc)).raw, "+4000000");
        assert_eq!(
            find(pool, None).formatted.as_deref(),
            Some("+1.000000000000000000")
        );
        // The zero address is not reported
        assert!(changes.iter().all(|c| !c.address.is_zero()));
        assert_eq!(changes.len(), 4);
        // Without a trace, native lines are partial and token lines are not
        assert!(changes.iter().all(|c| c.partial == c.token.is_none()));

        // A failed transaction moves no value, but the fee is still paid
        analysis.status = false;
        analysis.token_flows.clear();
        let failed = net_balance_changes(&analysis);
        assert_eq!(failed.len(), 1);
        assert_eq!(failed[0].raw, "-420000000000000");

        // Flows that cancel out, and NFTs, are left out
        analysis.status = true;
        analysis.value = U256::ZERO;
        analysis.fee_wei = None;
        let mut nft = flow(sender, pool, "1", 0);
        nft.token_id = Some("7".to_string());
        analysis.token_flows = vec![
            flow(sender, pool, "10", 0),
            flow(pool, sender, "10", 1),
            nft,
        ];
        assert!(net_balance_changes(&analysis).is_empty());
    }

    /// A swap of ETH for a token through a router that wraps the ETH, and
    /// the reverse swap, where the router unwraps and pays the sender.
    #[test]
    fn wraps_and_traces_keep_routers_at_zero() {
        use crate::tx::types::{TokenFlow, WrapEvent};
        let sender = address!("d8da6bf26964af9d7eed9e03e53415d37aa96045");
        let router = address!("7a250d5630b4cf539739df2c5dacb4c659f2488d");
        let pair = address!("2222222222222222222222222222222222222222");
        let weth = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        let token = address!("3333333333333333333333333333333333333333");
        let one_eth = U256::from(1_000_000_000_000_000_000u128);
        let flow = |token, from, to, amount: U256, log_index| TokenFlow {
            token,
            token_label: None,
            from,
            from_label: None,
            to,
            to_label: None,
            amount: amount.to_string(),
            decimals: Some(18),
            token_id: None,
            log_index,
        };
        let net = |changes: &[BalanceChange], address: Address, token: Option<Address>| {
            changes
                .iter()
                .find(|c| c.address == address && c.token == token)
                .map(|c| c.raw.clone())
        };

        // ETH -> token: 1 ETH sent to the router, wrapped, paid to the pair
        let mut buy = make_test_analysis();
        buy.to = Some(router);
        buy.value = one_eth;
        buy.wraps = vec![WrapEvent {
            token: weth,
            account: router,
            amount: one_eth,
            deposit: true,
        }];
        buy.token_flows = vec![
            flow(weth, router, pair, one_eth, 1),
            flow(token, pair, sender, U256::from(500u64), 2),
        ];
        let changes = net_balance_changes(&buy);
        // The router ends with nothing in either asset
        assert_eq!(net(&changes, router, None), None);
        assert_eq!(net(&changes, router, Some(weth)), None);
        assert_eq!(
            net(&changes, pair, Some(weth)).as_deref(),
            Some("+1000000000000000000")
        );
        assert_eq!(
            net(&changes, weth, None).as_deref(),
            Some("+1000000000000000000")
        );
        assert_eq!(net(&changes, sender, Some(token)).as_deref(), Some("+500"));
        assert_eq!(
            net(&changes, sender, None).as_deref(),
            Some("-1000420000000000000")
        );

        // The same from a trace: same figures, and no longer partial
        buy.native_transfers = Some(vec![(sender, router, one_eth), (router, weth, one_eth)]);
        let traced = net_balance_changes(&buy);
        assert_eq!(net(&traced, router, None), None);
        assert_eq!(
            net(&traced, sender, None).as_deref(),
            Some("-1000420000000000000")
        );
        assert!(traced.iter().all(|c| !c.partial));

        // token -> ETH: the router unwraps and pays the sender internally
        let mut sell = make_test_analysis();
        sell.to = Some(router);
        sell.value = U256::ZERO;
        sell.wraps = vec![WrapEvent {
            token: weth,
            account: router,
            amount: one_eth,
            deposit: false,
        }];
        sell.token_flows = vec![
            flow(token, sender, pair, U256::from(500u64), 0),
            flow(weth, pair, router, one_eth, 1),
        ];
        // Without a trace the payout to the sender is unknown: the router
        // appears to keep the ETH, and that line is flagged partial
        let untraced = net_balance_changes(&sell);
        assert_eq!(net(&untraced, router, Some(weth)), None);
        assert_eq!(
            net(&untraced, router, None).as_deref(),
            Some("+1000000000000000000")
        );
        assert!(untraced
            .iter()
            .filter(|c| c.token.is_none())
            .all(|c| c.partial));
        // With the trace the sender gets the ETH less the fee, the router nothing
        sell.native_transfers = Some(vec![(weth, router, one_eth), (router, sender, one_eth)]);
        let traced = net_balance_changes(&sell);
        assert_eq!(net(&traced, router, None), None);
        assert_eq!(
            net(&traced, sender, None).as_deref(),
            Some("+999580000000000000")
        );
        assert_eq!(
            net(&traced, weth, None).as_deref(),
            Some("-1000000000000000000")
        );

        // A wrapped token that also logs its mint as a Transfer is counted once
        let mut both = make_test_analysis();
        both.to = Some(weth);
        both.value = one_eth;
        both.wraps = vec![WrapEvent {
            token: weth,
            account: sender,
            amount: one_eth,
            deposit: true,
        }];
        both.token_flows = vec![flow(weth, Address::ZERO, sender, one_eth, 0)];
        let changes = net_balance_changes(&both);
        assert_eq!(
            net(&changes, sender, Some(weth)).as_deref(),
            Some("+1000000000000000000")
        );
    }

    #[test]
    fn refresh_sets_the_note_that_matches_how_native_was_worked_out() {
        let mut analysis = make_test_analysis();
        refresh_balance_changes(&mut analysis);
        assert_eq!(analysis.notes, vec![PARTIAL_NATIVE_NOTE.to_string()]);
        analysis.native_transfers = Some(vec![]);
        refresh_balance_changes(&mut analysis);
        assert_eq!(analysis.notes, vec![TRACED_NATIVE_NOTE.to_string()]);
        assert!(analysis.balance_changes.iter().all(|c| !c.partial));
    }

    #[test]
    fn flow_mermaid_draws_tokens_wraps_and_native_transfers() {
        use crate::tx::types::{TokenFlow, WrapEvent};
        let sender = address!("d8da6bf26964af9d7eed9e03e53415d37aa96045");
        let router = address!("7a250d5630b4cf539739df2c5dacb4c659f2488d");
        let weth = address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2");
        let one_eth = U256::from(1_000_000_000_000_000_000u128);
        let mut analysis = make_test_analysis();
        analysis.to = Some(router);
        analysis.value = one_eth;
        analysis.wraps = vec![WrapEvent {
            token: weth,
            account: router,
            amount: one_eth,
            deposit: true,
        }];
        analysis.token_flows = vec![TokenFlow {
            token: weth,
            token_label: Some("WE\"TH|x".to_string()),
            from: router,
            from_label: None,
            to: sender,
            to_label: None,
            amount: "1500000000000000000".to_string(),
            decimals: Some(18),
            token_id: None,
            log_index: 0,
        }];

        let untraced = flow_mermaid(&analysis);
        assert!(untraced.starts_with("flowchart LR\n"), "{untraced}");
        assert!(
            untraced.contains("n0[\"sender<br/>0xd8da…6045\"]"),
            "{untraced}"
        );
        assert!(
            untraced.contains("Uniswap V2 Router<br/>0x7a25…488d"),
            "{untraced}"
        );
        // Quotes and pipes from a token symbol cannot break the diagram
        assert!(untraced.contains("1.5 WE#quot;TH#124;x"), "{untraced}");
        assert!(untraced.contains("1 native, wrapped to WETH"), "{untraced}");
        assert!(untraced.contains("-->|\"1 native\"|"), "{untraced}");
        assert!(untraced.contains("No call trace"), "{untraced}");

        analysis.native_transfers = Some(vec![(sender, router, one_eth), (router, weth, one_eth)]);
        let traced = flow_mermaid(&analysis);
        assert!(!traced.contains("wrapped to"), "{traced}");
        assert!(!traced.contains("No call trace"), "{traced}");
        assert_eq!(traced.matches("1 native").count(), 2, "{traced}");
    }

    #[test]
    fn format_analysis_shows_time_fee_revert_and_net_changes() {
        let mut analysis = make_test_analysis();
        analysis.time = Some("2024-01-01T00:00:11Z".to_string());
        analysis.status = false;
        analysis.revert = Some(crate::abi::revert::unknown(&[0xde, 0xad, 0xbe, 0xef]));
        analysis.balance_changes = net_balance_changes(&analysis);
        analysis.notes = vec!["a note".to_string()];
        let out = format_analysis(&analysis);
        for expected in [
            "Block: 18500000 (2024-01-01T00:00:11Z)",
            "Index in block: 3",
            "Fee: 0.000420000000000000 (gas price 20000000000 wei)",
            "Reverted: custom error 0xdeadbeef (signature not found)",
            "Net Balance Changes (1):",
            "-0.000420000000000000 native (partial)",
            "Note: a note",
        ] {
            assert!(out.contains(expected), "missing {expected:?} in:\n{out}");
        }
    }

    #[test]
    fn test_format_analysis_failed_tx() {
        let mut analysis = make_test_analysis();
        analysis.status = false;
        let output = format_analysis(&analysis);

        assert!(output.contains("Status: Failed"));
    }

    #[test]
    fn test_format_analysis_contract_creation() {
        let mut analysis = make_test_analysis();
        analysis.to = None;
        let output = format_analysis(&analysis);

        assert!(output.contains("Contract Creation"));
    }

    // LOW-007: Edge case tests

    #[test]
    fn test_u256_to_eth_f64_normal() {
        // 1 ETH = 1e18 wei
        let one_eth = U256::from(1_000_000_000_000_000_000u128);
        let result = u256_to_eth_f64(&one_eth);
        assert!((result - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_u256_to_eth_f64_small() {
        // 1 wei
        let one_wei = U256::from(1u64);
        let result = u256_to_eth_f64(&one_wei);
        assert!(result > 0.0 && result < 1e-17);
    }

    #[test]
    fn test_u256_to_eth_f64_zero() {
        let zero = U256::ZERO;
        let result = u256_to_eth_f64(&zero);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_u256_to_eth_f64_large() {
        // Value larger than u128::MAX
        let large = U256::MAX;
        let result = u256_to_eth_f64(&large);
        // Should not panic, and should return a large positive number
        assert!(result > 0.0);
        assert!(result.is_finite() || result == f64::MAX);
    }

    #[test]
    fn test_format_token_amount_normal() {
        // 1.23 tokens (with 18 decimals)
        let amount = "1230000000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "1.23");
    }

    #[test]
    fn test_format_token_amount_small() {
        // 0.01 tokens
        let amount = "10000000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "0.01");
    }

    #[test]
    fn test_format_token_amount_very_small() {
        // Very small amount (less than 0.01)
        let amount = "1000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "0.000001");
    }

    #[test]
    fn test_format_token_amount_empty() {
        let result = format_token_amount("", 18);
        assert_eq!(result, "0.00");
    }

    #[test]
    fn test_format_token_amount_invalid() {
        let result = format_token_amount("not_a_number", 18);
        assert_eq!(result, "0.00");
    }

    #[test]
    fn test_format_token_amount_large() {
        // 1 million tokens
        let amount = "1000000000000000000000000";
        let result = format_token_amount(amount, 18);
        assert_eq!(result, "1000000.00");
    }

    #[test]
    fn test_format_analysis_many_events() {
        let mut analysis = make_test_analysis();
        // Add 25 events (more than the 20 limit)
        for i in 0..25 {
            analysis.events.push(AnalyzedEvent {
                log_index: i as u64,
                address: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
                address_label: Some("WETH".to_string()),
                name: Some("Transfer".to_string()),
                signature: Some("Transfer(address,address,uint256)".to_string()),
                params: HashMap::new(),
                topic0: B256::ZERO,
                is_transfer: true,
            });
        }

        let output = format_analysis(&analysis);
        assert!(output.contains("... and 5 more"));
    }

    #[test]
    fn test_format_analysis_many_token_flows() {
        use crate::tx::types::TokenFlow;

        let mut analysis = make_test_analysis();
        // Add 20 token flows (more than the 15 limit)
        for i in 0..20 {
            analysis.token_flows.push(TokenFlow {
                token: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
                token_label: Some("USDC".to_string()),
                from: address!("d8da6bf26964af9d7eed9e03e53415d37aa96045"),
                from_label: None,
                to: address!("c02aaa39b223fe8d0a0e5c4f27ead9083c756cc2"),
                to_label: None,
                amount: "1000000".to_string(),
                decimals: None,
                token_id: None,
                log_index: i as u64,
            });
        }

        let output = format_analysis(&analysis);
        assert!(output.contains("... and 5 more"));
    }

    #[test]
    fn test_format_analysis_with_function_call() {
        let mut analysis = make_test_analysis();
        analysis.function_call = Some(FunctionCall {
            selector: "0xa9059cbb".to_string(),
            name: Some("transfer".to_string()),
            signature: Some("transfer(address,uint256)".to_string()),
            params: vec![
                FunctionParam {
                    name: "to".to_string(),
                    ty: "address".to_string(),
                    value: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
                },
                FunctionParam {
                    name: "amount".to_string(),
                    ty: "uint256".to_string(),
                    value: "1000000000000000000".to_string(),
                },
            ],
        });

        let output = format_analysis(&analysis);
        assert!(output.contains("Function Call:"));
        assert!(output.contains("transfer"));
        assert!(output.contains("0xa9059cbb"));
        assert!(output.contains("Parameters:"));
    }

    #[test]
    fn test_format_analysis_empty_contracts() {
        let mut analysis = make_test_analysis();
        analysis.contracts.clear();
        let output = format_analysis(&analysis);

        // Should still have the "Contracts Involved:" header
        assert!(output.contains("Contracts Involved:"));
    }

    #[test]
    fn test_format_analysis_large_value() {
        let mut analysis = make_test_analysis();
        // Set a very large value (> u128::MAX would overflow, but let's test a large u128)
        analysis.value = U256::from(u128::MAX);
        let output = format_analysis(&analysis);

        // Should not panic and should contain ETH formatting
        assert!(output.contains("ETH"));
    }
}
