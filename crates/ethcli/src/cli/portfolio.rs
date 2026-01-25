//! Aggregated portfolio command
//!
//! Fetches wallet token balances from multiple sources in parallel and merges results.

use crate::aggregator::portfolio::{
    fetch_portfolio_all, fetch_portfolio_parallel, PortfolioBalance, PortfolioResult,
    PortfolioSource,
};
use crate::cli::OutputFormat;
use clap::{Args, ValueEnum};
use serde::Serialize;

/// Portfolio source selection for CLI
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum PortfolioSourceArg {
    /// Query all sources in parallel
    #[default]
    All,
    /// Alchemy Portfolio API
    Alchemy,
    /// Moralis Wallet API
    Moralis,
    /// Dune SIM Balances API
    Dsim,
}

impl From<PortfolioSourceArg> for PortfolioSource {
    fn from(arg: PortfolioSourceArg) -> Self {
        match arg {
            PortfolioSourceArg::All => PortfolioSource::All,
            PortfolioSourceArg::Alchemy => PortfolioSource::Alchemy,
            PortfolioSourceArg::Moralis => PortfolioSource::Moralis,
            PortfolioSourceArg::Dsim => PortfolioSource::DuneSim,
        }
    }
}

#[derive(Args)]
pub struct PortfolioArgs {
    /// Wallet address to query
    #[arg(value_name = "ADDRESS")]
    pub address: String,

    /// Chain(s) to query (can be specified multiple times)
    #[arg(long, short, default_value = "ethereum", value_name = "CHAIN")]
    pub chain: Vec<String>,

    /// Source(s) to query (default: all in parallel)
    #[arg(long, short, default_value = "all")]
    pub source: PortfolioSourceArg,

    /// Show per-source breakdown
    #[arg(long)]
    pub show_sources: bool,

    /// Exclude spam tokens
    #[arg(long)]
    pub exclude_spam: bool,

    /// Minimum USD value to show (filter small balances)
    #[arg(long, value_name = "USD")]
    pub min_value: Option<f64>,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

/// Portfolio command output
#[derive(Debug, Serialize)]
pub struct PortfolioOutput {
    pub address: String,
    pub chains: Vec<String>,
    pub aggregation: PortfolioResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<SourcePortfolio>>,
    pub total_latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct SourcePortfolio {
    pub source: String,
    pub token_count: Option<usize>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

/// Execute the portfolio command
pub async fn execute(args: &PortfolioArgs, quiet: bool) -> anyhow::Result<()> {
    let address = &args.address;
    let chains: Vec<&str> = args.chain.iter().map(|s| s.as_str()).collect();

    if !quiet {
        eprintln!("Fetching portfolio for {} on {:?}...", address, args.chain);
    }

    let result = match args.source {
        PortfolioSourceArg::All => fetch_portfolio_all(address, &chains).await,
        source => {
            let portfolio_source: PortfolioSource = source.into();
            fetch_portfolio_parallel(address, &chains, &[portfolio_source]).await
        }
    };

    // Apply filters
    let mut aggregation = result.aggregated;

    // Filter by min value
    if let Some(min_value) = args.min_value {
        aggregation
            .tokens
            .retain(|t| t.usd_value.map(|v| v >= min_value).unwrap_or(false));
        aggregation.token_count = aggregation.tokens.len();
        aggregation.total_usd_value = aggregation.tokens.iter().filter_map(|t| t.usd_value).sum();
    }

    // Build output
    let sources = if args.show_sources {
        Some(
            result
                .sources
                .iter()
                .map(|s| SourcePortfolio {
                    source: s.source.to_string(),
                    token_count: s.data.as_ref().map(|d| d.len()),
                    error: s.error.clone(),
                    latency_ms: s.latency_ms,
                })
                .collect(),
        )
    } else {
        None
    };

    let output = PortfolioOutput {
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
    output: &PortfolioOutput,
    sources: &[crate::aggregator::SourceResult<Vec<PortfolioBalance>>],
) {
    println!();
    println!("Portfolio for {}", output.address);
    println!("{}", "=".repeat(70));
    println!("  Total Value: ${:.2}", output.aggregation.total_usd_value);
    println!("  Tokens:      {}", output.aggregation.token_count);
    println!("  Chains:      {:?}", output.aggregation.chains_covered);
    println!();

    if !output.aggregation.tokens.is_empty() {
        println!("Token Holdings:");
        println!("{}", "-".repeat(90));
        println!(
            "{:<8} {:<16} {:>18} {:>14} {:>12} {:>10}",
            "Chain", "Symbol", "Balance", "USD Value", "Price", "Sources"
        );
        println!("{}", "-".repeat(90));

        for token in &output.aggregation.tokens {
            let usd_str = token
                .usd_value
                .map(|v| format!("${:.2}", v))
                .unwrap_or_else(|| "-".to_string());
            let price_str = token
                .price_usd
                .map(|v| format!("${:.4}", v))
                .unwrap_or_else(|| "-".to_string());
            let sources_count = token.found_in.len();

            let balance_str = if token.balance < 0.0001 && token.balance > 0.0 {
                format!("{:.2e}", token.balance)
            } else if token.balance >= 1_000_000.0 {
                format!("{:.2}M", token.balance / 1_000_000.0)
            } else if token.balance >= 1_000.0 {
                format!("{:.2}K", token.balance / 1_000.0)
            } else {
                format!("{:.4}", token.balance)
            };

            println!(
                "{:<8} {:<16} {:>18} {:>14} {:>12} {:>10}",
                truncate(&token.chain, 8),
                truncate(&token.symbol, 16),
                balance_str,
                usd_str,
                price_str,
                sources_count
            );
        }
        println!("{}", "-".repeat(90));
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
                .map(|d| format!("{} tokens", d.len()))
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
