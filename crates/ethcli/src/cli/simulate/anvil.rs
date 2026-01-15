use crate::config::Chain;
use crate::rpc::get_rpc_url;
use crate::utils::address::resolve_label;
use std::process::{Command, Stdio};
use tokio::time::{sleep, Duration};

/// Simulate using Anvil fork
#[allow(clippy::too_many_arguments)]
pub async fn simulate_via_anvil(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    rpc_url: &Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    let fork_url = rpc_url
        .clone()
        .or_else(|| {
            // Try to get URL from configured endpoints (smart selection)
            get_rpc_url(Chain::Ethereum).ok()
        })
        .unwrap_or_else(|| "https://eth.llamarpc.com".to_string());

    if !quiet {
        eprintln!("Starting Anvil fork of {}...", fork_url);
    }

    // Start anvil in background
    let mut anvil = Command::new("anvil")
        .arg("--fork-url")
        .arg(&fork_url)
        .arg("--port")
        .arg("8546") // Use non-default port to avoid conflicts
        .arg("--silent")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    // Wait for anvil to start
    sleep(Duration::from_secs(2)).await;

    // Run the simulation against local anvil
    let mut cmd = Command::new("cast");
    cmd.arg("call");

    // Resolve target address
    let resolved_to = resolve_label(to);
    cmd.arg(&resolved_to);

    if let Some(sig) = sig {
        cmd.arg(sig);
        for arg in args {
            // Resolve address labels in args
            cmd.arg(resolve_label(arg));
        }
    } else if let Some(data) = data {
        cmd.arg("--data").arg(data);
    } else {
        anvil.kill()?;
        return Err(anyhow::anyhow!("Must provide --sig or --data"));
    }

    if let Some(from) = from {
        cmd.arg("--from").arg(from);
    }

    if value != "0" {
        cmd.arg("--value").arg(value);
    }

    cmd.arg("--rpc-url").arg("http://localhost:8546");
    cmd.arg("--trace");

    let output = cmd.output()?;

    // Kill anvil
    anvil.kill()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Simulation failed: {}", stderr));
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));

    if !quiet {
        eprintln!("\nAnvil fork terminated.");
    }

    Ok(())
}
