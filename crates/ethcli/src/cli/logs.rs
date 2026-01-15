//! Log fetching command
//!
//! Fetch historical logs from EVM contracts

use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct LogsArgs {
    /// Contract address to fetch logs from
    #[arg(short, long)]
    pub contract: String,

    /// Event signature or name (can be repeated for multiple events)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub event: Vec<String>,

    /// Path to ABI JSON file
    #[arg(long)]
    pub abi: Option<PathBuf>,

    /// Start block number (omit or use "auto" to start from contract creation)
    #[arg(short = 'f', long, conflicts_with = "since")]
    pub from_block: Option<String>,

    /// Start from relative time ago (e.g., "30d", "6h", "2w", "90m")
    /// Supported units: m/min/minutes, h/hours, d/days, w/weeks
    #[arg(long, conflicts_with = "from_block")]
    pub since: Option<String>,

    /// End block number (or "latest")
    #[arg(short = 't', long, default_value = "latest")]
    pub to_block: String,

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Output format (json, ndjson, csv, sqlite)
    #[arg(long, default_value = "json")]
    pub format: String,

    /// Fetch raw logs without decoding
    #[arg(long)]
    pub raw: bool,

    /// Include block timestamps in output (requires extra RPC calls)
    #[arg(long)]
    pub timestamps: bool,

    /// Number of parallel requests
    #[arg(short = 'n', long)]
    pub concurrency: Option<usize>,

    /// Resume from checkpoint if available
    #[arg(long)]
    pub resume: bool,

    /// Checkpoint file path
    #[arg(long)]
    pub checkpoint: Option<PathBuf>,

    /// Fail if any chunk fails (default: warn and continue)
    #[arg(long)]
    pub strict: bool,

    /// Override block range per request (chunk size)
    #[arg(long)]
    pub chunk_size: Option<u64>,

    #[command(flatten)]
    pub rpc: RpcArgs,

    #[command(flatten)]
    pub proxy: ProxyArgs,
}

#[derive(Args)]
pub struct RpcArgs {
    /// Use only this RPC endpoint (can be repeated)
    #[arg(long = "rpc", action = clap::ArgAction::Append)]
    pub rpc_urls: Vec<String>,

    /// Add RPC to default pool (can be repeated)
    #[arg(long = "add-rpc", action = clap::ArgAction::Append)]
    pub add_rpc: Vec<String>,

    /// Exclude RPC from pool (can be repeated)
    #[arg(long = "exclude-rpc", action = clap::ArgAction::Append)]
    pub exclude_rpc: Vec<String>,

    /// Load RPC URLs from file
    #[arg(long)]
    pub rpc_file: Option<PathBuf>,

    /// Only use endpoints with priority >= N
    #[arg(long, default_value = "1")]
    pub min_priority: u8,

    /// Request timeout in seconds
    #[arg(long)]
    pub timeout: Option<u64>,

    /// Max retries per request
    #[arg(long)]
    pub retries: Option<u32>,
}

#[derive(Args)]
pub struct ProxyArgs {
    /// Use proxy for all requests (http/https/socks5)
    #[arg(long)]
    pub proxy: Option<String>,

    /// Load proxies from file, rotate between them
    #[arg(long)]
    pub proxy_file: Option<PathBuf>,

    /// Rotate proxy per request
    #[arg(long)]
    pub proxy_rotate: bool,
}
