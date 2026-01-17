//! Yield aggregation from Curve and DefiLlama
//!
//! Combines yield data from multiple sources to provide comprehensive
//! DeFi yield information with cross-source comparison.

use super::{AggregatedResult, LatencyMeasure, SourceResult};
use crate::config::ConfigFile;
use futures::future::join_all;
use serde::{Deserialize, Serialize};

/// Yield source enum for CLI selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum YieldSource {
    All,
    Curve,
    Llama,
}

impl std::str::FromStr for YieldSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(YieldSource::All),
            "curve" | "crv" => Ok(YieldSource::Curve),
            "llama" | "defillama" => Ok(YieldSource::Llama),
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
    let config = ConfigFile::load_default().ok().flatten();
    let api_key = config
        .as_ref()
        .and_then(|c| c.defillama.as_ref())
        .and_then(|l| l.api_key.clone())
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
            vec![
                Box::pin(fetch_curve_yields(curve_chain))
                    as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>,
                Box::pin(fetch_llama_yields(chain, project)),
            ]
        }
        YieldSource::Curve => {
            let curve_chain = chain.unwrap_or("ethereum");
            vec![Box::pin(fetch_curve_yields(curve_chain))]
        }
        YieldSource::Llama => {
            vec![Box::pin(fetch_llama_yields(chain, project))]
        }
    };

    let results = join_all(futures).await;

    // Combine all yields
    let mut all_yields: Vec<NormalizedYield> = Vec::new();
    let mut curve_count = 0;
    let mut llama_count = 0;

    for result in &results {
        if let Some(yields) = &result.data {
            if result.source == "curve" {
                curve_count = yields.len();
            } else if result.source == "llama" {
                llama_count = yields.len();
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

    let aggregation = YieldAggregation {
        total_pools,
        curve_pools: curve_count,
        llama_pools: llama_count,
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
