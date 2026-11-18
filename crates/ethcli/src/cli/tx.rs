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
    #[arg(long, default_value = "10", value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..))]
    pub batch_size: usize,

    /// Enrich with Etherscan data (contract names, token symbols, function decoding)
    #[arg(long)]
    pub enrich: bool,

    /// Also fetch a call trace, the block's time and, for a failed
    /// transaction, the reason it reverted (up to three more requests)
    ///
    /// The fee, the position in the block and the net balance change per
    /// address are always included, but native-token changes are marked
    /// partial without a trace, because transfers made inside the call are
    /// not seen. With the trace they are exact. The revert reason comes from
    /// replaying the call against the parent block's state.
    #[arg(long)]
    pub full: bool,

    /// Use only this RPC endpoint, not the configured pool (e.g. a local fork)
    #[arg(long, visible_alias = "rpc", value_name = "URL")]
    pub rpc_url: Option<String>,
}
