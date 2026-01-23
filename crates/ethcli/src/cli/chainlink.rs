//! Chainlink CLI commands
//!
//! Two modes of operation:
//! 1. RPC-based (default) - Query price feeds via RPC, no API key needed
//! 2. Data Streams - Low-latency streaming data, requires API key
//!
//! # Examples
//!
//! ```bash
//! # RPC-based (default, free)
//! ethcli chainlink price CVX
//! ethcli chainlink price ETH --block 18000000
//! ethcli chainlink feed CVX
//! ethcli chainlink oracles --chain ethereum
//!
//! # Data Streams (requires API key)
//! ethcli chainlink streams feeds
//! ethcli chainlink streams latest <feed_id>
//! ```

use crate::chainlink::{self, denominations, Aggregator, ChainlinkError, FeedRegistry, PriceData};
use crate::cli::rpc::parse_block_id;
use crate::cli::OutputFormat;
use crate::config::{Chain, ConfigFile, EndpointConfig};
use crate::rpc::Endpoint;
use alloy::primitives::Address;
use alloy::providers::Provider;
use clap::{Args, Subcommand};
use serde::Serialize;
use std::str::FromStr;

#[derive(Args)]
pub struct ChainlinkArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum ChainlinkCommands {
    /// Get price via RPC (Feed Registry or direct oracle)
    Price {
        /// Token symbol (ETH, CVX, etc.) or contract address
        token: String,

        /// Quote currency (usd, eth, btc)
        #[arg(long, default_value = "usd")]
        quote: String,

        /// Chain to query
        #[arg(long, short, default_value = "ethereum")]
        chain: String,

        /// Block number for historical price (requires archive node)
        #[arg(long, short)]
        block: Option<String>,

        /// Direct oracle address (bypasses Feed Registry)
        #[arg(long)]
        oracle: Option<String>,

        /// RPC URL override
        #[arg(long)]
        rpc_url: Option<String>,

        #[command(flatten)]
        args: ChainlinkArgs,
    },

    /// Get feed/oracle address for a token
    Feed {
        /// Token symbol or address
        token: String,

        /// Quote currency
        #[arg(long, default_value = "usd")]
        quote: String,

        /// Chain to query (Feed Registry only available on Ethereum mainnet)
        #[arg(long, short, default_value = "ethereum")]
        chain: String,

        /// RPC URL override
        #[arg(long)]
        rpc_url: Option<String>,

        #[command(flatten)]
        args: ChainlinkArgs,
    },

    /// List known oracle addresses
    Oracles {
        /// Filter by chain
        #[arg(long, short)]
        chain: Option<String>,

        #[command(flatten)]
        args: ChainlinkArgs,
    },

    /// Data Streams commands (requires API key)
    #[command(subcommand)]
    Streams(StreamsCommands),
}

/// Data Streams subcommands (require CHAINLINK_API_KEY)
#[derive(Subcommand)]
#[command(after_help = r#"CREDENTIALS:
    Requires API credentials from https://chain.link/data-streams
    Set CHAINLINK_API_KEY and CHAINLINK_USER_SECRET environment variables.

TERMS OF SERVICE:
    By using this feature, you agree to Chainlink's Terms of Service.
    See: https://chainlinklabs.com/terms
"#)]
pub enum StreamsCommands {
    /// List available data feeds
    Feeds {
        #[command(flatten)]
        args: ChainlinkArgs,
    },

    /// Get latest report for a feed
    Latest {
        /// Feed ID (hex string, e.g., 0x000359843a543ee2fe414dc14c7e7920ef10f4372990b79d6361cdc0dd1ba782)
        feed_id: String,

        #[command(flatten)]
        args: ChainlinkArgs,
    },

    /// Get report at a specific timestamp
    Report {
        /// Feed ID (hex string)
        feed_id: String,

        /// Unix timestamp (seconds)
        timestamp: u64,

        #[command(flatten)]
        args: ChainlinkArgs,
    },

    /// Get reports for multiple feeds at a timestamp
    Bulk {
        /// Feed IDs (comma-separated hex strings)
        #[arg(value_delimiter = ',')]
        feed_ids: Vec<String>,

        /// Unix timestamp (seconds)
        timestamp: u64,

        #[command(flatten)]
        args: ChainlinkArgs,
    },

    /// Get paginated reports for a feed
    History {
        /// Feed ID (hex string)
        feed_id: String,

        /// Start timestamp (seconds)
        timestamp: u64,

        /// Number of reports to fetch
        #[arg(long, default_value = "10")]
        limit: usize,

        #[command(flatten)]
        args: ChainlinkArgs,
    },
}

/// Price output for CLI
#[derive(Debug, Serialize)]
pub struct PriceOutput {
    pub token: String,
    pub quote: String,
    pub chain: String,
    pub price_usd: Option<f64>,
    pub raw_answer: String,
    pub decimals: u8,
    pub round_id: u64,
    pub updated_at: u64,
    pub feed_address: Option<String>,
    pub is_stale: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<String>,
    pub source: String,
}

impl From<PriceData> for PriceOutput {
    fn from(data: PriceData) -> Self {
        Self {
            token: String::new(),
            quote: String::new(),
            chain: String::new(),
            price_usd: data.to_f64(),
            raw_answer: data.answer.to_string(),
            decimals: data.decimals,
            round_id: data.round_id,
            updated_at: data.updated_at,
            feed_address: data.feed_address.map(|a| format!("{:#x}", a)),
            is_stale: data.is_stale(),
            block: None,
            source: "rpc".to_string(),
        }
    }
}

/// Oracle list output
#[derive(Debug, Serialize)]
pub struct OracleInfo {
    pub symbol: String,
    pub pair: String,
    pub address: String,
    pub chain: String,
}

/// Handle Chainlink commands
pub async fn handle(command: &ChainlinkCommands, quiet: bool) -> anyhow::Result<()> {
    match command {
        ChainlinkCommands::Price {
            token,
            quote,
            chain,
            block,
            oracle,
            rpc_url,
            args,
        } => handle_price(token, quote, chain, block.as_deref(), oracle.as_deref(), rpc_url.as_deref(), args, quiet).await,

        ChainlinkCommands::Feed {
            token,
            quote,
            chain,
            rpc_url,
            args,
        } => handle_feed(token, quote, chain, rpc_url.as_deref(), args, quiet).await,

        ChainlinkCommands::Oracles { chain, args } => handle_oracles(chain.as_deref(), args, quiet).await,

        ChainlinkCommands::Streams(streams_cmd) => handle_streams(streams_cmd, quiet).await,
    }
}

/// Handle price command
async fn handle_price(
    token: &str,
    quote: &str,
    chain: &str,
    block: Option<&str>,
    oracle: Option<&str>,
    rpc_url: Option<&str>,
    args: &ChainlinkArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    // Get provider
    let provider = get_provider(chain, rpc_url)?;

    if !quiet {
        if let Some(blk) = block {
            eprintln!("Fetching Chainlink price for {} at block {}...", token, blk);
        } else {
            eprintln!("Fetching Chainlink price for {}...", token);
        }
    }

    let price_data = if let Some(oracle_addr) = oracle {
        // Direct oracle query
        let addr = Address::from_str(oracle_addr)
            .map_err(|_| anyhow::anyhow!("Invalid oracle address: {}", oracle_addr))?;
        let aggregator = Aggregator::new(addr, provider);

        if let Some(blk) = block {
            let block_id = parse_block_id(blk)?;
            aggregator.price_at_block(block_id).await
        } else {
            aggregator.latest_price().await
        }
    } else if let Some(blk) = block {
        // Historical query
        let block_id = parse_block_id(blk)?;
        chainlink::fetch_price_at_block(provider, token, chain, block_id).await
    } else {
        // Latest price
        chainlink::fetch_price(provider, token, chain).await
    };

    match price_data {
        Ok(data) => {
            let mut output: PriceOutput = data.into();
            output.token = token.to_string();
            output.quote = quote.to_string();
            output.chain = chain.to_string();
            output.block = block.map(|s| s.to_string());

            print_price_output(&output, args.format)?;
        }
        Err(ChainlinkError::NoFeed) => {
            if !quiet {
                eprintln!("No Chainlink feed found for {} on {}", token, chain);
            }
            println!("No feed available");
        }
        Err(ChainlinkError::ArchiveNodeRequired) => {
            anyhow::bail!("Archive node required for historical queries. Your RPC endpoint doesn't support state queries for past blocks.");
        }
        Err(e) => {
            anyhow::bail!("Chainlink query failed: {}", e);
        }
    }

    Ok(())
}

/// Handle feed command
async fn handle_feed(
    token: &str,
    quote: &str,
    chain: &str,
    rpc_url: Option<&str>,
    args: &ChainlinkArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    // Feed Registry is only available on Ethereum mainnet
    if chain != "ethereum" && chain != "eth" && chain != "mainnet" && chain != "1" {
        anyhow::bail!(
            "Feed Registry is only available on Ethereum mainnet. \
            For other chains, use `ethcli chainlink price {} --chain {}` with a direct oracle lookup.",
            token, chain
        );
    }
    let provider = get_provider(chain, rpc_url)?;

    if !quiet {
        eprintln!("Looking up feed address for {}/{}...", token, quote);
    }

    // Resolve token address
    let base = if token.starts_with("0x") {
        Address::from_str(token).map_err(|_| anyhow::anyhow!("Invalid address: {}", token))?
    } else {
        chainlink::constants::symbol_to_address(token)
            .ok_or_else(|| anyhow::anyhow!("Unknown token symbol: {}", token))?
    };

    // Resolve quote
    let quote_addr = match quote.to_lowercase().as_str() {
        "usd" => denominations::USD,
        "eth" => denominations::ETH,
        "btc" => denominations::BTC,
        _ => anyhow::bail!("Unknown quote: {}. Use usd, eth, or btc", quote),
    };

    let registry = FeedRegistry::new(provider);

    match registry.get_feed(base, quote_addr).await {
        Ok(feed) => {
            let output = serde_json::json!({
                "token": token,
                "quote": quote,
                "feed_address": format!("{:#x}", feed),
            });

            match args.format {
                OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
                OutputFormat::Ndjson => println!("{}", serde_json::to_string(&output)?),
                OutputFormat::Table => {
                    println!("Feed Address: {:#x}", feed);
                }
            }
        }
        Err(ChainlinkError::NoFeed) => {
            println!("No feed found for {}/{}", token, quote);
        }
        Err(e) => {
            anyhow::bail!("Feed lookup failed: {}", e);
        }
    }

    Ok(())
}

/// Handle oracles command
async fn handle_oracles(
    chain: Option<&str>,
    args: &ChainlinkArgs,
    _quiet: bool,
) -> anyhow::Result<()> {
    let chains: Vec<&str> = if let Some(c) = chain {
        vec![c]
    } else {
        vec!["ethereum", "arbitrum", "polygon", "optimism", "base"]
    };

    let mut oracles: Vec<OracleInfo> = Vec::new();

    for chain_name in chains {
        let tokens = chainlink::constants::supported_tokens(chain_name);
        for token in tokens {
            if let Some(addr) = chainlink::constants::get_oracle_for_token(token, chain_name) {
                oracles.push(OracleInfo {
                    symbol: token.to_string(),
                    pair: format!("{}/USD", token),
                    address: format!("{:#x}", addr),
                    chain: chain_name.to_string(),
                });
            }
        }
    }

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&oracles)?),
        OutputFormat::Ndjson => {
            for oracle in &oracles {
                println!("{}", serde_json::to_string(oracle)?);
            }
        }
        OutputFormat::Table => {
            println!("{:<10} {:<12} {:<44} {}", "Symbol", "Pair", "Address", "Chain");
            println!("{}", "-".repeat(80));
            for oracle in &oracles {
                println!(
                    "{:<10} {:<12} {:<44} {}",
                    oracle.symbol, oracle.pair, oracle.address, oracle.chain
                );
            }
        }
    }

    Ok(())
}

/// Handle Data Streams commands
async fn handle_streams(command: &StreamsCommands, quiet: bool) -> anyhow::Result<()> {
    use chainlink_data_streams_report::feed_id::ID;
    use chainlink_data_streams_sdk::{client::Client, config::Config};

    // Get credentials
    let (api_key, user_secret, rest_url, ws_url) = get_streams_credentials()?;

    let config = Config::new(api_key, user_secret, rest_url, ws_url)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build config: {:?}", e))?;

    let client =
        Client::new(config).map_err(|e| anyhow::anyhow!("Failed to create client: {:?}", e))?;

    match command {
        StreamsCommands::Feeds { args } => {
            if !quiet {
                eprintln!("Fetching available feeds...");
            }

            let feeds = client
                .get_feeds()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get feeds: {:?}", e))?;

            print_streams_output(&feeds, args.format)?;
        }

        StreamsCommands::Latest { feed_id, args } => {
            let id = parse_feed_id(feed_id)?;

            if !quiet {
                eprintln!("Fetching latest report for {}...", feed_id);
            }

            let response = client
                .get_latest_report(id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get latest report: {:?}", e))?;

            let output = StreamsReportOutput {
                feed_id: response.report.feed_id.to_hex_string(),
                valid_from_timestamp: response.report.valid_from_timestamp,
                observations_timestamp: response.report.observations_timestamp,
                full_report: response.report.full_report,
            };

            print_streams_output(&output, args.format)?;
        }

        StreamsCommands::Report {
            feed_id,
            timestamp,
            args,
        } => {
            let id = parse_feed_id(feed_id)?;

            if !quiet {
                eprintln!("Fetching report for {} at timestamp {}...", feed_id, timestamp);
            }

            let response = client
                .get_report(id, u128::from(*timestamp))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get report: {:?}", e))?;

            let output = StreamsReportOutput {
                feed_id: response.report.feed_id.to_hex_string(),
                valid_from_timestamp: response.report.valid_from_timestamp,
                observations_timestamp: response.report.observations_timestamp,
                full_report: response.report.full_report,
            };

            print_streams_output(&output, args.format)?;
        }

        StreamsCommands::Bulk {
            feed_ids,
            timestamp,
            args,
        } => {
            let ids: Vec<ID> = feed_ids
                .iter()
                .map(|s| parse_feed_id(s))
                .collect::<Result<Vec<_>, _>>()?;

            if !quiet {
                eprintln!("Fetching {} reports at timestamp {}...", feed_ids.len(), timestamp);
            }

            let reports = client
                .get_reports_bulk(&ids, u128::from(*timestamp))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get bulk reports: {:?}", e))?;

            let outputs: Vec<StreamsReportOutput> = reports
                .iter()
                .map(|r| StreamsReportOutput {
                    feed_id: r.feed_id.to_hex_string(),
                    valid_from_timestamp: r.valid_from_timestamp,
                    observations_timestamp: r.observations_timestamp,
                    full_report: r.full_report.clone(),
                })
                .collect();

            print_streams_output(&outputs, args.format)?;
        }

        StreamsCommands::History {
            feed_id,
            timestamp,
            limit,
            args,
        } => {
            let id = parse_feed_id(feed_id)?;

            if !quiet {
                eprintln!("Fetching {} reports for {} starting at {}...", limit, feed_id, timestamp);
            }

            let reports = client
                .get_reports_page_with_limit(id, u128::from(*timestamp), *limit)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get reports: {:?}", e))?;

            let outputs: Vec<StreamsReportOutput> = reports
                .iter()
                .map(|r| StreamsReportOutput {
                    feed_id: r.feed_id.to_hex_string(),
                    valid_from_timestamp: r.valid_from_timestamp,
                    observations_timestamp: r.observations_timestamp,
                    full_report: r.full_report.clone(),
                })
                .collect();

            print_streams_output(&outputs, args.format)?;
        }
    }

    Ok(())
}

/// Streams report output
#[derive(Debug, Serialize)]
struct StreamsReportOutput {
    feed_id: String,
    valid_from_timestamp: usize,
    observations_timestamp: usize,
    full_report: String,
}

/// Get RPC provider for a chain
fn get_provider(
    chain: &str,
    rpc_url: Option<&str>,
) -> anyhow::Result<impl Provider + Clone> {
    let chain_enum = Chain::from_str(chain).unwrap_or(Chain::Ethereum);

    let endpoint = if let Some(url) = rpc_url {
        Endpoint::new(EndpointConfig::new(url.to_string()), 30, None)?
    } else {
        let config = ConfigFile::load_default()
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?
            .unwrap_or_default();

        let chain_endpoints: Vec<_> = config
            .endpoints
            .into_iter()
            .filter(|e| e.enabled && e.chain == chain_enum)
            .collect();

        if chain_endpoints.is_empty() {
            anyhow::bail!(
                "No RPC endpoints configured for {}. Add one with: ethcli endpoints add <url>",
                chain_enum.display_name()
            );
        }
        Endpoint::new(chain_endpoints[0].clone(), 30, None)?
    };

    Ok(endpoint.provider().clone())
}

/// Get Data Streams credentials
fn get_streams_credentials() -> anyhow::Result<(String, String, String, String)> {
    use secrecy::ExposeSecret;

    let file_config = ConfigFile::load_default().ok().flatten();
    let chainlink_config = file_config.as_ref().and_then(|c| c.chainlink.as_ref());

    let api_key = match chainlink_config.map(|c| c.api_key.expose_secret().to_string()) {
        Some(key) => key,
        None => std::env::var("CHAINLINK_API_KEY")
            .or_else(|_| std::env::var("CHAINLINK_CLIENT_ID"))
            .map_err(|_| anyhow::anyhow!("CHAINLINK_API_KEY not set. Data Streams requires API credentials."))?,
    };

    let user_secret = match chainlink_config.map(|c| c.user_secret.expose_secret().to_string()) {
        Some(secret) => secret,
        None => std::env::var("CHAINLINK_USER_SECRET")
            .or_else(|_| std::env::var("CHAINLINK_CLIENT_SECRET"))
            .map_err(|_| anyhow::anyhow!("CHAINLINK_USER_SECRET not set"))?,
    };

    let rest_url = chainlink_config
        .and_then(|c| c.rest_url.clone())
        .or_else(|| std::env::var("CHAINLINK_REST_URL").ok())
        .unwrap_or_else(|| "https://api.testnet-dataengine.chain.link".to_string());

    let ws_url = chainlink_config
        .and_then(|c| c.ws_url.clone())
        .or_else(|| std::env::var("CHAINLINK_WS_URL").ok())
        .unwrap_or_else(|| "wss://ws.testnet-dataengine.chain.link".to_string());

    Ok((api_key, user_secret, rest_url, ws_url))
}

fn parse_feed_id(s: &str) -> anyhow::Result<chainlink_data_streams_report::feed_id::ID> {
    chainlink_data_streams_report::feed_id::ID::from_hex_str(s)
        .map_err(|e| anyhow::anyhow!("Invalid feed ID '{}': {:?}", s, e))
}

fn print_price_output(output: &PriceOutput, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(output)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(output)?);
        }
        OutputFormat::Table => {
            println!();
            println!("Chainlink Price for {}/{}", output.token, output.quote);
            println!("{}", "=".repeat(50));
            if let Some(price) = output.price_usd {
                println!("  Price:       ${:.8}", price);
            } else {
                println!("  Price:       INVALID (stale or negative)");
            }
            println!("  Raw Answer:  {}", output.raw_answer);
            println!("  Decimals:    {}", output.decimals);
            println!("  Round ID:    {}", output.round_id);
            println!("  Updated:     {} (unix timestamp)", output.updated_at);
            if let Some(ref feed) = output.feed_address {
                println!("  Feed:        {}", feed);
            }
            if output.is_stale {
                println!("  Warning: Price data is stale!");
            }
            if let Some(ref blk) = output.block {
                println!("  Block:       {}", blk);
            }
            println!("  Source:      {}", output.source);
            println!();
        }
    }
    Ok(())
}

fn print_streams_output<T: serde::Serialize>(data: &T, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(data)?);
        }
        OutputFormat::Table => {
            // For table format, just use pretty JSON for now
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}
