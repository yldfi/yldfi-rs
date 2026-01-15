use crate::utils::address::resolve_label;
use std::process::Command;

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
    let mut cmd = Command::new("cast");
    cmd.arg("call");

    // Resolve target address
    let resolved_to = resolve_label(to);
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
