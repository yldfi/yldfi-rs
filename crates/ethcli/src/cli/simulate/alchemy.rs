//! Alchemy backend for `ethcli simulate`
//!
//! Alchemy retired its Transaction Simulation API (`alchemy_simulateAssetChanges`,
//! `alchemy_simulateExecution` and their bundle variants) on 2026-09-30, so this
//! backend now uses Alchemy's `debug_traceCall` / `debug_traceTransaction` RPC
//! methods (call-tracer output) instead.

use super::{build_calldata, AlchemyArgs};
use crate::utils::address::resolve_label;
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
    chain: crate::config::Chain,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alchemy.create_client_for(Some(chain))?;

    // Build calldata from signature or raw data
    let calldata = build_calldata(sig, data, args)?;

    // Address book labels, as the other backends accept
    let to = resolve_label(to);
    let from = from.as_deref().map(resolve_label);
    let call = TraceCallObject {
        to: to.clone(),
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
    let block = super::utils::block_to_param(block)?;

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

/// Trace an existing transaction via Alchemy's debug API
pub async fn trace_tx_via_alchemy(
    hash: &str,
    alchemy: &AlchemyArgs,
    chain: crate::config::Chain,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alchemy.create_client_for(Some(chain))?;

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

/// Format a value string to hex (the shared simulate value parser)
fn format_value_hex(value: &str) -> anyhow::Result<String> {
    super::utils::value_to_hex(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_param_converts_decimal_to_hex() {
        use super::super::utils::block_to_param;
        assert_eq!(block_to_param("19000000").unwrap(), "0x121eac0");
        assert_eq!(block_to_param("latest").unwrap(), "latest");
        assert_eq!(block_to_param("0x10").unwrap(), "0x10");
        assert!(block_to_param("soon").is_err());
    }

    #[test]
    fn value_hex_handles_units() {
        assert_eq!(format_value_hex("1").unwrap(), "0x1");
        assert_eq!(format_value_hex("1gwei").unwrap(), "0x3b9aca00");
        assert_eq!(format_value_hex("0xff").unwrap(), "0xff");
        assert_eq!(format_value_hex("0.1eth").unwrap(), "0x16345785d8a0000");
    }
}
