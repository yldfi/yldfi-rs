//! Compare a compiled artifact with deployed runtime code.
//!
//! The artifact is a forge `out/<File>.sol/<Name>.json` or a solc
//! standard-JSON output. Immutable references and library link placeholders
//! are masked (their deployed values are reported), compiler metadata is
//! compared separately, and a mismatch is located down to a function.
//!
//! Verdicts follow Sourcify's terms: `exact` (identical, metadata included)
//! and `partial` (identical once metadata is ignored). A partial match is not
//! a verification: the metadata hash commits to the exact sources and
//! settings, and those differ.

use super::metadata::{find_metadata_blocks, linear_sweep, push_size, MetadataBlock};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::ops::Range;

/// The parts of a compiled artifact `contract match` uses
#[derive(Debug, Clone, Default)]
pub struct Artifact {
    /// `forge` or `solc-standard-json`
    pub format: String,
    pub contract_name: Option<String>,
    pub source_path: Option<String>,
    pub compiler_version: Option<String>,
    pub runtime: Vec<u8>,
    pub creation: Option<Vec<u8>>,
    /// id -> byte ranges in the runtime
    pub immutables: BTreeMap<String, Vec<Range<usize>>>,
    /// library -> byte ranges in the runtime
    pub libraries: BTreeMap<String, Vec<Range<usize>>>,
    /// library -> byte ranges in the creation code
    pub creation_libraries: BTreeMap<String, Vec<Range<usize>>>,
    pub source_map: Option<String>,
    /// AST nodes of the sources, with their source index when known
    pub asts: Vec<(Option<u64>, Value)>,
    /// selector (hex, no 0x) -> signature
    pub methods: BTreeMap<String, String>,
    /// Number of user sources, when known (from standard JSON or build-info)
    pub source_count: Option<usize>,
    /// The deployed bytecode object as given (lowercase hex, no 0x), to tie
    /// a build-info to this exact build
    pub deployed_object: String,
}

/// Largest offset an artifact range may name; deployed code is far smaller
const MAX_RANGE_END: u64 = 1 << 20;

/// A `start`/`length` pair from an artifact, checked for overflow
fn checked_range(p: &Value, what: &str) -> anyhow::Result<Option<Range<usize>>> {
    let (Some(start), Some(len)) = (p["start"].as_u64(), p["length"].as_u64()) else {
        return Ok(None);
    };
    let end = start
        .checked_add(len)
        .filter(|e| *e <= MAX_RANGE_END && len <= 64)
        .ok_or_else(|| anyhow::anyhow!("{what} has an out-of-range start/length"))?;
    Ok(Some(start as usize..end as usize))
}

/// Every range must lie inside code of `len` bytes
fn check_within(
    ranges: &BTreeMap<String, Vec<Range<usize>>>,
    len: usize,
    what: &str,
) -> anyhow::Result<()> {
    if ranges.values().flatten().any(|r| r.end > len) {
        anyhow::bail!("{what} points past the end of the bytecode");
    }
    Ok(())
}

fn link_ranges(refs: Option<&Value>) -> anyhow::Result<BTreeMap<String, Vec<Range<usize>>>> {
    let mut out: BTreeMap<String, Vec<Range<usize>>> = BTreeMap::new();
    let Some(Value::Object(files)) = refs else {
        return Ok(out);
    };
    for (file, libs) in files {
        let Value::Object(libs) = libs else { continue };
        for (lib, positions) in libs {
            for p in positions.as_array().into_iter().flatten() {
                if let Some(r) = checked_range(p, "linkReferences")? {
                    out.entry(format!("{file}:{lib}")).or_default().push(r);
                }
            }
        }
    }
    Ok(out)
}

fn immutable_ranges(refs: Option<&Value>) -> anyhow::Result<BTreeMap<String, Vec<Range<usize>>>> {
    let mut out: BTreeMap<String, Vec<Range<usize>>> = BTreeMap::new();
    let Some(Value::Object(ids)) = refs else {
        return Ok(out);
    };
    for (id, positions) in ids {
        for p in positions.as_array().into_iter().flatten() {
            if let Some(r) = checked_range(p, "immutableReferences")? {
                out.entry(id.clone()).or_default().push(r);
            }
        }
    }
    Ok(out)
}

/// Decode bytecode hex, writing zeros over library placeholders first
fn decode_object(
    object: &str,
    links: &BTreeMap<String, Vec<Range<usize>>>,
) -> anyhow::Result<Vec<u8>> {
    let mut hex_text: Vec<u8> = object.trim().trim_start_matches("0x").as_bytes().to_vec();
    for range in links.values().flatten() {
        let (a, b) = (range.start * 2, range.end * 2);
        if b > hex_text.len() || a > b {
            anyhow::bail!("linkReferences points past the end of the bytecode");
        }
        hex_text[a..b].fill(b'0');
    }
    if let Some(pos) = hex_text.windows(2).position(|w| w == b"__") {
        anyhow::bail!(
            "bytecode has an unlinked library placeholder at hex offset {pos} that linkReferences does not list"
        );
    }
    hex::decode(&hex_text).map_err(|e| anyhow::anyhow!("bytecode is not valid hex: {e}"))
}

fn bytecode_object(value: &Value) -> Option<&str> {
    match value {
        Value::String(s) => Some(s),
        Value::Object(_) => value["object"].as_str(),
        _ => None,
    }
}

fn methods_from(value: Option<&Value>) -> BTreeMap<String, String> {
    value
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .filter_map(|(sig, sel)| Some((sel.as_str()?.to_lowercase(), sig.clone())))
                .collect()
        })
        .unwrap_or_default()
}

/// Read a forge artifact or a solc standard-JSON output
pub fn parse_artifact(json: &Value, contract: Option<&str>) -> anyhow::Result<Artifact> {
    // forge: deployedBytecode at the top level
    if let Some(deployed) = json.get("deployedBytecode") {
        let object = bytecode_object(deployed)
            .ok_or_else(|| anyhow::anyhow!("deployedBytecode.object missing"))?;
        let libraries = link_ranges(deployed.get("linkReferences"))?;
        let runtime = decode_object(object, &libraries)?;
        let creation_libraries = link_ranges(json["bytecode"].get("linkReferences"))?;
        let creation = bytecode_object(&json["bytecode"])
            .map(|o| decode_object(o, &creation_libraries))
            .transpose()?
            .filter(|c| !c.is_empty());
        let target = json["metadata"]["settings"]["compilationTarget"].as_object();
        let (source_path, name) = target
            .and_then(|t| t.iter().next())
            .map(|(p, n)| (Some(p.clone()), n.as_str().map(String::from)))
            .unwrap_or((None, None));
        if let (Some(want), Some(have)) = (contract, name.as_deref()) {
            if want != have {
                anyhow::bail!("artifact is for contract {have}, not {want}");
            }
        }
        let mut asts = Vec::new();
        if let Some(ast) = json.get("ast").filter(|a| a.is_object()) {
            asts.push((json["id"].as_u64(), ast.clone()));
        }
        let immutables = immutable_ranges(deployed.get("immutableReferences"))?;
        check_within(&immutables, runtime.len(), "immutableReferences")?;
        return Ok(Artifact {
            deployed_object: object.trim().trim_start_matches("0x").to_lowercase(),
            source_count: None,
            format: "forge".into(),
            contract_name: name.or_else(|| contract.map(String::from)),
            source_path,
            compiler_version: json["metadata"]["compiler"]["version"]
                .as_str()
                .map(String::from),
            runtime,
            creation,
            immutables,
            libraries,
            creation_libraries,
            source_map: deployed["sourceMap"].as_str().map(String::from),
            asts,
            methods: methods_from(json.get("methodIdentifiers")),
        });
    }

    // forge build-info wraps the standard-JSON output in `output`
    if json.get("contracts").is_none() && json["output"].get("contracts").is_some() {
        return parse_artifact(&json["output"], contract);
    }
    // solc standard JSON: contracts[file][name].evm
    let contracts = json
        .get("contracts")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "not a forge artifact (no deployedBytecode) or solc standard-JSON output (no contracts)"
            )
        })?;
    let mut found = Vec::new();
    for (file, by_name) in contracts {
        for (name, entry) in by_name.as_object().into_iter().flatten() {
            let has_code =
                bytecode_object(&entry["evm"]["deployedBytecode"]).is_some_and(|o| !o.is_empty());
            if has_code && contract.is_none_or(|c| c == name || c == format!("{file}:{name}")) {
                found.push((file.clone(), name.clone(), entry));
            }
        }
    }
    let (file, name, entry) = match found.len() {
        0 => anyhow::bail!(
            "no contract with deployed code{} in the standard-JSON output",
            contract.map(|c| format!(" named {c}")).unwrap_or_default()
        ),
        1 => found.remove(0),
        _ => anyhow::bail!(
            "several contracts in the output; pick one with --contract: {}",
            found
                .iter()
                .map(|(f, n, _)| format!("{f}:{n}"))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    };
    let evm = &entry["evm"];
    let deployed = &evm["deployedBytecode"];
    let libraries = link_ranges(deployed.get("linkReferences"))?;
    let creation_libraries = link_ranges(evm["bytecode"].get("linkReferences"))?;
    let metadata: Option<Value> = entry["metadata"]
        .as_str()
        .and_then(|m| serde_json::from_str(m).ok());
    let (asts, source_count) = asts_from_output(json);
    let runtime = decode_object(bytecode_object(deployed).unwrap_or_default(), &libraries)?;
    let immutables = immutable_ranges(deployed.get("immutableReferences"))?;
    check_within(&immutables, runtime.len(), "immutableReferences")?;
    Ok(Artifact {
        deployed_object: bytecode_object(deployed)
            .unwrap_or_default()
            .trim()
            .trim_start_matches("0x")
            .to_lowercase(),
        source_count,
        format: "solc-standard-json".into(),
        contract_name: Some(name),
        source_path: Some(file),
        compiler_version: metadata
            .as_ref()
            .and_then(|m| m["compiler"]["version"].as_str().map(String::from)),
        runtime,
        creation: bytecode_object(&evm["bytecode"])
            .map(|o| decode_object(o, &creation_libraries))
            .transpose()?
            .filter(|c| !c.is_empty()),
        immutables,
        libraries,
        creation_libraries,
        source_map: deployed["sourceMap"].as_str().map(String::from),
        asts,
        methods: methods_from(evm.get("methodIdentifiers")),
    })
}

/// All source ASTs of a standard-JSON output (or forge build-info), and the
/// number of user sources; source-map indexes at or above it are
/// compiler-generated
fn asts_from_output(json: &Value) -> (Vec<(Option<u64>, Value)>, Option<usize>) {
    let json = if json.get("sources").is_none() && json["output"].get("sources").is_some() {
        &json["output"]
    } else {
        json
    };
    let sources = json["sources"].as_object();
    let asts = sources
        .into_iter()
        .flatten()
        .filter_map(|(_, s)| s.get("ast").map(|a| (s["id"].as_u64(), a.clone())))
        .collect();
    (asts, sources.map(|s| s.len()))
}

impl Artifact {
    /// Replace the ASTs with every source's AST from a forge build-info or
    /// standard-JSON output, so code from base contracts and other files can
    /// be named
    pub fn attach_build_info(&mut self, json: &Value) -> anyhow::Result<()> {
        let (asts, count) = asts_from_output(json);
        if asts.is_empty() {
            anyhow::bail!("the build-info has no source ASTs");
        }
        self.asts = asts;
        self.source_count = count;
        Ok(())
    }

    /// True when the build-info lists this artifact's contract
    pub fn build_info_matches(&self, json: &Value) -> bool {
        let out = if json.get("contracts").is_some() {
            json
        } else {
            &json["output"]
        };
        // The same path and name is not enough: an older build-info of the
        // same contract has other source ids. Require the identical
        // deployed bytecode
        match (&self.source_path, &self.contract_name) {
            (Some(p), Some(n)) => {
                bytecode_object(&out["contracts"][p][n]["evm"]["deployedBytecode"]).is_some_and(
                    |o| o.trim().trim_start_matches("0x").to_lowercase() == self.deployed_object,
                )
            }
            _ => false,
        }
    }
}

/// A function's source range, from the AST
#[derive(Debug, Clone)]
struct FnSpan {
    name: String,
    start: u64,
    end: u64,
    file: u64,
}

fn parse_src(src: &str) -> Option<(u64, u64, u64)> {
    let mut parts = src.split(':');
    let s = parts.next()?.parse().ok()?;
    let l = parts.next()?.parse().ok()?;
    let f = parts.next()?.parse().ok()?;
    Some((s, l, f))
}

fn collect_functions(
    node: &Value,
    contract: Option<&str>,
    out: &mut Vec<FnSpan>,
    asm: &mut Vec<(u64, u64, u64)>,
) {
    match node {
        Value::Object(map) => {
            let kind = map.get("nodeType").and_then(Value::as_str).unwrap_or("");
            let contract = if kind == "ContractDefinition" {
                map.get("name").and_then(Value::as_str)
            } else {
                contract
            };
            if matches!(kind, "FunctionDefinition" | "ModifierDefinition") {
                if let Some((s, l, f)) = map.get("src").and_then(Value::as_str).and_then(parse_src)
                {
                    let raw = map.get("name").and_then(Value::as_str).unwrap_or("");
                    let fn_kind = map
                        .get("kind")
                        .and_then(Value::as_str)
                        .unwrap_or("function");
                    let label = if raw.is_empty() {
                        fn_kind.to_string()
                    } else {
                        raw.to_string()
                    };
                    let name = match (kind, contract) {
                        ("ModifierDefinition", Some(c)) => format!("{c}.modifier {label}"),
                        (_, Some(c)) => format!("{c}.{label}"),
                        _ => label,
                    };
                    out.push(FnSpan {
                        name,
                        start: s,
                        end: s + l,
                        file: f,
                    });
                }
            }
            if kind == "InlineAssembly" {
                if let Some(span) = map.get("src").and_then(Value::as_str).and_then(parse_src) {
                    asm.push(span);
                }
            }
            for v in map.values() {
                collect_functions(v, contract, out, asm);
            }
        }
        Value::Array(items) => {
            for v in items {
                collect_functions(v, contract, out, asm);
            }
        }
        _ => {}
    }
}

/// Decode a compressed source map into one (start, length, file) per instruction
fn decode_source_map(map: &str) -> Vec<Option<(u64, u64, i64)>> {
    let mut out = Vec::new();
    let (mut s, mut l, mut f) = (0i64, 0i64, -1i64);
    for entry in map.split(';') {
        let fields: Vec<&str> = entry.split(':').collect();
        let pick = |i: usize, cur: i64| {
            fields
                .get(i)
                .filter(|v| !v.is_empty())
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(cur)
        };
        s = pick(0, s);
        l = pick(1, l);
        f = pick(2, f);
        out.push((s >= 0 && l >= 0).then_some((s as u64, l as u64, f)));
    }
    out
}

/// How pcs are attributed to functions
pub struct FunctionMapper {
    /// "source map + AST", "dispatcher ranges (approximate)" or "none"
    pub method: String,
    /// Every source-map file index resolved to an AST or to generated code
    pub complete: bool,
    /// artifact instruction pc -> function label
    labels: BTreeMap<usize, String>,
}

impl FunctionMapper {
    pub fn label(&self, pc: usize) -> String {
        self.labels
            .range(..=pc)
            .next_back()
            .map(|(_, l)| l.clone())
            .unwrap_or_else(|| "(unmapped)".into())
    }
}

/// Build the pc -> function mapping for the artifact's runtime
pub fn function_mapper(artifact: &Artifact, exec_end: usize) -> FunctionMapper {
    let instructions: Vec<(usize, u8)> = linear_sweep(&artifact.runtime[..exec_end]);
    if let (Some(map), false) = (&artifact.source_map, artifact.asts.is_empty()) {
        let mut spans = Vec::new();
        let mut asm = Vec::new();
        for (_, ast) in &artifact.asts {
            collect_functions(ast, None, &mut spans, &mut asm);
        }
        let entries = decode_source_map(map);
        let covered: std::collections::BTreeSet<u64> =
            artifact.asts.iter().filter_map(|(id, _)| *id).collect();
        let mut complete = true;
        let mut labels = BTreeMap::new();
        for (i, (pc, _)) in instructions.iter().enumerate() {
            let label = match entries.get(i).copied().flatten() {
                Some((s, l, f)) if f >= 0 && covered.contains(&(f as u64)) => spans
                    .iter()
                    .filter(|sp| sp.file == f as u64 && sp.start <= s && s + l <= sp.end)
                    .min_by_key(|sp| sp.end - sp.start)
                    .map(|sp| sp.name.clone())
                    .unwrap_or_else(|| "(contract-level code)".into()),
                Some((_, _, f)) if f >= 0 => match artifact.source_count {
                    Some(n) if f as usize >= n => "(compiler-generated)".into(),
                    _ => {
                        complete = false;
                        format!("(source {f}: no AST; pass --build-info to name it)")
                    }
                },
                _ => "(compiler-generated)".into(),
            };
            labels.insert(*pc, label);
        }
        return FunctionMapper {
            method: if complete {
                "source map + AST".into()
            } else {
                "source map + AST (partial: some sources have no AST)".into()
            },
            complete,
            labels,
        };
    }
    // Dispatcher: each selector's handler entry; a pc belongs to the nearest
    // handler before it. Shared internal code lands on whichever is nearest.
    let functions = super::extract_selectors(&artifact.runtime);
    let mut labels = BTreeMap::new();
    labels.insert(0, "(dispatcher)".to_string());
    if let Ok(entries) = super::infer_dispatcher(&artifact.runtime, &functions) {
        for e in entries {
            let sel = e.selector.trim_start_matches("0x").to_lowercase();
            let name = artifact
                .methods
                .get(&sel)
                .cloned()
                .unwrap_or_else(|| format!("0x{sel}"));
            labels.insert(e.handler_offset, name);
        }
    }
    FunctionMapper {
        complete: false,
        method: if labels.len() > 1 {
            "dispatcher ranges (approximate: code after a handler is counted to it, shared internal code included)".into()
        } else {
            "none".into()
        },
        labels,
    }
}

/// Values found on chain at masked positions
#[derive(Debug, Clone, Serialize)]
pub struct MaskedValue {
    pub id: String,
    pub positions: usize,
    /// Distinct deployed values at those positions (hex)
    pub values: Vec<String>,
    /// All positions carry the same value, as they must
    pub consistent: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDiff {
    pub function: String,
    pub instructions: usize,
    pub status: &'static str,
    /// Aligned artifact/deployed instructions (`--diff`, functions marked
    /// different only): the changes with a few equal rows around them
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<Vec<DiffRow>>,
    /// Rows were cut at the per-function cap
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub diff_truncated: bool,
}

/// One row of an aligned instruction diff
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct DiffRow {
    /// equal | changed | inserted (deployed only) | deleted (artifact only)
    /// | skip (`skipped` equal rows left out)
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_pc: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_pc: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<usize>,
}

/// Equal rows kept on each side of a change
const DIFF_CONTEXT: usize = 3;
/// Rows kept per function
pub const DIFF_MAX_ROWS: usize = 200;

fn instruction_text(code: &[u8], pc: usize) -> String {
    let op = code[pc];
    let n = push_size(op);
    let name = super::mnemonic(op);
    if n == 0 {
        name
    } else {
        let end = (pc + 1 + n).min(code.len());
        format!("{name} 0x{}", hex::encode(&code[(pc + 1).min(end)..end]))
    }
}

fn diff_row(kind: &'static str, a: Option<(&[u8], usize)>, c: Option<(&[u8], usize)>) -> DiffRow {
    DiffRow {
        kind,
        artifact_pc: a.map(|(_, pc)| pc),
        artifact: a.map(|(code, pc)| instruction_text(code, pc)),
        onchain_pc: c.map(|(_, pc)| pc),
        onchain: c.map(|(code, pc)| instruction_text(code, pc)),
        skipped: None,
    }
}

/// Align the executable code of artifact (`a`) and deployment (`c`) and cut
/// the rows into per-function diffs. Labels come from `mapper`; a
/// deployed-only row takes the label of the artifact row before it. Masked
/// artifact bytes (immutables, libraries, embedded metadata) match any
/// deployed value; when the sizes differ PUSH2/PUSH3 operands (mostly code
/// offsets, which shift) are ignored. Returns label -> (rows, truncated)
/// and whether the alignment was approximate somewhere.
pub fn function_diffs(
    a: &[u8],
    c: &[u8],
    mask: &[bool],
    mapper: &FunctionMapper,
) -> (BTreeMap<String, (Vec<DiffRow>, bool)>, bool) {
    use super::opdiff::{align, Step};
    let ia = linear_sweep(a);
    let ic = linear_sweep(c);
    let sizes_differ = a.len() != c.len();
    let operand = |code: &[u8], pc: usize, op: u8| -> Range<usize> {
        (pc + 1).min(code.len())..(pc + 1 + push_size(op)).min(code.len())
    };
    let is_masked = |k: usize| mask.get(k).copied().unwrap_or(false);
    let ignore_operand = |op: u8| sizes_differ && matches!(push_size(op), 2 | 3);
    let eq = |i: usize, j: usize| {
        let ((pa, oa), (pb, ob)) = (ia[i], ic[j]);
        if oa != ob {
            return false;
        }
        if ignore_operand(oa) {
            return true;
        }
        let (ra, rb) = (operand(a, pa, oa), operand(c, pb, ob));
        ra.len() == rb.len() && ra.zip(rb).all(|(x, y)| a[x] == c[y] || is_masked(x))
    };
    let key_l = |i: usize| {
        let (pc, op) = ia[i];
        if (pc..pc + 1 + push_size(op)).any(is_masked) {
            return None;
        }
        let mut k = vec![op];
        if !ignore_operand(op) {
            k.extend_from_slice(&a[operand(a, pc, op)]);
        }
        Some(k)
    };
    let key_r = |j: usize| {
        let (pc, op) = ic[j];
        let mut k = vec![op];
        if !ignore_operand(op) {
            k.extend_from_slice(&c[operand(c, pc, op)]);
        }
        Some(k)
    };
    let (steps, approximate) = align(ia.len(), ic.len(), &eq, &key_l, &key_r);

    // Rows, pairing each run of deletions and insertions into changed rows
    let mut rows: Vec<(String, DiffRow)> = Vec::new();
    let mut label = mapper.label(0);
    let mut k = 0;
    while k < steps.len() {
        if let Step::Equal(i, j) = steps[k] {
            label = mapper.label(ia[i].0);
            rows.push((
                label.clone(),
                diff_row("equal", Some((a, ia[i].0)), Some((c, ic[j].0))),
            ));
            k += 1;
            continue;
        }
        let (mut dels, mut ins) = (Vec::new(), Vec::new());
        while k < steps.len() {
            match steps[k] {
                Step::Delete(i) => dels.push(ia[i].0),
                Step::Insert(j) => ins.push(ic[j].0),
                Step::Equal(..) => break,
            }
            k += 1;
        }
        for n in 0..dels.len().max(ins.len()) {
            let (d, s) = (dels.get(n).copied(), ins.get(n).copied());
            if let Some(pc) = d {
                label = mapper.label(pc);
            }
            let kind = match (d, s) {
                (Some(_), Some(_)) => "changed",
                (Some(_), None) => "deleted",
                _ => "inserted",
            };
            rows.push((
                label.clone(),
                diff_row(kind, d.map(|pc| (a, pc)), s.map(|pc| (c, pc))),
            ));
        }
    }

    // Per function: changes with context, equal runs left out
    let mut grouped: BTreeMap<String, Vec<DiffRow>> = BTreeMap::new();
    for (l, r) in rows {
        grouped.entry(l).or_default().push(r);
    }
    let mut out = BTreeMap::new();
    for (l, rows) in grouped {
        let near: Vec<bool> = (0..rows.len())
            .map(|i| {
                let lo = i.saturating_sub(DIFF_CONTEXT);
                let hi = (i + DIFF_CONTEXT + 1).min(rows.len());
                rows[lo..hi].iter().any(|r| r.kind != "equal")
            })
            .collect();
        let mut kept: Vec<DiffRow> = Vec::new();
        let mut skipped = 0;
        let mut truncated = false;
        for (r, keep) in rows.into_iter().zip(near) {
            if !keep {
                skipped += 1;
                continue;
            }
            if kept.len() >= DIFF_MAX_ROWS {
                truncated = true;
                break;
            }
            if skipped > 0 {
                kept.push(DiffRow {
                    kind: "skip",
                    artifact_pc: None,
                    artifact: None,
                    onchain_pc: None,
                    onchain: None,
                    skipped: Some(skipped),
                });
                skipped = 0;
            }
            kept.push(r);
        }
        out.insert(l, (kept, truncated));
    }
    (out, approximate)
}

#[derive(Debug, Clone, Serialize)]
pub struct Mismatch {
    pub first_difference_offset: usize,
    pub function: String,
    pub artifact_bytes: String,
    pub onchain_bytes: String,
    /// Offset of the context shown, in the artifact
    pub context_start: usize,
    /// Offset of the first difference in the deployed code (differs from
    /// the artifact's offset when sizes differ)
    pub onchain_offset: usize,
    /// Printable text around the difference, when it is inside a string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_text: Option<String>,
    pub size_difference: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    /// How the per-function table was built
    pub table_method: String,
    pub functions: Vec<FunctionDiff>,
    /// How `functions[].diff` was built (`--diff` only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_method: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MatchReport {
    /// exact | partial | mismatch
    pub verdict: &'static str,
    pub summary: String,
    pub artifact_size: usize,
    pub onchain_size: usize,
    pub executable_size_artifact: usize,
    pub executable_size_onchain: usize,
    pub metadata_artifact: Option<MetadataBlock>,
    pub metadata_onchain: Option<MetadataBlock>,
    pub metadata_equal: bool,
    pub immutables: Vec<MaskedValue>,
    pub libraries: Vec<MaskedValue>,
    pub function_mapping: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mismatch: Option<Mismatch>,
    pub warnings: Vec<String>,
}

fn masked_values(
    onchain: &[u8],
    ranges: &BTreeMap<String, Vec<Range<usize>>>,
    describe: impl Fn(&str) -> String,
) -> Vec<MaskedValue> {
    ranges
        .iter()
        .map(|(id, rs)| {
            let mut values: Vec<String> = rs
                .iter()
                .filter_map(|r| {
                    onchain
                        .get(r.clone())
                        .map(|b| format!("0x{}", hex::encode(b)))
                })
                .collect();
            values.sort();
            values.dedup();
            MaskedValue {
                id: describe(id),
                positions: rs.len(),
                consistent: values.len() <= 1,
                values,
            }
        })
        .collect()
}

/// Byte-for-byte equal outside the masks
fn equal_masked(a: &[u8], b: &[u8], masked: &[bool]) -> bool {
    a.len() == b.len()
        && a.iter()
            .zip(b)
            .enumerate()
            .all(|(i, (x, y))| x == y || masked.get(i).copied().unwrap_or(false))
}

fn first_diff(a: &[u8], b: &[u8], masked: &[bool]) -> Option<usize> {
    (0..a.len().max(b.len()))
        .find(|&i| !masked.get(i).copied().unwrap_or(false) && a.get(i) != b.get(i))
}

/// The printable ASCII run containing `at`, if it is at least 4 characters
fn text_around(code: &[u8], at: usize) -> Option<String> {
    let printable = |b: u8| (0x20..0x7f).contains(&b);
    if !code.get(at).copied().is_some_and(printable) {
        return None;
    }
    let mut start = at;
    while start > 0 && at - start < 64 && printable(code[start - 1]) {
        start -= 1;
    }
    let mut end = at;
    while end < code.len() && end - at < 64 && printable(code[end]) {
        end += 1;
    }
    (end - start >= 4).then(|| String::from_utf8_lossy(&code[start..end]).into_owned())
}

/// Instruction token for alignment when sizes differ: PUSH2/PUSH3 operands
/// are mostly code offsets, which shift, so they are left out
fn token(code: &[u8], pc: usize, op: u8) -> Vec<u8> {
    let n = push_size(op);
    if n == 2 || n == 3 {
        vec![op]
    } else {
        code[pc..(pc + 1 + n).min(code.len())].to_vec()
    }
}

/// CALL, STATICCALL, CALLCODE, DELEGATECALL, CREATE, CREATE2,
/// SELFDESTRUCT, LOG0-LOG4, JUMP, JUMPI, RETURN, SSTORE, TSTORE, ORIGIN,
/// CALLER, STOP, REVERT, INVALID
const EFFECTFUL_OPCODES: &[u8] = &[
    0xf1, 0xfa, 0xf2, 0xf4, 0xf0, 0xf5, 0xff, 0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0x56, 0x57, 0xf3, 0x55,
    0x5d, 0x32, 0x33, 0x00, 0xfd, 0xfe,
];

pub fn match_code(artifact: &Artifact, onchain: &[u8]) -> MatchReport {
    match_code_with(artifact, onchain, false)
}

/// `match_code`, adding per-function instruction diffs when `with_diff`
pub fn match_code_with(artifact: &Artifact, onchain: &[u8], with_diff: bool) -> MatchReport {
    let art = &artifact.runtime;
    let mut warnings = Vec::new();

    let art_blocks = find_metadata_blocks(art);
    let chain_blocks = find_metadata_blocks(onchain);
    let art_trailing = art_blocks.iter().find(|b| b.trailing).cloned();
    let chain_trailing = chain_blocks.iter().find(|b| b.trailing).cloned();
    let art_exec = art_trailing.as_ref().map(|b| b.start).unwrap_or(art.len());
    let chain_exec = chain_trailing
        .as_ref()
        .map(|b| b.start)
        .unwrap_or(onchain.len());

    // Masks over artifact positions: immutables and libraries always
    let mut value_mask = vec![false; art.len()];
    for r in artifact
        .immutables
        .values()
        .chain(artifact.libraries.values())
        .flatten()
    {
        for i in r.clone() {
            if i < value_mask.len() {
                value_mask[i] = true;
            }
        }
    }
    // ...plus nested metadata blocks (embedded contracts), when the deployed
    // code has a block of the same size at the same place
    let mut meta_mask = value_mask.clone();
    for b in art_blocks.iter().filter(|b| !b.trailing) {
        if chain_blocks
            .iter()
            .any(|c| c.start == b.start && c.end == b.end)
        {
            for flag in &mut meta_mask[b.start..b.end] {
                *flag = true;
            }
        } else {
            warnings.push(format!(
                "embedded metadata at {:#x} in the artifact has no counterpart at the same place on chain",
                b.start
            ));
        }
    }

    let immutables = masked_values(onchain, &artifact.immutables, |id| {
        format!("immutable AST id {id}")
    });
    let libraries = masked_values(onchain, &artifact.libraries, |l| l.to_string());
    for m in immutables.iter().chain(&libraries) {
        if !m.consistent {
            warnings.push(format!(
                "{} has different deployed values at its {} positions ({}): the code around it is not what the artifact describes",
                m.id,
                m.positions,
                m.values.join(", ")
            ));
        }
    }

    let exact = equal_masked(art, onchain, &value_mask);
    let exec_equal = art_exec == chain_exec
        && equal_masked(&art[..art_exec], &onchain[..chain_exec], &meta_mask);
    let metadata_equal = match (&art_trailing, &chain_trailing) {
        (Some(a), Some(c)) => art[a.start..a.end] == onchain[c.start..c.end],
        (None, None) => true,
        _ => false,
    };
    let mapper = function_mapper(artifact, art_exec);

    // Deployed bytes shaped like metadata that real code can enter (by
    // falling through, or by a static jump to a JUMPDEST inside) are code,
    // not metadata. A 0x5b in an honest hash is not a jump target, so honest
    // metadata whose hash differs still gives a partial match.
    // #312's call analysis executes every path it can resolve (computed
    // jumps included, as jump tables) and records the pcs it visited
    let analysis = super::calls::analyze_calls(onchain, super::calls::CallsBudget::default());
    let visited = analysis.visited;
    let analysis_complete = analysis.coverage.complete;
    let reach: Vec<(usize, u8, bool)> = linear_sweep(onchain)
        .into_iter()
        .map(|(pc, op)| (pc, op, visited.get(pc).copied().unwrap_or(false)))
        .collect();
    let mut unverified_metadata: Vec<String> = Vec::new();
    let mut executable_metadata: Vec<MetadataBlock> = Vec::new();
    for b in chain_blocks.iter().filter(|b| {
        chain_trailing.as_ref().is_some_and(|t| t.start == b.start)
            || art_blocks.iter().any(|a| !a.trailing && a.start == b.start)
    }) {
        let reached: Vec<u8> = reach
            .iter()
            .filter(|(pc, _, r)| *r && *pc >= b.start && *pc < b.end)
            .map(|(_, op, _)| *op)
            .collect();
        if reached.is_empty() {
            // No analysed path enters it; when the analysis is incomplete,
            // a JUMPDEST inside followed by an effectful opcode before the
            // next halt could still be entered
            // Checked whatever the coverage says: a JUMPDEST followed by an
            // effectful opcode inside metadata-looking bytes never passes
            // as a clean partial (fail closed)
            let _ = analysis_complete;
            {
                // From the block start to the end of the code: a run that
                // starts at a JUMPDEST inside the block may fall through
                // past its end into real code
                let ops: Vec<(usize, u8)> = reach
                    .iter()
                    .filter(|(pc, _, _)| *pc >= b.start)
                    .map(|(pc, op, _)| (*pc, *op))
                    .collect();
                let mut hits: Vec<String> = Vec::new();
                for (i, (pc, op)) in ops.iter().enumerate() {
                    if *op != 0x5b || *pc >= b.end {
                        continue;
                    }
                    for (next_pc, next) in &ops[i + 1..] {
                        if matches!(next, 0x00 | 0xfd | 0xfe) {
                            break;
                        }
                        if EFFECTFUL_OPCODES.contains(next) {
                            hits.push(format!(
                                "JUMPDEST at {pc:#x} then opcode {next:#04x} at {next_pc:#x}{}",
                                if *next_pc >= b.end {
                                    " (past the block end)"
                                } else {
                                    ""
                                }
                            ));
                            break;
                        }
                        if matches!(next, 0x56 | 0xf3 | 0xff) {
                            break;
                        }
                    }
                }
                if !hits.is_empty() {
                    unverified_metadata.push(format!(
                        "deployed bytes {:#x}..{:#x} look like compiler metadata but hold {}; no analysed path enters them, but a jump there is not ruled out",
                        b.start,
                        b.end,
                        hits.join(", ")
                    ));
                }
            }
            continue;
        }
        let effectful = reached.iter().any(|op| EFFECTFUL_OPCODES.contains(op));
        warnings.push(format!(
            "deployed bytes {:#x}..{:#x} look like compiler metadata but an analysed path executes them{}; they are compared as code",
            b.start,
            b.end,
            if effectful { ", and they hold effectful opcodes" } else { "" }
        ));
        executable_metadata.push(b.clone());
    }
    let exec_equal = exec_equal && executable_metadata.is_empty();

    let (verdict, summary) = if exact {
        (
            "exact",
            "Exact match: the deployed code equals the artifact, compiler metadata included (immutables and library addresses masked)".to_string(),
        )
    } else if exec_equal && !unverified_metadata.is_empty() {
        warnings.extend(unverified_metadata.iter().cloned());
        (
            "partial_unverified_metadata",
            "Partial match with unverified metadata: the executable code equals the artifact, but the deployed metadata-looking bytes contain a JUMPDEST followed by effectful opcodes, so they may be reachable. Not a clean partial match, and not verified".to_string(),
        )
    } else if exec_equal {
        (
            "partial",
            "Partial match: the executable code equals the artifact, but the compiler metadata differs. This is not a verified match: the metadata hash commits to the exact sources and settings, which are not the ones deployed".to_string(),
        )
    } else {
        (
            "mismatch",
            "Mismatch: the executable code differs".to_string(),
        )
    };

    let mismatch = (verdict == "mismatch").then(|| {
        let a = &art[..art_exec];
        let c = &onchain[..chain_exec];
        let mut first = first_diff(a, c, &meta_mask).unwrap_or(a.len().min(c.len()));
        let mut onchain_first = first;
        // Per-function table over the artifact's instructions
        let instructions = linear_sweep(a);
        let mut changed = vec![false; instructions.len()];
        let table_method;
        if a.len() == c.len() {
            table_method = "same size: each instruction compared byte for byte at its offset".to_string();
            for (i, (pc, op)) in instructions.iter().enumerate() {
                let end = (pc + 1 + push_size(*op)).min(a.len());
                changed[i] = (*pc..end).any(|j| a[j] != c[j] && !meta_mask[j]);
            }
        } else {
            table_method = "sizes differ: instructions aligned by common prefix and suffix (PUSH2/PUSH3 operands ignored, as code offsets shift); everything between the first and last difference counts as changed".to_string();
            let chain_ins = linear_sweep(c);
            let ta: Vec<Vec<u8>> = instructions.iter().map(|(pc, op)| token(a, *pc, *op)).collect();
            let tc: Vec<Vec<u8>> = chain_ins.iter().map(|(pc, op)| token(c, *pc, *op)).collect();
            let prefix = ta.iter().zip(&tc).take_while(|(x, y)| x == y).count();
            let max_suffix = ta.len().min(tc.len()) - prefix;
            let suffix = ta
                .iter()
                .rev()
                .zip(tc.iter().rev())
                .take(max_suffix)
                .take_while(|(x, y)| x == y)
                .count();
            if let (Some((pa, _)), Some((pb, _))) = (instructions.get(prefix), chain_ins.get(prefix)) {
                first = *pa;
                onchain_first = *pb;
            }
            let end = ta.len() - suffix;
            if prefix < end {
            for flag in &mut changed[prefix..end] {
                    *flag = true;
                }
            } else if !changed.is_empty() {
                // Pure insertion on chain: blame the instruction before it
                let last = changed.len() - 1;
                changed[prefix.saturating_sub(1).min(last)] = true;
            }
        }
        let mut per: BTreeMap<String, (usize, bool)> = BTreeMap::new();
        for (i, (pc, _)) in instructions.iter().enumerate() {
            let e = per.entry(mapper.label(*pc)).or_default();
            e.0 += 1;
            e.1 |= changed[i];
        }
        let mut functions: Vec<FunctionDiff> = per
            .into_iter()
            .map(|(function, (instructions, diff))| FunctionDiff {
                function,
                instructions,
                status: if diff { "different" } else { "equal" },
                diff: None,
                diff_truncated: false,
            })
            .collect();
        let mut diff_note = None;
        if with_diff {
            let (mut per_fn, approximate) = function_diffs(a, c, &meta_mask, &mapper);
            for f in functions.iter_mut().filter(|f| f.status == "different") {
                let (rows, truncated) = per_fn.remove(&f.function).unwrap_or_default();
                f.diff = Some(rows);
                f.diff_truncated = truncated;
            }
            diff_note = Some(format!(
                "instructions aligned by LCS over the executable code{}{}; masked bytes (immutables, libraries, embedded metadata) match any deployed value; {} equal rows of context, at most {} rows per function",
                if a.len() != c.len() {
                    ", PUSH2/PUSH3 operands ignored (code offsets shift)"
                } else {
                    ""
                },
                if approximate {
                    "; APPROXIMATE: a large region with no unique anchor is shown as replaced"
                } else {
                    ""
                },
                DIFF_CONTEXT,
                DIFF_MAX_ROWS
            ));
        }
        functions.sort_by_key(|f| (f.status == "equal", f.function.clone()));
        let window = |code: &[u8], at: usize| {
            let start = at.saturating_sub(16).min(code.len());
            hex::encode(&code[start..(at + 16).min(code.len())])
        };
        let function = mapper.label(first);
        let user_changed = functions
            .iter()
            .any(|f| f.status == "different" && (!f.function.starts_with('(') || f.function.starts_with("(source ")));
        let hint = (!user_changed && mapper.complete).then(|| {
            "the difference is only in code shared between functions (ABI coders, string and error helpers, checked arithmetic, the dispatcher); look for the constant or text shown here in the sources".to_string()
        });
        Mismatch {
            first_difference_offset: first,
            function,
            artifact_bytes: window(a, first),
            onchain_bytes: window(c, onchain_first),
            context_start: first.saturating_sub(16),
            onchain_offset: onchain_first,
            artifact_text: text_around(a, first),
            onchain_text: text_around(c, onchain_first),
            hint,
            size_difference: c.len() as i64 - a.len() as i64,
            table_method,
            functions,
            diff_method: diff_note,
        }
    });

    if art_trailing.is_none() {
        warnings.push("the artifact has no compiler metadata (compiled with bytecodeHash none or appendCBOR false); an exact match then cannot vouch for the sources".into());
    }

    MatchReport {
        verdict,
        summary,
        artifact_size: art.len(),
        onchain_size: onchain.len(),
        executable_size_artifact: art_exec,
        executable_size_onchain: chain_exec,
        metadata_artifact: art_trailing,
        metadata_onchain: chain_trailing,
        metadata_equal,
        immutables,
        libraries,
        function_mapping: mapper.method,
        mismatch,
        warnings,
    }
}

/// Share of the target contract's source that is inline assembly, from the AST
#[derive(Debug, Clone, Serialize)]
pub struct AssemblyShare {
    /// Bytes of source inside `assembly { }` blocks, all files with ASTs
    pub assembly_bytes: u64,
    /// Bytes of source inside function and modifier bodies
    pub function_bytes: u64,
    pub blocks: usize,
    pub percent: f64,
}

/// Collect ContractDefinition nodes: (id, name, linearizedBaseContracts, node)
fn contract_nodes<'a>(node: &'a Value, out: &mut Vec<(u64, String, Vec<u64>, &'a Value)>) {
    match node {
        Value::Object(map) => {
            if map.get("nodeType").and_then(Value::as_str) == Some("ContractDefinition") {
                let bases = map
                    .get("linearizedBaseContracts")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(Value::as_u64).collect())
                    .unwrap_or_default();
                if let Some(id) = map.get("id").and_then(Value::as_u64) {
                    out.push((
                        id,
                        map.get("name")
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                        bases,
                        node,
                    ));
                }
            }
            for v in map.values() {
                contract_nodes(v, out);
            }
        }
        Value::Array(items) => items.iter().for_each(|v| contract_nodes(v, out)),
        _ => {}
    }
}

/// Inline-assembly share of the matched contract and the contracts it
/// inherits (its linearised bases), not the whole build
pub fn assembly_share(artifact: &Artifact) -> Option<AssemblyShare> {
    if artifact.asts.is_empty() {
        return None;
    }
    let mut contracts = Vec::new();
    let mut own = Vec::new();
    let name = artifact.contract_name.as_deref()?;
    for (_, ast) in &artifact.asts {
        let mut here = Vec::new();
        contract_nodes(ast, &mut here);
        // The matched contract: this name in this artifact's own source file
        let path_ok = match (&artifact.source_path, ast["absolutePath"].as_str()) {
            (Some(want), Some(have)) => want == have,
            _ => artifact.asts.len() == 1,
        };
        if path_ok {
            own.extend(
                here.iter()
                    .filter(|(_, n, _, _)| n == name)
                    .map(|(_, _, b, _)| b.clone()),
            );
        }
        contracts.extend(here);
    }
    // Unknown when the name is not found in its file, or found twice
    if own.len() != 1 {
        return None;
    }
    let bases = own.remove(0);
    let mut spans = Vec::new();
    let mut asm = Vec::new();
    for (id, _, _, node) in &contracts {
        if bases.contains(id) {
            collect_functions(node, None, &mut spans, &mut asm);
        }
    }
    let function_bytes: u64 = spans.iter().map(|s| s.end - s.start).sum();
    let assembly_bytes: u64 = asm.iter().map(|(_, l, _)| *l).sum();
    Some(AssemblyShare {
        assembly_bytes,
        function_bytes,
        blocks: asm.len(),
        percent: if function_bytes == 0 {
            0.0
        } else {
            (assembly_bytes as f64 * 1000.0 / function_bytes as f64).round() / 10.0
        },
    })
}

/// Compare creation code from a deployment transaction with the artifact's
#[derive(Debug, Clone, Serialize)]
pub struct CreationReport {
    /// exact | partial | mismatch
    pub verdict: &'static str,
    pub creation_size: usize,
    pub constructor_args: String,
    pub notes: Vec<String>,
}

pub fn match_creation(artifact: &Artifact, input: &[u8]) -> Option<CreationReport> {
    let creation = artifact.creation.as_ref()?;
    let mut notes = Vec::new();
    if input.len() < creation.len() {
        return Some(CreationReport {
            verdict: "mismatch",
            creation_size: creation.len(),
            constructor_args: String::new(),
            notes: vec![
                "the transaction input is shorter than the artifact's creation code".into(),
            ],
        });
    }
    let head = &input[..creation.len()];
    let mut value_mask = vec![false; creation.len()];
    for r in artifact.creation_libraries.values().flatten() {
        for i in r.clone() {
            if i < value_mask.len() {
                value_mask[i] = true;
            }
        }
    }
    let exact = equal_masked(creation, head, &value_mask);
    let mut meta_mask = value_mask;
    for b in find_metadata_blocks(creation) {
        for flag in &mut meta_mask[b.start..b.end] {
            *flag = true;
        }
    }
    let partial = equal_masked(creation, head, &meta_mask);
    if !partial {
        notes.push(format!(
            "first difference at creation offset {:#x}",
            first_diff(creation, head, &meta_mask).unwrap_or(0)
        ));
    }
    Some(CreationReport {
        verdict: if exact {
            "exact"
        } else if partial {
            "partial"
        } else {
            "mismatch"
        },
        creation_size: creation.len(),
        constructor_args: format!("0x{}", hex::encode(&input[creation.len()..])),
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const META: &str = "a26469706673582212208b5d96ae75ef7e28d88d1c3820279a801b63c4e1319cce2a5166f9a221abe79564736f6c63430008150033";

    fn forge_artifact(runtime_hex: &str, immutables: Value, links: Value) -> Value {
        serde_json::json!({
            "abi": [],
            "bytecode": {"object": "0x6080", "linkReferences": {}},
            "deployedBytecode": {
                "object": runtime_hex,
                "linkReferences": links,
                "immutableReferences": immutables,
            },
            "methodIdentifiers": {"foo()": "c2985578"},
        })
    }

    fn with_meta(code: &str, meta: &str) -> String {
        format!("0x{code}{meta}")
    }

    #[test]
    fn exact_partial_and_mismatch() {
        let code = "6080604052600436106000575b00";
        let art = parse_artifact(
            &forge_artifact(&with_meta(code, META), Value::Null, Value::Null),
            None,
        )
        .unwrap();
        let same = hex::decode(&with_meta(code, META)[2..]).unwrap();
        assert_eq!(match_code(&art, &same).verdict, "exact");

        let other_meta = META.replace("8b5d96ae", "00000000");
        let partial = hex::decode(&with_meta(code, &other_meta)[2..]).unwrap();
        let report = match_code(&art, &partial);
        assert_eq!(
            report.verdict,
            "partial",
            "{:?} {:?} {:?} {:?}",
            report.warnings,
            report.metadata_onchain,
            report.executable_size_onchain,
            report.mismatch.as_ref().map(|m| m.first_difference_offset)
        );
        assert!(!report.metadata_equal);
        // Never call a partial match verified
        let lower = report.summary.to_lowercase();
        assert!(lower.contains("not a verified match"));

        let changed = hex::decode(&with_meta(&code.replace("6004", "6005"), META)[2..]).unwrap();
        let report = match_code(&art, &changed);
        assert_eq!(report.verdict, "mismatch");
        let m = report.mismatch.unwrap();
        assert_eq!(m.first_difference_offset, 6);
        assert_eq!(m.size_difference, 0);
        assert!(m.functions.iter().all(|f| f.diff.is_none()));
        assert!(m.diff_method.is_none());
    }

    #[test]
    fn diff_aligns_changes_and_masks_immutables() {
        // PUSH1 1, PUSH32 <immutable>, POP, PUSH1 2, PUSH1 3, ADD, STOP
        let zero = "00".repeat(32);
        let code = format!("60017f{zero}5060026003010000");
        let art = parse_artifact(
            &forge_artifact(
                &with_meta(&code, META),
                serde_json::json!({"7": [{"start": 3, "length": 32}]}),
                Value::Null,
            ),
            None,
        )
        .unwrap();
        // Deployed: the immutable has a value, PUSH1 3 became PUSH1 4 and a
        // PUSH1 0x0a was inserted before STOP
        let value = format!("{}{}", "00".repeat(12), "11".repeat(20));
        let deployed_hex = format!("60017f{value}50600260040160 0a0000").replace(' ', "");
        let deployed = hex::decode(&with_meta(&deployed_hex, META)[2..]).unwrap();
        let report = match_code_with(&art, &deployed, true);
        assert_eq!(report.verdict, "mismatch");
        let m = report.mismatch.unwrap();
        assert!(m.diff_method.is_some());
        let rows: Vec<&DiffRow> = m
            .functions
            .iter()
            .filter_map(|f| f.diff.as_ref())
            .flatten()
            .collect();
        let kinds: Vec<&str> = rows.iter().map(|r| r.kind).collect();
        // The masked PUSH32 is equal despite its different deployed value
        let push32 = rows
            .iter()
            .find(|r| {
                r.artifact
                    .as_deref()
                    .is_some_and(|t| t.starts_with("PUSH32"))
            })
            .expect("PUSH32 row in context");
        assert_eq!(push32.kind, "equal");
        assert!(push32
            .onchain
            .as_deref()
            .unwrap()
            .ends_with(&"11".repeat(20)));
        let changed = rows
            .iter()
            .find(|r| r.kind == "changed")
            .expect("a changed row");
        assert_eq!(changed.artifact.as_deref(), Some("PUSH1 0x03"));
        assert_eq!(changed.onchain.as_deref(), Some("PUSH1 0x04"));
        let inserted = rows
            .iter()
            .find(|r| r.kind == "inserted")
            .expect("an inserted row");
        assert_eq!(inserted.onchain.as_deref(), Some("PUSH1 0x0a"));
        assert!(!kinds.contains(&"deleted"), "{kinds:?}");
    }

    #[test]
    fn masks_immutables_and_checks_consistency() {
        // PUSH32 <imm> POP PUSH32 <imm> POP
        let zero = "00".repeat(32);
        let code = format!("7f{zero}507f{zero}50");
        let imm =
            serde_json::json!({"7": [{"start": 1, "length": 32}, {"start": 35, "length": 32}]});
        let art = parse_artifact(
            &forge_artifact(&format!("0x{code}"), imm, Value::Null),
            None,
        )
        .unwrap();
        let v = "11".repeat(32);
        let deployed = hex::decode(format!("7f{v}507f{v}50")).unwrap();
        let report = match_code(&art, &deployed);
        assert_eq!(report.verdict, "exact");
        assert!(report.immutables[0].consistent);
        assert_eq!(report.immutables[0].values, vec![format!("0x{v}")]);

        let w = "22".repeat(32);
        let report = match_code(&art, &hex::decode(format!("7f{v}507f{w}50")).unwrap());
        assert!(!report.immutables[0].consistent);
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("different deployed values")));
    }

    #[test]
    fn replaces_link_placeholders() {
        let placeholder = "__$1234567890abcdef1234567890abcdef12$__";
        assert_eq!(placeholder.len(), 40);
        let code = format!("0x73{placeholder}50");
        let links = serde_json::json!({"src/L.sol": {"L": [{"start": 1, "length": 20}]}});
        let art = parse_artifact(&forge_artifact(&code, Value::Null, links), None).unwrap();
        let deployed = hex::decode(format!("73{}50", "ab".repeat(20))).unwrap();
        let report = match_code(&art, &deployed);
        assert_eq!(report.verdict, "exact");
        assert_eq!(report.libraries[0].id, "src/L.sol:L");
    }

    #[test]
    fn unlisted_placeholder_is_an_error() {
        let code = format!("0x73{}50", "__$1234567890abcdef1234567890abcdef12$__");
        assert!(parse_artifact(&forge_artifact(&code, Value::Null, Value::Null), None).is_err());
    }

    #[test]
    fn reads_standard_json_and_requires_a_choice() {
        let entry = |code: &str| serde_json::json!({"evm": {"deployedBytecode": {"object": code}, "bytecode": {"object": "6080"}, "methodIdentifiers": {}}});
        let out = serde_json::json!({"contracts": {"a.sol": {"A": entry("6000"), "B": entry("6001")}}, "sources": {}});
        assert!(parse_artifact(&out, None).is_err());
        let art = parse_artifact(&out, Some("B")).unwrap();
        assert_eq!(art.runtime, vec![0x60, 0x01]);
        assert_eq!(art.format, "solc-standard-json");
    }

    #[test]
    fn source_map_decoding_carries_fields_forward() {
        let entries = decode_source_map("1:2:0;;3::;:4:-1");
        assert_eq!(entries[0], Some((1, 2, 0)));
        assert_eq!(entries[1], Some((1, 2, 0)));
        assert_eq!(entries[2], Some((3, 2, 0)));
        assert_eq!(entries[3], Some((3, 4, -1)));
    }

    #[test]
    fn size_change_localised_by_prefix_and_suffix() {
        let art = parse_artifact(
            &forge_artifact("0x6001600201600355", Value::Null, Value::Null),
            None,
        )
        .unwrap();
        // Extra instruction in the middle
        let deployed = hex::decode("600160020180600355").unwrap();
        let m = match_code(&art, &deployed).mismatch.unwrap();
        assert_eq!(m.size_difference, 1);
        assert!(m.table_method.starts_with("sizes differ"));
    }

    // Fixture: forge build --ast of a small contract (solc 0.8.21, optimizer
    // 200, shanghai). The second runtime is the same source with
    // `msg.sender == owner` changed to `tx.origin == owner` in sweep().
    const VAULT: &str = include_str!("../../tests/fixtures/match/Vault.json");
    const VAULT_TXORIGIN: &str =
        include_str!("../../tests/fixtures/match/Vault_txorigin_runtime.hex");

    fn deployed_from(art: &Artifact, runtime: &[u8], owner: u8) -> Vec<u8> {
        let mut code = runtime.to_vec();
        for r in art.immutables.values().flatten() {
            code[r.start + 12..r.end].fill(owner);
        }
        code
    }

    #[test]
    fn fixture_exact_partial_and_localised_mismatch() {
        let json: Value = serde_json::from_str(VAULT).unwrap();
        let art = parse_artifact(&json, Some("Vault")).unwrap();
        assert!(!art.asts.is_empty() && art.source_map.is_some());
        assert_eq!(art.immutables.len(), 1);

        let exact = deployed_from(&art, &art.runtime, 0xaa);
        let report = match_code(&art, &exact);
        assert_eq!(report.verdict, "exact");
        assert!(report.immutables[0].consistent);
        // A forge artifact alone cannot tell generated code from other files
        assert!(report.function_mapping.starts_with("source map + AST"));

        let mut partial = exact.clone();
        let meta = report.metadata_artifact.clone().unwrap();
        // Change a hash byte without creating a JUMPDEST or a PUSH, which
        // would make the metadata executable (and the verdict not partial)
        let at = meta.start + 10;
        partial[at] = if partial[at] == 0x01 { 0x02 } else { 0x01 };
        let report = match_code(&art, &partial);
        assert_eq!(
            report.verdict,
            "partial",
            "{:?} {:?}",
            report.warnings,
            report.mismatch.as_ref().map(|m| m.first_difference_offset)
        );
        let text = serde_json::to_string(&report).unwrap().to_lowercase();
        for (i, _) in text.match_indices("verified") {
            assert!(
                text[..i].ends_with("not a "),
                "partial report calls itself verified"
            );
        }

        let other = hex::decode(VAULT_TXORIGIN.trim().trim_start_matches("0x")).unwrap();
        let report = match_code(&art, &deployed_from(&art, &other, 0xaa));
        assert_eq!(report.verdict, "mismatch");
        let m = report.mismatch.unwrap();
        assert_eq!(m.function, "Vault.sweep");
        let different: Vec<&str> = m
            .functions
            .iter()
            .filter(|f| f.status == "different")
            .map(|f| f.function.as_str())
            .collect();
        assert_eq!(different, vec!["Vault.sweep"]);
        assert_eq!(assembly_share(&art).unwrap().blocks, 0);
    }

    #[test]
    fn hostile_ranges_error_instead_of_panicking() {
        for (start, len) in [
            (10u64, u64::MAX),
            (0, 9_000_000_000_000_000_000),
            (u64::MAX, 1),
            (4000, 20),
        ] {
            let imm = serde_json::json!({"1": [{"start": start, "length": len}]});
            let a = forge_artifact("0x600160015500", imm, Value::Null);
            assert!(parse_artifact(&a, None).is_err(), "immutable {start}/{len}");
            let links = serde_json::json!({"a.sol": {"L": [{"start": start, "length": len}]}});
            let a = forge_artifact("0x600160015500", Value::Null, links);
            assert!(parse_artifact(&a, None).is_err(), "link {start}/{len}");
        }
    }

    #[test]
    fn metadata_entered_by_a_jump_is_code() {
        // PUSH2 0x000f JUMP STOP: jumps to pc 15, which is inside the
        // metadata (the first ipfs hash byte)
        let code = "61000f5600";
        let art = parse_artifact(
            &forge_artifact(&with_meta(code, META), Value::Null, Value::Null),
            None,
        )
        .unwrap();
        for payload in ["5b33ff00", "5bf10000", "5b00aaaa", "5bfdaaaa", "5b600160"] {
            let evil = META.replacen("8b5d96ae", payload, 1);
            let deployed = hex::decode(&with_meta(code, &evil)[2..]).unwrap();
            let report = match_code(&art, &deployed);
            assert_ne!(report.verdict, "partial", "{payload}");
            assert!(report
                .warnings
                .iter()
                .any(|w| w.contains("compared as code")));
        }
    }

    #[test]
    fn metadata_jumpdest_run_fails_closed_even_with_full_coverage() {
        // Nothing jumps into the metadata: a 0x5b followed by CALLER in the
        // hash is just hash bytes
        // No computed jumps (POP instead of JUMPI), so an unreached
        // 0x5b run in the hash cannot be entered
        let code = "6080604052600436106000505b00";
        let art = parse_artifact(
            &forge_artifact(&with_meta(code, META), Value::Null, Value::Null),
            None,
        )
        .unwrap();
        let other = META.replacen("8b5d96ae", "5b33a2ff", 1);
        let deployed = hex::decode(&with_meta(code, &other)[2..]).unwrap();
        let report = match_code(&art, &deployed);
        // A JUMPDEST then CALLER: fail closed even with complete coverage
        assert_eq!(
            report.verdict, "partial_unverified_metadata",
            "{:?}",
            report.warnings
        );
        // A JUMPDEST then STOP is harmless
        let other = META.replacen("8b5d96ae", "5b00a2ff", 1);
        let deployed = hex::decode(&with_meta(code, &other)[2..]).unwrap();
        assert_eq!(match_code(&art, &deployed).verdict, "partial");
    }

    #[test]
    fn build_info_must_hold_the_same_bytecode() {
        let art = parse_artifact(
            &serde_json::json!({
                "deployedBytecode": {"object": "0x6001"},
                "bytecode": {"object": "0x6001"},
                "metadata": {"settings": {"compilationTarget": {"src/A.sol": "A"}}},
            }),
            None,
        )
        .unwrap();
        let same = serde_json::json!({"output": {"contracts": {"src/A.sol": {"A": {"evm": {"deployedBytecode": {"object": "6001"}}}}}}});
        let stale = serde_json::json!({"output": {"contracts": {"src/A.sol": {"A": {"evm": {"deployedBytecode": {"object": "6002"}}}}}}});
        assert!(art.build_info_matches(&same));
        assert!(!art.build_info_matches(&stale));
    }

    #[test]
    fn assembly_share_counts_only_the_contract_and_its_bases() {
        let ast_a = serde_json::json!({"nodeType": "SourceUnit", "nodes": [
            {"nodeType": "ContractDefinition", "id": 1, "name": "Base", "linearizedBaseContracts": [1], "nodes": [
                {"nodeType": "FunctionDefinition", "name": "f", "src": "0:100:0", "body": {}}]},
            {"nodeType": "ContractDefinition", "id": 2, "name": "A", "linearizedBaseContracts": [2, 1], "nodes": [
                {"nodeType": "FunctionDefinition", "name": "g", "src": "100:100:0", "body": {}}]}
        ]});
        let ast_other = serde_json::json!({"nodeType": "SourceUnit", "nodes": [
            {"nodeType": "ContractDefinition", "id": 3, "name": "Other", "linearizedBaseContracts": [3], "nodes": [
                {"nodeType": "FunctionDefinition", "name": "h", "src": "0:100:1", "body": {
                    "nodeType": "InlineAssembly", "src": "10:50:1"}}]}
        ]});
        let mut ast_a = ast_a;
        ast_a["absolutePath"] = serde_json::json!("src/A.sol");
        let art = Artifact {
            contract_name: Some("A".into()),
            source_path: Some("src/A.sol".into()),
            asts: vec![(Some(0), ast_a), (Some(1), ast_other)],
            ..Default::default()
        };
        let share = assembly_share(&art).unwrap();
        assert_eq!(share.blocks, 0);
        assert_eq!(share.function_bytes, 200);
    }

    #[test]
    fn same_name_in_another_file_does_not_count() {
        let a = |path: &str, asm: bool| {
            serde_json::json!({"nodeType": "SourceUnit", "absolutePath": path, "nodes": [
                {"nodeType": "ContractDefinition", "id": if asm { 9 } else { 1 }, "name": "A",
                 "linearizedBaseContracts": [if asm { 9 } else { 1 }], "nodes": [
                    {"nodeType": "FunctionDefinition", "name": "f", "src": "0:100:0",
                     "body": if asm { serde_json::json!({"nodeType": "InlineAssembly", "src": "1:50:0"}) } else { serde_json::json!({}) }}]}]})
        };
        let mut art = Artifact {
            contract_name: Some("A".into()),
            source_path: Some("src/A.sol".into()),
            asts: vec![
                (Some(0), a("src/A.sol", false)),
                (Some(1), a("lib/A.sol", true)),
            ],
            ..Default::default()
        };
        assert_eq!(assembly_share(&art).unwrap().blocks, 0);
        // Two definitions in the artifact's own file: unknown
        art.asts.push((Some(2), a("src/A.sol", true)));
        assert!(assembly_share(&art).is_none());
    }

    #[test]
    fn computed_jump_into_fake_metadata_is_flagged() {
        // PUSH1 0 CALLDATALOAD JUMP: a computed jump, then STOP
        let code = "6000355600";
        let art = parse_artifact(
            &forge_artifact(&with_meta(code, META), Value::Null, Value::Null),
            None,
        )
        .unwrap();
        let evil = META.replacen("8b5d96ae", "5b33ff00", 1);
        let deployed = hex::decode(&with_meta(code, &evil)[2..]).unwrap();
        let report = match_code(&art, &deployed);
        assert_eq!(
            report.verdict, "partial_unverified_metadata",
            "{:?}",
            report.warnings
        );
        assert!(report.warnings.iter().any(|w| w.contains("not ruled out")));
        // Honest hash: a 0x5b followed by STOP is not an effectful run
        let honest = META.replacen("8b5d96ae", "5b00aaaa", 1);
        let deployed = hex::decode(&with_meta(code, &honest)[2..]).unwrap();
        assert_eq!(match_code(&art, &deployed).verdict, "partial");
    }

    #[test]
    fn run_falling_off_a_mid_code_block_is_flagged() {
        // A nested metadata-shaped block: {"ipfs": <34 bytes>} + length
        let nested = |hash: &[u8]| {
            let mut b = hex::decode("a1646970667358").unwrap();
            b.push(34);
            b.extend(hash);
            let len = b.len() as u16;
            b.extend(len.to_be_bytes());
            hex::encode(b)
        };
        // JUMPDEST, 32 x ADD, PUSH1 (swallows the length's 0x00), then the
        // length's low byte (MUL) runs past the block end into CALL
        let mut evil = vec![0x5b];
        evil.extend([0x01; 32]);
        evil.push(0x60);
        let prefix = "6000355600"; // computed jump, then STOP
        let art_code = format!("{prefix}{}f100", nested(&[0u8; 34]));
        let dep_code = format!("{prefix}{}f100", nested(&evil));
        let art = parse_artifact(
            &forge_artifact(&with_meta(&art_code, META), Value::Null, Value::Null),
            None,
        )
        .unwrap();
        let deployed = hex::decode(&with_meta(&dep_code, META)[2..]).unwrap();
        let report = match_code(&art, &deployed);
        assert_eq!(
            report.verdict, "partial_unverified_metadata",
            "{:?}",
            report.warnings
        );
        assert!(report
            .warnings
            .iter()
            .any(|w| w.contains("past the block end")));
    }
}
