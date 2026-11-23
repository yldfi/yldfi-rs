//! Read contract storage variables by name, using the compiler's storage
//! layout (`storageLayout` in solc's standard JSON output)
//!
//! [`Layout::locate`] turns a variable path (`balances`, `allowed[a][b]`,
//! `positions.liquidity`) into a slot, byte offset and type, following
//! Solidity's storage rules: value types packed from the low-order end of a
//! slot, mapping values at `keccak256(key . slot)`, dynamic array data at
//! `keccak256(slot)`, struct members at their own relative slots.
//! [`Layout::decode`] then reads what it needs through a [`SlotReader`] and
//! returns the value as JSON.

use alloy::primitives::{keccak256, Address, I256, U256};
use anyhow::{anyhow, bail, Context, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

/// Most array elements or struct members expanded when a whole array or
/// struct is read
const MAX_ELEMENTS: usize = 32;
/// How deep nested structs and arrays are expanded
const MAX_DEPTH: usize = 4;
/// Longest `string`/`bytes` value read, in bytes
const MAX_BYTES_LEN: usize = 32 * 1024;

/// A variable or struct member as the layout lists it
#[derive(Debug, Clone)]
pub struct Var {
    pub label: String,
    pub slot: U256,
    pub offset: usize,
    pub type_id: String,
    /// Contract that declares it (source path stripped)
    pub contract: Option<String>,
}

/// A type as the layout describes it
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub label: String,
    /// `inplace`, `mapping`, `dynamic_array` or `bytes`
    pub encoding: String,
    pub size: usize,
    pub key: Option<String>,
    pub value: Option<String>,
    pub base: Option<String>,
    pub members: Option<Vec<Var>>,
}

/// A contract's storage layout
#[derive(Debug, Clone)]
pub struct Layout {
    pub storage: Vec<Var>,
    pub types: HashMap<String, TypeDef>,
}

/// Where a value lives
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub slot: U256,
    pub offset: usize,
    pub type_id: String,
    /// Dynamic array indexes on the way, to check against the stored length:
    /// (slot holding the length, index, path so far)
    pub array_checks: Vec<(U256, usize, String)>,
}

/// Reads one storage slot. Implemented over RPC by the CLI and over a map in
/// tests.
pub trait SlotReader: Sync {
    fn read(&self, slot: U256) -> Pin<Box<dyn Future<Output = Result<U256>> + Send + '_>>;
}

fn parse_u256(value: &Value) -> Option<U256> {
    match value {
        Value::String(s) => parse_uint(s).ok(),
        Value::Number(n) => n.as_u64().map(U256::from),
        _ => None,
    }
}

fn parse_usize(value: &Value) -> Option<usize> {
    parse_u256(value).and_then(|v| usize::try_from(v).ok())
}

fn parse_var(value: &Value) -> Result<Var> {
    let label = value["label"]
        .as_str()
        .ok_or_else(|| anyhow!("layout entry without a label"))?
        .to_string();
    Ok(Var {
        slot: parse_u256(&value["slot"]).ok_or_else(|| anyhow!("'{label}' has no slot"))?,
        offset: parse_usize(&value["offset"]).unwrap_or(0),
        type_id: value["type"]
            .as_str()
            .ok_or_else(|| anyhow!("'{label}' has no type"))?
            .to_string(),
        // `contract` is "path/to/File.sol:Name"; keep only the name
        contract: value["contract"]
            .as_str()
            .map(|c| c.rsplit(':').next().unwrap_or(c).to_string()),
        label,
    })
}

impl Layout {
    /// Parse a `storageLayout` object (`{"storage": [...], "types": {...}}`)
    pub fn parse(value: &Value) -> Result<Self> {
        let storage = value["storage"]
            .as_array()
            .ok_or_else(|| anyhow!("storage layout has no 'storage' list"))?
            .iter()
            .map(parse_var)
            .collect::<Result<Vec<_>>>()?;
        let mut types = HashMap::new();
        if let Some(map) = value["types"].as_object() {
            for (id, t) in map {
                let members = match t["members"].as_array() {
                    Some(list) => Some(list.iter().map(parse_var).collect::<Result<Vec<_>>>()?),
                    None => None,
                };
                let text = |k: &str| t[k].as_str().map(str::to_string);
                types.insert(
                    id.clone(),
                    TypeDef {
                        label: text("label").unwrap_or_else(|| id.clone()),
                        encoding: text("encoding").unwrap_or_else(|| "inplace".into()),
                        size: parse_usize(&t["numberOfBytes"]).unwrap_or(32),
                        key: text("key"),
                        value: text("value"),
                        base: text("base"),
                        members,
                    },
                );
            }
        }
        Ok(Self { storage, types })
    }

    pub fn type_def(&self, id: &str) -> Result<&TypeDef> {
        self.types
            .get(id)
            .ok_or_else(|| anyhow!("type '{id}' is missing from the storage layout"))
    }

    /// The label of a type, for display
    pub fn type_label(&self, id: &str) -> String {
        self.types
            .get(id)
            .map(|t| t.label.clone())
            .unwrap_or_else(|| id.to_string())
    }

    /// Find where `name` followed by `steps` (mapping keys, array indexes and
    /// struct member names, in order) is stored
    pub fn locate(&self, name: &str, steps: &[String]) -> Result<Location> {
        let (var, steps) = self.select_var(name, steps)?;
        let mut loc = Location {
            slot: var.slot,
            offset: var.offset,
            type_id: var.type_id.clone(),
            array_checks: Vec::new(),
        };
        let mut path = var.label.clone();
        for step in steps {
            let t = self.type_def(&loc.type_id)?;
            loc = if t.encoding == "mapping" {
                let key_type = self.type_def(t.key.as_deref().unwrap_or_default())?;
                let key = encode_key(&key_type.label, step)
                    .with_context(|| format!("key '{step}' for {path} ({})", t.label))?;
                let mut preimage = key;
                preimage.extend_from_slice(&loc.slot.to_be_bytes::<32>());
                Location {
                    slot: U256::from_be_bytes(keccak256(&preimage).0),
                    offset: 0,
                    type_id: t.value.clone().unwrap_or_default(),
                    array_checks: loc.array_checks,
                }
            } else if let Some(base) = &t.base {
                let index: usize = parse_uint(step)
                    .ok()
                    .and_then(|i| usize::try_from(i).ok())
                    .ok_or_else(|| anyhow!("'{step}' is not an index for {path} ({})", t.label))?;
                let elem = self.type_def(base)?;
                let mut array_checks = loc.array_checks;
                let start = if t.encoding == "dynamic_array" {
                    array_checks.push((loc.slot, index, path.clone()));
                    U256::from_be_bytes(keccak256(loc.slot.to_be_bytes::<32>()).0)
                } else {
                    if let Some(len) = static_len(&t.label) {
                        if index >= len {
                            bail!("index {index} is out of bounds for {path} ({})", t.label);
                        }
                    }
                    loc.slot
                };
                let (slot, offset) = element_position(start, elem.size, index);
                Location {
                    slot,
                    offset,
                    type_id: base.clone(),
                    array_checks,
                }
            } else if let Some(members) = &t.members {
                let member = members.iter().find(|m| m.label == *step).ok_or_else(|| {
                    let names: Vec<&str> = members.iter().map(|m| m.label.as_str()).collect();
                    anyhow!(
                        "{path} ({}) has no member '{step}'. Members: {}",
                        t.label,
                        names.join(", ")
                    )
                })?;
                Location {
                    slot: loc.slot.wrapping_add(member.slot),
                    offset: member.offset,
                    type_id: member.type_id.clone(),
                    array_checks: loc.array_checks,
                }
            } else {
                bail!(
                    "{path} is a {}, which takes no key, index or member ('{step}' left over)",
                    t.label
                );
            };
            path = format!("{path}[{step}]");
        }
        Ok(loc)
    }

    /// Pick the variable `name` names. Several variables can share a label
    /// (private variables of different base contracts, `__gap`); then
    /// `Contract.name` or `name@slot` selects one. Returns the variable and
    /// the steps left after a `Contract.` prefix.
    fn select_var<'a, 's>(
        &'a self,
        name: &str,
        steps: &'s [String],
    ) -> Result<(&'a Var, &'s [String])> {
        let (label, at_slot) = match name.split_once('@') {
            Some((label, slot)) => (
                label,
                Some(parse_uint(slot).with_context(|| format!("slot in '{name}'"))?),
            ),
            None => (name, None),
        };
        let by_label = |label: &str, contract: Option<&str>| -> Vec<&'a Var> {
            self.storage
                .iter()
                .filter(|v| v.label == label)
                .filter(|v| at_slot.is_none_or(|s| v.slot == s))
                .filter(|v| contract.is_none_or(|c| v.contract.as_deref() == Some(c)))
                .collect()
        };
        let mut rest = steps;
        let mut matches = by_label(label, None);
        // `Contract.var`: the first step is the variable
        if matches.is_empty() && at_slot.is_none() {
            if let Some((first, tail)) = steps.split_first() {
                let (inner, inner_slot) = match first.split_once('@') {
                    Some((l, s)) => (l, Some(parse_uint(s)?)),
                    None => (first.as_str(), None),
                };
                matches = self
                    .storage
                    .iter()
                    .filter(|v| v.label == inner && v.contract.as_deref() == Some(label))
                    .filter(|v| inner_slot.is_none_or(|s| v.slot == s))
                    .collect();
                rest = tail;
            }
        }
        match matches.as_slice() {
            [one] => Ok((one, rest)),
            [] => {
                let names: Vec<&str> = self.storage.iter().map(|v| v.label.as_str()).collect();
                bail!(
                    "no variable '{name}' in the storage layout. Variables: {}",
                    names.join(", ")
                )
            }
            many => {
                let list: Vec<String> = many
                    .iter()
                    .map(|v| match &v.contract {
                        Some(c) => format!("{}@{} ({c}.{})", v.label, v.slot, v.label),
                        None => format!("{}@{}", v.label, v.slot),
                    })
                    .collect();
                bail!(
                    "'{name}' is ambiguous: {} variables share that name. Pick one with name@slot or Contract.name: {}",
                    many.len(),
                    list.join(", ")
                )
            }
        }
    }

    /// Check dynamic array indexes in `loc` against the stored lengths
    pub async fn check_bounds<R: SlotReader>(&self, reader: &R, loc: &Location) -> Result<()> {
        for (len_slot, index, path) in &loc.array_checks {
            let len = reader.read(*len_slot).await?;
            if U256::from(*index) >= len {
                bail!("index {index} is out of bounds for {path} (length {len})");
            }
        }
        Ok(())
    }

    /// Read and decode the value at `loc`
    pub async fn decode<R: SlotReader>(&self, reader: &R, loc: &Location) -> Result<Value> {
        self.decode_at(reader, loc.slot, loc.offset, &loc.type_id, 0)
            .await
    }

    fn decode_at<'a, R: SlotReader>(
        &'a self,
        reader: &'a R,
        slot: U256,
        offset: usize,
        type_id: &'a str,
        depth: usize,
    ) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            let t = self.type_def(type_id)?;
            match t.encoding.as_str() {
                "mapping" => bail!("this is a {}: give a key to read one entry", t.label),
                "bytes" => {
                    let data = read_bytes(reader, slot).await?;
                    Ok(if t.label == "string" {
                        Value::String(String::from_utf8_lossy(&data).into_owned())
                    } else {
                        Value::String(format!("0x{}", hex::encode(&data)))
                    })
                }
                "dynamic_array" => {
                    let len = reader.read(slot).await?;
                    let base = t.base.as_deref().unwrap_or_default();
                    let elem = self.type_def(base)?;
                    let start = U256::from_be_bytes(keccak256(slot.to_be_bytes::<32>()).0);
                    let mut out = json!({ "length": len.to_string() });
                    if depth < MAX_DEPTH {
                        let shown = usize::try_from(len).unwrap_or(usize::MAX).min(MAX_ELEMENTS);
                        let mut items = Vec::with_capacity(shown);
                        for i in 0..shown {
                            let (s, o) = element_position(start, elem.size, i);
                            items.push(self.decode_at(reader, s, o, base, depth + 1).await?);
                        }
                        out["elements"] = Value::Array(items);
                        if U256::from(shown) < len {
                            out["truncated"] = Value::Bool(true);
                        }
                    }
                    Ok(out)
                }
                _ if t.members.is_some() => {
                    let members = t.members.as_ref().expect("checked");
                    if depth >= MAX_DEPTH {
                        return Ok(
                            json!({ "type": t.label, "note": "nested too deep; name a member" }),
                        );
                    }
                    let mut out = serde_json::Map::new();
                    for m in members {
                        let v = match self
                            .decode_at(
                                reader,
                                slot.wrapping_add(m.slot),
                                m.offset,
                                &m.type_id,
                                depth + 1,
                            )
                            .await
                        {
                            Ok(v) => v,
                            // a mapping inside a struct can't be listed
                            Err(_) if self.type_def(&m.type_id)?.encoding == "mapping" => {
                                Value::String(format!("<{}>", self.type_label(&m.type_id)))
                            }
                            Err(e) => return Err(e),
                        };
                        out.insert(m.label.clone(), v);
                    }
                    Ok(Value::Object(out))
                }
                _ if t.base.is_some() => {
                    let base = t.base.as_deref().expect("checked");
                    let elem = self.type_def(base)?;
                    let len = static_len(&t.label).unwrap_or(0);
                    if depth >= MAX_DEPTH {
                        return Ok(
                            json!({ "length": len, "note": "nested too deep; give an index" }),
                        );
                    }
                    let shown = len.min(MAX_ELEMENTS);
                    let mut items = Vec::with_capacity(shown);
                    for i in 0..shown {
                        let (s, o) = element_position(slot, elem.size, i);
                        items.push(self.decode_at(reader, s, o, base, depth + 1).await?);
                    }
                    if shown < len {
                        Ok(json!({ "length": len, "elements": items, "truncated": true }))
                    } else {
                        Ok(Value::Array(items))
                    }
                }
                _ => {
                    let word = reader.read(slot).await?;
                    Ok(decode_value(&t.label, &extract(word, offset, t.size)?))
                }
            }
        })
    }
}

/// Length of a fixed-size array type label such as `uint256[3]`
fn static_len(label: &str) -> Option<usize> {
    let inner = label.strip_suffix(']')?;
    inner[inner.rfind('[')? + 1..].parse().ok()
}

/// Slot and offset of element `index` of an array whose data starts at
/// `start`. Elements of 16 bytes or less share slots; larger ones each take
/// whole slots.
pub fn element_position(start: U256, elem_size: usize, index: usize) -> (U256, usize) {
    if elem_size > 0 && elem_size <= 16 {
        let per_slot = 32 / elem_size;
        (
            start.wrapping_add(U256::from(index / per_slot)),
            (index % per_slot) * elem_size,
        )
    } else {
        let slots = elem_size.div_ceil(32).max(1);
        (start.wrapping_add(U256::from(index) * U256::from(slots)), 0)
    }
}

/// The `size` bytes at `offset` (counted from the low-order end) of a word
pub fn extract(word: U256, offset: usize, size: usize) -> Result<Vec<u8>> {
    if size == 0 || offset + size > 32 {
        bail!("offset {offset} and size {size} do not fit in a slot");
    }
    let bytes = word.to_be_bytes::<32>();
    Ok(bytes[32 - offset - size..32 - offset].to_vec())
}

/// Decode a value type from its packed bytes
pub fn decode_value(label: &str, bytes: &[u8]) -> Value {
    let mut word = [0u8; 32];
    word[32 - bytes.len()..].copy_from_slice(bytes);
    let unsigned = U256::from_be_bytes(word);
    if label == "bool" {
        return Value::Bool(!unsigned.is_zero());
    }
    if (label == "address" || label == "address payable" || label.starts_with("contract "))
        && bytes.len() == 20
    {
        return Value::String(Address::from_slice(bytes).to_checksum(None));
    }
    if label.starts_with("uint") || label.starts_with("enum ") {
        return Value::String(unsigned.to_string());
    }
    if label.starts_with("int") {
        // sign-extend from the value's own width
        if bytes.first().is_some_and(|b| b & 0x80 != 0) {
            word[..32 - bytes.len()].fill(0xff);
        }
        return Value::String(I256::from_be_bytes(word).to_string());
    }
    Value::String(format!("0x{}", hex::encode(bytes)))
}

/// Read a `string` or `bytes` value stored at `slot`
async fn read_bytes<R: SlotReader>(reader: &R, slot: U256) -> Result<Vec<u8>> {
    let word = reader.read(slot).await?;
    let bytes = word.to_be_bytes::<32>();
    if !word.bit(0) {
        // short form: data in the high-order bytes, length * 2 in the lowest
        let len = (bytes[31] / 2) as usize;
        return Ok(bytes[..len.min(31)].to_vec());
    }
    let len = usize::try_from((word - U256::from(1)) >> 1)
        .ok()
        .filter(|l| *l <= MAX_BYTES_LEN)
        .ok_or_else(|| {
            anyhow!("value is longer than {MAX_BYTES_LEN} bytes; read it with rpc storage")
        })?;
    let start = U256::from_be_bytes(keccak256(slot.to_be_bytes::<32>()).0);
    let mut data = Vec::with_capacity(len);
    let mut i = 0u64;
    while data.len() < len {
        let chunk = reader.read(start.wrapping_add(U256::from(i))).await?;
        data.extend_from_slice(&chunk.to_be_bytes::<32>());
        i += 1;
    }
    data.truncate(len);
    Ok(data)
}

fn parse_uint(s: &str) -> Result<U256> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        U256::from_str_radix(hex, 16).map_err(|e| anyhow!("invalid hex number '{s}': {e}"))
    } else {
        U256::from_str_radix(s, 10).map_err(|e| anyhow!("invalid number '{s}': {e}"))
    }
}

/// Encode a mapping key the way Solidity hashes it: value types as one
/// 32-byte word, `string` and `bytes` as their raw bytes
pub fn encode_key(label: &str, key: &str) -> Result<Vec<u8>> {
    let word = |v: [u8; 32]| Ok(v.to_vec());
    if label == "string" {
        return Ok(key.as_bytes().to_vec());
    }
    if label == "bytes" {
        return hex::decode(key.trim_start_matches("0x")).map_err(|e| anyhow!("invalid hex: {e}"));
    }
    if label == "address" || label == "address payable" || label.starts_with("contract ") {
        let addr: Address = key.parse().map_err(|e| anyhow!("invalid address: {e}"))?;
        let mut w = [0u8; 32];
        w[12..].copy_from_slice(addr.as_slice());
        return word(w);
    }
    if label == "bool" {
        let v = match key {
            "true" | "1" => 1u8,
            "false" | "0" => 0,
            _ => bail!("expected true or false"),
        };
        let mut w = [0u8; 32];
        w[31] = v;
        return word(w);
    }
    if let Some(bits) = int_bits(label, "uint") {
        let v = parse_uint(key)?;
        if bits < 256 && v >> bits != U256::ZERO {
            bail!("{v} does not fit in {label}");
        }
        return word(v.to_be_bytes::<32>());
    }
    if label.starts_with("enum ") {
        let v = parse_uint(key)?;
        if v > U256::from(255) {
            bail!("{v} does not fit in an enum");
        }
        return word(v.to_be_bytes::<32>());
    }
    if let Some(bits) = int_bits(label, "int") {
        let v: I256 = if let Some(hex) = key.strip_prefix("0x").or_else(|| key.strip_prefix("0X")) {
            // hex is the type's own two's complement: sign-extend from its width
            let raw = U256::from_str_radix(hex, 16).map_err(|e| anyhow!("invalid hex: {e}"))?;
            if bits < 256 && raw >> bits != U256::ZERO {
                bail!("{key} is wider than {label}");
            }
            let negative = raw.bit(bits - 1);
            let extended = if negative && bits < 256 {
                raw | (U256::MAX << bits)
            } else {
                raw
            };
            I256::from_raw(extended)
        } else {
            let v = I256::from_dec_str(key).map_err(|e| anyhow!("invalid integer: {e}"))?;
            if bits < 256 {
                let max = I256::from_raw((U256::from(1) << (bits - 1)) - U256::from(1));
                let min = -max - I256::ONE;
                if v > max || v < min {
                    bail!("{v} does not fit in {label}");
                }
            }
            v
        };
        return word(v.to_be_bytes::<32>());
    }
    if let Some(n) = label
        .strip_prefix("bytes")
        .and_then(|n| n.parse::<usize>().ok())
    {
        let raw =
            hex::decode(key.trim_start_matches("0x")).map_err(|e| anyhow!("invalid hex: {e}"))?;
        if raw.len() != n {
            bail!("{label} key must be exactly {n} bytes, got {}", raw.len());
        }
        let mut w = [0u8; 32];
        w[..raw.len()].copy_from_slice(&raw);
        return word(w);
    }
    // user-defined value types and anything else: a full 32-byte word
    let raw = hex::decode(key.trim_start_matches("0x"))
        .ok()
        .filter(|r| r.len() == 32)
        .ok_or_else(|| {
            anyhow!("key type '{label}' is not supported; pass the 32-byte key as hex")
        })?;
    Ok(raw)
}

/// Width in bits of `uintN`/`intN` (`uint` alone is 256)
fn int_bits(label: &str, prefix: &str) -> Option<usize> {
    let rest = label.strip_prefix(prefix)?;
    if rest.is_empty() {
        return Some(256);
    }
    rest.parse::<usize>()
        .ok()
        .filter(|b| *b > 0 && *b <= 256 && b % 8 == 0)
}

/// Split `name[a][b].c` into the name and its steps (`a`, `b`, `c`)
pub fn split_path(path: &str) -> Result<(String, Vec<String>)> {
    let mut name = String::new();
    let mut steps = Vec::new();
    let mut current = String::new();
    let mut chars = path.chars();
    let mut first = true;
    let flush =
        |current: &mut String, name: &mut String, steps: &mut Vec<String>, first: &mut bool| {
            if *first {
                *name = std::mem::take(current);
                *first = false;
            } else if !current.is_empty() {
                steps.push(std::mem::take(current));
            }
        };
    while let Some(c) = chars.next() {
        match c {
            '.' => flush(&mut current, &mut name, &mut steps, &mut first),
            '[' => {
                flush(&mut current, &mut name, &mut steps, &mut first);
                let mut inner = String::new();
                loop {
                    match chars.next() {
                        Some(']') => break,
                        Some(c) => inner.push(c),
                        None => bail!("unclosed '[' in '{path}'"),
                    }
                }
                steps.push(inner.trim().trim_matches('"').to_string());
            }
            _ => current.push(c),
        }
    }
    flush(&mut current, &mut name, &mut steps, &mut first);
    if name.is_empty() {
        bail!("no variable name in '{path}'");
    }
    Ok((name, steps))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Reader over a fixed map; missing slots read as zero
    struct MapReader(Mutex<HashMap<U256, U256>>);

    impl SlotReader for MapReader {
        fn read(&self, slot: U256) -> Pin<Box<dyn Future<Output = Result<U256>> + Send + '_>> {
            let v = self
                .0
                .lock()
                .unwrap()
                .get(&slot)
                .copied()
                .unwrap_or_default();
            Box::pin(async move { Ok(v) })
        }
    }

    fn reader(entries: &[(U256, U256)]) -> MapReader {
        MapReader(Mutex::new(entries.iter().copied().collect()))
    }

    fn hex_u256(s: &str) -> U256 {
        U256::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()
    }

    /// Part of USDC's FiatTokenV2_2 layout as Sourcify serves it, plus a
    /// struct, arrays and strings for the other cases
    fn layout() -> Layout {
        Layout::parse(&json!({
            "storage": [
                {"label": "_owner", "slot": "0", "offset": 0, "type": "t_address", "contract": "a/FiatToken.sol:FiatTokenV2_2"},
                {"label": "pauser", "slot": "1", "offset": 0, "type": "t_address"},
                {"label": "paused", "slot": "1", "offset": 20, "type": "t_bool"},
                {"label": "name", "slot": "4", "offset": 0, "type": "t_string_storage"},
                {"label": "balanceAndBlacklistStates", "slot": "9", "offset": 0, "type": "t_mapping(t_address,t_uint256)"},
                {"label": "allowed", "slot": "10", "offset": 0, "type": "t_mapping(t_address,t_mapping(t_address,t_uint256))"},
                {"label": "small", "slot": "20", "offset": 0, "type": "t_array(t_uint64)dyn_storage"},
                {"label": "fixed", "slot": "21", "offset": 0, "type": "t_array(t_uint128)3_storage"},
                {"label": "info", "slot": "23", "offset": 0, "type": "t_struct(Info)"},
                {"label": "infos", "slot": "26", "offset": 0, "type": "t_mapping(t_uint256,t_struct(Info))"},
                {"label": "byName", "slot": "27", "offset": 0, "type": "t_mapping(t_string_memory_ptr,t_uint256)"},
                {"label": "delta", "slot": "28", "offset": 0, "type": "t_int24"}
            ],
            "types": {
                "t_address": {"label": "address", "encoding": "inplace", "numberOfBytes": "20"},
                "t_bool": {"label": "bool", "encoding": "inplace", "numberOfBytes": "1"},
                "t_uint64": {"label": "uint64", "encoding": "inplace", "numberOfBytes": "8"},
                "t_uint128": {"label": "uint128", "encoding": "inplace", "numberOfBytes": "16"},
                "t_uint256": {"label": "uint256", "encoding": "inplace", "numberOfBytes": "32"},
                "t_int24": {"label": "int24", "encoding": "inplace", "numberOfBytes": "3"},
                "t_string_storage": {"label": "string", "encoding": "bytes", "numberOfBytes": "32"},
                "t_string_memory_ptr": {"label": "string", "encoding": "bytes", "numberOfBytes": "32"},
                "t_mapping(t_address,t_uint256)": {"label": "mapping(address => uint256)", "encoding": "mapping", "key": "t_address", "value": "t_uint256", "numberOfBytes": "32"},
                "t_mapping(t_address,t_mapping(t_address,t_uint256))": {"label": "mapping(address => mapping(address => uint256))", "encoding": "mapping", "key": "t_address", "value": "t_mapping(t_address,t_uint256)", "numberOfBytes": "32"},
                "t_mapping(t_uint256,t_struct(Info))": {"label": "mapping(uint256 => struct Info)", "encoding": "mapping", "key": "t_uint256", "value": "t_struct(Info)", "numberOfBytes": "32"},
                "t_mapping(t_string_memory_ptr,t_uint256)": {"label": "mapping(string => uint256)", "encoding": "mapping", "key": "t_string_memory_ptr", "value": "t_uint256", "numberOfBytes": "32"},
                "t_array(t_uint64)dyn_storage": {"label": "uint64[]", "encoding": "dynamic_array", "base": "t_uint64", "numberOfBytes": "32"},
                "t_array(t_uint128)3_storage": {"label": "uint128[3]", "encoding": "inplace", "base": "t_uint128", "numberOfBytes": "64"},
                "t_struct(Info)": {"label": "struct Info", "encoding": "inplace", "numberOfBytes": "96", "members": [
                    {"label": "owner", "slot": "0", "offset": 0, "type": "t_address"},
                    {"label": "active", "slot": "0", "offset": 20, "type": "t_bool"},
                    {"label": "amount", "slot": "1", "offset": 0, "type": "t_uint256"},
                    {"label": "tag", "slot": "2", "offset": 0, "type": "t_string_storage"}
                ]}
            }
        }))
        .unwrap()
    }

    const HOLDER: &str = "0x47ac0Fb4F2D84898e4D9E7b4DaB3C24507a6D503";

    #[test]
    fn mapping_slot_matches_known_usdc_balance_slot() {
        // keccak256(abi.encode(holder, 9)), as `cast index address <holder> 9`
        let loc = layout()
            .locate("balanceAndBlacklistStates", &[HOLDER.to_string()])
            .unwrap();
        let mut pre = [0u8; 64];
        pre[12..32].copy_from_slice(HOLDER.parse::<Address>().unwrap().as_slice());
        pre[63] = 9;
        assert_eq!(loc.slot, U256::from_be_bytes(keccak256(pre).0));
        assert_eq!(loc.type_id, "t_uint256");
    }

    #[test]
    fn nested_mapping_hashes_twice() {
        let spender = "0x0000000000000000000000000000000000000001";
        let loc = layout()
            .locate("allowed", &[HOLDER.into(), spender.into()])
            .unwrap();
        let inner = layout().locate("allowed", &[HOLDER.into()]).unwrap().slot;
        let mut pre = encode_key("address", spender).unwrap();
        pre.extend_from_slice(&inner.to_be_bytes::<32>());
        assert_eq!(loc.slot, U256::from_be_bytes(keccak256(&pre).0));
    }

    #[test]
    fn known_mapping_slot_value() {
        // Well-known: balances mapping at slot 0, key = address(1)
        let l = Layout::parse(&json!({
            "storage": [{"label": "m", "slot": "0", "offset": 0, "type": "t_mapping(t_address,t_uint256)"}],
            "types": {
                "t_address": {"label": "address", "encoding": "inplace", "numberOfBytes": "20"},
                "t_uint256": {"label": "uint256", "encoding": "inplace", "numberOfBytes": "32"},
                "t_mapping(t_address,t_uint256)": {"label": "mapping(address => uint256)", "encoding": "mapping", "key": "t_address", "value": "t_uint256", "numberOfBytes": "32"}
            }
        }))
        .unwrap();
        let loc = l
            .locate("m", &["0x0000000000000000000000000000000000000001".into()])
            .unwrap();
        assert_eq!(
            loc.slot,
            hex_u256("0xada5013122d395ba3c54772283fb069b10426056ef8ca54750cb9bb552a59e7d")
        );
    }

    #[test]
    fn string_and_int_keys() {
        let loc = layout().locate("byName", &["abc".into()]).unwrap();
        let mut pre = b"abc".to_vec();
        pre.extend_from_slice(&U256::from(27).to_be_bytes::<32>());
        assert_eq!(loc.slot, U256::from_be_bytes(keccak256(&pre).0));
        assert_eq!(encode_key("int24", "-1").unwrap(), vec![0xff; 32]);
        let b4 = encode_key("bytes4", "0xa9059cbb").unwrap();
        assert_eq!(&b4[..4], &[0xa9, 0x05, 0x9c, 0xbb]);
        assert!(b4[4..].iter().all(|b| *b == 0));
    }

    #[test]
    fn packed_value_and_arrays() {
        let l = layout();
        let paused = l.locate("paused", &[]).unwrap();
        assert_eq!((paused.slot, paused.offset), (U256::from(1), 20));

        // dynamic uint64[]: four per slot from keccak(20)
        let base = U256::from_be_bytes(keccak256(U256::from(20).to_be_bytes::<32>()).0);
        let e5 = l.locate("small", &["5".into()]).unwrap();
        assert_eq!((e5.slot, e5.offset), (base + U256::from(1), 8));

        // uint128[3] at 21: two per slot, out of bounds at 3
        let f2 = l.locate("fixed", &["2".into()]).unwrap();
        assert_eq!((f2.slot, f2.offset), (U256::from(22), 0));
        assert!(l.locate("fixed", &["3".into()]).is_err());
    }

    #[test]
    fn struct_member_in_mapping() {
        let l = layout();
        let entry = l.locate("infos", &["7".into()]).unwrap();
        let amount = l.locate("infos", &["7".into(), "amount".into()]).unwrap();
        assert_eq!(amount.slot, entry.slot + U256::from(1));
        let active = l.locate("info", &["active".into()]).unwrap();
        assert_eq!((active.slot, active.offset), (U256::from(23), 20));
        let err = l.locate("info", &["nope".into()]).unwrap_err().to_string();
        assert!(err.contains("Members: owner, active, amount, tag"), "{err}");
        let err = l.locate("nope", &[]).unwrap_err().to_string();
        assert!(err.contains("Variables: _owner"), "{err}");
    }

    #[tokio::test]
    async fn decodes_packed_values_and_structs() {
        let l = layout();
        let owner = hex_u256("0x47ac0Fb4F2D84898e4D9E7b4DaB3C24507a6D503");
        // slot 1: pauser address in the low 20 bytes, paused=true at offset 20
        let slot1 = owner | (U256::from(1) << 160);
        // short string "abc": data left-aligned, length*2 in the low byte
        let mut short = [0u8; 32];
        short[..3].copy_from_slice(b"abc");
        short[31] = 6;
        // int24 -2 packed at offset 0
        let r = reader(&[
            (U256::from(1), slot1),
            (U256::from(23), slot1),
            (U256::from(24), U256::from(42)),
            (U256::from(25), U256::from_be_bytes(short)),
            (U256::from(28), U256::from(0xfffffe)),
        ]);
        let get = |name: &str, steps: &[&str]| {
            let steps: Vec<String> = steps.iter().map(|s| s.to_string()).collect();
            l.locate(name, &steps).unwrap()
        };
        assert_eq!(
            l.decode(&r, &get("paused", &[])).await.unwrap(),
            json!(true)
        );
        assert_eq!(
            l.decode(&r, &get("pauser", &[])).await.unwrap(),
            json!(HOLDER)
        );
        assert_eq!(l.decode(&r, &get("delta", &[])).await.unwrap(), json!("-2"));
        assert_eq!(
            l.decode(&r, &get("info", &[])).await.unwrap(),
            json!({"owner": HOLDER, "active": true, "amount": "42", "tag": "abc"})
        );
        assert!(l.decode(&r, &get("infos", &[])).await.is_err());
    }

    #[tokio::test]
    async fn decodes_long_string_and_dynamic_array() {
        let l = layout();
        let text = "a string that is longer than thirty-one bytes, so it spills";
        let data_start = U256::from_be_bytes(keccak256(U256::from(4).to_be_bytes::<32>()).0);
        let mut entries = vec![(U256::from(4), U256::from(text.len() * 2 + 1))];
        for (i, chunk) in text.as_bytes().chunks(32).enumerate() {
            let mut w = [0u8; 32];
            w[..chunk.len()].copy_from_slice(chunk);
            entries.push((data_start + U256::from(i), U256::from_be_bytes(w)));
        }
        // uint64[] of length 5: [1,2,3,4] in the first slot, [5] in the next
        let arr = U256::from_be_bytes(keccak256(U256::from(20).to_be_bytes::<32>()).0);
        let packed =
            U256::from(1) | (U256::from(2) << 64) | (U256::from(3) << 128) | (U256::from(4) << 192);
        entries.extend([
            (U256::from(20), U256::from(5)),
            (arr, packed),
            (arr + U256::from(1), U256::from(5)),
        ]);
        let r = reader(&entries);
        assert_eq!(
            l.decode(&r, &l.locate("name", &[]).unwrap()).await.unwrap(),
            json!(text)
        );
        assert_eq!(
            l.decode(&r, &l.locate("small", &[]).unwrap())
                .await
                .unwrap(),
            json!({"length": "5", "elements": ["1", "2", "3", "4", "5"]})
        );
        assert_eq!(
            l.decode(&r, &l.locate("small", &["3".into()]).unwrap())
                .await
                .unwrap(),
            json!("4")
        );
    }

    #[test]
    fn splits_paths() {
        assert_eq!(
            split_path("allowed[0xabc][0xdef]").unwrap(),
            (
                "allowed".to_string(),
                vec!["0xabc".to_string(), "0xdef".to_string()]
            )
        );
        assert_eq!(
            split_path("positions[3].liquidity").unwrap(),
            (
                "positions".to_string(),
                vec!["3".to_string(), "liquidity".to_string()]
            )
        );
        assert_eq!(
            split_path("slot0.tick").unwrap(),
            ("slot0".to_string(), vec!["tick".to_string()])
        );
        assert!(split_path("a[1").is_err());
    }

    #[test]
    fn keys_are_range_checked_and_hex_ints_sign_extended() {
        // int24 -887270 is 0xf2761a in the type's own 24-bit two's complement
        let dec = encode_key("int24", "-887270").unwrap();
        let hex = encode_key("int24", "0xf2761a").unwrap();
        assert_eq!(dec, hex);
        assert!(encode_key("int24", "0xfff2764a").is_err()); // wider than 24 bits
        assert!(encode_key("int24", "-8388609").is_err()); // below int24 min
        assert!(encode_key("int24", "8388608").is_err()); // above int24 max
        assert!(encode_key("int24", "-8388608").is_ok());
        assert!(encode_key("uint8", "256").is_err());
        assert!(encode_key("uint8", "255").is_ok());
        assert!(encode_key("uint256", "0x01").is_ok());
        assert!(encode_key("bytes32", "0x01").is_err()); // must be exactly 32 bytes
        assert!(encode_key("enum E", "256").is_err());
    }

    fn dup_layout() -> Layout {
        Layout::parse(&json!({
            "storage": [
                {"label": "__gap", "slot": "2", "offset": 0, "type": "t_array(t_uint256)49_storage", "contract": "a/A.sol:Initializable"},
                {"label": "x", "slot": "51", "offset": 0, "type": "t_uint256", "contract": "b/B.sol:Pod"},
                {"label": "__gap", "slot": "63", "offset": 0, "type": "t_array(t_uint256)35_storage", "contract": "b/B.sol:Pod"}
            ],
            "types": {
                "t_uint256": {"label": "uint256", "encoding": "inplace", "numberOfBytes": "32"},
                "t_array(t_uint256)49_storage": {"label": "uint256[49]", "encoding": "inplace", "base": "t_uint256", "numberOfBytes": "1568"},
                "t_array(t_uint256)35_storage": {"label": "uint256[35]", "encoding": "inplace", "base": "t_uint256", "numberOfBytes": "1120"}
            }
        }))
        .unwrap()
    }

    #[test]
    fn duplicate_labels_need_disambiguation() {
        let l = dup_layout();
        let err = l.locate("__gap", &["1".into()]).unwrap_err().to_string();
        assert!(
            err.contains("ambiguous") && err.contains("__gap@2") && err.contains("__gap@63"),
            "{err}"
        );
        assert_eq!(
            l.locate("__gap@63", &["1".into()]).unwrap().slot,
            U256::from(64)
        );
        assert_eq!(
            l.locate("Pod", &["__gap".into(), "1".into()]).unwrap().slot,
            U256::from(64)
        );
        let (name, steps) = split_path("Initializable.__gap[0]").unwrap();
        assert_eq!(l.locate(&name, &steps).unwrap().slot, U256::from(2));
        assert_eq!(l.locate("x", &[]).unwrap().slot, U256::from(51));
    }

    #[tokio::test]
    async fn dynamic_array_index_is_bounds_checked() {
        let l = layout();
        let r = reader(&[(U256::from(20), U256::from(5))]);
        let ok = l.locate("small", &["4".into()]).unwrap();
        l.check_bounds(&r, &ok).await.unwrap();
        let bad = l.locate("small", &["5".into()]).unwrap();
        let err = l.check_bounds(&r, &bad).await.unwrap_err().to_string();
        assert!(
            err.contains("out of bounds") && err.contains("length 5"),
            "{err}"
        );
    }
}
