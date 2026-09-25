//! Trace API implementation (Parity-style)

use super::types::{Trace, TraceCallRequest, TraceCallResponse, TraceFilter, TraceType};
use crate::client::{Client, Network};
use crate::error::{self, Result};

/// Reason reported for `trace_*` methods Alchemy dropped on Polygon PoS
const POLYGON_BOR_REASON: &str = "Alchemy migrated Polygon PoS from Erigon to Bor on 2026-08-01; \
     use trace_block, trace_transaction, trace_call or trace_replay* instead";

/// Fail fast for `trace_*` methods that Alchemy no longer serves on Polygon PoS
///
/// Since 2026-08-01 `trace_filter`, `trace_get` and `trace_rawTransaction` are
/// unavailable on `polygon-mainnet` and `polygon-amoy`.
fn ensure_supported_on(network: Network, method: &'static str) -> Result<()> {
    match network {
        Network::PolygonMainnet | Network::PolygonAmoy => Err(error::unsupported_method(
            method,
            network.slug(),
            POLYGON_BOR_REASON,
        )),
        _ => Ok(()),
    }
}

/// Trace API for Parity-style tracing
pub struct TraceApi<'a> {
    client: &'a Client,
}

impl<'a> TraceApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get traces for a block
    pub async fn block(&self, block: &str) -> Result<Vec<Trace>> {
        self.client.rpc("trace_block", vec![block]).await
    }

    /// Execute a call and return traces
    pub async fn call(
        &self,
        request: &TraceCallRequest,
        trace_types: &[TraceType],
        block: Option<&str>,
    ) -> Result<TraceCallResponse> {
        let block = block.unwrap_or("latest");
        self.client
            .rpc("trace_call", (request, trace_types, block))
            .await
    }

    /// Get a specific trace by position in transaction
    ///
    /// Not available on Polygon PoS (`polygon-mainnet`, `polygon-amoy`) since
    /// 2026-08-01; returns an error without making a request on those networks.
    pub async fn get(&self, tx_hash: &str, trace_indices: &[u32]) -> Result<Trace> {
        ensure_supported_on(self.client.network(), "trace_get")?;
        // Convert indices to hex
        let hex_indices: Vec<String> = trace_indices.iter().map(|i| format!("0x{i:x}")).collect();
        self.client.rpc("trace_get", (tx_hash, hex_indices)).await
    }

    /// Trace a raw transaction without executing
    ///
    /// Not available on Polygon PoS (`polygon-mainnet`, `polygon-amoy`) since
    /// 2026-08-01; returns an error without making a request on those networks.
    pub async fn raw_transaction(
        &self,
        raw_tx: &str,
        trace_types: &[TraceType],
    ) -> Result<TraceCallResponse> {
        ensure_supported_on(self.client.network(), "trace_rawTransaction")?;
        self.client
            .rpc("trace_rawTransaction", (raw_tx, trace_types))
            .await
    }

    /// Replay all transactions in a block
    pub async fn replay_block_transactions(
        &self,
        block: &str,
        trace_types: &[TraceType],
    ) -> Result<Vec<TraceCallResponse>> {
        self.client
            .rpc("trace_replayBlockTransactions", (block, trace_types))
            .await
    }

    /// Replay a transaction
    pub async fn replay_transaction(
        &self,
        tx_hash: &str,
        trace_types: &[TraceType],
    ) -> Result<TraceCallResponse> {
        self.client
            .rpc("trace_replayTransaction", (tx_hash, trace_types))
            .await
    }

    /// Get all traces for a transaction
    pub async fn transaction(&self, tx_hash: &str) -> Result<Vec<Trace>> {
        self.client.rpc("trace_transaction", vec![tx_hash]).await
    }

    /// Filter traces by criteria
    ///
    /// Not available on Polygon PoS (`polygon-mainnet`, `polygon-amoy`) since
    /// 2026-08-01; returns an error without making a request on those networks.
    pub async fn filter(&self, filter: &TraceFilter) -> Result<Vec<Trace>> {
        ensure_supported_on(self.client.network(), "trace_filter")?;
        self.client.rpc("trace_filter", vec![filter]).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polygon_rejects_erigon_only_trace_methods() {
        for network in [Network::PolygonMainnet, Network::PolygonAmoy] {
            let err = ensure_supported_on(network, "trace_filter").unwrap_err();
            let msg = err.to_string();
            assert!(msg.contains("trace_filter"), "{msg}");
            assert!(msg.contains(network.slug()), "{msg}");
        }
    }

    #[test]
    fn other_networks_allow_trace_methods() {
        assert!(ensure_supported_on(Network::EthMainnet, "trace_get").is_ok());
        assert!(ensure_supported_on(Network::ArbitrumMainnet, "trace_rawTransaction").is_ok());
    }
}
