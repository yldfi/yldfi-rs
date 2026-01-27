use crate::utils::{
    address::resolve_label, is_safe_cli_value, is_valid_eth_address, is_valid_tx_hash,
};
use std::process::Command;

/// Validate that a command-line argument doesn't contain injection attempts
fn validate_cli_arg(arg: &str, name: &str) -> anyhow::Result<()> {
    if !is_safe_cli_value(arg) {
        anyhow::bail!(
            "Invalid {}: '{}' contains potentially unsafe characters",
            name,
            arg
        );
    }
    Ok(())
}

/// Simulate using cast call --trace
#[allow(clippy::too_many_arguments)]
pub async fn simulate_via_cast(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    block: &str,
    rpc_url: &Option<String>,
    trace: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    // Resolve target address
    let resolved_to = resolve_label(to);

    // Validate resolved address looks like an address (unless it's a label)
    if !is_valid_eth_address(&resolved_to) && !to.contains('.') {
        // Allow ENS names (contain dots) and address book labels
        validate_cli_arg(&resolved_to, "to address")?;
    }

    let mut cmd = Command::new("cast");
    cmd.arg("call");

    // Add all flags FIRST (these are controlled by us, not user input)
    if let Some(from) = from {
        cmd.arg("--from").arg(from);
    }

    if value != "0" {
        cmd.arg("--value").arg(value);
    }

    cmd.arg("--block").arg(block);

    // Add RPC URL if provided
    if let Some(rpc) = rpc_url {
        cmd.arg("--rpc-url").arg(rpc);
    }

    // Only add --trace if requested (requires debug-capable node)
    if trace {
        cmd.arg("--trace");
    }

    // Add --data flag if using raw data (before the -- separator)
    if sig.is_none() {
        if let Some(data) = data {
            cmd.arg("--data").arg(data);
        } else {
            return Err(anyhow::anyhow!("Must provide --sig or --data"));
        }
    }

    // SEC-CAST-001: Add `--` to prevent flag injection from user-provided arguments.
    // Everything after `--` is interpreted as a positional argument, not a flag.
    // This prevents attacks like passing `--rpc-url=malicious.com` as a "to" address.
    cmd.arg("--");

    // Now add positional arguments (user-controlled, potentially untrusted)
    cmd.arg(&resolved_to);

    // Add signature and args if using sig mode
    if let Some(sig) = sig {
        cmd.arg(sig);
        for arg in args {
            // Resolve address labels in args
            cmd.arg(resolve_label(arg));
        }
    }

    if !quiet {
        let trace_str = if trace { " --trace" } else { "" };
        eprintln!("Running: cast call {}{} ...", to, trace_str);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("cast call failed: {}", stderr));
    }

    print!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}

/// Trace existing tx using cast run
pub async fn trace_tx_via_cast(
    hash: &str,
    trace: bool,
    debug: bool,
    rpc_url: &Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    // Validate transaction hash format
    if !is_valid_tx_hash(hash) {
        anyhow::bail!(
            "Invalid transaction hash: '{}'. Expected 0x followed by 64 hex characters.",
            hash
        );
    }

    let mut cmd = Command::new("cast");
    cmd.arg("run");

    // Add all flags FIRST (controlled by us)
    if trace {
        cmd.arg("--trace-printer");
    }

    if debug {
        cmd.arg("--debug");
    }

    if let Some(rpc) = rpc_url {
        cmd.arg("--rpc-url").arg(rpc);
    }

    // SEC-CAST-002: Add `--` to prevent flag injection from user-provided hash.
    // The hash is already validated as a proper tx hash format, but defense in depth.
    cmd.arg("--");

    // Now add positional argument (user-controlled)
    cmd.arg(hash);

    if !quiet {
        eprintln!("Running: cast run {} ...", hash);
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("cast run failed"));
    }

    Ok(())
}
