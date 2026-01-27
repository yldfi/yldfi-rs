//! Aggregated portfolio command
//!
//! Fetches wallet token balances from multiple sources in parallel and merges results.
//! Supports querying multiple wallets by address, label, or tag with optional aggregation.

use crate::aggregator::portfolio::{
    fetch_portfolio_all, fetch_portfolio_parallel, MergedToken, PortfolioResult, PortfolioSource,
};
use crate::aggregator::AggregatedResult;
use crate::cli::OutputFormat;
use crate::config::{AddressBook, TokenBlacklist};
use crate::utils::format::truncate_str;
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::collections::HashMap;

/// Type alias for portfolio fetch results
type PortfolioFetchResult =
    AggregatedResult<Vec<crate::aggregator::portfolio::PortfolioBalance>, PortfolioResult>;

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
    /// Uniswap V3 LP positions
    Uniswap,
    /// Yearn vault positions via Kong API
    Yearn,
}

impl From<PortfolioSourceArg> for PortfolioSource {
    fn from(arg: PortfolioSourceArg) -> Self {
        match arg {
            PortfolioSourceArg::All => PortfolioSource::All,
            PortfolioSourceArg::Alchemy => PortfolioSource::Alchemy,
            PortfolioSourceArg::Moralis => PortfolioSource::Moralis,
            PortfolioSourceArg::Dsim => PortfolioSource::DuneSim,
            PortfolioSourceArg::Uniswap => PortfolioSource::Uniswap,
            PortfolioSourceArg::Yearn => PortfolioSource::Yearn,
        }
    }
}

#[derive(Args)]
#[command(after_help = r#"Examples:
  ethcli portfolio 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045
  ethcli portfolio me                           # Use "me" from address book
  ethcli portfolio --tag defi                   # All wallets tagged "defi"
  ethcli portfolio wallet1 wallet2 --aggregate  # Combine multiple wallets
  ethcli portfolio --source alchemy --chain polygon
  ethcli portfolio --min-value 10               # Only tokens worth >$10

Environment Variables:
  ALCHEMY_API_KEY     Required for Alchemy source
  MORALIS_API_KEY     Required for Moralis source
  DUNE_SIM_API_KEY    Required for Dune SIM source
  THEGRAPH_API_KEY    Required for Uniswap positions"#)]
pub struct PortfolioArgs {
    /// Wallet address(es) to query (or labels from address book). Can specify multiple.
    /// If omitted, uses "me" from address book.
    #[arg(value_name = "ADDRESS")]
    pub addresses: Vec<String>,

    /// Query all addresses with this tag from address book
    #[arg(long, short = 't', value_name = "TAG")]
    pub tag: Option<String>,

    /// Aggregate all wallets into a single combined view
    #[arg(long, short = 'a')]
    pub aggregate: bool,

    /// Chain(s) to query (can be specified multiple times)
    #[arg(long, short, default_value = "ethereum", value_name = "CHAIN")]
    pub chain: Vec<String>,

    /// Source(s) to query (default: all in parallel)
    #[arg(long, short, default_value = "all")]
    pub source: PortfolioSourceArg,

    /// Show per-source breakdown
    #[arg(long)]
    pub show_sources: bool,

    /// Include blacklisted tokens (normally filtered out)
    #[arg(long)]
    pub show_blacklisted: bool,

    /// Minimum USD value to show (filter small balances)
    #[arg(long, value_name = "USD")]
    pub min_value: Option<f64>,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

/// Single wallet portfolio output
#[derive(Debug, Clone, Serialize)]
pub struct WalletPortfolio {
    /// Label (if from address book) or address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub address: String,
    pub total_usd_value: f64,
    pub token_count: usize,
    pub tokens: Vec<MergedToken>,
    pub latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklisted_count: Option<usize>,
}

/// Multi-wallet portfolio output
#[derive(Debug, Serialize)]
pub struct MultiPortfolioOutput {
    /// Individual wallet results (when not aggregated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wallets: Option<Vec<WalletPortfolio>>,
    /// Aggregated result (when --aggregate)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregated: Option<AggregatedPortfolio>,
    pub chains: Vec<String>,
    pub total_usd_value: f64,
    pub wallet_count: usize,
    pub total_latency_ms: u64,
}

/// Aggregated portfolio across multiple wallets
#[derive(Debug, Serialize)]
pub struct AggregatedPortfolio {
    pub addresses: Vec<String>,
    pub total_usd_value: f64,
    pub token_count: usize,
    pub tokens: Vec<MergedToken>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklisted_count: Option<usize>,
}

/// Legacy single-wallet output (for backwards compatibility)
#[derive(Debug, Serialize)]
pub struct PortfolioOutput {
    pub address: String,
    pub chains: Vec<String>,
    pub aggregation: PortfolioResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<SourcePortfolio>>,
    pub total_latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blacklisted_count: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct SourcePortfolio {
    pub source: String,
    pub token_count: Option<usize>,
    pub error: Option<String>,
    pub latency_ms: u64,
}

/// Resolved wallet with optional label
struct ResolvedWallet {
    label: Option<String>,
    address: String,
}

/// Execute the portfolio command
pub async fn execute(args: &PortfolioArgs, quiet: bool) -> anyhow::Result<()> {
    let address_book = AddressBook::load_default();
    let blacklist = TokenBlacklist::load_default();
    let chains: Vec<&str> = args.chain.iter().map(|s| s.as_str()).collect();

    // Resolve wallets to query
    let wallets = resolve_wallets(args, &address_book)?;

    if wallets.is_empty() {
        anyhow::bail!(
            "No wallets to query.\n\
             Provide addresses, use --tag to filter by tag, or save 'me' in address book:\n\
             ethcli address add me <your-address>"
        );
    }

    if !quiet {
        if wallets.len() == 1 {
            eprintln!(
                "Fetching portfolio for {} on {:?}...",
                wallets[0].label.as_ref().unwrap_or(&wallets[0].address),
                args.chain
            );
        } else {
            let labels: Vec<_> = wallets
                .iter()
                .map(|w| w.label.as_ref().unwrap_or(&w.address).clone())
                .collect();
            eprintln!(
                "Fetching portfolio for {} wallets ({}) on {:?}...",
                wallets.len(),
                labels.join(", "),
                args.chain
            );
        }
    }

    // Fetch portfolios for all wallets in parallel
    let start = std::time::Instant::now();
    let wallet_results: Vec<(ResolvedWallet, PortfolioFetchResult)> =
        fetch_wallets_parallel(&wallets, &chains, args).await;
    let total_latency_ms = start.elapsed().as_millis() as u64;

    // Apply blacklist and min_value filters
    let wallet_portfolios: Vec<WalletPortfolio> = wallet_results
        .into_iter()
        .map(|(wallet, result)| apply_filters(wallet, result, &blacklist, args, quiet))
        .collect();

    // Calculate grand total
    let grand_total: f64 = wallet_portfolios.iter().map(|w| w.total_usd_value).sum();

    // Build output based on aggregation mode
    if args.aggregate && wallet_portfolios.len() > 1 {
        let aggregated = aggregate_wallets(&wallet_portfolios);
        let output = MultiPortfolioOutput {
            wallets: None,
            aggregated: Some(aggregated),
            chains: args.chain.clone(),
            total_usd_value: grand_total,
            wallet_count: wallet_portfolios.len(),
            total_latency_ms,
        };

        match args.format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
            OutputFormat::Ndjson => println!("{}", serde_json::to_string(&output)?),
            OutputFormat::Table => print_aggregated_table(&output),
        }
    } else if wallet_portfolios.len() > 1 {
        // Multiple wallets, separate view
        let output = MultiPortfolioOutput {
            wallets: Some(wallet_portfolios.clone()),
            aggregated: None,
            chains: args.chain.clone(),
            total_usd_value: grand_total,
            wallet_count: wallet_portfolios.len(),
            total_latency_ms,
        };

        match args.format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
            OutputFormat::Ndjson => println!("{}", serde_json::to_string(&output)?),
            OutputFormat::Table => print_multi_wallet_table(&output, &wallet_portfolios),
        }
    } else {
        // Single wallet - use legacy format for backwards compatibility
        let wp = &wallet_portfolios[0];
        let legacy = PortfolioResult {
            tokens: wp.tokens.clone(),
            token_count: wp.token_count,
            total_usd_value: wp.total_usd_value,
            chains_covered: args.chain.clone(),
        };
        let output = PortfolioOutput {
            address: wp.address.clone(),
            chains: args.chain.clone(),
            aggregation: legacy,
            sources: None,
            total_latency_ms: wp.latency_ms,
            blacklisted_count: wp.blacklisted_count,
        };

        match args.format {
            OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&output)?),
            OutputFormat::Ndjson => println!("{}", serde_json::to_string(&output)?),
            OutputFormat::Table => print_single_wallet_table(&output),
        }
    }

    Ok(())
}

/// Resolve wallet addresses from args (addresses, tag, or default "me")
fn resolve_wallets(
    args: &PortfolioArgs,
    address_book: &AddressBook,
) -> anyhow::Result<Vec<ResolvedWallet>> {
    let mut wallets = Vec::new();

    // If --tag is specified, get all addresses with that tag
    if let Some(tag) = &args.tag {
        let entries = address_book.list(Some(tag));
        if entries.is_empty() {
            anyhow::bail!("No addresses found with tag '{}'", tag);
        }
        for (label, entry) in entries {
            wallets.push(ResolvedWallet {
                label: Some(label.clone()),
                address: entry.address.clone(),
            });
        }
    }

    // Add any explicitly provided addresses
    for addr in &args.addresses {
        let resolved = crate::utils::address::resolve_label(addr);
        let label = if resolved != *addr {
            Some(addr.clone())
        } else {
            // Check if it's in address book for reverse lookup
            address_book
                .entries
                .iter()
                .find(|(_, e)| e.address.eq_ignore_ascii_case(&resolved))
                .map(|(l, _)| l.clone())
        };
        wallets.push(ResolvedWallet {
            label,
            address: resolved,
        });
    }

    // If no wallets specified and no tag, use "me" from address book
    if wallets.is_empty() && args.tag.is_none() {
        let resolved = crate::utils::address::resolve_label("me");
        if resolved == "me" {
            // "me" wasn't found
            return Ok(vec![]);
        }
        wallets.push(ResolvedWallet {
            label: Some("me".to_string()),
            address: resolved,
        });
    }

    // Deduplicate by address (keep first label)
    let mut seen = std::collections::HashSet::new();
    wallets.retain(|w| seen.insert(w.address.to_lowercase()));

    Ok(wallets)
}

/// Fetch portfolios for multiple wallets in parallel
async fn fetch_wallets_parallel(
    wallets: &[ResolvedWallet],
    chains: &[&str],
    args: &PortfolioArgs,
) -> Vec<(ResolvedWallet, PortfolioFetchResult)> {
    use futures::future::join_all;

    let futures: Vec<_> = wallets
        .iter()
        .map(|wallet| {
            let address = wallet.address.clone();
            let label = wallet.label.clone();
            let chains = chains.to_vec();
            let source = args.source;

            async move {
                let result = match source {
                    PortfolioSourceArg::All => fetch_portfolio_all(&address, &chains).await,
                    source => {
                        let portfolio_source: PortfolioSource = source.into();
                        fetch_portfolio_parallel(&address, &chains, &[portfolio_source]).await
                    }
                };
                (ResolvedWallet { label, address }, result)
            }
        })
        .collect();

    join_all(futures).await
}

/// Apply blacklist and min_value filters to a wallet result
fn apply_filters(
    wallet: ResolvedWallet,
    result: PortfolioFetchResult,
    blacklist: &TokenBlacklist,
    args: &PortfolioArgs,
    quiet: bool,
) -> WalletPortfolio {
    let mut tokens = result.aggregated.tokens;
    let mut blacklisted_count = 0;

    // Apply blacklist filter
    if !args.show_blacklisted && !blacklist.is_empty() {
        let original_count = tokens.len();
        tokens.retain(|t| !blacklist.is_blacklisted(&t.address));
        blacklisted_count = original_count - tokens.len();

        if blacklisted_count > 0 && !quiet {
            let label = wallet.label.as_ref().unwrap_or(&wallet.address);
            eprintln!(
                "  {} - filtered {} blacklisted token(s)",
                label, blacklisted_count
            );
        }
    }

    // Apply min_value filter
    if let Some(min_value) = args.min_value {
        tokens.retain(|t| t.usd_value.map(|v| v >= min_value).unwrap_or(false));
    }

    let total_usd_value: f64 = tokens.iter().filter_map(|t| t.usd_value).sum();
    let token_count = tokens.len();

    WalletPortfolio {
        label: wallet.label,
        address: wallet.address,
        total_usd_value,
        token_count,
        tokens,
        latency_ms: result.total_latency_ms,
        blacklisted_count: if blacklisted_count > 0 {
            Some(blacklisted_count)
        } else {
            None
        },
    }
}

/// Aggregate multiple wallet portfolios into one combined view
fn aggregate_wallets(wallets: &[WalletPortfolio]) -> AggregatedPortfolio {
    let mut token_map: HashMap<String, MergedToken> = HashMap::new();
    let mut total_blacklisted = 0;

    for wallet in wallets {
        if let Some(count) = wallet.blacklisted_count {
            total_blacklisted += count;
        }

        for token in &wallet.tokens {
            let key = format!(
                "{}:{}",
                token.chain.to_lowercase(),
                token.address.to_lowercase()
            );

            if let Some(existing) = token_map.get_mut(&key) {
                // Merge balances
                existing.balance += token.balance;
                if let Some(usd) = token.usd_value {
                    existing.usd_value = Some(existing.usd_value.unwrap_or(0.0) + usd);
                }
                // Merge found_in sources
                for source in &token.found_in {
                    if !existing.found_in.contains(source) {
                        existing.found_in.push(source.clone());
                    }
                }
            } else {
                token_map.insert(key, token.clone());
            }
        }
    }

    let mut tokens: Vec<MergedToken> = token_map.into_values().collect();
    tokens.sort_by(|a, b| {
        b.usd_value
            .unwrap_or(0.0)
            .partial_cmp(&a.usd_value.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_usd_value: f64 = tokens.iter().filter_map(|t| t.usd_value).sum();
    let addresses: Vec<String> = wallets.iter().map(|w| w.address.clone()).collect();

    AggregatedPortfolio {
        addresses,
        total_usd_value,
        token_count: tokens.len(),
        tokens,
        blacklisted_count: if total_blacklisted > 0 {
            Some(total_blacklisted)
        } else {
            None
        },
    }
}

/// Print single wallet table (legacy format)
fn print_single_wallet_table(output: &PortfolioOutput) {
    println!();
    println!("Portfolio for {}", output.address);
    println!("{}", "=".repeat(70));
    println!("  Total Value: ${:.2}", output.aggregation.total_usd_value);
    println!("  Tokens:      {}", output.aggregation.token_count);
    println!("  Chains:      {:?}", output.aggregation.chains_covered);
    println!();

    print_token_table(&output.aggregation.tokens);

    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

/// Print multi-wallet table (separate view per wallet)
fn print_multi_wallet_table(output: &MultiPortfolioOutput, wallets: &[WalletPortfolio]) {
    println!();
    println!("Portfolio Summary ({} wallets)", output.wallet_count);
    println!("{}", "=".repeat(70));
    println!("  Grand Total:  ${:.2}", output.total_usd_value);
    println!("  Chains:       {:?}", output.chains);
    println!();

    // Summary table
    println!("{}", "-".repeat(60));
    println!(
        "{:<20} {:>15} {:>10} {:>10}",
        "Wallet", "Value", "Tokens", "Time"
    );
    println!("{}", "-".repeat(60));

    for wallet in wallets {
        let name = wallet
            .label
            .as_ref()
            .map(|l| truncate_str(l, 20).to_string())
            .unwrap_or_else(|| truncate_str(&wallet.address, 20).to_string());

        println!(
            "{:<20} {:>15} {:>10} {:>8}ms",
            name,
            format!("${:.2}", wallet.total_usd_value),
            wallet.token_count,
            wallet.latency_ms
        );
    }
    println!("{}", "-".repeat(60));
    println!();

    // Per-wallet token details
    for wallet in wallets {
        let name = wallet.label.as_ref().unwrap_or(&wallet.address);

        println!("── {} ──", name);
        if wallet.tokens.is_empty() {
            println!("  (no tokens)");
        } else {
            print_token_table(&wallet.tokens);
        }
    }

    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

/// Print aggregated multi-wallet table
fn print_aggregated_table(output: &MultiPortfolioOutput) {
    println!();
    println!("Aggregated Portfolio ({} wallets)", output.wallet_count);
    println!("{}", "=".repeat(70));
    println!("  Total Value: ${:.2}", output.total_usd_value);
    println!("  Chains:      {:?}", output.chains);

    if let Some(ref agg) = output.aggregated {
        println!("  Wallets:");
        for addr in &agg.addresses {
            println!("    - {}", truncate_str(addr, 42));
        }
        println!();

        print_token_table(&agg.tokens);

        if let Some(count) = agg.blacklisted_count {
            println!("  (filtered {} blacklisted tokens)", count);
        }
    }

    println!("Total time: {}ms (parallel)", output.total_latency_ms);
    println!();
}

/// Print token holdings table
fn print_token_table(tokens: &[MergedToken]) {
    if tokens.is_empty() {
        return;
    }

    println!("{}", "-".repeat(90));
    println!(
        "{:<8} {:<16} {:>18} {:>14} {:>12} {:>10}",
        "Chain", "Symbol", "Balance", "USD Value", "Price", "Sources"
    );
    println!("{}", "-".repeat(90));

    for token in tokens {
        let usd_str = token
            .usd_value
            .map(|v| format!("${:.2}", v))
            .unwrap_or_else(|| "-".to_string());
        let price_str = token
            .price_usd
            .map(|v| format!("${:.4}", v))
            .unwrap_or_else(|| "-".to_string());
        let sources_count = token.found_in.len();

        let balance_str = format_balance(token.balance);

        println!(
            "{:<8} {:<16} {:>18} {:>14} {:>12} {:>10}",
            truncate_str(&token.chain, 8),
            truncate_str(&token.symbol, 16),
            balance_str,
            usd_str,
            price_str,
            sources_count
        );
    }
    println!("{}", "-".repeat(90));
    println!();
}

/// Format balance with appropriate precision
fn format_balance(balance: f64) -> String {
    if balance < 0.0001 && balance > 0.0 {
        format!("{:.2e}", balance)
    } else if balance >= 1_000_000.0 {
        format!("{:.2}M", balance / 1_000_000.0)
    } else if balance >= 1_000.0 {
        format!("{:.2}K", balance / 1_000.0)
    } else {
        format!("{:.4}", balance)
    }
}
