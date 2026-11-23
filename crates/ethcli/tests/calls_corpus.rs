//! A/B harness for `contract calls`: run the analysis over a directory of
//! runtime bytecode (`<name>.hex`) and write every fact as a line, so two
//! builds can be compared with `diff`/`comm`. Ignored by default.
//!
//! CALLS_CORPUS_DIR=<dir> CALLS_CORPUS_OUT=<file> \
//!   cargo test -p ethcli --release --test calls_corpus -- --ignored --nocapture

use ethcli::bytecode::{analyze_calls, CallsBudget};
use serde_json::Value;
use std::collections::BTreeSet;

fn key(p: &Value) -> String {
    ["kind", "value", "slot", "offset", "call_pc"]
        .iter()
        .map(|k| p.get(k).map(|v| v.to_string()).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("/")
}

fn facts(name: &str, r: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for c in r["call_sites"].as_array().into_iter().flatten() {
        for f in c["functions"].as_array().into_iter().flatten() {
            let at = format!("{name}|call|{}|{}|{}", c["pc"], c["kind"], f);
            out.insert(at.clone());
            for field in ["target", "selector", "value"] {
                for p in c[field].as_array().into_iter().flatten() {
                    out.insert(format!("{at}|{field}|{}", key(p)));
                }
            }
            for (i, arg) in c["args"].as_array().into_iter().flatten().enumerate() {
                for p in arg.as_array().into_iter().flatten() {
                    out.insert(format!("{at}|arg{i}|{}", key(p)));
                }
            }
        }
    }
    for f in r["functions"].as_array().into_iter().flatten() {
        let func = &f["function"];
        for g in f["guards"].as_array().into_iter().flatten() {
            out.insert(format!(
                "{name}|guard|{func}|{}|{}|{}|{}",
                g["pc"],
                g["subject"],
                g["check"],
                key(&g["against"])
            ));
        }
        for s in f["sstores"].as_array().into_iter().flatten() {
            out.insert(format!(
                "{name}|sstore|{func}|{}|{}",
                s["pc"],
                key(&s["slot"])
            ));
        }
        for field in [
            "origin_pcs",
            "caller_pcs",
            "delegatecall_pcs",
            "selfdestruct_pcs",
            "create_pcs",
        ] {
            for pc in f[field].as_array().into_iter().flatten() {
                out.insert(format!("{name}|{field}|{func}|{pc}"));
            }
        }
    }
    for s in r["opcode_sites"].as_array().into_iter().flatten() {
        if s["reached"] == true {
            out.insert(format!("{name}|reached|{}|{}", s["opcode"], s["pc"]));
        }
    }
    out
}

#[test]
#[ignore]
fn corpus() {
    let Ok(dir) = std::env::var("CALLS_CORPUS_DIR") else {
        eprintln!("CALLS_CORPUS_DIR not set");
        return;
    };
    let out_path = std::env::var("CALLS_CORPUS_OUT").expect("CALLS_CORPUS_OUT");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "hex"))
        .collect();
    names.sort();
    let mut all = BTreeSet::new();
    let mut summary = Vec::new();
    for path in names {
        let name = path.file_stem().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        let code = hex::decode(text.trim().trim_start_matches("0x")).unwrap();
        let t = std::time::Instant::now();
        let r = serde_json::to_value(analyze_calls(&code, CallsBudget::default())).unwrap();
        let ms = t.elapsed().as_millis();
        let c = &r["coverage"];
        summary.push(format!(
            "{name}|blocks {}/{}|pruned {}|unresolved_jumps {}|aborted {}|{ms} ms",
            c["cfg_blocks_analysed"],
            c["cfg_reachable_blocks"],
            c["states_pruned"],
            c["unresolved_jumps"].as_array().map_or(0, Vec::len),
            c["paths_aborted"]
        ));
        all.extend(facts(&name, &r));
    }
    let mut text: Vec<String> = all.into_iter().collect();
    text.extend(summary.iter().map(|s| format!("~summary|{s}")));
    std::fs::write(out_path, text.join("\n") + "\n").unwrap();
    for s in summary {
        println!("{s}");
    }
}
