//! Yield aggregation CLI commands
//!
//! Combines yield data from Curve and DefiLlama for comprehensive DeFi yield information.

use crate::aggregator::{
    compare_curve_yields, fetch_lending_yields, fetch_yields_aggregated, YieldSource,
};
use crate::cli::OutputFormat;
use clap::Args;

#[derive(Args)]
pub struct YieldsArgs {
    /// Chain to query (default: all chains for DefiLlama, ethereum for Curve)
    #[arg(long, short)]
    pub chain: Option<String>,

    /// Filter by project/protocol name
    #[arg(long, short)]
    pub project: Option<String>,

    /// Source(s) to query: all, curve, llama, uniswap, yearn (default: all)
    #[arg(long, short = 's', default_value = "all")]
    pub source: String,

    /// Show lending yields from Curve
    #[arg(long)]
    pub lending: bool,

    /// Compare Curve yields between sources
    #[arg(long)]
    pub compare: bool,

    /// Minimum APY to show (filter out low yields)
    #[arg(long)]
    pub min_apy: Option<f64>,

    /// Maximum results to display
    #[arg(long, default_value = "50")]
    pub limit: usize,

    /// Show per-source breakdown
    #[arg(long)]
    pub show_sources: bool,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

/// Handle yields command
pub async fn handle(args: &YieldsArgs, quiet: bool) -> anyhow::Result<()> {
    // Handle comparison mode
    if args.compare {
        return handle_compare(args, quiet).await;
    }

    // Handle lending mode
    if args.lending {
        return handle_lending(args, quiet).await;
    }

    // Parse source
    let source: YieldSource = args
        .source
        .parse()
        .map_err(|e: String| anyhow::anyhow!(e))?;

    if !quiet {
        eprintln!("Fetching yields from {}...", source);
    }

    let result =
        fetch_yields_aggregated(args.chain.as_deref(), args.project.as_deref(), source).await;

    // Filter by min APY if specified
    let filtered_yields: Vec<_> = result
        .sources
        .iter()
        .filter_map(|s| s.data.as_ref())
        .flatten()
        .filter(|y| {
            if let Some(min) = args.min_apy {
                y.apy_total.unwrap_or(0.0) >= min
            } else {
                true
            }
        })
        .take(args.limit)
        .collect();

    match args.format {
        OutputFormat::Json => {
            if args.show_sources {
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                let output = serde_json::json!({
                    "aggregated": result.aggregated,
                    "yields": filtered_yields,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            }
        }
        OutputFormat::Ndjson => {
            for y in &filtered_yields {
                println!("{}", serde_json::to_string(y)?);
            }
        }
        OutputFormat::Table => {
            // Print aggregation summary
            println!("\nYield Aggregation Summary");
            println!("=========================");
            println!("Total pools:    {}", result.aggregated.total_pools);
            println!("Curve pools:    {}", result.aggregated.curve_pools);
            println!("Llama pools:    {}", result.aggregated.llama_pools);
            println!(
                "Uniswap pools:  {} (V2: {}, V3: {}, V4: {})",
                result.aggregated.uniswap_pools,
                result.aggregated.uniswap_v2_pools,
                result.aggregated.uniswap_v3_pools,
                result.aggregated.uniswap_v4_pools
            );
            println!("Yearn vaults:   {}", result.aggregated.yearn_vaults);
            if let Some(max) = result.aggregated.max_apy {
                println!("Max APY:        {:.2}%", max);
            }
            if let Some(avg) = result.aggregated.avg_apy {
                println!("Avg APY:        {:.2}%", avg);
            }
            if let Some(tvl) = result.aggregated.total_tvl_usd {
                println!("Total TVL:      ${:.2}M", tvl / 1_000_000.0);
            }
            println!("Query time:     {}ms", result.total_latency_ms);
            println!();

            // Print source status if requested
            if args.show_sources {
                println!("Per-Source Status:");
                for source in &result.sources {
                    let status = if source.is_success() { "OK" } else { "ERR" };
                    let count = source.data.as_ref().map(|d| d.len()).unwrap_or(0);
                    println!(
                        "  {}: {} ({} pools, {}ms)",
                        source.source, status, count, source.latency_ms
                    );
                    if let Some(err) = &source.error {
                        println!("    Error: {}", err);
                    }
                }
                println!();
            }

            // Print yields table
            if filtered_yields.is_empty() {
                println!("No yields found matching criteria.");
            } else {
                println!(
                    "{:<40} {:<12} {:<10} {:<10} {:<12} {:<12}",
                    "Pool", "Project", "Chain", "APY", "TVL", "Source"
                );
                println!("{}", "-".repeat(96));

                for y in &filtered_yields {
                    let symbol = if y.symbol.len() > 38 {
                        format!("{}...", &y.symbol[..35])
                    } else {
                        y.symbol.clone()
                    };
                    let project = if y.project.len() > 10 {
                        format!("{}...", &y.project[..7])
                    } else {
                        y.project.clone()
                    };
                    let chain = if y.chain.len() > 8 {
                        format!("{}...", &y.chain[..5])
                    } else {
                        y.chain.clone()
                    };
                    let apy = y
                        .apy_total
                        .map(|a| format!("{:.2}%", a))
                        .unwrap_or_else(|| "N/A".to_string());
                    let tvl = y
                        .tvl_usd
                        .map(|t| {
                            if t > 1_000_000.0 {
                                format!("${:.1}M", t / 1_000_000.0)
                            } else if t > 1000.0 {
                                format!("${:.1}K", t / 1000.0)
                            } else {
                                format!("${:.0}", t)
                            }
                        })
                        .unwrap_or_else(|| "N/A".to_string());

                    // Determine source based on project name
                    let source = if y.project.to_lowercase().starts_with("uniswap") {
                        "uniswap"
                    } else if y.project.to_lowercase() == "curve" && y.tvl_usd.is_none() {
                        "curve"
                    } else {
                        "llama"
                    };

                    println!(
                        "{:<40} {:<12} {:<10} {:<10} {:<12} {:<12}",
                        symbol, project, chain, apy, tvl, source
                    );
                }
            }
        }
    }

    Ok(())
}

/// Handle lending yields
async fn handle_lending(args: &YieldsArgs, quiet: bool) -> anyhow::Result<()> {
    let chain = args.chain.as_deref().unwrap_or("ethereum");

    if !quiet {
        eprintln!("Fetching Curve lending yields for {}...", chain);
    }

    let result = fetch_lending_yields(chain).await;

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        OutputFormat::Ndjson => {
            if let Some(yields) = result.sources.first().and_then(|s| s.data.as_ref()) {
                for y in yields {
                    println!("{}", serde_json::to_string(y)?);
                }
            }
        }
        OutputFormat::Table => {
            println!("\nCurve Lending Yields - {}", chain);
            println!("=============================");
            println!("Total vaults: {}", result.aggregated.total_vaults);
            if let Some(avg) = result.aggregated.avg_lend_apy {
                println!("Avg Lend APY: {:.2}%", avg);
            }
            if let Some(avg) = result.aggregated.avg_borrow_apy {
                println!("Avg Borrow APY: {:.2}%", avg);
            }
            println!();

            if let Some(yields) = result.sources.first().and_then(|s| s.data.as_ref()) {
                println!(
                    "{:<30} {:<10} {:<10} {:<12} {:<12} {:<10}",
                    "Vault", "Collat.", "Borrow", "Lend APY", "Borrow APY", "Util."
                );
                println!("{}", "-".repeat(84));

                for y in yields.iter().take(args.limit) {
                    let name = y.name.as_deref().unwrap_or(&y.address[..10]);
                    let name = if name.len() > 28 {
                        format!("{}...", &name[..25])
                    } else {
                        name.to_string()
                    };
                    let collateral = y.collateral_symbol.as_deref().unwrap_or("?");
                    let borrowed = y.borrowed_symbol.as_deref().unwrap_or("?");
                    let lend_apy = y
                        .lend_apy
                        .map(|a| format!("{:.2}%", a))
                        .unwrap_or_else(|| "N/A".to_string());
                    let borrow_apy = y
                        .borrow_apy
                        .map(|a| format!("{:.2}%", a))
                        .unwrap_or_else(|| "N/A".to_string());
                    let util = y
                        .utilization
                        .map(|u| format!("{:.1}%", u * 100.0))
                        .unwrap_or_else(|| "N/A".to_string());

                    println!(
                        "{:<30} {:<10} {:<10} {:<12} {:<12} {:<10}",
                        name, collateral, borrowed, lend_apy, borrow_apy, util
                    );
                }
            }
        }
    }

    Ok(())
}

/// Handle yield comparison between sources
async fn handle_compare(args: &YieldsArgs, quiet: bool) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Comparing Curve yields between sources...");
    }

    let comparisons = compare_curve_yields().await;

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&comparisons)?);
        }
        OutputFormat::Ndjson => {
            for c in &comparisons {
                println!("{}", serde_json::to_string(c)?);
            }
        }
        OutputFormat::Table => {
            println!("\nCurve Yield Comparison: Curve API vs DefiLlama");
            println!("==============================================");
            println!();

            if comparisons.is_empty() {
                println!("No matching pools found for comparison.");
            } else {
                println!(
                    "{:<40} {:<10} {:<12} {:<12} {:<12}",
                    "Pool", "Chain", "Curve APY", "Llama APY", "Diff"
                );
                println!("{}", "-".repeat(86));

                for c in comparisons.iter().take(args.limit) {
                    let symbol = if c.symbol.len() > 38 {
                        format!("{}...", &c.symbol[..35])
                    } else {
                        c.symbol.clone()
                    };
                    let chain = if c.chain.len() > 8 {
                        format!("{}...", &c.chain[..5])
                    } else {
                        c.chain.clone()
                    };
                    let diff_str = if c.difference >= 0.0 {
                        format!("+{:.2}%", c.difference)
                    } else {
                        format!("{:.2}%", c.difference)
                    };

                    println!(
                        "{:<40} {:<10} {:<12.2}% {:<12.2}% {:<12}",
                        symbol, chain, c.curve_apy, c.llama_apy, diff_str
                    );
                }
            }
        }
    }

    Ok(())
}
