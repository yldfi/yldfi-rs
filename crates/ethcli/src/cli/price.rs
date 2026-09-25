//! Aggregated price command
//!
//! Fetches token prices from multiple sources in parallel and aggregates results.

use crate::aggregator::{
    fetch_lp_price, fetch_prices_all, fetch_prices_parallel, is_token_address,
    symbol_to_eth_address, NormalizedPrice, PriceAggregation, PriceSource, SourceResult,
};
use crate::cli::OutputFormat;
use clap::Args;
use serde::Serialize;

#[derive(Args)]
#[command(after_help = r#"Examples:
  ethcli price ETH                              # Price by symbol
  ethcli price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain ethereum  # USDC by address
  ethcli price CRV --source coingecko           # Single source
  ethcli price 0x... --lp                       # LP token price via Curve
  ethcli price BTC --summary-only               # Only aggregated result
  ethcli price ETH --format json                # JSON output

Environment Variables:
  COINGECKO_API_KEY   Optional: Increases CoinGecko rate limit
  DEFILLAMA_API_KEY   Optional: DefiLlama Pro endpoints
  ALCHEMY_API_KEY     Optional: For Alchemy price source
  MORALIS_API_KEY     Optional: For Moralis price source"#)]
pub struct PriceArgs {
    /// Token symbol (ETH, BTC) or contract address (0x...)
    #[arg(value_name = "TOKEN")]
    pub token: String,

    /// Chain for contract addresses (ethereum, polygon, arbitrum, etc.)
    #[arg(long, short, default_value = "ethereum", value_name = "CHAIN")]
    pub chain: String,

    /// Source(s) to query (default: all in parallel)
    #[arg(long, short, default_value = "all")]
    pub source: PriceSource,

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
    #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
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
        ensure_any_price(token, chain, &result.sources)?;

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
        PriceSource::All => fetch_prices_all(&token_for_query, chain).await,
        source => fetch_prices_parallel(&token_for_query, chain, &[source]).await,
    };

    // Never report a fabricated $0 "median" when every source failed.
    ensure_any_price(token, chain, &result.sources)?;

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

fn print_table_output(output: &PriceOutput, sources: &[SourceResult<NormalizedPrice>]) {
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
            println!(
                "{:<12} {:>14} {:>8}ms {:>10}",
                source.source, price_str, source.latency_ms, status
            );
        }
        println!("{}", "-".repeat(60));
        print_error_footnotes(sources);
    }

    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

/// Print full (untruncated) error messages for failed sources below the table.
fn print_error_footnotes(sources: &[SourceResult<NormalizedPrice>]) {
    let failed: Vec<_> = sources
        .iter()
        .filter_map(|s| s.error.as_ref().map(|e| (s.source.as_ref(), e)))
        .collect();
    if failed.is_empty() {
        return;
    }
    println!("Errors:");
    for (source, err) in failed {
        println!("  {source}: {err}");
    }
}

/// Fail (non-zero exit) when no price source returned a price.
///
/// Previously the command printed "Median $0.000000 (sources disagree)" and
/// exited 0, which is indistinguishable from a real (zero) price to scripts.
fn ensure_any_price(
    token: &str,
    chain: &str,
    sources: &[SourceResult<NormalizedPrice>],
) -> anyhow::Result<()> {
    match no_price_error(token, chain, sources) {
        Some(msg) => Err(anyhow::anyhow!(msg)),
        None => Ok(()),
    }
}

fn no_price_error(
    token: &str,
    chain: &str,
    sources: &[SourceResult<NormalizedPrice>],
) -> Option<String> {
    if sources.iter().any(|s| s.is_success()) {
        return None;
    }
    let mut msg = format!(
        "No price found for {token} on {chain}: all {} source(s) failed",
        sources.len()
    );
    for s in sources {
        let err = s.error.as_deref().unwrap_or("no data returned");
        msg.push_str(&format!("\n  {}: {}", s.source, err));
    }
    Some(msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_price_error_when_all_sources_fail() {
        let sources = vec![
            SourceResult::<NormalizedPrice>::error("alchemy", "Rate limited by Alchemy (429)", 5),
            SourceResult::<NormalizedPrice>::error(
                "curve",
                "Curve requires contract address (0x...)",
                1,
            ),
        ];
        let msg = no_price_error("ETH", "ethereum", &sources).expect("should error");
        assert!(msg.contains("all 2 source(s) failed"), "{msg}");
        // Full, untruncated upstream messages are kept.
        assert!(
            msg.contains("Curve requires contract address (0x...)"),
            "{msg}"
        );
        assert!(
            msg.contains("alchemy: Rate limited by Alchemy (429)"),
            "{msg}"
        );
    }

    #[test]
    fn no_error_when_any_source_succeeds() {
        let sources = vec![
            SourceResult::error("alchemy", "boom", 5),
            SourceResult::success("gecko", NormalizedPrice::new(2690.0), 5),
        ];
        assert!(no_price_error("ETH", "ethereum", &sources).is_none());
    }
}
