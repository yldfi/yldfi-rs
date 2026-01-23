//! Log fetching command
//!
//! Fetch historical logs from EVM contracts

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
#[command(after_help = r#"Examples:
  # Fetch Transfer events from USDC
  ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 -e "Transfer(address,address,uint256)" -f 21500000 -t 21500010

  # Fetch all events from contract since deployment
  ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 -f auto

  # Fetch events from last 7 days, output to CSV
  ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 -e "Transfer" --since 7d -o csv -O transfers.csv

  # Multiple events with resume support
  ethcli logs -c 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D -e "Swap" -e "Sync" -f 19000000 --resume

  # Raw logs without decoding
  ethcli logs -c 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 -f 21500000 -t 21500100 --raw"#)]
pub struct LogsArgs {
    /// Contract address to fetch logs from
    #[arg(short, long, value_name = "ADDRESS")]
    pub contract: String,

    /// Event signature or name (can be repeated for multiple events)
    #[arg(short, long, action = clap::ArgAction::Append, value_name = "EVENT")]
    pub event: Vec<String>,

    /// Path to ABI JSON file
    #[arg(long, value_name = "FILE")]
    pub abi: Option<PathBuf>,

    /// Start block number (omit or use "auto" to start from contract creation)
    #[arg(short = 'f', long, conflicts_with = "since", value_name = "BLOCK")]
    pub from_block: Option<String>,

    /// Start from relative time ago (e.g., "30d", "6h", "2w", "90m")
    /// Supported units: m/min/minutes, h/hours, d/days, w/weeks
    #[arg(long, conflicts_with = "from_block", value_name = "DURATION")]
    pub since: Option<String>,

    /// End block number (or "latest")
    #[arg(short = 't', long, default_value = "latest", value_name = "BLOCK")]
    pub to_block: String,

    /// Output file path (stdout if not specified)
    #[arg(short = 'O', long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Output format (json, ndjson, csv, sqlite)
    #[arg(short = 'o', long, default_value = "json", value_name = "FORMAT")]
    pub format: String,

    /// Fetch raw logs without decoding
    #[arg(long)]
    pub raw: bool,

    /// Include block timestamps in output (requires extra RPC calls)
    #[arg(long)]
    pub timestamps: bool,

    /// Number of parallel requests
    #[arg(short = 'n', long, value_name = "N")]
    pub concurrency: Option<usize>,

    /// Resume from checkpoint if available
    #[arg(long)]
    pub resume: bool,

    /// Checkpoint file path
    #[arg(long, value_name = "FILE")]
    pub checkpoint: Option<PathBuf>,

    /// Fail if any chunk fails (default: warn and continue)
    #[arg(long)]
    pub strict: bool,

    /// Override block range per request (chunk size)
    #[arg(long, value_name = "SIZE")]
    pub chunk_size: Option<u64>,

    #[command(flatten)]
    pub rpc: RpcArgs,

    #[command(flatten)]
    pub proxy: ProxyArgs,
}

#[derive(Args)]
pub struct RpcArgs {
    /// Use only this RPC endpoint (can be repeated)
    #[arg(long = "rpc", action = clap::ArgAction::Append, value_name = "URL")]
    pub rpc_urls: Vec<String>,

    /// Add RPC to default pool (can be repeated)
    #[arg(long = "add-rpc", action = clap::ArgAction::Append, value_name = "URL")]
    pub add_rpc: Vec<String>,

    /// Exclude RPC from pool (can be repeated)
    #[arg(long = "exclude-rpc", action = clap::ArgAction::Append, value_name = "URL")]
    pub exclude_rpc: Vec<String>,

    /// Load RPC URLs from file
    #[arg(long, value_name = "FILE")]
    pub rpc_file: Option<PathBuf>,

    /// Only use endpoints with priority >= N
    #[arg(long, default_value = "1", value_name = "N")]
    pub min_priority: u8,

    /// Request timeout in seconds
    #[arg(long, value_name = "SECONDS")]
    pub timeout: Option<u64>,

    /// Max retries per request
    #[arg(long, value_name = "N")]
    pub retries: Option<u32>,
}

#[derive(Args)]
pub struct ProxyArgs {
    /// Use proxy for all requests (http/https/socks5)
    #[arg(long, value_name = "URL")]
    pub proxy: Option<String>,

    /// Load proxies from file, rotate between them
    #[arg(long, value_name = "FILE")]
    pub proxy_file: Option<PathBuf>,

    /// Rotate proxy per request
    #[arg(long)]
    pub proxy_rotate: bool,

    /// Force use proxy from config (overrides enabled=false)
    #[arg(long, conflicts_with = "no_proxy")]
    pub use_proxy: bool,

    /// Disable proxy for this request (overrides enabled=true)
    #[arg(long, conflicts_with = "use_proxy")]
    pub no_proxy: bool,
}
