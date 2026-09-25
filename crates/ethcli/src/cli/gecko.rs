//! Direct CoinGecko API commands
//!
//! Provides 1:1 access to CoinGecko API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};
use secrecy::ExposeSecret;

#[derive(Args)]
pub struct GeckoArgs {
    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "json")]
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
    // Try config first, then fall back to env var
    // CoinGecko has 3 tiers:
    // - Free public API (no key)
    // - Demo API (x-cg-demo-api-key header)
    // - Pro API (x-cg-pro-api-key header, different base URL)
    let config = ConfigFile::load_default().ok().flatten();
    let gecko_config = config.as_ref().and_then(|c| c.coingecko.as_ref());

    let (client, tier) = if let Some(cfg) = gecko_config {
        if cfg.use_pro {
            // Pro API - requires API key
            if let Some(ref api_key) = cfg.api_key {
                (cgko::Client::pro(api_key.expose_secret())?, GeckoTier::Pro)
            } else if let Ok(api_key) = std::env::var("COINGECKO_API_KEY") {
                (cgko::Client::pro(&api_key)?, GeckoTier::Pro)
            } else {
                anyhow::bail!("CoinGecko Pro enabled but no API key in config or COINGECKO_API_KEY environment")
            }
        } else if let Some(ref api_key) = cfg.api_key {
            // Demo API with key (higher rate limits than free)
            (
                cgko::Client::demo(Some(api_key.expose_secret().to_string()))?,
                GeckoTier::Demo,
            )
        } else {
            // Free public API
            (cgko::Client::new()?, GeckoTier::Public)
        }
    } else if let Ok(api_key) = std::env::var("COINGECKO_API_KEY") {
        // Env var present - check if pro mode
        if std::env::var("COINGECKO_PRO")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false)
        {
            (cgko::Client::pro(&api_key)?, GeckoTier::Pro)
        } else {
            (cgko::Client::demo(Some(api_key))?, GeckoTier::Demo)
        }
    } else {
        (cgko::Client::new()?, GeckoTier::Public)
    };

    let is_onchain = matches!(command, GeckoCommands::Onchain { .. });
    let result = match command {
        GeckoCommands::Simple { action, args } => handle_simple(&client, action, args, quiet).await,
        GeckoCommands::Coins { action, args } => handle_coins(&client, action, args, quiet).await,
        GeckoCommands::Global { action, args } => handle_global(&client, action, args, quiet).await,
        GeckoCommands::Nfts { action, args } => handle_nfts(&client, action, args, quiet).await,
        GeckoCommands::Onchain { action, args } => {
            handle_onchain(&client, action, args, quiet).await
        }
    };
    // Apply plan/key-aware error explanations to *every* gecko command.
    result.map_err(|e| enhance_gecko_error(e, tier, is_onchain))
}

/// Which CoinGecko API tier the client is using
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GeckoTier {
    /// Keyless public API
    Public,
    /// Demo key
    Demo,
    /// Pro (paid) key
    Pro,
}

async fn handle_simple(
    client: &cgko::Client,
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
    client: &cgko::Client,
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
    client: &cgko::Client,
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
    client: &cgko::Client,
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

const GECKO_KEY_HINT: &str = "Set COINGECKO_API_KEY (and COINGECKO_PRO=1 for a Pro key) \
     or configure via: ethcli config set-gecko-key <key>";

/// Explain CoinGecko plan / key errors instead of passing raw JSON through.
///
/// Classification uses the typed error variant/status (see
/// `cgko::error::Error`); the upstream message is always kept.
fn enhance_gecko_error(err: anyhow::Error, tier: GeckoTier, is_onchain: bool) -> anyhow::Error {
    let Some(api_err) = err.downcast_ref::<cgko::error::Error>() else {
        return err;
    };
    match explain_gecko_error(api_err, tier, is_onchain) {
        Some(msg) => anyhow::anyhow!(msg),
        None => err,
    }
}

fn explain_gecko_error(
    err: &cgko::error::Error,
    tier: GeckoTier,
    is_onchain: bool,
) -> Option<String> {
    use cgko::error::ApiError;

    let (status, upstream) = match err {
        ApiError::Api { status, message } => (*status, message.as_str()),
        ApiError::RateLimited { .. } => (429, ""),
        _ => return None,
    };

    // Keyless onchain (GeckoTerminal) requests are rejected by CoinGecko,
    // often first as a 429 "retry after 60s", which wrongly suggests waiting
    // will help.
    if is_onchain && tier == GeckoTier::Public && matches!(status, 401 | 403 | 429) {
        return Some(format!(
            "CoinGecko onchain endpoints require an API key; keyless requests are rejected \
             (upstream returned {status}{}). {GECKO_KEY_HINT}",
            if upstream.is_empty() {
                String::new()
            } else {
                format!(": {upstream}")
            }
        ));
    }

    if matches!(status, 401 | 403) {
        let lower = upstream.to_lowercase();
        // error_code 10005: "This request is limited Pro API subscribers"
        let pro_only = upstream.contains("10005") || lower.contains("pro api");
        let msg = if pro_only && tier != GeckoTier::Pro {
            format!(
                "This CoinGecko endpoint requires a paid Pro API plan \
                 (current: {}). Upstream ({status}): {upstream}. {GECKO_KEY_HINT}",
                match tier {
                    GeckoTier::Public => "no API key",
                    GeckoTier::Demo => "demo key",
                    GeckoTier::Pro => "pro key",
                }
            )
        } else if tier == GeckoTier::Public {
            format!(
                "CoinGecko rejected the keyless request ({status}): {upstream}. {GECKO_KEY_HINT}"
            )
        } else {
            format!(
                "CoinGecko rejected the API key or plan ({status}): {upstream}. {GECKO_KEY_HINT}"
            )
        };
        return Some(msg);
    }
    None
}

async fn handle_onchain(
    client: &cgko::Client,
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

#[cfg(test)]
mod error_tests {
    use super::*;
    use cgko::error::Error;

    #[test]
    fn keyless_onchain_rate_limit_explains_key_requirement() {
        let msg =
            explain_gecko_error(&Error::rate_limited(Some(60)), GeckoTier::Public, true).unwrap();
        assert!(msg.contains("require an API key"), "{msg}");
        assert!(msg.contains("config set-gecko-key"), "{msg}");
    }

    #[test]
    fn pro_only_endpoint_is_explained_with_upstream_message() {
        let body = r#"{"status":{"error_code":10005,"error_message":"This request is limited Pro API subscribers"}}"#;
        let msg = explain_gecko_error(&Error::api(401, body), GeckoTier::Demo, false).unwrap();
        assert!(msg.contains("paid Pro API plan"), "{msg}");
        assert!(msg.contains("demo key"), "{msg}");
        assert!(msg.contains("10005"), "{msg}");
    }

    #[test]
    fn non_auth_errors_pass_through() {
        assert!(explain_gecko_error(&Error::api(404, "nope"), GeckoTier::Public, false).is_none());
        // A public (non-onchain) rate limit is a genuine rate limit.
        assert!(
            explain_gecko_error(&Error::rate_limited(None), GeckoTier::Public, false).is_none()
        );
    }

    #[test]
    fn enhance_downcasts_anyhow() {
        let err = anyhow::Error::from(Error::api(401, "Requests without API key are not allowed"));
        let out = enhance_gecko_error(err, GeckoTier::Public, true).to_string();
        assert!(out.contains("require an API key"), "{out}");
    }
}
