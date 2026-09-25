//! Recent per-day pool statistics (for fee APY estimation)
//!
//! Cumulative `feesUSD`/`volumeUSD` on the pool entity are all-time totals and
//! cannot be annualized. These queries fetch completed daily snapshots
//! (`poolDayDatas` for V3/V4, `pairDayDatas` for V2) instead.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::subgraph::{SubgraphClient, UniswapVersion};

/// V2 swap fee (0.3%), used to derive fees from `dailyVolumeUSD`
pub const V2_FEE_RATE: f64 = 0.003;

/// One completed day of activity for a pool, normalized across versions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PoolDayStats {
    /// Pool (V3/V4 id) or pair (V2) address
    pub pool_id: String,
    /// Token0 symbol
    pub token0_symbol: String,
    /// Token1 symbol
    pub token1_symbol: String,
    /// Fee tier in hundredths of a bip (V3/V4), `None` for V2 (0.3%)
    pub fee_tier: Option<String>,
    /// V4 hooks address
    pub hooks: Option<String>,
    /// Day start (unix seconds)
    pub date: i64,
    /// Volume in USD for the day
    pub volume_usd: f64,
    /// Fees in USD for the day
    pub fees_usd: f64,
    /// TVL in USD at the end of the day
    pub tvl_usd: f64,
}

#[derive(Deserialize)]
struct Sym {
    symbol: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPoolRef {
    id: String,
    token0: Sym,
    token1: Sym,
    #[serde(default)]
    fee_tier: Option<String>,
    #[serde(default)]
    hooks: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPoolDay {
    date: i64,
    pool: RawPoolRef,
    #[serde(rename = "volumeUSD")]
    volume_usd: String,
    #[serde(rename = "feesUSD")]
    fees_usd: String,
    #[serde(rename = "tvlUSD")]
    tvl_usd: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPairDay {
    date: i64,
    pair_address: String,
    token0: Sym,
    token1: Sym,
    #[serde(rename = "reserveUSD")]
    reserve_usd: String,
    #[serde(rename = "dailyVolumeUSD")]
    daily_volume_usd: String,
}

fn num(s: &str) -> f64 {
    s.parse::<f64>()
        .ok()
        .filter(|v| v.is_finite())
        .unwrap_or(0.0)
}

impl From<RawPoolDay> for PoolDayStats {
    fn from(r: RawPoolDay) -> Self {
        Self {
            pool_id: r.pool.id,
            token0_symbol: r.pool.token0.symbol,
            token1_symbol: r.pool.token1.symbol,
            fee_tier: r.pool.fee_tier,
            hooks: r.pool.hooks,
            date: r.date,
            volume_usd: num(&r.volume_usd),
            fees_usd: num(&r.fees_usd),
            tvl_usd: num(&r.tvl_usd),
        }
    }
}

impl From<RawPairDay> for PoolDayStats {
    fn from(r: RawPairDay) -> Self {
        let volume = num(&r.daily_volume_usd);
        Self {
            pool_id: r.pair_address,
            token0_symbol: r.token0.symbol,
            token1_symbol: r.token1.symbol,
            fee_tier: None,
            hooks: None,
            date: r.date,
            volume_usd: volume,
            fees_usd: volume * V2_FEE_RATE,
            tvl_usd: num(&r.reserve_usd),
        }
    }
}

/// Start of the UTC day containing `ts`
#[must_use]
pub fn day_start(ts: u64) -> i64 {
    i64::try_from(ts - ts % 86_400).unwrap_or(i64::MAX)
}

/// Build the "top pools by recent volume" query (step 1)
fn top_recent_query(version: UniswapVersion, first: u32, since: i64) -> String {
    match version {
        UniswapVersion::V2 => format!(
            "query {{ pairDayDatas(first: {first}, orderBy: dailyVolumeUSD, orderDirection: desc, \
             where: {{ date_gte: {since} }}) {{ pairAddress }} }}"
        ),
        UniswapVersion::V3 | UniswapVersion::V4 => format!(
            "query {{ poolDayDatas(first: {first}, orderBy: volumeUSD, orderDirection: desc, \
             where: {{ date_gte: {since} }}) {{ pool {{ id }} }} }}"
        ),
    }
}

/// Build the per-day stats query for a set of pools (step 2)
fn day_stats_query(version: UniswapVersion, ids: &[String], from: i64, to: i64) -> String {
    let ids = ids
        .iter()
        .map(|id| format!("\"{}\"", id.to_lowercase()))
        .collect::<Vec<_>>()
        .join(",");
    match version {
        UniswapVersion::V2 => format!(
            "query {{ pairDayDatas(first: 1000, orderBy: date, orderDirection: desc, \
             where: {{ pairAddress_in: [{ids}], date_gte: {from}, date_lt: {to} }}) \
             {{ date pairAddress token0 {{ symbol }} token1 {{ symbol }} reserveUSD dailyVolumeUSD }} }}"
        ),
        UniswapVersion::V3 => format!(
            "query {{ poolDayDatas(first: 1000, orderBy: date, orderDirection: desc, \
             where: {{ pool_in: [{ids}], date_gte: {from}, date_lt: {to} }}) \
             {{ date pool {{ id token0 {{ symbol }} token1 {{ symbol }} feeTier }} volumeUSD feesUSD tvlUSD }} }}"
        ),
        UniswapVersion::V4 => format!(
            "query {{ poolDayDatas(first: 1000, orderBy: date, orderDirection: desc, \
             where: {{ pool_in: [{ids}], date_gte: {from}, date_lt: {to} }}) \
             {{ date pool {{ id token0 {{ symbol }} token1 {{ symbol }} feeTier hooks }} volumeUSD feesUSD tvlUSD }} }}"
        ),
    }
}

impl SubgraphClient {
    /// Daily stats for the `top` pools by recent volume over the last `days`
    /// **completed** UTC days (the current partial day is excluded).
    ///
    /// Pools are ranked by recent daily volume rather than TVL because
    /// subgraph TVL is trivially manipulated by spam-token pools.
    pub async fn get_recent_day_stats(
        &self,
        top: u32,
        days: u32,
        now_ts: u64,
    ) -> Result<Vec<PoolDayStats>> {
        #[derive(Deserialize)]
        struct PoolId {
            id: String,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TopResp {
            #[serde(default)]
            pool_day_datas: Vec<TopPool>,
            #[serde(default)]
            pair_day_datas: Vec<TopPair>,
        }
        #[derive(Deserialize)]
        struct TopPool {
            pool: PoolId,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct TopPair {
            pair_address: String,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PoolDays {
            pool_day_datas: Vec<RawPoolDay>,
        }
        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct PairDays {
            pair_day_datas: Vec<RawPairDay>,
        }

        let version = self.version();
        let today = day_start(now_ts);
        let from = today - i64::from(days) * 86_400;

        // Step 1: top pools by volume over the window (deduplicated)
        let top_resp: TopResp = self
            .query(&top_recent_query(
                version,
                top.saturating_mul(4).min(1000),
                from,
            ))
            .await?;
        let mut ids: Vec<String> = Vec::new();
        let candidates = top_resp
            .pool_day_datas
            .into_iter()
            .map(|p| p.pool.id)
            .chain(top_resp.pair_day_datas.into_iter().map(|p| p.pair_address));
        for id in candidates {
            if !ids.contains(&id) {
                ids.push(id);
            }
            if ids.len() >= top as usize {
                break;
            }
        }
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Step 2: all completed days in the window for those pools
        let query = day_stats_query(version, &ids, from, today);
        let stats = match version {
            UniswapVersion::V2 => {
                let r: PairDays = self.query(&query).await?;
                r.pair_day_datas.into_iter().map(Into::into).collect()
            }
            UniswapVersion::V3 | UniswapVersion::V4 => {
                let r: PoolDays = self.query(&query).await?;
                r.pool_day_datas.into_iter().map(Into::into).collect()
            }
        };
        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_start_rounds_down() {
        assert_eq!(day_start(86_400 * 3 + 5), 86_400 * 3);
    }

    #[test]
    fn queries_exclude_partial_day_and_use_version_fields() {
        let ids = vec!["0xABC".to_string()];
        let v3 = day_stats_query(UniswapVersion::V3, &ids, 100, 200);
        assert!(v3.contains("pool_in: [\"0xabc\"]") && v3.contains("date_lt: 200"));
        assert!(v3.contains("feesUSD") && !v3.contains("hooks"));
        let v4 = day_stats_query(UniswapVersion::V4, &ids, 100, 200);
        assert!(v4.contains("hooks"));
        let v2 = day_stats_query(UniswapVersion::V2, &ids, 100, 200);
        assert!(v2.contains("pairAddress_in") && v2.contains("dailyVolumeUSD"));
        assert!(top_recent_query(UniswapVersion::V2, 10, 5).contains("pairDayDatas"));
    }

    #[test]
    fn v2_fees_derive_from_volume() {
        let raw: RawPairDay = serde_json::from_str(
            r#"{"date":1,"pairAddress":"0xp","token0":{"symbol":"USDC"},"token1":{"symbol":"WETH"},
            "reserveUSD":"1000000","dailyVolumeUSD":"200000"}"#,
        )
        .unwrap();
        let s: PoolDayStats = raw.into();
        assert!((s.fees_usd - 600.0).abs() < 1e-9);
        assert!((s.tvl_usd - 1_000_000.0).abs() < 1e-9);
    }
}
