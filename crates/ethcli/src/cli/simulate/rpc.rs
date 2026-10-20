use super::types::DryRunFormat;
use super::utils::{
    block_to_param, build_calldata, build_state_overrides, format_request, get_debug_rpc_urls,
    get_trace_rpc_urls, post_jsonrpc_with_failover, value_to_hex,
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
    nonce_overrides: &[String],
    block_timestamp: Option<u64>,
    block_number_override: Option<u64>,
    dry_run: Option<DryRunFormat>,
    show_secrets: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpcs = get_debug_rpc_urls(rpc_url, chain);
    let rpc = rpcs.first().cloned().ok_or_else(|| anyhow::anyhow!(
            "Debug RPC URL required. Set via --rpc-url, add an endpoint with has_debug: true, or use 'config add-debug-rpc'"
        ))?;

    // Resolve target address
    let resolved_to = resolve_label(to);

    let calldata = build_calldata(sig, data, args)?;

    let from_addr = from
        .as_deref()
        .map(resolve_label)
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    let value_hex = value_to_hex(value)?;
    let block_param = block_to_param(block)?;

    let state_overrides = build_state_overrides(
        balance_overrides,
        storage_overrides,
        code_overrides,
        nonce_overrides,
    )?;

    let params = debug_trace_call_params(
        serde_json::json!({
            "from": from_addr,
            "to": resolved_to,
            "data": calldata,
            "value": value_hex
        }),
        block_param,
        serde_json::to_value(&state_overrides)?,
        block_timestamp,
        block_number_override,
    );

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "debug_traceCall",
        "params": params,
        "id": 1
    });

    // Handle dry-run mode - output request without executing
    if let Some(format) = dry_run {
        let headers = vec![("Content-Type", "application/json")];
        let shown_rpc = if show_secrets {
            rpc.clone()
        } else {
            crate::utils::url::redact_url(&rpc)
        };
        let output = format_request(&shown_rpc, "POST", &headers, &request, format, show_secrets);
        println!("{}", output);
        return Ok(());
    }

    let trace = post_jsonrpc_with_failover(&rpcs, &request, quiet).await?;

    println!("{}", serde_json::to_string_pretty(&trace)?);

    Ok(())
}

/// `debug_traceCall` params: `[call, block, traceConfig]`.
///
/// Geth takes state and block overrides inside the trace config
/// (`stateOverrides`, `blockOverrides`), not as a 4th parameter.
fn debug_trace_call_params(
    call: serde_json::Value,
    block_param: String,
    state_overrides: serde_json::Value,
    block_timestamp: Option<u64>,
    block_number_override: Option<u64>,
) -> serde_json::Value {
    let mut tracer_opts = serde_json::json!({
        "tracer": "callTracer",
        "tracerConfig": {
            "withLog": true
        }
    });
    if state_overrides.as_object().is_some_and(|o| !o.is_empty()) {
        tracer_opts["stateOverrides"] = state_overrides;
    }
    let mut block_overrides = serde_json::Map::new();
    if let Some(ts) = block_timestamp {
        block_overrides.insert("time".into(), format!("{ts:#x}").into());
    }
    if let Some(n) = block_number_override {
        block_overrides.insert("number".into(), format!("{n:#x}").into());
    }
    if !block_overrides.is_empty() {
        tracer_opts["blockOverrides"] = block_overrides.into();
    }
    serde_json::json!([call, block_param, tracer_opts])
}

/// Trace existing tx via debug_traceTransaction.
///
/// By default the callTracer result is decoded into a cast-style call tree
/// (contract names, function args, events); `raw` restores the JSON output.
pub async fn trace_tx_via_debug_rpc(
    hash: &str,
    rpc_url: &Option<String>,
    chain: Chain,
    etherscan_key: Option<String>,
    raw: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpcs = get_debug_rpc_urls(rpc_url, chain);
    rpcs.first().ok_or_else(|| anyhow::anyhow!(
            "Debug RPC URL required. Set via --rpc-url, add an endpoint with has_debug: true, or use 'config add-debug-rpc'"
        ))?;

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

    let trace = post_jsonrpc_with_failover(&rpcs, &request, quiet).await?;

    if raw {
        println!("{}", serde_json::to_string_pretty(&trace)?);
        return Ok(());
    }

    if !quiet {
        eprintln!("Decoding trace (ABIs, signatures, labels)...");
    }
    let decoder = super::decode::TraceDecoder::new(chain, etherscan_key)?;
    println!("Traces:");
    println!("{}", decoder.render(&trace).await);

    if let Some(gas) = trace
        .get("gasUsed")
        .and_then(|g| g.as_str())
        .and_then(|g| u64::from_str_radix(g.strip_prefix("0x").unwrap_or(g), 16).ok())
    {
        println!("\nGas used: {}", gas);
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
    nonce_overrides: &[String],
    dry_run: Option<DryRunFormat>,
    show_secrets: bool,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpcs = get_trace_rpc_urls(rpc_url, chain);
    let rpc = rpcs.first().cloned().ok_or_else(|| {
        anyhow::anyhow!(
            "Trace RPC URL required. Set via --rpc-url or add an endpoint with has_trace: true"
        )
    })?;

    // Resolve target address
    let resolved_to = resolve_label(to);

    let calldata = build_calldata(sig, data, args)?;

    let from_addr = from
        .as_deref()
        .map(resolve_label)
        .unwrap_or_else(|| "0x0000000000000000000000000000000000000000".to_string());

    let value_hex = value_to_hex(value)?;
    let block_param = block_to_param(block)?;

    let state_overrides = build_state_overrides(
        balance_overrides,
        storage_overrides,
        code_overrides,
        nonce_overrides,
    )?;

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
        let shown_rpc = if show_secrets {
            rpc.clone()
        } else {
            crate::utils::url::redact_url(&rpc)
        };
        let output = format_request(&shown_rpc, "POST", &headers, &request, format, show_secrets);
        println!("{}", output);
        return Ok(());
    }

    let trace = post_jsonrpc_with_failover(&rpcs, &request, quiet).await?;

    println!("{}", serde_json::to_string_pretty(&trace)?);

    Ok(())
}

/// Trace existing tx via trace_transaction (Parity/Erigon style)
pub async fn trace_tx_via_trace_rpc(
    hash: &str,
    rpc_url: &Option<String>,
    chain: Chain,
    quiet: bool,
) -> anyhow::Result<()> {
    let rpcs = get_trace_rpc_urls(rpc_url, chain);
    rpcs.first().ok_or_else(|| {
        anyhow::anyhow!(
            "Trace RPC URL required. Set via --rpc-url or add an endpoint with has_trace: true"
        )
    })?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "trace_transaction",
        "params": [hash],
        "id": 1
    });

    let trace = post_jsonrpc_with_failover(&rpcs, &request, quiet).await?;

    println!("{}", serde_json::to_string_pretty(&trace)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_trace_call_puts_overrides_in_the_trace_config_as_hex() {
        let overrides = build_state_overrides(
            &["0x00000000000000000000000000000000000000aa=1000000000000000000".to_string()],
            &["0x00000000000000000000000000000000000000bb:1=255".to_string()],
            &[],
            &["0x00000000000000000000000000000000000000aa=7".to_string()],
        )
        .unwrap();
        let params = debug_trace_call_params(
            serde_json::json!({"to": "0x00000000000000000000000000000000000000bb"}),
            "latest".to_string(),
            serde_json::to_value(&overrides).unwrap(),
            Some(1_700_000_000),
            Some(20_000_000),
        );

        let params = params.as_array().unwrap();
        assert_eq!(params.len(), 3, "overrides must not be a 4th param");
        let config = &params[2];
        let aa = &config["stateOverrides"]["0x00000000000000000000000000000000000000aa"];
        assert_eq!(aa["balance"], "0xde0b6b3a7640000");
        assert_eq!(aa["nonce"], "0x7");
        let bb = &config["stateOverrides"]["0x00000000000000000000000000000000000000bb"];
        assert_eq!(
            bb["stateDiff"]["0x0000000000000000000000000000000000000000000000000000000000000001"],
            "0x00000000000000000000000000000000000000000000000000000000000000ff"
        );
        assert_eq!(config["blockOverrides"]["time"], "0x6553f100");
        assert_eq!(config["blockOverrides"]["number"], "0x1312d00");
    }

    #[test]
    fn debug_trace_call_without_overrides_has_plain_config() {
        let params = debug_trace_call_params(
            serde_json::json!({}),
            "latest".to_string(),
            serde_json::json!({}),
            None,
            None,
        );
        assert!(params[2].get("stateOverrides").is_none());
        assert!(params[2].get("blockOverrides").is_none());
    }

    #[test]
    fn malformed_overrides_are_rejected() {
        assert!(build_state_overrides(&["0xaa".to_string()], &[], &[], &[]).is_err());
        assert!(build_state_overrides(&[], &["0xaa=1".to_string()], &[], &[]).is_err());
        assert!(build_state_overrides(&[], &[], &["0xaa=0xzz".to_string()], &[]).is_err());
    }
}
