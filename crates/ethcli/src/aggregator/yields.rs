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
    Yearn,
}

impl std::str::FromStr for YieldSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "all" => Ok(YieldSource::All),
            "curve" | "crv" => Ok(YieldSource::Curve),
            "llama" | "defillama" => Ok(YieldSource::Llama),
            "uniswap" | "uni" => Ok(YieldSource::Uniswap),
            "yearn" | "ykong" | "kong" => Ok(YieldSource::Yearn),
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
            YieldSource::Yearn => write!(f, "yearn"),
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
    /// Data source that produced this row (curve, llama, uniswap, yearn)
    #[serde(default)]
    pub source: String,
}

/// Maximum plausible TVL for a single pool/vault (USD). Subgraph TVL for
/// spam-token pools is trivially manipulated (values up to 1e34 observed);
/// anything above this is treated as unknown.
pub const MAX_PLAUSIBLE_TVL_USD: f64 = 2.0e10;

/// Returns the TVL if it is finite, positive and plausible.
#[must_use]
pub fn plausible_tvl(tvl: Option<f64>) -> Option<f64> {
    tvl.filter(|t| t.is_finite() && *t > 0.0 && *t <= MAX_PLAUSIBLE_TVL_USD)
}

/// APYs beyond this magnitude (percent) are treated as data errors
/// (e.g. Curve pools with ~$0 TVL report APYs around 1e71%).
pub const MAX_PLAUSIBLE_APY_PCT: f64 = 100_000.0;

/// Returns the APY if it is finite and within +/- [`MAX_PLAUSIBLE_APY_PCT`].
#[must_use]
pub fn plausible_apy(apy: Option<f64>) -> Option<f64> {
    apy.filter(|a| a.is_finite() && a.abs() <= MAX_PLAUSIBLE_APY_PCT)
}

/// Minimum TVL for Curve pools to be listed when TVL is known
const CURVE_MIN_TVL_USD: f64 = 1_000.0;

/// Filters applied to every source before aggregation
#[derive(Debug, Clone, Default)]
pub struct YieldFilters<'a> {
    /// Project filter (matches `project` exactly, or as a prefix before `-`,
    /// e.g. `uniswap` matches `uniswap-v3`)
    pub project: Option<&'a str>,
    /// Minimum total APY (percent)
    pub min_apy: Option<f64>,
}

impl YieldFilters<'_> {
    /// Whether a yield row passes the filters
    #[must_use]
    pub fn matches(&self, y: &NormalizedYield) -> bool {
        if let Some(p) = self.project {
            let proj = y.project.to_lowercase();
            let p = p.to_lowercase();
            if proj != p && !proj.starts_with(&format!("{p}-")) {
                return false;
            }
        }
        if let Some(min) = self.min_apy {
            if y.apy_total.unwrap_or(f64::NEG_INFINITY) < min {
                return false;
            }
        }
        true
    }
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
    /// Number of vaults from Yearn (Kong API)
    pub yearn_vaults: usize,
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
    /// Lend APY in percent (for suppliers)
    pub lend_apy: Option<f64>,
    /// Borrow APY in percent (cost for borrowers)
    pub borrow_apy: Option<f64>,
    /// Utilization rate
    pub utilization: Option<f64>,
    /// Total assets in vault (USD)
    pub total_assets_usd: Option<f64>,
}

/// Parse `/getBaseApys/{chain}` data into (address, weekly-or-daily APY %).
///
/// The API returns `{"baseApys": [{"address", "latestDailyApyPcent",
/// "latestWeeklyApyPcent", ...}]}`; a legacy `{address: apy}` map is also
/// accepted.
fn parse_curve_base_apys(data: &serde_json::Value) -> Vec<(String, f64)> {
    if let Some(list) = data.get("baseApys").and_then(|v| v.as_array()) {
        return list
            .iter()
            .filter_map(|e| {
                let address = e.get("address")?.as_str()?.to_string();
                let apy = e
                    .get("latestWeeklyApyPcent")
                    .and_then(serde_json::Value::as_f64)
                    .or_else(|| {
                        e.get("latestDailyApyPcent")
                            .and_then(serde_json::Value::as_f64)
                    })?;
                Some((address, apy))
            })
            .collect();
    }
    data.as_object()
        .map(|obj| {
            obj.iter()
                .filter_map(|(a, v)| v.as_f64().map(|apy| (a.clone(), apy)))
                .collect()
        })
        .unwrap_or_default()
}

/// Pool metadata used to enrich Curve base APYs
#[derive(Debug, Clone, Default)]
struct CurvePoolMeta {
    name: Option<String>,
    coins: Vec<(String, String)>, // (address, symbol)
    tvl_usd: Option<f64>,
}

fn curve_pool_meta(pools: &[crv::pools::Pool]) -> std::collections::HashMap<String, CurvePoolMeta> {
    pools
        .iter()
        .filter_map(|p| {
            let address = p.address()?.to_lowercase();
            let coins = p
                .coins()
                .map(|cs| {
                    cs.iter()
                        .filter_map(|c| {
                            Some((
                                c.get("address")?.as_str()?.to_lowercase(),
                                c.get("symbol")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("?")
                                    .to_string(),
                            ))
                        })
                        .collect()
                })
                .unwrap_or_default();
            Some((
                address,
                CurvePoolMeta {
                    name: p.name().map(str::to_string),
                    coins,
                    tvl_usd: p.usd_total(),
                },
            ))
        })
        .collect()
}

/// Fetch yields from Curve (base APYs from the volumes API, enriched with
/// pool names/coins/TVL from `getPools`)
async fn fetch_curve_yields(chain: &str) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    let client = match crv::Client::new() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("curve", e.to_string(), measure.elapsed_ms());
        }
    };

    let volumes = client.volumes();
    let pools_api = client.pools();
    let (apys, pools) = tokio::join!(
        volumes.get_base_apys(chain),
        pools_api.get_all_on_chain(chain)
    );

    match apys {
        Ok(response) => {
            // Metadata is best-effort; APYs are still useful without it
            let meta = pools
                .map(|r| curve_pool_meta(&r.data.pool_data))
                .unwrap_or_default();

            let yields: Vec<NormalizedYield> = parse_curve_base_apys(&response.data)
                .into_iter()
                .filter_map(|(address, apy)| {
                    let m = meta
                        .get(&address.to_lowercase())
                        .cloned()
                        .unwrap_or_default();
                    // Skip dust/empty pools: their base APYs are meaningless
                    // (tiny TVL denominators produce values like 1e71%).
                    if m.tvl_usd.is_some_and(|t| t < CURVE_MIN_TVL_USD) {
                        return None;
                    }
                    let apy = plausible_apy(Some(apy));
                    let symbols: Vec<String> = m.coins.iter().map(|(_, s)| s.clone()).collect();
                    let symbol = if symbols.is_empty() {
                        m.name.clone().unwrap_or_else(|| {
                            format!("Curve Pool {}", &address[..address.len().min(10)])
                        })
                    } else {
                        symbols.join("/")
                    };
                    let stablecoin = (!symbols.is_empty()).then(|| is_stablecoin_pool(&symbols));
                    Some(NormalizedYield {
                        pool_id: address.clone(),
                        symbol,
                        project: "curve".to_string(),
                        chain: chain.to_string(),
                        apy_base: apy,
                        apy_reward: None, // Would need gauge data
                        apy_total: apy,
                        tvl_usd: plausible_tvl(m.tvl_usd),
                        underlying_tokens: m.coins.iter().map(|(a, _)| a.clone()).collect(),
                        stablecoin,
                        url: Some(format!(
                            "https://curve.finance/dex/{}/pools/{}",
                            chain, address
                        )),
                        source: "curve".to_string(),
                    })
                })
                .collect();

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
                    source: "llama".to_string(),
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

/// Number of completed days used to annualize Uniswap fee APYs
const UNISWAP_APY_WINDOW_DAYS: u32 = 7;

/// Number of pools (by recent volume) fetched per Uniswap version
const UNISWAP_TOP_POOLS: u32 = 100;

/// Uniswap V4 `LPFeeLibrary.DYNAMIC_FEE_FLAG` (0x800000) as stored in `feeTier`
const V4_DYNAMIC_FEE_FLAG: &str = "8388608";

/// Minimum TVL for a Uniswap pool to be listed (filters dust/spam pools)
const UNISWAP_MIN_TVL_USD: f64 = 10_000.0;

/// Aggregate daily stats per pool into annualized fee APYs.
///
/// APY = (sum of daily fees over the window / `window_days`) * 365 / TVL,
/// where TVL is the latest day's `tvlUSD`. Pools with implausible TVL
/// (spam) or TVL below [`UNISWAP_MIN_TVL_USD`] are skipped.
fn uniswap_yields_from_stats(
    stats: &[unswp::PoolDayStats],
    project: &str,
    chain: &str,
    window_days: u32,
) -> Vec<NormalizedYield> {
    use std::collections::HashMap;

    let mut by_pool: HashMap<&str, Vec<&unswp::PoolDayStats>> = HashMap::new();
    for s in stats {
        by_pool.entry(s.pool_id.as_str()).or_default().push(s);
    }

    let mut yields: Vec<NormalizedYield> = by_pool
        .into_values()
        .filter_map(|days| {
            let latest = days.iter().max_by_key(|d| d.date)?;
            let tvl = plausible_tvl(Some(latest.tvl_usd))?;
            if tvl < UNISWAP_MIN_TVL_USD {
                return None;
            }
            // V4 dynamic-fee pools store the 0x800000 flag as feeTier; the
            // subgraph derives feesUSD from it, so their fee data is invalid.
            let dynamic_fee = latest
                .fee_tier
                .as_deref()
                .is_some_and(|ft| ft == V4_DYNAMIC_FEE_FLAG);
            let apy = if dynamic_fee {
                None
            } else {
                let fees: f64 = days.iter().map(|d| d.fees_usd).sum();
                let avg_daily_fees = fees / f64::from(window_days.max(1));
                plausible_apy(Some(avg_daily_fees * 365.0 / tvl * 100.0))
            };

            let fee_label = match &latest.fee_tier {
                _ if dynamic_fee => "dynamic fee".to_string(),
                Some(ft) => format!("{}%", ft.parse::<f64>().unwrap_or(0.0) / 10_000.0),
                None => "0.3%".to_string(),
            };
            let hooks_note = latest
                .hooks
                .as_deref()
                .filter(|h| !h.is_empty() && *h != "0x0000000000000000000000000000000000000000")
                .map(|_| " [hooks]")
                .unwrap_or("");
            let underlying = vec![latest.token0_symbol.clone(), latest.token1_symbol.clone()];
            let stablecoin = is_stablecoin_pool(&underlying);
            let url = match project {
                "uniswap-v2" => format!(
                    "https://app.uniswap.org/explore/pools/{chain}/{}",
                    latest.pool_id
                ),
                _ => format!(
                    "https://app.uniswap.org/explore/pools/{chain}/{}",
                    latest.pool_id
                ),
            };

            Some(NormalizedYield {
                pool_id: latest.pool_id.clone(),
                symbol: format!(
                    "{}/{} ({}){}",
                    latest.token0_symbol, latest.token1_symbol, fee_label, hooks_note
                ),
                project: project.to_string(),
                chain: chain.to_string(),
                apy_base: apy,
                apy_reward: None,
                apy_total: apy,
                tvl_usd: Some(tvl),
                underlying_tokens: underlying,
                stablecoin: Some(stablecoin),
                url: Some(url),
                source: "uniswap".to_string(),
            })
        })
        .collect();

    sort_by_apy_desc(&mut yields);
    yields
}

fn sort_by_apy_desc(yields: &mut [NormalizedYield]) {
    yields.sort_by(|a, b| {
        b.apy_total
            .unwrap_or(0.0)
            .partial_cmp(&a.apy_total.unwrap_or(0.0))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Subgraph config for a Uniswap version on a chain (None if unsupported)
fn uniswap_subgraph_config(
    version: unswp::UniswapVersion,
    chain: &str,
    api_key: &str,
) -> Option<unswp::SubgraphConfig> {
    use unswp::{subgraph_ids, SubgraphConfig, UniswapVersion};
    let chain = chain.to_lowercase();
    match version {
        UniswapVersion::V2 => matches!(chain.as_str(), "ethereum" | "mainnet" | "eth")
            .then(|| SubgraphConfig::mainnet_v2(api_key)),
        UniswapVersion::V3 => match chain.as_str() {
            "ethereum" | "mainnet" | "eth" => Some(SubgraphConfig::mainnet_v3(api_key)),
            "arbitrum" | "arb" => Some(SubgraphConfig::arbitrum_v3(api_key)),
            "optimism" | "op" => Some(SubgraphConfig::optimism_v3(api_key)),
            "polygon" | "matic" => {
                Some(SubgraphConfig::mainnet_v3(api_key).with_subgraph_id(subgraph_ids::POLYGON_V3))
            }
            "base" => Some(SubgraphConfig::base_v3(api_key)),
            "bsc" | "binance" => {
                Some(SubgraphConfig::mainnet_v3(api_key).with_subgraph_id(subgraph_ids::BSC_V3))
            }
            _ => None,
        },
        UniswapVersion::V4 => match chain.as_str() {
            "ethereum" | "mainnet" | "eth" => Some(SubgraphConfig::mainnet_v4(api_key)),
            "arbitrum" | "arb" => Some(SubgraphConfig::arbitrum_v4(api_key)),
            "base" => Some(SubgraphConfig::base_v4(api_key)),
            "polygon" | "matic" => {
                Some(SubgraphConfig::mainnet_v4(api_key).with_subgraph_id(subgraph_ids::POLYGON_V4))
            }
            _ => None,
        },
    }
}

/// Fetch fee APYs for one Uniswap version from recent daily data
async fn fetch_uniswap_version_yields(
    version: unswp::UniswapVersion,
    chain: &str,
    api_key: &str,
) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();
    let project = match version {
        unswp::UniswapVersion::V2 => "uniswap-v2",
        unswp::UniswapVersion::V3 => "uniswap-v3",
        unswp::UniswapVersion::V4 => "uniswap-v4",
    };

    let Some(config) = uniswap_subgraph_config(version, chain, api_key) else {
        return SourceResult::error(
            project,
            format!("Unsupported chain for {project} subgraph: {chain}"),
            measure.elapsed_ms(),
        );
    };
    let client = match unswp::SubgraphClient::new(config) {
        Ok(c) => c,
        Err(e) => return SourceResult::error(project, e.to_string(), measure.elapsed_ms()),
    };

    match client
        .get_recent_day_stats(UNISWAP_TOP_POOLS, UNISWAP_APY_WINDOW_DAYS, now_secs())
        .await
    {
        Ok(stats) => SourceResult::success(
            project,
            uniswap_yields_from_stats(&stats, project, chain, UNISWAP_APY_WINDOW_DAYS),
            measure.elapsed_ms(),
        ),
        Err(e) => SourceResult::error(project, e.to_string(), measure.elapsed_ms()),
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
        fetch_uniswap_version_yields(unswp::UniswapVersion::V2, chain, &api_key),
        fetch_uniswap_version_yields(unswp::UniswapVersion::V3, chain, &api_key),
        fetch_uniswap_version_yields(unswp::UniswapVersion::V4, chain, &api_key),
    );

    // Combine results
    let mut all_yields: Vec<NormalizedYield> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    if let Some(yields) = v2_result.data {
        all_yields.extend(yields);
    } else if let Some(e) = v2_result.error {
        // V2 error is expected for non-mainnet chains, don't treat as error
        if !e.contains("Unsupported chain") {
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

/// Fetch yields from Yearn vaults via Kong GraphQL API
async fn fetch_yearn_yields(chain_id: Option<u64>) -> SourceResult<Vec<NormalizedYield>> {
    let measure = LatencyMeasure::start();

    let client = match ykong::Client::new() {
        Ok(c) => c,
        Err(e) => {
            return SourceResult::error("yearn", e.to_string(), measure.elapsed_ms());
        }
    };

    // Fetch vaults - optionally filter by chain
    let vaults_result = if let Some(cid) = chain_id {
        client.vaults().by_chain(cid).await
    } else {
        client.vaults().list(None).await
    };

    match vaults_result {
        Ok(vaults) => {
            let mut yields: Vec<NormalizedYield> = Vec::new();

            for vault in vaults {
                // Skip shutdown vaults
                if vault.is_shutdown == Some(true) || vault.emergency_shutdown == Some(true) {
                    continue;
                }

                // Get APY - prefer net APY, fall back to weekly/monthly.
                // Kong reports APYs as fractions (0.0362 = 3.62%); convert
                // to percent like every other source.
                let apy_total = vault
                    .apy
                    .as_ref()
                    .and_then(|a| a.net.or(a.weekly_net).or(a.monthly_net))
                    .map(|f| f * 100.0);

                // Skip vaults with no APY data
                if apy_total.is_none() {
                    continue;
                }

                // Get TVL from the tvl field
                let tvl_usd = plausible_tvl(vault.tvl.as_ref().and_then(|t| t.close));

                // Determine chain name from chain_id
                let chain = match vault.chain_id {
                    1 => "ethereum".to_string(),
                    10 => "optimism".to_string(),
                    137 => "polygon".to_string(),
                    250 => "fantom".to_string(),
                    42161 => "arbitrum".to_string(),
                    43114 => "avalanche".to_string(),
                    8453 => "base".to_string(),
                    _ => format!("chain-{}", vault.chain_id),
                };

                // Build the symbol - prefer asset symbol if available
                let symbol = vault
                    .symbol
                    .clone()
                    .or_else(|| vault.asset.as_ref().and_then(|a| a.symbol.clone()))
                    .unwrap_or_else(|| {
                        vault
                            .name
                            .clone()
                            .unwrap_or_else(|| vault.address[..10].to_string())
                    });

                // Get underlying token
                let underlying = vault
                    .asset
                    .as_ref()
                    .and_then(|a| a.symbol.clone())
                    .map(|s| vec![s])
                    .unwrap_or_default();

                // Check if stablecoin based on underlying
                let stablecoin = if underlying.is_empty() {
                    None
                } else {
                    Some(is_stablecoin_pool(&underlying))
                };

                // Build URL to Yearn UI
                let url = Some(format!(
                    "https://yearn.fi/v3/{}/{}",
                    vault.chain_id, vault.address
                ));

                // Determine vault version for project name
                let project = if vault.v3 == Some(true) {
                    "yearn-v3".to_string()
                } else {
                    "yearn-v2".to_string()
                };

                yields.push(NormalizedYield {
                    pool_id: vault.address.clone(),
                    symbol,
                    project,
                    chain,
                    apy_base: apy_total, // Yearn APY is typically net
                    apy_reward: None,    // Yearn doesn't separate rewards
                    apy_total,
                    tvl_usd,
                    underlying_tokens: underlying,
                    stablecoin,
                    url,
                    source: "yearn".to_string(),
                });
            }

            // Sort by APY descending
            yields.sort_by(|a, b| {
                b.apy_total
                    .unwrap_or(0.0)
                    .partial_cmp(&a.apy_total.unwrap_or(0.0))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            SourceResult::success("yearn", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("yearn", e.to_string(), measure.elapsed_ms()),
    }
}

/// Map chain name to Yearn chain ID
fn chain_name_to_yearn_id(chain: &str) -> Option<u64> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "mainnet" | "eth" => Some(1),
        "optimism" | "op" => Some(10),
        "polygon" | "matic" => Some(137),
        "fantom" | "ftm" => Some(250),
        "arbitrum" | "arb" => Some(42161),
        "avalanche" | "avax" => Some(43114),
        "base" => Some(8453),
        _ => None,
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
                    collateral_symbol: v
                        .assets
                        .as_ref()
                        .and_then(|a| a.collateral.as_ref())
                        .and_then(|t| t.symbol.clone()),
                    borrowed_symbol: v
                        .assets
                        .as_ref()
                        .and_then(|a| a.borrowed.as_ref())
                        .and_then(|t| t.symbol.clone()),
                    // Prefer the API's percent fields; the plain lendApy/
                    // borrowApy values are fractions (0.1897 = 18.97%).
                    lend_apy: v
                        .rates
                        .as_ref()
                        .and_then(|r| r.lend_apy_pcent.or(r.lend_apy.map(|f| f * 100.0))),
                    borrow_apy: v
                        .rates
                        .as_ref()
                        .and_then(|r| r.borrow_apy_pcent.or(r.borrow_apy.map(|f| f * 100.0))),
                    utilization: v.total_supplied.as_ref().and_then(|s| {
                        v.borrowed.as_ref().and_then(|b| match (s.total, b.total) {
                            (Some(supply), Some(borrow)) if supply > 0.0 => Some(borrow / supply),
                            _ => None,
                        })
                    }),
                    total_assets_usd: v.total_supplied.and_then(|s| s.usd_total),
                })
                .collect();

            SourceResult::success("curve-lending", yields, measure.elapsed_ms())
        }
        Err(e) => SourceResult::error("curve-lending", e.to_string(), measure.elapsed_ms()),
    }
}

/// Fetch aggregated yields from all sources
///
/// `filters` (project, min APY) are applied to every source's rows before
/// counts and aggregate statistics are computed.
pub async fn fetch_yields_aggregated(
    chain: Option<&str>,
    filters: &YieldFilters<'_>,
    sources: YieldSource,
) -> AggregatedResult<Vec<NormalizedYield>, YieldAggregation> {
    let project = filters.project;
    let start = std::time::Instant::now();

    let futures: Vec<_> = match sources {
        YieldSource::All => {
            let curve_chain = chain.unwrap_or("ethereum");
            let uniswap_chain = chain.unwrap_or("ethereum");
            let yearn_chain_id = chain.and_then(chain_name_to_yearn_id);
            vec![
                Box::pin(fetch_curve_yields(curve_chain))
                    as std::pin::Pin<Box<dyn std::future::Future<Output = _> + Send>>,
                Box::pin(fetch_llama_yields(chain, project)),
                Box::pin(fetch_uniswap_yields(uniswap_chain)),
                Box::pin(fetch_yearn_yields(yearn_chain_id)),
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
        YieldSource::Yearn => {
            let yearn_chain_id = chain.and_then(chain_name_to_yearn_id);
            vec![Box::pin(fetch_yearn_yields(yearn_chain_id))]
        }
    };

    let mut results = join_all(futures).await;

    // Apply filters to every source (not just DefiLlama / table output)
    for result in &mut results {
        if let Some(data) = result.data.as_mut() {
            data.retain(|y| filters.matches(y));
        }
    }

    // Combine all yields
    let mut all_yields: Vec<NormalizedYield> = Vec::new();
    let mut curve_count = 0;
    let mut llama_count = 0;
    let mut uniswap_count = 0;
    let mut yearn_count = 0;

    for result in &results {
        if let Some(yields) = &result.data {
            if result.source == "curve" {
                curve_count = yields.len();
            } else if result.source == "llama" {
                llama_count = yields.len();
            } else if result.source == "uniswap" {
                uniswap_count = yields.len();
            } else if result.source == "yearn" {
                yearn_count = yields.len();
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
        let sum: f64 = all_yields
            .iter()
            .filter_map(|y| plausible_tvl(y.tvl_usd))
            .sum();
        if sum > 0.0 {
            Some(sum)
        } else {
            None
        }
    };

    // Count Uniswap pools by version - only rows from the Uniswap source
    // (DefiLlama also lists uniswap-v2/v3 projects)
    let uniswap_rows = || all_yields.iter().filter(|y| y.source == "uniswap");
    let uniswap_v2_count = uniswap_rows().filter(|y| y.project == "uniswap-v2").count();
    let uniswap_v3_count = uniswap_rows().filter(|y| y.project == "uniswap-v3").count();
    let uniswap_v4_count = uniswap_rows().filter(|y| y.project == "uniswap-v4").count();

    let aggregation = YieldAggregation {
        total_pools,
        curve_pools: curve_count,
        llama_pools: llama_count,
        uniswap_pools: uniswap_count,
        uniswap_v2_pools: uniswap_v2_count,
        uniswap_v3_pools: uniswap_v3_count,
        uniswap_v4_pools: uniswap_v4_count,
        yearn_vaults: yearn_count,
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

/// Compare Curve pool yields between the Curve API and DefiLlama
///
/// DefiLlama identifies pools by UUID, so pools are matched by their
/// (sorted) underlying token addresses; when several Curve pools share the
/// same coins, the one with the closest TVL is used.
pub async fn compare_curve_yields() -> Vec<YieldComparison> {
    let (llama_result, curve_result) = tokio::join!(
        fetch_llama_yields(Some("ethereum"), None),
        fetch_curve_yields("ethereum")
    );

    match (llama_result.data, curve_result.data) {
        (Some(llama), Some(curve)) => match_curve_yields(&llama, &curve),
        _ => Vec::new(),
    }
}

fn coin_key(tokens: &[String]) -> Option<String> {
    if tokens.len() < 2 {
        return None;
    }
    let mut t: Vec<String> = tokens.iter().map(|a| a.to_lowercase()).collect();
    t.sort();
    Some(t.join(","))
}

/// Match DefiLlama Curve rows to Curve API rows by coin set
fn match_curve_yields(
    llama: &[NormalizedYield],
    curve: &[NormalizedYield],
) -> Vec<YieldComparison> {
    use std::collections::HashMap;

    let mut by_coins: HashMap<String, Vec<&NormalizedYield>> = HashMap::new();
    for c in curve {
        if let Some(k) = coin_key(&c.underlying_tokens) {
            by_coins.entry(k).or_default().push(c);
        }
    }

    let mut comparisons: Vec<YieldComparison> = llama
        .iter()
        .filter(|l| {
            let p = l.project.to_lowercase();
            p == "curve" || p == "curve-dex"
        })
        .filter_map(|l| {
            let candidates = by_coins.get(&coin_key(&l.underlying_tokens)?)?;
            let best = candidates.iter().min_by(|a, b| {
                let d = |c: &&&NormalizedYield| match (c.tvl_usd, l.tvl_usd) {
                    (Some(x), Some(y)) => (x - y).abs(),
                    _ => f64::MAX,
                };
                d(a).total_cmp(&d(b))
            })?;
            let curve_apy = best.apy_total?;
            let llama_apy = l.apy_base.or(l.apy_total).unwrap_or(0.0);
            let diff = curve_apy - llama_apy;
            let diff_pct = if llama_apy != 0.0 {
                (diff / llama_apy) * 100.0
            } else {
                0.0
            };
            Some(YieldComparison {
                pool_id: best.pool_id.clone(),
                symbol: l.symbol.clone(),
                chain: l.chain.clone(),
                curve_apy,
                llama_apy,
                difference: diff,
                difference_pct: diff_pct,
            })
        })
        .collect();

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

#[cfg(test)]
mod tests {
    use super::*;

    fn day(pool: &str, date: i64, fees: f64, tvl: f64) -> unswp::PoolDayStats {
        unswp::PoolDayStats {
            pool_id: pool.to_string(),
            token0_symbol: "USDC".into(),
            token1_symbol: "WETH".into(),
            fee_tier: Some("500".into()),
            hooks: None,
            date,
            volume_usd: fees * 2000.0,
            fees_usd: fees,
            tvl_usd: tvl,
        }
    }

    #[test]
    fn curve_base_apys_parse_live_shape() {
        let data = serde_json::json!({"baseApys":[
            {"address":"0xbEbc","latestDailyApyPcent":1.0,"latestWeeklyApyPcent":2.5},
            {"address":"0xDC24","latestDailyApyPcent":3.0,"latestWeeklyApyPcent":null}
        ]});
        let parsed = parse_curve_base_apys(&data);
        assert_eq!(
            parsed,
            vec![("0xbEbc".to_string(), 2.5), ("0xDC24".to_string(), 3.0)]
        );
        // legacy map shape still accepted
        let legacy = serde_json::json!({"0xabc": 4.0});
        assert_eq!(
            parse_curve_base_apys(&legacy),
            vec![("0xabc".to_string(), 4.0)]
        );
    }

    #[test]
    fn uniswap_apy_annualizes_daily_fees() {
        // $1,000/day fees on $10M TVL over 7 days => 3.65% APY
        let stats: Vec<_> = (0..7)
            .map(|d| day("0xpool", d * 86_400, 1_000.0, 10_000_000.0))
            .collect();
        let y = uniswap_yields_from_stats(&stats, "uniswap-v3", "ethereum", 7);
        assert_eq!(y.len(), 1);
        assert!(
            (y[0].apy_total.unwrap() - 3.65).abs() < 1e-9,
            "{:?}",
            y[0].apy_total
        );
        assert_eq!(y[0].symbol, "USDC/WETH (0.05%)");
        assert_eq!(y[0].source, "uniswap");
    }

    #[test]
    fn uniswap_missing_days_count_as_zero_fees() {
        let stats = vec![day("0xpool", 0, 7_000.0, 10_000_000.0)];
        let y = uniswap_yields_from_stats(&stats, "uniswap-v3", "ethereum", 7);
        assert!((y[0].apy_total.unwrap() - 3.65).abs() < 1e-9);
    }

    #[test]
    fn uniswap_spam_and_dust_pools_are_dropped() {
        let stats = vec![
            day("spam", 0, 10.0, 3.5e34),
            day("dust", 0, 10.0, 500.0),
            day("ok", 0, 10.0, 1_000_000.0),
        ];
        let y = uniswap_yields_from_stats(&stats, "uniswap-v4", "ethereum", 7);
        assert_eq!(y.len(), 1);
        assert_eq!(y[0].pool_id, "ok");
    }

    #[test]
    fn uniswap_v4_dynamic_fee_pools_have_no_apy() {
        let mut d = day("dyn", 0, 1e9, 3_000_000.0);
        d.fee_tier = Some("8388608".into());
        let y = uniswap_yields_from_stats(&[d], "uniswap-v4", "ethereum", 7);
        assert_eq!(y[0].apy_total, None);
        assert!(y[0].symbol.contains("dynamic fee"), "{}", y[0].symbol);
    }

    #[test]
    fn plausible_apy_bounds() {
        assert_eq!(plausible_apy(Some(1.5e71)), None);
        assert_eq!(plausible_apy(Some(-16.3)), Some(-16.3));
        assert_eq!(plausible_apy(Some(f64::INFINITY)), None);
    }

    #[test]
    fn plausible_tvl_bounds() {
        assert_eq!(plausible_tvl(Some(3.5e34)), None);
        assert_eq!(plausible_tvl(Some(f64::NAN)), None);
        assert_eq!(plausible_tvl(Some(0.0)), None);
        assert_eq!(plausible_tvl(Some(1e6)), Some(1e6));
    }

    fn row(project: &str, apy: f64) -> NormalizedYield {
        NormalizedYield {
            pool_id: "p".into(),
            symbol: "s".into(),
            project: project.into(),
            chain: "ethereum".into(),
            apy_base: Some(apy),
            apy_reward: None,
            apy_total: Some(apy),
            tvl_usd: None,
            underlying_tokens: vec![],
            stablecoin: None,
            url: None,
            source: "llama".into(),
        }
    }

    #[test]
    fn filters_apply_project_prefix_and_min_apy() {
        let f = YieldFilters {
            project: Some("uniswap"),
            min_apy: Some(1.0),
        };
        assert!(f.matches(&row("uniswap-v3", 2.0)));
        assert!(!f.matches(&row("uniswap-v3", 0.5)));
        assert!(!f.matches(&row("aave-v3", 5.0)));
        let exact = YieldFilters {
            project: Some("aave-v3"),
            min_apy: None,
        };
        assert!(exact.matches(&row("AAVE-V3", 0.0)));
        assert!(!exact.matches(&row("aave-v2", 9.0)));
    }

    #[test]
    fn curve_llama_matching_uses_coin_sets() {
        let mut curve = row("curve", 2.0);
        curve.pool_id = "0xbebc".into();
        curve.underlying_tokens = vec!["0xB".into(), "0xa".into()];
        curve.tvl_usd = Some(100.0);
        let mut llama = row("curve-dex", 1.5);
        llama.underlying_tokens = vec!["0xA".into(), "0xb".into()];
        llama.tvl_usd = Some(110.0);
        let cmp = match_curve_yields(&[llama], &[curve]);
        assert_eq!(cmp.len(), 1);
        assert_eq!(cmp[0].pool_id, "0xbebc");
        assert!((cmp[0].difference - 0.5).abs() < 1e-9);
    }
}
