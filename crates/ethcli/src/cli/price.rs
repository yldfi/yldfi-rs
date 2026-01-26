//! Aggregated price command
//!
//! Fetches token prices from multiple sources in parallel and aggregates results.

use crate::aggregator::{
    fetch_lp_price, fetch_prices_all, fetch_prices_parallel, is_token_address,
    symbol_to_eth_address, NormalizedPrice, PriceAggregation, PriceSource,
};
use crate::cli::OutputFormat;
use clap::{Args, ValueEnum};
use serde::Serialize;

/// Price source selection for CLI
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum PriceSourceArg {
    /// Query all sources in parallel
    #[default]
    All,
    /// CoinGecko
    Gecko,
    /// DefiLlama
    Llama,
    /// Alchemy
    Alchemy,
    /// Moralis
    Moralis,
    /// Curve
    Curve,
    /// CCXT exchanges (Binance, Bitget, OKX)
    Ccxt,
    /// Chainlink Data Streams
    Chainlink,
    /// Pyth Network Hermes API
    Pyth,
    /// Uniswap subgraph (DEX pools)
    Uniswap,
    /// Yearn Kong API (DeFi token prices)
    Kong,
}

impl From<PriceSourceArg> for PriceSource {
    fn from(arg: PriceSourceArg) -> Self {
        match arg {
            PriceSourceArg::All => PriceSource::All,
            PriceSourceArg::Gecko => PriceSource::Gecko,
            PriceSourceArg::Llama => PriceSource::Llama,
            PriceSourceArg::Alchemy => PriceSource::Alchemy,
            PriceSourceArg::Moralis => PriceSource::Moralis,
            PriceSourceArg::Curve => PriceSource::Curve,
            PriceSourceArg::Ccxt => PriceSource::Ccxt,
            PriceSourceArg::Chainlink => PriceSource::Chainlink,
            PriceSourceArg::Pyth => PriceSource::Pyth,
            PriceSourceArg::Uniswap => PriceSource::Uniswap,
            PriceSourceArg::Kong => PriceSource::Kong,
        }
    }
}

#[derive(Args)]
pub struct PriceArgs {
    /// Token symbol (ETH, BTC) or contract address (0x...)
    #[arg(value_name = "TOKEN")]
    pub token: String,

    /// Chain for contract addresses (ethereum, polygon, arbitrum, etc.)
    #[arg(long, short, default_value = "ethereum", value_name = "CHAIN")]
    pub chain: String,

    /// Source(s) to query (default: all in parallel)
    #[arg(long, short, default_value = "all")]
    pub source: PriceSourceArg,

    /// Query LP token price (uses Curve as priority source)
    #[arg(long)]
    pub lp: bool,

    /// Show raw responses from each source
    #[arg(long)]
    pub show_raw: bool,

    /// Show only aggregated value (no per-source breakdown)
    #[arg(long)]
    pub summary_only: bool,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

/// Price command output
#[derive(Debug, Serialize)]
pub struct PriceOutput {
    pub token: String,
    pub chain: String,
    pub aggregation: PriceAggregation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<SourcePrice>>,
    pub total_latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct SourcePrice {
    pub source: String,
    pub price_usd: Option<f64>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

/// Execute the price command
pub async fn execute(args: &PriceArgs, quiet: bool) -> anyhow::Result<()> {
    let token = &args.token;
    let chain = &args.chain;

    // Handle LP token price query
    if args.lp {
        if !quiet {
            eprintln!(
                "Fetching LP token price for {} on {} (Curve priority)...",
                token, chain
            );
        }

        let result = fetch_lp_price(token, chain).await;

        // Build output
        let sources = if args.summary_only {
            None
        } else {
            Some(
                result
                    .sources
                    .iter()
                    .map(|s| SourcePrice {
                        source: s.source.to_string(),
                        price_usd: s.data.as_ref().map(|p| p.usd),
                        error: s.error.clone(),
                        latency_ms: s.latency_ms,
                    })
                    .collect(),
            )
        };

        let output = PriceOutput {
            token: token.clone(),
            chain: chain.clone(),
            aggregation: result.aggregated,
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
                println!();
                println!("LP Token Price for {}", output.token);
                println!("{}", "=".repeat(40));
                if let Some(best) = &output.aggregation.best_source {
                    println!("  Source:  {} (authoritative)", best);
                }
                println!("  Price:   ${:.6}", output.aggregation.median_usd);
                println!();
                print_table_output(&output, &result.sources);
            }
        }

        return Ok(());
    }

    // Regular token price query
    // If symbol provided, optionally resolve to address for some sources
    let token_for_query = if !is_token_address(token) {
        // Check if we should convert to address for better multi-source support
        if let Some(addr) = symbol_to_eth_address(token) {
            if !quiet {
                eprintln!("Resolved {} to {} on ethereum", token, addr);
            }
            addr.to_string()
        } else {
            token.clone()
        }
    } else {
        token.clone()
    };

    if !quiet {
        eprintln!("Fetching price for {} on {}...", token, chain);
    }

    let result = match args.source {
        PriceSourceArg::All => fetch_prices_all(&token_for_query, chain).await,
        source => {
            let price_source: PriceSource = source.into();
            fetch_prices_parallel(&token_for_query, chain, &[price_source]).await
        }
    };

    // Build output
    let sources = if args.summary_only {
        None
    } else {
        Some(
            result
                .sources
                .iter()
                .map(|s| SourcePrice {
                    source: s.source.to_string(),
                    price_usd: s.data.as_ref().map(|p| p.usd),
                    error: s.error.clone(),
                    latency_ms: s.latency_ms,
                })
                .collect(),
        )
    };

    let output = PriceOutput {
        token: token.clone(),
        chain: chain.clone(),
        aggregation: result.aggregated,
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
    output: &PriceOutput,
    sources: &[crate::aggregator::SourceResult<NormalizedPrice>],
) {
    println!();
    println!("Aggregated Price for {}", output.token);
    println!("{}", "=".repeat(40));
    println!("  Median:  ${:.6}", output.aggregation.median_usd);
    println!("  Mean:    ${:.6}", output.aggregation.mean_usd);
    println!(
        "  Spread:  {:.2}% {}",
        output.aggregation.spread_pct,
        if output.aggregation.sources_agreed {
            "(sources agree)"
        } else {
            "(sources disagree)"
        }
    );
    println!(
        "  Range:   ${:.6} - ${:.6}",
        output.aggregation.min_usd, output.aggregation.max_usd
    );
    println!();

    if output.sources.is_some() {
        println!("Per-Source Breakdown:");
        println!("{}", "-".repeat(60));
        println!(
            "{:<12} {:>14} {:>10} {:>10}",
            "Source", "Price USD", "Latency", "Status"
        );
        println!("{}", "-".repeat(60));

        for source in sources {
            let status = if source.is_success() { "OK" } else { "ERR" };
            let price_str = source
                .data
                .as_ref()
                .map(|p| format!("${:.6}", p.usd))
                .unwrap_or_else(|| "-".to_string());
            let error_note = source
                .error
                .as_ref()
                .map(|e| format!(" ({})", truncate_error(e, 20)))
                .unwrap_or_default();

            println!(
                "{:<12} {:>14} {:>8}ms {:>10}{}",
                source.source, price_str, source.latency_ms, status, error_note
            );
        }
        println!("{}", "-".repeat(60));
    }

    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

fn truncate_error(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
