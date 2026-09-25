//! Alchemy backend for `ethcli simulate`
//!
//! Alchemy retired its Transaction Simulation API (`alchemy_simulateAssetChanges`,
//! `alchemy_simulateExecution` and their bundle variants) on 2026-09-30, so this
//! backend now uses Alchemy's `debug_traceCall` / `debug_traceTransaction` RPC
//! methods (call-tracer output) instead.

use super::{build_calldata, AlchemyArgs};
use alcmy::debug::TraceCallObject;

/// Simulate a call via Alchemy's `debug_traceCall` RPC method
///
/// Prints the call-tracer frame (nested calls, gas used, output/revert data) as
/// JSON.
#[allow(clippy::too_many_arguments)]
pub async fn simulate_via_alchemy(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    block: &str,
    gas: Option<u64>,
    gas_price: Option<u64>,
    alchemy: &AlchemyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alchemy.create_client()?;

    // Build calldata from signature or raw data
    let calldata = build_calldata(sig, data, args)?;

    let call = TraceCallObject {
        to: to.to_string(),
        from: from.clone(),
        data: Some(calldata),
        value: if value == "0" {
            None
        } else {
            Some(format_value_hex(value)?)
        },
        gas: gas.map(|g| format!("0x{:x}", g)),
        gas_price: gas_price.map(|g| format!("0x{:x}", g)),
        ..Default::default()
    };
    let block = format_block_param(block);

    if !quiet {
        eprintln!("Simulating call via Alchemy debug_traceCall...");
        eprintln!("  To: {}", to);
        if let Some(ref f) = from {
            eprintln!("  From: {}", f);
        }
        if let Some(ref d) = call.data {
            eprintln!("  Data: {}...", &d[..d.len().min(20)]);
        }
        eprintln!("  Block: {}", block);
    }

    let frame = client
        .debug()
        .trace_call(&call, &block)
        .await
        .map_err(|e| anyhow::anyhow!("Alchemy debug_traceCall failed: {}", e))?;

    println!("{}", serde_json::to_string_pretty(&frame)?);

    Ok(())
}

/// Convert a block argument to a JSON-RPC block parameter
///
/// Decimal block numbers are converted to hex quantities; tags (`latest`,
/// `pending`, ...) and hex values are passed through unchanged.
fn format_block_param(block: &str) -> String {
    let block = block.trim();
    match block.parse::<u64>() {
        Ok(n) => format!("0x{:x}", n),
        Err(_) => block.to_string(),
    }
}

/// Trace an existing transaction via Alchemy's debug API
pub async fn trace_tx_via_alchemy(
    hash: &str,
    alchemy: &AlchemyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alchemy.create_client()?;

    if !quiet {
        eprintln!("Tracing transaction via Alchemy: {}", hash);
    }

    // Use the debug API to trace the transaction
    let trace = client
        .debug()
        .trace_transaction(hash)
        .await
        .map_err(|e| anyhow::anyhow!("Alchemy trace failed: {}", e))?;

    // Pretty-print the result as JSON
    let output = serde_json::to_string_pretty(&trace)?;
    println!("{}", output);

    Ok(())
}

/// Format a value string to hex
fn format_value_hex(value: &str) -> anyhow::Result<String> {
    // Handle different formats: decimal, hex, or with units
    let value = value.trim();

    if value.starts_with("0x") || value.starts_with("0X") {
        // Already hex
        Ok(value.to_string())
    } else if value.ends_with("eth") || value.ends_with("ETH") {
        // Convert ETH to wei
        let eth_str = value.trim_end_matches(|c: char| c.is_alphabetic()).trim();
        let eth: f64 = eth_str.parse()?;
        let wei = (eth * 1e18) as u128;
        Ok(format!("0x{:x}", wei))
    } else if value.ends_with("gwei") || value.ends_with("GWEI") {
        // Convert gwei to wei
        let gwei_str = value.trim_end_matches(|c: char| c.is_alphabetic()).trim();
        let gwei: f64 = gwei_str.parse()?;
        let wei = (gwei * 1e9) as u128;
        Ok(format!("0x{:x}", wei))
    } else {
        // Assume wei (decimal)
        let wei: u128 = value.parse()?;
        Ok(format!("0x{:x}", wei))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_param_converts_decimal_to_hex() {
        assert_eq!(format_block_param("19000000"), "0x121eac0");
        assert_eq!(format_block_param("latest"), "latest");
        assert_eq!(format_block_param("0x10"), "0x10");
    }

    #[test]
    fn value_hex_handles_units() {
        assert_eq!(format_value_hex("1").unwrap(), "0x1");
        assert_eq!(format_value_hex("1gwei").unwrap(), "0x3b9aca00");
        assert_eq!(format_value_hex("0xff").unwrap(), "0xff");
    }
}
