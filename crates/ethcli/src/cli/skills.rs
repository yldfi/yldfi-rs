//! Agent skills shipped inside the binary, and installing them for agents.
//!
//! Every file under `crates/ethcli/skills/<name>/` is embedded at build time
//! (see `build.rs`). `skills install` writes them to an agent's skills folder
//! and records what it wrote in a manifest next to ethcli's other local data
//! (never in the main config file). A destination whose content does not match
//! what ethcli last wrote there is treated as the user's and is not replaced
//! without `--force`.

use clap::{Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// (skill, path inside the skill folder, contents)
const EMBEDDED: &[(&str, &str, &str)] = include!(concat!(env!("OUT_DIR"), "/embedded_skills.rs"));

/// Version stamped on installed skills: they ship with this ethcli release.
pub const SKILLS_VERSION: &str = env!("CARGO_PKG_VERSION");

const MANIFEST_FILE: &str = "skills-installed.json";

#[derive(Subcommand, Debug, Clone)]
pub enum SkillsCommands {
    /// List the skills embedded in this binary, with version and content hash
    List {
        /// JSON output
        #[arg(long)]
        json: bool,
    },
    /// Print a skill (its SKILL.md, or another file with --file)
    Show {
        /// Skill name (see `skills list`)
        name: String,
        /// File inside the skill folder
        #[arg(long, default_value = "SKILL.md")]
        file: String,
    },
    /// Install skills into an agent's skills folder
    Install {
        /// Where to install: claude (~/.claude/skills), codex (~/.agents/skills,
        /// shared by Codex and other agents) or dir (--dir). Same locations as
        /// the `npx skills` CLI
        #[arg(long, value_enum, default_value = "claude")]
        target: InstallTarget,
        /// Install into the current project instead (.claude/skills for claude,
        /// .agents/skills for codex)
        #[arg(long, conflicts_with = "dir")]
        project: bool,
        /// Skills folder for --target dir (each skill goes in <dir>/<name>/)
        #[arg(long)]
        dir: Option<PathBuf>,
        /// Install only these skills (repeatable; default: all)
        #[arg(long = "name")]
        names: Vec<String>,
        /// Replace files ethcli did not write (a .bak-<timestamp> copy is kept)
        #[arg(long)]
        force: bool,
    },
    /// Refresh installed skills that are unchanged since ethcli wrote them
    Update,
    /// State of each installed skill: current, outdated, modified or missing
    Status {
        /// JSON output
        #[arg(long)]
        json: bool,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    Claude,
    Codex,
    Dir,
}

/// One embedded skill.
#[derive(Debug, Clone, Serialize)]
pub struct Skill {
    pub name: &'static str,
    pub version: &'static str,
    /// Skill folder hash, the `computedHash` of the `skills` CLI's
    /// skills-lock.json v2 (see [`computed_hash`])
    pub hash: String,
    pub files: Vec<&'static str>,
}

pub fn sha256_hex(data: &[u8]) -> String {
    hex::encode(Sha256::digest(data))
}

/// The skill folder hash the open agent-skills CLI (`npx skills`) records as
/// `computedHash` in skills-lock.json v2: files sorted by the UTF-8 bytes of
/// their `/`-separated relative paths, each hashed as `path NUL length NUL
/// content`, with CRLF turned into LF in files that hold no NUL byte.
pub fn computed_hash(files: &[(&str, &[u8])]) -> String {
    let mut entries: Vec<(String, Vec<u8>)> = files
        .iter()
        .map(|(p, c)| {
            let content = if c.contains(&0) || !c.windows(2).any(|w| w == b"\r\n") {
                c.to_vec()
            } else {
                let mut out = Vec::with_capacity(c.len());
                let mut i = 0;
                while i < c.len() {
                    if c[i] == b'\r' && c.get(i + 1) == Some(&b'\n') {
                        i += 1;
                        continue;
                    }
                    out.push(c[i]);
                    i += 1;
                }
                out
            };
            (p.replace('\\', "/"), content)
        })
        .collect();
    entries.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()));
    let mut h = Sha256::new();
    for (path, content) in &entries {
        h.update(path.as_bytes());
        h.update([0]);
        h.update(content.len().to_string().as_bytes());
        h.update([0]);
        h.update(content);
    }
    hex::encode(h.finalize())
}

/// The embedded skills, sorted by name.
pub fn embedded_skills() -> Vec<Skill> {
    let mut by_name: BTreeMap<&str, Vec<(&str, &str)>> = BTreeMap::new();
    for (skill, file, body) in EMBEDDED {
        by_name.entry(skill).or_default().push((file, body));
    }
    by_name
        .into_iter()
        .map(|(name, files)| {
            let hashed: Vec<(&str, &[u8])> =
                files.iter().map(|(f, b)| (*f, b.as_bytes())).collect();
            Skill {
                name,
                version: SKILLS_VERSION,
                hash: computed_hash(&hashed),
                files: files.iter().map(|(f, _)| *f).collect(),
            }
        })
        .collect()
}

fn embedded_file(skill: &str, file: &str) -> Option<&'static str> {
    EMBEDDED
        .iter()
        .find(|(s, f, _)| *s == skill && *f == file)
        .map(|(_, _, b)| *b)
}

/// A file ethcli wrote.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestEntry {
    pub path: PathBuf,
    pub skill: String,
    pub file: String,
    pub version: String,
    pub sha256: String,
    pub installed_at: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    #[serde(default)]
    pub entries: Vec<ManifestEntry>,
}

impl Manifest {
    pub fn default_path() -> PathBuf {
        if let Ok(dir) = std::env::var("ETHCLI_CONFIG_DIR") {
            return PathBuf::from(dir).join(MANIFEST_FILE);
        }
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("ethcli")
            .join(MANIFEST_FILE)
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        match std::fs::read_to_string(path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|e| anyhow::anyhow!("cannot parse {}: {e}", path.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(anyhow::anyhow!("cannot read {}: {e}", path.display())),
        }
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let text = serde_json::to_string_pretty(self)?;
        write_atomic(path, text.as_bytes())
    }

    fn get(&self, path: &Path) -> Option<&ManifestEntry> {
        self.entries.iter().find(|e| e.path == path)
    }

    fn record(&mut self, entry: ManifestEntry) {
        self.entries.retain(|e| e.path != entry.path);
        self.entries.push(entry);
        self.entries.sort_by(|a, b| a.path.cmp(&b.path));
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Write to a temporary file in the same folder, then rename over the target.
pub fn write_atomic(path: &Path, data: &[u8]) -> anyhow::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("no parent folder for {}", path.display()))?;
    std::fs::create_dir_all(dir)?;
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.subsec_nanos())
        .unwrap_or(0);
    let tmp = dir.join(format!(".{name}.tmp-{}-{nanos}", std::process::id()));
    // create_new: never reuse or follow an existing entry at the temp path
    write_new(&tmp, data)?;
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(e.into());
    }
    Ok(())
}

/// Create `path` exclusively (O_EXCL: fails if anything, including a
/// symlink, is already there) and write `data`.
fn write_new(path: &Path, data: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    f.write_all(data)?;
    f.sync_all()
}

fn is_symlink(path: &Path) -> bool {
    std::fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink())
}

/// Refuse when any component of `path` below `anchor` (the anchor itself is
/// trusted) is a symlink, so a planted link cannot redirect a write.
fn check_no_symlinks(anchor: &Path, path: &Path) -> anyhow::Result<()> {
    let rel = path
        .strip_prefix(anchor)
        .map_err(|_| anyhow::anyhow!("{} is not under {}", path.display(), anchor.display()))?;
    let mut cur = anchor.to_path_buf();
    for c in rel.components() {
        cur.push(c);
        if is_symlink(&cur) {
            anyhow::bail!(
                "refusing to write through a symlink: {} (remove it or install elsewhere)",
                cur.display()
            );
        }
    }
    Ok(())
}

/// After writing, the real path must still be under the real root.
fn check_contained(root: &Path, path: &Path) -> anyhow::Result<()> {
    let real_root = std::fs::canonicalize(root)?;
    let real = std::fs::canonicalize(path)?;
    if !real.starts_with(&real_root) {
        anyhow::bail!(
            "{} resolves outside {}; refusing",
            path.display(),
            root.display()
        );
    }
    Ok(())
}

/// The skills folder for a target.
///
/// Matches the `npx skills` CLI: Claude Code uses `$CLAUDE_CONFIG_DIR/skills`
/// (default `~/.claude/skills`) globally and `.claude/skills` in a project;
/// Codex reads the shared `~/.agents/skills` globally and `.agents/skills` in a
/// project.
pub fn target_root(
    target: InstallTarget,
    dir: Option<&Path>,
    project: Option<&Path>,
) -> anyhow::Result<PathBuf> {
    let home = || dirs::home_dir().ok_or_else(|| anyhow::anyhow!("cannot find the home folder"));
    match (target, project) {
        (InstallTarget::Claude, Some(p)) => Ok(p.join(".claude").join("skills")),
        (InstallTarget::Codex, Some(p)) => Ok(p.join(".agents").join("skills")),
        (InstallTarget::Dir, Some(_)) => anyhow::bail!("--project does not apply to --target dir"),
        (InstallTarget::Claude, None) => {
            let base = match std::env::var("CLAUDE_CONFIG_DIR") {
                Ok(v) if !v.trim().is_empty() => PathBuf::from(v.trim()),
                _ => home()?.join(".claude"),
            };
            Ok(base.join("skills"))
        }
        (InstallTarget::Codex, None) => Ok(home()?.join(".agents").join("skills")),
        (InstallTarget::Dir, None) => dir
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow::anyhow!("--target dir needs --dir <path>")),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallOutcome {
    Written,
    Unchanged,
    /// Replaced a file ethcli did not write; the old one was kept here.
    Replaced {
        backup: PathBuf,
    },
    /// Not written: the destination was not written by ethcli.
    Refused,
}

#[derive(Debug, Clone, Serialize)]
pub struct InstallResult {
    pub skill: String,
    pub path: PathBuf,
    pub outcome: InstallOutcome,
}

/// Install skills under `root`. Pure apart from the file system; `manifest`
/// is updated in memory. No path component below `anchor` (the project
/// folder for a project install, else `root` itself) may be a symlink.
pub fn install(
    root: &Path,
    anchor: &Path,
    names: &[String],
    force: bool,
    manifest: &mut Manifest,
) -> anyhow::Result<Vec<InstallResult>> {
    let skills = embedded_skills();
    for n in names {
        if !skills.iter().any(|s| s.name == n) {
            anyhow::bail!(
                "no embedded skill named `{n}` (have: {})",
                skills.iter().map(|s| s.name).collect::<Vec<_>>().join(", ")
            );
        }
    }
    let mut out = Vec::new();
    for skill in skills
        .iter()
        .filter(|s| names.is_empty() || names.iter().any(|n| n == s.name))
    {
        for file in &skill.files {
            let body = embedded_file(skill.name, file).unwrap_or_default();
            let path = root.join(skill.name).join(file);
            let new_hash = sha256_hex(body.as_bytes());
            check_no_symlinks(anchor, &path)?;
            let outcome = match std::fs::read(&path) {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    write_atomic(&path, body.as_bytes())?;
                    InstallOutcome::Written
                }
                Err(e) => anyhow::bail!("cannot read {}: {e}", path.display()),
                Ok(existing) => {
                    let current = sha256_hex(&existing);
                    let ours = manifest.get(&path).is_some_and(|e| e.sha256 == current);
                    if current == new_hash {
                        InstallOutcome::Unchanged
                    } else if ours {
                        write_atomic(&path, body.as_bytes())?;
                        InstallOutcome::Written
                    } else if force {
                        let backup = path.with_file_name(format!(
                            "{}.bak-{}",
                            path.file_name().unwrap_or_default().to_string_lossy(),
                            now_secs()
                        ));
                        check_no_symlinks(anchor, &backup)?;
                        write_new(&backup, &existing).map_err(|e| {
                            anyhow::anyhow!("cannot create backup {}: {e}", backup.display())
                        })?;
                        write_atomic(&path, body.as_bytes())?;
                        InstallOutcome::Replaced { backup }
                    } else {
                        InstallOutcome::Refused
                    }
                }
            };
            if outcome != InstallOutcome::Refused {
                check_no_symlinks(anchor, &path)?;
                check_contained(root, &path)?;
                manifest.record(ManifestEntry {
                    path: path.clone(),
                    skill: skill.name.to_string(),
                    file: file.to_string(),
                    version: SKILLS_VERSION.to_string(),
                    sha256: new_hash,
                    installed_at: now_secs(),
                });
            }
            out.push(InstallResult {
                skill: skill.name.to_string(),
                path,
                outcome,
            });
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryState {
    /// Matches what this binary ships.
    Current,
    /// Unchanged since ethcli wrote it, but this binary ships a newer version.
    Outdated,
    /// Changed since ethcli wrote it.
    Modified,
    /// The file is gone.
    Missing,
    /// This binary no longer ships that skill file.
    NotShipped,
    /// The file or its skill folder is now a symlink; never followed.
    Symlink,
}

pub fn entry_state(entry: &ManifestEntry) -> EntryState {
    let skill_dir = entry.path.parent();
    if is_symlink(&entry.path) || skill_dir.is_some_and(is_symlink) {
        return EntryState::Symlink;
    }
    let Ok(existing) = std::fs::read(&entry.path) else {
        return EntryState::Missing;
    };
    let current = sha256_hex(&existing);
    let Some(body) = embedded_file(&entry.skill, &entry.file) else {
        return EntryState::NotShipped;
    };
    if current == sha256_hex(body.as_bytes()) {
        EntryState::Current
    } else if current == entry.sha256 {
        EntryState::Outdated
    } else {
        EntryState::Modified
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateResult {
    pub path: PathBuf,
    pub skill: String,
    /// State before the update ran.
    pub state: EntryState,
    pub refreshed: bool,
}

/// Refresh every manifest entry that is unchanged since ethcli wrote it.
pub fn update(manifest: &mut Manifest) -> anyhow::Result<Vec<UpdateResult>> {
    let mut out = Vec::new();
    let entries = manifest.entries.clone();
    for entry in entries {
        let state = entry_state(&entry);
        let mut refreshed = false;
        if state == EntryState::Outdated {
            let body = embedded_file(&entry.skill, &entry.file).unwrap_or_default();
            let root = entry
                .path
                .parent()
                .and_then(Path::parent)
                .ok_or_else(|| anyhow::anyhow!("bad manifest path {}", entry.path.display()))?;
            check_no_symlinks(root, &entry.path)?;
            write_atomic(&entry.path, body.as_bytes())?;
            check_contained(root, &entry.path)?;
            manifest.record(ManifestEntry {
                version: SKILLS_VERSION.to_string(),
                sha256: sha256_hex(body.as_bytes()),
                installed_at: now_secs(),
                ..entry.clone()
            });
            refreshed = true;
        } else if state == EntryState::Current && entry.version != SKILLS_VERSION {
            manifest.record(ManifestEntry {
                version: SKILLS_VERSION.to_string(),
                ..entry.clone()
            });
        }
        out.push(UpdateResult {
            path: entry.path,
            skill: entry.skill,
            state,
            refreshed,
        });
    }
    Ok(out)
}

fn state_word(s: EntryState) -> &'static str {
    match s {
        EntryState::Current => "current",
        EntryState::Outdated => "outdated",
        EntryState::Modified => "modified",
        EntryState::Missing => "missing",
        EntryState::NotShipped => "not shipped",
        EntryState::Symlink => "symlink",
    }
}

pub fn handle(action: SkillsCommands) -> anyhow::Result<()> {
    match action {
        SkillsCommands::List { json } => {
            let skills = embedded_skills();
            if json {
                println!("{}", serde_json::to_string_pretty(&skills)?);
            } else {
                for s in &skills {
                    println!(
                        "{:<22} v{}  sha256:{}  {}",
                        s.name,
                        s.version,
                        &s.hash[..12],
                        s.files.join(", ")
                    );
                }
                println!("\nInstall with: ethcli skills install [--target claude|codex|dir]");
            }
        }
        SkillsCommands::Show { name, file } => match embedded_file(&name, &file) {
            Some(body) => print!("{body}"),
            None => {
                anyhow::bail!("no embedded skill file `{name}/{file}` (see `ethcli skills list`)")
            }
        },
        SkillsCommands::Install {
            target,
            project,
            dir,
            names,
            force,
        } => {
            if dir.is_some() && target != InstallTarget::Dir {
                anyhow::bail!("--dir is only used with --target dir");
            }
            let cwd = if project {
                Some(std::env::current_dir()?)
            } else {
                None
            };
            let root = target_root(target, dir.as_deref(), cwd.as_deref())?;
            let manifest_path = Manifest::default_path();
            let mut manifest = Manifest::load(&manifest_path)?;
            let anchor = cwd.clone().unwrap_or_else(|| root.clone());
            let results = install(&root, &anchor, &names, force, &mut manifest)?;
            manifest.save(&manifest_path)?;
            let mut refused = 0;
            for r in &results {
                match &r.outcome {
                    InstallOutcome::Written => println!("wrote      {}", r.path.display()),
                    InstallOutcome::Unchanged => println!("up to date {}", r.path.display()),
                    InstallOutcome::Replaced { backup } => println!(
                        "replaced   {} (previous file kept at {})",
                        r.path.display(),
                        backup.display()
                    ),
                    InstallOutcome::Refused => {
                        refused += 1;
                        eprintln!(
                            "refused    {}: it exists and was not written by ethcli (customised or hand-written). Re-run with --force to replace it; a backup is kept.",
                            r.path.display()
                        );
                    }
                }
            }
            if refused > 0 {
                anyhow::bail!("{refused} file(s) not installed; see above");
            }
        }
        SkillsCommands::Update => {
            let manifest_path = Manifest::default_path();
            let mut manifest = Manifest::load(&manifest_path)?;
            if manifest.entries.is_empty() {
                println!("No installed skills recorded; run `ethcli skills install`.");
                return Ok(());
            }
            let results = update(&mut manifest)?;
            manifest.save(&manifest_path)?;
            for r in &results {
                let what = match (r.refreshed, r.state) {
                    (true, _) => "refreshed".to_string(),
                    (false, EntryState::Current) => "current".to_string(),
                    (false, s) => format!("skipped ({})", state_word(s)),
                };
                println!("{what:<22} {}", r.path.display());
            }
        }
        SkillsCommands::Status { json } => {
            let manifest = Manifest::load(&Manifest::default_path())?;
            let rows: Vec<_> = manifest
                .entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "path": e.path,
                        "skill": e.skill,
                        "file": e.file,
                        "installed_version": e.version,
                        "state": entry_state(e),
                    })
                })
                .collect();
            if json {
                println!("{}", serde_json::to_string_pretty(&rows)?);
            } else if manifest.entries.is_empty() {
                println!("No installed skills recorded.");
            } else {
                for e in &manifest.entries {
                    println!(
                        "{:<10} v{:<8} {}",
                        state_word(entry_state(e)),
                        e.version,
                        e.path.display()
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names() -> Vec<String> {
        vec![]
    }

    #[test]
    fn embeds_both_skills() {
        let skills = embedded_skills();
        let n: Vec<_> = skills.iter().map(|s| s.name).collect();
        assert!(n.contains(&"ethcli"));
        assert!(n.contains(&"bytecode-to-source"));
        assert!(embedded_file("ethcli", "SKILL.md")
            .unwrap()
            .contains("name: ethcli"));
    }

    /// The crate's copy must match the workspace `skills/` folder.
    #[test]
    fn crate_copy_matches_workspace_skills() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../skills");
        if !root.is_dir() {
            return; // building from the packaged crate
        }
        let mut seen = 0;
        for (skill, file, body) in EMBEDDED {
            let p = root.join(skill).join(file);
            let disk = std::fs::read_to_string(&p).unwrap_or_default();
            assert_eq!(
                &disk,
                body,
                "crates/ethcli/skills/{skill}/{file} differs from skills/{skill}/{file}; copy skills/ into crates/ethcli/skills/"
            );
            seen += 1;
        }
        let mut on_disk = 0;
        for d in std::fs::read_dir(&root).unwrap().flatten() {
            if d.path().is_dir() {
                on_disk += std::fs::read_dir(d.path()).unwrap().count();
            }
        }
        assert_eq!(
            seen, on_disk,
            "skills/ has files not copied into crates/ethcli/skills/"
        );
    }

    #[test]
    fn install_update_and_safety() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().join("skills");
        let mut m = Manifest::default();

        let r = install(&root, &root, &names(), false, &mut m).unwrap();
        assert!(r.iter().all(|r| r.outcome == InstallOutcome::Written));
        let skill_md = root.join("ethcli/SKILL.md");
        assert!(skill_md.is_file());
        assert_eq!(m.entries.len(), r.len());

        // Re-install: unchanged
        let r = install(&root, &root, &names(), false, &mut m).unwrap();
        assert!(r.iter().all(|r| r.outcome == InstallOutcome::Unchanged));

        // Manifest round trip
        let mpath = tmp.path().join("m.json");
        m.save(&mpath).unwrap();
        assert_eq!(Manifest::load(&mpath).unwrap(), m);

        // User modifies a file: refused without --force, status says modified
        std::fs::write(&skill_md, "my own notes").unwrap();
        let e = m.get(&skill_md).unwrap().clone();
        assert_eq!(entry_state(&e), EntryState::Modified);
        let r = install(&root, &root, &["ethcli".into()], false, &mut m).unwrap();
        assert_eq!(r[0].outcome, InstallOutcome::Refused);
        assert_eq!(std::fs::read_to_string(&skill_md).unwrap(), "my own notes");

        // Update skips modified
        let u = update(&mut m).unwrap();
        let row = u.iter().find(|u| u.path == skill_md).unwrap();
        assert!(!row.refreshed);
        assert_eq!(row.state, EntryState::Modified);
        assert_eq!(std::fs::read_to_string(&skill_md).unwrap(), "my own notes");

        // --force replaces and keeps a backup
        let r = install(&root, &root, &["ethcli".into()], true, &mut m).unwrap();
        let InstallOutcome::Replaced { backup } = &r[0].outcome else {
            panic!("expected Replaced, got {:?}", r[0].outcome)
        };
        assert_eq!(std::fs::read_to_string(backup).unwrap(), "my own notes");
        assert!(std::fs::read_to_string(&skill_md)
            .unwrap()
            .contains("name: ethcli"));
    }

    #[test]
    fn hand_written_skill_is_not_replaced() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let p = root.join("bytecode-to-source/SKILL.md");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, "custom").unwrap();
        let mut m = Manifest::default();
        let r = install(root, root, &["bytecode-to-source".into()], false, &mut m).unwrap();
        assert_eq!(r[0].outcome, InstallOutcome::Refused);
        assert!(m.entries.is_empty());
        assert_eq!(std::fs::read_to_string(&p).unwrap(), "custom");
    }

    #[test]
    fn update_refreshes_unchanged_old_version() {
        let tmp = tempfile::tempdir().unwrap();
        let p = tmp.path().join("ethcli/SKILL.md");
        let old = "old skill text from an earlier release";
        write_atomic(&p, old.as_bytes()).unwrap();
        let mut m = Manifest::default();
        m.record(ManifestEntry {
            path: p.clone(),
            skill: "ethcli".into(),
            file: "SKILL.md".into(),
            version: "0.0.1".into(),
            sha256: sha256_hex(old.as_bytes()),
            installed_at: 0,
        });
        assert_eq!(entry_state(&m.entries[0]), EntryState::Outdated);
        let u = update(&mut m).unwrap();
        assert!(u[0].refreshed);
        assert_eq!(
            std::fs::read_to_string(&p).unwrap(),
            embedded_file("ethcli", "SKILL.md").unwrap()
        );
        assert_eq!(m.entries[0].version, SKILLS_VERSION);
        assert_eq!(entry_state(&m.entries[0]), EntryState::Current);

        // Missing file is reported, not recreated
        std::fs::remove_file(&p).unwrap();
        let u = update(&mut m).unwrap();
        assert_eq!(u[0].state, EntryState::Missing);
        assert!(!p.exists());
    }

    #[test]
    fn computed_hash_matches_skills_cli_vectors() {
        assert_eq!(
            computed_hash(&[]),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            computed_hash(&[("SKILL.md", b"# hi\n")]),
            "93c36952d8843b33e68ae970ebabcc03e1687c9bb0dd1ecaf357060318952f55"
        );
        assert_eq!(
            computed_hash(&[("docs/b.md", b"b\n"), ("SKILL.md", b"a\n")]),
            "b22c4d22aa319d2924312fa18fcbfedc88fa89c74dbd8f298c683e1caf7b7f24"
        );
        // CRLF is normalised in text files only
        assert_eq!(
            computed_hash(&[("SKILL.md", b"# hi\r\n")]),
            computed_hash(&[("SKILL.md", b"# hi\n")])
        );
        assert_ne!(
            computed_hash(&[("x.bin", b"\0\r\n")]),
            computed_hash(&[("x.bin", b"\0\n")])
        );
    }

    #[test]
    fn target_locations_match_skills_cli() {
        let p = Path::new("/proj");
        assert_eq!(
            target_root(InstallTarget::Claude, None, Some(p)).unwrap(),
            p.join(".claude/skills")
        );
        assert_eq!(
            target_root(InstallTarget::Codex, None, Some(p)).unwrap(),
            p.join(".agents/skills")
        );
        assert_eq!(
            target_root(InstallTarget::Dir, Some(Path::new("/x")), None).unwrap(),
            PathBuf::from("/x")
        );
        assert!(target_root(InstallTarget::Dir, None, None).is_err());
    }

    #[cfg(unix)]
    mod symlinks {
        use super::super::*;
        use std::os::unix::fs::symlink;

        fn setup() -> (tempfile::TempDir, PathBuf, PathBuf) {
            let tmp = tempfile::tempdir().unwrap();
            let project = tmp.path().join("proj");
            let outside = tmp.path().join("outside");
            std::fs::create_dir_all(&project).unwrap();
            std::fs::create_dir_all(&outside).unwrap();
            (tmp, project, outside)
        }

        fn empty_dir(p: &Path) -> bool {
            std::fs::read_dir(p).unwrap().next().is_none()
        }

        #[test]
        fn symlinked_parent_claude_dir_is_refused() {
            let (_t, project, outside) = setup();
            symlink(&outside, project.join(".claude")).unwrap();
            let root = project.join(".claude/skills");
            let mut m = Manifest::default();
            assert!(install(&root, &project, &[], false, &mut m).is_err());
            assert!(empty_dir(&outside));
            assert!(m.entries.is_empty());
        }

        #[test]
        fn symlinked_skill_dir_is_refused() {
            let (_t, project, outside) = setup();
            let root = project.join(".claude/skills");
            std::fs::create_dir_all(&root).unwrap();
            symlink(&outside, root.join("ethcli")).unwrap();
            let mut m = Manifest::default();
            assert!(install(&root, &project, &["ethcli".into()], true, &mut m).is_err());
            assert!(empty_dir(&outside));
        }

        #[test]
        fn symlinked_skill_file_is_refused_even_with_force() {
            let (_t, project, outside) = setup();
            let root = project.join(".claude/skills");
            std::fs::create_dir_all(root.join("ethcli")).unwrap();
            let target = outside.join("victim");
            std::fs::write(&target, "secret").unwrap();
            symlink(&target, root.join("ethcli/SKILL.md")).unwrap();
            let mut m = Manifest::default();
            assert!(install(&root, &project, &["ethcli".into()], true, &mut m).is_err());
            assert_eq!(std::fs::read_to_string(&target).unwrap(), "secret");
            assert_eq!(std::fs::read_dir(&outside).unwrap().count(), 1);
        }

        #[test]
        fn precreated_backup_symlink_is_not_followed() {
            let (_t, project, outside) = setup();
            let root = project.join(".claude/skills");
            let skill_md = root.join("ethcli/SKILL.md");
            std::fs::create_dir_all(skill_md.parent().unwrap()).unwrap();
            std::fs::write(&skill_md, "custom").unwrap();
            let target = outside.join("victim");
            // Plant links at every backup name the next seconds could produce
            let now = now_secs();
            for t in now..now + 5 {
                symlink(&target, root.join(format!("ethcli/SKILL.md.bak-{t}"))).unwrap();
            }
            let mut m = Manifest::default();
            assert!(install(&root, &project, &["ethcli".into()], true, &mut m).is_err());
            assert!(!target.exists());
            assert_eq!(std::fs::read_to_string(&skill_md).unwrap(), "custom");
        }

        #[test]
        fn update_skips_entry_that_became_a_symlink() {
            let (_t, project, outside) = setup();
            let root = project.join(".claude/skills");
            let mut m = Manifest::default();
            install(&root, &project, &["ethcli".into()], false, &mut m).unwrap();
            let p = root.join("ethcli/SKILL.md");
            m.entries[0].sha256 = sha256_hex(b"old");
            let target = outside.join("victim");
            std::fs::write(&target, "old").unwrap();
            std::fs::remove_file(&p).unwrap();
            symlink(&target, &p).unwrap();
            let u = update(&mut m).unwrap();
            assert_eq!(u[0].state, EntryState::Symlink);
            assert!(!u[0].refreshed);
            assert_eq!(std::fs::read_to_string(&target).unwrap(), "old");

            std::fs::remove_file(&p).unwrap();
            std::fs::remove_dir(root.join("ethcli")).unwrap();
            symlink(&outside, root.join("ethcli")).unwrap();
            let u = update(&mut m).unwrap();
            assert_eq!(u[0].state, EntryState::Symlink);
        }
    }

    #[test]
    fn unknown_skill_name_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let mut m = Manifest::default();
        assert!(install(tmp.path(), tmp.path(), &["nope".into()], false, &mut m).is_err());
    }
}
