//! Direct DefiLlama API commands
//!
//! Provides 1:1 access to DefiLlama API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct LlamaArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
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
    },

    /// Get price chart
    Chart {
        /// Tokens in chain:address format (comma-separated)
        tokens: String,
        /// Time span in seconds
        #[arg(long)]
        span: Option<u64>,
        /// Period: 1d, 4h, 1h
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
    // Try config first, then fall back to env var
    // Free tier by default, Pro key optional
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = config
        .as_ref()
        .and_then(|c| c.defillama.as_ref())
        .and_then(|l| l.api_key.clone())
        .or_else(|| std::env::var("DEFILLAMA_API_KEY").ok());

    let client = if let Some(key) = api_key {
        dllma::Client::with_api_key(&key)?
    } else {
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
        CoinsCommands::Historical { timestamp, tokens } => {
            if !quiet {
                eprintln!("Fetching historical prices at {}...", timestamp);
            }
            let token_list = parse_tokens(tokens)?;
            let response = client.coins().historical(*timestamp, &token_list).await?;
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
            let response = client.yields().pools().await?;
            print_output(&response, args.format)?;
        }
        YieldsCommands::Chart { pool } => {
            if !quiet {
                eprintln!("Fetching yield chart for {}...", pool);
            }
            let response = client.yields().chart(pool).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
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
