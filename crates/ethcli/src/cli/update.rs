//! Update command - check for and install updates from GitHub releases

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

struct ExtractedBinaries {
    ethcli: PathBuf,
    ethcli_mcp: Option<PathBuf>,
}

const REPO: &str = "yldfi/yldfi-rs";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const RELEASE_TAG_PREFIX: &str = "ethcli-v";

/// Check for updates and optionally install them
pub async fn handle(install: bool, quiet: bool) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Checking for updates...");
    }

    // Fetch releases from GitHub (not /latest, as that returns any crate's release)
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .pool_max_idle_per_host(2)
        .pool_idle_timeout(std::time::Duration::from_secs(60))
        .build()?;
    let url = format!("https://api.github.com/repos/{}/releases?per_page=50", REPO);
    let response = client
        .get(&url)
        .header("User-Agent", "ethcli")
        .send()
        .await?;

    if !response.status().is_success() {
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            anyhow::bail!(
                "Could not fetch release info. The repository may be private.\n\
                 Check manually: https://github.com/{}/releases",
                REPO
            );
        }
        anyhow::bail!("Failed to check for updates: {}", response.status());
    }

    // Filter for ethcli-specific releases (tag_name starts with "ethcli-v")
    let releases: Vec<GitHubRelease> = response.json().await?;
    let release = releases
        .into_iter()
        .find(|r| r.tag_name.starts_with(RELEASE_TAG_PREFIX))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No ethcli releases found. Check manually: https://github.com/{}/releases",
                REPO
            )
        })?;
    let latest_version_str = release.tag_name.trim_start_matches(RELEASE_TAG_PREFIX);

    println!("Current version: v{}", CURRENT_VERSION);
    println!("Latest version:  {}", release.tag_name);

    // Use semver for proper version comparison
    let current = Version::parse(CURRENT_VERSION)
        .map_err(|e| anyhow::anyhow!("Failed to parse current version: {}", e))?;
    let latest = Version::parse(latest_version_str)
        .map_err(|e| anyhow::anyhow!("Failed to parse latest version: {}", e))?;

    if current >= latest {
        println!("\n✓ You're on the latest version!");
        return Ok(());
    }

    // Determine which asset to download based on OS and arch
    let asset_name = get_asset_name_for_platform();
    let asset = release.assets.iter().find(|a| a.name == asset_name);

    if !install {
        println!("\nUpdate available!");
        println!("Download from: {}", release.html_url);
        if asset.is_some() {
            println!("\nOr run: ethcli upgrade");
        }
        return Ok(());
    }

    // Install the update
    let asset = asset.ok_or_else(|| {
        anyhow::anyhow!(
            "No binary available for your platform ({}). Download manually from: {}",
            asset_name,
            release.html_url
        )
    })?;

    if !quiet {
        eprintln!("Downloading {}...", asset.name);
    }

    // Download the asset (use longer timeout for large binary downloads)
    let download_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
        .build()?;
    let response = download_client
        .get(&asset.browser_download_url)
        .header("User-Agent", "ethcli")
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to download update: {}", response.status());
    }

    let bytes = response.bytes().await?;

    // Try to verify checksum if a .sha256 file is available
    let checksum_asset_name = format!("{}.sha256", asset.name);
    if let Some(checksum_asset) = release
        .assets
        .iter()
        .find(|a| a.name == checksum_asset_name)
    {
        if !quiet {
            eprintln!("Verifying checksum...");
        }

        let checksum_response = download_client
            .get(&checksum_asset.browser_download_url)
            .header("User-Agent", "ethcli")
            .send()
            .await?;

        if checksum_response.status().is_success() {
            let checksum_text = checksum_response.text().await?;
            // Checksum file format: "<hash>  <filename>" or just "<hash>"
            let expected_hash = checksum_text
                .split_whitespace()
                .next()
                .unwrap_or(&checksum_text)
                .trim()
                .to_lowercase();

            // Compute actual hash
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let actual_hash = hex::encode(hasher.finalize());

            if actual_hash != expected_hash {
                anyhow::bail!(
                    "Checksum verification failed!\nExpected: {}\nActual:   {}\n\nThe downloaded file may be corrupted or tampered with.",
                    expected_hash,
                    actual_hash
                );
            }

            if !quiet {
                eprintln!("Checksum verified.");
            }
        }
    } else {
        // Require checksum verification for security - abort if no checksum available
        anyhow::bail!(
            "Security: No checksum file ({}) available for verification.\n\
             Cannot safely install update without integrity verification.\n\
             Please download manually from: {}",
            checksum_asset_name,
            release.html_url
        );
    }

    // Extract and install using unique temp directory to prevent race conditions
    let random_suffix: u64 = rand::random();
    let temp_dir = std::env::temp_dir().join(format!("ethcli-update-{:016x}", random_suffix));
    std::fs::create_dir_all(&temp_dir)?;

    let archive_path = temp_dir.join(&asset.name);
    std::fs::write(&archive_path, &bytes)?;

    // Extract based on file type
    // SEC-UPDATE-001: Use --strip-components to prevent path traversal attacks.
    // Archives could contain paths like "../../../.bashrc" that would write outside temp_dir.
    // By stripping the first component and only extracting the expected binary name,
    // we prevent malicious archives from overwriting arbitrary files.
    let extracted = if asset.name.ends_with(".tar.gz") {
        ExtractedBinaries {
            ethcli: extract_tar_binary(&archive_path, &temp_dir, "ethcli", true)?.ok_or_else(
                || anyhow::anyhow!("Required binary 'ethcli' was not extracted from archive"),
            )?,
            ethcli_mcp: extract_tar_binary(&archive_path, &temp_dir, "ethcli-mcp", false)?,
        }
    } else if asset.name.ends_with(".zip") {
        ExtractedBinaries {
            ethcli: extract_zip_binary(&archive_path, &temp_dir, "ethcli.exe", true)?.ok_or_else(
                || anyhow::anyhow!("Required binary 'ethcli.exe' was not extracted from archive"),
            )?,
            ethcli_mcp: extract_zip_binary(&archive_path, &temp_dir, "ethcli-mcp.exe", false)?,
        }
    } else {
        anyhow::bail!("Unknown archive format: {}", asset.name);
    };

    // Find the install location. Resolve symlinks so a symlinked layout keeps
    // its links and the real binary behind them is the one that gets replaced.
    let install_path = std::fs::canonicalize(std::env::current_exe()?)?;

    if !quiet {
        eprintln!("Installing ethcli to {}...", install_path.display());
    }

    // Decide where ethcli-mcp goes before touching anything.
    let mcp_target = match &extracted.ethcli_mcp {
        Some(_) => Some(resolve_mcp_target(&install_path)?),
        None => None,
    };

    // Keep the current binaries until the new ethcli has been seen to run, so
    // a build that can't execute here (wrong libc, wrong arch) is rolled back.
    let mut backups = Vec::new();
    backups.push(backup_binary(&install_path).map_err(|e| install_error(e, &install_path))?);
    if let Some(McpTarget::Install(target)) = &mcp_target {
        backups.push(backup_binary(target).map_err(|e| install_error(e, target))?);
    }

    let installed = (|| -> anyhow::Result<()> {
        install_binary(&extracted.ethcli, &install_path)
            .map_err(|e| install_error(e, &install_path))?;
        if let (Some(ethcli_mcp), Some(McpTarget::Install(target))) =
            (&extracted.ethcli_mcp, &mcp_target)
        {
            if !quiet {
                eprintln!("Installing ethcli-mcp to {}...", target.display());
            }
            install_binary(ethcli_mcp, target).map_err(|e| install_error(e, target))?;
        }
        verify_installed(&install_path, latest_version_str)
    })();

    if let Err(e) = installed {
        let restored = backups.iter().flatten().all(|b| b.restore().is_ok());
        let _ = std::fs::remove_dir_all(&temp_dir);
        anyhow::bail!(
            "{}{}",
            e,
            if restored {
                "\nThe previous binaries were restored."
            } else {
                "\nCould not restore every previous binary; check the paths above."
            }
        );
    }
    for b in backups.iter().flatten() {
        b.discard();
    }

    if let Some(McpTarget::Skip(reason)) = &mcp_target {
        eprintln!(
            "Warning: ethcli-mcp was not updated: {}. Update it manually.",
            reason
        );
    }

    // Cleanup
    let _ = std::fs::remove_dir_all(&temp_dir);

    println!("\n✓ Updated to {}!", release.tag_name);
    println!("  Restart your terminal or run: ethcli --version");

    Ok(())
}

fn get_asset_name_for_platform() -> String {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    match (os, arch) {
        ("macos", "aarch64") => "ethcli-macos-aarch64.tar.gz".to_string(),
        ("macos", "x86_64") => "ethcli-macos-x86_64.tar.gz".to_string(),
        ("linux", "x86_64") => linux_x86_64_asset_name(),
        ("linux", "aarch64") => "ethcli-linux-aarch64.tar.gz".to_string(),
        ("windows", "x86_64") => "ethcli-windows-x86_64.zip".to_string(),
        _ => format!("ethcli-{}-{}.tar.gz", os, arch),
    }
}

fn linux_x86_64_asset_name() -> String {
    if cfg!(target_env = "musl") {
        "ethcli-linux-x86_64-musl.tar.gz".to_string()
    } else {
        "ethcli-linux-x86_64.tar.gz".to_string()
    }
}

fn extract_tar_binary(
    archive_path: &Path,
    temp_dir: &Path,
    expected_binary: &str,
    required: bool,
) -> anyhow::Result<Option<PathBuf>> {
    // Extract tar.gz - only extract the expected binary, strip path components
    let output = std::process::Command::new("tar")
        .args([
            "-xzf",
            &archive_path.to_string_lossy(),
            "-C",
            &temp_dir.to_string_lossy(),
            "--strip-components=1", // Flatten any directory structure
            "--wildcards",
            &format!("*/{}", expected_binary), // Only extract the binary
        ])
        .output()?;

    // If wildcards extraction fails, try without (binary might be at root)
    if !output.status.success() {
        let output = std::process::Command::new("tar")
            .args([
                "-xzf",
                &archive_path.to_string_lossy(),
                "-C",
                &temp_dir.to_string_lossy(),
                expected_binary,
            ])
            .output()?;
        if !output.status.success() {
            if required {
                anyhow::bail!(
                    "Failed to extract archive: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            return Ok(None);
        }
    }

    verify_extracted_binary(temp_dir, expected_binary, required)
}

fn extract_zip_binary(
    archive_path: &Path,
    temp_dir: &Path,
    expected_binary: &str,
    required: bool,
) -> anyhow::Result<Option<PathBuf>> {
    // Extract zip - only extract the expected binary
    let output = std::process::Command::new("unzip")
        .args([
            "-o",
            "-j", // Junk paths - extract files without directory structure
            &archive_path.to_string_lossy(),
            expected_binary,
            "-d",
            &temp_dir.to_string_lossy(),
        ])
        .output()?;
    if !output.status.success() {
        if required {
            anyhow::bail!(
                "Failed to extract archive: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        return Ok(None);
    }

    verify_extracted_binary(temp_dir, expected_binary, required)
}

fn verify_extracted_binary(
    temp_dir: &Path,
    expected_binary: &str,
    required: bool,
) -> anyhow::Result<Option<PathBuf>> {
    let binary = temp_dir.join(expected_binary);
    // Verify the binary exists and is within temp_dir (defense in depth)
    let canonical = match binary.canonicalize() {
        Ok(path) => path,
        Err(_) if required => {
            anyhow::bail!("Binary '{}' not found in archive", expected_binary);
        }
        Err(_) => return Ok(None),
    };
    let temp_canonical = temp_dir.canonicalize()?;
    if !canonical.starts_with(&temp_canonical) {
        anyhow::bail!("Security: extracted path escapes temp directory");
    }
    Ok(Some(binary))
}

enum McpTarget {
    Install(PathBuf),
    Skip(String),
}

/// Decide where ethcli-mcp should be installed. A symlink is followed to the
/// file it points to, so the link itself is left in place. The target is
/// skipped when it isn't a native executable (e.g. a wrapper script, which
/// replacing would break), when it is the ethcli binary itself, or when it
/// belongs to another user.
fn resolve_mcp_install_path(mcp_path: &Path, ethcli_path: &Path) -> anyhow::Result<McpTarget> {
    if std::fs::symlink_metadata(mcp_path).is_err() {
        return Ok(McpTarget::Install(mcp_path.to_path_buf()));
    }
    let target = match std::fs::canonicalize(mcp_path) {
        Ok(target) => target,
        Err(_) => {
            return Ok(McpTarget::Skip(format!(
                "it is a symlink to a missing file ({})",
                std::fs::read_link(mcp_path)
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            )))
        }
    };
    if target == ethcli_path {
        return Ok(McpTarget::Skip(format!(
            "it resolves to the ethcli binary ({})",
            target.display()
        )));
    }
    if let Some(reason) = other_owner(&target, ethcli_path) {
        return Ok(McpTarget::Skip(reason));
    }
    if !is_native_executable(&target) {
        return Ok(McpTarget::Skip(format!(
            "it resolves to {}, which is not a native executable (a wrapper script?)",
            target.display()
        )));
    }
    Ok(McpTarget::Install(target))
}

/// Why `target` belongs to a different install than `ethcli_path`, if it does.
/// Upgrade only replaces binaries owned by the same user as the ethcli being
/// upgraded; another user's install is left for that user to upgrade.
#[cfg(unix)]
fn other_owner(target: &Path, ethcli_path: &Path) -> Option<String> {
    use std::os::unix::fs::MetadataExt;

    let target_uid = std::fs::metadata(target).ok()?.uid();
    let ethcli_uid = std::fs::metadata(ethcli_path).ok()?.uid();
    (target_uid != ethcli_uid).then(|| {
        format!(
            "it resolves to {}, which belongs to another user's install (uid {}); run `ethcli upgrade` as that user",
            target.display(),
            target_uid
        )
    })
}

#[cfg(not(unix))]
fn other_owner(_target: &Path, _ethcli_path: &Path) -> Option<String> {
    None
}

/// True if the file starts with an ELF, Mach-O or PE header.
fn is_native_executable(path: &Path) -> bool {
    use std::io::Read;

    let mut magic = [0u8; 4];
    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    if file.read_exact(&mut magic).is_err() {
        return false;
    }
    matches!(
        magic,
        [0x7f, b'E', b'L', b'F']
            | [0xfe, 0xed, 0xfa, 0xce]
            | [0xfe, 0xed, 0xfa, 0xcf]
            | [0xce, 0xfa, 0xed, 0xfe]
            | [0xcf, 0xfa, 0xed, 0xfe]
            | [0xca, 0xfe, 0xba, 0xbe]
    ) || magic[..2] == *b"MZ"
}

fn install_error(err: anyhow::Error, path: &Path) -> anyhow::Error {
    let denied = err
        .downcast_ref::<std::io::Error>()
        .is_some_and(|e| e.kind() == std::io::ErrorKind::PermissionDenied);
    if denied {
        anyhow::anyhow!(
            "Permission denied writing {}. Run `ethcli upgrade` as the user that owns it.",
            path.display()
        )
    } else {
        err.context(format!("Failed to install {}", path.display()))
    }
}

/// Candidate locations for ethcli-mcp next to the ethcli binary, most specific
/// first: the name that shares ethcli's suffix (`ethcli.real` ->
/// `ethcli-mcp.real`), then `ethcli-mcp.real`, then plain `ethcli-mcp`.
fn mcp_candidates(ethcli_path: &Path) -> Vec<PathBuf> {
    let exe = std::env::consts::EXE_SUFFIX;
    let dir = ethcli_path.parent().unwrap_or_else(|| Path::new("."));
    let name = ethcli_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let stem = name.strip_suffix(exe).unwrap_or(&name);
    let suffix = stem.strip_prefix("ethcli").unwrap_or("");
    let mut candidates = Vec::new();
    for s in [suffix, ".real", ""] {
        let p = dir.join(format!("ethcli-mcp{}{}", s, exe));
        if !candidates.contains(&p) {
            candidates.push(p);
        }
    }
    candidates
}

/// Pick the ethcli-mcp binary to replace: the first candidate that resolves
/// (through any symlinks) to a native executable other than ethcli itself.
/// Wrapper scripts are skipped so the real binary behind them is found. If
/// none of the candidates exists, install plain `ethcli-mcp` next to ethcli.
fn resolve_mcp_target(ethcli_path: &Path) -> anyhow::Result<McpTarget> {
    let candidates = mcp_candidates(ethcli_path);
    let mut skipped = Vec::new();
    for candidate in &candidates {
        if std::fs::symlink_metadata(candidate).is_err() {
            continue;
        }
        match resolve_mcp_install_path(candidate, ethcli_path)? {
            McpTarget::Install(target) => return Ok(McpTarget::Install(target)),
            McpTarget::Skip(reason) => skipped.push(format!("{}: {}", candidate.display(), reason)),
        }
    }
    if skipped.is_empty() {
        let plain = candidates
            .last()
            .cloned()
            .unwrap_or_else(|| ethcli_path.with_file_name("ethcli-mcp"));
        return Ok(McpTarget::Install(plain));
    }
    Ok(McpTarget::Skip(skipped.join("; ")))
}

/// A copy of a binary taken before it is replaced.
struct Backup {
    original: PathBuf,
    copy: PathBuf,
}

impl Backup {
    fn restore(&self) -> std::io::Result<()> {
        std::fs::rename(&self.copy, &self.original)
    }

    fn discard(&self) {
        let _ = std::fs::remove_file(&self.copy);
    }
}

/// Copy `path` aside (keeping its permissions) so it can be restored.
/// Returns `None` when there is nothing to back up yet.
fn backup_binary(path: &Path) -> anyhow::Result<Option<Backup>> {
    if !path.exists() {
        return Ok(None);
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid install path: {}", path.display()))?;
    let copy = path.with_file_name(format!(
        ".{}.prev-{}",
        file_name.to_string_lossy(),
        std::process::id()
    ));
    std::fs::copy(path, &copy)?;
    Ok(Some(Backup {
        original: path.to_path_buf(),
        copy,
    }))
}

/// Run the newly installed ethcli and check it reports the expected version.
fn verify_installed(ethcli_path: &Path, expected_version: &str) -> anyhow::Result<()> {
    let output = std::process::Command::new(ethcli_path)
        .arg("--version")
        .output()
        .map_err(|e| {
            anyhow::anyhow!(
                "The new ethcli at {} could not be started: {}",
                ethcli_path.display(),
                e
            )
        })?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() || !stdout.contains(expected_version) {
        anyhow::bail!(
            "The new ethcli at {} did not report version {} (got: {})",
            ethcli_path.display(),
            expected_version,
            stdout.trim()
        );
    }
    Ok(())
}

fn install_binary(source: &Path, destination: &Path) -> anyhow::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Copy to a temp file next to the destination, then rename it into
        // place. Overwriting the existing file in place keeps its inode, and on
        // macOS the kernel's cached code signature for that inode no longer
        // matches the new contents, so the binary is SIGKILLed on launch.
        // A rename gives the new binary a fresh inode (and is atomic).
        let file_name = destination
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid install path: {}", destination.display()))?;
        let tmp_path = destination.with_file_name(format!(
            ".{}.new-{}",
            file_name.to_string_lossy(),
            std::process::id()
        ));
        let result = (|| -> anyhow::Result<()> {
            std::fs::copy(source, &tmp_path)?;
            std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
            std::fs::rename(&tmp_path, destination)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp_path);
        }
        result?;
    }

    #[cfg(windows)]
    {
        // On Windows, rename the running exe and copy new one.
        let backup_path = destination.with_extension("old.exe");
        if destination.exists() {
            std::fs::rename(destination, &backup_path)?;
        }
        std::fs::copy(source, destination)?;
        // Note: old exe will be cleaned up on next run or reboot.
    }

    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::{
        backup_binary, install_binary, mcp_candidates, resolve_mcp_install_path,
        resolve_mcp_target, verify_installed, McpTarget,
    };
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    #[test]
    fn install_replaces_destination_with_new_inode() {
        let dir = std::env::temp_dir().join(format!("ethcli-install-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("source");
        let destination = dir.join("ethcli-mcp");
        std::fs::write(&source, b"new binary").unwrap();
        std::fs::write(&destination, b"old binary").unwrap();
        let old_inode = std::fs::metadata(&destination).unwrap().ino();

        install_binary(&source, &destination).unwrap();

        let meta = std::fs::metadata(&destination).unwrap();
        assert_ne!(meta.ino(), old_inode, "must not overwrite in place");
        assert_eq!(std::fs::read(&destination).unwrap(), b"new binary");
        assert_eq!(meta.permissions().mode() & 0o777, 0o755);
        let leftovers: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".new-"))
            .collect();
        assert!(leftovers.is_empty(), "temp file left behind");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    fn test_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("ethcli-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    const ELF: &[u8] = b"\x7fELF rest of binary";

    #[test]
    fn mcp_missing_installs_next_to_ethcli() {
        let dir = test_dir("mcp-missing");
        let ethcli = dir.join("ethcli");
        std::fs::write(&ethcli, ELF).unwrap();
        let mcp = dir.join("ethcli-mcp");
        match resolve_mcp_install_path(&mcp, &ethcli).unwrap() {
            McpTarget::Install(p) => assert_eq!(p, mcp),
            McpTarget::Skip(r) => panic!("unexpected skip: {r}"),
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mcp_symlink_to_binary_updates_target_and_keeps_link() {
        let dir = test_dir("mcp-link").canonicalize().unwrap();
        let ethcli = dir.join("ethcli");
        std::fs::write(&ethcli, ELF).unwrap();
        let real = dir.join("ethcli-mcp.real");
        std::fs::write(&real, ELF).unwrap();
        let link = dir.join("ethcli-mcp");
        std::os::unix::fs::symlink("ethcli-mcp.real", &link).unwrap();

        let McpTarget::Install(target) = resolve_mcp_install_path(&link, &ethcli).unwrap() else {
            panic!("expected install");
        };
        assert_eq!(target, real);
        let source = dir.join("new");
        std::fs::write(&source, b"\x7fELF new").unwrap();
        install_binary(&source, &target).unwrap();

        assert!(std::fs::symlink_metadata(&link)
            .unwrap()
            .file_type()
            .is_symlink());
        assert_eq!(std::fs::read(&real).unwrap(), b"\x7fELF new");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mcp_symlink_to_wrapper_script_is_skipped() {
        let dir = test_dir("mcp-wrapper");
        let ethcli = dir.join("ethcli.real");
        std::fs::write(&ethcli, ELF).unwrap();
        let wrapper = dir.join("wrapper");
        std::fs::write(&wrapper, b"#!/bin/sh\nexec ethcli.real \"$@\"\n").unwrap();
        let link = dir.join("ethcli-mcp");
        std::os::unix::fs::symlink("wrapper", &link).unwrap();

        assert!(matches!(
            resolve_mcp_install_path(&link, &ethcli).unwrap(),
            McpTarget::Skip(_)
        ));
        assert!(std::fs::read(&wrapper).unwrap().starts_with(b"#!"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mcp_symlink_to_ethcli_or_missing_is_skipped() {
        let dir = test_dir("mcp-self").canonicalize().unwrap();
        let ethcli = dir.join("ethcli");
        std::fs::write(&ethcli, ELF).unwrap();
        let link = dir.join("ethcli-mcp");
        std::os::unix::fs::symlink("ethcli", &link).unwrap();
        assert!(matches!(
            resolve_mcp_install_path(&link, &ethcli).unwrap(),
            McpTarget::Skip(_)
        ));

        let dangling = dir.join("ethcli-mcp-dangling");
        std::os::unix::fs::symlink("missing", &dangling).unwrap();
        assert!(matches!(
            resolve_mcp_install_path(&dangling, &ethcli).unwrap(),
            McpTarget::Skip(_)
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mcp_candidates_prefer_matching_suffix() {
        let names = |p: &str| -> Vec<String> {
            mcp_candidates(std::path::Path::new(p))
                .iter()
                .map(|c| c.file_name().unwrap().to_string_lossy().into_owned())
                .collect()
        };
        assert_eq!(names("/x/ethcli"), ["ethcli-mcp", "ethcli-mcp.real"]);
        assert_eq!(names("/x/ethcli.real"), ["ethcli-mcp.real", "ethcli-mcp"]);
        assert_eq!(
            names("/x/ethcli.v2"),
            ["ethcli-mcp.v2", "ethcli-mcp.real", "ethcli-mcp"]
        );
    }

    #[test]
    fn plain_layout_updates_adjacent_mcp() {
        let dir = test_dir("plain").canonicalize().unwrap();
        let ethcli = dir.join("ethcli");
        std::fs::write(&ethcli, ELF).unwrap();
        std::fs::write(dir.join("ethcli-mcp"), ELF).unwrap();
        let McpTarget::Install(t) = resolve_mcp_target(&ethcli).unwrap() else {
            panic!()
        };
        assert_eq!(t, dir.join("ethcli-mcp"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn wrapper_layout_updates_real_binaries_not_wrapper() {
        // ethcli and ethcli-mcp both symlink to one wrapper script that runs
        // ethcli.real / ethcli-mcp.real. Upgrade runs as ethcli.real.
        let dir = test_dir("wrapper-layout").canonicalize().unwrap();
        let wrapper = dir.join("ethcli-op-wrapper");
        std::fs::write(&wrapper, b"#!/bin/bash\nexec ./ethcli.real \"$@\"\n").unwrap();
        std::os::unix::fs::symlink("ethcli-op-wrapper", dir.join("ethcli")).unwrap();
        std::os::unix::fs::symlink("ethcli-op-wrapper", dir.join("ethcli-mcp")).unwrap();
        let ethcli_real = dir.join("ethcli.real");
        std::fs::write(&ethcli_real, ELF).unwrap();
        std::fs::write(dir.join("ethcli-mcp.real"), ELF).unwrap();

        let McpTarget::Install(t) = resolve_mcp_target(&ethcli_real).unwrap() else {
            panic!()
        };
        assert_eq!(t, dir.join("ethcli-mcp.real"));
        assert!(std::fs::read(&wrapper).unwrap().starts_with(b"#!"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn symlink_into_another_directory_updates_target() {
        let dir = test_dir("cross-dir").canonicalize().unwrap();
        let other = dir.join("other");
        std::fs::create_dir_all(&other).unwrap();
        let real_mcp = other.join("ethcli-mcp");
        std::fs::write(&real_mcp, ELF).unwrap();
        let ethcli = dir.join("ethcli");
        std::fs::write(&ethcli, ELF).unwrap();
        std::os::unix::fs::symlink(&real_mcp, dir.join("ethcli-mcp")).unwrap();
        let McpTarget::Install(t) = resolve_mcp_target(&ethcli).unwrap() else {
            panic!()
        };
        assert_eq!(t, real_mcp);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn no_mcp_yet_installs_plain_name() {
        let dir = test_dir("no-mcp").canonicalize().unwrap();
        let ethcli = dir.join("ethcli.real");
        std::fs::write(&ethcli, ELF).unwrap();
        let McpTarget::Install(t) = resolve_mcp_target(&ethcli).unwrap() else {
            panic!()
        };
        assert_eq!(t, dir.join("ethcli-mcp"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn only_wrapper_and_no_real_binary_is_skipped() {
        let dir = test_dir("wrapper-only").canonicalize().unwrap();
        let ethcli = dir.join("ethcli.real");
        std::fs::write(&ethcli, ELF).unwrap();
        std::fs::write(dir.join("ethcli-mcp"), b"#!/bin/sh\n").unwrap();
        assert!(matches!(
            resolve_mcp_target(&ethcli).unwrap(),
            McpTarget::Skip(_)
        ));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn backup_restores_previous_binary() {
        let dir = test_dir("backup").canonicalize().unwrap();
        let bin = dir.join("ethcli");
        std::fs::write(&bin, b"old").unwrap();
        let backup = backup_binary(&bin).unwrap().expect("backup");
        std::fs::write(&bin, b"new").unwrap();
        backup.restore().unwrap();
        assert_eq!(std::fs::read(&bin).unwrap(), b"old");
        assert!(backup_binary(&dir.join("missing")).unwrap().is_none());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn verify_rejects_binary_that_cannot_run() {
        let dir = test_dir("verify").canonicalize().unwrap();
        let bin = dir.join("ethcli");
        std::fs::write(&bin, b"\x7fELF not really").unwrap();
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        assert!(verify_installed(&bin, "0.0.0").is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
