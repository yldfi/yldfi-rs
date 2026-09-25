//! Uniswap CLI commands
//!
//! Query Uniswap V2, V3, and V4 pools and data via on-chain lens
//! queries and The Graph subgraph.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use secrecy::ExposeSecret;

/// Uniswap protocol version
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum Version {
    /// Uniswap V2
    V2,
    /// Uniswap V3 (default)
    #[default]
    V3,
    /// Uniswap V4
    V4,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::V2 => write!(f, "v2"),
            Version::V3 => write!(f, "v3"),
            Version::V4 => write!(f, "v4"),
        }
    }
}

/// Uniswap commands
#[derive(Subcommand, Debug)]
pub enum UniswapCommands {
    /// Get pool state (on-chain, no API key needed)
    #[command(visible_alias = "state")]
    Pool(PoolArgs),

    /// Get pool liquidity (on-chain)
    Liquidity(LiquidityArgs),

    /// Get current ETH price (subgraph, requires API key)
    EthPrice(EthPriceArgs),

    /// Get top pools by TVL (subgraph, requires API key)
    TopPools(TopPoolsArgs),

    /// Get recent swaps for a pool (subgraph, requires API key)
    Swaps(SwapsArgs),

    /// Get daily data for a pool (subgraph, requires API key)
    DayData(DayDataArgs),

    /// Get LP positions for a wallet (subgraph, requires API key)
    Positions(PositionsArgs),

    /// Get token balance for an account
    Balance(BalanceArgs),

    /// List well-known factory and pool addresses
    Addresses(AddressesArgs),
}

/// Arguments for pool state query
#[derive(Args, Debug)]
pub struct PoolArgs {
    /// Pool address
    pub pool: String,

    /// RPC URL (defaults to public endpoint)
    #[arg(long, env = "ETH_RPC_URL", hide_env_values = true)]
    pub rpc_url: Option<String>,
}

/// Arguments for liquidity query
#[derive(Args, Debug)]
pub struct LiquidityArgs {
    /// Pool address
    pub pool: String,

    /// RPC URL
    #[arg(long, env = "ETH_RPC_URL", hide_env_values = true)]
    pub rpc_url: Option<String>,
}

/// Arguments for ETH price query
#[derive(Args, Debug)]
pub struct EthPriceArgs {
    /// The Graph API key (or set THEGRAPH_API_KEY env var, or add [thegraph] to config)
    #[arg(long, env = "THEGRAPH_API_KEY", hide_env_values = true)]
    pub api_key: Option<String>,

    /// Uniswap version
    #[arg(long, value_enum, default_value = "v3")]
    pub version: Version,
}

/// Arguments for top pools query
#[derive(Args, Debug)]
pub struct TopPoolsArgs {
    /// Number of pools to fetch
    #[arg(default_value = "10")]
    pub limit: u32,

    /// The Graph API key (or set THEGRAPH_API_KEY env var, or add [thegraph] to config)
    #[arg(long, env = "THEGRAPH_API_KEY", hide_env_values = true)]
    pub api_key: Option<String>,

    /// Uniswap version
    #[arg(long, value_enum, default_value = "v3")]
    pub version: Version,
}

/// Arguments for swaps query
#[derive(Args, Debug)]
pub struct SwapsArgs {
    /// Pool address
    pub pool: String,

    /// Number of swaps to fetch
    #[arg(long, default_value = "10")]
    pub limit: u32,

    /// The Graph API key (or set THEGRAPH_API_KEY env var, or add [thegraph] to config)
    #[arg(long, env = "THEGRAPH_API_KEY", hide_env_values = true)]
    pub api_key: Option<String>,

    /// Uniswap version
    #[arg(long, value_enum, default_value = "v3")]
    pub version: Version,
}

/// Arguments for day data query
#[derive(Args, Debug)]
pub struct DayDataArgs {
    /// Pool address
    pub pool: String,

    /// Number of days to fetch
    #[arg(long, default_value = "7")]
    pub days: u32,

    /// The Graph API key (or set THEGRAPH_API_KEY env var, or add [thegraph] to config)
    #[arg(long, env = "THEGRAPH_API_KEY", hide_env_values = true)]
    pub api_key: Option<String>,

    /// Uniswap version
    #[arg(long, value_enum, default_value = "v3")]
    pub version: Version,
}

/// Arguments for LP positions query
#[derive(Args, Debug)]
pub struct PositionsArgs {
    /// Wallet address (or label from address book)
    pub address: String,

    /// The Graph API key
    #[arg(long, env = "THEGRAPH_API_KEY", hide_env_values = true)]
    pub api_key: Option<String>,

    /// Uniswap version (omit to query all versions)
    #[arg(long, value_enum)]
    pub version: Option<Version>,

    /// Chain to query
    #[arg(long, short, default_value = "ethereum")]
    pub chain: String,

    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
    pub format: OutputFormat,
}

/// Arguments for balance query
#[derive(Args, Debug)]
pub struct BalanceArgs {
    /// Token address
    pub token: String,

    /// Account address
    pub account: String,

    /// RPC URL
    #[arg(long, env = "ETH_RPC_URL", hide_env_values = true)]
    pub rpc_url: Option<String>,
}

/// Arguments for listing addresses
#[derive(Args, Debug)]
pub struct AddressesArgs {
    /// Show only factory addresses
    #[arg(long)]
    pub factories: bool,

    /// Show only pool addresses
    #[arg(long)]
    pub pools: bool,

    /// Show only token addresses
    #[arg(long)]
    pub tokens: bool,

    /// Uniswap version filter
    #[arg(long, value_enum)]
    pub version: Option<Version>,
}

/// Fallback public RPC URL for Ethereum mainnet, used only when neither
/// `--rpc-url`/`ETH_RPC_URL` nor a configured Ethereum endpoint is available.
const FALLBACK_RPC_URL: &str = "https://ethereum-rpc.publicnode.com";

/// Resolve TheGraph API key from args, config, or env
fn resolve_api_key(arg_key: &Option<String>) -> anyhow::Result<String> {
    // 1. Check arg
    if let Some(key) = arg_key {
        if !key.is_empty() {
            return Ok(key.clone());
        }
    }

    // 2. Check config
    if let Some(config) = crate::aggregator::get_cached_config() {
        if let Some(thegraph) = &config.thegraph {
            return Ok(thegraph.api_key.expose_secret().to_string());
        }
    }

    // 3. Check env (fallback)
    if let Ok(key) = std::env::var("THEGRAPH_API_KEY") {
        if !key.is_empty() {
            return Ok(key);
        }
    }

    Err(anyhow::anyhow!(
        "TheGraph API key required. Set THEGRAPH_API_KEY env var, use --api-key, or add [thegraph] to config"
    ))
}

/// Resolve the RPC URL and V3 factory for on-chain lens commands on `chain`.
///
/// `--rpc-url` wins; otherwise the configured endpoint for the chain is used
/// (falling back to a public endpoint on mainnet). An `ETH_RPC_URL` env
/// value is only honoured for mainnet so that `--chain arbitrum` never
/// silently queries an Ethereum node.
fn resolve_lens_target(
    rpc_arg: Option<&str>,
    chain: &str,
) -> anyhow::Result<(String, alloy::primitives::Address, crate::config::Chain)> {
    use unswp::factories::v3;
    let chain = crate::config::Chain::from_str_or_id(chain)
        .map_err(|e| anyhow::anyhow!("Invalid --chain '{chain}': {e}"))?;
    let chain_id = chain.chain_id();
    let factory = match chain_id {
        1 => v3::MAINNET,
        10 => v3::OPTIMISM,
        137 => v3::POLYGON,
        8453 => v3::BASE,
        42161 => v3::ARBITRUM,
        _ => anyhow::bail!(
            "Uniswap lens queries support ethereum, optimism, polygon, base and arbitrum (got chain {chain_id})"
        ),
    };
    let env_url = std::env::var("ETH_RPC_URL").ok();
    let explicit = rpc_arg.filter(|u| chain_id == 1 || env_url.as_deref() != Some(*u));
    let url = match explicit {
        Some(u) => u.to_string(),
        None => match crate::rpc::selector::get_rpc_url(chain) {
            Ok(u) => u,
            Err(_) if chain_id == 1 => FALLBACK_RPC_URL.to_string(),
            Err(e) => anyhow::bail!(
                "No RPC endpoint configured for {chain}: {e}. Pass --rpc-url or add one with `ethcli endpoints add`"
            ),
        },
    };
    Ok((url, factory, chain))
}

/// Handle Uniswap CLI commands
///
/// `chain` is the global `--chain`, used by the on-chain lens commands
/// (`pool`, `liquidity`, `balance`).
pub async fn handle(action: &UniswapCommands, chain: &str, quiet: bool) -> anyhow::Result<()> {
    use alloy::primitives::Address;
    use unswp::{
        factories, pools, subgraph_ids, tokens, LensClient, SubgraphClient, SubgraphConfig,
    };

    match action {
        UniswapCommands::Pool(args) => {
            let (rpc_url, factory, lens_chain) =
                resolve_lens_target(args.rpc_url.as_deref(), chain)?;
            let pool: Address = args.pool.parse()?;

            if !quiet {
                eprintln!("Fetching pool state on {}...", lens_chain);
            }

            let client = LensClient::new(&rpc_url, factory)?;
            let state = client.get_pool_state(pool).await?;

            let output = serde_json::json!({
                "pool": format!("{:#x}", pool),
                "sqrtPriceX96": state.sqrt_price_x96.to_string(),
                "tick": state.tick,
                "observationIndex": state.observation_index,
                "observationCardinality": state.observation_cardinality,
                "observationCardinalityNext": state.observation_cardinality_next,
                "feeProtocol": state.fee_protocol,
                "unlocked": state.unlocked,
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        UniswapCommands::Liquidity(args) => {
            let (rpc_url, factory, lens_chain) =
                resolve_lens_target(args.rpc_url.as_deref(), chain)?;
            let pool: Address = args.pool.parse()?;

            if !quiet {
                eprintln!("Fetching liquidity on {}...", lens_chain);
            }

            let client = LensClient::new(&rpc_url, factory)?;
            let liquidity = client.get_liquidity(pool).await?;

            let output = serde_json::json!({
                "pool": format!("{:#x}", pool),
                "liquidity": liquidity.to_string(),
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        UniswapCommands::EthPrice(args) => {
            let api_key = resolve_api_key(&args.api_key)?;

            if !quiet {
                eprintln!("Fetching ETH price from subgraph ({})...", args.version);
            }

            let config = match args.version {
                Version::V2 => SubgraphConfig::mainnet_v2(&api_key),
                Version::V3 => SubgraphConfig::mainnet_v3(&api_key),
                Version::V4 => SubgraphConfig::mainnet_v4(&api_key),
            };

            let client = SubgraphClient::new(config)?;
            let price = client.get_eth_price().await?;

            let output = serde_json::json!({
                "ethPriceUSD": price,
                "version": args.version.to_string(),
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        UniswapCommands::TopPools(args) => {
            let api_key = resolve_api_key(&args.api_key)?;

            if !quiet {
                eprintln!(
                    "Fetching top {} pools from subgraph ({})...",
                    args.limit, args.version
                );
            }

            let config = match args.version {
                Version::V2 => SubgraphConfig::mainnet_v2(&api_key),
                Version::V3 => SubgraphConfig::mainnet_v3(&api_key),
                Version::V4 => SubgraphConfig::mainnet_v4(&api_key),
            };

            let client = SubgraphClient::new(config)?;

            // Use the correct query method for each version
            match args.version {
                Version::V2 => {
                    let pairs = client.get_top_pairs(args.limit).await?;
                    println!("{}", serde_json::to_string_pretty(&pairs)?);
                }
                Version::V3 => {
                    let pools = client.get_top_pools(args.limit).await?;
                    println!("{}", serde_json::to_string_pretty(&pools)?);
                }
                Version::V4 => {
                    let pools = client.get_top_pools_v4(args.limit).await?;
                    println!("{}", serde_json::to_string_pretty(&pools)?);
                }
            }
        }

        UniswapCommands::Swaps(args) => {
            let api_key = resolve_api_key(&args.api_key)?;

            if !quiet {
                eprintln!(
                    "Fetching {} recent swaps from subgraph ({})...",
                    args.limit, args.version
                );
            }

            let config = match args.version {
                Version::V2 => SubgraphConfig::mainnet_v2(&api_key),
                Version::V3 => SubgraphConfig::mainnet_v3(&api_key),
                Version::V4 => SubgraphConfig::mainnet_v4(&api_key),
            };

            let client = SubgraphClient::new(config)?;
            let swaps = client.get_swaps(&args.pool, args.limit).await?;

            println!("{}", serde_json::to_string_pretty(&swaps)?);
        }

        UniswapCommands::DayData(args) => {
            let api_key = resolve_api_key(&args.api_key)?;

            if !quiet {
                eprintln!(
                    "Fetching {} days of data from subgraph ({})...",
                    args.days, args.version
                );
            }

            let config = match args.version {
                Version::V2 => SubgraphConfig::mainnet_v2(&api_key),
                Version::V3 => SubgraphConfig::mainnet_v3(&api_key),
                Version::V4 => SubgraphConfig::mainnet_v4(&api_key),
            };

            let client = SubgraphClient::new(config)?;
            let data = client.get_pool_day_data(&args.pool, args.days).await?;

            println!("{}", serde_json::to_string_pretty(&data)?);
        }

        UniswapCommands::Positions(args) => {
            // Resolve address from address book
            let address = crate::utils::address::resolve_label(&args.address);

            // Get API key from args, config, or env
            let api_key = args
                .api_key
                .clone()
                .or_else(|| {
                    crate::aggregator::get_cached_config()
                        .as_ref()
                        .and_then(|c| c.thegraph.as_ref())
                        .map(|g| g.api_key.expose_secret().to_string())
                })
                .or_else(|| std::env::var("THEGRAPH_API_KEY").ok());

            let api_key = api_key.ok_or_else(|| {
                anyhow::anyhow!("TheGraph API key required. Set THEGRAPH_API_KEY or use --api-key")
            })?;

            if !quiet {
                let version_str = args
                    .version
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "all".to_string());
                eprintln!(
                    "Fetching {} LP positions for {} on {}...",
                    version_str, address, args.chain
                );
            }

            let chain = args.chain.to_lowercase();

            // Collect all positions
            let mut all_positions: Vec<serde_json::Value> = Vec::new();

            // Query V2 (mainnet only)
            if (args.version.is_none() || matches!(args.version, Some(Version::V2)))
                && (chain == "ethereum" || chain == "mainnet" || chain == "eth")
            {
                let config = SubgraphConfig::mainnet_v2(&api_key);
                if let Ok(client) = SubgraphClient::new(config) {
                    match client.get_positions_v2(&address).await {
                        Ok(positions) => {
                            for pos in positions {
                                let pair = &pos.pair;
                                let lp_balance: f64 =
                                    pos.liquidity_token_balance.parse().unwrap_or(0.0);
                                let total_supply: f64 = pair.total_supply.parse().unwrap_or(1.0);
                                let reserve0: f64 = pair.reserve0.parse().unwrap_or(0.0);
                                let reserve1: f64 = pair.reserve1.parse().unwrap_or(0.0);
                                let reserve_usd: f64 = pair
                                    .reserve_usd
                                    .as_ref()
                                    .and_then(|s| s.parse().ok())
                                    .unwrap_or(0.0);

                                // Calculate share of pool
                                let share = if total_supply > 0.0 {
                                    lp_balance / total_supply
                                } else {
                                    0.0
                                };
                                let token0_amount = reserve0 * share;
                                let token1_amount = reserve1 * share;
                                let usd_value = reserve_usd * share;

                                all_positions.push(serde_json::json!({
                                    "version": "v2",
                                    "positionId": pos.id,
                                    "pool": pair.id,
                                    "token0": {
                                        "symbol": pair.token0.symbol,
                                        "address": pair.token0.id,
                                        "amount": token0_amount,
                                    },
                                    "token1": {
                                        "symbol": pair.token1.symbol,
                                        "address": pair.token1.id,
                                        "amount": token1_amount,
                                    },
                                    "lpTokenBalance": lp_balance,
                                    "poolShare": format!("{:.4}%", share * 100.0),
                                    "usdValue": usd_value,
                                }));
                            }
                        }
                        Err(e) => {
                            if !quiet {
                                eprintln!("V2 query failed: {}", e);
                            }
                        }
                    }
                }
            }

            // Query V3
            if args.version.is_none() || matches!(args.version, Some(Version::V3)) {
                let config = match chain.as_str() {
                    "ethereum" | "mainnet" | "eth" => SubgraphConfig::mainnet_v3(&api_key),
                    "arbitrum" | "arb" => SubgraphConfig::arbitrum_v3(&api_key),
                    "optimism" | "op" => SubgraphConfig::optimism_v3(&api_key),
                    "base" => SubgraphConfig::base_v3(&api_key),
                    "polygon" | "matic" => SubgraphConfig::mainnet_v3(&api_key)
                        .with_subgraph_id(subgraph_ids::POLYGON_V3),
                    _ => SubgraphConfig::mainnet_v3(&api_key),
                };

                if let Ok(client) = SubgraphClient::new(config) {
                    match client.get_positions(&address).await {
                        Ok(positions) => {
                            for pos in positions {
                                let pool = &pos.pool;
                                let liquidity: f64 = pos.liquidity.parse().unwrap_or(0.0);
                                let tick_lower: i32 = pos.tick_lower.tick_idx.parse().unwrap_or(0);
                                let tick_upper: i32 = pos.tick_upper.tick_idx.parse().unwrap_or(0);
                                let current_tick: i32 =
                                    pool.tick.as_ref().and_then(|t| t.parse().ok()).unwrap_or(0);

                                let in_range =
                                    current_tick >= tick_lower && current_tick <= tick_upper;
                                let fee_tier: f64 = pool.fee_tier.parse().unwrap_or(0.0) / 10000.0;

                                let deposited0: f64 = pos.deposited_token0.parse().unwrap_or(0.0);
                                let deposited1: f64 = pos.deposited_token1.parse().unwrap_or(0.0);
                                let withdrawn0: f64 = pos.withdrawn_token0.parse().unwrap_or(0.0);
                                let withdrawn1: f64 = pos.withdrawn_token1.parse().unwrap_or(0.0);
                                let fees0: f64 = pos.collected_fees_token0.parse().unwrap_or(0.0);
                                let fees1: f64 = pos.collected_fees_token1.parse().unwrap_or(0.0);

                                all_positions.push(serde_json::json!({
                                    "version": "v3",
                                    "positionId": pos.id,
                                    "pool": pool.id,
                                    "feeTier": format!("{}%", fee_tier),
                                    "token0": {
                                        "symbol": pool.token0.symbol,
                                        "address": pool.token0.id,
                                        "deposited": deposited0,
                                        "withdrawn": withdrawn0,
                                        "collectedFees": fees0,
                                    },
                                    "token1": {
                                        "symbol": pool.token1.symbol,
                                        "address": pool.token1.id,
                                        "deposited": deposited1,
                                        "withdrawn": withdrawn1,
                                        "collectedFees": fees1,
                                    },
                                    "liquidity": liquidity,
                                    "tickRange": {
                                        "lower": tick_lower,
                                        "upper": tick_upper,
                                        "current": current_tick,
                                    },
                                    "inRange": in_range,
                                }));
                            }
                        }
                        Err(e) => {
                            if !quiet {
                                eprintln!("V3 query failed: {}", e);
                            }
                        }
                    }
                }
            }

            // Query V4
            if args.version.is_none() || matches!(args.version, Some(Version::V4)) {
                let config = match chain.as_str() {
                    "ethereum" | "mainnet" | "eth" => Some(SubgraphConfig::mainnet_v4(&api_key)),
                    "arbitrum" | "arb" => Some(SubgraphConfig::arbitrum_v4(&api_key)),
                    "base" => Some(SubgraphConfig::base_v4(&api_key)),
                    "polygon" | "matic" => Some(SubgraphConfig::polygon_v4(&api_key)),
                    _ => None,
                };

                if let Some(config) = config {
                    let client = SubgraphClient::new(config)?;
                    match client.get_positions_v4(&address).await {
                        Ok(positions) => {
                            // The V4 subgraph only indexes position NFT ownership;
                            // pool key, ticks and liquidity live in the on-chain
                            // PositionManager.
                            for pos in positions {
                                all_positions.push(serde_json::json!({
                                    "version": "v4",
                                    "positionId": pos.token_id,
                                    "owner": pos.owner,
                                    "origin": pos.origin,
                                    "createdAtTimestamp": pos.created_at_timestamp,
                                }));
                            }
                        }
                        Err(e) => {
                            // Surface V4 errors instead of silently returning [];
                            // fail if V4 was explicitly requested.
                            if matches!(args.version, Some(Version::V4)) {
                                return Err(anyhow::anyhow!("V4 positions query failed: {e}"));
                            }
                            eprintln!("Warning: V4 positions query failed: {e}");
                        }
                    }
                } else if matches!(args.version, Some(Version::V4)) {
                    anyhow::bail!("Uniswap V4 subgraph not available for chain '{}'", chain);
                }
            }

            // Output results
            if matches!(args.format, OutputFormat::Json) {
                println!("{}", serde_json::to_string_pretty(&all_positions)?);
            } else if matches!(args.format, OutputFormat::Ndjson) {
                println!("{}", serde_json::to_string(&all_positions)?);
            } else if all_positions.is_empty() {
                println!("No LP positions found for {} on {}", address, args.chain);
            } else {
                println!("\nUniswap LP Positions for {}", address);
                println!("{}", "=".repeat(80));
                println!("Chain: {}\n", args.chain);

                for pos in &all_positions {
                    let version = pos["version"].as_str().unwrap_or("?");
                    let token0 = pos["token0"]["symbol"].as_str().unwrap_or("?");
                    let token1 = pos["token1"]["symbol"].as_str().unwrap_or("?");
                    let fee = pos["feeTier"].as_str().unwrap_or("?");

                    println!(
                        "[{}] {}/{} ({})",
                        version.to_uppercase(),
                        token0,
                        token1,
                        fee
                    );
                    println!("  Position ID: {}", pos["positionId"]);
                    println!("  Pool: {}", pos["pool"]);

                    if version == "v2" {
                        let t0_amt = pos["token0"]["amount"].as_f64().unwrap_or(0.0);
                        let t1_amt = pos["token1"]["amount"].as_f64().unwrap_or(0.0);
                        let usd = pos["usdValue"].as_f64().unwrap_or(0.0);
                        let share = pos["poolShare"].as_str().unwrap_or("?");
                        println!("  {} Amount: {:.6}", token0, t0_amt);
                        println!("  {} Amount: {:.6}", token1, t1_amt);
                        println!("  Pool Share: {}", share);
                        if usd > 0.0 {
                            println!("  USD Value: ${:.2}", usd);
                        }
                    } else if version == "v3" {
                        let in_range = pos["inRange"].as_bool().unwrap_or(false);
                        let tick_lower = pos["tickRange"]["lower"].as_i64().unwrap_or(0);
                        let tick_upper = pos["tickRange"]["upper"].as_i64().unwrap_or(0);
                        let current = pos["tickRange"]["current"].as_i64().unwrap_or(0);
                        let liquidity = pos["liquidity"].as_f64().unwrap_or(0.0);

                        println!("  Liquidity: {:.0}", liquidity);
                        println!(
                            "  Tick Range: {} to {} (current: {})",
                            tick_lower, tick_upper, current
                        );
                        println!("  In Range: {}", if in_range { "✓ Yes" } else { "✗ No" });

                        let fees0 = pos["token0"]["collectedFees"].as_f64().unwrap_or(0.0);
                        let fees1 = pos["token1"]["collectedFees"].as_f64().unwrap_or(0.0);
                        if fees0 > 0.0 || fees1 > 0.0 {
                            println!(
                                "  Collected Fees: {:.6} {} / {:.6} {}",
                                fees0, token0, fees1, token1
                            );
                        }
                    } else if version == "v4" {
                        if let Some(ts) = pos["createdAtTimestamp"].as_str() {
                            println!("  Created: {}", ts);
                        }
                        println!(
                            "  (pool/liquidity: query the V4 PositionManager on-chain for this token ID)"
                        );
                    }
                    println!();
                }

                println!("Total positions: {}", all_positions.len());
            }
        }

        UniswapCommands::Balance(args) => {
            let (rpc_url, factory, lens_chain) =
                resolve_lens_target(args.rpc_url.as_deref(), chain)?;
            let token: Address = args.token.parse()?;
            let account: Address = args.account.parse()?;

            if !quiet {
                eprintln!("Fetching balance on {}...", lens_chain);
            }

            let client = LensClient::new(&rpc_url, factory)?;
            let balance = client.get_token_balance(token, account).await?;

            let output = serde_json::json!({
                "token": format!("{:#x}", token),
                "account": format!("{:#x}", account),
                "balance": balance.to_string(),
            });

            println!("{}", serde_json::to_string_pretty(&output)?);
        }

        UniswapCommands::Addresses(args) => {
            let show_all = !args.factories && !args.pools && !args.tokens;

            let mut output = serde_json::Map::new();

            // Factory addresses
            if args.factories || show_all {
                let mut factory_map = serde_json::Map::new();

                if args.version.is_none() || matches!(args.version, Some(Version::V2)) {
                    factory_map.insert(
                        "v2".to_string(),
                        serde_json::json!({
                            "mainnet": format!("{:#x}", factories::v2::MAINNET),
                            "arbitrum": format!("{:#x}", factories::v2::ARBITRUM),
                            "optimism": format!("{:#x}", factories::v2::OPTIMISM),
                            "polygon": format!("{:#x}", factories::v2::POLYGON),
                            "base": format!("{:#x}", factories::v2::BASE),
                        }),
                    );
                }

                if args.version.is_none() || matches!(args.version, Some(Version::V3)) {
                    factory_map.insert(
                        "v3".to_string(),
                        serde_json::json!({
                            "mainnet": format!("{:#x}", factories::v3::MAINNET),
                            "arbitrum": format!("{:#x}", factories::v3::ARBITRUM),
                            "optimism": format!("{:#x}", factories::v3::OPTIMISM),
                            "polygon": format!("{:#x}", factories::v3::POLYGON),
                            "base": format!("{:#x}", factories::v3::BASE),
                        }),
                    );
                }

                if args.version.is_none() || matches!(args.version, Some(Version::V4)) {
                    factory_map.insert(
                        "v4_pool_manager".to_string(),
                        serde_json::json!({
                            "mainnet": format!("{:#x}", factories::v4::MAINNET),
                            "arbitrum": format!("{:#x}", factories::v4::ARBITRUM),
                            "optimism": format!("{:#x}", factories::v4::OPTIMISM),
                            "polygon": format!("{:#x}", factories::v4::POLYGON),
                            "base": format!("{:#x}", factories::v4::BASE),
                        }),
                    );
                }

                output.insert(
                    "factories".to_string(),
                    serde_json::Value::Object(factory_map),
                );
            }

            // Pool addresses (V2 and V3 only, V4 pools are dynamic)
            if args.pools || show_all {
                let mut pool_map = serde_json::Map::new();

                if args.version.is_none() || matches!(args.version, Some(Version::V2)) {
                    pool_map.insert(
                        "v2".to_string(),
                        serde_json::json!({
                            "mainnet_weth_usdc": format!("{:#x}", pools::v2::MAINNET_WETH_USDC),
                            "mainnet_weth_usdt": format!("{:#x}", pools::v2::MAINNET_WETH_USDT),
                            "mainnet_wbtc_weth": format!("{:#x}", pools::v2::MAINNET_WBTC_WETH),
                            "mainnet_dai_weth": format!("{:#x}", pools::v2::MAINNET_DAI_WETH),
                        }),
                    );
                }

                if args.version.is_none() || matches!(args.version, Some(Version::V3)) {
                    pool_map.insert(
                        "v3".to_string(),
                        serde_json::json!({
                            "mainnet_weth_usdc_0.05%": format!("{:#x}", pools::v3::MAINNET_WETH_USDC_005),
                            "mainnet_weth_usdc_0.3%": format!("{:#x}", pools::v3::MAINNET_WETH_USDC_030),
                            "mainnet_weth_usdt_0.05%": format!("{:#x}", pools::v3::MAINNET_WETH_USDT_005),
                            "mainnet_wbtc_weth_0.3%": format!("{:#x}", pools::v3::MAINNET_WBTC_WETH_030),
                        }),
                    );
                }

                if args.version.is_none() || matches!(args.version, Some(Version::V4)) {
                    pool_map.insert(
                        "v4".to_string(),
                        serde_json::json!({
                            "note": "V4 pools are identified by PoolKey, not fixed addresses"
                        }),
                    );
                }

                output.insert("pools".to_string(), serde_json::Value::Object(pool_map));
            }

            // Token addresses
            if args.tokens || show_all {
                output.insert(
                    "tokens".to_string(),
                    serde_json::json!({
                        "mainnet_weth": format!("{:#x}", tokens::MAINNET_WETH),
                        "mainnet_usdc": format!("{:#x}", tokens::MAINNET_USDC),
                        "mainnet_usdt": format!("{:#x}", tokens::MAINNET_USDT),
                        "mainnet_wbtc": format!("{:#x}", tokens::MAINNET_WBTC),
                        "mainnet_dai": format!("{:#x}", tokens::MAINNET_DAI),
                    }),
                );
            }

            // Subgraph IDs
            if show_all {
                output.insert(
                    "subgraph_ids".to_string(),
                    serde_json::json!({
                        "v2": {
                            "mainnet": subgraph_ids::MAINNET_V2,
                        },
                        "v3": {
                            "mainnet": subgraph_ids::MAINNET_V3,
                            "arbitrum": subgraph_ids::ARBITRUM_V3,
                            "optimism": subgraph_ids::OPTIMISM_V3,
                            "polygon": subgraph_ids::POLYGON_V3,
                            "base": subgraph_ids::BASE_V3,
                            "bsc": subgraph_ids::BSC_V3,
                        },
                        "v4": {
                            "mainnet": subgraph_ids::MAINNET_V4,
                            "arbitrum": subgraph_ids::ARBITRUM_V4,
                            "base": subgraph_ids::BASE_V4,
                            "polygon": subgraph_ids::POLYGON_V4,
                        },
                    }),
                );
            }

            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::Value::Object(output))?
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_rpc_url_wins() {
        let (url, _, chain) =
            resolve_lens_target(Some("https://example.invalid"), "ethereum").unwrap();
        assert_eq!(url, "https://example.invalid");
        assert_eq!(chain.chain_id(), 1);
    }
}
