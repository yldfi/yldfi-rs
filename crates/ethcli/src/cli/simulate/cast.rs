use crate::utils::{address::resolve_label, is_safe_cli_value, is_valid_eth_address, is_valid_tx_hash};
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
    // Validate inputs to prevent flag injection
    validate_cli_arg(value, "value")?;
    validate_cli_arg(block, "block")?;
    if let Some(ref sig_str) = sig {
        validate_cli_arg(sig_str, "signature")?;
    }
    if let Some(ref data_str) = data {
        validate_cli_arg(data_str, "data")?;
    }
    if let Some(ref rpc) = rpc_url {
        validate_cli_arg(rpc, "rpc-url")?;
    }
    for arg in args {
        validate_cli_arg(arg, "argument")?;
    }

    let mut cmd = Command::new("cast");
    cmd.arg("call");

    // Resolve target address
    let resolved_to = resolve_label(to);

    // Validate resolved address looks like an address (unless it's a label)
    if !is_valid_eth_address(&resolved_to) && !to.contains('.') {
        // Allow ENS names (contain dots) and address book labels
        validate_cli_arg(&resolved_to, "to address")?;
    }

    cmd.arg(&resolved_to);

    // Add signature or data
    if let Some(sig) = sig {
        cmd.arg(sig);
        for arg in args {
            // Resolve address labels in args
            cmd.arg(resolve_label(arg));
        }
    } else if let Some(data) = data {
        cmd.arg("--data").arg(data);
    } else {
        return Err(anyhow::anyhow!("Must provide --sig or --data"));
    }

    // Add optional params
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

    // Validate RPC URL if provided
    if let Some(ref rpc) = rpc_url {
        validate_cli_arg(rpc, "rpc-url")?;
    }

    let mut cmd = Command::new("cast");
    cmd.arg("run");
    cmd.arg(hash);

    if trace {
        cmd.arg("--trace-printer");
    }

    if debug {
        cmd.arg("--debug");
    }

    if let Some(rpc) = rpc_url {
        cmd.arg("--rpc-url").arg(rpc);
    }

    if !quiet {
        eprintln!("Running: cast run {} ...", hash);
    }

    let status = cmd.status()?;

    if !status.success() {
        return Err(anyhow::anyhow!("cast run failed"));
    }

    Ok(())
}
