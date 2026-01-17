//! Gas Manager API implementation

use super::types::*;
use crate::client::Client;
use crate::error::{self, Error, Result};

const ADMIN_BASE_URL: &str = "https://manage.g.alchemy.com/api/gasManager";

/// Gas Manager API for gas sponsorship
pub struct GasManagerApi<'a> {
    client: &'a Client,
}

impl<'a> GasManagerApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    // ========== RPC Methods (Gas Sponsorship) ==========

    /// Request gas and paymaster data for a v0.6 UserOperation
    pub async fn request_gas_and_paymaster_data_v06(
        &self,
        policy_id: &str,
        entry_point: &str,
        user_op: &PartialUserOperationV06,
        dummy_signature: &str,
    ) -> Result<GasSponsorshipResponseV06> {
        let params = serde_json::json!({
            "policyId": policy_id,
            "entryPoint": entry_point,
            "userOperation": user_op,
            "dummySignature": dummy_signature
        });
        self.client
            .rpc("alchemy_requestGasAndPaymasterAndData", vec![params])
            .await
    }

    /// Request gas and paymaster data for a v0.7 UserOperation
    pub async fn request_gas_and_paymaster_data_v07(
        &self,
        policy_id: &str,
        entry_point: &str,
        user_op: &PartialUserOperationV07,
        dummy_signature: &str,
    ) -> Result<GasSponsorshipResponseV07> {
        let params = serde_json::json!({
            "policyId": policy_id,
            "entryPoint": entry_point,
            "userOperation": user_op,
            "dummySignature": dummy_signature
        });
        self.client
            .rpc("alchemy_requestGasAndPaymasterAndData", vec![params])
            .await
    }

    /// Request paymaster data only (no gas estimation) for a v0.6 UserOperation
    pub async fn request_paymaster_and_data_v06(
        &self,
        policy_id: &str,
        entry_point: &str,
        user_op: &PartialUserOperationV06,
    ) -> Result<RequestPaymasterAndDataResponse> {
        let params = serde_json::json!({
            "policyId": policy_id,
            "entryPoint": entry_point,
            "userOperation": user_op
        });
        self.client
            .rpc("alchemy_requestPaymasterAndData", vec![params])
            .await
    }

    /// Request paymaster data only (no gas estimation) for a v0.7 UserOperation
    pub async fn request_paymaster_and_data_v07(
        &self,
        policy_id: &str,
        entry_point: &str,
        user_op: &PartialUserOperationV07,
    ) -> Result<RequestPaymasterAndDataResponse> {
        let params = serde_json::json!({
            "policyId": policy_id,
            "entryPoint": entry_point,
            "userOperation": user_op
        });
        self.client
            .rpc("alchemy_requestPaymasterAndData", vec![params])
            .await
    }

    /// Get paymaster stub data for gas estimation
    pub async fn get_paymaster_stub_data(
        &self,
        policy_id: &str,
        entry_point: &str,
        chain_id: &str,
        user_op: &serde_json::Value,
    ) -> Result<PaymasterStubDataResponse> {
        let params = serde_json::json!([
            user_op,
            entry_point,
            chain_id,
            { "policyId": policy_id }
        ]);
        self.client.rpc("pm_getPaymasterStubData", params).await
    }

    /// Get paymaster data for a signed UserOperation
    pub async fn get_paymaster_data(
        &self,
        policy_id: &str,
        entry_point: &str,
        chain_id: &str,
        user_op: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let params = serde_json::json!([
            user_op,
            entry_point,
            chain_id,
            { "policyId": policy_id }
        ]);
        self.client.rpc("pm_getPaymasterData", params).await
    }

    /// Request ERC-20 token quote for a UserOperation
    pub async fn request_paymaster_token_quote(
        &self,
        policy_id: &str,
        entry_point: &str,
        user_op: &serde_json::Value,
        dummy_signature: &str,
        erc20_context: &Erc20Context,
    ) -> Result<TokenQuoteResponse> {
        let params = serde_json::json!({
            "policyId": policy_id,
            "entryPoint": entry_point,
            "userOperation": user_op,
            "dummySignature": dummy_signature,
            "erc20Context": erc20_context
        });
        self.client
            .rpc("alchemy_requestPaymasterTokenQuote", vec![params])
            .await
    }

    /// Request fee payer for Solana transaction
    pub async fn request_fee_payer(
        &self,
        policy_id: &str,
        serialized_transaction: &str,
    ) -> Result<String> {
        let params = serde_json::json!({
            "policyId": policy_id,
            "serializedTransaction": serialized_transaction
        });
        let response: serde_json::Value = self
            .client
            .rpc("alchemy_requestFeePayer", vec![params])
            .await?;
        response
            .get("serializedTransaction")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| error::rpc(-1, "Missing serializedTransaction in response"))
    }

    // ========== Admin Methods (Policy Management) ==========

    async fn admin_request<R>(
        &self,
        method: &str,
        path: &str,
        body: Option<&impl serde::Serialize>,
    ) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", ADMIN_BASE_URL, path);
        let request = self
            .client
            .http()
            .request(
                match method {
                    "GET" => reqwest::Method::GET,
                    "POST" => reqwest::Method::POST,
                    "PUT" => reqwest::Method::PUT,
                    "DELETE" => reqwest::Method::DELETE,
                    _ => reqwest::Method::GET,
                },
                &url,
            )
            .bearer_auth(self.client.api_key());

        let request = if let Some(b) = body {
            request.json(b)
        } else {
            request
        };

        let response = request.send().await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Create a new gas manager policy
    pub async fn create_policy(&self, request: &CreatePolicyRequest) -> Result<GasPolicy> {
        self.admin_request("POST", "/policy", Some(request)).await
    }

    /// Get a policy by ID
    pub async fn get_policy(&self, policy_id: &str) -> Result<GasPolicy> {
        self.admin_request::<GasPolicy>("GET", &format!("/policy/{}", policy_id), None::<&()>)
            .await
    }

    /// Update a policy
    pub async fn update_policy(
        &self,
        policy_id: &str,
        request: &UpdatePolicyRequest,
    ) -> Result<GasPolicy> {
        self.admin_request("PUT", &format!("/policy/{}", policy_id), Some(request))
            .await
    }

    /// Delete a policy
    pub async fn delete_policy(&self, policy_id: &str) -> Result<()> {
        let _: serde_json::Value = self
            .admin_request("DELETE", &format!("/policy/{}", policy_id), None::<&()>)
            .await?;
        Ok(())
    }

    /// List all policies
    pub async fn list_policies(&self) -> Result<ListPoliciesResponse> {
        self.admin_request("GET", "/policies", None::<&()>).await
    }

    /// Set policy status
    pub async fn set_policy_status(
        &self,
        policy_id: &str,
        status: PolicyStatus,
    ) -> Result<GasPolicy> {
        let body = serde_json::json!({ "status": status });
        self.admin_request("PUT", &format!("/policy/{}/status", policy_id), Some(&body))
            .await
    }

    /// Get policy statistics
    pub async fn get_policy_stats(&self, policy_id: &str) -> Result<PolicyStats> {
        self.admin_request(
            "GET",
            &format!("/policy/{}/stats/details", policy_id),
            None::<&()>,
        )
        .await
    }

    /// List sponsorships for a policy
    pub async fn list_sponsorships(&self, policy_id: &str) -> Result<ListSponsorshipsResponse> {
        self.admin_request(
            "GET",
            &format!("/policy/{}/sponsorships", policy_id),
            None::<&()>,
        )
        .await
    }
}
