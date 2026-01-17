//! Direct CoinGecko API commands
//!
//! Provides 1:1 access to CoinGecko API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct GeckoArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum GeckoCommands {
    /// Simple price queries
    Simple {
        #[command(subcommand)]
        action: SimpleCommands,

        #[command(flatten)]
        args: GeckoArgs,
    },

    /// Coin data and markets
    Coins {
        #[command(subcommand)]
        action: CoinsCommands,

        #[command(flatten)]
        args: GeckoArgs,
    },

    /// Global cryptocurrency data
    Global {
        #[command(subcommand)]
        action: GlobalCommands,

        #[command(flatten)]
        args: GeckoArgs,
    },

    /// NFT collections and markets
    Nfts {
        #[command(subcommand)]
        action: NftCommands,

        #[command(flatten)]
        args: GeckoArgs,
    },

    /// Onchain/GeckoTerminal DEX data
    Onchain {
        #[command(subcommand)]
        action: OnchainCommands,

        #[command(flatten)]
        args: GeckoArgs,
    },
}

#[derive(Subcommand)]
pub enum SimpleCommands {
    /// Get price for coins
    Price {
        /// Coin IDs (comma-separated, e.g., "bitcoin,ethereum")
        ids: String,
        /// Target currencies (comma-separated, e.g., "usd,eur")
        #[arg(long, default_value = "usd")]
        vs: String,
    },

    /// Get token price by contract address
    TokenPrice {
        /// Platform (e.g., "ethereum", "polygon-pos")
        platform: String,
        /// Contract addresses (comma-separated)
        addresses: String,
        /// Target currencies (comma-separated)
        #[arg(long, default_value = "usd")]
        vs: String,
    },

    /// List supported vs currencies
    Currencies,
}

#[derive(Subcommand)]
pub enum CoinsCommands {
    /// List all coins
    List {
        /// Include platform contract addresses
        #[arg(long)]
        with_platforms: bool,
    },

    /// Get coin market data
    Markets {
        /// Target currency (e.g., "usd")
        #[arg(default_value = "usd")]
        vs_currency: String,
    },

    /// Get coin data by ID
    Get {
        /// Coin ID (e.g., "bitcoin")
        id: String,
    },

    /// Get coin tickers
    Tickers {
        /// Coin ID
        id: String,
    },

    /// Get coin market chart
    Chart {
        /// Coin ID
        id: String,
        /// Target currency
        #[arg(long, default_value = "usd")]
        vs: String,
        /// Days of data (1, 7, 14, 30, 90, 180, 365, max)
        #[arg(long, default_value = "30")]
        days: String,
    },

    /// Get coin OHLC data
    Ohlc {
        /// Coin ID
        id: String,
        /// Target currency
        #[arg(long, default_value = "usd")]
        vs: String,
        /// Days of data (1, 7, 14, 30, 90, 180, 365)
        #[arg(long, default_value = "7")]
        days: u32,
    },

    /// Get coin historical data for a date
    History {
        /// Coin ID
        id: String,
        /// Date (dd-mm-yyyy format)
        date: String,
    },

    /// Get top gainers and losers
    TopMovers {
        /// Target currency
        #[arg(long, default_value = "usd")]
        vs: String,
        /// Duration: 1h, 24h, 7d, 14d, 30d, 60d, 1y
        #[arg(long, default_value = "24h")]
        duration: String,
    },

    /// Get recently added coins
    New,

    /// Get coin data by contract address
    ByContract {
        /// Platform ID (e.g., "ethereum")
        platform: String,
        /// Contract address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum GlobalCommands {
    /// Ping API status
    Ping,

    /// Get global crypto market data
    Data,

    /// Get global DeFi data
    Defi,

    /// Get trending coins, NFTs, categories
    Trending,

    /// Search coins, exchanges, categories, NFTs
    Search {
        /// Search query
        query: String,
    },

    /// Get BTC exchange rates
    ExchangeRates,

    /// List asset platforms (blockchains)
    Platforms,
}

#[derive(Subcommand)]
pub enum NftCommands {
    /// List NFT collections
    List,

    /// Get NFT collection by ID
    Get {
        /// Collection ID
        id: String,
    },

    /// Get NFT collection by contract
    ByContract {
        /// Platform (e.g., "ethereum")
        platform: String,
        /// Contract address
        address: String,
    },

    /// Get NFT markets data
    Markets,

    /// Get NFT collection tickers
    Tickers {
        /// Collection ID
        id: String,
    },
}

#[derive(Subcommand)]
pub enum OnchainCommands {
    /// List supported networks
    Networks,

    /// List DEXes on a network
    Dexes {
        /// Network ID (e.g., "eth", "polygon_pos")
        network: String,
    },

    /// Get trending pools
    TrendingPools {
        /// Network ID (optional, all networks if omitted)
        network: Option<String>,
    },

    /// Get top pools
    TopPools {
        /// Network ID (optional, all networks if omitted)
        network: Option<String>,
    },

    /// Get new pools
    NewPools {
        /// Network ID (optional, all networks if omitted)
        network: Option<String>,
    },

    /// Get pool data
    Pool {
        /// Network ID
        network: String,
        /// Pool address
        address: String,
    },

    /// Get token data
    Token {
        /// Network ID
        network: String,
        /// Token address
        address: String,
    },

    /// Get token price
    TokenPrice {
        /// Network ID
        network: String,
        /// Token addresses (comma-separated)
        addresses: String,
    },

    /// Get pools for a token
    TokenPools {
        /// Network ID
        network: String,
        /// Token address
        address: String,
    },

    /// Get pool OHLCV data
    PoolOhlcv {
        /// Network ID
        network: String,
        /// Pool address
        address: String,
        /// Timeframe: minute, hour, day
        #[arg(long, default_value = "hour")]
        timeframe: String,
    },

    /// Search pools
    SearchPools {
        /// Search query
        query: String,
    },
}

/// Handle CoinGecko commands
pub async fn handle(command: &GeckoCommands, quiet: bool) -> anyhow::Result<()> {
    // CoinGecko free API doesn't require an API key
    // Pro API key is optional via COINGECKO_API_KEY
    let client = if let Ok(api_key) = std::env::var("COINGECKO_API_KEY") {
        gecko::Client::pro(&api_key)?
    } else {
        gecko::Client::new()?
    };

    match command {
        GeckoCommands::Simple { action, args } => handle_simple(&client, action, args, quiet).await,
        GeckoCommands::Coins { action, args } => handle_coins(&client, action, args, quiet).await,
        GeckoCommands::Global { action, args } => handle_global(&client, action, args, quiet).await,
        GeckoCommands::Nfts { action, args } => handle_nfts(&client, action, args, quiet).await,
        GeckoCommands::Onchain { action, args } => {
            handle_onchain(&client, action, args, quiet).await
        }
    }
}

async fn handle_simple(
    client: &gecko::Client,
    action: &SimpleCommands,
    args: &GeckoArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        SimpleCommands::Price { ids, vs } => {
            if !quiet {
                eprintln!("Fetching prices for {}...", ids);
            }
            let id_list: Vec<&str> = ids.split(',').map(|s| s.trim()).collect();
            let vs_list: Vec<&str> = vs.split(',').map(|s| s.trim()).collect();
            let response = client.simple().price(&id_list, &vs_list).await?;
            print_output(&response, args.format)?;
        }
        SimpleCommands::TokenPrice {
            platform,
            addresses,
            vs,
        } => {
            if !quiet {
                eprintln!("Fetching token prices on {}...", platform);
            }
            let addr_list: Vec<&str> = addresses.split(',').map(|s| s.trim()).collect();
            let vs_list: Vec<&str> = vs.split(',').map(|s| s.trim()).collect();
            let response = client
                .simple()
                .token_price(platform, &addr_list, &vs_list)
                .await?;
            print_output(&response, args.format)?;
        }
        SimpleCommands::Currencies => {
            if !quiet {
                eprintln!("Fetching supported currencies...");
            }
            let response = client.simple().supported_vs_currencies().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_coins(
    client: &gecko::Client,
    action: &CoinsCommands,
    args: &GeckoArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        CoinsCommands::List { with_platforms } => {
            if !quiet {
                eprintln!("Fetching coin list...");
            }
            let response = if *with_platforms {
                client.coins().list_with_platforms().await?
            } else {
                client.coins().list().await?
            };
            print_output(&response, args.format)?;
        }
        CoinsCommands::Markets { vs_currency } => {
            if !quiet {
                eprintln!("Fetching market data...");
            }
            let response = client.coins().markets(vs_currency).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::Get { id } => {
            if !quiet {
                eprintln!("Fetching coin data for {}...", id);
            }
            let response = client.coins().get(id).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::Tickers { id } => {
            if !quiet {
                eprintln!("Fetching tickers for {}...", id);
            }
            let response = client.coins().tickers(id).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::Chart { id, vs, days } => {
            if !quiet {
                eprintln!("Fetching market chart for {}...", id);
            }
            let response = client.coins().market_chart(id, vs, days).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::Ohlc { id, vs, days } => {
            if !quiet {
                eprintln!("Fetching OHLC for {}...", id);
            }
            let response = client.coins().ohlc(id, vs, *days).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::History { id, date } => {
            if !quiet {
                eprintln!("Fetching historical data for {} on {}...", id, date);
            }
            let response = client.coins().history(id, date).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::TopMovers { vs, duration } => {
            if !quiet {
                eprintln!("Fetching top movers ({})...", duration);
            }
            let response = client.coins().top_gainers_losers(vs, duration).await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::New => {
            if !quiet {
                eprintln!("Fetching recently added coins...");
            }
            let response = client.coins().recently_added().await?;
            print_output(&response, args.format)?;
        }
        CoinsCommands::ByContract { platform, address } => {
            if !quiet {
                eprintln!("Fetching coin data for {} on {}...", address, platform);
            }
            let response = client.coins().by_contract(platform, address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_global(
    client: &gecko::Client,
    action: &GlobalCommands,
    args: &GeckoArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        GlobalCommands::Ping => {
            if !quiet {
                eprintln!("Pinging CoinGecko API...");
            }
            let response = client.global().ping().await?;
            print_output(&response, args.format)?;
        }
        GlobalCommands::Data => {
            if !quiet {
                eprintln!("Fetching global market data...");
            }
            let response = client.global().data().await?;
            print_output(&response, args.format)?;
        }
        GlobalCommands::Defi => {
            if !quiet {
                eprintln!("Fetching global DeFi data...");
            }
            let response = client.global().defi().await?;
            print_output(&response, args.format)?;
        }
        GlobalCommands::Trending => {
            if !quiet {
                eprintln!("Fetching trending...");
            }
            let response = client.global().trending().await?;
            print_output(&response, args.format)?;
        }
        GlobalCommands::Search { query } => {
            if !quiet {
                eprintln!("Searching for {}...", query);
            }
            let response = client.global().search(query).await?;
            print_output(&response, args.format)?;
        }
        GlobalCommands::ExchangeRates => {
            if !quiet {
                eprintln!("Fetching BTC exchange rates...");
            }
            let response = client.global().exchange_rates().await?;
            print_output(&response, args.format)?;
        }
        GlobalCommands::Platforms => {
            if !quiet {
                eprintln!("Fetching asset platforms...");
            }
            let response = client.global().asset_platforms().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_nfts(
    client: &gecko::Client,
    action: &NftCommands,
    args: &GeckoArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        NftCommands::List => {
            if !quiet {
                eprintln!("Fetching NFT collections...");
            }
            let response = client.nfts().list().await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Get { id } => {
            if !quiet {
                eprintln!("Fetching NFT collection {}...", id);
            }
            let response = client.nfts().get(id).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ByContract { platform, address } => {
            if !quiet {
                eprintln!("Fetching NFT collection {} on {}...", address, platform);
            }
            let response = client.nfts().by_contract(platform, address).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Markets => {
            if !quiet {
                eprintln!("Fetching NFT markets...");
            }
            let response = client.nfts().markets().await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Tickers { id } => {
            if !quiet {
                eprintln!("Fetching tickers for {}...", id);
            }
            let response = client.nfts().tickers(id).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_onchain(
    client: &gecko::Client,
    action: &OnchainCommands,
    args: &GeckoArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        OnchainCommands::Networks => {
            if !quiet {
                eprintln!("Fetching networks...");
            }
            let response = client.onchain().networks().await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::Dexes { network } => {
            if !quiet {
                eprintln!("Fetching DEXes on {}...", network);
            }
            let response = client.onchain().dexes(network).await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::TrendingPools { network } => {
            if !quiet {
                eprintln!("Fetching trending pools...");
            }
            let response = if let Some(net) = network {
                client.onchain().trending_pools_network(net).await?
            } else {
                client.onchain().trending_pools().await?
            };
            print_output(&response, args.format)?;
        }
        OnchainCommands::TopPools { network } => {
            if !quiet {
                eprintln!("Fetching top pools...");
            }
            let response = if let Some(net) = network {
                client.onchain().top_pools(net).await?
            } else {
                client.onchain().top_pools_all().await?
            };
            print_output(&response, args.format)?;
        }
        OnchainCommands::NewPools { network } => {
            if !quiet {
                eprintln!("Fetching new pools...");
            }
            let response = if let Some(net) = network {
                client.onchain().new_pools(net).await?
            } else {
                client.onchain().new_pools_all().await?
            };
            print_output(&response, args.format)?;
        }
        OnchainCommands::Pool { network, address } => {
            if !quiet {
                eprintln!("Fetching pool {} on {}...", address, network);
            }
            let response = client.onchain().pool(network, address).await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::Token { network, address } => {
            if !quiet {
                eprintln!("Fetching token {} on {}...", address, network);
            }
            let response = client.onchain().token(network, address).await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::TokenPrice { network, addresses } => {
            if !quiet {
                eprintln!("Fetching token prices on {}...", network);
            }
            let addr_list: Vec<&str> = addresses.split(',').map(|s| s.trim()).collect();
            let response = client.onchain().token_price(network, &addr_list).await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::TokenPools { network, address } => {
            if !quiet {
                eprintln!("Fetching pools for token {} on {}...", address, network);
            }
            let response = client.onchain().token_pools(network, address).await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::PoolOhlcv {
            network,
            address,
            timeframe,
        } => {
            if !quiet {
                eprintln!("Fetching OHLCV for pool {} on {}...", address, network);
            }
            let response = client
                .onchain()
                .pool_ohlcv(network, address, timeframe)
                .await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::SearchPools { query } => {
            if !quiet {
                eprintln!("Searching pools for {}...", query);
            }
            let response = client.onchain().search_pools(query).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
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
            // For table format, just use JSON since these are raw API responses
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}
