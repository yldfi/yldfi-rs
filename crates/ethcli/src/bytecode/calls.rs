//! Call sites and sensitive opcodes, recovered by a bounded abstract
//! interpretation of runtime bytecode.
//!
//! Execution starts at pc 0 with an abstract stack whose values remember
//! where they came from (a constant, `CALLDATALOAD`, `SLOAD` of a slot,
//! `CALLER`, the return of an earlier call, ...) and a small memory model
//! keyed by a symbolic base plus a constant offset, so the selector and
//! arguments an outgoing call writes to memory can be read back. Paths fork
//! at every `JUMPI` whose condition is not constant, and are cut by a step
//! budget, a time budget and a cap on distinct states per pc.
//!
//! The containing function is the dispatcher branch taken: a `JUMPI` on
//! `selector == PUSH4` (or a jump to an entry evmole found), or an entry of a
//! calldata-indexed jump table. Everything is reported as inferred, and what
//! cannot be resolved is reported as unresolved, never guessed.

use alloy::primitives::U256;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use std::time::{Duration, Instant};

use super::types::MAX_BYTECODE_SIZE;

/// Limits for [`analyze_calls`]
#[derive(Debug, Clone, Copy)]
pub struct CallsBudget {
    /// Opcodes executed over all paths
    pub max_steps: u64,
    /// Wall-clock limit
    pub max_time: Duration,
    /// Distinct abstract states explored per (pc, function)
    pub max_states_per_pc: usize,
}

impl Default for CallsBudget {
    fn default() -> Self {
        Self {
            max_steps: 4_000_000,
            max_time: Duration::from_secs(15),
            max_states_per_pc: 24,
        }
    }
}

// ---------------------------------------------------------------------------
// Abstract values

/// Deepest value tree kept; deeper values become `Unknown`
const MAX_DEPTH: usize = 8;
/// Most nodes in one value tree; larger values become `Unknown`. With the
/// depth cap this bounds every recursive walk (Drop, Hash, Eq, provenance)
/// whatever the bytecode does.
const MAX_NODES: usize = 64;

/// A boxed child with its subtree depth and size cached, so the caps are
/// checked in O(1)
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Bx {
    v: Box<V>,
    depth: u8,
    nodes: u8,
}

impl std::ops::Deref for Bx {
    type Target = V;
    fn deref(&self) -> &V {
        &self.v
    }
}

impl AsRef<V> for Bx {
    fn as_ref(&self) -> &V {
        &self.v
    }
}

/// Child list with cached depth and size
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Args {
    v: Vec<V>,
    depth: u8,
    nodes: u8,
}

impl std::ops::Deref for Args {
    type Target = [V];
    fn deref(&self) -> &[V] {
        &self.v
    }
}

impl Args {
    fn as_slice(&self) -> &[V] {
        &self.v
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
enum V {
    /// A constant; the second field is the PUSH size it came from (0 when
    /// computed from other constants)
    Const(U256, u8),
    CallData(Bx),
    Caller,
    Origin,
    This,
    CallValue,
    SLoad(Bx),
    TLoad(Bx),
    /// Return data (or success flag, or created address) of the call/create at pc
    Ret(usize),
    /// Keccak of memory words
    Hash(Args),
    /// Any other operation: opcode and arguments in pop order
    Op(u8, Args),
    /// Environment value with no useful provenance (opcode)
    Env(u8),
    Unknown,
}

/// Box a child, or `None` when the result would exceed the caps
fn bx(v: V) -> Option<Bx> {
    let (depth, nodes) = (v.depth() + 1, v.nodes() + 1);
    (depth <= MAX_DEPTH && nodes <= MAX_NODES).then(|| Bx {
        v: Box::new(v),
        depth: depth as u8,
        nodes: nodes as u8,
    })
}

fn args(v: Vec<V>) -> Option<Args> {
    let depth = 1 + v.iter().map(V::depth).max().unwrap_or(0);
    let nodes = 1 + v.iter().map(V::nodes).sum::<usize>();
    (depth <= MAX_DEPTH && nodes <= MAX_NODES).then_some(Args {
        v,
        depth: depth as u8,
        nodes: nodes as u8,
    })
}

// Every nesting constructor goes through these: past the caps the value
// is Unknown
fn calldata(off: V) -> V {
    bx(off).map_or(V::Unknown, V::CallData)
}
fn sload(slot: V) -> V {
    bx(slot).map_or(V::Unknown, V::SLoad)
}
fn tload(slot: V) -> V {
    bx(slot).map_or(V::Unknown, V::TLoad)
}
fn hash(words: Vec<V>) -> V {
    args(words).map_or(V::Unknown, V::Hash)
}
fn op(code: u8, a: Vec<V>) -> V {
    if code == 0x01 {
        // An ADD chain is one flat sum, so it counts as one level: Solidity
        // bumps the free memory pointer by `round(returndatasize)` after
        // every call, and a nested chain would otherwise hit the depth cap
        // after a few calls. The node cap still bounds its size.
        let depth = a
            .iter()
            .map(|x| match x {
                V::Op(0x01, _) => x.depth(),
                _ => x.depth() + 1,
            })
            .max()
            .unwrap_or(1)
            .max(1);
        let nodes = 1 + a.iter().map(V::nodes).sum::<usize>();
        return if depth <= MAX_DEPTH && nodes <= MAX_NODES {
            V::Op(
                code,
                Args {
                    v: a,
                    depth: depth as u8,
                    nodes: nodes as u8,
                },
            )
        } else {
            V::Unknown
        };
    }
    args(a).map_or(V::Unknown, |a| V::Op(code, a))
}

/// `t >= l` for every value: `t` is `l`, or `l` rounded up to 32 bytes
/// (`and(add(l, 31), not(31))` or `mul(div(add(l, 31), 32), 32)`)
fn at_least(t: &V, l: &V) -> bool {
    if t == l {
        return true;
    }
    let c = |v: &V, n: u64| v.as_const() == Some(U256::from(n));
    let plus31 = |v: &V| {
        matches!(v, V::Op(0x01, a) if a.len() == 2
            && ((c(&a[0], 31) && a[1] == *l) || (c(&a[1], 31) && a[0] == *l)))
    };
    match t {
        V::Op(0x16, a) if a.len() == 2 => {
            let m = !U256::from(31u8);
            (a[0].as_const() == Some(m) && plus31(&a[1]))
                || (a[1].as_const() == Some(m) && plus31(&a[0]))
        }
        V::Op(0x02, a) if a.len() == 2 => {
            let d = |v: &V| matches!(v, V::Op(0x04, b) if b.len() == 2 && plus31(&b[0]) && c(&b[1], 32));
            (c(&a[0], 32) && d(&a[1])) || (c(&a[1], 32) && d(&a[0]))
        }
        _ => false,
    }
}

impl V {
    fn c(v: U256) -> V {
        V::Const(v, 0)
    }
    fn as_const(&self) -> Option<U256> {
        match self {
            V::Const(v, _) => Some(*v),
            _ => None,
        }
    }
    /// Cached: O(1)
    fn depth(&self) -> usize {
        match self {
            V::CallData(x) | V::SLoad(x) | V::TLoad(x) => x.depth as usize,
            V::Hash(xs) | V::Op(_, xs) => xs.depth as usize,
            _ => 0,
        }
    }
    /// Cached: O(1)
    fn nodes(&self) -> usize {
        match self {
            V::CallData(x) | V::SLoad(x) | V::TLoad(x) => x.nodes as usize,
            V::Hash(xs) | V::Op(_, xs) => xs.nodes as usize,
            _ => 1,
        }
    }
    fn contains(&self, pred: &dyn Fn(&V) -> bool) -> bool {
        if pred(self) {
            return true;
        }
        match self {
            V::CallData(x) | V::SLoad(x) | V::TLoad(x) => x.contains(pred),
            V::Hash(xs) | V::Op(_, xs) => xs.iter().any(|x| x.contains(pred)),
            _ => false,
        }
    }
}

fn low_mask(v: U256) -> bool {
    // 2^n - 1 for n in 8..=256, byte aligned
    v != U256::ZERO && (v & (v + U256::from(1u8))) == U256::ZERO && v.bit_len().is_multiple_of(8)
}

fn fold(op: u8, a: &[U256]) -> Option<U256> {
    let b = |n: usize| a[n];
    Some(match op {
        0x01 => b(0).wrapping_add(b(1)),
        0x02 => b(0).wrapping_mul(b(1)),
        0x03 => b(0).wrapping_sub(b(1)),
        0x04 => {
            if b(1).is_zero() {
                U256::ZERO
            } else {
                b(0) / b(1)
            }
        }
        0x06 => {
            if b(1).is_zero() {
                U256::ZERO
            } else {
                b(0) % b(1)
            }
        }
        0x0a => b(0).pow(b(1)),
        0x10 => U256::from((b(0) < b(1)) as u8),
        0x11 => U256::from((b(0) > b(1)) as u8),
        0x14 => U256::from((b(0) == b(1)) as u8),
        0x15 => U256::from(b(0).is_zero() as u8),
        0x16 => b(0) & b(1),
        0x17 => b(0) | b(1),
        0x18 => b(0) ^ b(1),
        0x19 => !b(0),
        0x1a => {
            let i = b(0);
            if i >= U256::from(32u8) {
                U256::ZERO
            } else {
                (b(1) >> (8 * (31 - i.to::<usize>()))) & U256::from(0xffu8)
            }
        }
        0x1b => {
            if b(0) >= U256::from(256u16) {
                U256::ZERO
            } else {
                b(1) << b(0).to::<usize>()
            }
        }
        0x1c => {
            if b(0) >= U256::from(256u16) {
                U256::ZERO
            } else {
                b(1) >> b(0).to::<usize>()
            }
        }
        _ => return None,
    })
}

/// Build an operation, folding constants and dropping no-ops
fn mk(code: u8, args: Vec<V>) -> V {
    let consts: Option<Vec<U256>> = args.iter().map(V::as_const).collect();
    if let Some(c) = &consts {
        if let Some(v) = fold(code, c) {
            return V::c(v);
        }
    }
    match (code, args.as_slice()) {
        // x + 0, x - 0, x * 1, x | 0
        (0x01 | 0x17, [x, V::Const(z, _)]) | (0x01 | 0x17, [V::Const(z, _), x]) if z.is_zero() => {
            return x.clone()
        }
        (0x03, [x, V::Const(z, _)]) if z.is_zero() => return x.clone(),
        (0x02, [x, V::Const(o, _)]) | (0x02, [V::Const(o, _), x]) if *o == U256::from(1u8) => {
            return x.clone()
        }
        // shift by 0
        (0x1b | 0x1c, [V::Const(z, _), x]) if z.is_zero() => return x.clone(),
        // iszero(iszero(iszero(x))) = iszero(x)
        (0x15, [V::Op(0x15, inner)]) => {
            if let [V::Op(0x15, x)] = inner.as_slice() {
                return op(0x15, x.v.clone());
            }
        }
        // (a + c1) + c2
        (0x01, [V::Op(0x01, inner), V::Const(c2, _)])
        | (0x01, [V::Const(c2, _), V::Op(0x01, inner)]) => {
            if let [x, V::Const(c1, _)] | [V::Const(c1, _), x] = inner.as_slice() {
                return op(0x01, vec![x.clone(), V::c(c1.wrapping_add(*c2))]);
            }
        }
        _ => {}
    }
    op(code, args)
}

/// Strip masks, `iszero(iszero(x))` and the 12-byte shifts that move a packed
/// address to the low bytes, so provenance shows through
/// Like [`strip`], but only through masks that keep a whole address (at
/// least 160 bits): `caller & 0xff` is no longer the caller
fn strip_subject(v: &V) -> &V {
    match v {
        V::Op(0x16, a) if a.len() == 2 => match (&a[0], &a[1]) {
            (V::Const(m, _), x) | (x, V::Const(m, _)) if low_mask(*m) && m.bit_len() >= 160 => {
                strip_subject(x)
            }
            _ => v,
        },
        V::Op(0x15, a) => match a.as_slice() {
            [V::Op(0x15, b)] => strip_subject(&b[0]),
            _ => v,
        },
        _ => v,
    }
}

fn strip(v: &V) -> &V {
    match v {
        V::Op(0x16, a) if a.len() == 2 => match (&a[0], &a[1]) {
            (V::Const(m, _), x) | (x, V::Const(m, _)) if low_mask(*m) => strip(x),
            _ => v,
        },
        V::Op(0x15, a) => match a.as_slice() {
            [V::Op(0x15, b)] => strip(&b[0]),
            _ => v,
        },
        V::Op(0x1c, a) => match a.as_slice() {
            // shr(96, x): address packed in the high bytes of a word
            [V::Const(s, _), x] if *s == U256::from(96u8) => strip(x),
            _ => v,
        },
        V::Op(0x04, a) => match a.as_slice() {
            // div(x, 2^96)
            [x, V::Const(d, _)] if *d == U256::from(1u8) << 96 => strip(x),
            _ => v,
        },
        _ => v,
    }
}

// ---------------------------------------------------------------------------
// Provenance (serialised)

/// Where a value comes from
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Provenance {
    /// constant, code_constant, storage, storage_computed, transient_storage,
    /// calldata, msg_sender, tx_origin, address_this, msg_value,
    /// call_return, computed, unresolved
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_pc: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

impl Provenance {
    fn kind(kind: &str) -> Self {
        Self {
            kind: kind.to_string(),
            value: None,
            slot: None,
            offset: None,
            call_pc: None,
            note: None,
        }
    }
    fn with_note(mut self, note: &str) -> Self {
        self.note = Some(note.to_string());
        self
    }
    /// Human-readable one-liner
    pub fn describe(&self) -> String {
        let mut s = self.kind.clone();
        if let Some(v) = &self.value {
            s.push(' ');
            s.push_str(v);
        }
        if let Some(v) = &self.slot {
            s.push_str(&format!(" slot {v}"));
        }
        if let Some(v) = &self.offset {
            s.push_str(&format!(" @{v}"));
        }
        if let Some(pc) = self.call_pc {
            s.push_str(&format!(" of pc {pc}"));
        }
        s
    }
    pub fn is_unresolved(&self) -> bool {
        self.kind == "unresolved"
    }
}

fn hex_u256(v: U256) -> String {
    format!("{v:#x}")
}

fn addr_hex(v: U256) -> String {
    let bytes = v.to_be_bytes::<32>();
    format!("0x{}", hex::encode(&bytes[12..]))
}

fn describe_short(v: &V) -> String {
    let p = provenance(v);
    p.describe()
}

fn provenance(v: &V) -> Provenance {
    let v = strip(v);
    match v {
        V::Const(c, n) => {
            let mut p = Provenance::kind(if *n >= 20 {
                "code_constant"
            } else {
                "constant"
            });
            p.value = Some(if *n == 20 || (c.bit_len() > 64 && c.bit_len() <= 160) {
                addr_hex(*c)
            } else {
                hex_u256(*c)
            });
            if *n == 32 {
                p.note = Some("PUSH32 in runtime code: an immutable or a literal".into());
            }
            p
        }
        V::CallData(off) => {
            let mut p = Provenance::kind("calldata");
            p.offset = Some(match off.as_const() {
                Some(o) => hex_u256(o),
                None => format!("dynamic ({})", describe_short(off)),
            });
            p
        }
        V::Caller => Provenance::kind("msg_sender"),
        V::Origin => Provenance::kind("tx_origin"),
        V::This => Provenance::kind("address_this"),
        V::CallValue => Provenance::kind("msg_value"),
        V::SLoad(slot) | V::TLoad(slot) => {
            let transient = matches!(v, V::TLoad(_));
            match slot.as_const() {
                Some(s) => {
                    let mut p = Provenance::kind(if transient {
                        "transient_storage"
                    } else {
                        "storage"
                    });
                    p.slot = Some(hex_u256(s));
                    p
                }
                None => {
                    let mut p = Provenance::kind("storage_computed");
                    if transient {
                        p.kind = "transient_storage".into();
                    }
                    if let V::Hash(words) = slot.as_ref() {
                        let keys: Vec<String> = words.iter().map(describe_short).collect();
                        p.slot = Some(format!("keccak({})", keys.join(", ")));
                        p.note = Some("mapping entry".into());
                    } else {
                        p.slot = Some(describe_short(slot));
                    }
                    p
                }
            }
        }
        V::Ret(pc) => {
            let mut p = Provenance::kind("call_return");
            p.call_pc = Some(*pc);
            p
        }
        V::Unknown | V::Env(_) => Provenance::kind("unresolved"),
        V::Hash(words) => {
            let mut p = Provenance::kind("keccak");
            let inputs: Vec<String> = words.iter().map(describe_short).collect();
            p.value = Some(format!("keccak256({})", inputs.join(", ")));
            p.note =
                Some("hash of memory: a CREATE2 address, a mapping slot or a commitment".into());
            p
        }
        V::Op(..) => {
            // Partial provenance: a packed storage read keeps its slot
            if let V::Op(op, args) = v {
                if matches!(op, 0x16 | 0x1c | 0x04) {
                    for a in args.iter() {
                        if let V::SLoad(s) = strip(a) {
                            if let Some(slot) = s.as_const() {
                                let mut p = Provenance::kind("storage");
                                p.slot = Some(hex_u256(slot));
                                p.note = Some("packed with other values".into());
                                return p;
                            }
                        }
                        if let V::CallData(off) = strip(a) {
                            let mut p = Provenance::kind("calldata");
                            p.offset = off.as_const().map(hex_u256);
                            p.note = Some("part of a calldata word".into());
                            return p;
                        }
                    }
                }
            }
            let mut sources = BTreeSet::new();
            collect_sources(v, &mut sources);
            let sources: Vec<&str> = sources.into_iter().collect();
            if sources.is_empty() || sources == ["unresolved"] {
                return Provenance::kind("unresolved");
            }
            Provenance::kind("computed").with_note(&format!("from {}", sources.join(", ")))
        }
    }
}

fn collect_sources(v: &V, out: &mut BTreeSet<&'static str>) {
    match v {
        V::CallData(_) => {
            out.insert("calldata");
        }
        V::SLoad(_) | V::TLoad(_) => {
            out.insert("storage");
        }
        V::Caller => {
            out.insert("msg_sender");
        }
        V::Ret(_) => {
            out.insert("call_return");
        }
        V::Unknown | V::Env(_) => {
            out.insert("unresolved");
        }
        _ => {}
    }
    match v {
        V::CallData(x) | V::SLoad(x) | V::TLoad(x) => collect_sources(x, out),
        V::Hash(xs) | V::Op(_, xs) => xs.iter().for_each(|x| collect_sources(x, out)),
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Memory model

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
enum MemVal {
    Word(V),
    /// calldatacopy: source offset
    CallData(V),
    /// return data of the call at pc
    Ret(usize),
    /// codecopy from a constant code offset
    Code(u64),
    Unknown,
}

const ADDR_BIAS: u64 = 1 << 40;

/// A memory address as a sum: symbolic terms (each taken as unsigned, so
/// the address is at least `k`) plus a constant
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct Addr {
    /// Sorted, so equal sums compare equal
    terms: Vec<V>,
    k: u64,
    /// Proven lower bound of the address (at least `k`; more when a term
    /// is `x - small` with a known lower bound for `x`)
    lb: u64,
    /// Holds an unknown value: compared by lower bound only
    opaque: bool,
    /// A symbolic floor: the address is at least `floor_terms + floor_k`
    /// (each `x - small` term replaced by `x` and its bound), which places
    /// `end - (len & 31)` above the start of the same buffer
    floor_terms: Vec<V>,
    floor_k: i128,
}

/// An upper bound of a value, when it is masked or reduced to a small range
fn upper_bound(v: &V) -> Option<u64> {
    let small = |c: &U256| (c.bit_len() <= 40).then(|| c.to::<u64>());
    match v {
        V::Const(c, _) => small(c),
        V::Op(0x16, a) if a.len() == 2 => {
            a.iter().find_map(|x| x.as_const()).and_then(|m| small(&m))
        }
        V::Op(0x06, a) if a.len() == 2 => a[1]
            .as_const()
            .and_then(|m| small(&m))
            .map(|m| m.saturating_sub(1)),
        _ => None,
    }
}

/// A proven lower bound of one symbolic term
fn term_lower_bound(t: &V) -> u64 {
    match t {
        // x - y with y small: at least lb(x) - max(y) (Solidity's
        // `end - (len & 31)` padding arithmetic)
        V::Op(0x03, a) if a.len() == 2 => match (affine(&a[0]), upper_bound(&a[1])) {
            (Some(x), Some(y)) => x.lb.saturating_sub(y),
            _ => 0,
        },
        // x & ~(2^n - 1): x rounded down (Solidity's 32-byte alignment)
        V::Op(0x16, a) if a.len() == 2 => {
            let (x, m) = match (&a[0], &a[1]) {
                (V::Const(m, _), x) | (x, V::Const(m, _)) => (x, *m),
                _ => return 0,
            };
            let low = !m;
            if low.bit_len() > 16 || (low & (low + U256::from(1u8))) != U256::ZERO {
                return 0;
            }
            affine(x).map_or(0, |x| x.lb & !low.to::<u64>())
        }
        _ => 0,
    }
}

fn term_key(v: &V) -> u64 {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

/// Flatten nested ADDs into terms and a constant. `None` for an unknown
/// address or one too large to be real memory.
fn affine(v: &V) -> Option<Addr> {
    fn go(v: &V, terms: &mut Vec<V>, k: &mut u64) -> Option<()> {
        match v {
            V::Const(c, _) => {
                if c.bit_len() <= 40 {
                    *k = k.checked_add(c.to::<u64>())?;
                } else {
                    // A two's-complement negative constant: `add(p, not(31))`
                    // is p - 32
                    let neg = U256::ZERO.wrapping_sub(*c);
                    if neg.bit_len() > 40 {
                        return None;
                    }
                    *k = k.checked_sub(neg.to::<u64>())?;
                }
            }
            V::Op(0x01, a) if a.len() == 2 => {
                go(&a[0], terms, k)?;
                go(&a[1], terms, k)?;
            }
            other => terms.push(other.clone()),
        }
        (terms.len() <= 8).then_some(())
    }
    let mut terms = Vec::new();
    // `k` is biased by ADDR_BIAS so a negative offset (p - 32) is
    // representable; only differences of `k` and `lb` are used
    let mut k = ADDR_BIAS;
    go(v, &mut terms, &mut k)?;
    terms.sort_by_key(term_key);
    let lb = terms
        .iter()
        .map(term_lower_bound)
        .fold(k, u64::saturating_add)
        .saturating_sub(ADDR_BIAS);
    // A bare unknown term can stand for anything, so two of them are never
    // the same address: only the lower bound counts. (A term that merely
    // contains an unknown, like `32 * n`, is the same expression on both
    // sides of one path and is compared as such.)
    let opaque = terms.iter().any(|t| matches!(t, V::Unknown));
    let mut floor_terms = Vec::new();
    let mut floor_k = k as i128;
    for t in &terms {
        match t {
            V::Op(0x03, a) if a.len() == 2 => match (affine(&a[0]), upper_bound(&a[1])) {
                (Some(x), Some(y)) => {
                    floor_terms.extend(x.terms);
                    floor_k += x.k as i128 - ADDR_BIAS as i128 - y as i128;
                }
                _ => floor_terms.push(t.clone()),
            },
            _ => floor_terms.push(t.clone()),
        }
    }
    floor_terms.sort_by_key(term_key);
    Some(Addr {
        terms,
        k,
        lb,
        opaque,
        floor_terms,
        floor_k,
    })
}

/// `small` is a sub-multiset of `big`; returns what `big` has in addition
fn extra_terms<'a>(small: &[V], big: &'a [V]) -> Option<Vec<&'a V>> {
    let mut rest: Vec<&V> = big.iter().collect();
    for t in small {
        let i = rest.iter().position(|x| *x == t)?;
        rest.remove(i);
    }
    Some(rest)
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct MemWrite {
    at: Addr,
    /// `None`: a copy of unknown length, so everything from `at` up
    len: Option<u64>,
    /// The symbolic length of such a copy, to recognise a write that
    /// starts exactly at its end (`calldatacopy(p, o, n); mstore(p + n, x)`)
    len_v: Option<V>,
    val: MemVal,
}

/// Can a range starting at `a` (length `alen`, `None` = unbounded) share a
/// byte with the entry `e`? Symbolic terms are unsigned, so an address with
/// extra terms is at least as high; anything not provably apart overlaps.
fn may_overlap(a: &Addr, alen: Option<u64>, e: &MemWrite) -> bool {
    let end = |k: u64, len: Option<u64>| len.map_or(u64::MAX, |l| k.saturating_add(l));
    let opaque = a.opaque || e.at.opaque;
    // Floors: one range provably starts at or past the other's end
    if !opaque {
        if let Some(e_end) = e.len.map(|l| e.at.k as i128 + l as i128) {
            if a.floor_k >= e_end && extra_terms(&e.at.terms, &a.floor_terms).is_some() {
                return false;
            }
        }
        if let Some(a_end) = alen.map(|l| a.k as i128 + l as i128) {
            if e.at.floor_k >= a_end && extra_terms(&a.terms, &e.at.floor_terms).is_some() {
                return false;
            }
        }
    }
    if !opaque && a.terms == e.at.terms {
        return a.k < end(e.at.k, e.len) && e.at.k < end(a.k, alen);
    }
    if let Some(extra) = (!opaque)
        .then(|| extra_terms(&e.at.terms, &a.terms))
        .flatten()
    {
        // a = entry base + extra + a.k: starts at or above entry base + a.k
        if e.len.is_none() {
            // Starts exactly at (or past) the end of a copy of length L
            if let Some(l) = &e.len_v {
                if extra.iter().any(|x| at_least(x, l)) && a.k >= e.at.k {
                    return false;
                }
            }
        }
        // A constant entry: a's proven lower bound counts in full
        let start = if e.at.terms.is_empty() {
            a.lb + ADDR_BIAS
        } else {
            a.k
        };
        return start < end(e.at.k, e.len);
    }
    if !opaque && extra_terms(&a.terms, &e.at.terms).is_some() {
        // entry = a's base + extra + e.k: starts at or above a's base + e.k
        let start = if a.terms.is_empty() {
            e.at.lb + ADDR_BIAS
        } else {
            e.at.k
        };
        return start < end(a.k, alen);
    }
    // One side is a plain constant range, the other is at least its lower
    // bound
    if a.terms.is_empty() {
        return e.at.lb + ADDR_BIAS < end(a.k, alen);
    }
    if e.at.terms.is_empty() {
        return a.lb + ADDR_BIAS < end(e.at.k, e.len);
    }
    true
}

struct MemWriteView<'a> {
    off: u64,
    #[allow(dead_code)]
    len: Option<u64>,
    val: &'a MemVal,
}

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
struct Memory {
    writes: Rc<Vec<MemWrite>>,
}

const MAX_MEM_WRITES: usize = 192;

impl Memory {
    fn write(&mut self, addr: &V, len: Option<u64>, val: MemVal) {
        self.write_len(addr, len, &V::Unknown, val);
    }

    /// Write `val` over `len` bytes (`None`: unknown length, `len_v` its
    /// symbolic value). Entries this write certainly replaces (same base,
    /// fully covered) are dropped; others stay, and a read takes the newest
    /// entry that may overlap it, which is a value only when it is at the
    /// same base. An unknown address forgets everything.
    fn write_len(&mut self, addr: &V, len: Option<u64>, len_v: &V, val: MemVal) {
        let w = Rc::make_mut(&mut self.writes);
        if len == Some(0) {
            return;
        }
        let Some(at) = affine(addr) else {
            w.clear();
            return;
        };
        let end = len.map_or(u64::MAX, |l| at.k.saturating_add(l));
        w.retain(|e| {
            if !at.opaque && !e.at.opaque && e.at.terms == at.terms {
                // Same base: exact offsets; drop only what is fully covered,
                // a partly overwritten entry still answers for its other
                // bytes (a selector followed by arguments)
                !(e.at.k >= at.k && e.len.is_some_and(|l| e.at.k.saturating_add(l) <= end))
            } else {
                // Another base this write may alias: forget it
                !may_overlap(&at, len, e)
            }
        });
        w.push(MemWrite {
            at,
            len,
            len_v: (len.is_none() && !matches!(len_v, V::Unknown)).then(|| len_v.clone()),
            val,
        });
        if w.len() > MAX_MEM_WRITES {
            w.remove(0);
        }
    }

    fn read(&self, code: &[u8], addr: &V) -> V {
        self.read_prefix(code, addr, 32)
    }

    /// The word at `addr`, trusting only its first `n` bytes: a later write
    /// past them does not make it unknown (a selector followed by arguments)
    fn read_prefix(&self, code: &[u8], addr: &V, n: u64) -> V {
        let Some(at) = affine(addr) else {
            return V::Unknown;
        };
        let off = at.k;
        // A copy of symbolic length L that covers the read (same base): a
        // newer write exactly at its end (base + L) is taken not to reach
        // back into it, i.e. the bytes read are assumed to lie within L
        let cover = self.writes.iter().rev().find(|e| {
            !at.opaque
                && !e.at.opaque
                && e.at.terms == at.terms
                && e.len.is_none()
                && e.len_v.is_some()
                && off >= e.at.k
        });
        for e in self.writes.iter().rev() {
            if !may_overlap(&at, Some(n), e) {
                continue;
            }
            if let Some(c) = cover {
                let at_end = extra_terms(&c.at.terms, &e.at.terms).is_some_and(|extra| {
                    extra
                        .iter()
                        .any(|x| c.len_v.as_ref().is_some_and(|l| at_least(x, l)))
                        && e.at.k >= c.at.k
                });
                if at_end {
                    continue;
                }
            }
            // The latest write that may overlap decides: a value only when it
            // is at the same symbolic base and covers the bytes read
            if at.opaque || e.at.opaque || e.at.terms != at.terms {
                return V::Unknown;
            }
            let e_end = e.len.map_or(u64::MAX, |l| e.at.k.saturating_add(l));
            let inside = off >= e.at.k && off + n <= e_end;
            let e = MemWriteView {
                off: e.at.k,
                len: e.len,
                val: &e.val,
            };
            return match e.val {
                MemVal::Word(v) if e.off == off && e.len == Some(32) => v.clone(),
                MemVal::CallData(src) if inside => {
                    calldata(mk(0x01, vec![src.clone(), V::c(U256::from(off - e.off))]))
                }
                MemVal::Ret(pc) if inside => V::Ret(*pc),
                MemVal::Code(src) if inside => {
                    // Bytes past the copied range are not trusted: zero them
                    let start = (src + (off - e.off)) as usize;
                    let mut buf = [0u8; 32];
                    for (i, b) in buf.iter_mut().enumerate().take(n as usize) {
                        *b = code.get(start + i).copied().unwrap_or(0);
                    }
                    V::Const(U256::from_be_bytes(buf), 32)
                }
                _ => V::Unknown,
            };
        }
        V::Unknown
    }
}

// ---------------------------------------------------------------------------
// Report types

/// One CALL/DELEGATECALL/STATICCALL/CALLCODE/CREATE/CREATE2 site
#[derive(Debug, Clone, Serialize)]
pub struct CallSite {
    pub pc: usize,
    pub kind: String,
    /// Functions (dispatcher branches) whose paths reach this site
    pub functions: Vec<String>,
    /// Target address (empty for CREATE/CREATE2)
    pub target: Vec<Provenance>,
    /// 4-byte selector of the outgoing call, when it can be read from memory
    pub selector: Vec<Provenance>,
    /// ETH value (CALL, CALLCODE, CREATE, CREATE2)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub value: Vec<Provenance>,
    /// Up to three argument words after the selector
    pub args: Vec<Vec<Provenance>>,
    /// Length of the call's input (argsLength)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args_len: Vec<Provenance>,
    /// Whether every target is resolved
    pub resolved: bool,
}

/// A comparison of msg.sender, tx.origin or address(this) that feeds a branch
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Guard {
    /// pc of the JUMPI the comparison decides
    pub pc: usize,
    /// msg_sender, tx_origin or address_this
    pub subject: String,
    /// `eq` or `mapping_lookup` (a storage mapping keyed by the subject)
    pub check: String,
    pub against: Provenance,
}

/// A storage write
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct StoreSite {
    pub pc: usize,
    pub slot: Provenance,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub transient: bool,
}

/// Sensitive opcodes reached inside one function
#[derive(Debug, Clone, Serialize, Default)]
pub struct FunctionFacts {
    /// `0x12345678`, `jump_table[0x05]` or `dispatcher` (fallback and
    /// shared preamble)
    pub function: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    pub origin_pcs: Vec<usize>,
    pub caller_pcs: Vec<usize>,
    pub guards: Vec<Guard>,
    pub call_pcs: Vec<usize>,
    pub delegatecall_pcs: Vec<usize>,
    pub selfdestruct_pcs: Vec<usize>,
    pub create_pcs: Vec<usize>,
    pub sstores: Vec<StoreSite>,
}

/// A sensitive opcode found by a linear scan of the code
#[derive(Debug, Clone, Serialize)]
pub struct OpcodeSite {
    pub opcode: String,
    pub pc: usize,
    /// Whether any analysed path executed it
    pub reached: bool,
    /// Inside a trailing metadata-looking region with no JUMPDEST (most
    /// likely CBOR bytes, not code). Sites in a trailer that does hold a
    /// JUMPDEST are reachable code and are not flagged.
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub in_metadata: bool,
}

/// Opcodes listed by [`opcode_sites`]: ORIGIN, CALLER, CREATE, CALL,
/// CALLCODE, DELEGATECALL, CREATE2, STATICCALL, SELFDESTRUCT
pub const SENSITIVE_OPCODES: &[u8] = &[0x32, 0x33, 0xf0, 0xf1, 0xf2, 0xf4, 0xf5, 0xfa, 0xff];

/// pcs of the JUMPDESTs, by linear decoding (PUSH data skipped)
pub fn jumpdests(code: &[u8]) -> Vec<bool> {
    let mut out = vec![false; code.len()];
    let mut pc = 0;
    while pc < code.len() {
        let op = code[pc];
        if op == 0x5b {
            out[pc] = true;
        }
        pc += 1 + push_size(op);
    }
    out
}

/// Per pc: true when the byte lies in a dead region, i.e. after a halting
/// opcode (STOP, JUMP, RETURN, REVERT, INVALID, SELFDESTRUCT) with no
/// JUMPDEST since, by the same linear sweep as [`jumpdests`]. No jump can
/// land there and nothing falls through into it, so it is data.
#[must_use]
pub fn dead_regions(code: &[u8]) -> Vec<bool> {
    let mut out = vec![false; code.len()];
    let mut dead = false;
    let mut pc = 0;
    while pc < code.len() {
        let op = code[pc];
        if op == 0x5b {
            dead = false;
        }
        let end = (pc + 1 + push_size(op)).min(code.len());
        for d in &mut out[pc..end] {
            *d = dead;
        }
        if matches!(op, 0x00 | 0x56 | 0xf3 | 0xfd | 0xfe | 0xff) {
            dead = true;
        }
        pc += 1 + push_size(op);
    }
    out
}

fn push_size(op: u8) -> usize {
    if (0x60..=0x7f).contains(&op) {
        (op - 0x5f) as usize
    } else {
        0
    }
}

/// Every sensitive opcode in the code, by a linear sweep of the whole code
/// (`reached` is false here; `analyze_calls` fills it in).
///
/// `metadata_offset` is where a trailing metadata region (the CBOR trailer)
/// starts. The deployer controls those bytes, so the sweep never stops
/// there: sites inside the region are still listed, flagged `in_metadata`
/// only when the region holds no JUMPDEST (no jump can land in it).
#[must_use]
pub fn opcode_sites(code: &[u8], metadata_offset: Option<usize>) -> Vec<OpcodeSite> {
    let dests = jumpdests(code);
    let trailer = metadata_offset
        .filter(|start| *start < code.len())
        .filter(|start| !dests[*start..].iter().any(|d| *d));
    let mut out = Vec::new();
    let mut pc = 0;
    while pc < code.len() {
        let op = code[pc];
        if SENSITIVE_OPCODES.contains(&op) {
            out.push(OpcodeSite {
                opcode: opname(op).to_string(),
                pc,
                reached: false,
                in_metadata: trailer.is_some_and(|start| pc >= start),
            });
        }
        pc += 1 + push_size(op);
    }
    out
}

/// A jump whose target is computed from calldata and was enumerated
#[derive(Debug, Clone, Serialize)]
pub struct JumpTable {
    pub pc: usize,
    pub index: String,
    pub entries: usize,
}

/// How much of the code the analysis covered
#[derive(Debug, Clone, Serialize, Default)]
pub struct Coverage {
    /// Blocks evmole finds reachable from pc 0
    pub cfg_reachable_blocks: usize,
    /// Of those, blocks this analysis executed
    pub cfg_blocks_analysed: usize,
    /// JUMPDESTs executed that evmole's CFG does not reach (e.g. jump-table
    /// entries)
    pub extra_blocks_analysed: usize,
    /// Distinct pcs of jumps whose target stayed unresolved
    pub unresolved_jumps: Vec<usize>,
    /// Call sites with at least one unresolved target
    pub unresolved_call_sites: usize,
    pub steps: u64,
    pub step_limit: u64,
    pub elapsed_ms: u128,
    pub time_limit_ms: u128,
    pub step_budget_hit: bool,
    pub time_budget_hit: bool,
    /// Paths dropped because a pc had reached its distinct-state cap
    pub states_pruned: u64,
    /// Paths that ended in a stack underflow or invalid jump
    pub paths_aborted: u64,
    /// pcs of opcodes this analysis does not know (undefined, or newer than
    /// its table): paths stop there, so the code after them is unanalysed
    pub invalid_opcodes: Vec<usize>,
    /// True when no budget was hit and nothing was pruned or unresolved
    pub complete: bool,
}

/// Result of [`analyze_calls`]
#[derive(Debug, Clone, Serialize)]
pub struct CallsReport {
    /// Always `inferred`
    pub source: &'static str,
    pub bytecode_size: usize,
    pub call_sites: Vec<CallSite>,
    pub functions: Vec<FunctionFacts>,
    /// Every ORIGIN, CALLER, ADDRESS-free sensitive opcode in the code
    /// (before the metadata trailer), reached or not
    pub opcode_sites: Vec<OpcodeSite>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub jump_tables: Vec<JumpTable>,
    pub coverage: Coverage,
    pub notes: Vec<String>,
    /// Per pc: whether any analysed path executed it (not serialised)
    #[serde(skip)]
    pub visited: Vec<bool>,
}

impl CallsReport {
    /// Short summary for `contract analyze`
    pub fn summary(&self) -> serde_json::Value {
        let count = |kind: &str| self.call_sites.iter().filter(|c| c.kind == kind).count();
        let guards: Vec<serde_json::Value> = self
            .functions
            .iter()
            .flat_map(|f| {
                f.guards.iter().map(move |g| {
                    serde_json::json!({
                        "function": f.function,
                        "pc": g.pc,
                        "check": format!("{} {} {}", g.subject, g.check, g.against.describe()),
                    })
                })
            })
            .collect();
        let hardcoded: BTreeSet<String> = self
            .call_sites
            .iter()
            .flat_map(|c| c.target.iter())
            .filter(|p| p.kind == "constant" || p.kind == "code_constant")
            .filter_map(|p| p.value.clone())
            .collect();
        serde_json::json!({
            "source": "inferred",
            "call_sites": self.call_sites.len(),
            "by_kind": {
                "CALL": count("CALL"),
                "STATICCALL": count("STATICCALL"),
                "DELEGATECALL": count("DELEGATECALL"),
                "CALLCODE": count("CALLCODE"),
                "CREATE": count("CREATE"),
                "CREATE2": count("CREATE2"),
            },
            "calldata_controlled_targets": self.call_sites.iter().filter(|c| c.target.iter().any(|p| p.kind == "calldata" || p.note.as_deref().is_some_and(|n| n.contains("calldata")))).count(),
            "msg_sender_targets": self.call_sites.iter().filter(|c| c.target.iter().any(|p| p.kind == "msg_sender")).count(),
            "hardcoded_targets": hardcoded,
            "origin_pcs": self.opcode_sites.iter().filter(|s| s.opcode == "ORIGIN").map(|s| s.pc).collect::<Vec<_>>(),
            "guards": guards,
            "coverage": {
                "cfg_blocks_analysed": self.coverage.cfg_blocks_analysed,
                "cfg_reachable_blocks": self.coverage.cfg_reachable_blocks,
                "unresolved_jumps": self.coverage.unresolved_jumps.len(),
                "unresolved_call_sites": self.coverage.unresolved_call_sites,
                "complete": self.coverage.complete,
            },
        })
    }
}

// ---------------------------------------------------------------------------
// Interpreter

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Ctx {
    Dispatcher,
    Selector(u32),
    JumpTable(u8),
}

impl Ctx {
    fn label(self) -> String {
        match self {
            Ctx::Dispatcher => "dispatcher".into(),
            Ctx::Selector(s) => format!("0x{s:08x}"),
            Ctx::JumpTable(i) => format!("jump_table[0x{i:02x}]"),
        }
    }
}

#[derive(Clone)]
struct State {
    pc: usize,
    stack: Vec<V>,
    mem: Memory,
    ctx: Ctx,
    last_call: Option<usize>,
}

#[derive(Default)]
struct SiteAcc {
    kind: &'static str,
    target: BTreeSet<Provenance>,
    selector: BTreeSet<Provenance>,
    value: BTreeSet<Provenance>,
    args: [BTreeSet<Provenance>; 3],
    args_len: BTreeSet<Provenance>,
}

#[derive(Default)]
struct FnAcc {
    origin: BTreeSet<usize>,
    caller: BTreeSet<usize>,
    guards: BTreeSet<Guard>,
    calls: BTreeSet<usize>,
    delegatecall: BTreeSet<usize>,
    selfdestruct: BTreeSet<usize>,
    create: BTreeSet<usize>,
    sstores: BTreeSet<StoreSite>,
}

const MAX_PROVENANCE_PER_FIELD: usize = 6;

/// Whether a state entering a block is new, a memory variant of a known
/// stack (explored after everything else), or seen / over the cap
enum Admit {
    New,
    Variant,
    Seen,
}

/// Memory / last-call variants kept per distinct stack at one pc
const MEMORY_VARIANTS_PER_STACK: usize = 4;

fn insert_capped(set: &mut BTreeSet<Provenance>, p: Provenance) {
    if set.len() < MAX_PROVENANCE_PER_FIELD {
        set.insert(p);
    } else if !set.contains(&p) {
        set.insert(Provenance::kind("unresolved").with_note("more values than shown"));
    }
}

fn opname(op: u8) -> &'static str {
    match op {
        0xf0 => "CREATE",
        0xf1 => "CALL",
        0xf2 => "CALLCODE",
        0xf4 => "DELEGATECALL",
        0xf5 => "CREATE2",
        0xfa => "STATICCALL",
        0xff => "SELFDESTRUCT",
        0x32 => "ORIGIN",
        0x33 => "CALLER",
        _ => "?",
    }
}

/// Stack effect of the opcodes handled generically: (pops, pushes)
fn generic_arity(op: u8) -> Option<(usize, usize)> {
    Some(match op {
        0x01..=0x07 | 0x0a | 0x0b => (2, 1),
        0x08 | 0x09 => (3, 1),
        0x10..=0x14 | 0x16..=0x18 | 0x1a..=0x1d => (2, 1),
        0x15 | 0x19 | 0x1e => (1, 1), // 0x1e CLZ (EIP-7939)
        0x31 | 0x3b | 0x3f | 0x40 | 0x49 => (1, 1),
        0x36 | 0x38 | 0x3a | 0x3d | 0x41..=0x48 | 0x4a | 0x58 | 0x59 | 0x5a => (0, 1),
        0xa0..=0xa4 => (2 + (op - 0xa0) as usize, 0),
        _ => return None,
    })
}

struct Interp<'a> {
    code: &'a [u8],
    jumpdests: Vec<bool>,
    entries: HashMap<usize, u32>,
    budget: CallsBudget,
    started: Instant,
    steps: u64,
    step_hit: bool,
    time_hit: bool,
    pruned: u64,
    aborted: u64,
    /// Admitted states per (pc, ctx), keyed by the full stack, with the
    /// full memory / last-call variants. Exact keys, not hashes: a crafted
    /// hash collision must not make a distinct path look already explored
    #[allow(clippy::type_complexity)]
    seen: HashMap<(usize, Ctx), HashMap<Vec<V>, Vec<(Memory, Option<usize>)>>>,
    visited: Vec<bool>,
    unresolved_jumps: BTreeSet<usize>,
    invalid_opcodes: BTreeSet<usize>,
    sites: BTreeMap<(usize, Ctx), SiteAcc>,
    fns: BTreeMap<Ctx, FnAcc>,
    jump_tables: BTreeMap<usize, (String, usize)>,
    table_enumerations: HashMap<(usize, Ctx), usize>,
    /// pc and shift of the first jump table entered from the dispatcher
    first_table: Option<(usize, u32)>,
    chained_expansions: HashMap<usize, usize>,
}

fn is_selector_expr(v: &V) -> bool {
    v.contains(&|x| matches!(x, V::CallData(o) if o.as_const() == Some(U256::ZERO)))
}

/// An index bounded to one byte (BYTE, SHR 248 or AND 0xff), whatever its
/// source: every JUMPDEST it can select is enumerated, a sound superset
fn byte_index(v: &V) -> bool {
    match v {
        V::Op(0x1a, _) => true,
        V::Op(0x1c, a) => a[0].as_const() == Some(U256::from(248u16)),
        V::Op(0x16, a) => {
            let m = a.iter().find_map(V::as_const);
            m == Some(U256::from(0xffu8))
        }
        _ => false,
    }
}

impl<'a> Interp<'a> {
    fn new(code: &'a [u8], entries: HashMap<usize, u32>, budget: CallsBudget) -> Self {
        let mut jumpdests = vec![false; code.len()];
        let mut pc = 0;
        while pc < code.len() {
            let op = code[pc];
            if op == 0x5b {
                jumpdests[pc] = true;
            }
            pc += 1 + if (0x60..=0x7f).contains(&op) {
                (op - 0x5f) as usize
            } else {
                0
            };
        }
        Self {
            code,
            jumpdests,
            entries,
            budget,
            started: Instant::now(),
            steps: 0,
            step_hit: false,
            time_hit: false,
            pruned: 0,
            aborted: 0,
            seen: HashMap::new(),
            visited: vec![false; code.len()],
            unresolved_jumps: BTreeSet::new(),
            invalid_opcodes: BTreeSet::new(),
            sites: BTreeMap::new(),
            fns: BTreeMap::new(),
            jump_tables: BTreeMap::new(),
            table_enumerations: HashMap::new(),
            first_table: None,
            chained_expansions: HashMap::new(),
        }
    }

    fn fnacc(&mut self, ctx: Ctx) -> &mut FnAcc {
        self.fns.entry(ctx).or_default()
    }

    fn valid_dest(&self, d: U256) -> Option<usize> {
        if d.bit_len() > 32 {
            return None;
        }
        let d = d.to::<usize>();
        (d < self.code.len() && self.jumpdests[d]).then_some(d)
    }

    /// Record comparisons of caller/origin/this found in a branch condition
    fn scan_guards(&mut self, pc: usize, ctx: Ctx, cond: &V) {
        let mut found: Vec<Guard> = Vec::new();
        fn subject_of(v: &V) -> Option<&'static str> {
            match strip_subject(v) {
                V::Caller => Some("msg_sender"),
                V::Origin => Some("tx_origin"),
                V::This => Some("address_this"),
                _ => None,
            }
        }
        fn push_eq(a: &[V], pc: usize, out: &mut Vec<Guard>) {
            for (x, y) in [(&a[0], &a[1]), (&a[1], &a[0])] {
                let Some(subject) = subject_of(x) else {
                    continue;
                };
                // tx.origin == msg.sender is reported once, as origin;
                // msg.sender == msg.sender says nothing
                if (subject == "msg_sender" && matches!(strip(y), V::Origin | V::Caller))
                    || subject_of(y) == Some(subject)
                {
                    continue;
                }
                out.push(Guard {
                    pc,
                    subject: subject.into(),
                    check: "eq".into(),
                    against: provenance(y),
                });
            }
        }
        // `direct`: the value decides the branch as it is (only ISZERO in
        // between), so a SUB or XOR of it is zero exactly when its operands
        // are equal. Inside AND/OR or further arithmetic it is not an
        // equality test.
        fn walk(v: &V, pc: usize, out: &mut Vec<Guard>, depth: usize, direct: bool) {
            if depth > 6 {
                return;
            }
            match v {
                V::Op(0x14, a) if a.len() == 2 => {
                    if a.iter().any(|x| subject_of(x).is_some()) {
                        push_eq(a, pc, out);
                        return;
                    }
                    // eq(sub(a, b), 0) and eq(0, xor(a, b))
                    for (x, y) in [(&a[0], &a[1]), (&a[1], &a[0])] {
                        if y.as_const() == Some(U256::ZERO) && matches!(x, V::Op(0x03 | 0x18, _)) {
                            walk(x, pc, out, depth + 1, true);
                            return;
                        }
                    }
                }
                // The SUB / XOR solc emits for `a == b` / `a != b` before JUMPI
                V::Op(0x03 | 0x18, a)
                    if direct && a.len() == 2 && a.iter().any(|x| subject_of(x).is_some()) =>
                {
                    push_eq(a, pc, out);
                }
                V::Op(0x15, a) => walk(&a[0], pc, out, depth + 1, direct),
                V::Op(0x16 | 0x17 | 0x18 | 0x03, a) => {
                    for x in a.iter() {
                        walk(x, pc, out, depth + 1, false);
                    }
                }
                V::SLoad(slot) => {
                    if let V::Hash(words) = slot.as_ref() {
                        for (subject, pat) in [("msg_sender", V::Caller), ("tx_origin", V::Origin)]
                        {
                            if words.iter().any(|w| *strip(w) == pat) {
                                let mut p = Provenance::kind("storage_computed");
                                p.slot = Some(format!(
                                    "keccak({})",
                                    words
                                        .iter()
                                        .map(describe_short)
                                        .collect::<Vec<_>>()
                                        .join(", ")
                                ));
                                p.note = Some("mapping entry keyed by the subject".into());
                                out.push(Guard {
                                    pc,
                                    subject: subject.into(),
                                    check: "mapping_lookup".into(),
                                    against: p,
                                });
                            }
                        }
                    }
                }
                _ => {
                    // A packed storage flag read: and(shr(k, sload(keccak(..))), 0xff)
                    // Only shifts/divisions that unpack a flag; a comparison such
                    // as balance >= amount is not access control
                    if let V::Op(0x1c | 0x04, a) = v {
                        for x in a.iter() {
                            if matches!(x, V::SLoad(_)) {
                                walk(x, pc, out, depth + 1, false);
                            }
                        }
                    }
                }
            }
        }
        walk(cond, pc, &mut found, 0, true);
        if !found.is_empty() {
            let acc = self.fnacc(ctx);
            acc.guards.extend(found);
        }
    }

    fn out_of_budget(&mut self) -> bool {
        if self.steps >= self.budget.max_steps {
            self.step_hit = true;
            return true;
        }
        if self.steps.is_multiple_of(4096) && self.started.elapsed() >= self.budget.max_time {
            self.time_hit = true;
        }
        self.time_hit
    }

    /// Enter a block: dedupe states per (pc, ctx)
    fn admit(&mut self, s: &State) -> Admit {
        // Two levels: distinct stacks (what decides control flow) get the
        // per-pc cap; for each stack a few memory / last-call variants are
        // kept too, since two paths that differ only in memory can give a
        // later call a different target or selector. Memory variants get
        // their own small cap, so they cannot use up the stack budget and
        // starve other paths. Every dropped state is counted.
        let cap = self.budget.max_states_per_pc;
        let stacks = self.seen.entry((s.pc, s.ctx)).or_default();
        match stacks.get_mut(&s.stack) {
            Some(mems) => {
                if mems.iter().any(|(m, c)| *m == s.mem && *c == s.last_call) {
                    return Admit::Seen;
                }
                if mems.len() >= MEMORY_VARIANTS_PER_STACK {
                    self.pruned += 1;
                    return Admit::Seen;
                }
                mems.push((s.mem.clone(), s.last_call));
                Admit::Variant
            }
            None => {
                if stacks.len() >= cap {
                    self.pruned += 1;
                    return Admit::Seen;
                }
                stacks.insert(s.stack.clone(), vec![(s.mem.clone(), s.last_call)]);
                Admit::New
            }
        }
    }

    fn run(&mut self) {
        let mut work: Vec<State> = vec![State {
            pc: 0,
            stack: Vec::new(),
            mem: Memory::default(),
            ctx: Ctx::Dispatcher,
            last_call: None,
        }];
        // Memory variants (same stack, different memory) wait here until
        // every other path is done, so they use only what budget is left
        // and never crowd out a path the stack alone would have explored
        let mut deferred: Vec<State> = Vec::new();
        'paths: while let Some((mut s, admitted)) = work
            .pop()
            .map(|s| (s, false))
            .or_else(|| deferred.pop().map(|s| (s, true)))
        {
            if self.step_hit || self.time_hit {
                break;
            }
            if !admitted {
                match self.admit(&s) {
                    Admit::New => {}
                    Admit::Variant => {
                        deferred.push(s);
                        continue;
                    }
                    Admit::Seen => continue,
                }
            }
            loop {
                if self.out_of_budget() {
                    break 'paths;
                }
                self.steps += 1;
                let pc = s.pc;
                if pc >= self.code.len() {
                    continue 'paths; // implicit STOP
                }
                self.visited[pc] = true;
                let op = self.code[pc];
                macro_rules! pop {
                    () => {
                        match s.stack.pop() {
                            Some(v) => v,
                            None => {
                                self.aborted += 1;
                                continue 'paths;
                            }
                        }
                    };
                }
                macro_rules! push {
                    ($v:expr) => {{
                        if s.stack.len() >= 1024 {
                            self.aborted += 1;
                            continue 'paths;
                        }
                        s.stack.push($v);
                    }};
                }
                let mut next = pc + 1;
                match op {
                    0x00 | 0xf3 | 0xfd | 0xfe => continue 'paths,
                    0x5f => push!(V::Const(U256::ZERO, 1)),
                    0x60..=0x7f => {
                        let n = (op - 0x5f) as usize;
                        let end = (pc + 1 + n).min(self.code.len());
                        let mut buf = [0u8; 32];
                        let bytes = &self.code[pc + 1..end];
                        buf[32 - n..32 - n + bytes.len()].copy_from_slice(bytes);
                        push!(V::Const(U256::from_be_bytes(buf), n as u8));
                        next = pc + 1 + n;
                    }
                    0x80..=0x8f => {
                        let n = (op - 0x7f) as usize;
                        if s.stack.len() < n {
                            self.aborted += 1;
                            continue 'paths;
                        }
                        let v = s.stack[s.stack.len() - n].clone();
                        push!(v);
                    }
                    0x90..=0x9f => {
                        let n = (op - 0x8f) as usize;
                        let len = s.stack.len();
                        if len < n + 1 {
                            self.aborted += 1;
                            continue 'paths;
                        }
                        s.stack.swap(len - 1, len - 1 - n);
                    }
                    0x50 => {
                        pop!();
                    }
                    0x5b => {}
                    0x20 => {
                        let off = pop!();
                        let len = pop!();
                        let v = match len
                            .as_const()
                            .filter(|l| l.bit_len() <= 16)
                            .map(|l| l.to::<u64>())
                        {
                            Some(l) if l > 0 && l <= 128 => {
                                let words = (0..l.div_ceil(32))
                                    .map(|i| {
                                        let a =
                                            mk(0x01, vec![off.clone(), V::c(U256::from(32 * i))]);
                                        s.mem.read_prefix(self.code, &a, (l - 32 * i).min(32))
                                    })
                                    .collect();
                                hash(words)
                            }
                            _ => V::Unknown,
                        };
                        push!(v);
                    }
                    0x30 => push!(V::This),
                    0x32 => {
                        self.fnacc(s.ctx).origin.insert(pc);
                        push!(V::Origin)
                    }
                    0x33 => {
                        self.fnacc(s.ctx).caller.insert(pc);
                        push!(V::Caller)
                    }
                    0x34 => push!(V::CallValue),
                    0x35 => {
                        let off = pop!();
                        push!(calldata(off))
                    }
                    0x37 => {
                        let dest = pop!();
                        let src = pop!();
                        let len = pop!();
                        let l = len
                            .as_const()
                            .filter(|l| l.bit_len() <= 24)
                            .map(|l| l.to::<u64>());
                        s.mem.write_len(&dest, l, &len, MemVal::CallData(src));
                    }
                    0x39 | 0x3e => {
                        let dest = pop!();
                        let src = pop!();
                        let len = pop!();
                        let l = len
                            .as_const()
                            .filter(|l| l.bit_len() <= 24)
                            .map(|l| l.to::<u64>());
                        let val = match (op, s.last_call, src.as_const()) {
                            (0x3e, Some(c), _) => MemVal::Ret(c),
                            (0x39, _, Some(src)) if src.bit_len() <= 32 => {
                                MemVal::Code(src.to::<u64>())
                            }
                            _ => MemVal::Unknown,
                        };
                        s.mem.write_len(&dest, l, &len, val);
                    }
                    0x3c => {
                        // extcodecopy(addr, dest, offset, len): external code
                        pop!();
                        let dest = pop!();
                        pop!();
                        let len = pop!();
                        let l = len
                            .as_const()
                            .filter(|l| l.bit_len() <= 24)
                            .map(|l| l.to::<u64>());
                        s.mem.write_len(&dest, l, &len, MemVal::Unknown);
                    }
                    0x5e => {
                        let dest = pop!();
                        pop!();
                        let len = pop!();
                        let l = len
                            .as_const()
                            .filter(|l| l.bit_len() <= 24)
                            .map(|l| l.to::<u64>());
                        s.mem.write_len(&dest, l, &len, MemVal::Unknown);
                    }
                    0x51 => {
                        let a = pop!();
                        let v = s.mem.read(self.code, &a);
                        push!(v)
                    }
                    0x52 => {
                        let a = pop!();
                        let v = pop!();
                        s.mem.write(&a, Some(32), MemVal::Word(v));
                    }
                    0x53 => {
                        let a = pop!();
                        pop!();
                        s.mem.write(&a, Some(1), MemVal::Unknown);
                    }
                    0x54 => {
                        let k = pop!();
                        push!(sload(k))
                    }
                    0x5c => {
                        let k = pop!();
                        push!(tload(k))
                    }
                    0x55 | 0x5d => {
                        let k = pop!();
                        pop!();
                        let site = StoreSite {
                            pc,
                            slot: match k.as_const() {
                                Some(c) => {
                                    let mut p = Provenance::kind("constant");
                                    p.value = Some(hex_u256(c));
                                    p
                                }
                                // Display only: wrap without the cap, the key is already bounded
                                None => provenance(&V::SLoad(Bx {
                                    depth: (k.depth() + 1) as u8,
                                    nodes: (k.nodes() + 1).min(255) as u8,
                                    v: Box::new(k),
                                })),
                            },
                            transient: op == 0x5d,
                        };
                        self.fnacc(s.ctx).sstores.insert(site);
                    }
                    0x56 | 0x57 => {
                        let dest = pop!();
                        let cond = if op == 0x57 { Some(pop!()) } else { None };
                        if let Some(c) = &cond {
                            self.scan_guards(pc, s.ctx, c);
                        }
                        // Constant condition: one branch only
                        let (take, fall) = match cond.as_ref().map(|c| c.as_const()) {
                            None => (true, false),
                            Some(Some(c)) => (!c.is_zero(), c.is_zero()),
                            Some(None) => (true, true),
                        };
                        if fall {
                            let mut f = s.clone();
                            f.pc = pc + 1;
                            work.push(f);
                        }
                        if take {
                            let targets: Vec<(usize, Ctx)> = match dest.as_const() {
                                Some(d) => match self.valid_dest(d) {
                                    Some(d) => {
                                        let mut ctx = s.ctx;
                                        if ctx == Ctx::Dispatcher {
                                            if let Some(sel) = cond.as_ref().and_then(selector_of) {
                                                ctx = Ctx::Selector(sel);
                                            } else if let Some(sel) = self.entries.get(&d) {
                                                ctx = Ctx::Selector(*sel);
                                            }
                                        }
                                        vec![(d, ctx)]
                                    }
                                    None => {
                                        self.aborted += 1;
                                        vec![]
                                    }
                                },
                                None => self.jump_table(pc, &dest, s.ctx),
                            };
                            for (d, ctx) in targets {
                                let mut t = s.clone();
                                t.pc = d;
                                t.ctx = ctx;
                                work.push(t);
                            }
                        }
                        continue 'paths;
                    }
                    0xf1 | 0xf2 | 0xf4 | 0xfa => {
                        let _gas = pop!();
                        let target = pop!();
                        let value = if matches!(op, 0xf1 | 0xf2) {
                            Some(pop!())
                        } else {
                            None
                        };
                        let args_off = pop!();
                        let args_len = pop!();
                        let ret_off = pop!();
                        let ret_len = pop!();
                        self.record_call(
                            pc,
                            op,
                            &s,
                            Some(&target),
                            value.as_ref(),
                            &args_off,
                            &args_len,
                        );
                        let l = ret_len
                            .as_const()
                            .filter(|l| l.bit_len() <= 24)
                            .map(|l| l.to::<u64>());
                        s.mem.write_len(&ret_off, l, &ret_len, MemVal::Ret(pc));
                        s.last_call = Some(pc);
                        push!(V::Ret(pc))
                    }
                    0xf0 | 0xf5 => {
                        let value = pop!();
                        let off = pop!();
                        let len = pop!();
                        if op == 0xf5 {
                            pop!();
                        }
                        self.record_call(pc, op, &s, None, Some(&value), &off, &len);
                        s.last_call = Some(pc);
                        push!(V::Ret(pc))
                    }
                    0xff => {
                        pop!();
                        self.fnacc(s.ctx).selfdestruct.insert(pc);
                        continue 'paths;
                    }
                    _ => match generic_arity(op) {
                        Some((pops, pushes)) => {
                            let mut args = Vec::with_capacity(pops);
                            for _ in 0..pops {
                                args.push(pop!());
                            }
                            if pushes == 1 {
                                let v = if pops == 0
                                    || matches!(op, 0x31 | 0x3b | 0x3f | 0x40 | 0x49)
                                {
                                    V::Env(op)
                                } else {
                                    mk(op, args)
                                };
                                push!(v)
                            }
                        }
                        None => {
                            // Undefined here (or newer than this table): the
                            // path is not followed, and coverage says so
                            self.invalid_opcodes.insert(pc);
                            continue 'paths;
                        }
                    },
                }
                s.pc = next;
                if next < self.code.len() && self.jumpdests[next] {
                    // fall-through into a block: dedupe
                    match self.admit(&s) {
                        Admit::New => {}
                        Admit::Variant => {
                            deferred.push(s);
                            continue 'paths;
                        }
                        Admit::Seen => continue 'paths,
                    }
                }
            }
        }
    }

    /// Enumerate a calldata-indexed jump table: `jump(byte_index << k)` or
    /// `jump(byte_index * 2^k)`. Anything else stays unresolved.
    fn jump_table(&mut self, pc: usize, dest: &V, ctx: Ctx) -> Vec<(usize, Ctx)> {
        let (idx, shift) = match dest {
            V::Op(0x1b, a) => match (a[0].as_const(), &a[1]) {
                (Some(k), x) if k <= U256::from(16u8) && byte_index(x) => (x, k.to::<u32>()),
                _ => (dest, u32::MAX),
            },
            V::Op(0x02, a) => match (&a[0], &a[1]) {
                (x, V::Const(m, _)) | (V::Const(m, _), x)
                    if byte_index(x) && m.count_ones() == 1 && m.bit_len() <= 17 =>
                {
                    (x, (m.bit_len() - 1) as u32)
                }
                _ => (dest, u32::MAX),
            },
            _ => (dest, u32::MAX),
        };
        if shift == u32::MAX {
            self.unresolved_jumps.insert(pc);
            return Vec::new();
        }
        // A table jumped to from inside a table entry (a chain of steps
        // packed in calldata): the entries it can reach are the same
        // JUMPDESTs, each already analysed as its own entry from the first
        // table, so the path ends here instead of fanning out again
        if matches!(ctx, Ctx::JumpTable(_)) && self.first_table.is_some_and(|f| f.1 == shift) {
            // Chained: expanded a bounded number of times per table pc over
            // all functions (each expansion keeps the caller's stack, which
            // entries rely on), then the path is pruned
            let n = self.chained_expansions.entry(pc).or_insert(0);
            *n += 1;
            if *n > self.budget.max_states_per_pc {
                self.pruned += 1;
                return Vec::new();
            }
            let mut out = Vec::new();
            for i in 0..256usize {
                let d = i << shift;
                if d < self.code.len() && self.jumpdests[d] {
                    out.push((d, ctx));
                }
            }
            let entries = out.len();
            self.jump_tables.entry(pc).or_insert_with(|| {
                (
                    format!("{} << {shift} (chained)", describe_short(idx)),
                    entries,
                )
            });
            return out;
        }
        if ctx == Ctx::Dispatcher && self.first_table.is_none() {
            self.first_table = Some((pc, shift));
        }
        // Enumerate each (pc, function) a bounded number of times
        let seen = self.table_enumerations.entry((pc, ctx)).or_insert(0);
        *seen += 1;
        if *seen > self.budget.max_states_per_pc {
            self.pruned += 1;
            return Vec::new();
        }
        let mut out = Vec::new();
        for i in 0..256usize {
            let d = i << shift;
            if d < self.code.len() && self.jumpdests[d] {
                let c = if ctx == Ctx::Dispatcher {
                    Ctx::JumpTable(i as u8)
                } else {
                    ctx
                };
                out.push((d, c));
            }
        }
        self.jump_tables.insert(
            pc,
            (format!("{} << {shift}", describe_short(idx)), out.len()),
        );
        if out.is_empty() {
            self.unresolved_jumps.insert(pc);
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn record_call(
        &mut self,
        pc: usize,
        op: u8,
        s: &State,
        target: Option<&V>,
        value: Option<&V>,
        args_off: &V,
        args_len: &V,
    ) {
        let acc = self.fnacc(s.ctx);
        match op {
            0xf4 => {
                acc.delegatecall.insert(pc);
            }
            0xf0 | 0xf5 => {
                acc.create.insert(pc);
            }
            _ => {}
        }
        acc.calls.insert(pc);
        let site = self.sites.entry((pc, s.ctx)).or_default();
        site.kind = opname(op);
        if let Some(t) = target {
            insert_capped(&mut site.target, provenance(t));
        }
        if let Some(v) = value {
            insert_capped(&mut site.value, provenance(v));
        }
        insert_capped(&mut site.args_len, provenance(args_len));
        if matches!(op, 0xf0 | 0xf5) {
            return;
        }
        // Selector: the first word of the argument area
        let len = args_len.as_const();
        if len.is_some_and(|l| l < U256::from(4u8)) {
            insert_capped(
                &mut site.selector,
                Provenance::kind("none").with_note("no calldata (plain transfer or fallback)"),
            );
            return;
        }
        // A precompile (0x01-0x11, 0x100) takes raw input, not a selector
        let precompile = target.and_then(|t| strip(t).as_const()).is_some_and(|a| {
            (U256::from(1u8)..=U256::from(0x11u8)).contains(&a) || a == U256::from(0x100u16)
        });
        let word = s.mem.read_prefix(self.code, args_off, 4);
        let sel = match strip(&word) {
            _ if precompile => {
                Provenance::kind("unresolved").with_note("precompile: raw input, no selector")
            }
            // A hash is not a selector, whatever its top bytes
            V::Hash(_) => Provenance::kind("unresolved").with_note("first word is a hash"),
            V::Const(c, _) if (*c >> 224usize) << 224usize == *c || c.bit_len() > 224 => {
                let mut p = Provenance::kind("constant");
                p.value = Some(format!("0x{:08x}", (*c >> 224usize).to::<u32>()));
                p
            }
            V::CallData(o) => {
                let mut p = Provenance::kind("calldata");
                p.offset = Some(
                    o.as_const()
                        .map(hex_u256)
                        .unwrap_or_else(|| "dynamic".into()),
                );
                p.note = Some("forwarded from the caller's calldata".into());
                p
            }
            other => {
                let p = provenance(other);
                if p.kind == "constant" {
                    Provenance::kind("unresolved")
                        .with_note("first word is a constant that is not a selector")
                } else {
                    p
                }
            }
        };
        insert_capped(&mut site.selector, sel);
        for i in 0..3u64 {
            let a = s.mem.read(
                self.code,
                &mk(0x01, vec![args_off.clone(), V::c(U256::from(4 + 32 * i))]),
            );
            if matches!(a, V::Unknown) && i > 0 {
                break;
            }
            insert_capped(&mut site.args[i as usize], provenance(&a));
        }
    }
}

/// `selector == PUSH4` in a dispatcher condition
fn selector_of(cond: &V) -> Option<u32> {
    match cond {
        V::Op(0x14, a) => match (&a[0], &a[1]) {
            (V::Const(c, _), x) | (x, V::Const(c, _))
                if c.bit_len() <= 32 && is_selector_expr(x) =>
            {
                Some(c.to::<u32>())
            }
            _ => None,
        },
        _ => None,
    }
}

/// Run the analysis.
///
/// `entries` maps function entry pcs (from evmole) to selectors, so a
/// dispatcher evmole understands but this pass does not still attributes
/// functions. `cfg_reachable` are evmole's reachable block starts, used for
/// coverage only.
#[must_use]
pub fn analyze_calls_with(
    code: &[u8],
    entries: &[(u32, usize)],
    cfg_reachable: &[usize],
    metadata_offset: Option<usize>,
    budget: CallsBudget,
) -> CallsReport {
    let mut notes = Vec::new();
    if code.is_empty() || code.len() > MAX_BYTECODE_SIZE {
        notes.push("No code to analyse (empty or larger than the limit)".into());
        return CallsReport {
            source: "inferred",
            bytecode_size: code.len(),
            call_sites: Vec::new(),
            functions: Vec::new(),
            opcode_sites: Vec::new(),
            jump_tables: Vec::new(),
            coverage: Coverage {
                complete: code.is_empty(),
                ..Default::default()
            },
            notes,
            visited: Vec::new(),
        };
    }
    let entry_map: HashMap<usize, u32> = entries.iter().map(|(s, pc)| (*pc, *s)).collect();
    let mut it = Interp::new(code, entry_map, budget);
    it.run();

    // Merge per-(pc, ctx) sites that carry the same facts
    let mut call_sites: Vec<CallSite> = Vec::new();
    for ((pc, ctx), acc) in &it.sites {
        let site = CallSite {
            pc: *pc,
            kind: acc.kind.to_string(),
            functions: vec![ctx.label()],
            target: acc.target.iter().cloned().collect(),
            selector: acc.selector.iter().cloned().collect(),
            value: acc.value.iter().cloned().collect(),
            args: acc
                .args
                .iter()
                .filter(|a| !a.is_empty())
                .map(|a| a.iter().cloned().collect())
                .collect(),
            args_len: acc.args_len.iter().cloned().collect(),
            resolved: !acc.target.iter().any(Provenance::is_unresolved),
        };
        if let Some(prev) = call_sites.last_mut() {
            if prev.pc == site.pc
                && prev.target == site.target
                && prev.selector == site.selector
                && prev.value == site.value
                && prev.args == site.args
            {
                prev.functions.extend(site.functions);
                continue;
            }
        }
        call_sites.push(site);
    }
    let unresolved_call_sites = call_sites
        .iter()
        .filter(|c| !c.resolved && !c.target.is_empty())
        .map(|c| c.pc)
        .collect::<BTreeSet<_>>()
        .len();

    let functions: Vec<FunctionFacts> = it
        .fns
        .iter()
        .map(|(ctx, f)| FunctionFacts {
            function: ctx.label(),
            signature: None,
            origin_pcs: f.origin.iter().copied().collect(),
            caller_pcs: f.caller.iter().copied().collect(),
            guards: f.guards.iter().cloned().collect(),
            call_pcs: f.calls.iter().copied().collect(),
            delegatecall_pcs: f.delegatecall.iter().copied().collect(),
            selfdestruct_pcs: f.selfdestruct.iter().copied().collect(),
            create_pcs: f.create.iter().copied().collect(),
            sstores: f.sstores.iter().cloned().collect(),
        })
        .filter(|f| {
            !(f.origin_pcs.is_empty()
                && f.caller_pcs.is_empty()
                && f.guards.is_empty()
                && f.call_pcs.is_empty()
                && f.selfdestruct_pcs.is_empty()
                && f.sstores.is_empty())
        })
        .collect();

    // Linear scan for sensitive opcodes, so nothing is missed even if no
    // path reached it
    let mut opcode_sites = opcode_sites(code, metadata_offset);
    for site in &mut opcode_sites {
        site.reached = it.visited[site.pc];
    }

    let cfg_set: HashSet<usize> = cfg_reachable.iter().copied().collect();
    let cfg_blocks_analysed = cfg_reachable
        .iter()
        .filter(|pc| **pc < code.len() && it.visited[**pc])
        .count();
    let extra_blocks_analysed = (0..code.len())
        .filter(|pc| it.jumpdests[*pc] && it.visited[*pc] && !cfg_set.contains(pc))
        .count();

    let unreached = opcode_sites
        .iter()
        .filter(|s| !s.reached && !s.in_metadata)
        .count();
    if unreached > 0 {
        notes.push(format!(
            "{unreached} sensitive opcode(s) in the code were not reached by any analysed path \
             (dead code, data, or beyond the budget); see opcode_sites"
        ));
    }
    let in_metadata = opcode_sites.iter().filter(|s| s.in_metadata).count();
    if in_metadata > 0 {
        notes.push(format!(
            "{in_metadata} sensitive opcode byte(s) sit in the trailing metadata region, which \
             holds no JUMPDEST: data, not code, so not counted as unreached; they are listed \
             with in_metadata: true"
        ));
    }
    if let Some(start) = metadata_offset.filter(|s| *s < code.len()) {
        if it.jumpdests[start..].iter().any(|d| *d) {
            notes.push(format!(
                "The metadata-looking trailer at {start} contains a JUMPDEST, so it is treated \
                 as code"
            ));
        }
    }
    if !it.invalid_opcodes.is_empty() {
        notes.push(format!(
            "Paths stopped at {} opcode(s) this analysis does not know; the code after them \
             was not analysed (coverage.invalid_opcodes)",
            it.invalid_opcodes.len()
        ));
    }
    if !it.jump_tables.is_empty() {
        notes.push(
            "A jump target computed from calldata was enumerated over every JUMPDEST it can \
             reach; jump_table[i] names the entry for index i. A table jumped to from inside an \
             entry (chained) is expanded a bounded number of times and keeps the first entry as \
             its function; prunes are counted in coverage.states_pruned"
                .into(),
        );
    }
    let coverage = Coverage {
        cfg_reachable_blocks: cfg_reachable.len(),
        cfg_blocks_analysed,
        extra_blocks_analysed,
        unresolved_jumps: it.unresolved_jumps.iter().copied().collect(),
        unresolved_call_sites,
        steps: it.steps,
        step_limit: budget.max_steps,
        elapsed_ms: it.started.elapsed().as_millis(),
        time_limit_ms: budget.max_time.as_millis(),
        step_budget_hit: it.step_hit,
        time_budget_hit: it.time_hit,
        states_pruned: it.pruned,
        paths_aborted: it.aborted,
        invalid_opcodes: it.invalid_opcodes.iter().copied().collect(),
        complete: !it.step_hit
            && !it.time_hit
            && it.pruned == 0
            && it.unresolved_jumps.is_empty()
            && it.invalid_opcodes.is_empty()
            && unresolved_call_sites == 0,
    };
    if coverage.step_budget_hit || coverage.time_budget_hit {
        notes.push("A budget was hit: results are partial".into());
    }
    let jump_tables = it
        .jump_tables
        .iter()
        .map(|(pc, (index, entries))| JumpTable {
            pc: *pc,
            index: index.clone(),
            entries: *entries,
        })
        .collect();
    CallsReport {
        source: "inferred",
        bytecode_size: code.len(),
        call_sites,
        functions,
        opcode_sites,
        jump_tables,
        coverage,
        notes,
        visited: it.visited.clone(),
    }
}

/// Run evmole (selectors, CFG, metadata) and then the call analysis
#[must_use]
pub fn analyze_calls(code: &[u8], budget: CallsBudget) -> CallsReport {
    if code.is_empty() || code.len() > MAX_BYTECODE_SIZE {
        return analyze_calls_with(code, &[], &[], None, budget);
    }
    let info = evmole::contract_info(
        evmole::ContractInfoArgs::new(code)
            .with_selectors()
            .with_control_flow_graph(),
    );
    let entries: Vec<(u32, usize)> = info
        .functions
        .iter()
        .flatten()
        .map(|f| (u32::from_be_bytes(f.selector), f.bytecode_offset))
        .collect();
    let mut reachable: Vec<usize> = info
        .control_flow_graph
        .map(|cfg| cfg.blocks.values().map(|b| b.start).collect())
        .unwrap_or_default();
    reachable.sort_unstable();
    reachable.dedup();
    let meta = evmole::contract_info(evmole::ContractInfoArgs::new(code).with_metadata())
        .metadata
        .map(|m| m.bytecode_offset);
    analyze_calls_with(code, &entries, &reachable, meta, budget)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(hex_code: &str) -> CallsReport {
        analyze_calls(&hex::decode(hex_code).unwrap(), CallsBudget::default())
    }

    #[test]
    fn caller_guard_against_constant_and_call_to_calldata_target() {
        // if (caller == 0x11..11) { call(gas, calldataload(4), 0, 0, 0, 0, 0) }
        let code = concat!(
            "73",
            "1111111111111111111111111111111111111111",
            "33",
            "14",
            "6019",
            "57",
            "00",
            // 0x19 is not a jumpdest yet: pad to place it
        );
        // Build precisely: PUSH20 a; CALLER; EQ; PUSH1 dest; JUMPI; STOP; JUMPDEST; ...
        let mut c = hex::decode(code).unwrap();
        let dest = c.len();
        let n = c.len();
        c[n - 3] = dest as u8; // PUSH1 operand
        c.extend(hex::decode("5b5f5f5f5f5f6004355af100").unwrap());
        let r = analyze_calls(&c, CallsBudget::default());
        assert_eq!(r.call_sites.len(), 1);
        let site = &r.call_sites[0];
        assert_eq!(site.kind, "CALL");
        assert_eq!(site.target[0].kind, "calldata");
        assert_eq!(site.target[0].offset.as_deref(), Some("0x4"));
        let guards: Vec<_> = r.functions.iter().flat_map(|f| f.guards.clone()).collect();
        assert_eq!(guards.len(), 1);
        assert_eq!(guards[0].subject, "msg_sender");
        assert_eq!(
            guards[0].against.value.as_deref(),
            Some("0x1111111111111111111111111111111111111111")
        );
        assert!(r.coverage.complete);
    }

    #[test]
    fn unresolved_jump_is_reported() {
        // jump(calldataload(0))
        let r = run("5f355600");
        assert_eq!(r.coverage.unresolved_jumps, vec![2]);
        assert!(!r.coverage.complete);
    }

    #[test]
    fn origin_found_by_scan_even_if_unreached() {
        // STOP; ORIGIN
        let r = run("0032");
        assert_eq!(r.opcode_sites.len(), 1);
        assert!(!r.opcode_sites[0].reached);
    }

    #[test]
    fn selector_read_back_from_memory() {
        // mstore(0x80, 0xa9059cbb << 224); call(gas, 0xaa.., 0, 0x80, 0x44, 0, 0)
        let code = concat!(
            "63a9059cbb",
            "60e0",
            "1b",
            "6080",
            "52", // mstore(0x80, sel<<224)
            "5f5f",
            "6044",
            "6080",
            "5f", // retLen retOff argsLen argsOff value
            "73aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "5a",
            "f1",
            "00"
        );
        let r = run(code);
        let site = &r.call_sites[0];
        assert_eq!(site.selector[0].value.as_deref(), Some("0xa9059cbb"));
        assert_eq!(site.target[0].kind, "code_constant");
    }

    /// Hostile code that nests values thousands deep must not overflow the
    /// stack (Drop, Hash, Eq) or blow up: values past the caps are Unknown
    #[test]
    fn deep_value_chains_are_capped() {
        let mut code = String::from("5f");
        code.push_str(&"35".repeat(5000)); // calldataload(calldataload(...))
        code.push_str(&"54".repeat(5000)); // sload(sload(...))
        code.push_str(&"5c".repeat(2000)); // tload(...)
        code.push_str(&"8001".repeat(2000)); // x + x, doubling the tree
                                             // keccak(mstore(0, x)) chained
        code.push_str(&"5f526020 5f20".replace(' ', "").repeat(2000));
        // call(gas, x, 0, 0, 0, 0, 0) and a branch on x
        code.push_str("5f5f5f5f5f855af1");
        code.push_str("80600057");
        code.push_str("00");
        let r = run(&code);
        assert_eq!(r.call_sites.len(), 1);
        // Bounded: the last keccak is shallow again, over unknown inputs
        assert!(matches!(
            r.call_sites[0].target[0].kind.as_str(),
            "keccak" | "unresolved"
        ));
        // Direct check of the caps
        let mut v = V::Caller;
        for _ in 0..10_000 {
            v = calldata(sload(v));
        }
        assert!(v.depth() <= MAX_DEPTH && v.nodes() <= MAX_NODES);
        let mut w = V::Caller;
        for _ in 0..100 {
            w = mk(0x01, vec![w.clone(), w]);
        }
        assert!(w.depth() <= MAX_DEPTH && w.nodes() <= MAX_NODES);
    }

    /// Reviewer repro: x = keccak(x, x, x, x) twelve times grew 4x per round
    #[test]
    fn nested_keccak_stays_small() {
        let mut code = String::from("5f35"); // x = calldataload(0)
        for _ in 0..40 {
            // mstore(0,x) mstore(32,x) mstore(64,x) mstore(96,x); x = keccak(0,128)
            code.push_str("805f52 80602052 80604052 80606052 50 60805f20");
        }
        code.push_str("00");
        let code = code.replace(' ', "");
        let t = std::time::Instant::now();
        let r = run(&code);
        assert!(t.elapsed() < std::time::Duration::from_secs(5));
        assert!(r.coverage.invalid_opcodes.is_empty());
    }

    /// Paths that differ only in memory reach a shared call: both targets
    #[test]
    fn memory_differences_are_not_deduplicated_away() {
        // if calldataload(0) { mstore(0, 0x11..11) } else { mstore(0, calldataload(4)) }
        // call(gas, mload(0), 0, 0, 0, 0, 0)
        let c = concat!(
            "5f35",
            "6011",
            "57", // 0: jumpi(0x11, cd0)
            "600435",
            "5f52",
            "6033",
            "56", // 5: else branch, jump 0x33
            "00",
            "00",
            "00",
            "00", // pad to 0x11
            "5b",
            "73",
            "1111111111111111111111111111111111111111",
            "5f52", // 0x11
            "6033",
            "56", // jump 0x33
            "00",
            "00",
            "00",
            "00",
            "00",
            "00",
            "00", // pad to 0x33
            "5b",
            "5f5f5f5f5f",
            "5f51",
            "5af1",
            "00" // 0x33
        );
        let bytes = hex::decode(c).unwrap();
        // place the join JUMPDEST exactly at 0x33
        let join = bytes.iter().rposition(|b| *b == 0x5b).unwrap();
        assert_eq!(join, 0x33, "test layout");
        let r = analyze_calls(&bytes, CallsBudget::default());
        let kinds: Vec<&str> = r.call_sites[0]
            .target
            .iter()
            .map(|p| p.kind.as_str())
            .collect();
        assert!(kinds.contains(&"calldata"), "{kinds:?}");
        assert!(kinds.contains(&"code_constant"), "{kinds:?}");
    }

    /// A write through a calldata address or EXTCODECOPY may overwrite a
    /// constant: the constant must not be reported as the target
    #[test]
    fn symbolic_writes_and_extcodecopy_invalidate_memory() {
        let set = "731111111111111111111111111111111111111111 5f52";
        let call = "5f5f5f5f5f 5f51 5a f1 00";
        for middle in [
            "602435 600435 52",     // mstore(calldataload(4), calldataload(0x24))
            "6020 5f 5f 600435 3c", // extcodecopy(calldataload(4), 0, 0, 32)
        ] {
            let code = format!("{set}{middle}{call}").replace(' ', "");
            let r = run(&code);
            assert_eq!(r.call_sites[0].target[0].kind, "unresolved", "{middle}");
        }
    }

    /// CLZ (0x1e) is known; an undefined opcode stops the path and is counted
    #[test]
    fn clz_and_unknown_opcodes() {
        // PUSH1 1; CLZ; POP; ORIGIN; POP; STOP
        let r = run("60011e50325000");
        assert!(r
            .opcode_sites
            .iter()
            .any(|s| s.opcode == "ORIGIN" && s.reached));
        let r = run("0c325000"); // 0x0c is undefined
        assert_eq!(r.coverage.invalid_opcodes, vec![0]);
        assert!(!r.coverage.complete);
    }

    /// A constant in memory that a later write may have overwritten is not
    /// reported as the call target (the review's cases a-d, plus a write at
    /// returndatasize())
    #[test]
    fn stale_constant_cases_are_unresolved() {
        let k = "731111111111111111111111111111111111111111";
        let call_with = |target_load: &str| format!("5f5f5f5f5f{target_load}5af100");
        let cases = [
            // (a) constant at 0x40, write at calldataload(4), read 0x40
            format!("{k}604052 602435600435 52 {}", call_with("604051")),
            // (b) the same through extcodecopy to a calldata destination
            format!("{k}604052 6020 5f 600435 5f 3c {}", call_with("604051")),
            // (c) constant at cd4, write at 0, read cd4
            format!("{k}600435 52 5f35 5f 52 {}", call_with("60043551")),
            // (d) constant at cd4 + cd44, write at cd4, read cd4 + cd44
            format!(
                "{k}604435600435 01 52 602435 600435 52 {}",
                call_with("604435600435 01 51")
            ),
            // (e) constant at 0, write at returndatasize() (0 before a call)
            format!("{k}5f52 602435 3d 52 {}", call_with("5f51")),
        ];
        for (i, code) in cases.iter().enumerate() {
            let r = run(&code.replace(' ', ""));
            assert_eq!(
                r.call_sites[0].target[0].kind, "unresolved",
                "case {i}: {:?}",
                r.call_sites[0].target
            );
        }
    }

    /// 1inch v5 style: selector at 0x80, a dynamic copy after it, then a
    /// word written right at the copy's end; Safe fallback style: calldata
    /// copied to 0 and the caller appended at calldatasize
    #[test]
    fn writes_past_a_dynamic_copy_keep_earlier_words() {
        let inch = concat!(
            "63aabbccdd60e01b608052", // mstore(0x80, sel << 224)
            "33608452",               // mstore(0x84, caller)
            "602435604460a437",       // calldatacopy(0xa4, 0x44, cd(0x24))
            "6001602435 60a4 01 52",  // mstore(0xa4 + cd(0x24), 1)
            "5f5f60446080 5f 600435 5af100"
        )
        .replace(' ', "");
        let r = run(&inch);
        let site = &r.call_sites[0];
        assert_eq!(site.selector[0].value.as_deref(), Some("0xaabbccdd"));
        assert_eq!(site.args[0][0].kind, "msg_sender");

        let safe = concat!(
            "365f5f37",         // calldatacopy(0, 0, calldatasize)
            "33 6060 1b 36 52", // mstore(calldatasize, caller << 96)
            "5f5f 6014 36 01 5f 5f 731111111111111111111111111111111111111111 5af100"
        )
        .replace(' ', "");
        let r = run(&safe);
        let site = &r.call_sites[0];
        assert_eq!(site.selector[0].kind, "calldata");
        assert_eq!(site.selector[0].offset.as_deref(), Some("0x0"));
    }

    fn guards_for(cond: &str) -> Vec<Guard> {
        let cond = cond.replace(' ', "");
        let dest = cond.len() / 2 + 4;
        let code = format!("{cond}60{dest:02x}57005b00");
        run(&code)
            .functions
            .iter()
            .flat_map(|f| f.guards.clone())
            .collect()
    }

    /// SUB / XOR are equality tests only as the branch condition itself
    /// (or under ISZERO / eq(_, 0)); masked or further arithmetic is not
    #[test]
    fn sub_xor_guards_only_when_direct() {
        for not_guard in [
            "5f54 33 03 6001 16",   // and(sub(caller, s0), 1)
            "600435 5f54 33 03 03", // sub(sub(caller, s0), cd4)
            "600435 5f54 33 18 18", // xor(xor(caller, s0), cd4)
            "5f54 33 18 60ff 16",   // and(xor(caller, s0), 0xff)
        ] {
            assert!(guards_for(not_guard).is_empty(), "{not_guard}");
        }
        for guard in [
            "5f54 33 03",       // sub(caller, s0)
            "5f54 33 18 15",    // iszero(xor(caller, s0))
            "5f 5f54 33 03 14", // eq(sub(caller, s0), 0)
        ] {
            let g = guards_for(guard);
            assert_eq!(g.len(), 1, "{guard}");
            assert_eq!(g[0].against.slot.as_deref(), Some("0x0"));
        }
        // address(this) compared with itself is not a guard
        assert!(guards_for("30 30 14").is_empty());
    }

    /// Sensitive opcodes inside a forged trailer are still listed; a trailer
    /// with a JUMPDEST is code, not metadata
    #[test]
    fn opcode_sites_scan_past_the_metadata_offset() {
        let code = hex::decode("5f35560000005b32ff").unwrap();
        let with_dest = opcode_sites(&code, Some(4));
        assert_eq!(with_dest.len(), 2);
        assert!(with_dest.iter().all(|s| !s.in_metadata));
        let plain = hex::decode("00a232ff").unwrap();
        let sites = opcode_sites(&plain, Some(1));
        assert_eq!(sites.len(), 2);
        assert!(sites.iter().all(|s| s.in_metadata));
    }

    #[test]
    fn empty_code() {
        let r = analyze_calls(&[], CallsBudget::default());
        assert!(r.call_sites.is_empty());
    }

    #[test]
    fn dedup_compares_full_states() {
        let code = hex::decode("5b00").unwrap();
        let mut it = Interp::new(&code, HashMap::new(), CallsBudget::default());
        let st = |stack: Vec<V>| State {
            pc: 0,
            stack,
            mem: Memory::default(),
            ctx: Ctx::Dispatcher,
            last_call: None,
        };
        let a = st(vec![V::Caller]);
        let b = st(vec![V::Origin]);
        assert!(matches!(it.admit(&a), Admit::New));
        assert!(matches!(it.admit(&b), Admit::New));
        assert!(matches!(it.admit(&a), Admit::Seen));
        // Stored keys are the full stacks, not hashes
        let stacks = &it.seen[&(0, Ctx::Dispatcher)];
        assert!(stacks.contains_key(&vec![V::Caller]) && stacks.contains_key(&vec![V::Origin]));
    }
}
