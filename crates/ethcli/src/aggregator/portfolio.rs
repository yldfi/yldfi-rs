//! Portfolio aggregation from multiple API sources
//!
//! This module fetches wallet token balances from multiple sources in parallel
//! and merges them into a unified view.

use super::{
    chain_map::normalize_chain_for_source, AggregatedResult, LatencyMeasure, SourceResult,
};
use crate::config::ConfigFile;
use futures::future::join_all;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported portfolio data sources
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortfolioSource {
    /// Query all sources in parallel
    All,
    /// Alchemy Portfolio API
    Alchemy,
    /// Moralis Wallet API
    Moralis,
    /// Dune SIM Balances API
    DuneSim,
}

impl PortfolioSource {
    pub fn name(&self) -> &'static str {
        match self {
            PortfolioSource::All => "all",
            PortfolioSource::Alchemy => "alchemy",
            PortfolioSource::Moralis => "moralis",
            PortfolioSource::DuneSim => "dsim",
        }
    }
}

/// Portfolio token balance (distinct from normalize::PortfolioBalance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioBalance {
    /// Token contract address (checksummed) or "native"
    pub address: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: Option<String>,
    /// Chain/network
    pub chain: String,
    /// Raw balance as string (full precision)
    pub balance_raw: String,
    /// Balance formatted with decimals
    pub balance_formatted: f64,
    /// Token decimals
    pub decimals: u8,
    /// USD value (if available)
    pub usd_value: Option<f64>,
    /// Token price in USD
    pub price_usd: Option<f64>,
    /// Is spam token
    pub is_spam: Option<bool>,
    /// Logo URL
    pub logo: Option<String>,
}

impl PortfolioBalance {
    pub fn new(address: &str, symbol: &str, chain: &str, balance_raw: &str, decimals: u8) -> Self {
        let balance_formatted = parse_balance(balance_raw, decimals);
        Self {
            address: address.to_string(),
            symbol: symbol.to_string(),
            name: None,
            chain: chain.to_string(),
            balance_raw: balance_raw.to_string(),
            balance_formatted,
            decimals,
            usd_value: None,
            price_usd: None,
            is_spam: None,
            logo: None,
        }
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = name;
        self
    }

    pub fn with_usd_value(mut self, usd_value: Option<f64>) -> Self {
        self.usd_value = usd_value;
        self
    }

    pub fn with_price_usd(mut self, price_usd: Option<f64>) -> Self {
        self.price_usd = price_usd;
        self
    }

    pub fn with_is_spam(mut self, is_spam: Option<bool>) -> Self {
        self.is_spam = is_spam;
        self
    }

    pub fn with_logo(mut self, logo: Option<String>) -> Self {
        self.logo = logo;
        self
    }
}

/// Aggregated portfolio result (distinct from normalize::PortfolioResult)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioResult {
    /// Total portfolio value in USD
    pub total_usd_value: f64,
    /// Merged token balances (deduplicated by address+chain)
    pub tokens: Vec<MergedToken>,
    /// Chains covered in the query
    pub chains_covered: Vec<String>,
    /// Number of unique tokens
    pub token_count: usize,
}

/// A token merged from multiple sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedToken {
    /// Token contract address
    pub address: String,
    /// Token symbol
    pub symbol: String,
    /// Token name
    pub name: Option<String>,
    /// Chain
    pub chain: String,
    /// Balance (highest precision found)
    pub balance: f64,
    /// Balance raw string
    pub balance_raw: String,
    /// Decimals
    pub decimals: u8,
    /// USD value (average across sources if different)
    pub usd_value: Option<f64>,
    /// Price USD
    pub price_usd: Option<f64>,
    /// Logo URL
    pub logo: Option<String>,
    /// Sources that reported this token
    pub found_in: Vec<String>,
}

/// Fetch portfolio from all available sources in parallel
pub async fn fetch_portfolio_all(
    address: &str,
    chains: &[&str],
) -> AggregatedResult<Vec<PortfolioBalance>, PortfolioResult> {
    let sources = vec![
        PortfolioSource::Alchemy,
        PortfolioSource::Moralis,
        PortfolioSource::DuneSim,
    ];

    fetch_portfolio_parallel(address, chains, &sources).await
}

/// Fetch portfolio from specified sources in parallel
pub async fn fetch_portfolio_parallel(
    address: &str,
    chains: &[&str],
    sources: &[PortfolioSource],
) -> AggregatedResult<Vec<PortfolioBalance>, PortfolioResult> {
    let start = LatencyMeasure::start();

    // Build futures for each source
    let futures: Vec<_> = sources
        .iter()
        .filter(|s| **s != PortfolioSource::All)
        .map(|source| {
            let address = address.to_string();
            let chains: Vec<String> = chains.iter().map(|c| c.to_string()).collect();
            let source = *source;
            async move { fetch_portfolio_from_source(&address, &chains, source).await }
        })
        .collect();

    // Execute ALL in parallel
    let results: Vec<SourceResult<Vec<PortfolioBalance>>> = join_all(futures).await;

    // Merge and deduplicate tokens across sources
    let aggregation = merge_portfolio_results(&results);

    AggregatedResult::new(aggregation, results, start.elapsed_ms())
}

/// Fetch portfolio from a single source
pub async fn fetch_portfolio_from_source(
    address: &str,
    chains: &[String],
    source: PortfolioSource,
) -> SourceResult<Vec<PortfolioBalance>> {
    let measure = LatencyMeasure::start();

    match source {
        PortfolioSource::Alchemy => fetch_alchemy_portfolio(address, chains, measure).await,
        PortfolioSource::Moralis => fetch_moralis_portfolio(address, chains, measure).await,
        PortfolioSource::DuneSim => fetch_dsim_portfolio(address, chains, measure).await,
        PortfolioSource::All => SourceResult::error("all", "Use fetch_portfolio_all instead", 0),
    }
}

/// Fetch portfolio from Alchemy
async fn fetch_alchemy_portfolio(
    address: &str,
    chains: &[String],
    measure: LatencyMeasure,
) -> SourceResult<Vec<PortfolioBalance>> {
    // Get API key from config
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = match config
        .as_ref()
        .and_then(|c| c.alchemy.as_ref())
        .map(|a| a.api_key.expose_secret().to_string())
    {
        Some(key) => key,
        None => match std::env::var("ALCHEMY_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return SourceResult::error(
                    "alchemy",
                    "ALCHEMY_API_KEY not configured",
                    measure.elapsed_ms(),
                )
            }
        },
    };

    // Convert chains to Alchemy network names
    let networks: Vec<String> = chains
        .iter()
        .map(|c| normalize_chain_for_source("alchemy", c))
        .collect();
    let networks_refs: Vec<&str> = networks.iter().map(|s| s.as_str()).collect();

    // Use default network for client initialization
    let network_str = networks
        .first()
        .map(|s| s.as_str())
        .unwrap_or("eth-mainnet");
    let network = crate::cli::simulate::AlchemyArgs::parse_network(network_str);
    let client = match alcmy::Client::new(&api_key, network) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "alchemy",
                format!("Client creation error: {}", e),
                measure.elapsed_ms(),
            );
        }
    };

    // Build address-network pairs
    let addr_networks: Vec<(&str, &[&str])> = vec![(address, networks_refs.as_slice())];

    match client.portfolio().get_token_balances(&addr_networks).await {
        Ok(response) => {
            let mut balances = Vec::new();
            for wallet in &response.data {
                for token in &wallet.token_balances {
                    let balance = PortfolioBalance::new(
                        &token.address,
                        token.symbol.as_deref().unwrap_or("???"),
                        &token.network,
                        &token.balance,
                        token.decimals.unwrap_or(18),
                    )
                    .with_name(token.name.clone())
                    .with_usd_value(token.usd_value)
                    .with_price_usd(token.token_price_usd)
                    .with_logo(token.logo.clone());
                    balances.push(balance);
                }
            }
            SourceResult::success("alchemy", balances, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("alchemy", format!("API error: {}", e), measure.elapsed_ms()),
    }
}

/// Fetch portfolio from Moralis
async fn fetch_moralis_portfolio(
    address: &str,
    chains: &[String],
    measure: LatencyMeasure,
) -> SourceResult<Vec<PortfolioBalance>> {
    // Get API key from config
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = match config
        .as_ref()
        .and_then(|c| c.moralis.as_ref())
        .map(|m| m.api_key.expose_secret().to_string())
    {
        Some(key) => key,
        None => match std::env::var("MORALIS_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return SourceResult::error(
                    "moralis",
                    "MORALIS_API_KEY not configured",
                    measure.elapsed_ms(),
                )
            }
        },
    };

    let client = match mrls::Client::new(&api_key) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "moralis",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    let mut all_balances = Vec::new();

    // Moralis queries one chain at a time
    for chain in chains {
        let chain_name = normalize_chain_for_source("moralis", chain);
        let query = mrls::wallet::WalletQuery::new().chain(chain_name);

        match client
            .wallet()
            .get_token_balances(address, Some(&query))
            .await
        {
            Ok(tokens) => {
                for token in tokens {
                    let balance = PortfolioBalance::new(
                        &token.token_address,
                        token.symbol.as_deref().unwrap_or("???"),
                        chain,
                        &token.balance,
                        token.decimals.unwrap_or(18),
                    )
                    .with_name(token.name.clone())
                    .with_usd_value(token.usd_value)
                    .with_price_usd(token.usd_price)
                    .with_is_spam(token.possible_spam)
                    .with_logo(token.logo.clone());
                    all_balances.push(balance);
                }
            }
            Err(e) => {
                // Log error but continue with other chains
                eprintln!("Moralis error for chain {}: {}", chain, e);
            }
        }
    }

    if all_balances.is_empty() {
        SourceResult::error("moralis", "No balances found", measure.elapsed_ms())
    } else {
        SourceResult::success("moralis", all_balances, measure.elapsed_ms())
    }
}

/// Fetch portfolio from Dune SIM
async fn fetch_dsim_portfolio(
    address: &str,
    chains: &[String],
    measure: LatencyMeasure,
) -> SourceResult<Vec<PortfolioBalance>> {
    // Get API key from config
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = match config
        .as_ref()
        .and_then(|c| c.dune_sim.as_ref())
        .map(|d| d.api_key.expose_secret().to_string())
    {
        Some(key) => key,
        None => match std::env::var("DUNE_SIM_API_KEY") {
            Ok(key) => key,
            Err(_) => {
                return SourceResult::error(
                    "dsim",
                    "DUNE_SIM_API_KEY not configured",
                    measure.elapsed_ms(),
                )
            }
        },
    };

    let client = match dnsim::Client::new(&api_key) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error(
                "dsim",
                format!("Client error: {}", e),
                measure.elapsed_ms(),
            )
        }
    };

    // Build chain IDs filter
    let chain_ids: Vec<&str> = chains.iter().filter_map(|c| chain_to_id(c)).collect();

    let options = if chain_ids.is_empty() {
        dnsim::balances::BalancesOptions::new()
    } else {
        let mut opts = dnsim::balances::BalancesOptions::new();
        opts.chain_ids = Some(chain_ids.join(","));
        opts
    };

    match client.balances().get_with_options(address, &options).await {
        Ok(response) => {
            let balances: Vec<PortfolioBalance> = response
                .balances
                .iter()
                .map(|b| {
                    let mut balance = PortfolioBalance::new(
                        &b.address, &b.symbol, &b.chain, &b.amount, b.decimals,
                    )
                    .with_name(b.name.clone())
                    .with_usd_value(b.value_usd)
                    .with_price_usd(b.price_usd);

                    if let Some(ref meta) = b.token_metadata {
                        balance = balance.with_logo(meta.logo.clone());
                    }

                    balance
                })
                .collect();
            SourceResult::success("dsim", balances, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("dsim", format!("API error: {}", e), measure.elapsed_ms()),
    }
}

/// Merge portfolio results from multiple sources
fn merge_portfolio_results(results: &[SourceResult<Vec<PortfolioBalance>>]) -> PortfolioResult {
    // Key: (lowercase address, chain) -> Vec<(source, balance)>
    let mut token_map: HashMap<(String, String), Vec<(&str, &PortfolioBalance)>> = HashMap::new();
    let mut chains_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    for result in results {
        if let Some(balances) = &result.data {
            for balance in balances {
                let key = (balance.address.to_lowercase(), balance.chain.to_lowercase());
                chains_set.insert(balance.chain.clone());
                token_map
                    .entry(key)
                    .or_default()
                    .push((&result.source, balance));
            }
        }
    }

    let mut tokens: Vec<MergedToken> = token_map
        .into_iter()
        .map(|((addr, chain), entries)| {
            // Take the first entry as base
            let first = entries[0].1;
            let found_in: Vec<String> = entries.iter().map(|(s, _)| s.to_string()).collect();

            // Average USD values across sources that have them
            let usd_values: Vec<f64> = entries.iter().filter_map(|(_, b)| b.usd_value).collect();
            let avg_usd_value = if usd_values.is_empty() {
                None
            } else {
                Some(usd_values.iter().sum::<f64>() / usd_values.len() as f64)
            };

            let prices: Vec<f64> = entries.iter().filter_map(|(_, b)| b.price_usd).collect();
            let avg_price = if prices.is_empty() {
                None
            } else {
                Some(prices.iter().sum::<f64>() / prices.len() as f64)
            };

            // Take highest precision balance
            let best_balance = entries
                .iter()
                .max_by(|(_, a), (_, b)| {
                    a.balance_formatted
                        .partial_cmp(&b.balance_formatted)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(_, b)| b)
                .unwrap_or(&first);

            // Pick best name (prefer non-None)
            let name = entries
                .iter()
                .find_map(|(_, b)| b.name.clone())
                .or_else(|| first.name.clone());

            // Pick best logo
            let logo = entries
                .iter()
                .find_map(|(_, b)| b.logo.clone())
                .or_else(|| first.logo.clone());

            MergedToken {
                address: addr,
                symbol: first.symbol.clone(),
                name,
                chain,
                balance: best_balance.balance_formatted,
                balance_raw: best_balance.balance_raw.clone(),
                decimals: first.decimals,
                usd_value: avg_usd_value,
                price_usd: avg_price,
                logo,
                found_in,
            }
        })
        .collect();

    // Sort by USD value descending
    tokens.sort_by(|a, b| {
        b.usd_value
            .unwrap_or(0.0)
            .partial_cmp(&a.usd_value.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let total_usd_value: f64 = tokens.iter().filter_map(|t| t.usd_value).sum();
    let token_count = tokens.len();
    let chains_covered: Vec<String> = chains_set.into_iter().collect();

    PortfolioResult {
        total_usd_value,
        tokens,
        chains_covered,
        token_count,
    }
}

/// Parse balance string to f64 with decimals
fn parse_balance(balance: &str, decimals: u8) -> f64 {
    // Handle hex strings
    let balance = if let Some(stripped) = balance.strip_prefix("0x") {
        u128::from_str_radix(stripped, 16)
            .map(|v| v.to_string())
            .unwrap_or_else(|_| balance.to_string())
    } else {
        balance.to_string()
    };

    // Parse as u128 and divide by 10^decimals
    if let Ok(raw) = balance.parse::<u128>() {
        let divisor = 10u128.pow(decimals as u32);
        raw as f64 / divisor as f64
    } else {
        balance.parse::<f64>().unwrap_or(0.0)
    }
}

/// Map chain name to chain ID for dsim
fn chain_to_id(chain: &str) -> Option<&'static str> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" | "eth-mainnet" => Some("1"),
        "polygon" | "matic" | "polygon-mainnet" => Some("137"),
        "arbitrum" | "arb" | "arbitrum-mainnet" | "arb-mainnet" => Some("42161"),
        "optimism" | "op" | "optimism-mainnet" | "op-mainnet" => Some("10"),
        "base" | "base-mainnet" => Some("8453"),
        "avalanche" | "avax" => Some("43114"),
        "bsc" | "bnb" => Some("56"),
        _ => None,
    }
}
