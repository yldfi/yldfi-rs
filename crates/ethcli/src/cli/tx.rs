//! Transaction analysis command

use super::OutputFormat;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct TxArgs {
    /// Transaction hash(es) (with or without 0x prefix).
    /// Partial hashes are supported when --block is specified.
    #[arg(value_name = "TX_HASH")]
    pub hashes: Vec<String>,

    /// Read transaction hashes from file (one per line)
    #[arg(long, short = 'f', value_name = "FILE")]
    pub file: Option<PathBuf>,

    /// Read transaction hashes from stdin (one per line)
    #[arg(long)]
    pub stdin: bool,

    /// Block number for partial hash resolution.
    /// When provided with a partial tx hash (e.g. from `account txs` output),
    /// the transaction will be looked up by matching the prefix within this block.
    #[arg(long, short = 'b', value_name = "BLOCK")]
    pub block: Option<u64>,

    /// Output format (json, table/pretty, ndjson)
    #[arg(
        long,
        short,
        visible_alias = "format",
        value_enum,
        default_value = "table"
    )]
    pub output: OutputFormat,

    /// Process transactions in parallel batches
    #[arg(long, default_value = "10")]
    pub batch_size: usize,

    /// Enrich with Etherscan data (contract names, token symbols, function decoding)
    #[arg(long)]
    pub enrich: bool,
}
