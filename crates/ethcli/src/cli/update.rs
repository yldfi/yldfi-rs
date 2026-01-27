//! Update command - check for and install updates from GitHub releases

use semver::Version;
use serde::Deserialize;
use sha2::{Digest, Sha256};

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
            println!("\nOr run: ethcli update --install");
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

    // Download the asset
    let response = client
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

        let checksum_response = client
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
            let actual_hash = format!("{:x}", hasher.finalize());

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
    let binary_path = if asset.name.ends_with(".tar.gz") {
        // Extract tar.gz - only extract the expected binary, strip path components
        let expected_binary = "ethcli";
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
                anyhow::bail!(
                    "Failed to extract archive: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        let binary = temp_dir.join(expected_binary);
        // Verify the binary exists and is within temp_dir (defense in depth)
        let canonical = binary.canonicalize().map_err(|_| {
            anyhow::anyhow!("Binary '{}' not found in archive", expected_binary)
        })?;
        let temp_canonical = temp_dir.canonicalize()?;
        if !canonical.starts_with(&temp_canonical) {
            anyhow::bail!("Security: extracted path escapes temp directory");
        }
        binary
    } else if asset.name.ends_with(".zip") {
        // Extract zip - only extract the expected binary
        let expected_binary = "ethcli.exe";
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
            anyhow::bail!(
                "Failed to extract archive: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        let binary = temp_dir.join(expected_binary);
        // Verify the binary exists and is within temp_dir (defense in depth)
        let canonical = binary.canonicalize().map_err(|_| {
            anyhow::anyhow!("Binary '{}' not found in archive", expected_binary)
        })?;
        let temp_canonical = temp_dir.canonicalize()?;
        if !canonical.starts_with(&temp_canonical) {
            anyhow::bail!("Security: extracted path escapes temp directory");
        }
        binary
    } else {
        anyhow::bail!("Unknown archive format: {}", asset.name);
    };

    // Find the install location
    let install_path = std::env::current_exe()?;

    if !quiet {
        eprintln!("Installing to {}...", install_path.display());
    }

    // On Unix, we need to handle the case where we can't overwrite a running binary
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        // Make the new binary executable
        std::fs::set_permissions(&binary_path, std::fs::Permissions::from_mode(0o755))?;

        // Try direct copy first, fall back to rename trick if needed
        if std::fs::copy(&binary_path, &install_path).is_err() {
            // Rename the old binary and copy new one
            let backup_path = install_path.with_extension("old");
            std::fs::rename(&install_path, &backup_path)?;
            std::fs::copy(&binary_path, &install_path)?;
            let _ = std::fs::remove_file(&backup_path);
        }
    }

    #[cfg(windows)]
    {
        // On Windows, rename the running exe and copy new one
        let backup_path = install_path.with_extension("old.exe");
        std::fs::rename(&install_path, &backup_path)?;
        std::fs::copy(&binary_path, &install_path)?;
        // Note: old exe will be cleaned up on next run or reboot
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
        ("linux", "x86_64") => "ethcli-linux-x86_64.tar.gz".to_string(),
        ("linux", "aarch64") => "ethcli-linux-aarch64.tar.gz".to_string(),
        ("windows", "x86_64") => "ethcli-windows-x86_64.zip".to_string(),
        _ => format!("ethcli-{}-{}.tar.gz", os, arch),
    }
}
