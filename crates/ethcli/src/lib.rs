//! ethcli - Comprehensive Ethereum CLI
//!
//! A Rust library and CLI for Ethereum data: fetching logs, analyzing transactions,
//! querying accounts, and exploring contracts.
//!
//! # Example - Fetching Logs
//!
//! ```rust,no_run
//! use ethcli::{Config, LogFetcher, Chain, OutputFormat};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = Config::builder()
//!         .chain(Chain::Ethereum)
//!         .contract("0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
//!         .event("Transfer(address indexed from, address indexed to, uint256 value)")
//!         .from_block(18_000_000)
//!         .to_block_number(18_001_000)
//!         .concurrency(10)
//!         .build()?;
//!
//!     let fetcher = LogFetcher::new(config).await?;
//!     let result = fetcher.fetch_all().await?;
//!
//!     println!("Fetched {} logs", result.len());
//!     Ok(())
//! }
//! ```

pub mod abi;
pub mod checkpoint;
pub mod cli;
pub mod config;
pub mod error;
pub mod etherscan;
pub mod fetcher;
pub mod output;
pub mod proxy;
pub mod rpc;
pub mod tx;
pub mod utils;

// Legacy alias for cache module (now in etherscan::cache)
pub mod cache {
    pub use crate::etherscan::cache::*;
}

// Re-exports for convenience
pub use abi::{AbiFetcher, DecodedLog, EventSignature, LogDecoder};
pub use checkpoint::{Checkpoint, CheckpointManager};
pub use config::{
    BlockNumber, BlockRange, Chain, ChainId, Config, ConfigBuilder, ConfigFile, EndpointConfig,
    NodeType, OutputConfig, OutputFormat, ProxyConfig, RpcConfig,
};
pub use error::{AbiError, CheckpointError, ConfigError, Error, OutputError, Result, RpcError};
pub use etherscan::{CacheStats, Client as EtherscanClient, SignatureCache};
pub use fetcher::{
    FetchLogs, FetchProgress, FetchResult, FetchStats, LogFetcher, StreamingFetcher,
};
pub use output::{create_writer, CsvWriter, JsonWriter, OutputWriter, SqliteWriter};
pub use proxy::{validate_proxy_url, ProxyRotator, RotationMode};
pub use rpc::{
    optimize_endpoint, test_connectivity, Endpoint, EndpointHealth, HealthTracker,
    OptimizationResult, RpcPool,
};
pub use tx::{format_analysis, TransactionAnalysis, TxAnalyzer};
pub use utils::TokenMetadata;
