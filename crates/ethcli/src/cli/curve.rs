//! Direct Curve Finance API commands
//!
//! Provides 1:1 access to Curve Finance API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct CurveArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum CurveCommands {
    /// Find swap routes between tokens (local router)
    Router {
        #[command(subcommand)]
        action: RouterCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// Pool operations (api.curve.finance)
    Pools {
        #[command(subcommand)]
        action: PoolsCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// Volume and APY data (api.curve.finance)
    Volumes {
        #[command(subcommand)]
        action: VolumesCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// Lending vaults (api.curve.finance)
    Lending {
        #[command(subcommand)]
        action: LendingCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// Token information (api.curve.finance)
    Tokens {
        #[command(subcommand)]
        action: TokensCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// crvUSD data
    Crvusd {
        #[command(subcommand)]
        action: CrvusdCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// Token prices (prices.curve.finance)
    Prices {
        #[command(subcommand)]
        action: PricesCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// OHLC data (prices.curve.finance)
    Ohlc {
        #[command(subcommand)]
        action: OhlcCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// Trade data (prices.curve.finance)
    Trades {
        #[command(subcommand)]
        action: TradesCommands,

        #[command(flatten)]
        args: CurveArgs,
    },

    /// DAO data - gauges, proposals, lockers (prices.curve.finance)
    Dao {
        #[command(subcommand)]
        action: DaoCommands,

        #[command(flatten)]
        args: CurveArgs,
    },
}

#[derive(Subcommand)]
pub enum RouterCommands {
    /// Find routes between two tokens
    Route {
        /// Input token address
        from: String,
        /// Output token address
        to: String,
        /// Chain name (default: ethereum)
        #[arg(long, short, default_value = "ethereum")]
        chain: String,
        /// Maximum number of routes to return
        #[arg(long, default_value = "5")]
        limit: usize,
    },

    /// Get calldata for a swap
    Encode {
        /// Input token address
        from: String,
        /// Output token address
        to: String,
        /// Input amount (raw, no decimals)
        amount: String,
        /// Minimum output amount (raw, no decimals)
        min_out: String,
        /// Chain name (default: ethereum)
        #[arg(long, short, default_value = "ethereum")]
        chain: String,
    },

    /// Show router graph statistics
    Stats {
        /// Chain name (default: ethereum)
        #[arg(long, short, default_value = "ethereum")]
        chain: String,
    },

    /// Get router contract address for a chain
    Address {
        /// Chain name (default: ethereum)
        #[arg(default_value = "ethereum")]
        chain: String,
    },
}

#[derive(Subcommand)]
pub enum PoolsCommands {
    /// Get all pools on a chain
    List {
        /// Chain name (e.g., "ethereum", "polygon", "arbitrum")
        chain: String,
    },

    /// Get pools from a specific registry
    Registry {
        /// Chain name
        chain: String,
        /// Registry ID (e.g., "main", "factory", "factory-crypto")
        registry: String,
    },

    /// Get all pools across all chains
    All,

    /// Get pools with TVL >= $10k
    Big {
        /// Chain name (optional, all chains if omitted)
        chain: Option<String>,
    },

    /// Get pools with TVL < $10k
    Small {
        /// Chain name (optional, all chains if omitted)
        chain: Option<String>,
    },

    /// Get pools with $0 TVL
    Empty {
        /// Chain name (optional, all chains if omitted)
        chain: Option<String>,
    },

    /// Get pool addresses on a chain
    Addresses {
        /// Chain name
        chain: String,
    },

    /// Get hidden/dysfunctional pools
    Hidden,
}

#[derive(Subcommand)]
pub enum VolumesCommands {
    /// Get all gauges
    Gauges,

    /// Get total 24h volume for a chain
    Total {
        /// Chain name
        chain: String,
    },

    /// Get base APYs for pools on a chain
    Apys {
        /// Chain name
        chain: String,
    },

    /// Get volumes for pools on a chain
    Pools {
        /// Chain name
        chain: String,
    },

    /// Get crvUSD AMM volumes
    Crvusd,
}

#[derive(Subcommand)]
pub enum LendingCommands {
    /// Get all lending vaults
    All,

    /// Get lending vaults on a chain
    Chain {
        /// Chain name
        chain: String,
    },

    /// Get lending vaults from a specific registry
    Registry {
        /// Chain name
        chain: String,
        /// Registry ID
        registry: String,
    },
}

#[derive(Subcommand)]
pub enum TokensCommands {
    /// Get all tokens on a chain (in pools with $10k+ TVL)
    List {
        /// Chain name (e.g., "ethereum", "polygon", "arbitrum")
        chain: String,
    },
}

#[derive(Subcommand)]
pub enum CrvusdCommands {
    /// Get crvUSD total supply
    TotalSupply,

    /// Get crvUSD circulating supply
    CirculatingSupply,

    /// Get scrvUSD total supply
    ScrvusdSupply,

    /// Get crvUSD markets (from prices API)
    Markets {
        /// Chain name (optional, all chains if omitted)
        chain: Option<String>,
    },

    /// Get crvUSD savings stats
    Savings,
}

#[derive(Subcommand)]
pub enum PricesCommands {
    /// Get supported chains
    Chains,

    /// Get chain stats
    ChainStats,

    /// Get USD prices for all tokens on a chain
    All {
        /// Chain name (e.g., "ethereum", "arbitrum")
        chain: String,
    },

    /// Get USD price for a specific token
    Token {
        /// Chain name
        chain: String,
        /// Token contract address
        address: String,
    },

    /// Get price history for a token
    History {
        /// Chain name
        chain: String,
        /// Token contract address
        address: String,
    },

    /// Get top tokens by volume
    TopVolume,
}

#[derive(Subcommand)]
pub enum OhlcCommands {
    /// Get OHLC data for a pool
    Pool {
        /// Chain name
        chain: String,
        /// Pool contract address
        address: String,
    },

    /// Get LP token OHLC data
    LpToken {
        /// Chain name
        chain: String,
        /// LP token contract address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum TradesCommands {
    /// Get trades for a contract
    Get {
        /// Chain name
        chain: String,
        /// Contract address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum DaoCommands {
    /// Get gauge overview
    Gauges,

    /// Get DAO proposals
    Proposals,

    /// Get top CRV lockers
    Lockers {
        /// Number of top lockers to return
        #[arg(default_value = "100")]
        top: u32,
    },
}

/// Handle Curve Finance commands
pub async fn handle(command: &CurveCommands, quiet: bool) -> anyhow::Result<()> {
    let client = crv::Client::new()?;
    let prices_client = crv::PricesClient::new()?;

    match command {
        CurveCommands::Router { action, args } => handle_router(&client, action, args, quiet).await,
        CurveCommands::Pools { action, args } => handle_pools(&client, action, args, quiet).await,
        CurveCommands::Volumes { action, args } => {
            handle_volumes(&client, action, args, quiet).await
        }
        CurveCommands::Lending { action, args } => {
            handle_lending(&client, action, args, quiet).await
        }
        CurveCommands::Tokens { action, args } => handle_tokens(&client, action, args, quiet).await,
        CurveCommands::Crvusd { action, args } => {
            handle_crvusd(&client, &prices_client, action, args, quiet).await
        }
        CurveCommands::Prices { action, args } => {
            handle_prices(&prices_client, action, args, quiet).await
        }
        CurveCommands::Ohlc { action, args } => {
            handle_ohlc(&prices_client, action, args, quiet).await
        }
        CurveCommands::Trades { action, args } => {
            handle_trades(&prices_client, action, args, quiet).await
        }
        CurveCommands::Dao { action, args } => {
            handle_dao(&prices_client, action, args, quiet).await
        }
    }
}

async fn handle_router(
    client: &crv::Client,
    action: &RouterCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        RouterCommands::Route {
            from,
            to,
            chain,
            limit,
        } => {
            if !quiet {
                eprintln!("Building router graph for {}...", chain);
            }
            let router = client.build_router(chain).await?;

            if !quiet {
                eprintln!(
                    "Graph: {} tokens, {} edges",
                    router.stats().token_count,
                    router.stats().edge_count
                );
                eprintln!("Finding routes from {} to {}...", from, to);
            }

            let routes = router.find_routes(from, to);
            let routes: Vec<_> = routes.into_iter().take(*limit).collect();

            if routes.is_empty() {
                if !quiet {
                    eprintln!("No routes found");
                }
                return Ok(());
            }

            #[derive(serde::Serialize)]
            struct RouteOutput {
                hops: usize,
                min_tvl_usd: f64,
                total_tvl_usd: f64,
                steps: Vec<StepOutput>,
            }

            #[derive(serde::Serialize)]
            struct StepOutput {
                pool_id: String,
                pool_address: String,
                input: String,
                output: String,
                swap_type: u8,
                tvl_usd: f64,
            }

            let output: Vec<RouteOutput> = routes
                .iter()
                .map(|r| RouteOutput {
                    hops: r.steps.len(),
                    min_tvl_usd: r.min_tvl,
                    total_tvl_usd: r.total_tvl,
                    steps: r
                        .steps
                        .iter()
                        .map(|s| StepOutput {
                            pool_id: s.pool_id.clone(),
                            pool_address: s.pool_address.clone(),
                            input: s.input_coin.clone(),
                            output: s.output_coin.clone(),
                            swap_type: s.swap_params.swap_type.as_u8(),
                            tvl_usd: s.tvl_usd,
                        })
                        .collect(),
                })
                .collect();

            print_output(&output, args.format)?;
        }

        RouterCommands::Encode {
            from,
            to,
            amount,
            min_out,
            chain,
        } => {
            if !quiet {
                eprintln!("Building router graph for {}...", chain);
            }
            let router = client.build_router(chain).await?;

            let best = router
                .find_best_route(from, to)
                .ok_or_else(|| anyhow::anyhow!("No route found from {} to {}", from, to))?;

            if !quiet {
                eprintln!("Best route: {} hops", best.steps.len());
            }

            let calldata = router.encode_swap(&best, amount, min_out)?;

            #[derive(serde::Serialize)]
            struct EncodeOutput {
                router_address: String,
                calldata: String,
                calldata_length: usize,
                route_hops: usize,
            }

            let output = EncodeOutput {
                router_address: router.router_address().unwrap_or("unknown").to_string(),
                calldata: format!("0x{}", hex_encode(&calldata)),
                calldata_length: calldata.len(),
                route_hops: best.steps.len(),
            };

            print_output(&output, args.format)?;
        }

        RouterCommands::Stats { chain } => {
            if !quiet {
                eprintln!("Building router graph for {}...", chain);
            }
            let router = client.build_router(chain).await?;
            let stats = router.stats();

            #[derive(serde::Serialize)]
            struct StatsOutput {
                chain: String,
                token_count: usize,
                edge_count: usize,
                router_address: Option<String>,
            }

            let output = StatsOutput {
                chain: stats.chain,
                token_count: stats.token_count,
                edge_count: stats.edge_count,
                router_address: router.router_address().map(String::from),
            };

            print_output(&output, args.format)?;
        }

        RouterCommands::Address { chain } => {
            let addr = crv::router::router_address(chain);

            #[derive(serde::Serialize)]
            struct AddressOutput {
                chain: String,
                router_address: Option<String>,
            }

            let output = AddressOutput {
                chain: chain.clone(),
                router_address: addr.map(String::from),
            };

            print_output(&output, args.format)?;
        }
    }
    Ok(())
}

/// Encode bytes as hex string
fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

async fn handle_pools(
    client: &crv::Client,
    action: &PoolsCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        PoolsCommands::List { chain } => {
            if !quiet {
                eprintln!("Fetching pools on {}...", chain);
            }
            let response = client.pools().get_all_on_chain(chain).await?;
            print_output(&response, args.format)?;
        }
        PoolsCommands::Registry { chain, registry } => {
            if !quiet {
                eprintln!("Fetching pools from {} on {}...", registry, chain);
            }
            let response = client.pools().get(chain, registry).await?;
            print_output(&response, args.format)?;
        }
        PoolsCommands::All => {
            if !quiet {
                eprintln!("Fetching all pools...");
            }
            let response = client.pools().get_all().await?;
            print_output(&response, args.format)?;
        }
        PoolsCommands::Big { chain } => {
            if let Some(chain) = chain {
                if !quiet {
                    eprintln!("Fetching big pools on {}...", chain);
                }
                let response = client.pools().get_big(chain).await?;
                print_output(&response, args.format)?;
            } else {
                if !quiet {
                    eprintln!("Fetching all big pools...");
                }
                let response = client.pools().get_all_big().await?;
                print_output(&response, args.format)?;
            }
        }
        PoolsCommands::Small { chain } => {
            if let Some(chain) = chain {
                if !quiet {
                    eprintln!("Fetching small pools on {}...", chain);
                }
                let response = client.pools().get_small(chain).await?;
                print_output(&response, args.format)?;
            } else {
                if !quiet {
                    eprintln!("Fetching all small pools...");
                }
                let response = client.pools().get_all_small().await?;
                print_output(&response, args.format)?;
            }
        }
        PoolsCommands::Empty { chain } => {
            if let Some(chain) = chain {
                if !quiet {
                    eprintln!("Fetching empty pools on {}...", chain);
                }
                let response = client.pools().get_empty(chain).await?;
                print_output(&response, args.format)?;
            } else {
                if !quiet {
                    eprintln!("Fetching all empty pools...");
                }
                let response = client.pools().get_all_empty().await?;
                print_output(&response, args.format)?;
            }
        }
        PoolsCommands::Addresses { chain } => {
            if !quiet {
                eprintln!("Fetching pool addresses on {}...", chain);
            }
            let response = client.pools().list(chain).await?;
            print_output(&response, args.format)?;
        }
        PoolsCommands::Hidden => {
            if !quiet {
                eprintln!("Fetching hidden pools...");
            }
            let response = client.pools().get_hidden().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_volumes(
    client: &crv::Client,
    action: &VolumesCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        VolumesCommands::Gauges => {
            if !quiet {
                eprintln!("Fetching all gauges...");
            }
            let response = client.volumes().get_all_gauges().await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::Total { chain } => {
            if !quiet {
                eprintln!("Fetching total volume on {}...", chain);
            }
            let response = client.volumes().get_total_volume(chain).await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::Apys { chain } => {
            if !quiet {
                eprintln!("Fetching base APYs on {}...", chain);
            }
            let response = client.volumes().get_base_apys(chain).await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::Pools { chain } => {
            if !quiet {
                eprintln!("Fetching volumes on {}...", chain);
            }
            let response = client.volumes().get_volumes(chain).await?;
            print_output(&response, args.format)?;
        }
        VolumesCommands::Crvusd => {
            if !quiet {
                eprintln!("Fetching crvUSD AMM volumes...");
            }
            let response = client.volumes().get_crvusd_amm_volumes().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_lending(
    client: &crv::Client,
    action: &LendingCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        LendingCommands::All => {
            if !quiet {
                eprintln!("Fetching all lending vaults...");
            }
            let response = client.lending().get_all().await?;
            print_output(&response, args.format)?;
        }
        LendingCommands::Chain { chain } => {
            if !quiet {
                eprintln!("Fetching lending vaults on {}...", chain);
            }
            let response = client.lending().get_all_on_chain(chain).await?;
            print_output(&response, args.format)?;
        }
        LendingCommands::Registry { chain, registry } => {
            if !quiet {
                eprintln!("Fetching lending vaults from {} on {}...", registry, chain);
            }
            let response = client.lending().get(chain, registry).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_tokens(
    client: &crv::Client,
    action: &TokensCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TokensCommands::List { chain } => {
            if !quiet {
                eprintln!("Fetching tokens on {}...", chain);
            }
            let response = client.tokens().get_all(chain).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_crvusd(
    client: &crv::Client,
    prices_client: &crv::PricesClient,
    action: &CrvusdCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        CrvusdCommands::TotalSupply => {
            if !quiet {
                eprintln!("Fetching crvUSD total supply...");
            }
            let response = client.crvusd().get_total_supply().await?;
            print_output(&response, args.format)?;
        }
        CrvusdCommands::CirculatingSupply => {
            if !quiet {
                eprintln!("Fetching crvUSD circulating supply...");
            }
            let response = client.crvusd().get_circulating_supply().await?;
            print_output(&response, args.format)?;
        }
        CrvusdCommands::ScrvusdSupply => {
            if !quiet {
                eprintln!("Fetching scrvUSD supply...");
            }
            let response = client.crvusd().get_scrvusd_supply().await?;
            print_output(&response, args.format)?;
        }
        CrvusdCommands::Markets { chain } => {
            if let Some(chain) = chain {
                if !quiet {
                    eprintln!("Fetching crvUSD markets on {}...", chain);
                }
                let response = prices_client.get_crvusd_markets_on_chain(chain).await?;
                print_output(&response, args.format)?;
            } else {
                if !quiet {
                    eprintln!("Fetching all crvUSD markets...");
                }
                let response = prices_client.get_crvusd_markets().await?;
                print_output(&response, args.format)?;
            }
        }
        CrvusdCommands::Savings => {
            if !quiet {
                eprintln!("Fetching crvUSD savings stats...");
            }
            let response = prices_client.get_crvusd_savings_stats().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_prices(
    client: &crv::PricesClient,
    action: &PricesCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        PricesCommands::Chains => {
            if !quiet {
                eprintln!("Fetching supported chains...");
            }
            let response = client.get_chains().await?;
            print_output(&response, args.format)?;
        }
        PricesCommands::ChainStats => {
            if !quiet {
                eprintln!("Fetching chain stats...");
            }
            let response = client.get_chain_stats().await?;
            print_output(&response, args.format)?;
        }
        PricesCommands::All { chain } => {
            if !quiet {
                eprintln!("Fetching USD prices on {}...", chain);
            }
            let response = client.get_usd_prices(chain).await?;
            print_output(&response, args.format)?;
        }
        PricesCommands::Token { chain, address } => {
            if !quiet {
                eprintln!("Fetching USD price for {} on {}...", address, chain);
            }
            let response = client.get_usd_price(chain, address).await?;
            print_output(&response, args.format)?;
        }
        PricesCommands::History { chain, address } => {
            if !quiet {
                eprintln!("Fetching price history for {} on {}...", address, chain);
            }
            let response = client.get_price_history(chain, address).await?;
            print_output(&response, args.format)?;
        }
        PricesCommands::TopVolume => {
            if !quiet {
                eprintln!("Fetching top tokens by volume...");
            }
            let response = client.get_top_volume_tokens().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_ohlc(
    client: &crv::PricesClient,
    action: &OhlcCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        OhlcCommands::Pool { chain, address } => {
            if !quiet {
                eprintln!("Fetching OHLC for pool {} on {}...", address, chain);
            }
            let response = client.get_ohlc(chain, address).await?;
            print_output(&response, args.format)?;
        }
        OhlcCommands::LpToken { chain, address } => {
            if !quiet {
                eprintln!("Fetching LP OHLC for {} on {}...", address, chain);
            }
            let response = client.get_lp_ohlc(chain, address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_trades(
    client: &crv::PricesClient,
    action: &TradesCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TradesCommands::Get { chain, address } => {
            if !quiet {
                eprintln!("Fetching trades for {} on {}...", address, chain);
            }
            let response = client.get_trades(chain, address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_dao(
    client: &crv::PricesClient,
    action: &DaoCommands,
    args: &CurveArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        DaoCommands::Gauges => {
            if !quiet {
                eprintln!("Fetching gauge overview...");
            }
            let response = client.get_gauges_overview().await?;
            print_output(&response, args.format)?;
        }
        DaoCommands::Proposals => {
            if !quiet {
                eprintln!("Fetching DAO proposals...");
            }
            let response = client.get_proposals().await?;
            print_output(&response, args.format)?;
        }
        DaoCommands::Lockers { top } => {
            if !quiet {
                eprintln!("Fetching top {} CRV lockers...", top);
            }
            let response = client.get_top_lockers(*top).await?;
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
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}
