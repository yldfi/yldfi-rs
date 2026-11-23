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
    /// Pipeline guess from code shape: "legacy", "via-ir" or "unknown"
    pub pipeline_guess: String,
    pub pipeline_evidence: Vec<String>,
    /// Distinct PUSH32 operands shaped like an address (12 zero bytes then
    /// 20 non-zero bytes). Immutable addresses look like this, and so do
    /// address constants, so this is only an upper bound on immutables.
    pub address_like_push32: usize,
    pub notes: Vec<String>,
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
    Fingerprint {
        code_size: code.len(),
        code_hash: format!("{:#x}", alloy::primitives::keccak256(code)),
        metadata: trailing.clone(),
        embedded_metadata: blocks
            .into_iter()
            .filter(|b| !b.trailing && trailing.as_ref().is_none_or(|t| t.start != b.start))
            .collect(),
        min_evm_version: min_evm,
        evm_evidence: evidence,
        pipeline_guess: pipeline.to_string(),
        pipeline_evidence,
        address_like_push32: address_like.len(),
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
