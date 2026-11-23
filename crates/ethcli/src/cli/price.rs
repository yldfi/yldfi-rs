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
use std::str::FromStr;

#[derive(Args)]
#[command(after_help = r#"Examples:
  ethcli price ETH                              # Price by symbol
  ethcli price 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --chain ethereum  # USDC by address
  ethcli price CRV --source coingecko           # Single source
  ethcli price 0x... --lp                       # LP token price via Curve
  ethcli price BTC --summary-only               # Only aggregated result
  ethcli price ETH --format json                # JSON output
  ethcli price ETH --at 2025-01-01              # Price at midnight UTC that day
  ethcli price 0xA0b8... --at 21000000          # Price at a block's time
  ethcli price CRV --at 30d                     # Price 30 days ago

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

    /// Historical price: a date (`2025-01-01`, midnight UTC), a time
    /// (`2025-01-01T12:00:00Z`), `@<unix>`, an age (`7d`, `12h`) or a block
    /// number. Uses DefiLlama; the output gives the time of the price point
    /// found, which can differ from the time asked for by up to 4 hours.
    #[arg(long, value_name = "WHEN", conflicts_with_all = ["lp", "source", "summary_only", "show_raw"])]
    pub at: Option<String>,

    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
    pub format: OutputFormat,
}

/// A historical price from `--at`
#[derive(Debug, Serialize)]
pub struct HistoricalPrice {
    pub token: String,
    pub chain: String,
    pub source: &'static str,
    /// What was asked for, as written
    pub at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block: Option<u64>,
    pub requested_timestamp: u64,
    pub requested_time: String,
    pub price_usd: f64,
    /// When the price point DefiLlama returned was taken
    pub price_timestamp: u64,
    pub price_time: String,
    /// price_timestamp minus requested_timestamp, in seconds
    pub offset_seconds: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

/// How far from the asked-for time DefiLlama may look for a price point
const HISTORICAL_SEARCH_WIDTH: &str = "4h";

/// Turn `--at` into a Unix timestamp, and the block when one was given
async fn resolve_at(at: &str, chain: &str, now: u64) -> anyhow::Result<(u64, Option<u64>)> {
    if let Some(timestamp) = crate::rpc::block_time::parse_time_spec(at, now) {
        return Ok((timestamp, None));
    }
    let digits = at.trim().replace('_', "");
    let block: u64 = digits.parse().map_err(|_| {
        anyhow::anyhow!(
            "--at '{at}' is not a date, time, @unix timestamp, age (7d) or block number"
        )
    })?;
    let chain_id = crate::config::Chain::from_str(chain)
        .map_err(|e| anyhow::anyhow!("Unknown chain '{chain}': {e}"))?;
    let endpoint = crate::rpc::get_rpc_endpoint(chain_id)?;
    let header = alloy::providers::Provider::get_block_by_number(
        endpoint.provider(),
        alloy::eips::BlockNumberOrTag::Number(block),
    )
    .await
    .map_err(|e| anyhow::anyhow!("Could not read block {block}: {e}"))?
    .ok_or_else(|| anyhow::anyhow!("Block {block} does not exist on {chain} yet"))?;
    Ok((header.header.timestamp, Some(block)))
}

async fn execute_historical(args: &PriceArgs, at: &str, quiet: bool) -> anyhow::Result<()> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_secs();
    let (requested, block) = resolve_at(at, &args.chain, now).await?;
    if requested > now {
        anyhow::bail!("--at '{at}' is in the future");
    }

    let token = if args.token.starts_with("0x") {
        dllma::coins::Token::new(
            crate::aggregator::chain_map::normalize_chain_for_source("llama", &args.chain),
            &args.token,
        )
    } else {
        dllma::coins::Token::coingecko(crate::aggregator::price::symbol_to_coingecko_id(
            &args.token,
        ))
    };
    if !quiet {
        eprintln!(
            "Fetching the price of {} at {}...",
            args.token,
            crate::rpc::block_time::format_timestamp(requested)
        );
    }
    let client = crate::aggregator::price::get_llama_client()
        .map_err(|e| anyhow::anyhow!("DefiLlama client error: {e}"))?;
    let response = client
        .coins()
        .historical_with_search_width(
            requested,
            std::slice::from_ref(&token),
            HISTORICAL_SEARCH_WIDTH,
        )
        .await
        .map_err(|e| anyhow::anyhow!("DefiLlama historical price request failed: {e}"))?;
    let coin = response.coins.get(&token.format()).ok_or_else(|| {
        anyhow::anyhow!(
            "DefiLlama has no price for {} within {HISTORICAL_SEARCH_WIDTH} of {}",
            args.token,
            crate::rpc::block_time::format_timestamp(requested)
        )
    })?;

    let price_timestamp = coin.timestamp.unwrap_or(requested);
    let output = HistoricalPrice {
        token: args.token.clone(),
        chain: args.chain.clone(),
        source: "defillama",
        at: at.to_string(),
        block,
        requested_timestamp: requested,
        requested_time: crate::rpc::block_time::format_timestamp(requested),
        price_usd: coin.price,
        price_timestamp,
        price_time: crate::rpc::block_time::format_timestamp(price_timestamp),
        offset_seconds: price_timestamp as i64 - requested as i64,
        symbol: coin.symbol.clone(),
        confidence: coin.confidence,
    };

    match args.format {
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
        OutputFormat::Ndjson => println!("{}", serde_json::to_string(&output)?),
        OutputFormat::Table => {
            let name = output.symbol.as_deref().unwrap_or(&output.token);
            println!("{name}: ${}", format_usd(output.price_usd));
            match output.block {
                Some(b) => println!("  Asked for: block {b} ({})", output.requested_time),
                None => println!("  Asked for: {}", output.requested_time),
            }
            println!(
                "  Price at:  {} ({:+} s)",
                output.price_time, output.offset_seconds
            );
            println!("  Source:    DefiLlama");
        }
    }
    Ok(())
}

/// A USD price with enough digits for cheap tokens
fn format_usd(price: f64) -> String {
    if price >= 1.0 {
        format!("{price:.4}")
    } else {
        format!("{price:.8}")
    }
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
    if let Some(at) = &args.at {
        return execute_historical(args, at, quiet).await;
    }
    let token = &args.token;
    let chain = &args.chain;

    // Handle LP token price query
    if args.lp {
        if !quiet {
            eprintln!("Fetching LP token price for {} on {}...", token, chain);
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
                    let basis = if best == crate::aggregator::curve_lp::SOURCE {
                        "pool balances x coin prices, on-chain"
                    } else {
                        "single source, not checked on-chain"
                    };
                    println!("  Source:  {} ({})", best, basis);
                }
                println!("  Price:   ${:.6}", output.aggregation.median_usd);
                let agg = &output.aggregation;
                if agg.max_usd > agg.min_usd && !agg.sources_agreed {
                    println!(
                        "  Warning: sources differ by {:.2}% (${:.6} to ${:.6})",
                        agg.spread_pct, agg.min_usd, agg.max_usd
                    );
                }
                println!();
                print_table_output(&output, &result.sources);
            }
        }

        return Ok(());
    }

    // Regular token price query
    // If symbol provided, optionally resolve to address for some sources
    let token_for_query = if !is_token_address(token) {
        // Resolve the symbol on the requested chain: a mainnet address
        // queried on another chain prices a different (or no) token
        match symbol_address_on_chain(token, chain) {
            Some(addr) => {
                if !quiet {
                    eprintln!("Resolved {} to {} on {}", token, addr, chain);
                }
                addr
            }
            None => token.clone(),
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

/// Address of a well-known token symbol on `chain`, if known there. Native
/// symbols (ETH, MATIC, ...) stay symbols: every source prices them by name.
fn symbol_address_on_chain(symbol: &str, chain: &str) -> Option<String> {
    let chain_id = crate::aggregator::chain_id_from_name(chain)?;
    if chain_id == 1 {
        return symbol_to_eth_address(symbol).map(String::from);
    }
    crate::aggregator::swap::tokens::resolve_token(symbol, chain_id)
        .ok()
        .filter(|addr| !addr.eq_ignore_ascii_case(crate::aggregator::swap::NATIVE_TOKEN))
}

#[cfg(test)]
mod symbol_tests {
    use super::symbol_address_on_chain;

    #[tokio::test]
    async fn at_accepts_times_and_rejects_words() {
        let now = 1_800_000_000;
        assert_eq!(
            super::resolve_at("2025-01-01", "ethereum", now)
                .await
                .unwrap(),
            (1_735_689_600, None)
        );
        assert_eq!(
            super::resolve_at("7d", "ethereum", now).await.unwrap(),
            (now - 7 * 86_400, None)
        );
        assert!(super::resolve_at("yesterday", "ethereum", now)
            .await
            .is_err());
        assert_eq!(super::format_usd(3333.54), "3333.5400");
        assert_eq!(super::format_usd(0.269436), "0.26943600");
    }

    #[test]
    fn symbols_resolve_on_the_requested_chain() {
        let mainnet = symbol_address_on_chain("USDC", "ethereum").unwrap();
        let polygon = symbol_address_on_chain("USDC", "polygon");
        assert_ne!(Some(mainnet), polygon);
        assert_eq!(symbol_address_on_chain("ETH", "arbitrum"), None);
    }
}
