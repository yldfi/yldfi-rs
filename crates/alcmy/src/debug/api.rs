//! Debug API implementation

use super::types::{
    BlockTrace, CallFrame, PrestateTrace, TraceCallObject, TraceCallOptions, TracerOptions,
};
use crate::client::Client;
use crate::error::Result;
use serde::de::DeserializeOwned;

/// Debug API for transaction and block tracing
pub struct DebugApi<'a> {
    client: &'a Client,
}

impl<'a> DebugApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Trace a transaction by hash
    ///
    /// Returns detailed execution trace of the transaction.
    pub async fn trace_transaction(&self, tx_hash: &str) -> Result<CallFrame> {
        self.trace_transaction_with_options(tx_hash, TracerOptions::call_tracer())
            .await
    }

    /// Trace a transaction with the prestate tracer (state touched, before execution)
    pub async fn trace_transaction_prestate(&self, tx_hash: &str) -> Result<PrestateTrace> {
        self.trace_transaction_with_options(tx_hash, TracerOptions::prestate_tracer())
            .await
    }

    /// Trace a transaction with custom options
    ///
    /// The result shape depends on the tracer: [`CallFrame`] for
    /// `callTracer`, [`PrestateTrace`](super::PrestateTrace) /
    /// [`PrestateDiff`](super::PrestateDiff) for `prestateTracer`, or
    /// `serde_json::Value` for anything else.
    pub async fn trace_transaction_with_options<R: DeserializeOwned>(
        &self,
        tx_hash: &str,
        options: TracerOptions,
    ) -> Result<R> {
        self.client
            .rpc("debug_traceTransaction", (tx_hash, options))
            .await
    }

    /// Trace a call without executing it on-chain
    ///
    /// Executes a call in the context of a specific block.
    pub async fn trace_call(&self, call: &TraceCallObject, block: &str) -> Result<CallFrame> {
        self.trace_call_with_options(call, block, TraceCallOptions::default())
            .await
    }

    /// Trace a call with custom options
    ///
    /// The result type depends on the tracer, as for
    /// [`Self::trace_transaction_with_options`].
    pub async fn trace_call_with_options<R: DeserializeOwned>(
        &self,
        call: &TraceCallObject,
        block: &str,
        options: TraceCallOptions,
    ) -> Result<R> {
        self.client
            .rpc("debug_traceCall", (call, block, options))
            .await
    }

    /// Trace all transactions in a block by hash
    pub async fn trace_block_by_hash(&self, block_hash: &str) -> Result<Vec<BlockTrace>> {
        self.trace_block_by_hash_with_options(block_hash, TracerOptions::call_tracer())
            .await
    }

    /// Trace all transactions in a block by hash with options
    pub async fn trace_block_by_hash_with_options(
        &self,
        block_hash: &str,
        options: TracerOptions,
    ) -> Result<Vec<BlockTrace>> {
        self.client
            .rpc("debug_traceBlockByHash", (block_hash, options))
            .await
    }

    /// Trace all transactions in a block by number
    pub async fn trace_block_by_number(&self, block_number: &str) -> Result<Vec<BlockTrace>> {
        self.trace_block_by_number_with_options(block_number, TracerOptions::call_tracer())
            .await
    }

    /// Trace all transactions in a block by number with options
    pub async fn trace_block_by_number_with_options(
        &self,
        block_number: &str,
        options: TracerOptions,
    ) -> Result<Vec<BlockTrace>> {
        self.client
            .rpc("debug_traceBlockByNumber", (block_number, options))
            .await
    }

    /// Get RLP-encoded block
    pub async fn get_raw_block(&self, block: &str) -> Result<String> {
        self.client.rpc("debug_getRawBlock", vec![block]).await
    }

    /// Get RLP-encoded block header
    pub async fn get_raw_header(&self, block: &str) -> Result<String> {
        self.client.rpc("debug_getRawHeader", vec![block]).await
    }

    /// Get EIP-2718 binary-encoded receipts
    pub async fn get_raw_receipts(&self, block: &str) -> Result<Vec<String>> {
        self.client.rpc("debug_getRawReceipts", vec![block]).await
    }
}
