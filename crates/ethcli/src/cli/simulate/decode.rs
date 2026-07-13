//! Decoded pretty-printing for `callTracer` traces from the debug backend.
//!
//! Produces cast-run-style output (labeled contracts, decoded function args
//! and events) without replaying the transaction: the call tree comes from
//! `debug_traceTransaction`, and decoding uses Etherscan ABIs (disk-cached),
//! the local signature cache, and 4byte.directory fallbacks.
//!
//! Three passes keep everything simple and bounded:
//!   1. walk the frame tree collecting unique addresses / selectors / topics
//!   2. resolve ABIs, contract names, and fallback signatures (async, capped)
//!   3. render the tree synchronously from the resolved tables

use std::collections::{HashMap, HashSet};

use alloy::dyn_abi::{DynSolType, DynSolValue, EventExt, FunctionExt, JsonAbiExt};
use alloy::json_abi::{Function, JsonAbi};
use alloy::primitives::{B256, U256};

use crate::abi::AbiFetcher;
use crate::config::Chain;
use crate::tx::addresses;

/// Caps so a pathological trace cannot trigger unbounded network lookups.
const MAX_ABI_LOOKUPS: usize = 24;
const MAX_SELECTOR_LOOKUPS: usize = 48;
const MAX_EVENT_LOOKUPS: usize = 48;
const MAX_RENDERED_FRAMES: usize = 600;

/// Resolved lookup tables shared by the synchronous renderer.
struct DecodeTables {
    /// address (lowercase) -> verified ABI
    abis: HashMap<String, JsonAbi>,
    /// address (lowercase) -> display name
    names: HashMap<String, String>,
    /// selector (0x + 8 hex, lowercase) -> text signature from 4byte
    selectors: HashMap<String, String>,
    /// topic0 (lowercase) -> event text signature from 4byte
    events: HashMap<String, String>,
}

pub struct TraceDecoder {
    fetcher: AbiFetcher,
    chain: Chain,
}

impl TraceDecoder {
    pub fn new(chain: Chain, etherscan_key: Option<String>) -> anyhow::Result<Self> {
        Ok(Self {
            fetcher: AbiFetcher::new(etherscan_key)?,
            chain,
        })
    }

    /// Render a callTracer root frame as a decoded, indented call tree.
    pub async fn render(&self, root: &serde_json::Value) -> String {
        let tables = self.resolve_tables(root).await;
        let mut lines = Vec::new();
        render_frame(root, &tables, "", "", true, &mut lines);
        if lines.len() >= MAX_RENDERED_FRAMES {
            lines.push(format!(
                "… trace truncated at {} frames",
                MAX_RENDERED_FRAMES
            ));
        }
        lines.join("\n")
    }

    /// Pass 1 + 2: collect unique identifiers, then resolve them.
    async fn resolve_tables(&self, root: &serde_json::Value) -> DecodeTables {
        let mut addrs: Vec<String> = Vec::new();
        let mut seen_addrs = HashSet::new();
        let mut selectors: Vec<(String, String)> = Vec::new(); // (addr, selector)
        let mut seen_selectors = HashSet::new();
        let mut topics: Vec<(String, String)> = Vec::new(); // (addr, topic0)
        let mut seen_topics = HashSet::new();

        collect_ids(
            root,
            &mut addrs,
            &mut seen_addrs,
            &mut selectors,
            &mut seen_selectors,
            &mut topics,
            &mut seen_topics,
        );

        let mut tables = DecodeTables {
            abis: HashMap::new(),
            names: HashMap::new(),
            selectors: HashMap::new(),
            events: HashMap::new(),
        };

        // Static label DB first (free), then Etherscan ABI + contract name.
        let mut first_addr = true;
        for addr in addrs.iter().take(MAX_ABI_LOOKUPS) {
            if precompile_name(addr).is_some() {
                continue;
            }
            if let Ok(parsed) = addr.parse::<alloy::primitives::Address>() {
                if let Some(label) = addresses::get_label(&parsed) {
                    tables.names.insert(addr.clone(), label.to_string());
                }
            }

            // Spread Etherscan requests out enough to stay under the
            // free-tier rate limit (getabi + getsourcecode per address).
            if !first_addr {
                tokio::time::sleep(std::time::Duration::from_millis(220)).await;
            }
            first_addr = false;

            // ABI fetch is disk-cached inside AbiFetcher; failures are fine
            // (unverified contracts fall back to 4byte signatures below).
            if let Ok(abi) = self.fetcher.fetch_from_etherscan(self.chain, addr).await {
                tables.abis.insert(addr.clone(), abi);
            }
            if !tables.names.contains_key(addr) {
                if let Ok(meta) = self.fetcher.get_contract_metadata(self.chain, addr).await {
                    if let Some(name) = meta.name {
                        tables.names.insert(addr.clone(), name);
                    }
                }
            }
            // Token symbol via RPC beats no label at all (and survives
            // Etherscan rate limits); a revert on non-tokens is harmless.
            if !tables.names.contains_key(addr) {
                if let Ok(meta) = self.fetcher.get_token_metadata_rpc(self.chain, addr).await {
                    if let Some(symbol) = meta.symbol {
                        tables.names.insert(addr.clone(), symbol);
                    }
                }
            }
        }

        // 4byte fallbacks only for selectors the fetched ABIs cannot explain.
        let mut selector_budget = MAX_SELECTOR_LOOKUPS;
        for (addr, selector) in &selectors {
            if selector_budget == 0 {
                break;
            }
            if abi_has_selector(tables.abis.get(addr), selector) {
                continue;
            }
            if tables.selectors.contains_key(selector) {
                continue;
            }
            selector_budget -= 1;
            if let Some(sig) = self.fetcher.lookup_selector(selector).await {
                tables.selectors.insert(selector.clone(), sig);
            }
        }

        let mut event_budget = MAX_EVENT_LOOKUPS;
        for (addr, topic0) in &topics {
            if event_budget == 0 {
                break;
            }
            if abi_has_event(tables.abis.get(addr), topic0) {
                continue;
            }
            if tables.events.contains_key(topic0) {
                continue;
            }
            event_budget -= 1;
            if let Some(sig) = self.fetcher.lookup_event(topic0).await {
                tables.events.insert(topic0.clone(), sig);
            }
        }

        tables
    }
}

// ---------------------------------------------------------------------------
// Pass 1: identifier collection
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn collect_ids(
    frame: &serde_json::Value,
    addrs: &mut Vec<String>,
    seen_addrs: &mut HashSet<String>,
    selectors: &mut Vec<(String, String)>,
    seen_selectors: &mut HashSet<String>,
    topics: &mut Vec<(String, String)>,
    seen_topics: &mut HashSet<String>,
) {
    let to = frame["to"].as_str().unwrap_or("").to_lowercase();
    if !to.is_empty() && seen_addrs.insert(to.clone()) {
        addrs.push(to.clone());
    }

    if let Some(input) = frame["input"].as_str() {
        if input.len() >= 10 {
            let sel = input[..10].to_lowercase();
            if seen_selectors.insert(format!("{to}:{sel}")) {
                selectors.push((to.clone(), sel));
            }
        }
    }

    if let Some(logs) = frame["logs"].as_array() {
        for log in logs {
            let log_addr = log["address"].as_str().unwrap_or("").to_lowercase();
            if let Some(topic0) = log["topics"]
                .as_array()
                .and_then(|t| t.first())
                .and_then(|t| t.as_str())
            {
                let topic0 = topic0.to_lowercase();
                if seen_topics.insert(format!("{log_addr}:{topic0}")) {
                    topics.push((log_addr, topic0));
                }
            }
        }
    }

    if let Some(calls) = frame["calls"].as_array() {
        for call in calls {
            collect_ids(
                call,
                addrs,
                seen_addrs,
                selectors,
                seen_selectors,
                topics,
                seen_topics,
            );
        }
    }
}

fn abi_has_selector(abi: Option<&JsonAbi>, selector: &str) -> bool {
    let Some(abi) = abi else { return false };
    let Some(sel_bytes) = parse_hex_bytes(selector) else {
        return false;
    };
    if sel_bytes.len() != 4 {
        return false;
    }
    abi.functions()
        .any(|f| f.selector().as_slice() == sel_bytes.as_slice())
}

fn abi_has_event(abi: Option<&JsonAbi>, topic0: &str) -> bool {
    let Some(abi) = abi else { return false };
    let Some(topic_bytes) = parse_hex_bytes(topic0) else {
        return false;
    };
    abi.events()
        .any(|e| e.selector().as_slice() == topic_bytes.as_slice())
}

// ---------------------------------------------------------------------------
// Pass 3: rendering
// ---------------------------------------------------------------------------

/// A child item of a frame: subcall or emitted log, ordered by callTracer's
/// `position` field (logs carry the number of subcalls emitted before them).
enum Child<'a> {
    Call(&'a serde_json::Value),
    Log(&'a serde_json::Value),
}

fn ordered_children(frame: &serde_json::Value) -> Vec<Child<'_>> {
    let calls: Vec<&serde_json::Value> = frame["calls"]
        .as_array()
        .map(|c| c.iter().collect())
        .unwrap_or_default();
    let logs: Vec<&serde_json::Value> = frame["logs"]
        .as_array()
        .map(|l| l.iter().collect())
        .unwrap_or_default();

    // logs[i].position = index into calls before which the log belongs
    let mut children: Vec<Child> = Vec::with_capacity(calls.len() + logs.len());
    let mut log_idx = 0;
    for (call_pos, call) in calls.iter().enumerate() {
        while log_idx < logs.len() {
            let pos = logs[log_idx]["position"]
                .as_str()
                .and_then(parse_hex_usize)
                .unwrap_or(usize::MAX);
            if pos <= call_pos {
                children.push(Child::Log(logs[log_idx]));
                log_idx += 1;
            } else {
                break;
            }
        }
        children.push(Child::Call(call));
    }
    while log_idx < logs.len() {
        children.push(Child::Log(logs[log_idx]));
        log_idx += 1;
    }
    children
}

fn render_frame(
    frame: &serde_json::Value,
    tables: &DecodeTables,
    line_prefix: &str,
    child_prefix: &str,
    is_root: bool,
    lines: &mut Vec<String>,
) {
    if lines.len() >= MAX_RENDERED_FRAMES {
        return;
    }

    lines.push(format!("{}{}", line_prefix, frame_headline(frame, tables)));

    let children = ordered_children(frame);
    let return_line = frame_return_line(frame, tables);
    let total = children.len() + usize::from(return_line.is_some());
    let mut idx = 0;

    for child in children {
        idx += 1;
        let last = idx == total;
        let (lp, cp) = branch_prefixes(child_prefix, last);
        match child {
            Child::Call(call) => render_frame(call, tables, &lp, &cp, false, lines),
            Child::Log(log) => {
                if lines.len() < MAX_RENDERED_FRAMES {
                    lines.push(format!("{}{}", lp, format_log(log, tables)));
                }
            }
        }
    }

    if let Some(ret) = return_line {
        let (lp, _) = branch_prefixes(child_prefix, true);
        if is_root {
            lines.push(format!("{}{}", child_prefix, ret));
        } else {
            lines.push(format!("{}{}", lp, ret));
        }
    }
}

fn branch_prefixes(child_prefix: &str, last: bool) -> (String, String) {
    if last {
        (format!("{child_prefix}└─ "), format!("{child_prefix}   "))
    } else {
        (format!("{child_prefix}├─ "), format!("{child_prefix}│  "))
    }
}

/// `[gas] CALL Name(0x1234…5678)::fn(arg: value, …) [value: 0.5 ETH] !revert`
fn frame_headline(frame: &serde_json::Value, tables: &DecodeTables) -> String {
    let call_type = frame["type"].as_str().unwrap_or("CALL");
    let to = frame["to"].as_str().unwrap_or("").to_lowercase();
    let gas_used = frame["gasUsed"]
        .as_str()
        .and_then(parse_hex_u64)
        .unwrap_or(0);

    let target = display_name(&to, tables);
    let call = format_call(&to, frame["input"].as_str().unwrap_or(""), tables);

    let mut line = format!("[{}] {} {}::{}", gas_used, call_type, target, call);

    if let Some(value) = frame["value"].as_str().and_then(parse_hex_u256) {
        if !value.is_zero() {
            line.push_str(&format!(" [value: {} ETH]", format_eth(value)));
        }
    }

    if let Some(err) = frame["error"].as_str() {
        let reason = frame["revertReason"]
            .as_str()
            .map(|r| format!(": {r}"))
            .or_else(|| {
                decode_revert_reason(frame["output"].as_str().unwrap_or(""))
                    .map(|r| format!(": {r}"))
            })
            .unwrap_or_default();
        line.push_str(&format!(" !! {}{}", err, reason));
    }

    line
}

/// Decoded return values for the frame, when the ABI knows the outputs.
fn frame_return_line(frame: &serde_json::Value, tables: &DecodeTables) -> Option<String> {
    if frame["error"].as_str().is_some() {
        return Some("← [Revert]".to_string());
    }
    let output = frame["output"].as_str().unwrap_or("0x");
    if output == "0x" || output.is_empty() {
        return None;
    }

    let to = frame["to"].as_str().unwrap_or("").to_lowercase();
    let input = frame["input"].as_str().unwrap_or("");
    if input.len() >= 10 {
        if let Some(abi) = tables.abis.get(&to) {
            if let Some(sel) = parse_hex_bytes(&input[..10]) {
                if let Some(func) = abi
                    .functions()
                    .find(|f| f.selector().as_slice() == sel.as_slice())
                {
                    if let Some(out_bytes) = parse_hex_bytes(output) {
                        if let Ok(values) = func.abi_decode_output(&out_bytes) {
                            if !values.is_empty() {
                                let rendered: Vec<String> =
                                    values.iter().map(format_value).collect();
                                return Some(format!("← {}", rendered.join(", ")));
                            }
                        }
                    }
                }
            }
        }
    }

    Some(format!("← {}", truncate_hex(output)))
}

/// Decoded `fn(name: value, …)` for a frame's calldata.
fn format_call(to: &str, input: &str, tables: &DecodeTables) -> String {
    if input.len() < 10 {
        return if input == "0x" || input.is_empty() {
            "(receive)".to_string()
        } else {
            truncate_hex(input)
        };
    }

    let selector = input[..10].to_lowercase();
    let calldata = parse_hex_bytes(input).unwrap_or_default();

    // Precompile input is not selector-prefixed calldata
    if precompile_name(to).is_some() {
        return format!("({} bytes)", calldata.len());
    }

    // 1. Verified ABI: names + full arg decode (handles tuples/structs)
    if let Some(abi) = tables.abis.get(to) {
        if let Some(sel) = parse_hex_bytes(&selector) {
            if let Some(func) = abi
                .functions()
                .find(|f| f.selector().as_slice() == sel.as_slice())
            {
                return format!("{}({})", func.name, decode_args(func, &calldata[4..]));
            }
        }
    }

    // 2. 4byte text signature: parse into a Function, decode args (no names)
    if let Some(sig) = tables.selectors.get(&selector) {
        if let Ok(func) = Function::parse(sig) {
            return format!("{}({})", func.name, decode_args(&func, &calldata[4..]));
        }
        if let Some((name, _)) = split_signature(sig) {
            return format!("{}({} bytes)", name, calldata.len().saturating_sub(4));
        }
    }

    // 3. Raw selector
    format!("<{}>({} bytes)", selector, calldata.len().saturating_sub(4))
}

/// Decode ABI-encoded args and render `name: value, …` (bounded per element).
fn decode_args(func: &Function, data: &[u8]) -> String {
    if func.inputs.is_empty() {
        return String::new();
    }
    match func.abi_decode_input(data) {
        Ok(values) => values
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let name = func.inputs.get(i).map(|p| p.name.as_str()).unwrap_or("");
                if name.is_empty() {
                    format_value(v)
                } else {
                    format!("{}: {}", name, format_value(v))
                }
            })
            .collect::<Vec<_>>()
            .join(", "),
        Err(_) => format!("{} bytes", data.len()),
    }
}

/// `emit Transfer(from: 0x…, to: 0x…, value: 1234 [1.2e3])`
fn format_log(log: &serde_json::Value, tables: &DecodeTables) -> String {
    let addr = log["address"].as_str().unwrap_or("").to_lowercase();
    let topics: Vec<&str> = log["topics"]
        .as_array()
        .map(|t| t.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();
    let data = parse_hex_bytes(log["data"].as_str().unwrap_or("0x")).unwrap_or_default();

    let Some(topic0) = topics.first() else {
        return "emit <anonymous>".to_string();
    };
    let topic0 = topic0.to_lowercase();

    // 1. Verified ABI event (indexed flags + names known, tuples handled)
    if let Some(abi) = tables.abis.get(&addr) {
        if let Some(topic_bytes) = parse_hex_bytes(&topic0) {
            if let Some(event) = abi
                .events()
                .find(|e| e.selector().as_slice() == topic_bytes.as_slice())
            {
                let topic_b256: Vec<B256> = topics
                    .iter()
                    .filter_map(|t| {
                        parse_hex_bytes(t).and_then(|b| B256::try_from(b.as_slice()).ok())
                    })
                    .collect();
                if topic_b256.len() == topics.len() {
                    if let Ok(decoded) = event.decode_log_parts(topic_b256, &data) {
                        let mut indexed = decoded.indexed.iter();
                        let mut body = decoded.body.iter();
                        let rendered: Vec<String> = event
                            .inputs
                            .iter()
                            .filter_map(|p| {
                                let value = if p.indexed {
                                    indexed.next()
                                } else {
                                    body.next()
                                }?;
                                let label = if p.name.is_empty() {
                                    p.ty.clone()
                                } else {
                                    p.name.clone()
                                };
                                Some(format!("{}: {}", label, format_value(value)))
                            })
                            .collect();
                        return format!("emit {}({})", event.name, rendered.join(", "));
                    }
                }
            }
        }
    }

    // 2. 4byte event signature (assume first N params indexed, N = topics-1)
    if let Some(sig) = tables.events.get(&topic0) {
        if let Some((name, types)) = split_signature(sig) {
            let n_indexed = topics.len() - 1;
            let params: Vec<(String, String, bool)> = types
                .iter()
                .enumerate()
                .map(|(i, ty)| (String::new(), ty.clone(), i < n_indexed))
                .collect();
            if let Some(args) = decode_event_args(&params, &topics[1..], &data) {
                return format!("emit {}({})", name, args);
            }
            return format!("emit {}", name);
        }
    }

    format!("emit <{}…>", &topic0[..10.min(topic0.len())])
}

/// Decode event params from indexed topics + data section, in declaration order.
fn decode_event_args(
    params: &[(String, String, bool)],
    topics: &[&str],
    data: &[u8],
) -> Option<String> {
    let data_types: Vec<DynSolType> = params
        .iter()
        .filter(|(_, _, indexed)| !indexed)
        .filter_map(|(_, ty, _)| ty.parse::<DynSolType>().ok())
        .collect();
    let n_data = params.iter().filter(|(_, _, i)| !i).count();
    if data_types.len() != n_data {
        return None;
    }

    let mut data_values = if data_types.is_empty() {
        Vec::new()
    } else {
        match DynSolType::Tuple(data_types).abi_decode(data) {
            Ok(DynSolValue::Tuple(values)) => values,
            _ => return None,
        }
    }
    .into_iter();

    let mut topic_iter = topics.iter();
    let mut rendered = Vec::with_capacity(params.len());

    for (i, (name, ty, indexed)) in params.iter().enumerate() {
        let value_str = if *indexed {
            let topic = topic_iter.next()?;
            format_indexed_topic(ty, topic)
        } else {
            format_value(&data_values.next()?)
        };
        let label = if name.is_empty() {
            format!("p{i}")
        } else {
            name.clone()
        };
        rendered.push(format!("{}: {}", label, value_str));
    }

    Some(rendered.join(", "))
}

/// Indexed params are one 32-byte topic; dynamic types are only their hash.
fn format_indexed_topic(ty: &str, topic: &str) -> String {
    let Some(bytes) = parse_hex_bytes(topic) else {
        return topic.to_string();
    };
    if bytes.len() != 32 {
        return topic.to_string();
    }
    match ty.parse::<DynSolType>() {
        Ok(DynSolType::String)
        | Ok(DynSolType::Bytes)
        | Ok(DynSolType::Array(_))
        | Ok(DynSolType::Tuple(_))
        | Ok(DynSolType::FixedArray(_, _)) => {
            format!("<hash {}…>", &topic[..10.min(topic.len())])
        }
        Ok(sol_ty) => match sol_ty.abi_decode(&bytes) {
            Ok(value) => format_value(&value),
            Err(_) => topic.to_string(),
        },
        Err(_) => topic.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Value formatting
// ---------------------------------------------------------------------------

/// Names for the standard precompiled contracts (0x01–0x0a).
fn precompile_name(addr: &str) -> Option<&'static str> {
    let stripped = addr.strip_prefix("0x")?.trim_start_matches('0');
    match stripped {
        "1" => Some("ecrecover"),
        "2" => Some("sha256"),
        "3" => Some("ripemd160"),
        "4" => Some("identity"),
        "5" => Some("modexp"),
        "6" => Some("ecadd"),
        "7" => Some("ecmul"),
        "8" => Some("ecpairing"),
        "9" => Some("blake2f"),
        "a" => Some("kzg-point-eval"),
        _ => None,
    }
}

fn display_name(addr: &str, tables: &DecodeTables) -> String {
    if let Some(name) = precompile_name(addr) {
        return format!("precompile::{}", name);
    }
    match tables.names.get(addr) {
        Some(name) => format!("{}({})", name, short_addr(addr)),
        None => addr.to_string(),
    }
}

fn short_addr(addr: &str) -> String {
    if addr.len() >= 12 {
        format!("{}…{}", &addr[..6], &addr[addr.len() - 4..])
    } else {
        addr.to_string()
    }
}

/// Format one decoded Solidity value, truncating blobs and long arrays.
fn format_value(value: &DynSolValue) -> String {
    match value {
        DynSolValue::Bool(b) => b.to_string(),
        DynSolValue::Int(i, _) => i.to_string(),
        DynSolValue::Uint(u, _) => {
            let s = u.to_string();
            if s.len() > 9 {
                format!("{} [{}]", s, scientific(&s))
            } else {
                s
            }
        }
        DynSolValue::Address(a) => format!("{a:#x}"),
        DynSolValue::String(s) => format!("\"{}\"", s.chars().take(64).collect::<String>()),
        DynSolValue::Bytes(b) => truncate_hex(&format!("0x{}", hex::encode(b))),
        DynSolValue::FixedBytes(b, size) => {
            truncate_hex(&format!("0x{}", hex::encode(&b.as_slice()[..*size])))
        }
        DynSolValue::Array(items) | DynSolValue::FixedArray(items) => {
            if items.len() > 4 {
                let head: Vec<String> = items.iter().take(2).map(format_value).collect();
                format!("[{}, … {} items]", head.join(", "), items.len())
            } else {
                let all: Vec<String> = items.iter().map(format_value).collect();
                format!("[{}]", all.join(", "))
            }
        }
        DynSolValue::Tuple(items) => {
            let all: Vec<String> = items.iter().map(format_value).collect();
            format!("({})", all.join(", "))
        }
        other => format!("{other:?}"),
    }
}

/// `18944228725135839340510` -> `1.894e22`
fn scientific(digits: &str) -> String {
    let exp = digits.len() - 1;
    let mantissa = if digits.len() > 4 {
        &digits[..4]
    } else {
        digits
    };
    format!("{}.{}e{}", &mantissa[..1], &mantissa[1..], exp)
}

fn format_eth(wei: U256) -> String {
    let ether = wei / U256::from(10u64.pow(14));
    let whole = ether / U256::from(10_000u64);
    let frac = (ether % U256::from(10_000u64)).to::<u64>();
    format!("{}.{:04}", whole, frac)
}

fn decode_revert_reason(output: &str) -> Option<String> {
    let bytes = parse_hex_bytes(output)?;
    // Error(string) selector 0x08c379a0
    if bytes.len() >= 68 && bytes[..4] == [0x08, 0xc3, 0x79, 0xa0] {
        if let Ok(DynSolValue::String(reason)) = DynSolType::String.abi_decode(&bytes[4..]) {
            return Some(reason);
        }
    }
    // Panic(uint256) selector 0x4e487b71
    if bytes.len() >= 36 && bytes[..4] == [0x4e, 0x48, 0x7b, 0x71] {
        if let Ok(DynSolValue::Uint(code, _)) = DynSolType::Uint(256).abi_decode(&bytes[4..]) {
            return Some(format!("Panic({:#x})", code));
        }
    }
    None
}

fn truncate_hex(hex_str: &str) -> String {
    let len = hex_str.len();
    if len <= 20 {
        hex_str.to_string()
    } else {
        format!("{}…({} bytes)", &hex_str[..14], (len - 2) / 2)
    }
}

/// `"transfer(address,uint256)"` -> `("transfer", ["address", "uint256"])`,
/// respecting nested parentheses/brackets in tuple types.
fn split_signature(sig: &str) -> Option<(String, Vec<String>)> {
    let open = sig.find('(')?;
    let name = sig[..open].to_string();
    let inner = sig.get(open + 1..sig.len().checked_sub(1)?)?;
    if inner.is_empty() {
        return Some((name, Vec::new()));
    }

    let mut params = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    for (i, c) in inner.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                params.push(inner[start..i].to_string());
                start = i + 1;
            }
            _ => {}
        }
    }
    params.push(inner[start..].to_string());
    Some((name, params))
}

fn parse_hex_bytes(s: &str) -> Option<Vec<u8>> {
    hex::decode(s.strip_prefix("0x").unwrap_or(s)).ok()
}

fn parse_hex_u64(s: &str) -> Option<u64> {
    u64::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok()
}

fn parse_hex_usize(s: &str) -> Option<usize> {
    usize::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok()
}

fn parse_hex_u256(s: &str) -> Option<U256> {
    U256::from_str_radix(s.strip_prefix("0x").unwrap_or(s), 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_flat_signature() {
        let (name, params) = split_signature("transfer(address,uint256)").unwrap();
        assert_eq!(name, "transfer");
        assert_eq!(params, vec!["address", "uint256"]);
    }

    #[test]
    fn splits_nested_tuple_signature() {
        let (name, params) =
            split_signature("claimMulti(address,(address,uint256,uint256,bytes32[])[])").unwrap();
        assert_eq!(name, "claimMulti");
        assert_eq!(
            params,
            vec!["address", "(address,uint256,uint256,bytes32[])[]"]
        );
    }

    #[test]
    fn scientific_hint() {
        assert_eq!(scientific("18944228725135839340510"), "1.894e22");
    }

    #[test]
    fn decodes_error_string_revert() {
        // Error("insufficient balance")
        let output = "0x08c379a000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000014696e73756666696369656e742062616c616e6365000000000000000000000000";
        assert_eq!(
            decode_revert_reason(output).as_deref(),
            Some("insufficient balance")
        );
    }

    #[test]
    fn truncates_long_hex() {
        let long = format!("0x{}", "ab".repeat(100));
        let out = truncate_hex(&long);
        assert!(out.ends_with("(100 bytes)"));
    }
}
