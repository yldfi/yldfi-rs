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
    /// Entity id (pagination cursor)
    #[serde(default)]
    id: String,
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
    /// Entity id (pagination cursor)
    #[serde(default)]
    id: String,
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

/// Build the "top pools by recent volume" query (step 1), over completed
/// days in `[since, until)`
fn top_recent_query(version: UniswapVersion, first: u32, since: i64, until: i64) -> String {
    match version {
        UniswapVersion::V2 => format!(
            "query {{ pairDayDatas(first: {first}, orderBy: dailyVolumeUSD, orderDirection: desc, \
             where: {{ date_gte: {since}, date_lt: {until} }}) {{ pairAddress }} }}"
        ),
        UniswapVersion::V3 | UniswapVersion::V4 => format!(
            "query {{ poolDayDatas(first: {first}, orderBy: volumeUSD, orderDirection: desc, \
             where: {{ date_gte: {since}, date_lt: {until} }}) {{ pool {{ id }} }} }}"
        ),
    }
}

/// Day rows to request in step 1 so `top` distinct pools can be found even
/// when each top pool fills every day of the window
fn top_candidate_rows(top: u32, days: u32) -> u32 {
    top.saturating_mul(days.max(1))
        .saturating_add(top)
        .min(1000)
}

/// Page size for the per-day stats query (The Graph caps `first` at 1000)
const DAY_STATS_PAGE: usize = 1000;

/// Safety cap on pages fetched for one window
const DAY_STATS_MAX_PAGES: usize = 50;

/// Build the per-day stats query and variables for a set of pools (step 2)
///
/// Pages are walked by `id_gt` (ordered by `id`), so windows larger than one
/// page (`top * days > 1000`) are fetched completely. Pool ids and the cursor
/// are sent as GraphQL variables, never interpolated into the query text.
fn day_stats_query(
    version: UniswapVersion,
    ids: &[String],
    from: i64,
    to: i64,
    after: Option<&str>,
) -> (String, serde_json::Value) {
    let ids: Vec<String> = ids.iter().map(|id| id.to_lowercase()).collect();
    let (entity, filter_type, ids_key, fields) = match version {
        UniswapVersion::V2 => (
            "pairDayDatas",
            "PairDayData_filter",
            "pairAddress_in",
            "id date pairAddress token0 { symbol } token1 { symbol } reserveUSD dailyVolumeUSD",
        ),
        UniswapVersion::V3 => (
            "poolDayDatas",
            "PoolDayData_filter",
            "pool_in",
            "id date pool { id token0 { symbol } token1 { symbol } feeTier } volumeUSD feesUSD tvlUSD",
        ),
        UniswapVersion::V4 => (
            "poolDayDatas",
            "PoolDayData_filter",
            "pool_in",
            "id date pool { id token0 { symbol } token1 { symbol } feeTier hooks } \
             volumeUSD feesUSD tvlUSD",
        ),
    };
    let mut filter = serde_json::json!({
        ids_key: ids,
        "date_gte": from,
        "date_lt": to,
    });
    if let Some(after) = after {
        filter["id_gt"] = after.into();
    }
    let query = format!(
        "query DayStats($where: {filter_type}) {{ {entity}(first: {DAY_STATS_PAGE}, \
         orderBy: id, orderDirection: asc, where: $where) {{ {fields} }} }}"
    );
    (query, serde_json::json!({ "where": filter }))
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
                top_candidate_rows(top, days),
                from,
                today,
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

        // Step 2: all completed days in the window for those pools, paged
        let mut stats: Vec<PoolDayStats> = Vec::new();
        let mut cursor: Option<String> = None;
        for _ in 0..DAY_STATS_MAX_PAGES {
            let (query, variables) = day_stats_query(version, &ids, from, today, cursor.as_deref());
            let (page_len, last_id) = match version {
                UniswapVersion::V2 => {
                    let r: PairDays = self.query_with_variables(&query, variables).await?;
                    let len = r.pair_day_datas.len();
                    let last = r.pair_day_datas.last().map(|d| d.id.clone());
                    stats.extend(r.pair_day_datas.into_iter().map(PoolDayStats::from));
                    (len, last)
                }
                UniswapVersion::V3 | UniswapVersion::V4 => {
                    let r: PoolDays = self.query_with_variables(&query, variables).await?;
                    let len = r.pool_day_datas.len();
                    let last = r.pool_day_datas.last().map(|d| d.id.clone());
                    stats.extend(r.pool_day_datas.into_iter().map(PoolDayStats::from));
                    (len, last)
                }
            };
            match last_id {
                Some(id) if page_len >= DAY_STATS_PAGE && !id.is_empty() => cursor = Some(id),
                _ => break,
            }
        }
        // Newest first, as before pagination was added
        stats.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.pool_id.cmp(&b.pool_id)));
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
        let (v3, vars) = day_stats_query(UniswapVersion::V3, &ids, 100, 200, None);
        assert!(v3.contains("$where: PoolDayData_filter") && !v3.contains("0xabc"));
        assert_eq!(
            vars,
            serde_json::json!({
                "where": { "pool_in": ["0xabc"], "date_gte": 100, "date_lt": 200 }
            })
        );
        assert!(v3.contains("feesUSD") && !v3.contains("hooks"));
        let (v4, _) = day_stats_query(UniswapVersion::V4, &ids, 100, 200, None);
        assert!(v4.contains("hooks"));
        let (v2, vars) = day_stats_query(UniswapVersion::V2, &ids, 100, 200, None);
        assert!(v2.contains("PairDayData_filter") && v2.contains("dailyVolumeUSD"));
        assert_eq!(
            vars["where"]["pairAddress_in"],
            serde_json::json!(["0xabc"])
        );
        let top = top_recent_query(UniswapVersion::V2, 10, 5, 9);
        assert!(top.contains("pairDayDatas") && top.contains("date_lt: 9"));
        // Each of the top pools may fill every day of the window
        assert_eq!(top_candidate_rows(10, 7), 80);
        assert_eq!(top_candidate_rows(500, 7), 1000);
    }

    #[test]
    fn day_stats_query_pages_by_id() {
        let ids = vec!["0xabc".to_string()];
        let (q, first) = day_stats_query(UniswapVersion::V3, &ids, 100, 200, None);
        assert!(q.contains("first: 1000") && q.contains("orderBy: id, orderDirection: asc"));
        assert!(first["where"].get("id_gt").is_none());
        let (_, next) = day_stats_query(UniswapVersion::V3, &ids, 100, 200, Some("0xabc-19000"));
        assert_eq!(next["where"]["id_gt"], "0xabc-19000");
    }

    #[tokio::test]
    async fn recent_day_stats_follow_pages() {
        use wiremock::matchers::{body_partial_json, method};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let day = |i: usize| {
            serde_json::json!({
                "id": format!("0xp-{i:05}"), "date": 86_400 * (i as i64 % 3),
                "pool": { "id": "0xp", "token0": { "symbol": "A" },
                          "token1": { "symbol": "B" }, "feeTier": "500" },
                "volumeUSD": "1", "feesUSD": "1", "tvlUSD": "1"
            })
        };
        // Step 1: top pools
        Mock::given(method("POST"))
            .and(body_partial_json(serde_json::json!({ "variables": null })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "poolDayDatas": [{ "pool": { "id": "0xp" } }] }
            })))
            .mount(&server)
            .await;
        // Step 2, page 2 (cursor = last id of page 1)
        Mock::given(method("POST"))
            .and(body_partial_json(serde_json::json!({
                "variables": { "where": { "id_gt": "0xp-00999" } }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "poolDayDatas": [day(1000)] }
            })))
            .with_priority(1)
            .mount(&server)
            .await;
        // Step 2, page 1 (full page)
        let page1: Vec<_> = (0..DAY_STATS_PAGE).map(day).collect();
        Mock::given(method("POST"))
            .and(body_partial_json(serde_json::json!({
                "variables": { "where": { "pool_in": ["0xp"] } }
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "data": { "poolDayDatas": page1 }
            })))
            .with_priority(2)
            .mount(&server)
            .await;

        let client = SubgraphClient::for_test_endpoint(&server.uri(), UniswapVersion::V3);
        let stats = client
            .get_recent_day_stats(1, 30, 86_400 * 40)
            .await
            .unwrap();
        assert_eq!(stats.len(), DAY_STATS_PAGE + 1);
        assert!(stats.windows(2).all(|w| w[0].date >= w[1].date));
        assert_eq!(server.received_requests().await.unwrap().len(), 3);
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
