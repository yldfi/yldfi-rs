use crate::{
    config::Chain,
    utils::{address::resolve_label, is_safe_cli_value, is_valid_eth_address, is_valid_tx_hash},
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

/// Additional options supported by Foundry `cast call`.
#[derive(Debug, Clone, Default)]
pub struct CastCallOptions {
    pub chain: Chain,
    pub trace: bool,
    pub gas: Option<u64>,
    pub gas_price: Option<u64>,
    pub access_list: Option<String>,
    pub balance_overrides: Vec<String>,
    pub storage_overrides: Vec<String>,
    pub code_overrides: Vec<String>,
    pub nonce_overrides: Vec<String>,
    pub block_timestamp: Option<u64>,
    pub block_number_override: Option<u64>,
    pub decode_internal: bool,
    pub disable_labels: bool,
    pub labels: Vec<String>,
    pub evm_version: Option<String>,
    pub with_local_artifacts: bool,
    pub no_proxy: bool,
    pub rpc_timeout: Option<u64>,
    pub rpc_headers: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct CastCallRequest<'a> {
    pub to: &'a str,
    pub sig: &'a Option<String>,
    pub data: &'a Option<String>,
    pub args: &'a [String],
    pub from: &'a Option<String>,
    pub value: &'a str,
    pub block: &'a str,
}

/// Additional options supported by Foundry `cast run`.
#[derive(Debug, Clone, Default)]
pub struct CastTxOptions {
    pub chain: Chain,
    pub trace: bool,
    pub debug: bool,
    pub quick: bool,
    pub decode_internal: bool,
    pub trace_depth: Option<u32>,
    pub replay_system_txs: bool,
    pub disable_labels: bool,
    pub labels: Vec<String>,
    pub evm_version: Option<String>,
    pub with_local_artifacts: bool,
    pub no_proxy: bool,
    pub rpc_timeout: Option<u64>,
    pub rpc_headers: Vec<String>,
    pub enable_tx_gas_limit: bool,
    pub disable_block_gas_limit: bool,
    pub etherscan_api_key: Option<String>,
}

fn normalize_address_value_override(value: &str, name: &str) -> anyhow::Result<String> {
    validate_cli_arg(value, name)?;
    let (address, replacement) = value
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("Invalid {name}: expected address=value"))?;

    if address.is_empty() || replacement.is_empty() {
        anyhow::bail!("Invalid {name}: address and value must be non-empty");
    }

    Ok(format!("{address}:{replacement}"))
}

fn normalize_storage_override(value: &str) -> anyhow::Result<String> {
    validate_cli_arg(value, "storage override")?;
    let (account_and_slot, replacement) = value
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("Invalid storage override: expected address:slot=value"))?;
    let (address, slot) = account_and_slot
        .split_once(':')
        .ok_or_else(|| anyhow::anyhow!("Invalid storage override: expected address:slot=value"))?;

    if address.is_empty() || slot.is_empty() || replacement.is_empty() {
        anyhow::bail!("Invalid storage override: address, slot, and value must be non-empty");
    }

    Ok(format!("{address}:{slot}:{replacement}"))
}

fn normalize_overrides(
    values: &[String],
    name: &str,
    normalize: fn(&str, &str) -> anyhow::Result<String>,
) -> anyhow::Result<Option<String>> {
    if values.is_empty() {
        return Ok(None);
    }

    values
        .iter()
        .map(|value| normalize(value, name))
        .collect::<anyhow::Result<Vec<_>>>()
        .map(|values| Some(values.join(",")))
}

fn normalize_storage_overrides(values: &[String]) -> anyhow::Result<Option<String>> {
    if values.is_empty() {
        return Ok(None);
    }

    values
        .iter()
        .map(|value| normalize_storage_override(value))
        .collect::<anyhow::Result<Vec<_>>>()
        .map(|values| Some(values.join(",")))
}

fn validate_values(values: &[String], name: &str) -> anyhow::Result<()> {
    for value in values {
        validate_cli_arg(value, name)?;
    }
    Ok(())
}

fn push_rpc_options(cmd: &mut Command, options: &CastCallOptions, rpc_url: &Option<String>) {
    if let Some(rpc) = rpc_url {
        cmd.arg("--rpc-url").arg(rpc);
    }

    if let Some(timeout) = options.rpc_timeout {
        cmd.arg("--rpc-timeout").arg(timeout.to_string());
    }

    if options.no_proxy {
        cmd.arg("--no-proxy");
    }

    for header in &options.rpc_headers {
        cmd.arg("--rpc-headers").arg(header);
    }
}

fn push_trace_options(cmd: &mut Command, options: &CastCallOptions) -> anyhow::Result<()> {
    if options.trace {
        cmd.arg("--trace");
    }

    if options.decode_internal {
        cmd.arg("--decode-internal");
    }

    if options.disable_labels {
        cmd.arg("--disable-labels");
    }

    validate_values(&options.labels, "trace label")?;
    if !options.labels.is_empty() {
        cmd.arg("--labels").arg(options.labels.join(","));
    }

    if let Some(evm_version) = &options.evm_version {
        validate_cli_arg(evm_version, "EVM version")?;
        cmd.arg("--evm-version").arg(evm_version);
    }

    if options.with_local_artifacts {
        cmd.arg("--with-local-artifacts");
    }

    Ok(())
}

fn push_wallet_options(cmd: &mut Command, options: &CastCallOptions) -> anyhow::Result<()> {
    cmd.arg("--chain").arg(options.chain.chain_id().to_string());

    if let Some(overrides) = normalize_overrides(
        &options.balance_overrides,
        "balance override",
        normalize_address_value_override,
    )? {
        cmd.arg("--override-balance").arg(overrides);
    }

    if let Some(overrides) = normalize_storage_overrides(&options.storage_overrides)? {
        cmd.arg("--override-state-diff").arg(overrides);
    }

    if let Some(overrides) = normalize_overrides(
        &options.code_overrides,
        "code override",
        normalize_address_value_override,
    )? {
        cmd.arg("--override-code").arg(overrides);
    }

    if let Some(overrides) = normalize_overrides(
        &options.nonce_overrides,
        "nonce override",
        normalize_address_value_override,
    )? {
        cmd.arg("--override-nonce").arg(overrides);
    }

    if let Some(timestamp) = options.block_timestamp {
        cmd.arg("--block.time").arg(timestamp.to_string());
    }

    if let Some(block_number) = options.block_number_override {
        cmd.arg("--block.number").arg(block_number.to_string());
    }

    Ok(())
}

/// Simulate using cast call --trace
pub async fn simulate_via_cast(
    request: CastCallRequest<'_>,
    rpc_url: &Option<String>,
    options: &CastCallOptions,
    quiet: bool,
) -> anyhow::Result<()> {
    run_cast_call(request, rpc_url, options, quiet).await
}

/// Run a `cast call` using the provided RPC endpoint. Shared by the Cast and
/// Anvil backends so forked simulations honor the same call-level options.
pub(crate) fn build_cast_call_command(
    request: CastCallRequest<'_>,
    rpc_url: &Option<String>,
    options: &CastCallOptions,
) -> anyhow::Result<Command> {
    // Resolve target address
    let resolved_to = resolve_label(request.to);

    // Validate resolved address looks like an address (unless it's a label)
    if !is_valid_eth_address(&resolved_to) && !request.to.contains('.') {
        // Allow ENS names (contain dots) and address book labels
        validate_cli_arg(&resolved_to, "to address")?;
    }

    let mut cmd = Command::new("cast");
    cmd.arg("call");

    // Add all flags FIRST (these are controlled by us, not user input)
    if let Some(from) = request.from {
        cmd.arg("--from").arg(from);
    }

    if request.value != "0" {
        cmd.arg("--value").arg(request.value);
    }

    if let Some(gas) = options.gas {
        cmd.arg("--gas-limit").arg(gas.to_string());
    }

    if let Some(gas_price) = options.gas_price {
        cmd.arg("--gas-price").arg(gas_price.to_string());
    }

    if let Some(access_list) = &options.access_list {
        validate_cli_arg(access_list, "access list")?;
        cmd.arg("--access-list");
        if !access_list.is_empty() {
            cmd.arg(access_list);
        }
    }

    cmd.arg("--block").arg(request.block);

    push_rpc_options(&mut cmd, options, rpc_url);
    push_trace_options(&mut cmd, options)?;
    push_wallet_options(&mut cmd, options)?;

    // Add --data flag if using raw data (before the -- separator)
    if request.sig.is_none() {
        if let Some(data) = request.data {
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
    if let Some(sig) = request.sig {
        cmd.arg(sig);
        for arg in request.args {
            // Resolve address labels in args
            cmd.arg(resolve_label(arg));
        }
    }

    Ok(cmd)
}

/// Run a `cast call` using the provided RPC endpoint. Shared by the Cast and
/// Anvil backends so forked simulations honor the same call-level options.
pub(crate) async fn run_cast_call(
    request: CastCallRequest<'_>,
    rpc_url: &Option<String>,
    options: &CastCallOptions,
    quiet: bool,
) -> anyhow::Result<()> {
    let mut cmd = build_cast_call_command(request, rpc_url, options)?;

    if !quiet {
        let trace_str = if options.trace { " --trace" } else { "" };
        eprintln!("Running: cast call {}{} ...", request.to, trace_str);
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
    rpc_url: &Option<String>,
    options: &CastTxOptions,
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
    if options.trace {
        cmd.arg("--trace-printer");
    }

    if options.debug {
        cmd.arg("--debug");
    }

    if options.quick {
        cmd.arg("--quick");
    }

    if options.decode_internal {
        cmd.arg("--decode-internal");
    }

    if let Some(depth) = options.trace_depth {
        cmd.arg("--trace-depth").arg(depth.to_string());
    }

    if options.replay_system_txs {
        cmd.arg("--replay-system-txes");
    }

    if options.disable_labels {
        cmd.arg("--disable-labels");
    }

    validate_values(&options.labels, "trace label")?;
    for label in &options.labels {
        cmd.arg("--label").arg(label);
    }

    if let Some(key) = &options.etherscan_api_key {
        cmd.arg("--etherscan-api-key").arg(key);
    }

    cmd.arg("--chain").arg(options.chain.chain_id().to_string());

    if let Some(rpc) = rpc_url {
        cmd.arg("--rpc-url").arg(rpc);
    }

    if let Some(timeout) = options.rpc_timeout {
        cmd.arg("--rpc-timeout").arg(timeout.to_string());
    }

    if options.no_proxy {
        cmd.arg("--no-proxy");
    }

    for header in &options.rpc_headers {
        cmd.arg("--rpc-headers").arg(header);
    }

    if let Some(evm_version) = &options.evm_version {
        validate_cli_arg(evm_version, "EVM version")?;
        cmd.arg("--evm-version").arg(evm_version);
    }

    if options.with_local_artifacts {
        cmd.arg("--with-local-artifacts");
    }

    if options.disable_block_gas_limit {
        cmd.arg("--disable-block-gas-limit");
    }

    if options.enable_tx_gas_limit {
        cmd.arg("--enable-tx-gas-limit");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_address_value_override_for_cast() {
        assert_eq!(
            normalize_address_value_override("0xabc=100", "balance override").unwrap(),
            "0xabc:100"
        );
    }

    #[test]
    fn normalizes_storage_override_for_cast_state_diff() {
        assert_eq!(
            normalize_storage_override("0xabc:0x01=0xff").unwrap(),
            "0xabc:0x01:0xff"
        );
    }

    #[test]
    fn rejects_malformed_storage_override() {
        let err = normalize_storage_override("0xabc=0xff").unwrap_err();
        assert!(err.to_string().contains("address:slot=value"));
    }
}
