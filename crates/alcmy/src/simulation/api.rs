//! Transaction Simulation API implementation

use super::types::{
    ExecutionFormat, SimulateAssetChangesResponse, SimulateExecutionResponse, SimulationTransaction,
};
use crate::client::Client;
use crate::error::Result;

/// Simulation API for transaction simulation
pub struct SimulationApi<'a> {
    client: &'a Client,
}

impl<'a> SimulationApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Simulate a transaction and return asset changes
    ///
    /// Shows what tokens/NFTs would be transferred, approved, etc.
    pub async fn simulate_asset_changes(
        &self,
        tx: &SimulationTransaction,
    ) -> Result<SimulateAssetChangesResponse> {
        self.client
            .rpc("alchemy_simulateAssetChanges", vec![tx])
            .await
    }

    /// Simulate a bundle of transactions and return asset changes
    ///
    /// Transactions are executed sequentially.
    pub async fn simulate_asset_changes_bundle(
        &self,
        txs: &[SimulationTransaction],
    ) -> Result<Vec<SimulateAssetChangesResponse>> {
        self.client
            .rpc("alchemy_simulateAssetChangesBundle", vec![txs])
            .await
    }

    /// Simulate execution with decoded traces and logs
    ///
    /// Returns detailed call traces with decoded function calls and events.
    pub async fn simulate_execution(
        &self,
        tx: &SimulationTransaction,
        format: ExecutionFormat,
        block_tag: &str,
    ) -> Result<SimulateExecutionResponse> {
        let params = serde_json::json!({
            "format": format,
            "transaction": tx,
            "blockTag": block_tag
        });
        self.client
            .rpc("alchemy_simulateExecution", vec![params])
            .await
    }

    /// Simulate execution with nested call format
    pub async fn simulate_execution_nested(
        &self,
        tx: &SimulationTransaction,
    ) -> Result<SimulateExecutionResponse> {
        self.simulate_execution(tx, ExecutionFormat::Nested, "latest")
            .await
    }

    /// Simulate execution with flat call format
    pub async fn simulate_execution_flat(
        &self,
        tx: &SimulationTransaction,
    ) -> Result<SimulateExecutionResponse> {
        self.simulate_execution(tx, ExecutionFormat::Flat, "latest")
            .await
    }

    /// Simulate a bundle with decoded execution traces
    pub async fn simulate_execution_bundle(
        &self,
        txs: &[SimulationTransaction],
    ) -> Result<Vec<SimulateExecutionResponse>> {
        self.client
            .rpc("alchemy_simulateExecutionBundle", vec![txs])
            .await
    }
}
