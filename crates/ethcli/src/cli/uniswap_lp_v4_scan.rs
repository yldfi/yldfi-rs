//! Discovery of the Uniswap v4 positions an address owns
//!
//! The v4 PositionManager is an ERC-721 without enumeration, so the token ids
//! an address may hold are found from its `Transfer(from, to, tokenId)` logs
//! with `to` = the owner, from the PositionManager's deployment block up to the
//! valuation block (or, optionally, from the explorer's ERC-721 transfer list).
//! Every candidate is then checked with `ownerOf` at the valuation block: ids
//! transferred away or burned since are dropped, and ids whose owner could not
//! be read are reported, never silently dropped.

use alloy::eips::BlockId;
use alloy::primitives::{address, b256, Address, B256, U256};
use alloy::providers::{DynProvider, Provider};
use alloy::rpc::types::{Filter, Log};
use futures::future::BoxFuture;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

use super::uniswap_lp_reads::Outcome;
use crate::error::RpcError;

/// `Transfer(address,address,uint256)`
pub const TRANSFER_TOPIC: B256 =
    b256!("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

/// Blocks asked for in the first `eth_getLogs` request. The filter matches
/// few logs, so wide ranges are cheap where the endpoint allows them.
pub const INITIAL_CHUNK: u64 = 1_000_000;

/// Largest range asked for in one request
pub const MAX_CHUNK: u64 = 5_000_000;

/// Time limit of one eth_getLogs or eth_getCode request
pub const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60);

/// Default cap on `eth_getLogs` requests for one scan
pub const DEFAULT_MAX_REQUESTS: u64 = 2_000;

/// Consecutive failures of one range (other than range/size limits) before
/// the scan gives up
const MAX_CONSECUTIVE_FAILURES: u32 = 5;

/// The block each v4 PositionManager was deployed in (the first block with
/// its code), by chain id. Checked on-chain: the code is absent at the block
/// before and present at this one. A unit test checks each address against
/// the uniswap-sdk-core address map, so an SDK update cannot pair a new
/// address with a stale block.
pub const V4_POSITION_MANAGER_DEPLOYMENTS: &[(u64, Address, u64)] = &[
    (
        1,
        address!("0xbd216513d74c8cf14cf4747e6aaa6420ff64ee9e"),
        21_689_089,
    ),
    (
        10,
        address!("0x3c3ea4b57a46241e54610e5f022e5c45859a1017"),
        130_947_681,
    ),
    (
        56,
        address!("0x7a4a5c919ae2541aed11041a1aeee68f1287f95b"),
        45_970_613,
    ),
    (
        137,
        address!("0x1ec2ebf4f37e7363fdfe3551602425af0b3ceef9"),
        66_980_394,
    ),
    (
        8453,
        address!("0x7c5f5a4bbd8fd63184577525326123b519429bdc"),
        25_350_993,
    ),
    (
        42161,
        address!("0xd88f38f930b7952f2db2432cb002e7abbf3dd869"),
        297_842_893,
    ),
    (
        43114,
        address!("0xb74b1f14d2754acfcbbe1a221023a5cf50ab8acd"),
        56_195_382,
    ),
];

/// Deployment block of the v4 PositionManager `pm` on `chain_id`, when known
pub fn deployment_block(chain_id: u64, pm: Address) -> Option<u64> {
    V4_POSITION_MANAGER_DEPLOYMENTS
        .iter()
        .find(|(c, a, _)| *c == chain_id && *a == pm)
        .map(|(_, _, b)| *b)
}

/// How the v4 ids were discovered, reported with the positions
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct V4Discovery {
    /// "logs" (PositionManager Transfer logs) or "explorer" (the block
    /// explorer's ERC-721 transfer list)
    pub source: String,
    pub position_manager: String,
    pub from_block: u64,
    /// "deployment" (built-in deployment block), "code_search" (deployment
    /// block found by binary search on eth_getCode), "--from-block", or
    /// "genesis" (explorer source when the deployment block is unknown)
    pub from_block_source: String,
    /// The PositionManager's deployment block, when known
    pub deployment_block: Option<u64>,
    /// Valuation block: the scan's end, and the block every read is pinned to
    pub to_block: u64,
    /// Explorer source: the last block of a transfer the explorer listed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_last_block: Option<u64>,
    /// First block scanned with eth_getLogs (explorer source: the gap after
    /// the explorer's last listed transfer, so indexer lag cannot drop ids)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs_from: Option<u64>,
    /// Last block covered without a gap (null when nothing was covered)
    pub scanned_to: Option<u64>,
    /// Whether the whole range was covered
    pub complete: bool,
    /// eth_getLogs requests plus explorer pages
    pub requests: u64,
    /// Distinct token ids ever transferred to the owner in the range
    pub candidates: usize,
    /// Of those, still owned at the valuation block
    pub still_owned: usize,
    pub transferred_out: usize,
    pub burned: usize,
    /// Candidates whose owner could not be read (listed in `failed`)
    pub owner_unknown: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Where the v4 scan starts
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartPlan {
    /// The PositionManager did not exist yet at the valuation block: there
    /// are no v4 positions, and that is complete
    NotDeployed { deployed: u64 },
    /// Scan from `from`; `gap` says why ids received earlier are not covered
    Scan {
        from: u64,
        source: &'static str,
        gap: Option<String>,
    },
}

/// Decide where the scan starts.
///
/// - `deployment`: the deployment block and how it was found, when known
/// - `code_before_from`: whether the PositionManager had code at
///   `from_block - 1` (asked only when `from_block` is given and the
///   deployment block is unknown; `None` when that read failed)
/// - `explorer`: without a start block the explorer source starts at 0
pub fn plan_start(
    from_block: Option<u64>,
    deployment: Option<(u64, &'static str)>,
    valuation: u64,
    code_before_from: Option<bool>,
    explorer: bool,
) -> Result<StartPlan, String> {
    if let Some((d, _)) = deployment {
        if valuation < d {
            return Ok(StartPlan::NotDeployed { deployed: d });
        }
    }
    match (from_block, deployment) {
        (Some(f), _) if f > valuation => Err(format!(
            "--from-block {f} is after the valuation block {valuation}"
        )),
        (Some(f), Some((d, _))) => Ok(StartPlan::Scan {
            from: f,
            source: "--from-block",
            gap: (f > d).then(|| {
                format!("v4 ids received before block {f} are not covered (the PositionManager was deployed at block {d})")
            }),
        }),
        (Some(f), None) => Ok(StartPlan::Scan {
            from: f,
            source: "--from-block",
            gap: match (f, code_before_from) {
                (0, _) | (_, Some(false)) => None,
                (_, Some(true)) => Some(format!(
                    "v4 ids received before block {f} are not covered (the PositionManager already had code at block {})",
                    f - 1
                )),
                (_, None) => Some(format!(
                    "v4 ids received before block {f} may not be covered (could not read the PositionManager's code at block {})",
                    f - 1
                )),
            },
        }),
        (None, Some((d, source))) => Ok(StartPlan::Scan {
            from: d,
            source,
            gap: None,
        }),
        (None, None) if explorer => Ok(StartPlan::Scan {
            from: 0,
            source: "genesis",
            gap: None,
        }),
        (None, None) => Err("the PositionManager's deployment block is unknown and could not be found; pass --from-block".into()),
    }
}

/// Result of a Transfer log scan
#[derive(Debug, Clone, PartialEq)]
pub struct ScanOutcome {
    /// Distinct candidate token ids, ascending
    pub ids: Vec<U256>,
    pub scanned_to: Option<u64>,
    pub requests: u64,
    /// Why the scan stopped before the end of the range
    pub error: Option<String>,
}

/// One `eth_getLogs` per call: Transfer logs of the PositionManager to the
/// owner in `[from, to]`
pub trait LogSource: Sync {
    fn logs<'a>(&'a self, from: u64, to: u64) -> BoxFuture<'a, Result<Vec<Log>, String>>;
}

/// [`LogSource`] over an RPC provider
pub struct ProviderLogs {
    pub provider: DynProvider,
    pub pm: Address,
    pub owner: Address,
}

impl LogSource for ProviderLogs {
    fn logs<'a>(&'a self, from: u64, to: u64) -> BoxFuture<'a, Result<Vec<Log>, String>> {
        Box::pin(async move {
            let filter = Filter::new()
                .address(self.pm)
                .event_signature(TRANSFER_TOPIC)
                .topic2(self.owner.into_word())
                .from_block(from)
                .to_block(to);
            match tokio::time::timeout(REQUEST_TIMEOUT, self.provider.get_logs(&filter)).await {
                Ok(r) => {
                    r.map_err(|e| one_line(&crate::utils::url::redact_urls_in_text(&e.to_string())))
                }
                Err(_) => Err(format!(
                    "eth_getLogs timed out after {} s",
                    REQUEST_TIMEOUT.as_secs()
                )),
            }
        })
    }
}

/// Whether an error says the request took too long (a gateway or server
/// timeout), which a smaller block range avoids
pub fn is_timeout(msg: &str) -> bool {
    let m = msg.to_lowercase();
    [
        "http error 504",
        "gateway time",
        "timed out",
        "timeout",
        "deadline exceeded",
    ]
    .iter()
    .any(|p| m.contains(p))
}

/// An error message on one line, at most 300 characters (some gateways
/// answer with a whole HTML page)
pub fn one_line(msg: &str) -> String {
    let s = msg.split_whitespace().collect::<Vec<_>>().join(" ");
    match s.char_indices().nth(300) {
        Some((i, _)) => format!("{}...", &s[..i]),
        None => s,
    }
}

/// The token id of an ERC-721 `Transfer` of `pm` to `owner`, or None for any
/// other log (an ERC-20 Transfer has 3 topics; the id is the 4th topic)
pub fn decode_transfer_to(log: &Log, pm: Address, owner: Address) -> Option<U256> {
    let t = log.topics();
    if log.address() != pm || t.len() != 4 || t[0] != TRANSFER_TOPIC || t[2] != owner.into_word() {
        return None;
    }
    Some(U256::from_be_bytes(t[3].0))
}

/// Options of [`scan_transfers`] and [`scan_segment`]
#[derive(Debug, Clone)]
pub struct ScanOpts {
    /// Cap on requests across all segments
    pub max_requests: u64,
    pub retry_delay: std::time::Duration,
    /// Consecutive block ranges scanned concurrently
    pub segments: u64,
    /// No request is started after this time
    pub deadline: Option<tokio::time::Instant>,
}

/// Scan `[from, to]` for Transfer logs to `owner` with one request at a
/// time, adapting the range to the endpoint:
///
/// - a block-range error stating a limit sets the range to that limit and
///   never grows past it again; one without a limit, or a result-size error,
///   halves the range;
/// - a timeout (gateway 504, "timed out") halves the range at once and caps
///   it there;
/// - other errors (rate limits, connection errors) are retried with a
///   growing delay, halving the range from the second failure;
/// - `MAX_CONSECUTIVE_FAILURES` failures in a row other than range/size
///   limits end the scan;
/// - each success doubles the range (up to [`MAX_CHUNK`] or the known limit).
///
/// `used` counts requests across every segment sharing `opts.max_requests`.
/// Stops with an error, keeping what it found, when a one-block range is too
/// large, failures repeat, the request cap is reached or the deadline passes.
pub async fn scan_segment(
    src: &dyn LogSource,
    pm: Address,
    owner: Address,
    from: u64,
    to: u64,
    opts: &ScanOpts,
    used: &AtomicU64,
) -> ScanOutcome {
    let mut ids = std::collections::BTreeSet::new();
    let mut out = ScanOutcome {
        ids: Vec::new(),
        scanned_to: None,
        requests: 0,
        error: None,
    };
    let mut chunk = INITIAL_CHUNK;
    let mut ceiling = MAX_CHUNK;
    let mut failures = 0u32;
    let mut cur = from;
    while cur <= to {
        if opts
            .deadline
            .is_some_and(|d| tokio::time::Instant::now() >= d)
        {
            out.error = Some(format!("scan time limit reached at block {cur} of {to}"));
            break;
        }
        if used.fetch_add(1, Ordering::SeqCst) >= opts.max_requests {
            out.error = Some(format!(
                "request cap of {} eth_getLogs requests reached at block {cur} of {to}",
                opts.max_requests
            ));
            break;
        }
        let end = cur.saturating_add(chunk - 1).min(to);
        let span = end - cur + 1;
        out.requests += 1;
        let res = src.logs(cur, end).await;
        match &res {
            Ok(l) => tracing::info!("v4 scan: blocks {cur}-{end}: {} logs", l.len()),
            Err(e) => tracing::info!("v4 scan: blocks {cur}-{end}: {e}"),
        }
        match res {
            Ok(logs) => {
                ids.extend(logs.iter().filter_map(|l| decode_transfer_to(l, pm, owner)));
                out.scanned_to = Some(end);
                failures = 0;
                if end == u64::MAX {
                    break;
                }
                cur = end + 1;
                chunk = chunk.saturating_mul(2).min(ceiling);
            }
            Err(msg) => match crate::rpc::classify_logs_error(&msg, span) {
                Some(RpcError::BlockRangeTooLarge { max, .. }) if max > 0 && max < span => {
                    chunk = max;
                    ceiling = max;
                }
                Some(RpcError::BlockRangeTooLarge { .. }) | Some(RpcError::ResponseTooLarge(_)) => {
                    if span == 1 {
                        out.error = Some(format!("block {cur}: {msg}"));
                        break;
                    }
                    chunk = (span / 2).max(1);
                    ceiling = ceiling.min(chunk);
                }
                _ => {
                    failures += 1;
                    if failures >= MAX_CONSECUTIVE_FAILURES {
                        out.error = Some(format!("blocks {cur}-{end}: {msg}"));
                        break;
                    }
                    if is_timeout(&msg) && span > 1 {
                        // The range took the endpoint too long: smaller
                        // ranges from now on, retried at once
                        chunk = (span / 2).max(1);
                        ceiling = ceiling.min(chunk);
                    } else {
                        if failures >= 2 && span > 1 {
                            chunk = (span / 2).max(1);
                        }
                        tokio::time::sleep(opts.retry_delay * failures).await;
                    }
                }
            },
        }
    }
    out.ids = ids.into_iter().collect();
    out
}

/// Scan `[from, to]` as up to `opts.segments` consecutive ranges at once,
/// sharing one request cap (never more segments than the cap allows). The
/// ids found anywhere are kept; `scanned_to` is the end of the gap-free
/// coverage from `from`, and `error` the first segment's error in block
/// order.
pub async fn scan_transfers(
    src: &dyn LogSource,
    pm: Address,
    owner: Address,
    from: u64,
    to: u64,
    opts: &ScanOpts,
) -> ScanOutcome {
    let span = to.saturating_sub(from).saturating_add(1);
    let n = opts
        .segments
        .min(opts.max_requests.max(1))
        .clamp(1, span.max(1));
    let size = span.div_ceil(n);
    let used = AtomicU64::new(0);
    let ranges: Vec<(u64, u64)> = (0..n)
        .map(|i| from + i * size)
        .take_while(|s| *s <= to)
        .map(|s| (s, s.saturating_add(size - 1).min(to)))
        .collect();
    let outs = futures::future::join_all(
        ranges
            .iter()
            .map(|(f, t)| scan_segment(src, pm, owner, *f, *t, opts, &used)),
    )
    .await;
    let mut ids = std::collections::BTreeSet::new();
    let mut total = ScanOutcome {
        ids: Vec::new(),
        scanned_to: None,
        requests: 0,
        error: None,
    };
    let mut gap_free = true;
    for ((_, t), o) in ranges.iter().zip(outs) {
        ids.extend(o.ids);
        total.requests += o.requests;
        if gap_free {
            if o.scanned_to.is_some() {
                total.scanned_to = o.scanned_to;
            }
            if o.error.is_some() || o.scanned_to != Some(*t) {
                gap_free = false;
            }
        }
        if total.error.is_none() {
            total.error = o.error;
        }
    }
    total.ids = ids.into_iter().collect();
    total
}

/// Whether an address has code at a block (one `eth_getCode`)
pub trait CodeSource: Sync {
    fn has_code<'a>(&'a self, block: u64) -> BoxFuture<'a, Result<bool, String>>;
}

/// [`CodeSource`] over an RPC provider
pub struct ProviderCode {
    pub provider: DynProvider,
    pub address: Address,
}

impl CodeSource for ProviderCode {
    fn has_code<'a>(&'a self, block: u64) -> BoxFuture<'a, Result<bool, String>> {
        Box::pin(async move {
            let fut = self
                .provider
                .get_code_at(self.address)
                .block_id(BlockId::number(block));
            match tokio::time::timeout(REQUEST_TIMEOUT, fut).await {
                Ok(Ok(code)) => Ok(!code.is_empty()),
                Ok(Err(e)) => Err(one_line(&crate::utils::url::redact_urls_in_text(
                    &e.to_string(),
                ))),
                Err(_) => Err("eth_getCode timed out".into()),
            }
        })
    }
}

/// The first block at or before `head` where the contract has code, by
/// binary search (as `rpc find-change --code` does; about log2(head) reads,
/// which need historical state). Assumes the code, once deployed, stays.
/// Returns the block and the number of reads.
pub async fn find_deployment_block(src: &dyn CodeSource, head: u64) -> Result<(u64, u64), String> {
    let mut reads = 1u64;
    if !src.has_code(head).await? {
        return Err(format!("no code at block {head}"));
    }
    // Invariant: no code at lo (or lo is before genesis), code at hi
    let (mut lo, mut hi) = (None::<u64>, head);
    loop {
        let start = lo.map_or(0, |l| l + 1);
        if start >= hi {
            return Ok((hi, reads));
        }
        let mid = start + (hi - start) / 2;
        reads += 1;
        if src.has_code(mid).await? {
            hi = mid;
        } else {
            lo = Some(mid);
        }
    }
}

/// What `ownerOf(tokenId)` at the valuation block says about a candidate
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ownership {
    Owned,
    /// Owned by someone else now
    TransferredOut,
    /// `ownerOf` reverted with `NOT_MINTED`: the token was burned
    Burned,
    /// The read failed or reverted for another reason; ownership is not known
    Unknown(String),
}

/// The revert reason the PositionManager's ERC-721 (solmate) gives for a
/// token that does not exist
pub const NOT_MINTED: &str = "NOT_MINTED";

pub fn classify_owner(out: &Outcome, owner: Address) -> Ownership {
    alloy::sol! {
        function ownerOf(uint256 tokenId) external view returns (address);
    }
    match out {
        Outcome::Reverted(r) if r.contains(NOT_MINTED) => Ownership::Burned,
        Outcome::Reverted(r) => Ownership::Unknown(if r.is_empty() {
            "ownerOf() reverted without a reason".into()
        } else {
            format!("ownerOf() reverted: {r}")
        }),
        Outcome::Failed(e) => Ownership::Unknown(e.clone()),
        Outcome::Ok(_) => match out.decode::<ownerOfCall>("ownerOf()") {
            Ok(a) if a == owner => Ownership::Owned,
            Ok(_) => Ownership::TransferredOut,
            Err(e) => Ownership::Unknown(e),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    const PM: Address = address!("0x00000000000000000000000000000000000000aa");
    const OWNER: Address = address!("0x00000000000000000000000000000000000000bb");
    const OTHER: Address = address!("0x00000000000000000000000000000000000000cc");

    fn transfer(from: Address, to: Address, id: u64) -> Log {
        Log {
            inner: alloy::primitives::Log::new_unchecked(
                PM,
                vec![
                    TRANSFER_TOPIC,
                    from.into_word(),
                    to.into_word(),
                    B256::from(U256::from(id)),
                ],
                Default::default(),
            ),
            ..Default::default()
        }
    }

    #[test]
    fn decode_only_erc721_transfers_to_owner() {
        assert_eq!(
            decode_transfer_to(&transfer(Address::ZERO, OWNER, 7), PM, OWNER),
            Some(U256::from(7u8))
        );
        // to someone else
        assert_eq!(
            decode_transfer_to(&transfer(OWNER, OTHER, 7), PM, OWNER),
            None
        );
        // another contract
        assert_eq!(
            decode_transfer_to(&transfer(Address::ZERO, OWNER, 7), OTHER, OWNER),
            None
        );
        // ERC-20 shaped (3 topics, amount in data)
        let mut erc20 = transfer(Address::ZERO, OWNER, 7);
        let topics = erc20.inner.data.topics()[..3].to_vec();
        erc20.inner.data = alloy::primitives::LogData::new_unchecked(topics, Default::default());
        assert_eq!(decode_transfer_to(&erc20, PM, OWNER), None);
    }

    #[test]
    fn one_line_flattens_and_caps() {
        assert_eq!(
            one_line("HTTP error 504 with body: <html>\n<h1>x</h1>"),
            "HTTP error 504 with body: <html> <h1>x</h1>"
        );
        let long = "a".repeat(400);
        assert_eq!(one_line(&long).len(), 303);
    }

    #[test]
    fn classify_owner_cases() {
        let word = |a: Address| Outcome::Ok(a.into_word().0.to_vec().into());
        assert_eq!(classify_owner(&word(OWNER), OWNER), Ownership::Owned);
        assert_eq!(
            classify_owner(&word(OTHER), OWNER),
            Ownership::TransferredOut
        );
        assert_eq!(
            classify_owner(&Outcome::Reverted("revert: NOT_MINTED".into()), OWNER),
            Ownership::Burned
        );
        // Any other revert (empty data, out of gas, another reason) is unknown
        for r in ["", "out of gas", "revert: PAUSED"] {
            assert!(matches!(
                classify_owner(&Outcome::Reverted(r.into()), OWNER),
                Ownership::Unknown(_)
            ));
        }
        assert!(matches!(
            classify_owner(&Outcome::Failed("timeout".into()), OWNER),
            Ownership::Unknown(_)
        ));
        assert!(matches!(
            classify_owner(&Outcome::Ok(Default::default()), OWNER),
            Ownership::Unknown(_)
        ));
    }

    /// A fake endpoint: a stated or unstated range limit, logs at fixed
    /// blocks, and scripted transient failures
    struct FakeLogs {
        limit: u64,
        state_limit: bool,
        logs: Vec<(u64, Log)>,
        transient: Mutex<u32>,
        calls: Mutex<Vec<(u64, u64)>>,
    }

    impl FakeLogs {
        fn new(limit: u64, state_limit: bool) -> Self {
            Self {
                limit,
                state_limit,
                logs: Vec::new(),
                transient: Mutex::new(0),
                calls: Mutex::new(Vec::new()),
            }
        }
    }

    impl LogSource for FakeLogs {
        fn logs<'a>(&'a self, from: u64, to: u64) -> BoxFuture<'a, Result<Vec<Log>, String>> {
            self.calls.lock().unwrap().push((from, to));
            let r = {
                let mut t = self.transient.lock().unwrap();
                if *t > 0 {
                    *t -= 1;
                    Err("connection reset".to_string())
                } else if to - from + 1 > self.limit {
                    Err(if self.state_limit {
                        format!("query exceeds max block range {}", self.limit)
                    } else {
                        "block range is too wide".to_string()
                    })
                } else {
                    Ok(self
                        .logs
                        .iter()
                        .filter(|(b, _)| *b >= from && *b <= to)
                        .map(|(_, l)| l.clone())
                        .collect())
                }
            };
            Box::pin(async move { r })
        }
    }

    const NO_WAIT: std::time::Duration = std::time::Duration::ZERO;

    async fn seg(
        src: &dyn LogSource,
        from: u64,
        to: u64,
        max_requests: u64,
        retry_delay: std::time::Duration,
    ) -> ScanOutcome {
        let opts = ScanOpts {
            max_requests,
            retry_delay,
            segments: 1,
            deadline: None,
        };
        scan_segment(src, PM, OWNER, from, to, &opts, &AtomicU64::new(0)).await
    }

    #[tokio::test]
    async fn stated_limit_is_used_and_not_exceeded() {
        let mut src = FakeLogs::new(10_000, true);
        src.logs = vec![
            (5, transfer(Address::ZERO, OWNER, 1)),
            (25_000, transfer(OTHER, OWNER, 2)),
            (25_001, transfer(OWNER, OTHER, 2)),
            (39_999, transfer(OTHER, OWNER, 1)),
        ];
        let out = seg(&src, 0, 39_999, 100, NO_WAIT).await;
        assert_eq!(out.error, None);
        assert_eq!(out.ids, vec![U256::from(1u8), U256::from(2u8)]);
        assert_eq!(out.scanned_to, Some(39_999));
        // one failed wide request, then four of 10,000 blocks
        let calls = src.calls.lock().unwrap().clone();
        assert_eq!(calls.len(), 5);
        assert!(calls[1..].iter().all(|(f, t)| t - f < 10_000));
        assert_eq!(out.requests, 5);
    }

    #[tokio::test]
    async fn unstated_limit_halves_until_it_fits() {
        let src = FakeLogs::new(100_000, false);
        let out = seg(&src, 0, 999_999, 100, NO_WAIT).await;
        assert_eq!(out.error, None);
        assert_eq!(out.scanned_to, Some(999_999));
        let calls = src.calls.lock().unwrap().clone();
        // 1M, 500k, 250k, 125k, 62.5k fits; never grows past it again
        let ok: Vec<u64> = calls
            .iter()
            .map(|(f, t)| t - f + 1)
            .filter(|s| *s <= 100_000)
            .collect();
        assert!(ok.iter().all(|s| *s == 62_500));
    }

    #[tokio::test]
    async fn gateway_timeouts_shrink_the_range() {
        struct Slow(Mutex<Vec<(u64, u64)>>);
        impl LogSource for Slow {
            fn logs<'a>(&'a self, f: u64, t: u64) -> BoxFuture<'a, Result<Vec<Log>, String>> {
                self.0.lock().unwrap().push((f, t));
                let r = if t - f >= 250_000 {
                    Err("HTTP error 504 with body: <html>Gateway Time-out</html>".to_string())
                } else {
                    Ok(Vec::new())
                };
                Box::pin(async move { r })
            }
        }
        let src = Slow(Mutex::new(Vec::new()));
        let out = seg(&src, 0, 999_999, 100, std::time::Duration::from_secs(60)).await;
        assert_eq!(out.error, None);
        assert_eq!(out.scanned_to, Some(999_999));
        // 1M and 500k time out; 250k ranges from then on, never grown back
        // (and no sleep: the 60 s retry delay would make this test hang)
        let calls = src.0.lock().unwrap().clone();
        assert_eq!(calls.len(), 2 + 4);
        assert!(calls[2..].iter().all(|(f, t)| t - f + 1 == 250_000));
    }

    #[tokio::test]
    async fn transient_errors_are_retried() {
        let src = FakeLogs::new(u64::MAX, true);
        *src.transient.lock().unwrap() = 2;
        let out = seg(&src, 0, 10, 100, NO_WAIT).await;
        assert_eq!(out.error, None);
        // two failures (the second halves the range), then two successes
        assert_eq!(out.requests, 4);
        assert_eq!(out.scanned_to, Some(10));
    }

    #[tokio::test]
    async fn repeated_failures_stop_with_partial_result() {
        let mut src = FakeLogs::new(u64::MAX, true);
        src.logs = vec![(3, transfer(Address::ZERO, OWNER, 9))];
        let out = seg(&src, 0, 10, 100, NO_WAIT).await;
        assert_eq!(out.ids, vec![U256::from(9u8)]);
        *src.transient.lock().unwrap() = 100;
        let out = seg(&src, 0, 10, 100, NO_WAIT).await;
        assert!(out.error.unwrap().contains("connection reset"));
        assert_eq!(out.scanned_to, None);
        assert!(out.ids.is_empty());
    }

    #[tokio::test]
    async fn request_budget_is_reported() {
        let src = FakeLogs::new(10, true);
        let out = seg(&src, 0, 1_000, 5, NO_WAIT).await;
        assert_eq!(out.requests, 5);
        let e = out.error.unwrap();
        assert!(e.contains("request cap of 5 eth_getLogs requests"), "{e}");
        // first request failed (range), the next four covered 40 blocks
        assert_eq!(out.scanned_to, Some(39));
    }

    #[tokio::test]
    async fn segments_cover_the_range_and_merge() {
        let mut src = FakeLogs::new(u64::MAX, true);
        src.logs = vec![
            (10, transfer(Address::ZERO, OWNER, 3)),
            (95, transfer(OTHER, OWNER, 1)),
            (99, transfer(OTHER, OWNER, 3)),
        ];
        let opts = ScanOpts {
            max_requests: 100,
            retry_delay: NO_WAIT,
            segments: 4,
            deadline: None,
        };
        let out = scan_transfers(&src, PM, OWNER, 0, 99, &opts).await;
        assert_eq!(out.error, None);
        assert_eq!(out.ids, vec![U256::from(1u8), U256::from(3u8)]);
        assert_eq!(out.scanned_to, Some(99));
        let mut calls = src.calls.lock().unwrap().clone();
        calls.sort();
        assert_eq!(calls, vec![(0, 24), (25, 49), (50, 74), (75, 99)]);

        // A short range is not split into empty segments
        let out = scan_transfers(&src, PM, OWNER, 5, 6, &opts).await;
        assert_eq!(out.requests, 2);
        assert_eq!(out.scanned_to, Some(6));
    }

    #[tokio::test]
    async fn segment_failure_stops_gap_free_coverage() {
        // Every request fails: nothing is covered and the error is kept
        let src = FakeLogs::new(u64::MAX, true);
        *src.transient.lock().unwrap() = 1_000;
        let opts = ScanOpts {
            max_requests: 100,
            retry_delay: NO_WAIT,
            segments: 2,
            deadline: None,
        };
        let out = scan_transfers(&src, PM, OWNER, 0, 99, &opts).await;
        assert_eq!(out.scanned_to, None);
        assert!(out.error.is_some());
    }

    #[tokio::test]
    async fn request_cap_is_shared_across_segments() {
        // Every request fails on range: segments would each spend their own
        // cap if it were not shared
        let src = FakeLogs::new(1, true);
        let opts = ScanOpts {
            max_requests: 10,
            retry_delay: NO_WAIT,
            segments: 4,
            deadline: None,
        };
        let out = scan_transfers(&src, PM, OWNER, 0, 999, &opts).await;
        assert_eq!(out.requests, 10);
        assert_eq!(src.calls.lock().unwrap().len(), 10);
        assert!(out.error.unwrap().contains("request cap of 10 "));

        // A cap below the segment count uses fewer segments
        let src = FakeLogs::new(u64::MAX, true);
        let opts = ScanOpts {
            max_requests: 2,
            retry_delay: NO_WAIT,
            segments: 4,
            deadline: None,
        };
        let out = scan_transfers(&src, PM, OWNER, 0, 999, &opts).await;
        assert_eq!(out.error, None);
        assert_eq!(out.requests, 2);
        assert_eq!(out.scanned_to, Some(999));
    }

    #[tokio::test]
    async fn deadline_ends_the_scan() {
        let src = FakeLogs::new(u64::MAX, true);
        let opts = ScanOpts {
            max_requests: 100,
            retry_delay: NO_WAIT,
            segments: 1,
            deadline: Some(tokio::time::Instant::now()),
        };
        let out = scan_transfers(&src, PM, OWNER, 0, 999, &opts).await;
        assert_eq!(out.requests, 0);
        assert_eq!(out.scanned_to, None);
        assert!(out.error.unwrap().contains("time limit"));
    }

    struct FakeCode {
        deployed: u64,
        reads: Mutex<u64>,
    }

    impl CodeSource for FakeCode {
        fn has_code<'a>(&'a self, block: u64) -> BoxFuture<'a, Result<bool, String>> {
            *self.reads.lock().unwrap() += 1;
            let r = Ok(block >= self.deployed);
            Box::pin(async move { r })
        }
    }

    #[tokio::test]
    async fn code_search_finds_first_block_with_code() {
        for (deployed, head) in [(21_689_089u64, 26_486_919u64), (0, 10), (10, 10), (1, 2)] {
            let src = FakeCode {
                deployed,
                reads: Mutex::new(0),
            };
            let (b, reads) = find_deployment_block(&src, head).await.unwrap();
            assert_eq!(b, deployed, "head {head}");
            assert_eq!(reads, *src.reads.lock().unwrap());
            assert!(reads <= 2 + 64 - head.leading_zeros() as u64);
        }
        // Not deployed yet at the head
        let src = FakeCode {
            deployed: 100,
            reads: Mutex::new(0),
        };
        assert!(find_deployment_block(&src, 50).await.is_err());
    }

    #[test]
    fn plan_start_cases() {
        let dep = Some((100u64, "deployment"));
        // Valuation block before the deployment: no v4 positions, complete
        assert_eq!(
            plan_start(None, dep, 99, None, false),
            Ok(StartPlan::NotDeployed { deployed: 100 })
        );
        assert_eq!(
            plan_start(Some(50), dep, 99, None, false),
            Ok(StartPlan::NotDeployed { deployed: 100 })
        );
        // Default: from the deployment block
        assert_eq!(
            plan_start(None, dep, 500, None, false),
            Ok(StartPlan::Scan {
                from: 100,
                source: "deployment",
                gap: None
            })
        );
        // --from-block at or before the deployment covers everything
        assert_eq!(
            plan_start(Some(90), dep, 500, None, false),
            Ok(StartPlan::Scan {
                from: 90,
                source: "--from-block",
                gap: None
            })
        );
        // --from-block after it leaves a gap
        let StartPlan::Scan { gap: Some(g), .. } =
            plan_start(Some(200), dep, 500, None, false).unwrap()
        else {
            panic!("expected a gap");
        };
        assert!(g.contains("before block 200 are not covered"), "{g}");
        assert!(plan_start(Some(600), dep, 500, None, false).is_err());
        // Unknown deployment: code before --from-block decides
        let gap = |c| match plan_start(Some(200), None, 500, c, false).unwrap() {
            StartPlan::Scan { gap, .. } => gap,
            _ => panic!(),
        };
        assert_eq!(gap(Some(false)), None);
        assert!(gap(Some(true)).unwrap().contains("not covered"));
        assert!(gap(None).unwrap().contains("may not be covered"));
        // No start at all: an error for logs, genesis for the explorer
        assert!(plan_start(None, None, 500, None, false).is_err());
        assert_eq!(
            plan_start(None, None, 500, None, true),
            Ok(StartPlan::Scan {
                from: 0,
                source: "genesis",
                gap: None
            })
        );
    }

    #[test]
    fn deployments_match_the_sdk_address_map() {
        use uniswap_v4_sdk::prelude::sdk_core::addresses::CHAIN_TO_ADDRESSES_MAP;
        for (chain, pm, block) in V4_POSITION_MANAGER_DEPLOYMENTS {
            let a = CHAIN_TO_ADDRESSES_MAP
                .get(chain)
                .and_then(|a| a.v4_position_manager);
            assert_eq!(a, Some(*pm), "chain {chain}");
            assert!(*block > 0);
            assert_eq!(deployment_block(*chain, *pm), Some(*block));
        }
        assert_eq!(deployment_block(1, OTHER), None);
    }
}
