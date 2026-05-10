use super::cast::{run_cast_call, CastCallOptions, CastCallRequest};
use crate::config::Chain;
use crate::rpc::get_rpc_url;
use serde_json::json;
use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use tokio::time::{sleep, Duration, Instant};

/// Additional options supported by Foundry `anvil` fork mode.
#[derive(Debug, Clone, Default)]
pub struct AnvilOptions {
    pub chain: Chain,
    pub fork_urls: Vec<String>,
    pub fork_block_number: Option<String>,
    pub fork_transaction_hash: Option<String>,
    pub fork_chain_id: Option<u64>,
    pub fork_headers: Vec<String>,
    pub hardfork: Option<String>,
    pub network: Option<String>,
    pub no_rate_limit: bool,
    pub no_storage_caching: bool,
    pub timeout_ms: Option<u64>,
    pub retries: Option<u32>,
    pub block_gas_limit: Option<u64>,
    pub block_base_fee: Option<u64>,
    pub disable_block_gas_limit: bool,
    pub enable_tx_gas_limit: bool,
}

struct AnvilProcess {
    child: Child,
}

impl AnvilProcess {
    fn try_wait(&mut self) -> std::io::Result<Option<std::process::ExitStatus>> {
        self.child.try_wait()
    }
}

impl Drop for AnvilProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn pick_free_port() -> anyhow::Result<u16> {
    let listener = TcpListener::bind(("127.0.0.1", 0))?;
    Ok(listener.local_addr()?.port())
}

fn resolve_fork_urls(
    chain: Chain,
    rpc_url: &Option<String>,
    extra_fork_urls: &[String],
) -> anyhow::Result<Vec<String>> {
    let mut fork_urls = Vec::new();

    if let Some(rpc_url) = rpc_url {
        fork_urls.push(rpc_url.clone());
    }

    fork_urls.extend(extra_fork_urls.iter().cloned());

    if fork_urls.is_empty() {
        if let Ok(configured_rpc) = get_rpc_url(chain) {
            fork_urls.push(configured_rpc);
        } else if chain == Chain::Ethereum {
            fork_urls.push("https://eth.llamarpc.com".to_string());
        } else {
            anyhow::bail!(
                "No fork RPC URL configured for {}. Pass --rpc-url or --fork-url.",
                chain.display_name()
            );
        }
    }

    Ok(fork_urls)
}

fn build_anvil_command(port: u16, fork_urls: &[String], options: &AnvilOptions) -> Command {
    let mut cmd = Command::new("anvil");
    cmd.arg("--host")
        .arg("127.0.0.1")
        .arg("--port")
        .arg(port.to_string())
        .arg("--quiet");

    for url in fork_urls {
        cmd.arg("--fork-url").arg(url);
    }

    if let Some(block) = &options.fork_block_number {
        cmd.arg("--fork-block-number").arg(block);
    }

    if let Some(tx_hash) = &options.fork_transaction_hash {
        cmd.arg("--fork-transaction-hash").arg(tx_hash);
    }

    if let Some(chain_id) = options.fork_chain_id {
        cmd.arg("--fork-chain-id").arg(chain_id.to_string());
    }

    for header in &options.fork_headers {
        cmd.arg("--fork-header").arg(header);
    }

    if let Some(hardfork) = &options.hardfork {
        cmd.arg("--hardfork").arg(hardfork);
    }

    if let Some(network) = &options.network {
        cmd.arg("--network").arg(network);
    }

    if options.no_rate_limit {
        cmd.arg("--no-rate-limit");
    }

    if options.no_storage_caching {
        cmd.arg("--no-storage-caching");
    }

    if let Some(timeout_ms) = options.timeout_ms {
        cmd.arg("--timeout").arg(timeout_ms.to_string());
    }

    if let Some(retries) = options.retries {
        cmd.arg("--retries").arg(retries.to_string());
    }

    if let Some(gas_limit) = options.block_gas_limit {
        cmd.arg("--gas-limit").arg(gas_limit.to_string());
    }

    if let Some(base_fee) = options.block_base_fee {
        cmd.arg("--block-base-fee-per-gas")
            .arg(base_fee.to_string());
    }

    if options.disable_block_gas_limit {
        cmd.arg("--disable-block-gas-limit");
    }

    if options.enable_tx_gas_limit {
        cmd.arg("--enable-tx-gas-limit");
    }

    cmd.stdout(Stdio::null()).stderr(Stdio::null());
    cmd
}

async fn wait_for_anvil(
    rpc_url: &str,
    process: &mut AnvilProcess,
    no_proxy: bool,
) -> anyhow::Result<()> {
    let mut builder = reqwest::Client::builder().timeout(Duration::from_millis(750));
    if no_proxy {
        builder = builder.no_proxy();
    }
    let client = builder.build()?;
    let deadline = Instant::now() + Duration::from_secs(10);

    while Instant::now() < deadline {
        if let Some(status) = process.try_wait()? {
            anyhow::bail!("Anvil exited before becoming ready: {status}");
        }

        let response = client
            .post(rpc_url)
            .json(&json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "eth_chainId",
                "params": [],
            }))
            .send()
            .await;

        if matches!(response, Ok(response) if response.status().is_success()) {
            return Ok(());
        }

        sleep(Duration::from_millis(100)).await;
    }

    anyhow::bail!("Timed out waiting for Anvil at {rpc_url}");
}

/// Simulate using Anvil fork
pub async fn simulate_via_anvil(
    request: CastCallRequest<'_>,
    rpc_url: &Option<String>,
    options: &AnvilOptions,
    cast_options: &CastCallOptions,
    quiet: bool,
) -> anyhow::Result<()> {
    if request.sig.is_none() && request.data.is_none() {
        anyhow::bail!("Must provide --sig or --data");
    }

    let fork_urls = resolve_fork_urls(options.chain, rpc_url, &options.fork_urls)?;
    let port = pick_free_port()?;
    let local_rpc_url = format!("http://127.0.0.1:{port}");

    if !quiet {
        let extra_count = fork_urls.len().saturating_sub(1);
        if extra_count == 0 {
            eprintln!("Starting Anvil fork of {}...", fork_urls[0]);
        } else {
            eprintln!(
                "Starting Anvil fork of {} plus {extra_count} fallback endpoint(s)...",
                fork_urls[0]
            );
        }
    }

    let child = build_anvil_command(port, &fork_urls, options).spawn()?;
    let mut anvil = AnvilProcess { child };
    wait_for_anvil(&local_rpc_url, &mut anvil, cast_options.no_proxy).await?;

    let mut local_cast_options = cast_options.clone();
    local_cast_options.trace = true;
    let local_rpc_url = Some(local_rpc_url);

    run_cast_call(request, &local_rpc_url, &local_cast_options, quiet).await?;

    if !quiet {
        eprintln!("\nAnvil fork terminated.");
    }

    Ok(())
}
