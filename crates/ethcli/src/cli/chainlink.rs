//! Chainlink Data Streams CLI commands
//!
//! Provides access to Chainlink's low-latency, cryptographically verifiable market data.
//!
//! **Note:** Requires API credentials from <https://chain.link/data-streams>.
//! Users must accept Chainlink's Terms of Service: <https://chainlinklabs.com/terms>

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use chainlink_data_streams_report::feed_id::ID;
use chainlink_data_streams_sdk::{client::Client, config::Config};
use clap::{Args, Subcommand};
use serde::Serialize;

#[derive(Args)]
pub struct ChainlinkArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
#[command(after_help = r#"CREDENTIALS:
    Requires API credentials from https://chain.link/data-streams
    Set CHAINLINK_API_KEY and CHAINLINK_USER_SECRET environment variables.

TERMS OF SERVICE:
    By using this feature, you agree to Chainlink's Terms of Service.
    See: https://chainlinklabs.com/terms
"#)]
pub enum ChainlinkCommands {
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

/// Simplified report output for serialization
#[derive(Debug, Serialize)]
struct ReportOutput {
    feed_id: String,
    valid_from_timestamp: usize,
    observations_timestamp: usize,
    full_report: String,
}

/// Handle Chainlink commands
pub async fn handle(command: &ChainlinkCommands, quiet: bool) -> anyhow::Result<()> {
    // Try config first, then fall back to env vars
    let (api_key, user_secret, rest_url, ws_url) =
        if let Ok(Some(config)) = ConfigFile::load_default() {
            if let Some(ref chainlink_config) = config.chainlink {
                let rest = chainlink_config
                    .rest_url
                    .clone()
                    .unwrap_or_else(|| "https://api.testnet-dataengine.chain.link".to_string());
                let ws = chainlink_config
                    .ws_url
                    .clone()
                    .unwrap_or_else(|| "wss://ws.testnet-dataengine.chain.link".to_string());
                (
                    chainlink_config.api_key.clone(),
                    chainlink_config.user_secret.clone(),
                    rest,
                    ws,
                )
            } else {
                get_chainlink_from_env()?
            }
        } else {
            get_chainlink_from_env()?
        };

    let config = Config::new(api_key, user_secret, rest_url, ws_url)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to build config: {:?}", e))?;

    let client =
        Client::new(config).map_err(|e| anyhow::anyhow!("Failed to create client: {:?}", e))?;

    match command {
        ChainlinkCommands::Feeds { args } => {
            if !quiet {
                eprintln!("Fetching available feeds...");
            }

            let feeds = client
                .get_feeds()
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get feeds: {:?}", e))?;

            // Feed implements Serialize directly
            print_output(&feeds, args.format)?;
        }

        ChainlinkCommands::Latest { feed_id, args } => {
            let id = parse_feed_id(feed_id)?;

            if !quiet {
                eprintln!("Fetching latest report for {}...", feed_id);
            }

            let response = client
                .get_latest_report(id)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get latest report: {:?}", e))?;

            // Convert to serializable output
            let output = ReportOutput {
                feed_id: response.report.feed_id.to_hex_string(),
                valid_from_timestamp: response.report.valid_from_timestamp,
                observations_timestamp: response.report.observations_timestamp,
                full_report: response.report.full_report,
            };

            print_output(&output, args.format)?;
        }

        ChainlinkCommands::Report {
            feed_id,
            timestamp,
            args,
        } => {
            let id = parse_feed_id(feed_id)?;

            if !quiet {
                eprintln!(
                    "Fetching report for {} at timestamp {}...",
                    feed_id, timestamp
                );
            }

            let response = client
                .get_report(id, u128::from(*timestamp))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get report: {:?}", e))?;

            let output = ReportOutput {
                feed_id: response.report.feed_id.to_hex_string(),
                valid_from_timestamp: response.report.valid_from_timestamp,
                observations_timestamp: response.report.observations_timestamp,
                full_report: response.report.full_report,
            };

            print_output(&output, args.format)?;
        }

        ChainlinkCommands::Bulk {
            feed_ids,
            timestamp,
            args,
        } => {
            let ids: Vec<ID> = feed_ids
                .iter()
                .map(|s| parse_feed_id(s))
                .collect::<Result<Vec<_>, _>>()?;

            if !quiet {
                eprintln!(
                    "Fetching {} reports at timestamp {}...",
                    feed_ids.len(),
                    timestamp
                );
            }

            let reports = client
                .get_reports_bulk(&ids, u128::from(*timestamp))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get bulk reports: {:?}", e))?;

            // Convert to serializable outputs
            let outputs: Vec<ReportOutput> = reports
                .iter()
                .map(|r| ReportOutput {
                    feed_id: r.feed_id.to_hex_string(),
                    valid_from_timestamp: r.valid_from_timestamp,
                    observations_timestamp: r.observations_timestamp,
                    full_report: r.full_report.clone(),
                })
                .collect();

            print_output(&outputs, args.format)?;
        }

        ChainlinkCommands::History {
            feed_id,
            timestamp,
            limit,
            args,
        } => {
            let id = parse_feed_id(feed_id)?;

            if !quiet {
                eprintln!(
                    "Fetching {} reports for {} starting at {}...",
                    limit, feed_id, timestamp
                );
            }

            let reports = client
                .get_reports_page_with_limit(id, u128::from(*timestamp), *limit)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get reports: {:?}", e))?;

            // Convert to serializable outputs
            let outputs: Vec<ReportOutput> = reports
                .iter()
                .map(|r| ReportOutput {
                    feed_id: r.feed_id.to_hex_string(),
                    valid_from_timestamp: r.valid_from_timestamp,
                    observations_timestamp: r.observations_timestamp,
                    full_report: r.full_report.clone(),
                })
                .collect();

            print_output(&outputs, args.format)?;
        }
    }

    Ok(())
}

fn parse_feed_id(s: &str) -> anyhow::Result<ID> {
    ID::from_hex_str(s).map_err(|e| anyhow::anyhow!("Invalid feed ID '{}': {:?}", s, e))
}

fn get_chainlink_from_env() -> anyhow::Result<(String, String, String, String)> {
    let api_key = std::env::var("CHAINLINK_API_KEY")
        .or_else(|_| std::env::var("CHAINLINK_CLIENT_ID"))
        .map_err(|_| anyhow::anyhow!("CHAINLINK_API_KEY not set in config or environment"))?;

    let user_secret = std::env::var("CHAINLINK_USER_SECRET")
        .or_else(|_| std::env::var("CHAINLINK_CLIENT_SECRET"))
        .map_err(|_| anyhow::anyhow!("CHAINLINK_USER_SECRET not set in config or environment"))?;

    let rest_url = std::env::var("CHAINLINK_REST_URL")
        .unwrap_or_else(|_| "https://api.testnet-dataengine.chain.link".to_string());

    let ws_url = std::env::var("CHAINLINK_WS_URL")
        .unwrap_or_else(|_| "wss://ws.testnet-dataengine.chain.link".to_string());

    Ok((api_key, user_secret, rest_url, ws_url))
}

fn print_output<T: serde::Serialize>(data: &T, format: OutputFormat) -> anyhow::Result<()> {
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
