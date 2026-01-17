//! Wallet API implementation

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Wallet API for smart wallet operations
pub struct WalletApi<'a> {
    client: &'a Client,
}

impl<'a> WalletApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Request or create a smart account
    pub async fn request_account(&self, params: &RequestAccountParams) -> Result<AccountResponse> {
        self.client.rpc("wallet_requestAccount", vec![params]).await
    }

    /// List accounts for a signer
    pub async fn list_accounts(
        &self,
        signer_address: Option<&str>,
        limit: Option<u32>,
    ) -> Result<ListAccountsResponse> {
        let mut params = serde_json::Map::new();
        if let Some(addr) = signer_address {
            params.insert("signerAddress".to_string(), serde_json::json!(addr));
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), serde_json::json!(l));
        }
        self.client
            .rpc(
                "wallet_listAccounts",
                vec![serde_json::Value::Object(params)],
            )
            .await
    }

    /// Prepare calls for signing
    pub async fn prepare_calls(
        &self,
        request: &PrepareCallsRequest,
    ) -> Result<PreparedCallsResponse> {
        self.client.rpc("wallet_prepareCalls", vec![request]).await
    }

    /// Send signed prepared calls
    pub async fn send_prepared_calls(
        &self,
        request: &SendPreparedCallsRequest,
    ) -> Result<SendPreparedCallsResponse> {
        self.client
            .rpc("wallet_sendPreparedCalls", vec![request])
            .await
    }

    /// Get status of submitted calls
    pub async fn get_calls_status(&self, call_id: &str) -> Result<CallsStatusResponse> {
        self.client
            .rpc("wallet_getCallsStatus", vec![call_id])
            .await
    }

    /// Get wallet capabilities
    pub async fn get_capabilities(
        &self,
        wallet_address: &str,
        chain_ids: Option<&[&str]>,
    ) -> Result<std::collections::HashMap<String, WalletCapabilities>> {
        let mut params = vec![serde_json::json!(wallet_address)];
        if let Some(chains) = chain_ids {
            params.push(serde_json::json!(chains));
        }
        self.client.rpc("wallet_getCapabilities", params).await
    }

    /// Create a session with permissions
    pub async fn create_session(
        &self,
        request: &CreateSessionRequest,
    ) -> Result<CreateSessionResponse> {
        self.client.rpc("wallet_createSession", vec![request]).await
    }

    /// Format a signature for the wallet
    pub async fn format_sign(
        &self,
        from: &str,
        chain_id: &str,
        signature: &Signature,
    ) -> Result<String> {
        let params = serde_json::json!({
            "from": from,
            "chainId": chain_id,
            "signature": signature
        });
        self.client.rpc("wallet_formatSign", vec![params]).await
    }

    /// Prepare a sign request
    pub async fn prepare_sign(
        &self,
        from: &str,
        chain_id: &str,
        signature_request: &serde_json::Value,
    ) -> Result<SignatureRequest> {
        let params = serde_json::json!({
            "from": from,
            "chainId": chain_id,
            "signatureRequest": signature_request
        });
        self.client.rpc("wallet_prepareSign", vec![params]).await
    }

    /// Get cross-chain status
    pub async fn get_cross_chain_status(&self, call_id: &str) -> Result<serde_json::Value> {
        self.client
            .rpc("wallet_getCrossChainStatus_v0", vec![call_id])
            .await
    }

    /// Request a swap quote
    pub async fn request_quote(
        &self,
        from: &str,
        from_token: &str,
        to_token: &str,
        chain_id: &str,
        from_amount: &str,
        slippage: &str,
    ) -> Result<serde_json::Value> {
        let params = serde_json::json!({
            "from": from,
            "fromToken": from_token,
            "toToken": to_token,
            "chainId": chain_id,
            "fromAmount": from_amount,
            "slippage": slippage
        });
        self.client
            .rpc("wallet_requestQuote_v0", vec![params])
            .await
    }
}
