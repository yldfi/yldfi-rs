//! Aggregated swap quote command
//!
//! Fetches swap quotes from multiple DEX aggregators in parallel and finds the best quote.

use crate::aggregator::chain_name_to_id;
use crate::aggregator::swap::{
    fetch_quote_from_source, fetch_quotes_all, NormalizedQuote, QuoteAggregation, SwapSource,
};
use crate::cli::OutputFormat;
use crate::utils::format::truncate_str;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

/// Swap source selection for CLI
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum SwapSourceArg {
    /// Query all sources in parallel
    #[default]
    All,
    /// OpenOcean
    #[value(alias = "oo")]
    OpenOcean,
    /// KyberSwap
    #[value(alias = "kyber")]
    KyberSwap,
    /// 0x
    #[value(name = "0x", alias = "zerox")]
    Zerox,
    /// 1inch
    #[value(name = "1inch", alias = "oneinch")]
    OneInch,
    /// CowSwap
    #[value(alias = "cow")]
    CowSwap,
    /// LI.FI
    #[value(alias = "li.fi")]
    LiFi,
    /// Velora (ParaSwap)
    #[value(alias = "paraswap")]
    Velora,
    /// Enso Finance
    #[value(alias = "ensofi")]
    Enso,
}

impl From<SwapSourceArg> for SwapSource {
    fn from(arg: SwapSourceArg) -> Self {
        match arg {
            SwapSourceArg::All => SwapSource::All,
            SwapSourceArg::OpenOcean => SwapSource::OpenOcean,
            SwapSourceArg::KyberSwap => SwapSource::KyberSwap,
            SwapSourceArg::Zerox => SwapSource::Zerox,
            SwapSourceArg::OneInch => SwapSource::OneInch,
            SwapSourceArg::CowSwap => SwapSource::CowSwap,
            SwapSourceArg::LiFi => SwapSource::LiFi,
            SwapSourceArg::Velora => SwapSource::Velora,
            SwapSourceArg::Enso => SwapSource::Enso,
        }
    }
}

impl std::fmt::Display for SwapSourceArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SwapSourceArg::All => write!(f, "all sources"),
            SwapSourceArg::OpenOcean => write!(f, "OpenOcean"),
            SwapSourceArg::KyberSwap => write!(f, "KyberSwap"),
            SwapSourceArg::Zerox => write!(f, "0x"),
            SwapSourceArg::OneInch => write!(f, "1inch"),
            SwapSourceArg::CowSwap => write!(f, "CowSwap"),
            SwapSourceArg::LiFi => write!(f, "LI.FI"),
            SwapSourceArg::Velora => write!(f, "Velora"),
            SwapSourceArg::Enso => write!(f, "Enso"),
        }
    }
}

#[derive(Subcommand)]
pub enum QuoteCommands {
    /// Get the best quote from all aggregators
    Best(QuoteArgs),

    /// Get quote from a specific aggregator
    From {
        /// The aggregator to use
        source: SwapSourceArg,

        #[command(flatten)]
        args: QuoteArgs,
    },

    /// Compare quotes from all aggregators side-by-side
    Compare(QuoteArgs),
}

#[derive(Args, Clone)]
pub struct QuoteArgs {
    /// Input token address (or symbol like ETH, USDC)
    pub token_in: String,

    /// Output token address (or symbol like ETH, USDC)
    pub token_out: String,

    /// Input amount with decimals (e.g., 1000000000000000000 for 1 ETH)
    /// Or use human format like "1.5" with --decimals flag
    pub amount: String,

    /// Chain (ethereum, polygon, arbitrum, etc.) or chain ID
    #[arg(long, short, default_value = "ethereum")]
    pub chain: String,

    /// Sender address (required by some aggregators for accurate quotes)
    #[arg(long, short)]
    pub sender: Option<String>,

    /// Number of decimals for input amount (converts human amount to raw)
    /// E.g., --decimals 18 converts "1.5" to "1500000000000000000"
    #[arg(long, short)]
    pub decimals: Option<u8>,

    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    #[arg(long, default_value = "50")]
    pub slippage: u32,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,

    /// Show transaction data in output
    #[arg(long)]
    pub show_tx: bool,
}

/// Quote command output
#[derive(Debug, Serialize)]
pub struct QuoteOutput {
    pub token_in: String,
    pub token_out: String,
    pub amount_in: String,
    pub chain: String,
    pub chain_id: u64,
    pub aggregation: QuoteAggregation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<QuoteSummary>>,
    pub total_latency_ms: u64,
}

#[derive(Debug, Serialize)]
pub struct QuoteSummary {
    pub source: String,
    pub amount_out: Option<String>,
    pub gas_usd: Option<f64>,
    pub price_impact: Option<f64>,
    pub protocols: Option<Vec<String>>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

/// Execute the quote command
pub async fn execute(cmd: &QuoteCommands, quiet: bool) -> anyhow::Result<()> {
    match cmd {
        QuoteCommands::Best(args) => execute_best(args, quiet).await,
        QuoteCommands::From { source, args } => execute_from(*source, args, quiet).await,
        QuoteCommands::Compare(args) => execute_compare(args, quiet).await,
    }
}

async fn execute_best(args: &QuoteArgs, quiet: bool) -> anyhow::Result<()> {
    let (chain_id, chain_name) = resolve_chain(&args.chain)?;
    let amount = resolve_amount(&args.amount, args.decimals)?;

    if !quiet {
        eprintln!(
            "Finding best quote for {} {} -> {} on {}...",
            amount, args.token_in, args.token_out, chain_name
        );
    }

    let result = fetch_quotes_all(
        chain_id,
        &args.token_in,
        &args.token_out,
        &amount,
        args.sender.as_deref(),
    )
    .await;

    let output = build_output(args, chain_id, &chain_name, &amount, &result, false);

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(&output)?);
        }
        OutputFormat::Table => {
            print_best_output(&output, args.show_tx, &result.sources);
        }
    }

    Ok(())
}

async fn execute_from(source: SwapSourceArg, args: &QuoteArgs, quiet: bool) -> anyhow::Result<()> {
    let (chain_id, chain_name) = resolve_chain(&args.chain)?;
    let amount = resolve_amount(&args.amount, args.decimals)?;
    let swap_source: SwapSource = source.into();

    if !quiet {
        eprintln!(
            "Getting quote from {} for {} {} -> {} on {}...",
            source, amount, args.token_in, args.token_out, chain_name
        );
    }

    let result = fetch_quote_from_source(
        chain_id,
        &args.token_in,
        &args.token_out,
        &amount,
        args.sender.as_deref(),
        swap_source,
    )
    .await;

    match args.format {
        OutputFormat::Json => {
            if let Some(quote) = &result.data {
                println!("{}", serde_json::to_string_pretty(quote)?);
            } else {
                let error = result.error.unwrap_or_else(|| "Unknown error".to_string());
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({
                        "error": error,
                        "source": result.source,
                        "latency_ms": result.latency_ms
                    }))?
                );
            }
        }
        OutputFormat::Ndjson => {
            if let Some(quote) = &result.data {
                println!("{}", serde_json::to_string(quote)?);
            } else {
                let error = result.error.unwrap_or_else(|| "Unknown error".to_string());
                println!(
                    "{}",
                    serde_json::to_string(&serde_json::json!({
                        "error": error,
                        "source": result.source
                    }))?
                );
            }
        }
        OutputFormat::Table => {
            println!();
            println!("Quote from {}", source);
            println!("{}", "=".repeat(50));
            println!();

            if let Some(quote) = &result.data {
                println!("  Token In:    {}", quote.token_in);
                println!("  Token Out:   {}", quote.token_out);
                println!("  Amount In:   {}", quote.amount_in);
                println!("  Amount Out:  {}", quote.amount_out);

                if let Some(min) = &quote.amount_out_min {
                    println!("  Min Out:     {}", min);
                }
                if let Some(gas) = quote.estimated_gas {
                    println!("  Est. Gas:    {}", gas);
                }
                if let Some(gas_usd) = quote.gas_usd {
                    println!("  Gas USD:     ${:.4}", gas_usd);
                }
                if let Some(impact) = quote.price_impact {
                    println!("  Price Impact: {:.4}%", impact);
                }
                if let Some(protocols) = &quote.protocols {
                    println!("  Protocols:   {}", protocols.join(", "));
                }

                if args.show_tx {
                    if let Some(router) = &quote.router_address {
                        println!();
                        println!("  Router:      {}", router);
                    }
                    if let Some(data) = &quote.tx_data {
                        println!("  Tx Data:     {}...", &data[..data.len().min(66)]);
                    }
                    if let Some(value) = &quote.tx_value {
                        println!("  Tx Value:    {}", value);
                    }
                }
            } else {
                println!(
                    "  Error: {}",
                    result.error.unwrap_or_else(|| "Unknown error".to_string())
                );
            }

            println!();
            println!("Latency: {}ms", result.latency_ms);
            println!();
        }
    }

    Ok(())
}

async fn execute_compare(args: &QuoteArgs, quiet: bool) -> anyhow::Result<()> {
    let (chain_id, chain_name) = resolve_chain(&args.chain)?;
    let amount = resolve_amount(&args.amount, args.decimals)?;

    if !quiet {
        eprintln!(
            "Comparing quotes for {} {} -> {} on {}...",
            amount, args.token_in, args.token_out, chain_name
        );
    }

    let result = fetch_quotes_all(
        chain_id,
        &args.token_in,
        &args.token_out,
        &amount,
        args.sender.as_deref(),
    )
    .await;

    let output = build_output(args, chain_id, &chain_name, &amount, &result, true);

    match args.format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(&output)?);
        }
        OutputFormat::Table => {
            print_compare_output(&output, args.show_tx, &result.sources);
        }
    }

    Ok(())
}

fn resolve_chain(chain: &str) -> anyhow::Result<(u64, String)> {
    // Try parsing as chain ID first
    if let Ok(id) = chain.parse::<u64>() {
        let name = crate::aggregator::chain_id_to_name(id)
            .unwrap_or("unknown")
            .to_string();
        return Ok((id, name));
    }

    // Try parsing as chain name
    if let Some(id) = chain_name_to_id(chain) {
        return Ok((id, chain.to_lowercase()));
    }

    Err(anyhow::anyhow!(
        "Unknown chain: {}. Use chain ID or name (ethereum, polygon, arbitrum, etc.)",
        chain
    ))
}

fn resolve_amount(amount: &str, decimals: Option<u8>) -> anyhow::Result<String> {
    if let Some(dec) = decimals {
        // Parse as decimal and convert to raw
        let parts: Vec<&str> = amount.split('.').collect();
        let (whole, frac) = match parts.len() {
            1 => (parts[0], ""),
            2 => (parts[0], parts[1]),
            _ => return Err(anyhow::anyhow!("Invalid amount format")),
        };

        let whole: u128 = whole
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid amount"))?;

        let frac_digits = frac.len();

        // Validate fractional digits don't exceed token decimals
        if frac_digits > dec as usize {
            return Err(anyhow::anyhow!(
                "Too many decimal places: {} has {} decimals, but {} were provided",
                amount,
                dec,
                frac_digits
            ));
        }

        let frac_val: u128 = if frac.is_empty() {
            0
        } else {
            frac.parse()
                .map_err(|_| anyhow::anyhow!("Invalid amount"))?
        };

        // Calculate: whole * 10^dec + frac * 10^(dec - frac_digits)
        let multiplier = 10u128.pow(dec as u32);
        let frac_multiplier = 10u128.pow((dec as usize - frac_digits) as u32);

        // Use checked arithmetic to prevent overflow
        let whole_scaled = whole.checked_mul(multiplier).ok_or_else(|| {
            anyhow::anyhow!(
                "Amount overflow: {} is too large for {} decimals",
                whole,
                dec
            )
        })?;

        let frac_scaled = frac_val
            .checked_mul(frac_multiplier)
            .ok_or_else(|| anyhow::anyhow!("Fractional amount overflow"))?;

        let raw = whole_scaled
            .checked_add(frac_scaled)
            .ok_or_else(|| anyhow::anyhow!("Total amount overflow"))?;

        Ok(raw.to_string())
    } else {
        // Assume already in raw format
        Ok(amount.to_string())
    }
}

fn build_output(
    args: &QuoteArgs,
    chain_id: u64,
    chain_name: &str,
    amount: &str,
    result: &crate::aggregator::AggregatedResult<NormalizedQuote, QuoteAggregation>,
    include_all_quotes: bool,
) -> QuoteOutput {
    let quotes = if include_all_quotes {
        Some(
            result
                .sources
                .iter()
                .map(|s| QuoteSummary {
                    source: s.source.to_string(),
                    amount_out: s.data.as_ref().map(|q| q.amount_out.clone()),
                    gas_usd: s.data.as_ref().and_then(|q| q.gas_usd),
                    price_impact: s.data.as_ref().and_then(|q| q.price_impact),
                    protocols: s.data.as_ref().and_then(|q| q.protocols.clone()),
                    error: s.error.clone(),
                    latency_ms: s.latency_ms,
                })
                .collect(),
        )
    } else {
        None
    };

    QuoteOutput {
        token_in: args.token_in.clone(),
        token_out: args.token_out.clone(),
        amount_in: amount.to_string(),
        chain: chain_name.to_string(),
        chain_id,
        aggregation: result.aggregated.clone(),
        quotes,
        total_latency_ms: result.total_latency_ms,
    }
}

fn print_best_output(
    output: &QuoteOutput,
    show_tx: bool,
    sources: &[crate::aggregator::SourceResult<NormalizedQuote>],
) {
    println!();
    println!("Best Swap Quote");
    println!("{}", "=".repeat(50));
    println!();
    println!("  Swap:       {} -> {}", output.token_in, output.token_out);
    println!("  Amount In:  {}", output.amount_in);
    println!("  Chain:      {} ({})", output.chain, output.chain_id);
    println!();

    if let Some(best) = &output.aggregation.best_quote {
        println!("  Best Source: {}", best.source);
        println!("  Amount Out:  {}", best.amount_out);

        if let Some(gas_usd) = best.gas_usd {
            println!("  Gas Cost:    ${:.4}", gas_usd);
        }
        if let Some(impact) = best.price_impact {
            println!("  Price Impact: {:.4}%", impact);
        }
        if let Some(improvement) = best.improvement_pct {
            println!("  vs Worst:    +{:.2}% better", improvement);
        }

        // Find the best quote source for tx data
        if show_tx {
            if let Some(best_result) = sources.iter().find(|s| s.source == best.source) {
                if let Some(quote) = &best_result.data {
                    if let Some(router) = &quote.router_address {
                        println!();
                        println!("  Router:      {}", router);
                    }
                    if let Some(data) = &quote.tx_data {
                        let display_len = data.len().min(66);
                        println!("  Tx Data:     {}...", &data[..display_len]);
                    }
                    if let Some(value) = &quote.tx_value {
                        println!("  Tx Value:    {}", value);
                    }
                }
            }
        }
    } else {
        println!("  No valid quotes received");
    }

    println!();
    println!(
        "Queried {} sources, {} succeeded",
        output.aggregation.total_sources, output.aggregation.valid_quotes
    );
    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

fn print_compare_output(
    output: &QuoteOutput,
    show_tx: bool,
    sources: &[crate::aggregator::SourceResult<NormalizedQuote>],
) {
    println!();
    println!("Quote Comparison");
    println!("{}", "=".repeat(70));
    println!();
    println!("  Swap:       {} -> {}", output.token_in, output.token_out);
    println!("  Amount In:  {}", output.amount_in);
    println!("  Chain:      {} ({})", output.chain, output.chain_id);
    println!();

    if let Some(best) = &output.aggregation.best_quote {
        println!("  Best:       {} with {}", best.source, best.amount_out);
        if let Some(spread) = output.aggregation.output_spread_pct {
            println!("  Spread:     {:.2}%", spread);
        }
    }

    println!();
    println!("{}", "-".repeat(90));
    println!(
        "{:<12} {:>22} {:>10} {:>12} {:>8} {:>12}",
        "Source", "Amount Out", "Gas USD", "Impact", "Latency", "Status"
    );
    println!("{}", "-".repeat(90));

    // Sort by amount_out descending
    let mut sorted_sources: Vec<_> = sources.iter().collect();
    sorted_sources.sort_by(|a, b| {
        let a_out = a
            .data
            .as_ref()
            .and_then(|q| q.amount_out_u128())
            .unwrap_or(0);
        let b_out = b
            .data
            .as_ref()
            .and_then(|q| q.amount_out_u128())
            .unwrap_or(0);
        b_out.cmp(&a_out)
    });

    for (i, source) in sorted_sources.iter().enumerate() {
        let status = if source.is_success() { "OK" } else { "ERR" };
        let rank = if source.is_success() {
            format!("#{}", i + 1)
        } else {
            "-".to_string()
        };

        let amount_out = source
            .data
            .as_ref()
            .map(|q| format_large_number(&q.amount_out))
            .unwrap_or_else(|| "-".to_string());

        let gas_usd = source
            .data
            .as_ref()
            .and_then(|q| q.gas_usd)
            .map(|g| format!("${:.2}", g))
            .unwrap_or_else(|| "-".to_string());

        let impact = source
            .data
            .as_ref()
            .and_then(|q| q.price_impact)
            .map(|i| format!("{:.2}%", i))
            .unwrap_or_else(|| "-".to_string());

        let latency = format!("{}ms", source.latency_ms);

        println!(
            "{:<12} {:>22} {:>10} {:>12} {:>8} {:>6} {}",
            source.source, amount_out, gas_usd, impact, latency, status, rank
        );

        // Show error if any
        if let Some(err) = &source.error {
            println!("{:>12} Error: {}", "", truncate_str(err, 60));
        }

        // Show protocols if available
        if let Some(quote) = &source.data {
            if let Some(protocols) = &quote.protocols {
                println!("{:>12} via: {}", "", protocols.join(" -> "));
            }
        }
    }

    println!("{}", "-".repeat(90));

    if show_tx {
        println!();
        println!("Transaction Data (best quote):");
        if let Some(best) = &output.aggregation.best_quote {
            if let Some(best_result) = sources.iter().find(|s| s.source == best.source) {
                if let Some(quote) = &best_result.data {
                    if let Some(router) = &quote.router_address {
                        println!("  Router: {}", router);
                    }
                    if let Some(data) = &quote.tx_data {
                        println!("  Data:   {} bytes", data.len() / 2 - 1);
                        println!("          {}...", &data[..data.len().min(66)]);
                    }
                }
            }
        }
    }

    println!();
    println!(
        "Queried {} sources, {} succeeded",
        output.aggregation.total_sources, output.aggregation.valid_quotes
    );
    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

fn format_large_number(s: &str) -> String {
    // For very large numbers, add thousand separators or use scientific notation
    if s.len() > 15 {
        // Scientific notation for very large numbers
        // Note: u128 -> f64 loses precision for values > 2^53, but this is
        // only for display purposes in the comparison table
        if let Ok(n) = s.parse::<u128>() {
            return format!("{:.4e}", n as f64);
        }
    }
    s.to_string()
}

// Use truncate_str from utils::format for Unicode-safe truncation
