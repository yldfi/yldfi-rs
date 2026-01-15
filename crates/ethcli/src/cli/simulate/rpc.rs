use super::types::DryRunFormat;
use super::utils::{
    block_to_param, build_calldata, build_state_overrides, format_request, get_debug_rpc_url,
    get_trace_rpc_url, value_to_hex,
};
use crate::config::Chain;
use crate::utils::address::resolve_label;

/// Simulate using debug_traceCall RPC
#[allow(clippy::too_many_arguments)]
pub async fn simulate_via_debug_rpc(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    block: &str,
    rpc_url: &Option<String>,
    chain: Chain,
    balance_overrides: &[String],
    storage_overrides: &[String],
    code_overrides: &[String],
    dry_run: Option<DryRunFormat>,
    show_secrets: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpc = get_debug_rpc_url(rpc_url, chain)
        .ok_or_else(|| anyhow::anyhow!(
            "Debug RPC URL required. Set via --rpc-url, add an endpoint with has_debug: true, or use 'config add-debug-rpc'"
        ))?;

    // Resolve target address
    let resolved_to = resolve_label(to);

    let calldata = build_calldata(sig, data, args)?;

    let from_addr = from
        .clone()
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    let value_hex = value_to_hex(value)?;
    let block_param = block_to_param(block)?;

    // Build state overrides if any are provided
    let state_overrides =
        build_state_overrides(balance_overrides, storage_overrides, code_overrides)?;

    // Build tracer options with optional state overrides
    let mut tracer_opts = serde_json::json!({
        "tracer": "callTracer",
        "tracerConfig": {
            "withLog": true
        }
    });

    if !state_overrides.is_empty() {
        tracer_opts["stateOverrides"] = serde_json::to_value(&state_overrides)?;
    }

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "debug_traceCall",
        "params": [
            {
                "from": from_addr,
                "to": resolved_to,
                "data": calldata,
                "value": value_hex
            },
            block_param,
            tracer_opts
        ],
        "id": 1
    });

    // Handle dry-run mode - output request without executing
    if let Some(format) = dry_run {
        let headers = vec![("Content-Type", "application/json")];
        let output = format_request(&rpc, "POST", &headers, &request, format, show_secrets);
        println!("{}", output);
        return Ok(());
    }

    if !quiet {
        eprintln!("Calling debug_traceCall on {}...", rpc);
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&rpc)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;

    if let Some(error) = result.get("error") {
        return Err(anyhow::anyhow!("RPC error: {}", error));
    }

    if let Some(trace) = result.get("result") {
        println!("{}", serde_json::to_string_pretty(trace)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Trace existing tx via debug_traceTransaction
pub async fn trace_tx_via_debug_rpc(
    hash: &str,
    rpc_url: &Option<String>,
    chain: Chain,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpc = get_debug_rpc_url(rpc_url, chain)
        .ok_or_else(|| anyhow::anyhow!(
            "Debug RPC URL required. Set via --rpc-url, add an endpoint with has_debug: true, or use 'config add-debug-rpc'"
        ))?;

    if !quiet {
        eprintln!("Calling debug_traceTransaction on {}...", rpc);
    }

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "debug_traceTransaction",
        "params": [
            hash,
            {
                "tracer": "callTracer",
                "tracerConfig": {
                    "withLog": true
                }
            }
        ],
        "id": 1
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&rpc)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;

    if let Some(error) = result.get("error") {
        return Err(anyhow::anyhow!("RPC error: {}", error));
    }

    if let Some(trace) = result.get("result") {
        println!("{}", serde_json::to_string_pretty(trace)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Simulate using trace_call RPC (Parity/Erigon style)
#[allow(clippy::too_many_arguments)]
pub async fn simulate_via_trace_rpc(
    to: &str,
    sig: &Option<String>,
    data: &Option<String>,
    args: &[String],
    from: &Option<String>,
    value: &str,
    block: &str,
    rpc_url: &Option<String>,
    chain: Chain,
    balance_overrides: &[String],
    storage_overrides: &[String],
    code_overrides: &[String],
    dry_run: Option<DryRunFormat>,
    show_secrets: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpc = get_trace_rpc_url(rpc_url, chain).ok_or_else(|| {
        anyhow::anyhow!(
            "Trace RPC URL required. Set via --rpc-url or add an endpoint with has_trace: true"
        )
    })?;

    // Resolve target address
    let resolved_to = resolve_label(to);

    let calldata = build_calldata(sig, data, args)?;

    let from_addr = from
        .clone()
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    let value_hex = value_to_hex(value)?;
    let block_param = block_to_param(block)?;

    // Build state overrides if any are provided
    let state_overrides =
        build_state_overrides(balance_overrides, storage_overrides, code_overrides)?;

    // trace_call params: [tx_object, ["trace", "vmTrace", "stateDiff"], block_number, state_overrides?]
    let mut params = serde_json::json!([
        {
            "from": from_addr,
            "to": resolved_to,
            "data": calldata,
            "value": value_hex
        },
        ["trace", "vmTrace"],
        block_param
    ]);

    // Add state overrides as 4th parameter if any are provided
    if !state_overrides.is_empty() {
        params
            .as_array_mut()
            .unwrap()
            .push(serde_json::to_value(&state_overrides)?);
    }

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "trace_call",
        "params": params,
        "id": 1
    });

    // Handle dry-run mode - output request without executing
    if let Some(format) = dry_run {
        let headers = vec![("Content-Type", "application/json")];
        let output = format_request(&rpc, "POST", &headers, &request, format, show_secrets);
        println!("{}", output);
        return Ok(());
    }

    if !quiet {
        eprintln!("Calling trace_call on {}...", rpc);
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&rpc)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;

    if let Some(error) = result.get("error") {
        return Err(anyhow::anyhow!("RPC error: {}", error));
    }

    if let Some(trace) = result.get("result") {
        println!("{}", serde_json::to_string_pretty(trace)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}

/// Trace existing tx via trace_transaction (Parity/Erigon style)
pub async fn trace_tx_via_trace_rpc(
    hash: &str,
    rpc_url: &Option<String>,
    chain: Chain,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpc = get_trace_rpc_url(rpc_url, chain).ok_or_else(|| {
        anyhow::anyhow!(
            "Trace RPC URL required. Set via --rpc-url or add an endpoint with has_trace: true"
        )
    })?;

    if !quiet {
        eprintln!("Calling trace_transaction on {}...", rpc);
    }

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "trace_transaction",
        "params": [hash],
        "id": 1
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&rpc)
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;

    let result: serde_json::Value = response.json().await?;

    if let Some(error) = result.get("error") {
        return Err(anyhow::anyhow!("RPC error: {}", error));
    }

    if let Some(trace) = result.get("result") {
        println!("{}", serde_json::to_string_pretty(trace)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&result)?);
    }

    Ok(())
}
