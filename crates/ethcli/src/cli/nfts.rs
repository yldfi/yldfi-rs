//! Aggregated NFT command
//!
//! Fetches wallet NFT holdings from multiple sources in parallel and merges results.

use crate::aggregator::nft::{fetch_nfts_all, fetch_nfts_parallel, NftEntry, NftResult, NftSource};
use crate::cli::OutputFormat;
use clap::{Args, ValueEnum};
use serde::Serialize;

/// NFT source selection for CLI
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum NftSourceArg {
    /// Query all sources in parallel
    #[default]
    All,
    /// Alchemy NFT API
    Alchemy,
    /// Moralis NFT API
    Moralis,
    /// Dune SIM Collectibles API
    Dsim,
}

impl From<NftSourceArg> for NftSource {
    fn from(arg: NftSourceArg) -> Self {
        match arg {
            NftSourceArg::All => NftSource::All,
            NftSourceArg::Alchemy => NftSource::Alchemy,
            NftSourceArg::Moralis => NftSource::Moralis,
            NftSourceArg::Dsim => NftSource::DuneSim,
        }
    }
}

#[derive(Args)]
pub struct NftsArgs {
    /// Wallet address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,

    /// Chain(s) to query (can be specified multiple times)
    #[arg(long, short, default_value = "ethereum", value_name = "CHAIN")]
    pub chain: Vec<String>,

    /// Source(s) to query (default: all in parallel)
    #[arg(long, short, default_value = "all")]
    pub source: NftSourceArg,

    /// Show per-source breakdown
    #[arg(long)]
    pub show_sources: bool,

    /// Exclude spam NFTs
    #[arg(long)]
    pub exclude_spam: bool,

    /// Show only verified collections
    #[arg(long)]
    pub verified_only: bool,

    /// Limit number of NFTs shown
    #[arg(long, value_name = "N")]
    pub limit: Option<usize>,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

/// NFT command output
#[derive(Debug, Serialize)]
pub struct NftsOutput {
    pub address: String,
    pub chains: Vec<String>,
    pub aggregation: NftResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<SourceNfts>>,
    pub total_latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct SourceNfts {
    pub source: String,
    pub nft_count: Option<usize>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

/// Execute the NFTs command
pub async fn execute(args: &NftsArgs, quiet: bool) -> anyhow::Result<()> {
    let address = &args.address;
    let chains: Vec<&str> = args.chain.iter().map(|s| s.as_str()).collect();

    if !quiet {
        eprintln!("Fetching NFTs for {} on {:?}...", address, args.chain);
    }

    let result = match args.source {
        NftSourceArg::All => fetch_nfts_all(address, &chains).await,
        source => {
            let nft_source: NftSource = source.into();
            fetch_nfts_parallel(address, &chains, &[nft_source]).await
        }
    };

    // Apply filters
    let mut aggregation = result.aggregated;

    // Filter spam
    if args.exclude_spam {
        aggregation.nfts.retain(|n| !n.is_spam.unwrap_or(false));
        aggregation.nft_count = aggregation.nfts.len();
    }

    // Filter verified only
    if args.verified_only {
        aggregation.nfts.retain(|n| n.is_verified.unwrap_or(false));
        aggregation.nft_count = aggregation.nfts.len();
    }

    // Apply limit
    if let Some(limit) = args.limit {
        aggregation.nfts.truncate(limit);
        aggregation.nft_count = aggregation.nfts.len();
    }

    // Recalculate estimated value after filtering
    let estimated_value: f64 = aggregation
        .nfts
        .iter()
        .filter_map(|n| n.floor_price_usd)
        .sum();
    aggregation.estimated_value_usd = if estimated_value > 0.0 {
        Some(estimated_value)
    } else {
        None
    };

    // Build output
    let sources = if args.show_sources {
        Some(
            result
                .sources
                .iter()
                .map(|s| SourceNfts {
                    source: s.source.clone(),
                    nft_count: s.data.as_ref().map(|d| d.len()),
                    error: s.error.clone(),
                    latency_ms: s.latency_ms,
                })
                .collect(),
        )
    } else {
        None
    };

    let output = NftsOutput {
        address: address.clone(),
        chains: args.chain.clone(),
        aggregation,
        sources,
        total_latency_ms: result.total_latency_ms,
    };

    // Format and print output
    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(&output)?);
        }
        OutputFormat::Table => {
            print_table_output(&output, &result.sources);
        }
    }

    Ok(())
}

fn print_table_output(
    output: &NftsOutput,
    sources: &[crate::aggregator::SourceResult<Vec<NftEntry>>],
) {
    println!();
    println!("NFTs for {}", output.address);
    println!("{}", "=".repeat(90));
    println!("  Total NFTs:      {}", output.aggregation.nft_count);
    if let Some(value) = output.aggregation.estimated_value_usd {
        println!("  Est. Value:      ${:.2}", value);
    }
    println!("  Chains:          {:?}", output.aggregation.chains_covered);
    println!();

    if !output.aggregation.nfts.is_empty() {
        println!("NFT Holdings:");
        println!("{}", "-".repeat(110));
        println!(
            "{:<8} {:<20} {:<16} {:>10} {:>12} {:>8} {:>10}",
            "Chain", "Collection", "Token ID", "Balance", "Floor (USD)", "Spam", "Sources"
        );
        println!("{}", "-".repeat(110));

        for nft in &output.aggregation.nfts {
            let collection = nft
                .collection_name
                .as_ref()
                .or(nft.name.as_ref())
                .map(|s| truncate(s, 20))
                .unwrap_or_else(|| "-".to_string());

            let token_id = truncate(&nft.token_id, 16);

            let floor_str = nft
                .floor_price_usd
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "-".to_string());

            let spam_str = match nft.is_spam {
                Some(true) => "Yes",
                Some(false) => "No",
                None => "-",
            };

            let sources_count = nft.found_in.len();

            println!(
                "{:<8} {:<20} {:<16} {:>10} {:>12} {:>8} {:>10}",
                truncate(&nft.chain, 8),
                collection,
                token_id,
                nft.balance,
                floor_str,
                spam_str,
                sources_count
            );
        }
        println!("{}", "-".repeat(110));
    } else {
        println!("No NFTs found.");
    }

    if output.sources.is_some() {
        println!();
        println!("Per-Source Results:");
        println!("{}", "-".repeat(50));
        for source in sources {
            let status = if source.is_success() { "OK" } else { "ERR" };
            let count_str = source
                .data
                .as_ref()
                .map(|d| format!("{} NFTs", d.len()))
                .unwrap_or_else(|| "-".to_string());
            let error_note = source
                .error
                .as_ref()
                .map(|e| format!(" ({})", truncate_error(e, 30)))
                .unwrap_or_default();

            println!(
                "{:<10} {:>12} {:>8}ms {:>6}{}",
                source.source, count_str, source.latency_ms, status, error_note
            );
        }
        println!("{}", "-".repeat(50));
    }

    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn truncate_error(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
