//! Trace API implementation (Parity-style)

use super::types::*;
use crate::client::Client;
use crate::error::Result;

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
    pub async fn get(&self, tx_hash: &str, trace_indices: &[u32]) -> Result<Trace> {
        // Convert indices to hex
        let hex_indices: Vec<String> = trace_indices.iter().map(|i| format!("0x{:x}", i)).collect();
        self.client.rpc("trace_get", (tx_hash, hex_indices)).await
    }

    /// Trace a raw transaction without executing
    pub async fn raw_transaction(
        &self,
        raw_tx: &str,
        trace_types: &[TraceType],
    ) -> Result<TraceCallResponse> {
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
    pub async fn filter(&self, filter: &TraceFilter) -> Result<Vec<Trace>> {
        self.client.rpc("trace_filter", vec![filter]).await
    }
}
