//! Configuration management for ethcli

mod addressbook;
mod chain;
mod endpoint;
mod file;

pub use addressbook::{AddressBook, AddressEntry};
pub use chain::{Chain, ChainId};
pub use endpoint::{
    EndpointConfig, NodeType, DEFAULT_MAX_BLOCK_RANGE, DEFAULT_MAX_LOGS, DEFAULT_PRIORITY,
    MIN_TX_FETCH_CONCURRENCY,
};
pub use file::{ConfigFile, TenderlyConfig};

use crate::error::{ConfigError, Result};
use std::path::PathBuf;

/// Main configuration for the log fetcher
#[derive(Debug, Clone)]
pub struct Config {
    /// Chain to query
    pub chain: Chain,
    /// Contract address
    pub contract: String,
    /// Event filters (names, signatures, or topic hashes). Empty = all events
    pub events: Vec<String>,
    /// ABI file path (optional)
    pub abi_path: Option<PathBuf>,
    /// Block range to fetch
    pub block_range: BlockRange,
    /// Output configuration
    pub output: OutputConfig,
    /// RPC configuration
    pub rpc: RpcConfig,
    /// Etherscan API key (optional)
    pub etherscan_key: Option<String>,
    /// Resume from checkpoint
    pub resume: bool,
    /// Checkpoint file path
    pub checkpoint_path: PathBuf,
    /// Verbosity level
    pub verbosity: u8,
    /// Quiet mode (no progress)
    pub quiet: bool,
    /// Fetch raw logs without decoding
    pub raw: bool,
    /// Auto-detect from_block from contract creation
    pub auto_from_block: bool,
}

/// Block range specification
#[derive(Debug, Clone)]
pub enum BlockRange {
    /// Specific range
    Range { from: u64, to: BlockNumber },
    /// From block to latest
    FromToLatest { from: u64 },
}

/// Block number (can be specific or "latest")
#[derive(Debug, Clone, Copy)]
pub enum BlockNumber {
    Number(u64),
    Latest,
}

impl BlockRange {
    pub fn from_block(&self) -> u64 {
        match self {
            BlockRange::Range { from, .. } => *from,
            BlockRange::FromToLatest { from } => *from,
        }
    }

    pub fn to_block(&self) -> BlockNumber {
        match self {
            BlockRange::Range { to, .. } => *to,
            BlockRange::FromToLatest { .. } => BlockNumber::Latest,
        }
    }
}

/// Output configuration
#[derive(Debug, Clone)]
pub struct OutputConfig {
    /// Output file path (None for stdout)
    pub path: Option<PathBuf>,
    /// Output format
    pub format: OutputFormat,
}

/// Supported output formats
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    NdJson,
    Csv,
    Sqlite,
}

impl std::str::FromStr for OutputFormat {
    type Err = ConfigError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "ndjson" | "jsonl" => Ok(OutputFormat::NdJson),
            "csv" => Ok(OutputFormat::Csv),
            "sqlite" | "db" => Ok(OutputFormat::Sqlite),
            _ => Err(ConfigError::InvalidFile(format!(
                "Unknown output format: {}",
                s
            ))),
        }
    }
}

/// RPC configuration
#[derive(Debug, Clone)]
pub struct RpcConfig {
    /// Custom RPC endpoints (if empty, use defaults)
    pub endpoints: Vec<EndpointConfig>,
    /// Additional endpoints to add to defaults
    pub add_endpoints: Vec<String>,
    /// Endpoints to exclude from pool
    pub exclude_endpoints: Vec<String>,
    /// Minimum priority for endpoints
    pub min_priority: u8,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Max retries per request
    pub max_retries: u32,
    /// Number of parallel requests
    pub concurrency: usize,
    /// Proxy configuration
    pub proxy: Option<ProxyConfig>,
    /// Override chunk size (max block range per request)
    pub chunk_size: Option<u64>,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            endpoints: Vec::new(),
            add_endpoints: Vec::new(),
            exclude_endpoints: Vec::new(),
            min_priority: 1,
            timeout_secs: 30,
            max_retries: 3,
            concurrency: 5,
            proxy: None,
            chunk_size: None,
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// Default proxy URL
    pub url: Option<String>,
    /// Proxy file for rotation
    pub file: Option<PathBuf>,
    /// Rotate proxy per request
    pub rotate_per_request: bool,
}

/// Builder for Config
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    chain: Option<Chain>,
    contract: Option<String>,
    events: Vec<String>,
    abi_path: Option<PathBuf>,
    from_block: Option<u64>,
    to_block: Option<BlockNumber>,
    output_path: Option<PathBuf>,
    output_format: OutputFormat,
    rpc: RpcConfig,
    etherscan_key: Option<String>,
    resume: bool,
    checkpoint_path: Option<PathBuf>,
    verbosity: u8,
    quiet: bool,
    raw: bool,
    auto_from_block: bool,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn chain(mut self, chain: Chain) -> Self {
        self.chain = Some(chain);
        self
    }

    pub fn contract(mut self, address: impl Into<String>) -> Self {
        self.contract = Some(address.into());
        self
    }

    pub fn event(mut self, signature: impl Into<String>) -> Self {
        self.events.push(signature.into());
        self
    }

    pub fn events(mut self, signatures: Vec<String>) -> Self {
        self.events = signatures;
        self
    }

    pub fn abi_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.abi_path = Some(path.into());
        self
    }

    pub fn from_block(mut self, block: u64) -> Self {
        self.from_block = Some(block);
        self
    }

    pub fn to_block(mut self, block: BlockNumber) -> Self {
        self.to_block = Some(block);
        self
    }

    pub fn to_block_number(mut self, block: u64) -> Self {
        self.to_block = Some(BlockNumber::Number(block));
        self
    }

    pub fn to_latest(mut self) -> Self {
        self.to_block = Some(BlockNumber::Latest);
        self
    }

    pub fn output_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.output_path = Some(path.into());
        self
    }

    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn concurrency(mut self, n: usize) -> Self {
        self.rpc.concurrency = n;
        self
    }

    pub fn timeout(mut self, secs: u64) -> Self {
        self.rpc.timeout_secs = secs;
        self
    }

    pub fn etherscan_key(mut self, key: impl Into<String>) -> Self {
        self.etherscan_key = Some(key.into());
        self
    }

    pub fn resume(mut self, resume: bool) -> Self {
        self.resume = resume;
        self
    }

    pub fn checkpoint_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.checkpoint_path = Some(path.into());
        self
    }

    pub fn verbosity(mut self, level: u8) -> Self {
        self.verbosity = level;
        self
    }

    pub fn quiet(mut self, quiet: bool) -> Self {
        self.quiet = quiet;
        self
    }

    pub fn raw(mut self, raw: bool) -> Self {
        self.raw = raw;
        self
    }

    pub fn auto_from_block(mut self, auto: bool) -> Self {
        self.auto_from_block = auto;
        self
    }

    pub fn rpc_config(mut self, rpc: RpcConfig) -> Self {
        self.rpc = rpc;
        self
    }

    pub fn build(self) -> Result<Config> {
        let contract = self
            .contract
            .ok_or_else(|| ConfigError::MissingField("contract".to_string()))?;

        let from_block = self.from_block.unwrap_or(0);
        let to_block = self.to_block.unwrap_or(BlockNumber::Latest);

        // Validate block range: from_block must be <= to_block when to_block is a specific number
        if let BlockNumber::Number(to) = to_block {
            if from_block > to {
                return Err(ConfigError::InvalidBlockNumber(format!(
                    "from_block ({}) cannot be greater than to_block ({})",
                    from_block, to
                ))
                .into());
            }
        }

        // Validate concurrency: must be at least 1
        if self.rpc.concurrency == 0 {
            return Err(
                ConfigError::InvalidFile("concurrency must be at least 1".to_string()).into(),
            );
        }

        let block_range = BlockRange::Range {
            from: from_block,
            to: to_block,
        };

        Ok(Config {
            chain: self.chain.unwrap_or_default(),
            contract,
            events: self.events,
            abi_path: self.abi_path,
            block_range,
            output: OutputConfig {
                path: self.output_path,
                format: self.output_format,
            },
            rpc: self.rpc,
            etherscan_key: self.etherscan_key,
            resume: self.resume,
            checkpoint_path: self
                .checkpoint_path
                .unwrap_or_else(|| PathBuf::from(".eth-log-fetch.checkpoint")),
            verbosity: self.verbosity,
            quiet: self.quiet,
            raw: self.raw,
            auto_from_block: self.auto_from_block,
        })
    }
}

impl Config {
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}
