//! Yield aggregation from Curve and DefiLlama
//!
//! Combines yield data from multiple sources to provide comprehensive
//! DeFi yield information with cross-source comparison.

use super::{get_cached_config, AggregatedResult, LatencyMeasure, SourceResult};
use futures::future::join_all;
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};

/// Yield source enum for CLI selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldSource {
    All,
    Curve,
    Llama,
    Uniswap,
}

impl std::str::FromStr for YieldSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(YieldSource::All),
            "curve" | "crv" => Ok(YieldSource::Curve),
            "llama" | "defillama" => Ok(YieldSource::Llama),
            "uniswap" | "uni" => Ok(YieldSource::Uniswap),
            _ => Err(format!("Unknown yield source: {}", s)),
        }
    }
}

impl std::fmt::Display for YieldSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YieldSource::All => write!(f, "all"),
            YieldSource::Curve => write!(f, "curve"),
            YieldSource::Llama => write!(f, "llama"),
            YieldSource::Uniswap => write!(f, "uniswap"),
        }
    }
}

/// Normalized yield pool data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedYield {
    /// Pool/vault identifier (address or ID)
    pub pool_id: String,
    /// Pool name or symbol
    pub symbol: String,
    /// Project/protocol name
    pub project: String,
    /// Chain name
    pub chain: String,
    /// Base APY (trading fees, interest)
    pub apy_base: Option<f64>,
    /// Reward APY (token incentives)
    pub apy_reward: Option<f64>,
    /// Total APY (base + reward)
    pub apy_total: Option<f64>,
    /// Total value locked in USD
    pub tvl_usd: Option<f64>,
    /// Underlying tokens
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub underlying_tokens: Vec<String>,
    /// Whether this is a stablecoin pool
    pub stablecoin: Option<bool>,
    /// URL for more info
    pub url: Option<String>,
}

/// Yield aggregation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldAggregation {
    /// Total pools found
    pub total_pools: usize,
    /// Number of pools from Curve
    pub curve_pools: usize,
    /// Number of pools from DefiLlama
    pub llama_pools: usize,
    /// Number of pools from Uniswap (all versions combined)
    pub uniswap_pools: usize,
    /// Number of V2 pools from Uniswap
    pub uniswap_v2_pools: usize,
    /// Number of V3 pools from Uniswap
    pub uniswap_v3_pools: usize,
    /// Number of V4 pools from Uniswap
    pub uniswap_v4_pools: usize,
    /// Highest APY found
    pub max_apy: Option<f64>,
    /// Average APY across all pools
    pub avg_apy: Option<f64>,
    /// Total TVL across all pools
    pub total_tvl_usd: Option<f64>,
}

/// Lending yield data (from Curve lending vaults)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedLendingYield {
    /// Vault address
    pub address: String,
    /// Vault name
    pub name: Option<String>,
    /// Chain
    pub chain: String,
    /// Collateral token symbol
    pub collateral_symbol: Option<String>,
    /// Borrowed token symbol
    pub borrowed_symbol: Option<String>,
    /// Lend APY (for suppliers)
    pub lend_apy: Option<f64>,
    /// Borrow APY (cost for borrowers)
    pub borrow_apy: Option<f64>,
    /// Utilization rate
    pub utilization: Option<f64>,
    /// Total assets in vault
    pub total_assets: Option<String>,
}

/// Fetch yields from Curve (pool APYs from volumes API)
async fn fetch_curve_yields(chain: &str) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    let client = match crv::Client::new() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("curve", e.to_string(), measure.elapsed_ms());
        }
    };

    // Get base APYs from volumes API
    match client.volumes().get_base_apys(chain).await {
        Ok(response) => {
            // Parse the JSON data - BaseApysResponse has generic JSON
            let mut yields: Vec<NormalizedYield> = Vec::new();

            // The data is a map of pool addresses to APY values
            if let Some(obj) = response.data.as_object() {
                for (address, value) in obj {
                    let apy: Option<f64> = value.as_f64();
                    if let Some(apy_val) = apy {
                        yields.push(NormalizedYield {
                            pool_id: address.clone(),
                            symbol: format!("Curve Pool {}", &address[..8]),
                            project: "curve".to_string(),
                            chain: chain.to_string(),
                            apy_base: Some(apy_val),
                            apy_reward: None, // Would need gauge data
                            apy_total: Some(apy_val),
                            tvl_usd: None,
                            underlying_tokens: vec![],
                            stablecoin: None,
                            url: Some(format!("https://curve.fi/#/{}/pools/{}", chain, address)),
                        });
                    }
                }
            }

            SourceResult::success("curve", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("curve", e.to_string(), measure.elapsed_ms()),
    }
}

/// Fetch yields from DefiLlama
async fn fetch_llama_yields(
    chain: Option<&str>,
    project: Option<&str>,
) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    // Try config first for Pro API key
    let config = get_cached_config();
    let api_key = config
        .as_ref()
        .and_then(|c| c.defillama.as_ref())
        .and_then(|l| l.api_key.as_ref().map(|s| s.expose_secret().to_string()))
        .or_else(|| std::env::var("DEFILLAMA_API_KEY").ok());

    let client = match api_key {
        Some(key) => match dllma::Client::with_api_key(&key) {
            Ok(c) => c,
            Err(e) => {
                return SourceResult::error("llama", e.to_string(), measure.elapsed_ms());
            }
        },
        None => match dllma::Client::new() {
            Ok(c) => c,
            Err(e) => {
                return SourceResult::error("llama", e.to_string(), measure.elapsed_ms());
            }
        },
    };

    match client.yields().pools().await {
        Ok(pools) => {
            let mut yields: Vec<NormalizedYield> = pools
                .into_iter()
                .filter(|p| {
                    // Filter by chain if specified
                    if let Some(c) = chain {
                        if !p.chain.eq_ignore_ascii_case(c) {
                            return false;
                        }
                    }
                    // Filter by project if specified
                    if let Some(proj) = project {
                        if !p.project.eq_ignore_ascii_case(proj) {
                            return false;
                        }
                    }
                    true
                })
                .map(|p| NormalizedYield {
                    pool_id: p.pool.clone(),
                    symbol: p.symbol,
                    project: p.project,
                    chain: p.chain,
                    apy_base: p.apy_base,
                    apy_reward: p.apy_reward,
                    apy_total: p.apy,
                    tvl_usd: p.tvl_usd,
                    underlying_tokens: p.underlying_tokens,
                    stablecoin: p.stablecoin,
                    url: p.url,
                })
                .collect();

            // Sort by APY descending
            yields.sort_by(|a, b| {
                b.apy_total
                    .unwrap_or(0.0)
                    .partial_cmp(&a.apy_total.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            SourceResult::success("llama", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("llama", e.to_string(), measure.elapsed_ms()),
    }
}

/// Fetch yields from Uniswap V2 pairs via The Graph subgraph
async fn fetch_uniswap_v2_yields(chain: &str, api_key: &str) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    // V2 only available on mainnet
    if !matches!(chain.to_lowercase().as_str(), "ethereum" | "mainnet" | "eth") {
        return SourceResult::error(
            "uniswap-v2",
            format!("Uniswap V2 subgraph only available on Ethereum mainnet, not {}", chain),
            measure.elapsed_ms(),
        );
    }

    let subgraph_config = unswp::SubgraphConfig::mainnet_v2(api_key);

    let client = match unswp::SubgraphClient::new(subgraph_config) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("uniswap-v2", e.to_string(), measure.elapsed_ms());
        }
    };

    match client.get_top_pairs(100).await {
        Ok(pairs) => {
            let mut yields: Vec<NormalizedYield> = Vec::new();

            for pair in pairs {
                // V2 has 0.3% fee on all swaps
                // APY = (volume * 0.003 / TVL) * 365 annualized
                let tvl_f: f64 = pair.reserve_usd.parse().unwrap_or(0.0);
                let volume_f: f64 = pair.volume_usd.parse().unwrap_or(0.0);
                let apy = if tvl_f > 0.0 && volume_f > 0.0 {
                    // Estimate daily volume from total volume and estimate APY
                    // This is approximate - cumulative volume / TVL gives rough fee yield
                    Some((volume_f * 0.003 / tvl_f) * 100.0)
                } else {
                    None
                };

                let symbol = format!("{}/{}", pair.token0.symbol, pair.token1.symbol);
                let symbol_with_fee = format!("{} (0.3%)", symbol);

                let tvl_usd: Option<f64> = pair.reserve_usd.parse().ok();

                let underlying = vec![pair.token0.symbol.clone(), pair.token1.symbol.clone()];
                let stablecoin = is_stablecoin_pool(&underlying);

                yields.push(NormalizedYield {
                    pool_id: pair.id.clone(),
                    symbol: symbol_with_fee,
                    project: "uniswap-v2".to_string(),
                    chain: chain.to_string(),
                    apy_base: apy,
                    apy_reward: None,
                    apy_total: apy,
                    tvl_usd,
                    underlying_tokens: underlying,
                    stablecoin: Some(stablecoin),
                    url: Some(format!(
                        "https://info.uniswap.org/#/pools/{}",
                        pair.id
                    )),
                });
            }

            yields.sort_by(|a, b| {
                b.apy_total
                    .unwrap_or(0.0)
                    .partial_cmp(&a.apy_total.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            SourceResult::success("uniswap-v2", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("uniswap-v2", e.to_string(), measure.elapsed_ms()),
    }
}

/// Fetch yields from Uniswap V3 pools via The Graph subgraph
async fn fetch_uniswap_v3_yields(chain: &str, api_key: &str) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    // Create subgraph config based on chain
    let subgraph_config = match chain.to_lowercase().as_str() {
        "ethereum" | "mainnet" | "eth" => unswp::SubgraphConfig::mainnet_v3(api_key),
        "arbitrum" | "arb" => unswp::SubgraphConfig::arbitrum_v3(api_key),
        "optimism" | "op" => unswp::SubgraphConfig::optimism_v3(api_key),
        "polygon" | "matic" => unswp::SubgraphConfig::mainnet_v3(api_key)
            .with_subgraph_id(unswp::subgraph_ids::POLYGON_V3),
        "base" => unswp::SubgraphConfig::base_v3(api_key),
        "bsc" | "binance" => unswp::SubgraphConfig::mainnet_v3(api_key)
            .with_subgraph_id(unswp::subgraph_ids::BSC_V3),
        _ => {
            return SourceResult::error(
                "uniswap-v3",
                format!("Unsupported chain for Uniswap V3 subgraph: {}", chain),
                measure.elapsed_ms(),
            );
        }
    };

    let client = match unswp::SubgraphClient::new(subgraph_config) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("uniswap-v3", e.to_string(), measure.elapsed_ms());
        }
    };

    match client.get_top_pools(100).await {
        Ok(pools) => {
            let mut yields: Vec<NormalizedYield> = Vec::new();

            for pool in pools {
                let tvl_f: f64 = pool.total_value_locked_usd.parse().unwrap_or(0.0);
                let fees_f: f64 = pool.fees_usd.parse().unwrap_or(0.0);
                let apy = if tvl_f > 0.0 {
                    Some((fees_f / tvl_f) * 100.0)
                } else {
                    None
                };

                let symbol = format!("{}/{}", pool.token0.symbol, pool.token1.symbol);
                let fee_tier: f64 = pool.fee_tier.parse().unwrap_or(0.0) / 10000.0;
                let symbol_with_fee = format!("{} ({}%)", symbol, fee_tier);

                let tvl_usd: Option<f64> = pool.total_value_locked_usd.parse().ok();

                let underlying = vec![pool.token0.symbol.clone(), pool.token1.symbol.clone()];
                let stablecoin = is_stablecoin_pool(&underlying);

                yields.push(NormalizedYield {
                    pool_id: pool.id.clone(),
                    symbol: symbol_with_fee,
                    project: "uniswap-v3".to_string(),
                    chain: chain.to_string(),
                    apy_base: apy,
                    apy_reward: None,
                    apy_total: apy,
                    tvl_usd,
                    underlying_tokens: underlying,
                    stablecoin: Some(stablecoin),
                    url: Some(format!(
                        "https://info.uniswap.org/#/{}/pools/{}",
                        chain, pool.id
                    )),
                });
            }

            yields.sort_by(|a, b| {
                b.apy_total
                    .unwrap_or(0.0)
                    .partial_cmp(&a.apy_total.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            SourceResult::success("uniswap-v3", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("uniswap-v3", e.to_string(), measure.elapsed_ms()),
    }
}

/// Fetch yields from Uniswap V4 pools via The Graph subgraph
async fn fetch_uniswap_v4_yields(chain: &str, api_key: &str) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    // V4 subgraph config based on chain
    let subgraph_config = match chain.to_lowercase().as_str() {
        "ethereum" | "mainnet" | "eth" => unswp::SubgraphConfig::mainnet_v4(api_key),
        "arbitrum" | "arb" => unswp::SubgraphConfig::arbitrum_v4(api_key),
        "base" => unswp::SubgraphConfig::base_v4(api_key),
        "polygon" | "matic" => unswp::SubgraphConfig::mainnet_v4(api_key)
            .with_subgraph_id(unswp::subgraph_ids::POLYGON_V4),
        _ => {
            return SourceResult::error(
                "uniswap-v4",
                format!("Unsupported chain for Uniswap V4 subgraph: {}", chain),
                measure.elapsed_ms(),
            );
        }
    };

    let client = match unswp::SubgraphClient::new(subgraph_config) {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("uniswap-v4", e.to_string(), measure.elapsed_ms());
        }
    };

    match client.get_top_pools_v4(100).await {
        Ok(pools) => {
            let mut yields: Vec<NormalizedYield> = Vec::new();

            for pool in pools {
                let tvl_f: f64 = pool.total_value_locked_usd.parse().unwrap_or(0.0);
                let fees_f: f64 = pool.fees_usd.parse().unwrap_or(0.0);
                let apy = if tvl_f > 0.0 {
                    Some((fees_f / tvl_f) * 100.0)
                } else {
                    None
                };

                let symbol = format!("{}/{}", pool.token0.symbol, pool.token1.symbol);
                // V4 feeTier is in hundredths of a bip (e.g., 3000 = 0.3%)
                let fee_f: f64 = pool.fee_tier.parse().unwrap_or(0.0) / 10000.0;
                let symbol_with_fee = format!("{} ({}%)", symbol, fee_f);

                let tvl_usd: Option<f64> = pool.total_value_locked_usd.parse().ok();

                let underlying = vec![pool.token0.symbol.clone(), pool.token1.symbol.clone()];
                let stablecoin = is_stablecoin_pool(&underlying);

                // Note if pool has hooks
                let hooks_note = pool.hooks.as_ref()
                    .filter(|h| !h.is_empty() && *h != "0x0000000000000000000000000000000000000000")
                    .map(|_| " [hooks]")
                    .unwrap_or("");

                yields.push(NormalizedYield {
                    pool_id: pool.id.clone(),
                    symbol: format!("{}{}", symbol_with_fee, hooks_note),
                    project: "uniswap-v4".to_string(),
                    chain: chain.to_string(),
                    apy_base: apy,
                    apy_reward: None,
                    apy_total: apy,
                    tvl_usd,
                    underlying_tokens: underlying,
                    stablecoin: Some(stablecoin),
                    url: Some(format!(
                        "https://app.uniswap.org/explore/pools/{}/{}",
                        chain, pool.id
                    )),
                });
            }

            yields.sort_by(|a, b| {
                b.apy_total
                    .unwrap_or(0.0)
                    .partial_cmp(&a.apy_total.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            SourceResult::success("uniswap-v4", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("uniswap-v4", e.to_string(), measure.elapsed_ms()),
    }
}

/// Fetch yields from all Uniswap versions (V2, V3, V4) via The Graph subgraph
async fn fetch_uniswap_yields(chain: &str) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    // Get API key from config or env
    let config = get_cached_config();
    let api_key = config
        .as_ref()
        .and_then(|c| c.thegraph.as_ref())
        .map(|g| g.api_key.expose_secret().to_string())
        .or_else(|| std::env::var("THEGRAPH_API_KEY").ok());

    let api_key = match api_key {
        Some(key) => key,
        None => {
            return SourceResult::error(
                "uniswap",
                "TheGraph API key not configured (set THEGRAPH_API_KEY or add [thegraph] to config)",
                measure.elapsed_ms(),
            );
        }
    };

    // Fetch from all versions in parallel
    let (v2_result, v3_result, v4_result) = tokio::join!(
        fetch_uniswap_v2_yields(chain, &api_key),
        fetch_uniswap_v3_yields(chain, &api_key),
        fetch_uniswap_v4_yields(chain, &api_key),
    );

    // Combine results
    let mut all_yields: Vec<NormalizedYield> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    if let Some(yields) = v2_result.data {
        all_yields.extend(yields);
    } else if let Some(e) = v2_result.error {
        // V2 error is expected for non-mainnet chains, don't treat as error
        if !e.contains("only available on Ethereum mainnet") {
            errors.push(format!("V2: {}", e));
        }
    }

    if let Some(yields) = v3_result.data {
        all_yields.extend(yields);
    } else if let Some(e) = v3_result.error {
        errors.push(format!("V3: {}", e));
    }

    if let Some(yields) = v4_result.data {
        all_yields.extend(yields);
    } else if let Some(e) = v4_result.error {
        // V4 error for unsupported chains is expected
        if !e.contains("Unsupported chain") {
            errors.push(format!("V4: {}", e));
        }
    }

    // Sort combined results by APY descending
    all_yields.sort_by(|a, b| {
        b.apy_total
            .unwrap_or(0.0)
            .partial_cmp(&a.apy_total.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if all_yields.is_empty() && !errors.is_empty() {
        SourceResult::error("uniswap", errors.join("; "), measure.elapsed_ms())
    } else {
        let mut result = SourceResult::success("uniswap", all_yields, measure.elapsed_ms());
        if !errors.is_empty() {
            result.error = Some(errors.join("; "));
        }
        result
    }
}

/// Check if a pool contains stablecoin pairs
fn is_stablecoin_pool(tokens: &[String]) -> bool {
    let stablecoins = [
        "USDC", "USDT", "DAI", "FRAX", "TUSD", "USDP", "GUSD", "LUSD", "BUSD", "USDD", "PYUSD",
        "EURC", "EUROC", "EURS", "EURT", "CEUR",
    ];

    // Pool is stablecoin if all tokens are stablecoins
    tokens.iter().all(|t| {
        let upper = t.to_uppercase();
        stablecoins.iter().any(|s| upper.contains(s))
    })
}

/// Fetch lending yields from Curve
async fn fetch_curve_lending_yields(chain: &str) -> SourceResult<Vec<NormalizedLendingYield>> {
    let measure = LatencyMeasure::start();

    let client = match crv::Client::new() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("curve-lending", e.to_string(), measure.elapsed_ms());
        }
    };

    match client.lending().get_all_on_chain(chain).await {
        Ok(response) => {
            let yields: Vec<NormalizedLendingYield> = response
                .data
                .lending_vault_data
                .into_iter()
                .map(|v| NormalizedLendingYield {
                    address: v.address,
                    name: v.name,
                    chain: chain.to_string(),
                    collateral_symbol: v.collateral_token.and_then(|t| t.symbol),
                    borrowed_symbol: v.borrowed_token.and_then(|t| t.symbol),
                    lend_apy: v.lend_apy,
                    borrow_apy: v.borrow_apy,
                    utilization: v.utilization,
                    total_assets: v.total_assets,
                })
                .collect();

            SourceResult::success("curve-lending", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("curve-lending", e.to_string(), measure.elapsed_ms()),
    }
}

/// Fetch aggregated yields from all sources
pub async fn fetch_yields_aggregated(
    chain: Option<&str>,
    project: Option<&str>,
    sources: YieldSource,
) -> AggregatedResult<Vec<NormalizedYield>, YieldAggregation> {
    let start = std::time::Instant::now();

    let futures: Vec<_> = match sources {
        YieldSource::All => {
            let curve_chain = chain.unwrap_or("ethereum");
            let uniswap_chain = chain.unwrap_or("ethereum");
            vec![
                Box::pin(fetch_curve_yields(curve_chain))
                    as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>,
                Box::pin(fetch_llama_yields(chain, project)),
                Box::pin(fetch_uniswap_yields(uniswap_chain)),
            ]
        }
        YieldSource::Curve => {
            let curve_chain = chain.unwrap_or("ethereum");
            vec![Box::pin(fetch_curve_yields(curve_chain))]
        }
        YieldSource::Llama => {
            vec![Box::pin(fetch_llama_yields(chain, project))]
        }
        YieldSource::Uniswap => {
            let uniswap_chain = chain.unwrap_or("ethereum");
            vec![Box::pin(fetch_uniswap_yields(uniswap_chain))]
        }
    };

    let results = join_all(futures).await;

    // Combine all yields
    let mut all_yields: Vec<NormalizedYield> = Vec::new();
    let mut curve_count = 0;
    let mut llama_count = 0;
    let mut uniswap_count = 0;

    for result in &results {
        if let Some(yields) = &result.data {
            if result.source == "curve" {
                curve_count = yields.len();
            } else if result.source == "llama" {
                llama_count = yields.len();
            } else if result.source == "uniswap" {
                uniswap_count = yields.len();
            }
            all_yields.extend(yields.clone());
        }
    }

    // Sort by total APY descending
    all_yields.sort_by(|a, b| {
        b.apy_total
            .unwrap_or(0.0)
            .partial_cmp(&a.apy_total.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // Calculate aggregation stats
    let total_pools = all_yields.len();
    let max_apy = all_yields
        .iter()
        .filter_map(|y| y.apy_total)
        .fold(None, |acc, v| Some(acc.map_or(v, |a: f64| a.max(v))));
    let avg_apy = if total_pools > 0 {
        let sum: f64 = all_yields.iter().filter_map(|y| y.apy_total).sum();
        let count = all_yields.iter().filter(|y| y.apy_total.is_some()).count();
        if count > 0 {
            Some(sum / count as f64)
        } else {
            None
        }
    } else {
        None
    };
    let total_tvl: Option<f64> = {
        let sum: f64 = all_yields.iter().filter_map(|y| y.tvl_usd).sum();
        if sum > 0.0 {
            Some(sum)
        } else {
            None
        }
    };

    // Count Uniswap pools by version
    let uniswap_v2_count = all_yields.iter().filter(|y| y.project == "uniswap-v2").count();
    let uniswap_v3_count = all_yields.iter().filter(|y| y.project == "uniswap-v3").count();
    let uniswap_v4_count = all_yields.iter().filter(|y| y.project == "uniswap-v4").count();

    let aggregation = YieldAggregation {
        total_pools,
        curve_pools: curve_count,
        llama_pools: llama_count,
        uniswap_pools: uniswap_count,
        uniswap_v2_pools: uniswap_v2_count,
        uniswap_v3_pools: uniswap_v3_count,
        uniswap_v4_pools: uniswap_v4_count,
        max_apy,
        avg_apy,
        total_tvl_usd: total_tvl,
    };

    // Convert results to the expected format
    let source_results: Vec<SourceResult<Vec<NormalizedYield>>> = results;

    AggregatedResult::new(
        aggregation,
        source_results,
        start.elapsed().as_millis() as u64,
    )
}

/// Fetch Curve lending yields
pub async fn fetch_lending_yields(
    chain: &str,
) -> AggregatedResult<Vec<NormalizedLendingYield>, LendingYieldAggregation> {
    let start = std::time::Instant::now();

    let result = fetch_curve_lending_yields(chain).await;

    let (total_vaults, avg_lend_apy, avg_borrow_apy) = if let Some(yields) = &result.data {
        let total = yields.len();
        let lend_sum: f64 = yields.iter().filter_map(|y| y.lend_apy).sum();
        let lend_count = yields.iter().filter(|y| y.lend_apy.is_some()).count();
        let borrow_sum: f64 = yields.iter().filter_map(|y| y.borrow_apy).sum();
        let borrow_count = yields.iter().filter(|y| y.borrow_apy.is_some()).count();

        (
            total,
            if lend_count > 0 {
                Some(lend_sum / lend_count as f64)
            } else {
                None
            },
            if borrow_count > 0 {
                Some(borrow_sum / borrow_count as f64)
            } else {
                None
            },
        )
    } else {
        (0, None, None)
    };

    let aggregation = LendingYieldAggregation {
        total_vaults,
        avg_lend_apy,
        avg_borrow_apy,
    };

    AggregatedResult::new(
        aggregation,
        vec![result],
        start.elapsed().as_millis() as u64,
    )
}

/// Lending yield aggregation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LendingYieldAggregation {
    /// Total lending vaults
    pub total_vaults: usize,
    /// Average lend APY
    pub avg_lend_apy: Option<f64>,
    /// Average borrow APY
    pub avg_borrow_apy: Option<f64>,
}

/// Compare yields for the same protocol between sources
pub async fn compare_curve_yields() -> Vec<YieldComparison> {
    let llama_result = fetch_llama_yields(None, Some("curve")).await;
    let curve_result = fetch_curve_yields("ethereum").await;

    let mut comparisons = Vec::new();

    if let (Some(llama_yields), Some(curve_yields)) = (llama_result.data, curve_result.data) {
        // Create a map of Curve pool addresses to their APYs
        let curve_map: std::collections::HashMap<String, f64> = curve_yields
            .into_iter()
            .filter_map(|y| y.apy_total.map(|apy| (y.pool_id.to_lowercase(), apy)))
            .collect();

        // Find matching pools in DefiLlama data
        for llama_yield in llama_yields {
            if llama_yield.project.to_lowercase() == "curve" {
                let pool_id = llama_yield.pool_id.to_lowercase();
                if let Some(&curve_apy) = curve_map.get(&pool_id) {
                    let llama_apy = llama_yield.apy_total.unwrap_or(0.0);
                    let diff = curve_apy - llama_apy;
                    let diff_pct = if llama_apy != 0.0 {
                        (diff / llama_apy) * 100.0
                    } else {
                        0.0
                    };

                    comparisons.push(YieldComparison {
                        pool_id: llama_yield.pool_id,
                        symbol: llama_yield.symbol,
                        chain: llama_yield.chain,
                        curve_apy,
                        llama_apy,
                        difference: diff,
                        difference_pct: diff_pct,
                    });
                }
            }
        }
    }

    // Sort by absolute difference
    comparisons.sort_by(|a, b| {
        b.difference
            .abs()
            .partial_cmp(&a.difference.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    comparisons
}

/// Yield comparison between sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldComparison {
    /// Pool identifier
    pub pool_id: String,
    /// Pool symbol
    pub symbol: String,
    /// Chain
    pub chain: String,
    /// APY from Curve API
    pub curve_apy: f64,
    /// APY from DefiLlama
    pub llama_apy: f64,
    /// Absolute difference (curve - llama)
    pub difference: f64,
    /// Percentage difference
    pub difference_pct: f64,
}
