//! ethcli - Comprehensive Ethereum CLI

use alloy::providers::Provider;
use clap::Parser;
use ethcli::cli::{
    config::ConfigCommands,
    endpoints::EndpointCommands,
    logs::{LogsArgs, ProxyArgs, RpcArgs},
    tx::TxArgs,
    Cli, Commands,
};
use ethcli::{
    format_analysis, Chain, Config, ConfigFile, DecodedLog, Endpoint, EndpointConfig, FetchLogs,
    FetchProgress, FetchStats, LogFetcher, OutputFormat, OutputWriter, ProxyConfig, RpcConfig,
    RpcPool, StreamingFetcher, TxAnalyzer,
};
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Parse a duration string like "30d", "6h", "2w", "90m" into seconds
///
/// Supported units:
/// - m, min, minutes: minutes
/// - h, hr, hours: hours
/// - d, days: days
/// - w, weeks: weeks
fn parse_duration_string(s: &str) -> anyhow::Result<f64> {
    let s = s.trim().to_lowercase();

    // Try to find where the number ends and unit begins
    let (num_str, unit) = if let Some(pos) = s.find(|c: char| c.is_alphabetic()) {
        (&s[..pos], s[pos..].trim())
    } else {
        // No unit found, assume days for backwards compatibility
        (s.as_str(), "d")
    };

    let value: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid duration number: '{}'", num_str))?;

    let seconds = match unit {
        "m" | "min" | "mins" | "minute" | "minutes" => value * 60.0,
        "h" | "hr" | "hrs" | "hour" | "hours" => value * 3600.0,
        "d" | "day" | "days" => value * 86400.0,
        "w" | "wk" | "wks" | "week" | "weeks" => value * 604800.0,
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown duration unit: '{}'. Use m/h/d/w (e.g., 30d, 6h, 2w, 90m)",
                unit
            ))
        }
    };

    Ok(seconds)
}

/// Load config file with proper error reporting
///
/// Returns None if file doesn't exist, but warns on parse errors
fn load_config_with_warning() -> Option<ConfigFile> {
    match ConfigFile::load_default() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Warning: Failed to load config file: {}", e);
            eprintln!("Using default settings. Fix the config or run: ethcli config path");
            None
        }
    }
}

use ethcli::utils::format::format_thousands;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up logging
    let filter = match cli.verbose {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(EnvFilter::new(filter))
        .init();

    // Parse chain once for use in handlers
    let chain: Chain = cli.chain.parse()?;

    // Load config file and merge etherscan key (CLI takes precedence over config)
    let config_file = load_config_with_warning();
    let etherscan_key = cli.etherscan_key.clone().or_else(|| {
        config_file
            .as_ref()
            .and_then(|c| c.etherscan_api_key.clone())
    });

    // Handle subcommands
    match &cli.command {
        Commands::Logs(args) => {
            return run_logs(args, &cli).await;
        }
        Commands::Tx(args) => {
            return handle_tx(args, &cli).await;
        }
        Commands::Account { action } => {
            return ethcli::cli::account::handle(action, chain, etherscan_key.clone(), cli.quiet)
                .await;
        }
        Commands::Address { action } => {
            return ethcli::cli::address::handle(action, cli.quiet);
        }
        Commands::Contract { action } => {
            return ethcli::cli::contract::handle(action, chain, etherscan_key.clone(), cli.quiet)
                .await;
        }
        Commands::Token { action } => {
            return ethcli::cli::token::handle(action, chain, etherscan_key.clone(), cli.quiet)
                .await;
        }
        Commands::Gas { action } => {
            return ethcli::cli::gas::handle(action, chain, etherscan_key.clone(), cli.quiet).await;
        }
        Commands::Sig { action } => {
            return ethcli::cli::sig::handle(action, chain, etherscan_key.clone(), cli.quiet).await;
        }
        Commands::Endpoints { action } => {
            return handle_endpoints(action, &cli).await;
        }
        Commands::Config { action } => {
            return handle_config(action).await;
        }
        Commands::Cast { action } => {
            return ethcli::cli::cast::handle(action);
        }
        Commands::Rpc { action, rpc_url } => {
            return ethcli::cli::rpc::handle(action, chain, rpc_url.clone(), cli.quiet).await;
        }
        Commands::Ens { action, rpc_url } => {
            return ethcli::cli::ens::handle(action, chain, rpc_url.clone(), cli.quiet).await;
        }
        Commands::Simulate { action } => {
            return ethcli::cli::simulate::handle(action, chain, cli.quiet).await;
        }
        Commands::Tenderly { action } => {
            return ethcli::cli::tenderly::handle(action, chain, cli.quiet).await;
        }

        Commands::Update { install } => {
            return ethcli::cli::update::handle(*install, cli.quiet).await;
        }

        Commands::Doctor => {
            return ethcli::cli::doctor::handle(cli.quiet).await;
        }
    }
}

/// Run logs command with LogsArgs
async fn run_logs(args: &LogsArgs, cli: &Cli) -> anyhow::Result<()> {
    let contract = &args.contract;

    // Parse chain
    let chain: Chain = cli.chain.parse()?;

    // Parse output format
    let format: OutputFormat = args.format.parse()?;

    // Parse to_block
    let to_block = if args.to_block.to_lowercase() == "latest" {
        ethcli::BlockNumber::Latest
    } else {
        ethcli::BlockNumber::Number(args.to_block.parse()?)
    };

    // Load config file for additional settings
    let config_file = load_config_with_warning();

    // Get Etherscan API key
    let etherscan_key = cli.etherscan_key.clone().or_else(|| {
        config_file
            .as_ref()
            .and_then(|c| c.etherscan_api_key.clone())
    });

    // Apply defaults: CLI > config file > hardcoded defaults
    let concurrency = args.concurrency.unwrap_or_else(|| {
        config_file
            .as_ref()
            .map(|c| c.settings.concurrency)
            .unwrap_or(5)
    });

    // Build RPC config (with optional chunk_size override)
    let rpc_config = build_rpc_config_from_logs_args_full(
        &args.rpc,
        &args.proxy,
        &config_file,
        concurrency,
        args.chunk_size,
    )?;

    // Parse from_block (can be number, "auto", --since, or omitted for auto-detect)
    let (from_block, auto_from_block) = if let Some(since_str) = &args.since {
        // Parse the duration string and calculate from_block
        let duration_secs = parse_duration_string(since_str)?;
        let blocks_back = chain.blocks_for_duration(duration_secs);

        // Get current block number via a quick RPC call
        let quick_pool = RpcPool::new(chain, &rpc_config)?;
        let current_block = quick_pool.get_block_number().await?;

        let target_block = current_block.saturating_sub(blocks_back);

        if !cli.quiet {
            eprintln!(
                "Using --since {}: ~{} blocks back from {} to block {}",
                since_str, blocks_back, current_block, target_block
            );
        }

        (target_block, false)
    } else {
        match &args.from_block {
            Some(s) if s.to_lowercase() == "auto" => (0, true),
            Some(s) => (s.parse::<u64>()?, false),
            None => (0, true), // Default to auto-detect from contract creation
        }
    };

    // Build main config
    let mut builder = Config::builder()
        .chain(chain)
        .contract(contract)
        .from_block(from_block)
        .to_block(to_block)
        .output_format(format)
        .concurrency(concurrency)
        .raw(args.raw)
        .resume(args.resume)
        .quiet(cli.quiet)
        .verbosity(cli.verbose)
        .auto_from_block(auto_from_block)
        .rpc_config(rpc_config);

    // Add event filters (supports multiple: -e Transfer -e Approval)
    for event in &args.event {
        builder = builder.event(event);
    }

    if let Some(abi) = &args.abi {
        builder = builder.abi_path(abi);
    }

    if let Some(output) = &args.output {
        builder = builder.output_path(output);
    }

    if let Some(checkpoint) = &args.checkpoint {
        builder = builder.checkpoint_path(checkpoint);
    }

    if let Some(key) = etherscan_key {
        builder = builder.etherscan_key(key);
    }

    let config = builder.build()?;

    // Create output writer early for streaming mode
    let mut writer = ethcli::create_writer(format, args.output.as_deref())?;

    if !cli.quiet {
        eprintln!("Connecting to {} endpoints...", chain.display_name());
    }

    let start = Instant::now();
    let (total_logs, stats) = if args.resume {
        // Use streaming mode with checkpoint support
        run_streaming_fetch(args, cli, config, &mut writer).await?
    } else {
        // Use batch mode (faster for smaller queries)
        run_batch_fetch_logs(args, cli, config, &mut writer).await?
    };
    let elapsed = start.elapsed();

    writer.finalize()?;

    // Report failures
    if !stats.is_complete() {
        if args.strict {
            eprintln!(
                "Error: {} of {} chunks failed (--strict mode)",
                stats.chunks_failed, stats.chunks_total
            );
            for (from, to, err) in &stats.failed_ranges {
                eprintln!("  - Blocks {}-{}: {}", from, to, err);
            }
            return Err(anyhow::anyhow!(
                "Fetch incomplete: {} chunks failed",
                stats.chunks_failed
            ));
        } else {
            eprintln!(
                "Warning: {} of {} chunks failed ({:.1}% success rate)",
                stats.chunks_failed,
                stats.chunks_total,
                stats.success_rate()
            );
            if cli.verbose > 0 {
                for (from, to, err) in &stats.failed_ranges {
                    eprintln!("  - Blocks {}-{}: {}", from, to, err);
                }
            }
        }
    }

    if !cli.quiet {
        let status = if stats.is_complete() {
            String::new()
        } else {
            format!(" (incomplete: {} chunks failed)", stats.chunks_failed)
        };
        let mode = if args.resume { " [streaming]" } else { "" };
        eprintln!(
            "Fetched {} logs in {:.2}s{}{}",
            total_logs,
            elapsed.as_secs_f64(),
            status,
            mode
        );
    }

    Ok(())
}

/// Run fetch in batch mode (loads all into memory, faster for small queries)
async fn run_batch_fetch_logs(
    args: &LogsArgs,
    cli: &Cli,
    config: Config,
    writer: &mut Box<dyn ethcli::OutputWriter>,
) -> anyhow::Result<(usize, FetchStats)> {
    let fetcher = LogFetcher::new(config).await?;

    if !cli.quiet {
        eprintln!("Using {} RPC endpoints", fetcher.endpoint_count());
    }

    // Set up progress bar
    let pb = if !cli.quiet {
        let pb = ProgressBar::new(100);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}% ({msg})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    } else {
        None
    };

    let pb_clone = pb.clone();
    // Select an endpoint for timestamp fetching before moving the fetcher
    let endpoint_for_timestamps = fetcher.pool().select_archive_endpoints(1);
    let fetcher = fetcher.with_progress(move |progress: FetchProgress| {
        if let Some(ref pb) = pb_clone {
            pb.set_position(progress.percent as u64);
            pb.set_message(format!(
                "{} logs, {:.0} blocks/s",
                progress.logs_fetched, progress.blocks_per_second
            ));
        }
    });

    let mut result = fetcher.fetch_all().await?;

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    // Add timestamps if requested
    if args.timestamps {
        if let FetchLogs::Decoded(ref mut logs) = result.logs {
            if !cli.quiet {
                eprintln!("Fetching block timestamps for {} logs...", logs.len());
            }
            add_timestamps_to_logs(logs, &endpoint_for_timestamps).await?;
        }
    }

    let total_logs = result.len();
    let stats = result.stats.clone();

    writer.write_logs(&result)?;

    Ok((total_logs, stats))
}

/// Fetch block timestamps and add them to logs (parallel fetching)
async fn add_timestamps_to_logs(
    logs: &mut [DecodedLog],
    endpoints: &[Endpoint],
) -> anyhow::Result<()> {
    // Collect unique block numbers
    let mut block_numbers: Vec<u64> = logs.iter().map(|l| l.block_number).collect();
    block_numbers.sort_unstable();
    block_numbers.dedup();

    if block_numbers.is_empty() {
        return Ok(());
    }

    // Get an endpoint
    let endpoint = endpoints
        .first()
        .ok_or_else(|| anyhow::anyhow!("No RPC endpoints available for timestamp fetching"))?;
    let provider = endpoint.provider();

    // Fetch timestamps for unique blocks in parallel (batches of 10)
    let mut timestamps: HashMap<u64, u64> = HashMap::new();
    let mut failed_count = 0usize;

    // Process in batches to avoid overwhelming the RPC
    const BATCH_SIZE: usize = 50;
    for batch in block_numbers.chunks(BATCH_SIZE) {
        let futures: Vec<_> = batch
            .iter()
            .map(|&block_num| {
                let provider = provider.clone();
                async move {
                    let result = provider
                        .get_block_by_number(alloy::eips::BlockNumberOrTag::Number(block_num))
                        .await;
                    (block_num, result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        for (block_num, result) in results {
            match result {
                Ok(Some(block)) => {
                    timestamps.insert(block_num, block.header.timestamp);
                }
                Ok(None) => {
                    // Block not found, skip
                    failed_count += 1;
                }
                Err(_) => {
                    // Log error but continue
                    failed_count += 1;
                }
            }
        }
    }

    // Report failures if any
    if failed_count > 0 {
        eprintln!(
            "Warning: Failed to fetch timestamps for {} of {} blocks",
            failed_count,
            block_numbers.len()
        );
    }

    // Update logs with timestamps
    for log in logs.iter_mut() {
        if let Some(&ts) = timestamps.get(&log.block_number) {
            log.timestamp = Some(ts);
        }
    }

    Ok(())
}

/// Run fetch in streaming mode (writes incrementally, supports resume)
async fn run_streaming_fetch(
    args: &LogsArgs,
    cli: &Cli,
    config: Config,
    writer: &mut Box<dyn OutputWriter>,
) -> anyhow::Result<(usize, FetchStats)> {
    let fetcher = StreamingFetcher::new(config.clone()).await?;

    // Enable checkpointing if path specified or use default
    let checkpoint_path = args.checkpoint.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            ".eth-log-fetch-{}.checkpoint",
            &args.contract[..8.min(args.contract.len())]
        ))
    });

    // Get an endpoint for timestamp fetching before moving the fetcher
    let endpoint_for_timestamps = if args.timestamps {
        Some(fetcher.pool().select_archive_endpoints(1))
    } else {
        None
    };

    let mut fetcher = fetcher.with_checkpoint(&checkpoint_path)?;

    if !cli.quiet {
        eprintln!(
            "Using {} RPC endpoints (streaming mode)",
            fetcher.endpoint_count()
        );
        eprintln!("Checkpoint: {}", checkpoint_path.display());
        if args.timestamps {
            eprintln!("Timestamps: enabled (fetching per batch)");
        }
    }

    // Set up progress (simplified for streaming)
    let pb = if !cli.quiet {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        pb.enable_steady_tick(std::time::Duration::from_millis(100));
        Some(pb)
    } else {
        None
    };

    let mut total_logs = 0usize;

    // Stream logs and write incrementally
    let stats = fetcher
        .fetch_streaming(|mut result| {
            // Add timestamps if requested
            if let Some(ref endpoints) = endpoint_for_timestamps {
                if let FetchLogs::Decoded(ref mut logs) = result.logs {
                    // Use block_in_place to safely run async code from sync callback
                    // This yields the current thread to the runtime, avoiding deadlock
                    tokio::task::block_in_place(|| {
                        let rt = tokio::runtime::Handle::current();
                        rt.block_on(async {
                            if let Err(e) = add_timestamps_to_logs(logs, endpoints).await {
                                eprintln!("Warning: Failed to fetch timestamps: {}", e);
                            }
                        });
                    });
                }
            }

            total_logs += result.len();

            if let Some(ref pb) = pb {
                pb.set_message(format!("{} logs fetched", total_logs));
            }

            writer.write_logs(&result)?;
            Ok(())
        })
        .await?;

    if let Some(ref pb) = pb {
        pb.finish_and_clear();
    }

    Ok((total_logs, stats))
}

/// Build RPC config from LogsArgs (has embedded RPC/Proxy args)
fn build_rpc_config_from_logs_args(
    rpc: &RpcArgs,
    proxy: &ProxyArgs,
    config_file: &Option<ConfigFile>,
    concurrency: usize,
) -> anyhow::Result<RpcConfig> {
    let mut rpc_config = RpcConfig::default();

    // Custom endpoints from CLI
    if !rpc.rpc_urls.is_empty() {
        rpc_config.endpoints = rpc.rpc_urls.iter().map(EndpointConfig::new).collect();
    }

    // Load from file
    if let Some(path) = &rpc.rpc_file {
        let content = std::fs::read_to_string(path)?;
        for line in content.lines() {
            let url = line.trim();
            if !url.is_empty() && !url.starts_with('#') {
                rpc_config.endpoints.push(EndpointConfig::new(url));
            }
        }
    }

    // Add custom endpoints from config file
    if let Some(cf) = config_file {
        for endpoint in &cf.endpoints {
            if !rpc_config.endpoints.iter().any(|e| e.url == endpoint.url) {
                rpc_config.endpoints.push(endpoint.clone());
            }
        }
    }

    // Additional endpoints
    rpc_config.add_endpoints = rpc.add_rpc.clone();

    // Excluded endpoints from CLI
    rpc_config.exclude_endpoints = rpc.exclude_rpc.clone();

    // Add disabled endpoints from config file
    if let Some(cf) = config_file {
        rpc_config
            .exclude_endpoints
            .extend(cf.disabled_endpoints.urls.clone());
    }

    rpc_config.min_priority = rpc.min_priority;

    // Apply defaults: CLI > config file > hardcoded defaults
    rpc_config.timeout_secs = rpc.timeout.unwrap_or_else(|| {
        config_file
            .as_ref()
            .map(|c| c.settings.timeout_seconds)
            .unwrap_or(30)
    });

    rpc_config.max_retries = rpc.retries.unwrap_or_else(|| {
        config_file
            .as_ref()
            .map(|c| c.settings.retry_attempts)
            .unwrap_or(3)
    });

    rpc_config.concurrency = concurrency;

    // Proxy config
    if proxy.proxy.is_some() || proxy.proxy_file.is_some() {
        rpc_config.proxy = Some(ProxyConfig {
            url: proxy.proxy.clone(),
            file: proxy.proxy_file.clone(),
            rotate_per_request: proxy.proxy_rotate,
        });
    } else if let Some(cf) = config_file {
        rpc_config.proxy = cf.proxy_config();
    }

    Ok(rpc_config)
}

/// Build RPC config from LogsArgs with chunk_size support
fn build_rpc_config_from_logs_args_full(
    rpc: &RpcArgs,
    proxy: &ProxyArgs,
    config_file: &Option<ConfigFile>,
    concurrency: usize,
    chunk_size: Option<u64>,
) -> anyhow::Result<RpcConfig> {
    let mut rpc_config = build_rpc_config_from_logs_args(rpc, proxy, config_file, concurrency)?;
    rpc_config.chunk_size = chunk_size;
    Ok(rpc_config)
}

/// Build default RPC config (for tx command that doesn't have RPC args)
fn build_default_rpc_config(config_file: &Option<ConfigFile>) -> anyhow::Result<RpcConfig> {
    let mut rpc_config = RpcConfig::default();

    // Add custom endpoints from config file
    if let Some(cf) = config_file {
        for endpoint in &cf.endpoints {
            if !rpc_config.endpoints.iter().any(|e| e.url == endpoint.url) {
                rpc_config.endpoints.push(endpoint.clone());
            }
        }
    }

    // Add disabled endpoints from config file
    if let Some(cf) = config_file {
        rpc_config
            .exclude_endpoints
            .extend(cf.disabled_endpoints.urls.clone());
    }

    // Apply config file defaults
    if let Some(cf) = config_file {
        rpc_config.timeout_secs = cf.settings.timeout_seconds;
        rpc_config.max_retries = cf.settings.retry_attempts;
        rpc_config.concurrency = cf.settings.concurrency;
        rpc_config.proxy = cf.proxy_config();
    }

    Ok(rpc_config)
}

async fn handle_endpoints(action: &EndpointCommands, cli: &Cli) -> anyhow::Result<()> {
    use ethcli::{optimize_endpoint, NodeType};

    match action {
        EndpointCommands::List {
            archive,
            debug,
            chain: chain_filter,
            detailed,
        } => {
            // Load endpoints from config file
            let config_file = load_config_with_warning();
            let endpoints: Vec<EndpointConfig> =
                config_file.map(|cf| cf.endpoints).unwrap_or_default();

            // Filter by chain if specified
            let chain_filter_parsed: Option<Chain> =
                chain_filter.as_ref().map(|c| c.parse()).transpose()?;

            let filtered: Vec<_> = endpoints
                .iter()
                .filter(|ep| {
                    // Filter by chain
                    if let Some(ref chain) = chain_filter_parsed {
                        if ep.chain != *chain {
                            return false;
                        }
                    }
                    // Filter by archive
                    if *archive && ep.node_type != NodeType::Archive {
                        return false;
                    }
                    // Filter by debug
                    if *debug && !ep.has_debug {
                        return false;
                    }
                    // Only enabled endpoints
                    ep.enabled
                })
                .collect();

            if filtered.is_empty() {
                println!("No endpoints found matching filters.");
                return Ok(());
            }

            println!(
                "RPC ENDPOINTS ({} total, {} shown)\n",
                endpoints.len(),
                filtered.len()
            );

            // Group by chain
            let mut by_chain: std::collections::BTreeMap<String, Vec<_>> =
                std::collections::BTreeMap::new();
            for ep in &filtered {
                by_chain
                    .entry(ep.chain.name().to_string())
                    .or_default()
                    .push(*ep);
            }

            for (chain_name, mut eps) in by_chain {
                // Sort by priority descending
                eps.sort_by(|a, b| b.priority.cmp(&a.priority));

                println!("=== {} ({}) ===", chain_name.to_uppercase(), eps.len());
                for ep in eps {
                    let node_type_badge = match ep.node_type {
                        NodeType::Archive => "[ARCHIVE]",
                        NodeType::Full => "[FULL]",
                        NodeType::Unknown => "[?]",
                    };
                    let debug_badge = if ep.has_debug { "[DEBUG]" } else { "" };

                    println!(
                        "  P{} {} {} {}",
                        ep.priority, node_type_badge, debug_badge, ep.url
                    );

                    if *detailed {
                        println!(
                            "      Block range: {} | Max logs: {}",
                            if ep.max_block_range == 0 {
                                "unlimited".to_string()
                            } else {
                                format_thousands(ep.max_block_range)
                            },
                            if ep.max_logs == 0 {
                                "unlimited".to_string()
                            } else {
                                format_thousands(ep.max_logs as u64)
                            }
                        );
                        if let Some(note) = &ep.note {
                            println!("      Note: {}", note);
                        }
                        if let Some(tested) = &ep.last_tested {
                            println!("      Last tested: {}", tested);
                        }
                    }
                }
                println!();
            }
        }

        EndpointCommands::Add {
            url,
            chain: chain_override,
            no_optimize,
        } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();

            // Check if already exists
            if config.endpoints.iter().any(|e| e.url == *url) {
                println!("Endpoint already exists: {}", url);
                return Ok(());
            }

            let endpoint = if *no_optimize {
                // Just add with defaults
                let chain: Chain = if let Some(c) = chain_override {
                    c.parse()?
                } else {
                    cli.chain.parse()?
                };
                EndpointConfig::new(url).with_chain(chain)
            } else {
                // Optimize to detect capabilities
                println!("Optimizing endpoint: {}\n", url);

                let expected_chain: Option<Chain> =
                    chain_override.as_ref().map(|c| c.parse()).transpose()?;

                let result = optimize_endpoint(url, expected_chain, 10).await?;

                if !result.connectivity_ok {
                    println!(
                        "Failed to connect: {}",
                        result.error.unwrap_or_else(|| "Unknown error".to_string())
                    );
                    return Ok(());
                }

                println!(
                    "  Chain: {} (ID: {})",
                    result.config.chain.name(),
                    result.chain_id
                );
                println!("  Current block: {}", result.current_block);
                println!("  Node type: {}", result.config.node_type);
                println!(
                    "  Debug namespace: {}",
                    if result.config.has_debug { "Yes" } else { "No" }
                );
                println!(
                    "  Max block range: {}",
                    format_thousands(result.config.max_block_range)
                );
                println!(
                    "  Max logs: {}",
                    format_thousands(result.config.max_logs as u64)
                );

                result.config
            };

            config.endpoints.push(endpoint);
            config.save_default()?;
            println!("\nEndpoint added to config.");
        }

        EndpointCommands::Remove { url } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();

            let initial_len = config.endpoints.len();
            config.endpoints.retain(|e| e.url != *url);

            if config.endpoints.len() == initial_len {
                println!("Endpoint not found: {}", url);
                return Ok(());
            }

            config.save_default()?;
            println!("Endpoint removed from config: {}", url);
        }

        EndpointCommands::Optimize {
            url,
            all,
            chain: chain_filter,
            timeout,
        } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();

            let chain_filter_parsed: Option<Chain> =
                chain_filter.as_ref().map(|c| c.parse()).transpose()?;

            let indices_to_optimize: Vec<usize> = if let Some(target_url) = url {
                // Find specific endpoint
                config
                    .endpoints
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| e.url == *target_url)
                    .map(|(i, _)| i)
                    .collect()
            } else if *all {
                // All endpoints, optionally filtered by chain
                config
                    .endpoints
                    .iter()
                    .enumerate()
                    .filter(|(_, e)| {
                        if let Some(ref chain) = chain_filter_parsed {
                            e.chain == *chain
                        } else {
                            true
                        }
                    })
                    .map(|(i, _)| i)
                    .collect()
            } else {
                println!("Specify a URL or use --all to optimize all endpoints.");
                return Ok(());
            };

            if indices_to_optimize.is_empty() {
                println!("No endpoints found to optimize.");
                return Ok(());
            }

            println!("Optimizing {} endpoint(s)...\n", indices_to_optimize.len());

            for idx in indices_to_optimize {
                let ep_url = config.endpoints[idx].url.clone();
                let expected_chain = Some(config.endpoints[idx].chain);

                print!("Testing {}... ", ep_url);
                std::io::Write::flush(&mut std::io::stdout())?;

                match optimize_endpoint(&ep_url, expected_chain, *timeout).await {
                    Ok(result) => {
                        if result.connectivity_ok {
                            println!("OK");
                            println!(
                                "  {} | {} | range:{} | logs:{}",
                                result.config.node_type,
                                if result.config.has_debug {
                                    "debug"
                                } else {
                                    "no-debug"
                                },
                                format_thousands(result.config.max_block_range),
                                format_thousands(result.config.max_logs as u64)
                            );
                            config.endpoints[idx] = result.config;
                        } else {
                            println!(
                                "FAILED: {}",
                                result.error.unwrap_or_else(|| "Unknown".to_string())
                            );
                        }
                    }
                    Err(e) => {
                        println!("ERROR: {}", e);
                    }
                }
            }

            config.save_default()?;
            println!("\nConfig updated.");
        }

        EndpointCommands::Test { url } => {
            println!("Testing endpoint: {}\n", url);

            // Test connectivity
            print!("[1/3] Connectivity.............. ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let rpc_config = RpcConfig {
                endpoints: vec![EndpointConfig::new(url)],
                timeout_secs: 10,
                ..Default::default()
            };

            let chain: Chain = cli.chain.parse()?;
            let pool = match RpcPool::new(chain, &rpc_config) {
                Ok(p) => {
                    println!("OK");
                    p
                }
                Err(e) => {
                    println!("FAILED: {}", e);
                    return Ok(());
                }
            };

            // Test block number
            print!("[2/3] Current block............. ");
            std::io::Write::flush(&mut std::io::stdout())?;

            match pool.get_block_number().await {
                Ok(block) => println!("Block {}", block),
                Err(e) => {
                    println!("FAILED: {}", e);
                    return Ok(());
                }
            }

            // Test archive support
            print!("[3/3] Archive support........... ");
            std::io::Write::flush(&mut std::io::stdout())?;

            // Create endpoint directly to test
            let endpoint = Endpoint::new(EndpointConfig::new(url), 10, None)?;

            match endpoint.test_archive_support().await {
                Ok(true) => println!("YES (historical state accessible)"),
                Ok(false) => println!("NO (pruned node)"),
                Err(e) => println!("UNKNOWN: {}", e),
            }

            println!("\nEndpoint test complete.");
        }

        EndpointCommands::Enable { url } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();

            if let Some(ep) = config.endpoints.iter_mut().find(|e| e.url == *url) {
                ep.enabled = true;
                config.save_default()?;
                println!("Endpoint enabled: {}", url);
            } else {
                println!("Endpoint not found: {}", url);
            }
        }

        EndpointCommands::Disable { url } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();

            if let Some(ep) = config.endpoints.iter_mut().find(|e| e.url == *url) {
                ep.enabled = false;
                config.save_default()?;
                println!("Endpoint disabled: {}", url);
            } else {
                println!("Endpoint not found: {}", url);
            }
        }
    }

    Ok(())
}

async fn handle_config(action: &ConfigCommands) -> anyhow::Result<()> {
    match action {
        ConfigCommands::Init { force } => {
            let path = ConfigFile::default_path();
            if path.exists() && !force {
                anyhow::bail!(
                    "Config file already exists at: {}\nUse --force to overwrite.",
                    path.display()
                );
            }

            let template = generate_config_template();

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(&path, template)?;

            // Set restrictive permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
            }

            println!("Config file created at: {}", path.display());
            println!("\nEdit the file to:");
            println!("  - Add your Etherscan API key");
            println!("  - Add your Tenderly credentials (optional)");
            println!("  - Add/remove RPC endpoints");
        }

        ConfigCommands::Path => {
            println!("{}", ConfigFile::default_path().display());
        }

        ConfigCommands::SetEtherscanKey { key } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();
            config.set_etherscan_key(key.clone())?;
            println!("Etherscan API key saved to config file.");
        }

        ConfigCommands::SetTenderly {
            key,
            account,
            project,
        } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();
            config.set_tenderly(key.clone(), account.clone(), project.clone())?;
            println!("Tenderly credentials saved to config file.");
            println!("  Account: {}", account);
            println!("  Project: {}", project);
        }

        ConfigCommands::AddDebugRpc { url } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();
            config.add_debug_rpc(url.clone())?;
            println!("Debug RPC URL added to config file.");
            println!("  URL: {}", url);
        }

        ConfigCommands::RemoveDebugRpc { url } => {
            let mut config = ConfigFile::load_default()?.unwrap_or_default();
            config.remove_debug_rpc(url)?;
            println!("Debug RPC URL removed from config file.");
        }

        ConfigCommands::Show => {
            let path = ConfigFile::default_path();
            if path.exists() {
                let content = std::fs::read_to_string(&path)?;
                println!("# {}\n", path.display());
                println!("{}", content);
            } else {
                println!("No config file found at: {}", path.display());
                println!("\nCreate one with:");
                println!("  ethcli config set-etherscan-key YOUR_KEY");
                println!(
                    "  ethcli config set-tenderly --key KEY --account ACCOUNT --project PROJECT"
                );
                println!("  ethcli config add-debug-rpc URL");
            }
        }
    }

    Ok(())
}

fn generate_config_template() -> String {
    r#"# ethcli configuration file
# Documentation: https://github.com/yldfi/ethcli

# =============================================================================
# General Settings
# =============================================================================
[settings]
# Number of concurrent RPC requests (default: 5)
concurrency = 5

# Request timeout in seconds (default: 30)
timeout_seconds = 30

# Number of retry attempts on failure (default: 3)
retry_attempts = 3

# Save checkpoint every N blocks when fetching logs (default: 1000)
checkpoint_interval = 1000

# =============================================================================
# API Keys (optional but recommended)
# =============================================================================

# Etherscan API key - increases rate limits for ABI fetching
# Get one free at: https://etherscan.io/apis
# etherscan_api_key = "YOUR_ETHERSCAN_API_KEY"

# =============================================================================
# Tenderly Configuration (optional)
# =============================================================================
# Required for: ethcli tenderly, ethcli simulate --via tenderly
# Get credentials at: https://dashboard.tenderly.co/account/authorization

# [tenderly]
# access_key = "YOUR_TENDERLY_ACCESS_KEY"
# account = "your-account-slug"
# project = "your-project-slug"

# =============================================================================
# Debug RPC URLs (optional)
# =============================================================================
# URLs with debug_traceCall support for transaction tracing
debug_rpc_urls = []

# =============================================================================
# RPC Endpoints
# =============================================================================
# Add your own endpoints or use the public ones below.
# Higher priority (1-15) = preferred. Endpoints are auto-tested and optimized.
#
# Fields:
#   url            - RPC endpoint URL (required)
#   chain          - ethereum, polygon, arbitrum, optimism, base, bsc, avalanche
#   priority       - Selection preference 1-15 (higher = preferred)
#   max_block_range - Max blocks per eth_getLogs query
#   max_logs       - Max logs returned per query
#   has_debug      - Supports debug_traceCall
#   has_trace      - Supports trace_call (Erigon/OpenEthereum)
#   enabled        - Set to false to disable
#   note           - Optional description

# -----------------------------------------------------------------------------
# Ethereum Mainnet
# -----------------------------------------------------------------------------
[[endpoints]]
url = "https://eth-mainnet.public.blastapi.io"
chain = "ethereum"
priority = 10
max_block_range = 18303
max_logs = 200000
note = "Excellent - highest log limit"

[[endpoints]]
url = "https://ethereum.publicnode.com"
chain = "ethereum"
priority = 8
max_block_range = 44864
max_logs = 20000

[[endpoints]]
url = "https://eth.drpc.org"
chain = "ethereum"
priority = 6
max_block_range = 10000
max_logs = 5000

# -----------------------------------------------------------------------------
# Polygon
# -----------------------------------------------------------------------------
[[endpoints]]
url = "https://polygon-mainnet.public.blastapi.io"
chain = "polygon"
priority = 10
max_block_range = 100000
max_logs = 10000

[[endpoints]]
url = "https://polygon.publicnode.com"
chain = "polygon"
priority = 5
max_block_range = 10000
max_logs = 10000

# -----------------------------------------------------------------------------
# Arbitrum
# -----------------------------------------------------------------------------
[[endpoints]]
url = "https://arbitrum-mainnet.public.blastapi.io"
chain = "arbitrum"
priority = 10
max_block_range = 100000
max_logs = 10000

[[endpoints]]
url = "https://arb1.arbitrum.io/rpc"
chain = "arbitrum"
priority = 5
max_block_range = 1999024
max_logs = 10000

# -----------------------------------------------------------------------------
# Base
# -----------------------------------------------------------------------------
[[endpoints]]
url = "https://base-mainnet.public.blastapi.io"
chain = "base"
priority = 10
max_block_range = 100000
max_logs = 10000

[[endpoints]]
url = "https://mainnet.base.org"
chain = "base"
priority = 8
max_block_range = 10000
max_logs = 10000
note = "Official Base RPC"

# -----------------------------------------------------------------------------
# Optimism
# -----------------------------------------------------------------------------
[[endpoints]]
url = "https://optimism-mainnet.public.blastapi.io"
chain = "optimism"
priority = 10
max_block_range = 100000
max_logs = 10000

[[endpoints]]
url = "https://mainnet.optimism.io"
chain = "optimism"
priority = 8
max_block_range = 10000
max_logs = 10000
note = "Official Optimism RPC"

# =============================================================================
# Disabled Endpoints
# =============================================================================
# URLs listed here will be skipped even if defined above
[disabled_endpoints]
urls = []

# =============================================================================
# Proxy Configuration (optional)
# =============================================================================
# [proxy]
# default = "socks5://127.0.0.1:9050"
# rotate = false
# file = "/path/to/proxy-list.txt"
"#
    .to_string()
}

async fn handle_tx(args: &TxArgs, cli: &Cli) -> anyhow::Result<()> {
    use std::io::BufRead;

    // Collect all hashes from various sources
    let mut all_hashes: Vec<String> = args.hashes.clone();

    // Read from file if specified
    if let Some(path) = &args.file {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let hash = line.trim();
            if !hash.is_empty() && !hash.starts_with('#') {
                all_hashes.push(hash.to_string());
            }
        }
    }

    // Read from stdin if specified
    if args.stdin {
        let stdin = std::io::stdin();
        let reader = stdin.lock();
        for line in reader.lines() {
            let line = line?;
            let hash = line.trim();
            if !hash.is_empty() && !hash.starts_with('#') {
                all_hashes.push(hash.to_string());
            }
        }
    }

    if all_hashes.is_empty() {
        return Err(anyhow::anyhow!(
            "No transaction hashes provided. Use positional args, --file, or --stdin"
        ));
    }

    // Parse chain
    let chain: Chain = cli.chain.parse()?;

    // Load config file for additional settings
    let config_file = load_config_with_warning();

    // Build RPC config with defaults
    let rpc_config = build_default_rpc_config(&config_file)?;

    // Create RPC pool
    let pool = RpcPool::new(chain, &rpc_config)?;

    let tx_count = all_hashes.len();
    let endpoint_count = pool.endpoint_count();

    if !cli.quiet {
        eprintln!(
            "Analyzing {} transaction{} using {} RPC endpoints (batch size: {})...",
            tx_count,
            if tx_count == 1 { "" } else { "s" },
            endpoint_count,
            args.batch_size
        );
    }

    // Create analyzer
    let analyzer = std::sync::Arc::new(TxAnalyzer::new(pool, chain)?);

    let start = Instant::now();
    let mut all_analyses = Vec::new();
    let mut total_events = 0;
    let mut total_transfers = 0;
    let mut failed_count = 0;

    // Process in batches for parallelism
    for (batch_idx, batch) in all_hashes.chunks(args.batch_size).enumerate() {
        let batch_start = batch_idx * args.batch_size;

        if !cli.quiet && tx_count > 1 {
            eprint!(
                "\r[{}-{}/{}] Processing batch...",
                batch_start + 1,
                (batch_start + batch.len()).min(tx_count),
                tx_count
            );
        }

        // Process batch in parallel
        let enrich = args.enrich;
        let futures: Vec<_> = batch
            .iter()
            .enumerate()
            .map(|(i, hash)| {
                let analyzer = analyzer.clone();
                let hash = if hash.starts_with("0x") || hash.starts_with("0X") {
                    hash.to_string()
                } else {
                    format!("0x{}", hash)
                };
                let idx = batch_start + i;

                async move {
                    let result = if enrich {
                        analyzer.analyze_enriched(&hash).await
                    } else {
                        analyzer.analyze(&hash).await
                    };
                    (idx, hash.clone(), result)
                }
            })
            .collect();

        let results = futures::future::join_all(futures).await;

        for (idx, hash, result) in results {
            match result {
                Ok(analysis) => {
                    total_events += analysis.events.len();
                    total_transfers += analysis.token_flows.len();
                    all_analyses.push((idx, analysis));
                }
                Err(e) => {
                    failed_count += 1;
                    if !cli.quiet {
                        eprintln!(
                            "\n[{}] Error: {} - {}",
                            idx + 1,
                            &hash[..hash.len().min(12)],
                            e
                        );
                    }
                }
            }
        }
    }

    // Sort by original index to maintain order
    all_analyses.sort_by_key(|(idx, _)| *idx);
    let analyses: Vec<_> = all_analyses.into_iter().map(|(_, a)| a).collect();

    if !cli.quiet && tx_count > 1 {
        eprintln!(); // Clear the progress line
    }

    let elapsed = start.elapsed();

    // Output
    if args.output.is_json() {
        if analyses.len() == 1 {
            let json = serde_json::to_string_pretty(&analyses[0])?;
            println!("{}", json);
        } else {
            let json = serde_json::to_string_pretty(&analyses)?;
            println!("{}", json);
        }
    } else if args.output.is_ndjson() {
        // Newline-delimited JSON - one per line, good for streaming/large datasets
        for analysis in &analyses {
            let json = serde_json::to_string(analysis)?;
            println!("{}", json);
        }
    } else {
        // Pretty/table print
        for (i, analysis) in analyses.iter().enumerate() {
            if i > 0 {
                println!("\n{}", "=".repeat(80));
                println!();
            }
            println!("{}", format_analysis(analysis));
            // Add explorer link
            if let Some(explorer) = chain.explorer_url() {
                println!("\nExplorer: {}/tx/{:#x}", explorer, analysis.hash);
            }
        }
    }

    if !cli.quiet {
        let failed_msg = if failed_count > 0 {
            format!(", {} failed", failed_count)
        } else {
            String::new()
        };
        eprintln!(
            "\nAnalyzed {} transaction{} in {:.2}s ({} events, {} transfers{})",
            analyses.len(),
            if analyses.len() == 1 { "" } else { "s" },
            elapsed.as_secs_f64(),
            total_events,
            total_transfers,
            failed_msg
        );
        if tx_count > 10 {
            eprintln!(
                "Throughput: {:.1} tx/s",
                analyses.len() as f64 / elapsed.as_secs_f64()
            );
        }
    }

    Ok(())
}
