//! Embeds every file under `skills/<name>/` into the binary.
//!
//! The canonical skills live at the workspace root (`skills/`); this crate
//! carries a copy so `cargo package` ships them. A test checks the two match.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

fn collect(dir: &Path, base: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, base, out);
        } else if path.is_file() {
            out.push(path.strip_prefix(base).unwrap().to_path_buf());
        }
    }
}

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let skills_dir = manifest_dir.join("skills");
    println!("cargo:rerun-if-changed=skills");

    let mut files = Vec::new();
    collect(&skills_dir, &skills_dir, &mut files);
    files.sort();

    let mut code = String::from("&[\n");
    for rel in &files {
        let parts: Vec<String> = rel
            .components()
            .map(|c| c.as_os_str().to_string_lossy().into_owned())
            .collect();
        if parts.len() < 2 || parts.iter().any(|p| p.starts_with('.')) {
            continue;
        }
        let skill = &parts[0];
        let inner = parts[1..].join("/");
        let abs = skills_dir.join(rel);
        println!("cargo:rerun-if-changed={}", abs.display());
        writeln!(
            code,
            "    ({skill:?}, {inner:?}, include_str!({:?})),",
            abs.display().to_string()
        )
        .unwrap();
    }
    code.push(']');

    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded_skills.rs");
    std::fs::write(out, code).unwrap();
}
