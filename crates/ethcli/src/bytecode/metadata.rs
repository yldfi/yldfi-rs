//! Compiler metadata (CBOR) in runtime bytecode, a linear opcode sweep, and
//! a compiler fingerprint built from both.
//!
//! solc appends a CBOR map followed by its two-byte big-endian length to the
//! runtime code; vyper (0.3.4 and later) appends a CBOR array the same way.
//! Code that embeds another contract's creation code (`new X`) also carries
//! X's metadata in the middle, so every block is found, not only the last.
//! A trailing length is never trusted on its own: hand-written contracts end
//! in arbitrary bytes, so a block counts only when the CBOR decodes exactly
//! up to the length field.

use serde::Serialize;

/// A decoded CBOR value (the subset compilers emit)
#[derive(Debug, Clone, PartialEq)]
pub enum Cbor {
    Uint(u64),
    Bytes(Vec<u8>),
    Text(String),
    Array(Vec<Cbor>),
    Map(Vec<(Cbor, Cbor)>),
    Bool(bool),
    Null,
}

/// Decode one CBOR item at `pos`, returning it and the position after it
fn decode_cbor(data: &[u8], pos: usize, depth: usize) -> Option<(Cbor, usize)> {
    if depth > 8 {
        return None;
    }
    let first = *data.get(pos)?;
    let major = first >> 5;
    let info = first & 0x1f;
    let (arg, mut next) = match info {
        0..=23 => (info as u64, pos + 1),
        24 => (*data.get(pos + 1)? as u64, pos + 2),
        25 => (
            u16::from_be_bytes([*data.get(pos + 1)?, *data.get(pos + 2)?]) as u64,
            pos + 3,
        ),
        26 => {
            let b = data.get(pos + 1..pos + 5)?;
            (u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as u64, pos + 5)
        }
        _ => return None,
    };
    match major {
        0 => Some((Cbor::Uint(arg), next)),
        2 | 3 => {
            let len = usize::try_from(arg).ok()?;
            // Hashes are at most 64 bytes, versions and keys far less
            if len > 64 {
                return None;
            }
            let raw = data.get(next..next.checked_add(len)?)?;
            next += len;
            if major == 2 {
                Some((Cbor::Bytes(raw.to_vec()), next))
            } else {
                Some((Cbor::Text(String::from_utf8(raw.to_vec()).ok()?), next))
            }
        }
        4 => {
            if arg > 16 {
                return None;
            }
            let mut items = Vec::new();
            for _ in 0..arg {
                let (item, after) = decode_cbor(data, next, depth + 1)?;
                items.push(item);
                next = after;
            }
            Some((Cbor::Array(items), next))
        }
        5 => {
            if arg > 16 {
                return None;
            }
            let mut entries = Vec::new();
            for _ in 0..arg {
                let (key, after) = decode_cbor(data, next, depth + 1)?;
                let (value, after) = decode_cbor(data, after, depth + 1)?;
                entries.push((key, value));
                next = after;
            }
            Some((Cbor::Map(entries), next))
        }
        7 => match first {
            0xf4 => Some((Cbor::Bool(false), pos + 1)),
            0xf5 => Some((Cbor::Bool(true), pos + 1)),
            0xf6 => Some((Cbor::Null, pos + 1)),
            _ => None,
        },
        _ => None,
    }
}

/// One metadata block found in the code
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MetadataBlock {
    /// Offset of the CBOR data
    pub start: usize,
    /// Offset just past the two length bytes
    pub end: usize,
    /// `solc`, `vyper` or `unknown`
    pub compiler: String,
    /// Compiler version, when the metadata records one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// `ipfs`, `bzzr0`, `bzzr1` or `none`
    pub hash_type: String,
    /// The metadata hash, hex
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,
    /// solc's `experimental` flag (ABI coder v2 before 0.8, or SMTChecker)
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub experimental: bool,
    /// True for the block that ends the code
    pub trailing: bool,
}

impl MetadataBlock {
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn is_empty(&self) -> bool {
        self.end == self.start
    }
}

/// A compiler version as solc and vyper write it: `X.Y.Z`, optionally
/// followed by `-prerelease` or `+commit.<hex>` parts of letters, digits
/// and dots. Anything else from the chain is not shown as a version
fn is_version_text(t: &str) -> bool {
    if t.is_empty() || t.len() > 40 {
        return false;
    }
    let core_end = t.find(['+', '-']).unwrap_or(t.len());
    let core: Vec<&str> = t[..core_end].split('.').collect();
    core.len() == 3
        && core
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 4 && p.chars().all(|c| c.is_ascii_digit()))
        && t[core_end..]
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '+' | '-'))
}

fn describe(cbor: &Cbor, start: usize, end: usize, trailing: bool) -> Option<MetadataBlock> {
    let mut block = MetadataBlock {
        start,
        end,
        compiler: "unknown".into(),
        version: None,
        hash_type: "none".into(),
        hash: None,
        experimental: false,
        trailing,
    };
    let version_from = |value: &Cbor| match value {
        Cbor::Bytes(b) if b.len() == 3 => Some(format!("{}.{}.{}", b[0], b[1], b[2])),
        // Untrusted text from the chain: a strict charset only
        Cbor::Text(t) => is_version_text(t).then(|| t.clone()),
        Cbor::Array(parts) => {
            let nums: Vec<String> = parts
                .iter()
                .filter_map(|p| match p {
                    Cbor::Uint(n) => Some(n.to_string()),
                    _ => None,
                })
                .collect();
            (nums.len() == parts.len() && !nums.is_empty()).then(|| nums.join("."))
        }
        _ => None,
    };
    let entries: &[(Cbor, Cbor)] = match cbor {
        Cbor::Map(entries) => entries,
        // vyper >= 0.3.10: [runtime size, data sizes, immutables size, {"vyper": [..]}]
        Cbor::Array(items) => match items.last() {
            Some(Cbor::Map(entries)) => entries,
            _ => return None,
        },
        _ => return None,
    };
    let mut known = false;
    for (key, value) in entries {
        let Cbor::Text(key) = key else { return None };
        match (key.as_str(), value) {
            ("ipfs" | "bzzr0" | "bzzr1", Cbor::Bytes(h)) if h.len() != 32 && h.len() != 34 => {
                return None
            }
            ("ipfs" | "bzzr0" | "bzzr1", Cbor::Bytes(hash)) => {
                block.hash_type = key.clone();
                block.hash = Some(hex::encode(hash));
                known = true;
            }
            ("solc", Cbor::Bytes(b)) if b.len() != 3 => return None,
            ("solc" | "vyper", Cbor::Text(t)) if t.len() > 40 => return None,
            ("solc", v) => {
                block.compiler = "solc".into();
                block.version = version_from(v);
                known = true;
            }
            ("vyper", v) => {
                block.compiler = "vyper".into();
                block.version = version_from(v);
                known = true;
            }
            ("experimental", Cbor::Bool(b)) => block.experimental = *b,
            _ => {}
        }
    }
    if block.compiler == "unknown" && block.hash_type.starts_with("bzzr") {
        // solc before 0.5.9 recorded only the swarm hash
        block.compiler = "solc".into();
    }
    known.then_some(block)
}

/// Try to read a metadata block starting at `start`
fn block_at(code: &[u8], start: usize, trailing_end: usize) -> Option<MetadataBlock> {
    let first = *code.get(start)?;
    // Maps of 1-5 entries, or a vyper array of 4
    if !matches!(first, 0xa1..=0xa5 | 0x84) {
        return None;
    }
    let (value, after) = decode_cbor(code, start, 0)?;
    let len_bytes = code.get(after..after + 2)?;
    let declared = u16::from_be_bytes([len_bytes[0], len_bytes[1]]) as usize;
    if declared != after - start {
        return None;
    }
    describe(&value, start, after + 2, after + 2 == trailing_end)
}

/// Every metadata block in `code`, in order
pub fn find_metadata_blocks(code: &[u8]) -> Vec<MetadataBlock> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i + 2 < code.len() {
        if let Some(block) = block_at(code, i, code.len()) {
            i = block.end;
            blocks.push(block);
        } else {
            i += 1;
        }
    }
    blocks
}

/// The metadata block that ends the code, if there is one
pub fn trailing_metadata(code: &[u8]) -> Option<MetadataBlock> {
    if code.len() < 2 {
        return None;
    }
    let declared = u16::from_be_bytes([code[code.len() - 2], code[code.len() - 1]]) as usize;
    let start = code.len().checked_sub(declared + 2)?;
    block_at(code, start, code.len())
}

/// A linear sweep over the code: each instruction's pc and opcode
///
/// A linear sweep cannot tell code from data embedded in it (jump tables,
/// constants, embedded contracts), so counts from it are upper bounds.
pub fn linear_sweep(code: &[u8]) -> Vec<(usize, u8)> {
    let mut out = Vec::with_capacity(code.len() / 2);
    let mut pc = 0;
    while pc < code.len() {
        let op = code[pc];
        out.push((pc, op));
        pc += 1 + push_size(op);
    }
    out
}

/// Immediate bytes taken by an opcode
pub fn push_size(op: u8) -> usize {
    if (0x60..=0x7f).contains(&op) {
        (op - 0x5f) as usize
    } else {
        0
    }
}

/// What can be told about the compiler from the runtime code alone
#[derive(Debug, Clone, Serialize)]
pub struct Fingerprint {
    pub code_size: usize,
    pub code_hash: String,
    /// The trailing metadata block, decoded
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<MetadataBlock>,
    /// Further metadata blocks inside the code (embedded contracts)
    pub embedded_metadata: Vec<MetadataBlock>,
    /// Lowest EVM version the opcodes in use require
    pub min_evm_version: String,
    /// The newer opcodes that decided it, with a count each
    pub evm_evidence: Vec<String>,
    /// Highest EVM version the code is consistent with, when it can be told:
    /// "paris" for solc ≥ 0.8.20 code with no PUSH0 anywhere in its
    /// executable part. A consistency bound, not proof (see
    /// `max_evm_version_confidence`)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_evm_version: Option<String>,
    /// "consistent" when `max_evm_version` is set: the code fits it, but a
    /// newer target that never needed the newer opcode would look the same
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_evm_version_confidence: Option<String>,
    /// Why each bound was set
    pub evm_version_reasons: Vec<String>,
    /// PUSH0 (0x5f) as an executed opcode, not inside PUSH data or metadata
    pub push0_executed: usize,
    /// PUSH0 opcodes in a linear sweep of the executable part (before the
    /// trailing metadata), reached or not
    pub push0_in_code: usize,
    /// Pipeline guess from code shape: "legacy", "via-ir" or "unknown"
    pub pipeline_guess: String,
    pub pipeline_evidence: Vec<String>,
    /// Distinct PUSH32 operands shaped like an address (12 zero bytes then
    /// 20 non-zero bytes). Immutable addresses look like this, and so do
    /// address constants, so this is only an upper bound on immutables.
    pub address_like_push32: usize,
    /// Optimizer runs inferred from how constants are written: "likely high
    /// (≥ ~5000)", "likely mid (≈1000-4000)", "likely low (≈200)",
    /// "likely < ~5000", "likely ≥ ~1000", "likely ≤ ~2000" or "unknown"
    pub optimizer_runs_hint: String,
    pub optimizer_runs_evidence: RunsEvidence,
    /// Coarse, approximate bucket from the hint: "low (≤ ~200)", "medium
    /// (~1000)", "high (≥ ~5000)", or "unknown" when the evidence is weak,
    /// mixed or of one class only
    pub optimizer_runs_bucket: String,
    /// Counts behind the bucket
    pub optimizer_runs_bucket_evidence: Vec<String>,
    /// Selectors found by the dispatcher scan
    pub selector_count: usize,
    /// Set when there is no selector dispatcher but reached code reads
    /// calldata: "no selector dispatcher: fallback-only / packed-calldata
    /// executor"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dispatcher: Option<String>,
    /// Reached CALLDATALOAD / CALLDATACOPY pcs, when `dispatcher` is set
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub calldata_reads: Vec<usize>,
    pub notes: Vec<String>,
}

/// Where constants were found written out (high runs) or computed (low runs)
#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub struct RunsEvidence {
    /// `PUSH32 0x08c379a0..` (Error(string) selector written out)
    pub error_selector_literal: Vec<usize>,
    /// `PUSH3 0x461bcd PUSH1 0xe5 SHL` (the same selector computed)
    pub error_selector_shifted: Vec<usize>,
    /// Other constants written out: PUSH32 of a 1-4 byte value followed by
    /// zero bytes, PUSH32 of mostly 0xff, PUSH20 of all 0xff
    pub other_literal: Vec<usize>,
    /// Other constants computed: `PUSHn v PUSH1 s SHL` (s ≥ 0x60), `PUSHn v NOT`
    pub other_shifted: Vec<usize>,
    /// `PUSH1 1 PUSH1 0xa0 SHL` (the address mask, `(1 << 160) - 1`). Not
    /// evidence either way: solc 0.8.23 keeps it computed at 10000 runs
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub address_mask_shifted: Vec<usize>,
}

/// Label used when the code has no selector dispatcher but reads calldata
pub const FALLBACK_ONLY: &str = "no selector dispatcher: fallback-only / packed-calldata executor";

/// Reached CALLDATALOAD / CALLDATACOPY pcs
pub fn calldata_reads(code: &[u8], visited: &[bool]) -> Vec<usize> {
    linear_sweep(code)
        .into_iter()
        .filter(|(pc, op)| matches!(op, 0x35 | 0x37) && visited.get(*pc).copied().unwrap_or(false))
        .map(|(pc, _)| pc)
        .collect()
}

/// Read solc's constant optimizer choices from reached code. With few runs
/// it computes wide constants (`PUSH3 0x461bcd PUSH1 0xe5 SHL`); with many
/// it writes them out (`PUSH32 0x08c379a0..`). Each constant switches at its
/// own run count: measured with solc 0.8.29 (legacy pipeline), selector and
/// address masks are written out from about 1000 runs, the `Error(string)`
/// selector only from about 5000.
pub fn optimizer_runs_hint(code: &[u8], visited: &[bool]) -> (String, RunsEvidence) {
    let ins: Vec<(usize, u8)> = linear_sweep(code);
    let seen = |pc: usize| visited.get(pc).copied().unwrap_or(false);
    let arg = |pc: usize, op: u8| {
        &code[(pc + 1).min(code.len())..(pc + 1 + push_size(op)).min(code.len())]
    };
    let mut ev = RunsEvidence::default();
    let shifts_reached = ins
        .iter()
        .any(|(pc, op)| (0x1b..=0x1d).contains(op) && seen(*pc));
    for (k, &(pc, op)) in ins.iter().enumerate() {
        if !seen(pc) {
            continue;
        }
        let a = arg(pc, op);
        if op == 0x7f && a.len() == 32 {
            let nz = a.iter().rposition(|b| *b != 0).map_or(0, |p| p + 1);
            if a[..4] == [0x08, 0xc3, 0x79, 0xa0] && nz == 4 {
                ev.error_selector_literal.push(pc);
            } else if (a[0] != 0 && nz <= 4) || a.iter().filter(|b| **b == 0xff).count() >= 28 {
                ev.other_literal.push(pc);
            }
        } else if op == 0x73 && a.iter().all(|b| *b == 0xff) {
            ev.other_literal.push(pc);
        } else if (0x60..=0x63).contains(&op) {
            let next = ins.get(k + 1).copied();
            let after = ins.get(k + 2).copied();
            if let (Some((p1, 0x60)), Some((_, 0x1b))) = (next, after) {
                let s = arg(p1, 0x60).first().copied().unwrap_or(0);
                if s >= 0x60 {
                    if a == [0x46, 0x1b, 0xcd] && s == 0xe5 {
                        ev.error_selector_shifted.push(pc);
                    } else if a == [0x01] && s == 0xa0 {
                        ev.address_mask_shifted.push(pc);
                    } else {
                        ev.other_shifted.push(pc);
                    }
                }
            } else if let (Some((_, 0x19)), false) = (next, a == [0x1f]) {
                // `PUSH1 0x1f NOT` (round down to 32) stays at any runs
                ev.other_shifted.push(pc);
            }
        }
    }
    (runs_from_evidence(&ev, shifts_reached).to_string(), ev)
}

/// The hint for a set of evidence. The `Error(string)` selector decides
/// high against low (one side must outnumber the other 3:1); the share of
/// other constants written out separates mid from low.
fn runs_from_evidence(ev: &RunsEvidence, shifts_reached: bool) -> &'static str {
    // Pre-Constantinople code has no shift opcodes and builds constants
    // with EXP, which is not read here
    if !shifts_reached {
        return "unknown";
    }
    let (l, s) = (
        ev.error_selector_literal.len(),
        ev.error_selector_shifted.len(),
    );
    let (lo, sh) = (ev.other_literal.len(), ev.other_shifted.len());
    let share = if lo + sh == 0 {
        None
    } else {
        Some(lo as f64 / (lo + sh) as f64)
    };
    // A bucket needs two classes of evidence to agree: the Error(string)
    // selector and the other wide constants
    if l > 0 && l >= 3 * s {
        if lo > 0 && sh <= lo {
            "likely high (≥ ~5000)"
        } else if lo + sh == 0 {
            "likely ≥ ~5000 (Error(string) selector only)"
        } else {
            "unknown"
        }
    } else if s > 0 && s >= 3 * l {
        match share {
            None => "likely < ~5000",
            Some(0.0) => "likely low (≈200)",
            Some(x) if x >= 0.1 => "likely mid (≈1000-4000)",
            Some(_) => "likely < ~5000",
        }
    } else if l == 0 && s == 0 {
        match share {
            Some(x) if x >= 0.9 => "likely ≥ ~1000",
            Some(0.0) => "likely ≤ ~2000",
            _ => "unknown",
        }
    } else {
        "unknown"
    }
}

/// The coarse runs bucket for a hint, with the counts behind it. Only the
/// three clear hints give a bucket; bounds such as "likely ≥ ~1000" stay
/// unknown
pub fn runs_bucket(hint: &str, ev: &RunsEvidence) -> (String, Vec<String>) {
    let bucket = if hint.starts_with("likely high") {
        "high (≥ ~5000)"
    } else if hint.starts_with("likely mid") {
        "medium (~1000)"
    } else if hint.starts_with("likely low") {
        "low (≤ ~200)"
    } else {
        "unknown"
    };
    let mut evidence = Vec::new();
    if bucket != "unknown" {
        evidence.push(format!(
            "Error(string) selector written out (PUSH32 0x08c379a0..) x{}, computed (PUSH3 0x461bcd PUSH1 0xe5 SHL) x{}",
            ev.error_selector_literal.len(),
            ev.error_selector_shifted.len()
        ));
        evidence.push(format!(
            "other wide constants written out x{}, computed x{}",
            ev.other_literal.len(),
            ev.other_shifted.len()
        ));
        if !ev.address_mask_shifted.is_empty() {
            evidence.push(format!(
                "address mask computed (PUSH1 1 PUSH1 0xa0 SHL) x{}: not counted, solc keeps it at high runs too",
                ev.address_mask_shifted.len()
            ));
        }
    }
    (bucket.to_string(), evidence)
}

/// Is this solc version (`0.8.23`) at least `0.8.20`?
fn solc_at_least_0_8_20(version: &str) -> bool {
    let core = version.split(['+', '-']).next().unwrap_or("");
    let parts: Vec<u32> = core.split('.').filter_map(|p| p.parse().ok()).collect();
    matches!(parts.as_slice(), [maj, min, patch] if (*maj, *min, *patch) >= (0, 8, 20))
}

/// The upper EVM version bound and the reasons for both bounds. The lower
/// bound comes from executed opcodes; the upper one is an absence claim, so
/// it uses a sweep of the whole executable part, not only the paths the
/// call analysis reached
fn evm_bounds(
    min_evm: &str,
    min_evidence: &[String],
    solc_version: Option<&str>,
    push0_executed: usize,
    push0_in_code: usize,
) -> (Option<String>, Vec<String>) {
    let mut reasons = Vec::new();
    if !min_evidence.is_empty() {
        reasons.push(format!(
            "min {min_evm}: executed {}",
            min_evidence.join(", ")
        ));
    }
    if push0_executed > 0 {
        return (None, reasons);
    }
    let Some(v) = solc_version.filter(|v| solc_at_least_0_8_20(v)) else {
        return (None, reasons);
    };
    if push0_in_code > 0 {
        reasons.push(format!(
            "no executed PUSH0, but {push0_in_code} PUSH0 byte(s) in the executable part that the analysis did not reach: no upper bound"
        ));
        return (None, reasons);
    }
    if min_evm.starts_with("cancun") || min_evm.starts_with("shanghai") {
        reasons.push(format!(
            "no PUSH0 with solc {v}, yet {min_evm} opcodes are executed: inconsistent, no upper bound"
        ));
        return (None, reasons);
    }
    reasons.push(format!(
        "consistent with paris or earlier: solc {v} normally pushes every zero with PUSH0 when targeting shanghai or later (its default from 0.8.20), and the executable code has none. Not proof: a shanghai build that never pushes a zero has none either"
    ));
    (Some("paris".to_string()), reasons)
}

/// Opcodes introduced by each fork, newest first
const FORK_OPCODES: &[(&str, &[(u8, &str)])] = &[
    (
        "cancun",
        &[
            (0x5c, "TLOAD"),
            (0x5d, "TSTORE"),
            (0x5e, "MCOPY"),
            (0x49, "BLOBHASH"),
            (0x4a, "BLOBBASEFEE"),
        ],
    ),
    ("shanghai", &[(0x5f, "PUSH0")]),
    ("london", &[(0x48, "BASEFEE")]),
    ("istanbul", &[(0x46, "CHAINID"), (0x47, "SELFBALANCE")]),
    (
        "constantinople",
        &[
            (0x1b, "SHL"),
            (0x1c, "SHR"),
            (0x1d, "SAR"),
            (0x3f, "EXTCODEHASH"),
            (0xf5, "CREATE2"),
        ],
    ),
    (
        "byzantium",
        &[
            (0x3d, "RETURNDATASIZE"),
            (0x3e, "RETURNDATACOPY"),
            (0xfa, "STATICCALL"),
            (0xfd, "REVERT"),
        ],
    ),
];

/// `visited` marks the pcs an analysed path executed (from
/// `bytecode::calls::analyze_calls`); only those count for the EVM version,
/// so data, dead code and bytes after the metadata (vyper's immutables) do
/// not
pub fn fingerprint(code: &[u8], visited: &[bool]) -> Fingerprint {
    use std::collections::{BTreeMap, BTreeSet};

    let blocks = find_metadata_blocks(code);
    let seen = |pc: usize| visited.get(pc).copied().unwrap_or(false);
    let ops: Vec<(usize, u8)> = linear_sweep(code)
        .into_iter()
        .filter(|(pc, _)| seen(*pc))
        .collect();
    let dests = super::calls::jumpdests(code);

    let mut counts: BTreeMap<u8, usize> = BTreeMap::new();
    let mut address_like = BTreeSet::new();
    for (pc, op) in &ops {
        *counts.entry(*op).or_default() += 1;
        if *op == 0x7f {
            if let Some(word) = code.get(pc + 1..pc + 33) {
                if word[..12].iter().all(|b| *b == 0)
                    && word[12..].iter().filter(|b| **b != 0).count() >= 16
                {
                    address_like.insert(word.to_vec());
                }
            }
        }
    }

    let mut min_evm = "frontier (or unknown)".to_string();
    let mut evidence = Vec::new();
    for (fork, opcodes) in FORK_OPCODES {
        let seen: Vec<String> = opcodes
            .iter()
            .filter_map(|(op, name)| counts.get(op).map(|n| format!("{name} x{n}")))
            .collect();
        if !seen.is_empty() {
            min_evm = format!("{fork} or later");
            evidence = seen;
            break;
        }
    }

    // Code shape: solc's legacy pipeline starts nonpayable contracts with
    // the callvalue check before the selector; the IR pipeline compares
    // calldatasize first. Both are guesses.
    let (pipeline, pipeline_evidence) = if code.starts_with(&[0x60, 0x80, 0x60, 0x40, 0x52]) {
        let rest = &code[5..code.len().min(16)];
        if rest.starts_with(&[0x34, 0x80, 0x15]) {
            (
                "legacy",
                vec![
                    "callvalue check right after the free memory pointer (legacy shape)"
                        .to_string(),
                ],
            )
        } else if rest.starts_with(&[0x60, 0x04, 0x36, 0x10]) || rest.starts_with(&[0x36, 0x15]) {
            (
                "unknown",
                vec!["calldatasize check first: seen in both pipelines".to_string()],
            )
        } else if rest.starts_with(&[0x34, 0x15])
            || rest.starts_with(&[0x60, 0x04, 0x36, 0x10, 0x15])
        {
            (
                "via-ir",
                vec!["callvalue tested with ISZERO directly (IR shape)".to_string()],
            )
        } else {
            (
                "unknown",
                vec!["solc-style free memory pointer, no known shape after it".to_string()],
            )
        }
    } else if code.starts_with(&[0x60, 0x80, 0x80, 0x60, 0x40, 0x52]) {
        (
            "via-ir",
            vec!["`PUSH1 0x80 DUP1 PUSH1 0x40 MSTORE` prologue (IR shape)".to_string()],
        )
    } else {
        (
            "unknown",
            vec!["no solc prologue: possibly vyper, huff or hand-written".to_string()],
        )
    };

    let mut trailing = blocks.iter().find(|b| b.trailing).cloned();
    let mut extra_notes = Vec::new();
    if trailing.is_none() {
        // vyper appends immutables after its metadata, so the block is not
        // at the very end; take the last vyper block followed only by data
        // Accept it only when nothing after it can run, the block itself is
        // excluded as never-executed, and what follows is whole 32-byte words
        if let Some(last) = blocks.last().filter(|b| {
            b.compiler == "vyper"
                && (code.len() - b.end).is_multiple_of(32)
                && !dests[b.start..].iter().any(|d| *d)
                && !(b.start..code.len()).any(seen)
        }) {
            extra_notes.push(format!(
                "vyper metadata at {:#x}, followed by {} bytes of immutables",
                last.start,
                code.len() - last.end
            ));
            trailing = Some(last.clone());
        }
    }
    let mut notes = vec![
        "Opcodes come from the paths ethcli's call analysis executed; with incomplete coverage some code is not counted".to_string(),
        "The pipeline guess is a heuristic; confirm by compiling".to_string(),
    ];
    notes.extend(extra_notes);
    if trailing.is_none() {
        notes.push("No compiler metadata at the end of the code: compiled with metadata disabled, or not by solc/vyper".to_string());
    } else if let Some(t) = &trailing {
        if t.compiler == "solc" && t.version.is_none() {
            notes.push("Metadata has no version: solc before 0.5.9".to_string());
        }
    }
    let (optimizer_runs_hint, optimizer_runs_evidence) = optimizer_runs_hint(code, visited);
    let (optimizer_runs_bucket, optimizer_runs_bucket_evidence) =
        runs_bucket(&optimizer_runs_hint, &optimizer_runs_evidence);
    let exec_end = trailing
        .as_ref()
        .map_or(code.len(), |t| t.start.min(code.len()));
    let push0_in_code = linear_sweep(&code[..exec_end])
        .iter()
        .filter(|(_, op)| *op == 0x5f)
        .count();
    let push0_executed = counts.get(&0x5f).copied().unwrap_or(0);
    let solc_version = trailing
        .as_ref()
        .filter(|t| t.compiler == "solc")
        .and_then(|t| t.version.clone());
    let (max_evm_version, evm_version_reasons) = evm_bounds(
        &min_evm,
        &evidence,
        solc_version.as_deref(),
        push0_executed,
        push0_in_code,
    );
    if optimizer_runs_hint != "unknown" {
        notes.push("The optimizer runs hint reads how solc's legacy optimizer wrote constants (measured with 0.8.29); confirm by compiling at the suggested runs. Optimizer off or via-IR code can mislead it".to_string());
    }
    let selector_count = super::extract_selectors(code).len();
    let reads = if selector_count == 0 {
        calldata_reads(code, visited)
    } else {
        Vec::new()
    };
    Fingerprint {
        optimizer_runs_hint,
        optimizer_runs_evidence,
        selector_count,
        dispatcher: (!reads.is_empty()).then(|| FALLBACK_ONLY.to_string()),
        calldata_reads: reads,
        code_size: code.len(),
        code_hash: format!("{:#x}", alloy::primitives::keccak256(code)),
        metadata: trailing.clone(),
        embedded_metadata: blocks
            .into_iter()
            .filter(|b| !b.trailing && trailing.as_ref().is_none_or(|t| t.start != b.start))
            .collect(),
        min_evm_version: min_evm,
        evm_evidence: evidence,
        max_evm_version_confidence: max_evm_version.as_ref().map(|_| "consistent".to_string()),
        max_evm_version,
        evm_version_reasons,
        push0_executed,
        push0_in_code,
        optimizer_runs_bucket,
        optimizer_runs_bucket_evidence,
        pipeline_guess: pipeline.to_string(),
        pipeline_evidence,
        address_like_push32: address_like.len(),
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_visited(code: &[u8]) -> Vec<bool> {
        vec![true; code.len()]
    }

    #[test]
    fn runs_hint_reads_constant_forms() {
        let sel32 = format!("7f08c379a0{}", "00".repeat(28));
        let mask20 = format!("73{}", "ff".repeat(20));
        // High: selector and mask written out; a SHR so shifts exist
        let high = hex::decode(format!("{sel32}50{mask20}5060e01c00")).unwrap();
        let (h, ev) = optimizer_runs_hint(&high, &all_visited(&high));
        assert_eq!(h, "likely high (≥ ~5000)");
        assert_eq!(ev.error_selector_literal, vec![0]);
        assert_eq!(ev.other_literal, vec![34]);
        // The computed Error(string) selector alone (PUSH3 0x461bcd PUSH1
        // 0xe5 SHL) only bounds runs from above; the address mask
        // (PUSH1 1 PUSH1 0xa0 SHL) is kept apart, solc computes it at high
        // runs too
        let one = hex::decode("62461bcd60e51b50600160a01b5000").unwrap();
        let (h, ev) = optimizer_runs_hint(&one, &all_visited(&one));
        assert_eq!(h, "likely < ~5000");
        assert_eq!(ev.error_selector_shifted, vec![0]);
        assert!(ev.other_shifted.is_empty());
        assert_eq!(ev.address_mask_shifted, vec![8]);
        // Low: that plus another constant computed (PUSH4 sel PUSH1 0xe0 SHL)
        let low = hex::decode("62461bcd60e51b50600160a01b5063aabbccdd60e01b5000").unwrap();
        let (h, ev) = optimizer_runs_hint(&low, &all_visited(&low));
        assert_eq!(h, "likely low (≈200)");
        assert_eq!(ev.other_shifted, vec![14]);
        // The Error(string) selector written out alone is not a high bucket
        let sel_only = hex::decode(format!("{sel32}5060e01c00")).unwrap();
        let (h, ev) = optimizer_runs_hint(&sel_only, &all_visited(&sel_only));
        assert_eq!(runs_bucket(&h, &ev).0, "unknown");
        // Mid: selector computed, masks written out
        let mid = hex::decode(format!("62461bcd60e51b50{mask20}5000")).unwrap();
        assert_eq!(
            optimizer_runs_hint(&mid, &all_visited(&mid)).0,
            "likely mid (≈1000-4000)"
        );
        // Unreached code does not count; `PUSH1 0x1f NOT` is never evidence
        assert_eq!(
            optimizer_runs_hint(&low, &vec![false; low.len()]).0,
            "unknown"
        );
        let round = hex::decode("601f1960e01c00").unwrap();
        let (h, ev) = optimizer_runs_hint(&round, &all_visited(&round));
        assert_eq!(h, "unknown");
        assert!(ev.other_shifted.is_empty());
        // No shift opcodes at all (pre-Constantinople): unknown
        let old = hex::decode(format!("{sel32}5000")).unwrap();
        assert_eq!(optimizer_runs_hint(&old, &all_visited(&old)).0, "unknown");
    }

    const YEARN_STRATEGY: &str =
        include_str!("../../tests/fixtures/match/yearn_v3_strategy_runtime.hex");

    #[test]
    fn yearn_strategy_bounds_and_runs() {
        // A Yearn V3 tokenized strategy on mainnet: solc 0.8.23, deployed
        // with evm_version paris and 10000 optimizer runs
        let code = hex::decode(YEARN_STRATEGY.trim()).unwrap();
        let analysis =
            crate::bytecode::analyze_calls(&code, crate::bytecode::CallsBudget::default());
        let fp = fingerprint(&code, &analysis.visited);
        assert_eq!(fp.push0_executed, 0);
        assert_eq!(fp.push0_in_code, 0);
        assert_eq!(fp.max_evm_version.as_deref(), Some("paris"));
        assert!(fp.evm_version_reasons.iter().any(|r| r.contains("0.8.23")));
        assert_eq!(fp.optimizer_runs_bucket, "high (≥ ~5000)");
        assert_eq!(fp.max_evm_version_confidence.as_deref(), Some("consistent"));
        assert!(!fp.optimizer_runs_evidence.address_mask_shifted.is_empty());
        assert!(fp.optimizer_runs_evidence.other_shifted.is_empty());
    }

    #[test]
    fn evm_bounds_from_push0_and_version() {
        // PUSH0 executed: no upper bound, lower bound from the opcode list
        let (max, _) = evm_bounds(
            "shanghai or later",
            &["PUSH0 x3".into()],
            Some("0.8.24"),
            3,
            3,
        );
        assert_eq!(max, None);
        // No PUSH0 at all with solc >= 0.8.20: paris at most
        let (max, why) = evm_bounds("constantinople or later", &[], Some("0.8.20"), 0, 0);
        assert_eq!(max.as_deref(), Some("paris"));
        assert!(why.iter().any(|r| r.starts_with("consistent with paris")));
        // Older solc says nothing about the upper bound
        assert_eq!(
            evm_bounds("constantinople or later", &[], Some("0.8.19"), 0, 0).0,
            None
        );
        // A PUSH0 the analysis did not reach blocks the absence claim
        assert_eq!(
            evm_bounds("constantinople or later", &[], Some("0.8.25"), 0, 1).0,
            None
        );
        // Cancun opcodes without PUSH0 contradict each other: no bound
        assert_eq!(
            evm_bounds(
                "cancun or later",
                &["TLOAD x1".into()],
                Some("0.8.25"),
                0,
                0
            )
            .0,
            None
        );
        // No metadata version: nothing
        assert_eq!(
            evm_bounds("constantinople or later", &[], None, 0, 0).0,
            None
        );
        assert!(solc_at_least_0_8_20("0.8.20+commit.a1b79de6"));
        assert!(!solc_at_least_0_8_20("0.7.6"));
    }

    #[test]
    fn push0_in_push_data_or_metadata_is_not_counted() {
        // PUSH1 0x5f STOP, then solc 0.8.21 metadata
        let mut code = hex::decode("605f00").unwrap();
        code.extend(hex::decode(TAIL).unwrap());
        let fp = fingerprint(&code, &all_visited(&code[..3]));
        assert_eq!(fp.push0_executed, 0);
        assert_eq!(fp.push0_in_code, 0);
        assert_eq!(fp.max_evm_version.as_deref(), Some("paris"));
        // A real PUSH0
        let mut code = hex::decode("5f5000").unwrap();
        code.extend(hex::decode(TAIL).unwrap());
        let fp = fingerprint(&code, &all_visited(&code[..3]));
        assert_eq!(fp.push0_executed, 1);
        assert_eq!(fp.max_evm_version, None);
        assert!(fp.min_evm_version.starts_with("shanghai"));
    }

    #[test]
    fn runs_bucket_only_for_clear_hints() {
        let ev = RunsEvidence::default();
        assert_eq!(
            runs_bucket("likely high (≥ ~5000)", &ev).0,
            "high (≥ ~5000)"
        );
        assert_eq!(
            runs_bucket("likely mid (≈1000-4000)", &ev).0,
            "medium (~1000)"
        );
        assert_eq!(runs_bucket("likely low (≈200)", &ev).0, "low (≤ ~200)");
        for weak in [
            "likely ≥ ~1000",
            "likely ≥ ~5000 (Error(string) selector only)",
            "likely < ~5000",
            "likely ≤ ~2000",
            "unknown",
        ] {
            let (b, e) = runs_bucket(weak, &ev);
            assert_eq!(b, "unknown");
            assert!(e.is_empty());
        }
    }

    #[test]
    fn fallback_only_code_is_named() {
        // calldataload(0) then stop: no selector dispatcher
        let code = hex::decode("5f3550600060006000376000").unwrap();
        let fp = fingerprint(&code, &all_visited(&code));
        assert_eq!(fp.selector_count, 0);
        assert_eq!(fp.dispatcher.as_deref(), Some(FALLBACK_ONLY));
        assert_eq!(fp.calldata_reads, vec![1, 9]);
        // Nothing reached: no claim
        let fp = fingerprint(&code, &vec![false; code.len()]);
        assert!(fp.dispatcher.is_none());
    }

    // solc 0.8.21 ipfs metadata, as at the end of a real contract
    const TAIL: &str = "a26469706673582212208b5d96ae75ef7e28d88d1c3820279a801b63c4e1319cce2a5166f9a221abe79564736f6c63430008150033";

    #[test]
    fn decodes_trailing_solc_metadata() {
        let mut code = hex::decode("6080604052348015600f57600080fd5b50").unwrap();
        code.extend(hex::decode(TAIL).unwrap());
        let block = trailing_metadata(&code).expect("metadata");
        assert_eq!(block.compiler, "solc");
        assert_eq!(block.version.as_deref(), Some("0.8.21"));
        assert_eq!(block.hash_type, "ipfs");
        assert!(block.trailing);
        assert_eq!(block.end, code.len());
        assert_eq!(find_metadata_blocks(&code), vec![block]);
    }

    #[test]
    fn ignores_a_fake_length() {
        // Ends in 0x0033 but the bytes before are not CBOR
        let code = hex::decode("6000600055aabbccdd0033").unwrap();
        assert!(trailing_metadata(&code).is_none());
        assert!(find_metadata_blocks(&code).is_empty());
    }

    #[test]
    fn finds_nested_blocks() {
        let tail = hex::decode(TAIL).unwrap();
        let mut code = hex::decode("600160020100").unwrap();
        code.extend(&tail);
        code.extend(hex::decode("5b00").unwrap());
        code.extend(&tail);
        let blocks = find_metadata_blocks(&code);
        assert_eq!(blocks.len(), 2);
        assert!(!blocks[0].trailing);
        assert!(blocks[1].trailing);
    }

    #[test]
    fn decodes_bzzr0_without_version() {
        let mut code = vec![0x00];
        code.extend(hex::decode("a165627a7a72305820").unwrap());
        code.extend([0x11; 32]);
        code.extend(hex::decode("0029").unwrap());
        let block = trailing_metadata(&code).unwrap();
        assert_eq!(block.hash_type, "bzzr0");
        assert_eq!(block.compiler, "solc");
        assert!(block.version.is_none());
    }

    #[test]
    fn oversized_cbor_strings_are_not_metadata() {
        let mut cbor = hex::decode("a1646970667358").unwrap();
        cbor.push(80);
        cbor.extend([0x11; 80]);
        let mut code = vec![0x00];
        let len = cbor.len() as u16;
        code.extend(&cbor);
        code.extend(len.to_be_bytes());
        assert!(trailing_metadata(&code).is_none());
    }

    #[test]
    fn version_text_is_sanitised() {
        let mut cbor = hex::decode("a164736f6c63").unwrap();
        let text = b"0.8.21\nVerified\x1b[31m";
        cbor.push(0x60 + text.len() as u8);
        cbor.extend(text);
        let mut code = vec![0x00];
        let len = cbor.len() as u16;
        code.extend(&cbor);
        code.extend(len.to_be_bytes());
        let m = trailing_metadata(&code).unwrap();
        assert!(m.version.is_none());
    }

    #[test]
    fn version_text_must_look_like_a_version() {
        assert!(is_version_text("0.8.21"));
        assert!(is_version_text("0.8.21+commit.d9974bed"));
        assert!(is_version_text("0.4.0-nightly.2017.1.1"));
        assert!(!is_version_text("exact.match.verified"));
        assert!(!is_version_text("0.8"));
    }

    fn visited(code: &[u8]) -> Vec<bool> {
        crate::bytecode::calls::analyze_calls(code, Default::default()).visited
    }

    #[test]
    fn min_evm_from_executed_code_only() {
        // PUSH1 0 STOP TLOAD: the TLOAD is never executed
        let code = hex::decode("6000005c").unwrap();
        assert!(!fingerprint(&code, &visited(&code))
            .min_evm_version
            .starts_with("cancun"));
        let code = hex::decode("60005c00").unwrap();
        assert!(fingerprint(&code, &visited(&code))
            .min_evm_version
            .starts_with("cancun"));
        let code = hex::decode("5f5f5500").unwrap();
        assert!(fingerprint(&code, &visited(&code))
            .min_evm_version
            .starts_with("shanghai"));
    }

    #[test]
    fn vyper_metadata_followed_by_immutables() {
        let mut code = hex::decode("600000").unwrap();
        code.extend(hex::decode("a165767970657283000307000b").unwrap());
        code.extend([0x11; 64]);
        let fp = fingerprint(&code, &visited(&code));
        let m = fp.metadata.expect("vyper metadata");
        assert_eq!(m.version.as_deref(), Some("0.3.7"));
        // A JUMPDEST after the block, or a tail that is not whole words,
        // rules the fallback out
        let mut bad = hex::decode("600000").unwrap();
        bad.extend(hex::decode("a165767970657283000307000b").unwrap());
        let mut tail = vec![0x5b, 0x33, 0xff];
        tail.resize(32, 0);
        bad.extend(tail);
        assert!(fingerprint(&bad, &visited(&bad)).metadata.is_none());
        let mut odd = hex::decode("600000").unwrap();
        odd.extend(hex::decode("a165767970657283000307000b").unwrap());
        odd.extend([0x11; 31]);
        assert!(fingerprint(&odd, &visited(&odd)).metadata.is_none());
    }
}
