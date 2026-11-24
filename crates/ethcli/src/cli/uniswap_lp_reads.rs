//! Batched contract reads for the Uniswap LP valuation
//!
//! Every read goes through a [`Reader`], pinned to one block. Where Multicall3
//! has code at that block the reads of one stage go out as `aggregate3`
//! batches with `allowFailure` set on every call, so one failing call does
//! not fail the others. Before Multicall3 existed (or on a chain without it)
//! each call is sent on its own.

use alloy::eips::BlockId;
use alloy::primitives::{Address, Bytes};
use alloy::providers::{DynProvider, Provider};
use alloy::sol_types::SolCall;
use futures::future::BoxFuture;
use futures::StreamExt;

use crate::rpc::multicall::MulticallBuilder;

/// Calls per `aggregate3` request
pub const MULTICALL_CHUNK: usize = 200;

/// `aggregate3` requests (or single calls, without Multicall3) in flight at
/// once
const CONCURRENCY: usize = 4;

/// One read: `target.call(data)`
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Call {
    pub target: Address,
    pub data: Bytes,
}

impl Call {
    pub fn new<C: SolCall>(target: Address, call: C) -> Self {
        Self {
            target,
            data: call.abi_encode().into(),
        }
    }
}

/// What a read gave
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// Return data
    Ok(Bytes),
    /// The call reverted (the reason, when there is one)
    Reverted(String),
    /// The read did not happen (RPC error)
    Failed(String),
}

impl Outcome {
    /// Decode the return data of `C`; `what` names the call in errors
    pub fn decode<C: SolCall>(&self, what: &str) -> Result<C::Return, String> {
        match self {
            Outcome::Ok(b) => C::abi_decode_returns(b).map_err(|e| format!("{what}: {e}")),
            Outcome::Reverted(r) if r.is_empty() => Err(format!("{what} reverted")),
            Outcome::Reverted(r) => Err(format!("{what} reverted: {r}")),
            Outcome::Failed(e) => Err(format!("{what} failed: {e}")),
        }
    }

    pub fn is_revert(&self) -> bool {
        matches!(self, Outcome::Reverted(_))
    }
}

/// Reads at one block
pub trait Reader: Sync {
    /// Run `calls` (from no particular sender); results in input order
    fn read<'a>(&'a self, calls: Vec<Call>) -> BoxFuture<'a, Vec<Outcome>>;
    /// One eth_call sent from `from` (never batched: `msg.sender` matters)
    fn call_from<'a>(&'a self, from: Address, call: Call) -> BoxFuture<'a, Outcome>;
    /// Whether `read` goes through Multicall3
    fn via_multicall(&self) -> bool;
}

/// [`Reader`] over an RPC provider
pub struct ChainReader {
    provider: DynProvider,
    block: BlockId,
    multicall: bool,
}

impl ChainReader {
    pub fn new(provider: DynProvider, block: BlockId, multicall: bool) -> Self {
        Self {
            provider,
            block,
            multicall,
        }
    }

    async fn single(&self, from: Option<Address>, call: Call) -> Outcome {
        let mut tx = alloy::rpc::types::TransactionRequest::default()
            .to(call.target)
            .input(call.data.into());
        if let Some(f) = from {
            tx = tx.from(f);
        }
        // Retry briefly: shared endpoints may rate-limit. A revert is final.
        let mut last = String::new();
        for attempt in 0..3u64 {
            match self.provider.call(tx.clone()).block(self.block).await {
                Ok(b) => return Outcome::Ok(b),
                Err(e) => {
                    let msg = crate::utils::url::redact_urls_in_text(&e.to_string());
                    if msg.to_lowercase().contains("revert") {
                        return Outcome::Reverted(msg);
                    }
                    last = msg;
                    tokio::time::sleep(std::time::Duration::from_millis(300 * (attempt + 1))).await;
                }
            }
        }
        Outcome::Failed(last)
    }

    async fn batch(&self, calls: Vec<Call>) -> Vec<Outcome> {
        let n = calls.len();
        for attempt in 0..2u64 {
            let mut b = MulticallBuilder::new();
            for c in &calls {
                b = b.add_call_allow_failure(c.target, c.data.clone());
            }
            match b.execute_at(&self.provider, self.block).await {
                Ok(rs) if rs.len() == n => {
                    return rs
                        .into_iter()
                        .map(|r| {
                            if r.success {
                                Outcome::Ok(r.data)
                            } else {
                                Outcome::Reverted(
                                    alloy::sol_types::decode_revert_reason(&r.data)
                                        .unwrap_or_default(),
                                )
                            }
                        })
                        .collect();
                }
                // A whole batch can fail on one call (a token that burns
                // all the gas) or on a flaky request: retry once, then send
                // its calls one by one so only the bad call fails
                _ => {}
            }
            tokio::time::sleep(std::time::Duration::from_millis(300 * (attempt + 1))).await;
        }
        futures::stream::iter(calls)
            .map(|c| self.single(None, c))
            .buffered(CONCURRENCY)
            .collect()
            .await
    }
}

impl Reader for ChainReader {
    fn read<'a>(&'a self, calls: Vec<Call>) -> BoxFuture<'a, Vec<Outcome>> {
        Box::pin(async move {
            if self.multicall {
                let chunks: Vec<Vec<Call>> =
                    calls.chunks(MULTICALL_CHUNK).map(|c| c.to_vec()).collect();
                futures::stream::iter(chunks)
                    .map(|c| self.batch(c))
                    .buffered(CONCURRENCY)
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .flatten()
                    .collect()
            } else {
                futures::stream::iter(calls)
                    .map(|c| self.single(None, c))
                    .buffered(CONCURRENCY)
                    .collect()
                    .await
            }
        })
    }

    fn call_from<'a>(&'a self, from: Address, call: Call) -> BoxFuture<'a, Outcome> {
        Box::pin(self.single(Some(from), call))
    }

    fn via_multicall(&self) -> bool {
        self.multicall
    }
}

/// Distinct values in first-seen order, and each input's index into them
pub fn dedup<T: Clone + Eq + std::hash::Hash>(items: &[T]) -> (Vec<T>, Vec<usize>) {
    let mut seen = std::collections::HashMap::new();
    let mut uniq = Vec::new();
    let idx = items
        .iter()
        .map(|it| {
            *seen.entry(it.clone()).or_insert_with(|| {
                uniq.push(it.clone());
                uniq.len() - 1
            })
        })
        .collect();
    (uniq, idx)
}

#[cfg(test)]
pub mod mock {
    //! A [`Reader`] answering from a table, counting round trips
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    pub struct MockReader {
        pub answers: HashMap<Call, Outcome>,
        pub from_answers: HashMap<(Address, Call), Outcome>,
        /// Sizes of each `read` (one round trip each)
        pub batches: Mutex<Vec<usize>>,
        pub single_calls: Mutex<usize>,
    }

    impl MockReader {
        pub fn set<C: SolCall>(&mut self, target: Address, call: C, out: Outcome) {
            self.answers.insert(Call::new(target, call), out);
        }
        pub fn set_from<C: SolCall>(&mut self, from: Address, target: Address, c: C, o: Outcome) {
            self.from_answers.insert((from, Call::new(target, c)), o);
        }
        pub fn batch_sizes(&self) -> Vec<usize> {
            self.batches.lock().unwrap().clone()
        }
    }

    impl Reader for MockReader {
        fn read<'a>(&'a self, calls: Vec<Call>) -> BoxFuture<'a, Vec<Outcome>> {
            self.batches.lock().unwrap().push(calls.len());
            let out = calls
                .iter()
                .map(|c| {
                    self.answers
                        .get(c)
                        .cloned()
                        .unwrap_or_else(|| Outcome::Reverted(String::new()))
                })
                .collect();
            Box::pin(async move { out })
        }
        fn call_from<'a>(&'a self, from: Address, call: Call) -> BoxFuture<'a, Outcome> {
            *self.single_calls.lock().unwrap() += 1;
            let out = self
                .from_answers
                .get(&(from, call))
                .cloned()
                .unwrap_or_else(|| Outcome::Failed("no answer".into()));
            Box::pin(async move { out })
        }
        fn via_multicall(&self) -> bool {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dedup_keeps_first_seen_order() {
        let (u, i) = dedup(&["b", "a", "b", "c", "a"]);
        assert_eq!(u, vec!["b", "a", "c"]);
        assert_eq!(i, vec![0, 1, 0, 2, 1]);
    }

    #[test]
    fn outcome_decode_messages() {
        alloy::sol! { function f() external view returns (uint256); }
        let ok = Outcome::Ok(alloy::primitives::U256::from(7u8).to_be_bytes_vec().into());
        assert_eq!(
            ok.decode::<fCall>("f()").unwrap(),
            alloy::primitives::U256::from(7u8)
        );
        assert_eq!(
            Outcome::Reverted(String::new())
                .decode::<fCall>("f()")
                .unwrap_err(),
            "f() reverted"
        );
        assert_eq!(
            Outcome::Failed("timeout".into())
                .decode::<fCall>("f()")
                .unwrap_err(),
            "f() failed: timeout"
        );
        assert!(Outcome::Ok(Bytes::new()).decode::<fCall>("f()").is_err());
    }
}
