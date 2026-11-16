//! Direct DefiLlama API commands
//!
//! Provides 1:1 access to DefiLlama API endpoints.

use crate::aggregator::yields::llama_key_hint;
use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct LlamaArgs {
    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum LlamaCommands {
    /// TVL and protocol data
    Tvl {
        #[command(subcommand)]
        action: TvlCommands,

        #[command(flatten)]
        args: LlamaArgs,
    },

    /// Token prices
    Coins {
        #[command(subcommand)]
        action: CoinsCommands,

        #[command(flatten)]
        args: LlamaArgs,
    },

    /// Yields/APY data
    Yields {
        #[command(subcommand)]
        action: YieldsCommands,

        #[command(flatten)]
        args: LlamaArgs,
    },

    /// DEX and trading volumes
    Volumes {
        #[command(subcommand)]
        action: VolumesCommands,

        #[command(flatten)]
        args: LlamaArgs,
    },

    /// Protocol fees and revenue
    Fees {
        #[command(subcommand)]
        action: FeesCommands,

        #[command(flatten)]
        args: LlamaArgs,
    },

    /// Stablecoin data
    Stablecoins {
        #[command(subcommand)]
        action: StablecoinsCommands,

        #[command(flatten)]
        args: LlamaArgs,
    },
}

#[derive(Subcommand)]
pub enum TvlCommands {
    /// List all protocols with TVL
    Protocols,

    /// Get protocol details and historical TVL
    Protocol {
        /// Protocol slug (e.g., "aave", "uniswap")
        slug: String,
    },

    /// Get current TVL for a protocol
    ProtocolTvl {
        /// Protocol slug
        slug: String,
    },

    /// Get TVL of all chains
    Chains,

    /// Get historical TVL for all chains
    HistoricalTvl,

    /// Get historical TVL for a chain
    ChainTvl {
        /// Chain name (e.g., "Ethereum", "Arbitrum")
        chain: String,
    },
}

#[derive(Subcommand)]
pub enum CoinsCommands {
    /// Get current prices
    Current {
        /// Tokens in chain:address format (comma-separated)
        /// e.g., "coingecko:ethereum,ethereum:0xA0b..."
        tokens: String,
    },

    /// Get historical prices at timestamp
    Historical {
        /// Unix timestamp
        timestamp: u64,
        /// Tokens in chain:address format (comma-separated)
        tokens: String,
        /// How far from the timestamp to look for a price, e.g. "10m" or "4h"
        /// (DefiLlama `searchWidth`; its default is 6h)
        #[arg(long)]
        search_width: Option<String>,
    },

    /// Get price chart
    Chart {
        /// Tokens in chain:address format (comma-separated)
        tokens: String,
        /// Number of data points, at most 500 (a count, not seconds: 30 with
        /// period 1d is the last 30 days)
        #[arg(long)]
        span: Option<u64>,
        /// Time between data points: 1d, 4h, 1h (default 24h)
        #[arg(long)]
        period: Option<String>,
    },

    /// Get price percentage changes
    Percentage {
        /// Tokens in chain:address format (comma-separated)
        tokens: String,
    },

    /// Get first recorded price
    FirstPrice {
        /// Tokens in chain:address format (comma-separated)
        tokens: String,
    },

    /// Get block number at timestamp
    Block {
        /// Chain name (e.g., "ethereum", "bsc")
        chain: String,
        /// Unix timestamp
        timestamp: u64,
    },
}

#[derive(Subcommand)]
pub enum YieldsCommands {
    /// Get all yield pools
    Pools,

    /// Get historical chart for a pool
    Chart {
        /// Pool ID
        pool: String,
    },

    /// Yields v2 earn pools, filtered and paginated (Pro key required)
    Earn {
        /// Chain slug, lowercase (comma-separated or repeated; any match)
        #[arg(long, value_delimiter = ',')]
        chain: Vec<String>,
        /// Protocol slug, lowercase, e.g. aave-v3 (comma-separated or repeated; any match)
        #[arg(long, value_delimiter = ',')]
        protocol: Vec<String>,
        /// true: only all-stablecoin pools; false: only the rest
        #[arg(long, value_name = "BOOL")]
        stablecoin: Option<bool>,
        /// true: only single-asset pools; false: only the rest
        #[arg(long, value_name = "BOOL")]
        single_asset: Option<bool>,
        /// true: only pools with impermanent loss risk; false: only the rest
        #[arg(long, value_name = "BOOL")]
        il_risk: Option<bool>,
        /// Minimum TVL in USD
        #[arg(long)]
        min_tvl: Option<f64>,
        /// Page number (1-based)
        #[arg(long)]
        page: Option<u32>,
        /// Page size, 1 to 300 (default 100)
        #[arg(long)]
        limit: Option<u32>,
        /// Every earn pool in one unfiltered list instead of a page
        #[arg(
            long,
            conflicts_with_all = ["chain", "protocol", "stablecoin", "single_asset", "il_risk", "min_tvl", "page", "limit"]
        )]
        all: bool,
    },

    /// Daily history of an earn pool, oldest first (Pro key required)
    EarnHistory {
        /// Earn pool ID
        id: String,
        /// History window (default: full series)
        #[arg(long, value_enum)]
        range: Option<HistoryRangeArg>,
    },

    /// Measured (verified) yield of every simulated earn pool (Pro key required)
    Verified,

    /// Measured yield of one earn pool, with the simulated on-chain calls (Pro key required)
    VerifiedPool {
        /// Earn pool ID
        id: String,
    },

    /// Daily measured history of an earn pool, oldest first (Pro key required)
    VerifiedHistory {
        /// Earn pool ID
        id: String,
        /// History window (default: full series)
        #[arg(long, value_enum)]
        range: Option<HistoryRangeArg>,
    },

    /// Latest data for every borrow market (Pro key required)
    BorrowMarkets,

    /// Daily borrow-side history of a borrow market, oldest first (Pro key required)
    BorrowHistory {
        /// Borrow market ID
        id: String,
        /// History window (default: full series)
        #[arg(long, value_enum)]
        range: Option<HistoryRangeArg>,
    },

    /// Every borrowing route: a collateral position paired with a debt market (Pro key required)
    BorrowRoutes,

    /// Every leveraged looping strategy (Pro key required)
    Loops,
}

/// History window for the yields v2 history commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum HistoryRangeArg {
    /// Last 30 days
    #[value(name = "30d")]
    Days30,
    /// Last 90 days
    #[value(name = "90d")]
    Days90,
    /// The full series
    Max,
}

impl From<HistoryRangeArg> for dllma::yields::HistoryRange {
    fn from(range: HistoryRangeArg) -> Self {
        match range {
            HistoryRangeArg::Days30 => Self::Days30,
            HistoryRangeArg::Days90 => Self::Days90,
            HistoryRangeArg::Max => Self::Max,
        }
    }
}

#[derive(Subcommand)]
pub enum VolumesCommands {
    /// Get DEX volumes overview
    Dex,

    /// Get DEX volumes for a chain
    DexChain {
        /// Chain name
        chain: String,
    },

    /// Get volume for a DEX protocol
    DexProtocol {
        /// Protocol slug
        protocol: String,
    },

    /// Get options trading volumes
    Options,

    /// Get options volumes for a chain
    OptionsChain {
        /// Chain name
        chain: String,
    },

    /// Get open interest overview
    OpenInterest,
}

#[derive(Subcommand)]
pub enum FeesCommands {
    /// Get fees overview
    Overview,

    /// Get fees for a chain
    Chain {
        /// Chain name
        chain: String,
    },

    /// Get fees for a protocol
    Protocol {
        /// Protocol slug
        protocol: String,
    },
}

#[derive(Subcommand)]
pub enum StablecoinsCommands {
    /// List all stablecoins
    List,
}

/// Handle DefiLlama commands
pub async fn handle(command: &LlamaCommands, quiet: bool) -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    // Try config first, then fall back to env var
    // Free tier by default, Pro key optional
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = config
        .as_ref()
        .and_then(|c| c.defillama.as_ref())
        .and_then(|l| l.api_key.as_ref().map(|s| s.expose_secret().to_string()))
        .or_else(|| std::env::var("DEFILLAMA_API_KEY").ok());

    // Opt-in: route the free endpoints through the Pro API (needs a key)
    let pro_routes = config
        .as_ref()
        .and_then(|c| c.defillama.as_ref())
        .is_some_and(|l| l.pro_routes)
        || std::env::var("DEFILLAMA_PRO_ROUTES").is_ok_and(|v| is_truthy(&v));

    let client = if let Some(key) = api_key {
        dllma::Client::with_config(dllma::Config::with_api_key(key).pro_routes(pro_routes))?
    } else {
        if pro_routes && !quiet {
            eprintln!("Note: DefiLlama pro_routes has no effect without an API key");
        }
        dllma::Client::new()?
    };

    match command {
        LlamaCommands::Tvl { action, args } => handle_tvl(&client, action, args, quiet).await,
        LlamaCommands::Coins { action, args } => handle_coins(&client, action, args, quiet).await,
        LlamaCommands::Yields { action, args } => handle_yields(&client, action, args, quiet).await,
        LlamaCommands::Volumes { action, args } => {
            handle_volumes(&client, action, args, quiet).await
        }
        LlamaCommands::Fees { action, args } => handle_fees(&client, action, args, quiet).await,
        LlamaCommands::Stablecoins { action, args } => {
            handle_stablecoins(&client, action, args, quiet).await
        }
    }
}

async fn handle_tvl(
    client: &dllma::Client,
    action: &TvlCommands,
    args: &LlamaArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TvlCommands::Protocols => {
            if !quiet {
                eprintln!("Fetching all protocols...");
            }
            let response = client.tvl().protocols().await?;
            print_output(&response, args.format)?;
        }
        TvlCommands::Protocol { slug } => {
            if !quiet {
                eprintln!("Fetching protocol {}...", slug);
            }
            let response = client.tvl().protocol(slug).await?;
            print_output(&response, args.format)?;
        }
        TvlCommands::ProtocolTvl { slug } => {
            if !quiet {
                eprintln!("Fetching TVL for {}...", slug);
            }
            let response = client.tvl().protocol_tvl(slug).await?;
            print_output(&response, args.format)?;
        }
        TvlCommands::Chains => {
            if !quiet {
                eprintln!("Fetching chain TVLs...");
            }
            let response = client.tvl().chains().await?;
            print_output(&response, args.format)?;
        }
        TvlCommands::HistoricalTvl => {
            if !quiet {
                eprintln!("Fetching historical TVL...");
            }
            let response = client.tvl().historical_tvl().await?;
            print_output(&response, args.format)?;
        }
        TvlCommands::ChainTvl { chain } => {
            if !quiet {
                eprintln!("Fetching historical TVL for {}...", chain);
            }
            let response = client.tvl().chain_historical_tvl(chain).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_coins(
    client: &dllma::Client,
    action: &CoinsCommands,
    args: &LlamaArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        CoinsCommands::Current { tokens } => {
            if !quiet {
                eprintln!("Fetching current prices...");
            }
            let token_list = parse_tokens(tokens)?;
            let response = client.coins().current(&token_list).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::Historical {
            timestamp,
            tokens,
            search_width,
        } => {
            if !quiet {
                eprintln!("Fetching historical prices at {}...", timestamp);
            }
            let token_list = parse_tokens(tokens)?;
            let response = match search_width {
                Some(width) => {
                    client
                        .coins()
                        .historical_with_search_width(*timestamp, &token_list, width)
                        .await?
                }
                None => client.coins().historical(*timestamp, &token_list).await?,
            };
            print_output(&response, args.format)?;
        }
        CoinsCommands::Chart {
            tokens,
            span,
            period,
        } => {
            if !quiet {
                eprintln!("Fetching price chart...");
            }
            let token_list = parse_tokens(tokens)?;
            let response = client
                .coins()
                .chart(&token_list, *span, period.as_deref(), None)
                .await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::Percentage { tokens } => {
            if !quiet {
                eprintln!("Fetching price changes...");
            }
            let token_list = parse_tokens(tokens)?;
            let response = client.coins().percentage(&token_list).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::FirstPrice { tokens } => {
            if !quiet {
                eprintln!("Fetching first prices...");
            }
            let token_list = parse_tokens(tokens)?;
            let response = client.coins().first_price(&token_list).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::Block { chain, timestamp } => {
            if !quiet {
                eprintln!("Fetching block at {} on {}...", timestamp, chain);
            }
            let response = client.coins().block(chain, *timestamp).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_yields(
    client: &dllma::Client,
    action: &YieldsCommands,
    args: &LlamaArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        YieldsCommands::Pools => {
            if !quiet {
                eprintln!("Fetching yield pools...");
            }
            let response = client
                .yields()
                .pools()
                .await
                .map_err(|e| anyhow::anyhow!(llama_key_hint(e.to_string())))?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::Chart { pool } => {
            if !quiet {
                eprintln!("Fetching yield chart for {}...", pool);
            }
            let response = client
                .yields()
                .chart(pool)
                .await
                .map_err(|e| anyhow::anyhow!(llama_key_hint(e.to_string())))?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::Earn {
            chain,
            protocol,
            stablecoin,
            single_asset,
            il_risk,
            min_tvl,
            page,
            limit,
            all,
        } => {
            if *all {
                if !quiet {
                    eprintln!("Fetching all earn pools...");
                }
                let response = client.yields().earn().await.map_err(key_hint)?;
                print_output(&response, args.format)?;
            } else {
                if !quiet {
                    eprintln!("Fetching earn pools...");
                }
                let query = earn_query(
                    chain,
                    protocol,
                    *stablecoin,
                    *single_asset,
                    *il_risk,
                    *min_tvl,
                    *page,
                    *limit,
                );
                let response = client.yields().earn_query(&query).await.map_err(key_hint)?;
                print_output(&response, args.format)?;
            }
        }
        YieldsCommands::EarnHistory { id, range } => {
            if !quiet {
                eprintln!("Fetching earn history for {}...", id);
            }
            let response = client
                .yields()
                .earn_history(id, range.map(Into::into))
                .await
                .map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::Verified => {
            if !quiet {
                eprintln!("Fetching verified earn yields...");
            }
            let response = client.yields().earn_verified().await.map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::VerifiedPool { id } => {
            if !quiet {
                eprintln!("Fetching verified yield for {}...", id);
            }
            let response = client
                .yields()
                .earn_verified_pool(id)
                .await
                .map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::VerifiedHistory { id, range } => {
            if !quiet {
                eprintln!("Fetching verified history for {}...", id);
            }
            let response = client
                .yields()
                .earn_verified_history(id, range.map(Into::into))
                .await
                .map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::BorrowMarkets => {
            if !quiet {
                eprintln!("Fetching borrow markets...");
            }
            let response = client.yields().borrow_markets().await.map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::BorrowHistory { id, range } => {
            if !quiet {
                eprintln!("Fetching borrow history for {}...", id);
            }
            let response = client
                .yields()
                .borrow_market_history(id, range.map(Into::into))
                .await
                .map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::BorrowRoutes => {
            if !quiet {
                eprintln!("Fetching borrow routes...");
            }
            let response = client.yields().borrow_routes().await.map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::Loops => {
            if !quiet {
                eprintln!("Fetching looping strategies...");
            }
            let response = client.yields().loops().await.map_err(key_hint)?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

/// Add the `ethcli config set-defillama` hint to Pro-key errors
fn key_hint(e: dllma::error::Error) -> anyhow::Error {
    anyhow::anyhow!(llama_key_hint(e.to_string()))
}

/// Build the `/yields/v2/earn/query` filters
#[allow(clippy::too_many_arguments)]
fn earn_query(
    chains: &[String],
    protocols: &[String],
    stablecoin: Option<bool>,
    single_asset: Option<bool>,
    il_risk: Option<bool>,
    min_tvl: Option<f64>,
    page: Option<u32>,
    limit: Option<u32>,
) -> dllma::yields::EarnQuery {
    let mut query = dllma::yields::EarnQuery::new();
    for chain in chains.iter().map(|c| c.trim()).filter(|c| !c.is_empty()) {
        query = query.chain(chain.to_lowercase());
    }
    for protocol in protocols.iter().map(|p| p.trim()).filter(|p| !p.is_empty()) {
        query = query.protocol(protocol.to_lowercase());
    }
    if let Some(v) = stablecoin {
        query = query.stablecoin(v);
    }
    if let Some(v) = single_asset {
        query = query.single_asset_exposure(v);
    }
    if let Some(v) = il_risk {
        query = query.impermanent_loss_risk(v);
    }
    if let Some(v) = min_tvl {
        query = query.min_tvl(v);
    }
    if let Some(v) = page {
        query = query.page(v);
    }
    if let Some(v) = limit {
        query = query.limit(v);
    }
    query
}

/// `1`, `true`, `yes` and `on` (any case) enable a boolean env setting
fn is_truthy(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

async fn handle_volumes(
    client: &dllma::Client,
    action: &VolumesCommands,
    args: &LlamaArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        VolumesCommands::Dex => {
            if !quiet {
                eprintln!("Fetching DEX volumes...");
            }
            let response = client.volumes().dex_overview().await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::DexChain { chain } => {
            if !quiet {
                eprintln!("Fetching DEX volumes for {}...", chain);
            }
            let response = client.volumes().dex_chain(chain).await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::DexProtocol { protocol } => {
            if !quiet {
                eprintln!("Fetching volumes for {}...", protocol);
            }
            let response = client.volumes().dex_protocol(protocol).await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::Options => {
            if !quiet {
                eprintln!("Fetching options volumes...");
            }
            let response = client.volumes().options_overview().await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::OptionsChain { chain } => {
            if !quiet {
                eprintln!("Fetching options volumes for {}...", chain);
            }
            let response = client.volumes().options_chain(chain).await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::OpenInterest => {
            if !quiet {
                eprintln!("Fetching open interest...");
            }
            let response = client.volumes().open_interest().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_fees(
    client: &dllma::Client,
    action: &FeesCommands,
    args: &LlamaArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        FeesCommands::Overview => {
            if !quiet {
                eprintln!("Fetching fees overview...");
            }
            let response = client.fees().overview().await?;
            print_output(&response, args.format)?;
        }
        FeesCommands::Chain { chain } => {
            if !quiet {
                eprintln!("Fetching fees for {}...", chain);
            }
            let response = client.fees().chain(chain).await?;
            print_output(&response, args.format)?;
        }
        FeesCommands::Protocol { protocol } => {
            if !quiet {
                eprintln!("Fetching fees for {}...", protocol);
            }
            let response = client.fees().protocol(protocol).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_stablecoins(
    client: &dllma::Client,
    action: &StablecoinsCommands,
    args: &LlamaArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        StablecoinsCommands::List => {
            if !quiet {
                eprintln!("Fetching stablecoins...");
            }
            let response = client.stablecoins().list().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

/// Parse token strings into Token objects
fn parse_tokens(tokens: &str) -> anyhow::Result<Vec<dllma::coins::Token>> {
    let result: Vec<dllma::coins::Token> = tokens
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| {
            // Format is chain:address (e.g., "coingecko:ethereum" or "ethereum:0xA0b...")
            if let Some((chain, addr)) = s.split_once(':') {
                match chain.to_lowercase().as_str() {
                    "coingecko" => dllma::coins::Token::coingecko(addr),
                    "ethereum" => dllma::coins::Token::ethereum(addr),
                    "bsc" => dllma::coins::Token::bsc(addr),
                    "polygon" => dllma::coins::Token::polygon(addr),
                    "arbitrum" => dllma::coins::Token::arbitrum(addr),
                    // Use Token::new() for chains without dedicated constructors
                    "optimism" => dllma::coins::Token::new("optimism", addr),
                    "avalanche" => dllma::coins::Token::new("avax", addr),
                    "fantom" => dllma::coins::Token::new("fantom", addr),
                    "base" => dllma::coins::Token::new("base", addr),
                    _ => dllma::coins::Token::new(chain, addr),
                }
            } else {
                // Assume coingecko ID if no chain specified
                dllma::coins::Token::coingecko(s)
            }
        })
        .collect();
    Ok(result)
}

fn print_output<T: serde::Serialize>(data: &T, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(data)?);
        }
        OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        command: LlamaCommands,
    }

    fn try_yields(args: &[&str]) -> Result<YieldsCommands, clap::Error> {
        let argv = ["llama", "yields"].into_iter().chain(args.iter().copied());
        Cli::try_parse_from(argv).map(|cli| match cli.command {
            LlamaCommands::Yields { action, .. } => action,
            _ => unreachable!(),
        })
    }

    fn yields(args: &[&str]) -> YieldsCommands {
        try_yields(args).unwrap()
    }

    #[test]
    fn earn_filters_parse_into_query() {
        let YieldsCommands::Earn {
            chain,
            protocol,
            stablecoin,
            single_asset,
            il_risk,
            min_tvl,
            page,
            limit,
            all,
        } = yields(&[
            "earn",
            "--chain",
            "Ethereum,base",
            "--protocol",
            "aave-v3",
            "--stablecoin",
            "true",
            "--min-tvl",
            "1000000",
            "--limit",
            "50",
        ])
        else {
            panic!("expected earn");
        };
        assert!(!all);
        let query = earn_query(
            &chain,
            &protocol,
            stablecoin,
            single_asset,
            il_risk,
            min_tvl,
            page,
            limit,
        );
        assert_eq!(
            query.to_query_string(),
            "?chain=ethereum&chain=base&protocol=aave-v3&stablecoin=true&min_tvl=1000000&limit=50"
        );
        assert!(try_yields(&["earn", "--all", "--chain", "ethereum"]).is_err());
        assert!(matches!(
            yields(&["earn", "--all"]),
            YieldsCommands::Earn { all: true, .. }
        ));
    }

    #[test]
    fn history_ranges_parse() {
        assert!(matches!(
            yields(&["earn-history", "pool-1", "--range", "30d"]),
            YieldsCommands::EarnHistory {
                range: Some(HistoryRangeArg::Days30),
                ..
            }
        ));
        assert!(matches!(
            yields(&["verified-history", "pool-1", "--range", "max"]),
            YieldsCommands::VerifiedHistory {
                range: Some(HistoryRangeArg::Max),
                ..
            }
        ));
        assert!(matches!(
            yields(&["borrow-history", "m-1"]),
            YieldsCommands::BorrowHistory { range: None, .. }
        ));
        assert!(try_yields(&["earn-history", "pool-1", "--range", "7d"]).is_err());
        assert_eq!(
            dllma::yields::HistoryRange::from(HistoryRangeArg::Days90).as_str(),
            "90d"
        );
    }

    #[test]
    fn v2_list_commands_parse() {
        assert!(matches!(yields(&["verified"]), YieldsCommands::Verified));
        assert!(matches!(
            yields(&["verified-pool", "pool-1"]),
            YieldsCommands::VerifiedPool { .. }
        ));
        assert!(matches!(
            yields(&["borrow-markets"]),
            YieldsCommands::BorrowMarkets
        ));
        assert!(matches!(
            yields(&["borrow-routes"]),
            YieldsCommands::BorrowRoutes
        ));
        assert!(matches!(yields(&["loops"]), YieldsCommands::Loops));
        // v1 commands are unchanged
        assert!(matches!(yields(&["pools"]), YieldsCommands::Pools));
    }

    #[test]
    fn pro_routes_env_values() {
        for v in ["1", "true", "TRUE", " yes ", "on"] {
            assert!(is_truthy(v), "{v}");
        }
        for v in ["", "0", "false", "off", "no"] {
            assert!(!is_truthy(v), "{v}");
        }
    }

    #[test]
    fn pro_routes_config_key() {
        let cfg: crate::config::LlamaConfig = toml::from_str("pro_routes = true").unwrap();
        assert!(cfg.pro_routes && cfg.api_key.is_none());
        let cfg: crate::config::LlamaConfig = toml::from_str("").unwrap();
        assert!(!cfg.pro_routes);
        // Default (off) isn't written back to the config file
        assert!(!toml::to_string(&cfg).unwrap().contains("pro_routes"));
    }
}
