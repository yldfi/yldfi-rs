//! Transaction analysis command

use super::OutputFormat;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct TxArgs {
    /// Transaction hash(es) (with or without 0x prefix)
    pub hashes: Vec<String>,

    /// Read transaction hashes from file (one per line)
    #[arg(long, short = 'f')]
    pub file: Option<PathBuf>,

    /// Read transaction hashes from stdin (one per line)
    #[arg(long)]
    pub stdin: bool,

    /// Output format (json, table/pretty, ndjson)
    #[arg(long, short, value_enum, default_value = "table")]
    pub output: OutputFormat,

    /// Process transactions in parallel batches
    #[arg(long, default_value = "10")]
    pub batch_size: usize,

    /// Enrich with Etherscan data (contract names, token symbols, function decoding)
    #[arg(long)]
    pub enrich: bool,
}
