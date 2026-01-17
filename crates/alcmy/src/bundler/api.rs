//! Bundler API implementation (ERC-4337)

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Bundler API for ERC-4337 Account Abstraction
pub struct BundlerApi<'a> {
    client: &'a Client,
}

impl<'a> BundlerApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get supported entry points
    pub async fn supported_entry_points(&self) -> Result<Vec<String>> {
        self.client.rpc("eth_supportedEntryPoints", ()).await
    }

    /// Send a UserOperation
    ///
    /// Returns the UserOperation hash.
    pub async fn send_user_operation(
        &self,
        user_op: &UserOperation,
        entry_point: &str,
    ) -> Result<String> {
        self.client
            .rpc("eth_sendUserOperation", (user_op, entry_point))
            .await
    }

    /// Estimate gas for a UserOperation
    pub async fn estimate_user_operation_gas(
        &self,
        user_op: &UserOperation,
        entry_point: &str,
    ) -> Result<GasEstimation> {
        self.client
            .rpc("eth_estimateUserOperationGas", (user_op, entry_point))
            .await
    }

    /// Estimate gas with state overrides
    pub async fn estimate_user_operation_gas_with_overrides(
        &self,
        user_op: &UserOperation,
        entry_point: &str,
        state_overrides: &std::collections::HashMap<String, BundlerStateOverride>,
    ) -> Result<GasEstimation> {
        self.client
            .rpc(
                "eth_estimateUserOperationGas",
                (user_op, entry_point, state_overrides),
            )
            .await
    }

    /// Get UserOperation by hash
    pub async fn get_user_operation_by_hash(
        &self,
        user_op_hash: &str,
    ) -> Result<Option<UserOperationByHash>> {
        self.client
            .rpc("eth_getUserOperationByHash", vec![user_op_hash])
            .await
    }

    /// Get UserOperation receipt
    pub async fn get_user_operation_receipt(
        &self,
        user_op_hash: &str,
    ) -> Result<Option<UserOperationReceipt>> {
        self.client
            .rpc("eth_getUserOperationReceipt", vec![user_op_hash])
            .await
    }

    /// Get recommended max priority fee per gas
    ///
    /// Alchemy-specific method for better fee estimation.
    pub async fn max_priority_fee_per_gas(&self) -> Result<String> {
        self.client.rpc("rundler_maxPriorityFeePerGas", ()).await
    }
}
