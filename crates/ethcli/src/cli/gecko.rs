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

    /// Tokenized real-world assets (stocks, commodities, ETFs)
    #[command(visible_alias = "rwa")]
    Rwas {
        #[command(subcommand)]
        action: RwasCommands,

        #[command(flatten)]
        args: GeckoArgs,
    },

    /// Onchain wallet balances, transfers, trades and PnL (Analyst plan or above)
    #[command(visible_alias = "wallet")]
    Wallets {
        #[command(subcommand)]
        action: WalletsCommands,

        #[command(flatten)]
        args: GeckoArgs,
    },

    /// Latest crypto news and guides (Analyst plan or above)
    News {
        /// Only news about this coin ID (also returns guides)
        #[arg(long)]
        coin: Option<String>,
        /// Language code, e.g. en (default), de, ja, zh-tw
        #[arg(long)]
        language: Option<String>,
        /// all (default), news or guides (guides needs --coin)
        #[arg(long = "type", value_name = "TYPE")]
        news_type: Option<String>,
        /// Page, 1 to 20
        #[arg(long)]
        page: Option<u32>,
        /// Results per page, 1 to 20
        #[arg(long)]
        per_page: Option<u32>,

        #[command(flatten)]
        args: GeckoArgs,
    },

    /// Latest coin insights (Analyst plan or above)
    Insights {
        /// Only insights about this coin ID
        #[arg(long)]
        coin: Option<String>,
        /// Start date (YYYY-MM-DD)
        #[arg(long)]
        from: Option<String>,
        /// End date (YYYY-MM-DD)
        #[arg(long)]
        to: Option<String>,
        /// Page, 1 to 20
        #[arg(long)]
        page: Option<u32>,
        /// Results per page, 1 to 20
        #[arg(long)]
        per_page: Option<u32>,

        #[command(flatten)]
        args: GeckoArgs,
    },
}

#[derive(Subcommand)]
pub enum RwasCommands {
    /// List every RWA (ID, symbol, name, asset type)
    List {
        /// Filter: stock, commodity or etf
        #[arg(long)]
        asset_type: Option<String>,
    },

    /// RWAs with price, market cap, volume and price changes
    Markets {
        /// Filter: stock, commodity or etf
        #[arg(long)]
        asset_type: Option<String>,
        /// RWA IDs (comma-separated)
        #[arg(long)]
        ids: Option<String>,
        /// Issuer ID (see: gecko rwas issuers)
        #[arg(long)]
        issuer: Option<String>,
        /// market_cap_desc (default), market_cap_asc, volume_desc, volume_asc, id_asc, id_desc
        #[arg(long)]
        order: Option<String>,
        /// Results per page, 1 to 250 (default 100)
        #[arg(long)]
        per_page: Option<u32>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
    },

    /// Get an RWA's metadata
    Get {
        /// RWA ID (e.g., "1-800-flowers-com")
        id: String,
        /// Also include its tokens, tokenized and perpetuals market data
        #[arg(long)]
        full: bool,
    },

    /// List RWA issuers (ID and name)
    Issuers,

    /// Get an issuer's market data and tokens
    Issuer {
        /// Issuer ID
        id: String,
    },

    /// CEX and DEX tickers of the tokens tracking an RWA (Basic plan or above)
    Tickers {
        /// RWA ID
        id: String,
        /// Page number (100 tickers per page)
        #[arg(long)]
        page: Option<u32>,
        /// volume_desc (default), volume_asc, last_traded_at_desc, last_traded_at_asc
        #[arg(long)]
        order: Option<String>,
        /// Include 2% order book depth
        #[arg(long)]
        depth: bool,
    },

    /// Historical tokenized price, market cap and volume (Basic plan or above)
    Chart {
        /// RWA ID
        id: String,
        /// Days of data (number or "max")
        #[arg(long, default_value = "30")]
        days: String,
        /// Data interval: 5m, hourly or daily (default: chosen from --days)
        #[arg(long)]
        interval: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum WalletsCommands {
    /// Token balances across networks (Analyst plan or above)
    Balances {
        /// Wallet address
        address: String,
        /// Network IDs, comma-separated, all EVM or all non-EVM (e.g., "eth,base")
        #[arg(long, required = true)]
        networks: String,
        /// value_usd_desc (default) or value_usd_asc
        #[arg(long)]
        sort: Option<String>,
        /// native or non_native (both when omitted)
        #[arg(long)]
        token_type: Option<String>,
        /// Minimum holding value in USD
        #[arg(long)]
        min_value_usd: Option<f64>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Results per page, 1 to 100 (default 50)
        #[arg(long)]
        per_page: Option<u32>,
    },

    /// Token transfers on one network, native transfers excluded (Analyst plan or above)
    Transfers {
        /// Network ID (e.g., "eth")
        network: String,
        /// Wallet address
        address: String,
        /// Only this token contract address
        #[arg(long)]
        token: Option<String>,
        /// Window start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (default: last 7 days)
        #[arg(long, requires = "to")]
        from: Option<String>,
        /// Window end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp
        #[arg(long, requires = "from")]
        to: Option<String>,
        /// in (received) or out (sent); both when omitted
        #[arg(long)]
        direction: Option<String>,
        /// meta.next_cursor from the previous page
        #[arg(long)]
        cursor: Option<String>,
        /// Results per page, 1 to 300 (default 100)
        #[arg(long)]
        per_page: Option<u32>,
    },

    /// DEX trades on one network (Analyst plan or above)
    Trades {
        /// Network ID (e.g., "eth")
        network: String,
        /// Wallet address
        address: String,
        /// Only trades involving this token contract address
        #[arg(long)]
        token: Option<String>,
        /// Window start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (default: last 7 days)
        #[arg(long, requires = "to")]
        from: Option<String>,
        /// Window end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp
        #[arg(long, requires = "from")]
        to: Option<String>,
        /// Include trades flagged as anomalous
        #[arg(long)]
        include_anomalous: bool,
        /// meta.next_cursor from the previous page
        #[arg(long)]
        cursor: Option<String>,
        /// Results per page, 1 to 300 (default 100)
        #[arg(long)]
        per_page: Option<u32>,
    },

    /// Realized and unrealized PnL across networks (Analyst plan or above)
    Pnl {
        /// Wallet address
        address: String,
        /// Network IDs, comma-separated, all EVM or all non-EVM (e.g., "eth,base")
        #[arg(long, required = true)]
        networks: String,
        /// realized_pnl_usd_desc (default), unrealized_pnl_usd_desc, total_buy_usd_desc, total_sell_usd_desc
        #[arg(long)]
        sort: Option<String>,
        /// Page number
        #[arg(long)]
        page: Option<u32>,
        /// Results per page, 1 to 300 (default 100)
        #[arg(long)]
        per_page: Option<u32>,
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

    /// Get circulating vs non-circulating supply and the wallets holding it (Analyst plan or above)
    SupplyBreakdown {
        /// Coin ID (e.g., "uniswap")
        id: String,
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
        /// Network ID. If omitted, uses the megafilter endpoint sorted by
        /// 24h volume across all networks (requires Analyst plan or above)
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

    /// Get token prices across networks in one request (Enterprise plan)
    TokenPriceMulti {
        /// Up to 50 network:address pairs, comma-separated (e.g., "eth:0xc02a...,base:0x4200...")
        tokens: String,
        /// Include market cap
        #[arg(long)]
        market_cap: bool,
        /// Include 24h volume
        #[arg(long)]
        volume: bool,
        /// Include 24h price change
        #[arg(long)]
        price_change: bool,
        /// Include total reserve in USD
        #[arg(long)]
        reserve: bool,
    },

    /// Get token data across networks in one request (Enterprise plan)
    TokensMulti {
        /// Up to 50 network:address pairs, comma-separated
        tokens: String,
        /// Include each token's top pools
        #[arg(long)]
        top_pools: bool,
        /// Include pool composition (with --top-pools)
        #[arg(long, requires = "top_pools")]
        composition: bool,
    },

    /// Get pool trades; --from/--to selects a time range (Analyst plan or above)
    PoolTrades {
        /// Network ID
        network: String,
        /// Pool address
        address: String,
        /// Range start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (at most 30 days before --to)
        #[arg(long, requires = "to")]
        from: Option<String>,
        /// Range end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp
        #[arg(long, requires = "from")]
        to: Option<String>,
        /// Lookback period without a range: 1d (default), 7d, 30d (7d/30d: Analyst plan)
        #[arg(long, conflicts_with = "from")]
        period: Option<String>,
        /// Trade side: base (default), quote or a token address
        #[arg(long)]
        token: Option<String>,
        /// Only trades worth more than this many USD
        #[arg(long)]
        min_volume_usd: Option<f64>,
        /// Include trades flagged as anomalous
        #[arg(long)]
        include_anomalous: bool,
        /// meta.next_cursor from the previous page (Analyst plan)
        #[arg(long)]
        cursor: Option<String>,
        /// Results per page, 1 to 300 (Analyst plan)
        #[arg(long)]
        per_page: Option<u32>,
    },

    /// Get token trades across all pools; --from/--to selects a time range (paid API; range: Analyst plan or above)
    TokenTrades {
        /// Network ID
        network: String,
        /// Token address
        address: String,
        /// Range start: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp (at most 30 days before --to)
        #[arg(long, requires = "to")]
        from: Option<String>,
        /// Range end: YYYY-MM-DD, YYYY-MM-DDTHH:MM or Unix timestamp
        #[arg(long, requires = "from")]
        to: Option<String>,
        /// Lookback period without a range: 1d (default), 7d, 30d (7d/30d: Analyst plan)
        #[arg(long, conflicts_with = "from")]
        period: Option<String>,
        /// Only trades worth more than this many USD
        #[arg(long)]
        min_volume_usd: Option<f64>,
        /// Include trades flagged as anomalous
        #[arg(long)]
        include_anomalous: bool,
        /// meta.next_cursor from the previous page (Analyst plan)
        #[arg(long)]
        cursor: Option<String>,
        /// Results per page, 1 to 300 (Analyst plan)
        #[arg(long)]
        per_page: Option<u32>,
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

    if tier != GeckoTier::Pro {
        if let Some(plan) = paid_plan_required(command) {
            anyhow::bail!(
                "This CoinGecko endpoint requires a paid plan ({plan} or above; current: {}). \
                 {GECKO_KEY_HINT}",
                match tier {
                    GeckoTier::Public => "no API key",
                    GeckoTier::Demo => "demo key",
                    GeckoTier::Pro => "pro key",
                }
            );
        }
    }

    let is_onchain = matches!(
        command,
        GeckoCommands::Onchain { .. } | GeckoCommands::Wallets { .. }
    );
    let result = match command {
        GeckoCommands::Simple { action, args } => handle_simple(&client, action, args, quiet).await,
        GeckoCommands::Coins { action, args } => handle_coins(&client, action, args, quiet).await,
        GeckoCommands::Global { action, args } => handle_global(&client, action, args, quiet).await,
        GeckoCommands::Nfts { action, args } => handle_nfts(&client, action, args, quiet).await,
        GeckoCommands::Onchain { action, args } => {
            handle_onchain(&client, action, args, quiet).await
        }
        GeckoCommands::Rwas { action, args } => handle_rwas(&client, action, args, quiet).await,
        GeckoCommands::Wallets { action, args } => {
            handle_wallets(&client, action, args, quiet).await
        }
        GeckoCommands::News {
            coin,
            language,
            news_type,
            page,
            per_page,
            args,
        } => {
            if !quiet {
                eprintln!("Fetching news...");
            }
            let options = cgko::news::NewsOptions {
                page: *page,
                per_page: *per_page,
                coin_id: coin.clone(),
                language: language.clone(),
                news_type: news_type.clone(),
            };
            match client.news().news(&options).await {
                Ok(response) => print_output(&response, args.format),
                Err(e) => Err(e.into()),
            }
        }
        GeckoCommands::Insights {
            coin,
            from,
            to,
            page,
            per_page,
            args,
        } => {
            if !quiet {
                eprintln!("Fetching insights...");
            }
            let options = cgko::news::InsightsOptions {
                page: *page,
                per_page: *per_page,
                coin_id: coin.clone(),
                from: from.clone(),
                to: to.clone(),
            };
            match client.news().insights(&options).await {
                Ok(response) => print_output(&response, args.format),
                Err(e) => Err(e.into()),
            }
        }
    };
    // Apply plan/key-aware error explanations to *every* gecko command.
    result.map_err(|e| enhance_gecko_error(e, tier, is_onchain))
}

/// The paid CoinGecko plan a command needs, if any.
///
/// Keyless and Demo requests for these would fail upstream (keyless
/// onchain paths even go to GeckoTerminal, which doesn't serve them), so
/// they are refused before any request is sent.
fn paid_plan_required(command: &GeckoCommands) -> Option<&'static str> {
    match command {
        GeckoCommands::Wallets { .. }
        | GeckoCommands::News { .. }
        | GeckoCommands::Insights { .. }
        | GeckoCommands::Coins {
            action: CoinsCommands::SupplyBreakdown { .. },
            ..
        } => Some("Analyst"),
        GeckoCommands::Rwas {
            action: RwasCommands::Tickers { .. } | RwasCommands::Chart { .. },
            ..
        } => Some("Basic"),
        GeckoCommands::Onchain { action, .. } => match action {
            OnchainCommands::TokenPriceMulti { .. } | OnchainCommands::TokensMulti { .. } => {
                Some("Enterprise")
            }
            OnchainCommands::PoolTrades { from: Some(_), .. }
            | OnchainCommands::TokenTrades { from: Some(_), .. } => Some("Analyst"),
            _ => None,
        },
        _ => None,
    }
}

/// Split a comma-separated list, dropping empty entries
fn split_list(s: &str) -> Vec<&str> {
    s.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect()
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
        CoinsCommands::SupplyBreakdown { id } => {
            if !quiet {
                eprintln!("Fetching supply breakdown for {}...", id);
            }
            let response = client.coins().supply_breakdown(id).await?;
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

async fn handle_rwas(
    client: &cgko::Client,
    action: &RwasCommands,
    args: &GeckoArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        RwasCommands::List { asset_type } => {
            if !quiet {
                eprintln!("Fetching RWA list...");
            }
            let response = client.rwas().list(asset_type.as_deref()).await?;
            print_output(&response, args.format)?;
        }
        RwasCommands::Markets {
            asset_type,
            ids,
            issuer,
            order,
            per_page,
            page,
        } => {
            if !quiet {
                eprintln!("Fetching RWA markets...");
            }
            let options = cgko::rwas::RwaMarketsOptions {
                asset_type: asset_type.clone(),
                ids: ids
                    .as_deref()
                    .map(|ids| split_list(ids).into_iter().map(String::from).collect()),
                issuer: issuer.clone(),
                order: order.clone(),
                per_page: *per_page,
                page: *page,
                ..Default::default()
            };
            let response = client.rwas().markets(&options).await?;
            print_output(&response, args.format)?;
        }
        RwasCommands::Get { id, full } => {
            if !quiet {
                eprintln!("Fetching RWA {}...", id);
            }
            let response = if *full {
                client
                    .rwas()
                    .get_with_options(id, &cgko::rwas::RwaDataOptions::all())
                    .await?
            } else {
                client.rwas().get(id).await?
            };
            print_output(&response, args.format)?;
        }
        RwasCommands::Issuers => {
            if !quiet {
                eprintln!("Fetching RWA issuers...");
            }
            let response = client.rwas().issuers_list().await?;
            print_output(&response, args.format)?;
        }
        RwasCommands::Issuer { id } => {
            if !quiet {
                eprintln!("Fetching RWA issuer {}...", id);
            }
            let response = client.rwas().issuer(id).await?;
            print_output(&response, args.format)?;
        }
        RwasCommands::Tickers {
            id,
            page,
            order,
            depth,
        } => {
            if !quiet {
                eprintln!("Fetching tickers for RWA {}...", id);
            }
            let options = cgko::rwas::RwaTickersOptions {
                page: *page,
                order: order.clone(),
                depth: depth.then_some(true),
                ..Default::default()
            };
            let response = client.rwas().tickers_with_options(id, &options).await?;
            print_output(&response, args.format)?;
        }
        RwasCommands::Chart { id, days, interval } => {
            if !quiet {
                eprintln!("Fetching market chart for RWA {}...", id);
            }
            let options = cgko::rwas::RwaMarketChartOptions {
                interval: interval.clone(),
                ..Default::default()
            };
            let response = client.rwas().market_chart(id, days, &options).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_wallets(
    client: &cgko::Client,
    action: &WalletsCommands,
    args: &GeckoArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        WalletsCommands::Balances {
            address,
            networks,
            sort,
            token_type,
            min_value_usd,
            page,
            per_page,
        } => {
            if !quiet {
                eprintln!("Fetching balances for {}...", address);
            }
            let options = cgko::wallets::WalletBalancesOptions {
                sort: sort.clone(),
                token_type: token_type.clone(),
                value_usd_min: *min_value_usd,
                page: *page,
                per_page: *per_page,
                ..Default::default()
            };
            let response = client
                .wallets()
                .balances(address, &split_list(networks), &options)
                .await?;
            print_output(&response, args.format)?;
        }
        WalletsCommands::Transfers {
            network,
            address,
            token,
            from,
            to,
            direction,
            cursor,
            per_page,
        } => {
            if !quiet {
                eprintln!("Fetching transfers for {} on {}...", address, network);
            }
            let options = cgko::wallets::WalletTransfersOptions {
                token: token.clone(),
                from: from.clone(),
                to: to.clone(),
                direction: direction.clone(),
                cursor: cursor.clone(),
                per_page: *per_page,
            };
            let response = client
                .wallets()
                .transfers(network, address, &options)
                .await?;
            print_output(&response, args.format)?;
        }
        WalletsCommands::Trades {
            network,
            address,
            token,
            from,
            to,
            include_anomalous,
            cursor,
            per_page,
        } => {
            if !quiet {
                eprintln!("Fetching trades for {} on {}...", address, network);
            }
            let options = cgko::wallets::WalletTradesOptions {
                token: token.clone(),
                from: from.clone(),
                to: to.clone(),
                include_anomalous: include_anomalous.then_some(true),
                cursor: cursor.clone(),
                per_page: *per_page,
            };
            let response = client.wallets().trades(network, address, &options).await?;
            print_output(&response, args.format)?;
        }
        WalletsCommands::Pnl {
            address,
            networks,
            sort,
            page,
            per_page,
        } => {
            if !quiet {
                eprintln!("Fetching PnL for {}...", address);
            }
            let options = cgko::wallets::WalletPnlOptions {
                sort: sort.clone(),
                page: *page,
                per_page: *per_page,
            };
            let response = client
                .wallets()
                .pnl(address, &split_list(networks), &options)
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

const GECKO_KEY_HINT: &str = "Set COINGECKO_API_KEY (and COINGECKO_PRO=1 for a Pro key) \
     or add api_key under [coingecko] in the config file (see: ethcli config path)";

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

    // Keyless onchain requests go to GeckoTerminal's free API (see cgko's
    // `Client::url_for`): a 429 is its rate limit, a 401/403 an endpoint
    // only CoinGecko's paid onchain API serves.
    if is_onchain && tier == GeckoTier::Public && matches!(status, 401 | 403 | 429) {
        let what = if status == 429 {
            "GeckoTerminal's free API is rate-limited (a few calls a minute); wait a minute and retry"
        } else {
            "this onchain endpoint isn't on GeckoTerminal's free API and needs a CoinGecko key"
        };
        return Some(format!(
            "{what} (upstream returned {status}{}). {GECKO_KEY_HINT}",
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
                // CoinGecko has no all-network top pools endpoint; megafilter
                // sorted by 24h volume is the closest equivalent (paid plans).
                let options =
                    cgko::onchain::MegafilterOptions::new().with_sort("h24_volume_usd_desc");
                client.onchain().megafilter(&options).await.map_err(|e| {
                    anyhow::anyhow!(
                        "{e}\nhint: top pools across all networks uses the megafilter \
                         endpoint (Analyst plan or above); pass a network ID instead"
                    )
                })?
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
        OnchainCommands::TokenPriceMulti {
            tokens,
            market_cap,
            volume,
            price_change,
            reserve,
        } => {
            if !quiet {
                eprintln!("Fetching token prices across networks...");
            }
            let token_list = split_list(tokens);
            let options = cgko::onchain::TokenPriceOptions {
                include_market_cap: market_cap.then_some(true),
                include_24hr_vol: volume.then_some(true),
                include_24hr_price_change: price_change.then_some(true),
                include_total_reserve_in_usd: reserve.then_some(true),
                ..Default::default()
            };
            let response = client
                .onchain()
                .token_price_multi_network(&token_list, &options)
                .await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::TokensMulti {
            tokens,
            top_pools,
            composition,
        } => {
            if !quiet {
                eprintln!("Fetching token data across networks...");
            }
            let token_list = split_list(tokens);
            let options = cgko::onchain::TokensMultiNetworkOptions {
                include: top_pools.then(|| "top_pools".to_string()),
                include_composition: composition.then_some(true),
                ..Default::default()
            };
            let response = client
                .onchain()
                .tokens_multi_network(&token_list, &options)
                .await?;
            print_output(&response, args.format)?;
        }
        OnchainCommands::PoolTrades {
            network,
            address,
            from,
            to,
            period,
            token,
            min_volume_usd,
            include_anomalous,
            cursor,
            per_page,
        } => {
            if !quiet {
                eprintln!("Fetching trades for pool {} on {}...", address, network);
            }
            let options = cgko::onchain::TradesOptions {
                trade_volume_in_usd_greater_than: *min_volume_usd,
                token: token.clone(),
                trading_period: period.clone(),
                include_anomalous: include_anomalous.then_some(true),
                cursor: cursor.clone(),
                per_page: *per_page,
            };
            let response = match (from, to) {
                (Some(from), Some(to)) => {
                    client
                        .onchain()
                        .pool_trades_range(network, address, from, to, &options)
                        .await?
                }
                _ => {
                    client
                        .onchain()
                        .pool_trades_with_options(network, address, &options)
                        .await?
                }
            };
            print_output(&response, args.format)?;
        }
        OnchainCommands::TokenTrades {
            network,
            address,
            from,
            to,
            period,
            min_volume_usd,
            include_anomalous,
            cursor,
            per_page,
        } => {
            if !quiet {
                eprintln!("Fetching trades for token {} on {}...", address, network);
            }
            let options = cgko::onchain::TradesOptions {
                trade_volume_in_usd_greater_than: *min_volume_usd,
                token: None,
                trading_period: period.clone(),
                include_anomalous: include_anomalous.then_some(true),
                cursor: cursor.clone(),
                per_page: *per_page,
            };
            let response = match (from, to) {
                (Some(from), Some(to)) => {
                    client
                        .onchain()
                        .token_trades_range(network, address, from, to, &options)
                        .await?
                }
                _ => {
                    client
                        .onchain()
                        .token_trades_with_options(network, address, &options)
                        .await?
                }
            };
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
    fn keyless_onchain_rate_limit_is_geckoterminals() {
        let msg =
            explain_gecko_error(&Error::rate_limited(Some(60)), GeckoTier::Public, true).unwrap();
        assert!(
            msg.contains("GeckoTerminal's free API is rate-limited"),
            "{msg}"
        );
        assert!(msg.contains("[coingecko]"), "{msg}");
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
        assert!(out.contains("needs a CoinGecko key"), "{out}");
    }
}

#[cfg(test)]
mod parse_tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        command: GeckoCommands,
    }

    fn try_parse(args: &[&str]) -> Result<GeckoCommands, clap::Error> {
        let argv = std::iter::once("gecko").chain(args.iter().copied());
        Cli::try_parse_from(argv).map(|cli| cli.command)
    }

    fn parse(args: &[&str]) -> GeckoCommands {
        try_parse(args).unwrap()
    }

    fn fails(args: &[&str]) -> bool {
        try_parse(args).is_err()
    }

    #[test]
    fn rwas_subcommands_parse() {
        assert!(matches!(
            parse(&["rwas", "list", "--asset-type", "stock"]),
            GeckoCommands::Rwas {
                action: RwasCommands::List { asset_type: Some(t) },
                ..
            } if t == "stock"
        ));
        assert!(matches!(
            parse(&["rwas", "markets", "--ids", "a,b", "--per-page", "10"]),
            GeckoCommands::Rwas {
                action: RwasCommands::Markets {
                    ids: Some(_),
                    per_page: Some(10),
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            parse(&["rwa", "get", "1-800-flowers-com", "--full"]),
            GeckoCommands::Rwas {
                action: RwasCommands::Get { full: true, .. },
                ..
            }
        ));
        assert!(matches!(
            parse(&["rwas", "-o", "ndjson", "issuers"]),
            GeckoCommands::Rwas {
                action: RwasCommands::Issuers,
                args: GeckoArgs {
                    format: OutputFormat::Ndjson
                }
            }
        ));
        assert!(matches!(
            parse(&["rwas", "issuer", "some-issuer"]),
            GeckoCommands::Rwas {
                action: RwasCommands::Issuer { .. },
                ..
            }
        ));
        assert!(matches!(
            parse(&["rwas", "tickers", "x", "--depth"]),
            GeckoCommands::Rwas {
                action: RwasCommands::Tickers { depth: true, .. },
                ..
            }
        ));
        assert!(matches!(
            parse(&["rwas", "chart", "x", "--days", "max"]),
            GeckoCommands::Rwas {
                action: RwasCommands::Chart { days, .. },
                ..
            } if days == "max"
        ));
    }

    #[test]
    fn wallets_subcommands_parse() {
        assert!(matches!(
            parse(&["wallets", "balances", "0xabc", "--networks", "eth, base"]),
            GeckoCommands::Wallets {
                action: WalletsCommands::Balances { networks, .. },
                ..
            } if split_list(&networks) == ["eth", "base"]
        ));
        assert!(
            fails(&["wallets", "balances", "0xabc"]),
            "--networks is required"
        );
        assert!(matches!(
            parse(&[
                "wallets",
                "transfers",
                "eth",
                "0xabc",
                "--from",
                "2026-11-01",
                "--to",
                "2026-11-07"
            ]),
            GeckoCommands::Wallets {
                action: WalletsCommands::Transfers {
                    from: Some(_),
                    to: Some(_),
                    ..
                },
                ..
            }
        ));
        assert!(fails(&[
            "wallets",
            "transfers",
            "eth",
            "0xabc",
            "--from",
            "2026-11-01"
        ]));
        assert!(matches!(
            parse(&["wallet", "trades", "eth", "0xabc", "--include-anomalous"]),
            GeckoCommands::Wallets {
                action: WalletsCommands::Trades {
                    include_anomalous: true,
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            parse(&["wallets", "pnl", "0xabc", "--networks", "eth"]),
            GeckoCommands::Wallets {
                action: WalletsCommands::Pnl { .. },
                ..
            }
        ));
    }

    #[test]
    fn news_insights_and_supply_breakdown_parse() {
        assert!(matches!(
            parse(&["news", "--coin", "bitcoin", "--type", "guides"]),
            GeckoCommands::News { coin: Some(_), news_type: Some(t), .. } if t == "guides"
        ));
        assert!(matches!(
            parse(&["insights", "--from", "2026-11-01"]),
            GeckoCommands::Insights {
                from: Some(_),
                to: None,
                ..
            }
        ));
        assert!(matches!(
            parse(&["coins", "supply-breakdown", "uniswap"]),
            GeckoCommands::Coins {
                action: CoinsCommands::SupplyBreakdown { .. },
                ..
            }
        ));
    }

    #[test]
    fn onchain_multi_and_trades_parse() {
        assert!(matches!(
            parse(&[
                "onchain",
                "token-price-multi",
                "eth:0xa,base:0xb",
                "--market-cap"
            ]),
            GeckoCommands::Onchain {
                action: OnchainCommands::TokenPriceMulti {
                    market_cap: true,
                    volume: false,
                    ..
                },
                ..
            }
        ));
        assert!(matches!(
            parse(&[
                "onchain",
                "tokens-multi",
                "eth:0xa",
                "--top-pools",
                "--composition"
            ]),
            GeckoCommands::Onchain {
                action: OnchainCommands::TokensMulti {
                    top_pools: true,
                    composition: true,
                    ..
                },
                ..
            }
        ));
        assert!(fails(&[
            "onchain",
            "tokens-multi",
            "eth:0xa",
            "--composition"
        ]));
        assert!(matches!(
            parse(&[
                "onchain",
                "pool-trades",
                "eth",
                "0xp",
                "--from",
                "2026-11-01",
                "--to",
                "2026-11-02"
            ]),
            GeckoCommands::Onchain {
                action: OnchainCommands::PoolTrades {
                    from: Some(_),
                    to: Some(_),
                    ..
                },
                ..
            }
        ));
        assert!(fails(&[
            "onchain",
            "pool-trades",
            "eth",
            "0xp",
            "--to",
            "b"
        ]));
        assert!(fails(&[
            "onchain",
            "pool-trades",
            "eth",
            "0xp",
            "--from",
            "a",
            "--to",
            "b",
            "--period",
            "7d"
        ]));
        assert!(matches!(
            parse(&["onchain", "token-trades", "eth", "0xt", "--period", "7d"]),
            GeckoCommands::Onchain {
                action: OnchainCommands::TokenTrades {
                    from: None,
                    period: Some(_),
                    ..
                },
                ..
            }
        ));
    }

    #[test]
    fn paid_endpoints_are_flagged() {
        assert_eq!(paid_plan_required(&parse(&["news"])), Some("Analyst"));
        assert_eq!(paid_plan_required(&parse(&["insights"])), Some("Analyst"));
        assert_eq!(
            paid_plan_required(&parse(&["wallets", "pnl", "0x", "--networks", "eth"])),
            Some("Analyst")
        );
        assert_eq!(
            paid_plan_required(&parse(&["coins", "supply-breakdown", "uniswap"])),
            Some("Analyst")
        );
        assert_eq!(
            paid_plan_required(&parse(&["onchain", "tokens-multi", "eth:0xa"])),
            Some("Enterprise")
        );
        assert_eq!(
            paid_plan_required(&parse(&["rwas", "chart", "x"])),
            Some("Basic")
        );
        assert_eq!(
            paid_plan_required(&parse(&[
                "onchain",
                "pool-trades",
                "eth",
                "0xp",
                "--from",
                "a",
                "--to",
                "b"
            ])),
            Some("Analyst")
        );
        // Endpoints on the keyless API stay available
        assert_eq!(paid_plan_required(&parse(&["rwas", "list"])), None);
        assert_eq!(paid_plan_required(&parse(&["rwas", "get", "x"])), None);
        assert_eq!(
            paid_plan_required(&parse(&["onchain", "pool-trades", "eth", "0xp"])),
            None
        );
        assert_eq!(
            paid_plan_required(&parse(&["coins", "get", "bitcoin"])),
            None
        );
    }
}
