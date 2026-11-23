//! Facts evmole infers from bytecode alone: storage layout, control flow and
//! the compiler's CBOR metadata, wrapped in ethcli's own types.
//!
//! Everything here is inferred from bytecode, not read from verified source,
//! and the output says so (`"source": "inferred"`).

use evmole::control_flow_graph::BlockType;
use evmole::{contract_info, CborValue, ContractInfoArgs};
use serde::Serialize;

use super::types::MAX_BYTECODE_SIZE;

/// One storage slot as evmole infers it
#[derive(Debug, Clone, Serialize)]
pub struct InferredStorageSlot {
    /// Slot, 0x-prefixed 32-byte hex
    pub slot: String,
    /// Byte offset inside the slot
    pub offset: u8,
    /// Inferred Solidity type (`uint256`, `mapping(address => uint256)`, ...)
    #[serde(rename = "type")]
    pub kind: String,
    /// Selectors of functions that read it
    pub reads: Vec<String>,
    /// Selectors of functions that write it
    pub writes: Vec<String>,
}

/// Inferred storage layout
#[derive(Debug, Clone, Serialize)]
pub struct InferredStorage {
    /// Always `inferred`
    pub source: &'static str,
    pub slots: Vec<InferredStorageSlot>,
    /// EIP-1153 transient storage slots
    pub transient: Vec<InferredStorageSlot>,
}

/// Control-flow graph summary
#[derive(Debug, Clone, Default, Serialize)]
pub struct ControlFlowSummary {
    /// Always `inferred`
    pub source: &'static str,
    /// Basic blocks found by a linear scan (includes data and metadata
    /// bytes that happen to decode as code)
    pub basic_blocks: usize,
    /// Blocks reachable from pc 0 after evmole resolved dynamic jumps
    pub reachable_blocks: usize,
    /// Reachable blocks ending in a jump whose target is computed
    pub dynamic_jump_blocks: usize,
    /// Dynamic jump paths evmole could not resolve
    pub unresolved_jumps: usize,
    /// Reachable blocks ending in STOP/RETURN
    pub success_exits: usize,
    /// Reachable blocks ending in REVERT/INVALID
    pub revert_exits: usize,
}

/// One CBOR metadata entry
#[derive(Debug, Clone, Serialize)]
pub struct MetadataEntry {
    pub key: String,
    /// Decoded value: text, number, hex bytes or a decoded compiler version
    pub value: String,
}

/// The compiler metadata appended to runtime code
#[derive(Debug, Clone, Serialize)]
pub struct CompilerMetadata {
    /// Always `bytecode`: read from the CBOR trailer, which a deployer can
    /// forge
    pub source: &'static str,
    /// Offset where the CBOR trailer starts
    pub offset: usize,
    pub length: usize,
    /// `solc 0.8.20`, `vyper 0.3.10`, ...
    pub compiler: Option<String>,
    /// IPFS CIDv0 hash of the metadata file, when present
    pub ipfs: Option<String>,
    /// Swarm hash (`bzzr0`/`bzzr1`), when present
    pub bzzr: Option<String>,
    pub entries: Vec<MetadataEntry>,
}

/// Everything evmole infers, in one pass
#[derive(Debug, Clone, Serialize)]
pub struct InferredFacts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub storage: Option<InferredStorage>,
    pub control_flow: ControlFlowSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<CompilerMetadata>,
}

fn hex0x(bytes: &[u8]) -> String {
    format!("0x{}", hex::encode(bytes))
}

fn base58(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut digits: Vec<u8> = Vec::new();
    for &byte in bytes {
        let mut carry = byte as u32;
        for d in digits.iter_mut() {
            carry += (*d as u32) << 8;
            *d = (carry % 58) as u8;
            carry /= 58;
        }
        while carry > 0 {
            digits.push((carry % 58) as u8);
            carry /= 58;
        }
    }
    let zeros = bytes.iter().take_while(|b| **b == 0).count();
    std::iter::repeat_n(b'1', zeros)
        .chain(digits.iter().rev().map(|d| ALPHABET[*d as usize]))
        .map(char::from)
        .collect()
}

/// Read the CBOR trailer only (cheap)
#[must_use]
pub fn compiler_metadata(bytecode: &[u8]) -> Option<CompilerMetadata> {
    if bytecode.is_empty() || bytecode.len() > MAX_BYTECODE_SIZE {
        return None;
    }
    let info = contract_info(ContractInfoArgs::new(bytecode).with_metadata());
    info.metadata.map(convert_metadata)
}

fn convert_metadata(meta: evmole::CborMetadata) -> CompilerMetadata {
    let mut compiler = None;
    let mut ipfs = None;
    let mut bzzr = None;
    let entries = meta
        .entries
        .into_iter()
        .map(|entry| {
            let value = match &entry.value {
                CborValue::String(s) => s.clone(),
                CborValue::Integer(i) => i.to_string(),
                CborValue::Bool(b) => b.to_string(),
                CborValue::Bytes(b) | CborValue::Undecoded(b) => hex0x(b),
            };
            match (entry.key.as_str(), &entry.value) {
                ("solc" | "vyper", CborValue::Bytes(v)) if v.len() == 3 => {
                    compiler = Some(format!("{} {}.{}.{}", entry.key, v[0], v[1], v[2]));
                }
                ("solc" | "vyper", CborValue::String(s)) => {
                    compiler = Some(format!("{} {s}", entry.key));
                }
                ("ipfs", CborValue::Bytes(b)) => ipfs = Some(base58(b)),
                ("bzzr0" | "bzzr1", CborValue::Bytes(b)) => {
                    bzzr = Some(format!("{}:{}", entry.key, hex::encode(b)))
                }
                _ => {}
            }
            MetadataEntry {
                key: entry.key,
                value,
            }
        })
        .collect();
    CompilerMetadata {
        source: "bytecode",
        offset: meta.bytecode_offset,
        length: meta.cbor_length,
        compiler,
        ipfs,
        bzzr,
        entries,
    }
}

/// Summarise evmole's control-flow graph
#[must_use]
pub fn control_flow_summary(bytecode: &[u8]) -> ControlFlowSummary {
    if bytecode.is_empty() || bytecode.len() > MAX_BYTECODE_SIZE {
        return ControlFlowSummary {
            source: "inferred",
            ..Default::default()
        };
    }
    let info = contract_info(ContractInfoArgs::new(bytecode).with_control_flow_graph());
    summarize_cfg(
        info.basic_blocks.as_ref().map_or(0, Vec::len),
        info.control_flow_graph.as_ref(),
    )
}

fn summarize_cfg(
    basic_blocks: usize,
    cfg: Option<&evmole::control_flow_graph::ControlFlowGraph>,
) -> ControlFlowSummary {
    let mut out = ControlFlowSummary {
        source: "inferred",
        basic_blocks,
        ..Default::default()
    };
    let Some(cfg) = cfg else { return out };
    out.reachable_blocks = cfg.blocks.len();
    for block in cfg.blocks.values() {
        match &block.btype {
            BlockType::Terminate { success: true } => out.success_exits += 1,
            BlockType::Terminate { success: false } => out.revert_exits += 1,
            BlockType::DynamicJump { to } | BlockType::DynamicJumpi { true_to: to, .. } => {
                out.dynamic_jump_blocks += 1;
                out.unresolved_jumps += to.iter().filter(|j| j.to.is_none()).count();
                if to.is_empty() {
                    out.unresolved_jumps += 1;
                }
            }
            _ => {}
        }
    }
    out
}

fn convert_slots(records: Vec<evmole::StorageRecord>) -> Vec<InferredStorageSlot> {
    let sel = |v: Vec<[u8; 4]>| v.iter().map(|s| hex0x(s)).collect::<Vec<_>>();
    records
        .into_iter()
        .map(|r| InferredStorageSlot {
            slot: hex0x(&r.slot),
            offset: r.offset,
            kind: r.r#type,
            reads: sel(r.reads),
            writes: sel(r.writes),
        })
        .collect()
}

/// Storage layout, control flow and metadata in one evmole pass.
/// `with_storage` costs the most (it runs every function); leave it off
/// when only the CFG and metadata are needed.
#[must_use]
pub fn infer_facts(bytecode: &[u8], with_storage: bool) -> InferredFacts {
    if bytecode.is_empty() || bytecode.len() > MAX_BYTECODE_SIZE {
        return InferredFacts {
            storage: None,
            control_flow: control_flow_summary(bytecode),
            metadata: None,
        };
    }
    let mut args = ContractInfoArgs::new(bytecode)
        .with_control_flow_graph()
        .with_metadata();
    if with_storage {
        args = args.with_storage();
    }
    let info = contract_info(args);
    let control_flow = summarize_cfg(
        info.basic_blocks.as_ref().map_or(0, Vec::len),
        info.control_flow_graph.as_ref(),
    );
    let storage = info.storage.map(|slots| InferredStorage {
        source: "inferred",
        slots: convert_slots(slots),
        transient: convert_slots(info.transient_storage.unwrap_or_default()),
    });
    InferredFacts {
        storage,
        control_flow,
        metadata: info.metadata.map(convert_metadata),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base58_matches_known_cid_prefix() {
        // 0x1220 + 32 zero bytes is a CIDv0 starting with "Qm"
        let mut bytes = vec![0x12, 0x20];
        bytes.extend([0u8; 32]);
        assert!(base58(&bytes).starts_with("Qm"));
        assert_eq!(base58(&[0, 0, 1]), "112");
    }

    #[test]
    fn empty_code_has_no_facts() {
        let facts = infer_facts(&[], true);
        assert!(facts.storage.is_none());
        assert_eq!(facts.control_flow.reachable_blocks, 0);
        assert!(compiler_metadata(&[]).is_none());
    }
}
