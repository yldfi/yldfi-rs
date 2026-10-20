//! Gas Manager API implementation

use super::types::{
    CreatePolicyRequest, DataEnvelope, Erc20Context, GasPolicy, GasSponsorshipResponseV06,
    GasSponsorshipResponseV07, ListPoliciesOptions, ListPoliciesResponse, ListSponsorshipsOptions,
    ListSponsorshipsResponse, PartialUserOperationV06, PartialUserOperationV07,
    PaymasterStubDataResponse, PolicyData, PolicyStats, PolicyStatus,
    RequestPaymasterAndDataResponse, TokenQuoteResponse, UpdatePolicyRequest,
};
use crate::client::Client;
use crate::error::{self, Result};

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

    /// Request gas and paymaster data for a v0.6 `UserOperation`
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

    /// Request gas and paymaster data for a v0.7 `UserOperation`
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

    /// Request paymaster data only (no gas estimation) for a v0.6 `UserOperation`
    ///
    /// Alchemy marks `alchemy_requestPaymasterAndData` as "to be deprecated";
    /// prefer [`Self::request_gas_and_paymaster_data_v06`]
    /// (`alchemy_requestGasAndPaymasterAndData`).
    #[deprecated(
        note = "alchemy_requestPaymasterAndData is being deprecated by Alchemy; use request_gas_and_paymaster_data_v06 (alchemy_requestGasAndPaymasterAndData)"
    )]
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

    /// Request paymaster data only (no gas estimation) for a v0.7 `UserOperation`
    ///
    /// Alchemy marks `alchemy_requestPaymasterAndData` as "to be deprecated";
    /// prefer [`Self::request_gas_and_paymaster_data_v07`]
    /// (`alchemy_requestGasAndPaymasterAndData`).
    #[deprecated(
        note = "alchemy_requestPaymasterAndData is being deprecated by Alchemy; use request_gas_and_paymaster_data_v07 (alchemy_requestGasAndPaymasterAndData)"
    )]
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

    /// Get paymaster data for a signed `UserOperation`
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

    /// Request ERC-20 token quote for a `UserOperation`
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
            .map(std::string::ToString::to_string)
            .ok_or_else(|| error::rpc(-1, "Missing serializedTransaction in response"))
    }

    // ========== Admin Methods (Policy Management) ==========

    /// Build an authenticated Admin API request
    fn build_admin_request(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&impl serde::Serialize>,
    ) -> Result<reqwest::Request> {
        let url = format!("{ADMIN_BASE_URL}{path}");
        let mut request = self
            .client
            .http()
            .request(method, &url)
            .bearer_auth(self.client.access_key()?);
        if !query.is_empty() {
            request = request.query(query);
        }
        if let Some(b) = body {
            request = request.json(b);
        }
        request.build().map_err(crate::error::http)
    }

    /// Send an Admin API request and return the successful response
    async fn send_admin(&self, request: reqwest::Request) -> Result<reqwest::Response> {
        let response = self
            .client
            .http()
            .execute(request)
            .await
            .map_err(crate::error::http)?;
        if response.status().is_success() {
            Ok(response)
        } else {
            Err(crate::error::handle_error_response(response).await)
        }
    }

    /// Send an Admin API request and unwrap the `data` envelope
    async fn admin_request<R>(
        &self,
        method: reqwest::Method,
        path: &str,
        query: &[(&str, String)],
        body: Option<&impl serde::Serialize>,
    ) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let request = self.build_admin_request(method, path, query, body)?;
        let envelope: DataEnvelope<R> = self
            .send_admin(request)
            .await?
            .json()
            .await
            .map_err(crate::error::http)?;
        Ok(envelope.data)
    }

    /// Create a new gas manager policy (created in `inactive` status)
    pub async fn create_policy(&self, request: &CreatePolicyRequest) -> Result<GasPolicy> {
        let data: PolicyData = self
            .admin_request(reqwest::Method::POST, "/policy", &[], Some(request))
            .await?;
        Ok(data.policy)
    }

    /// Get a policy by ID
    pub async fn get_policy(&self, policy_id: &str) -> Result<GasPolicy> {
        let data: PolicyData = self
            .admin_request(
                reqwest::Method::GET,
                &format!("/policy/{policy_id}"),
                &[],
                None::<&()>,
            )
            .await?;
        Ok(data.policy)
    }

    /// Replace a policy's rules (`PUT /policy/{id}`)
    pub async fn update_policy(
        &self,
        policy_id: &str,
        request: &UpdatePolicyRequest,
    ) -> Result<GasPolicy> {
        let data: PolicyData = self
            .admin_request(
                reqwest::Method::PUT,
                &format!("/policy/{policy_id}"),
                &[],
                Some(request),
            )
            .await?;
        Ok(data.policy)
    }

    /// Delete a policy
    pub async fn delete_policy(&self, policy_id: &str) -> Result<()> {
        // The API returns 200 without a documented body; don't parse it.
        let request = self.build_admin_request(
            reqwest::Method::DELETE,
            &format!("/policy/{policy_id}"),
            &[],
            None::<&()>,
        )?;
        self.send_admin(request).await?;
        Ok(())
    }

    /// List the first page of policies (default page size)
    pub async fn list_policies(&self) -> Result<ListPoliciesResponse> {
        self.list_policies_with_options(&ListPoliciesOptions::default())
            .await
    }

    /// List policies with filters and `before`/`after` pagination
    pub async fn list_policies_with_options(
        &self,
        options: &ListPoliciesOptions,
    ) -> Result<ListPoliciesResponse> {
        self.admin_request(
            reqwest::Method::GET,
            "/policies",
            &list_policies_query(options),
            None::<&()>,
        )
        .await
    }

    /// Set policy status (`Active` or `Inactive`)
    pub async fn set_policy_status(
        &self,
        policy_id: &str,
        status: PolicyStatus,
    ) -> Result<GasPolicy> {
        let body = serde_json::json!({ "status": status });
        let data: PolicyData = self
            .admin_request(
                reqwest::Method::PUT,
                &format!("/policy/{policy_id}/status"),
                &[],
                Some(&body),
            )
            .await?;
        Ok(data.policy)
    }

    /// Get policy statistics
    pub async fn get_policy_stats(&self, policy_id: &str) -> Result<PolicyStats> {
        self.admin_request(
            reqwest::Method::GET,
            &format!("/policy/{policy_id}/stats/detailed"),
            &[],
            None::<&()>,
        )
        .await
    }

    /// List the first page of sponsorships for a policy (default page size)
    pub async fn list_sponsorships(&self, policy_id: &str) -> Result<ListSponsorshipsResponse> {
        self.list_sponsorships_with_options(policy_id, &ListSponsorshipsOptions::default())
            .await
    }

    /// List sponsorships for a policy with `before`/`after` pagination
    pub async fn list_sponsorships_with_options(
        &self,
        policy_id: &str,
        options: &ListSponsorshipsOptions,
    ) -> Result<ListSponsorshipsResponse> {
        self.admin_request(
            reqwest::Method::GET,
            &format!("/policy/{policy_id}/sponsorships"),
            &list_sponsorships_query(options),
            None::<&()>,
        )
        .await
    }
}

/// Query pairs for `GET /policies`
fn list_policies_query(options: &ListPoliciesOptions) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    if let Some(v) = &options.app_id {
        query.push(("appId", v.clone()));
    }
    if let Some(v) = options.limit {
        query.push(("limit", v.to_string()));
    }
    if let Some(v) = &options.before {
        query.push(("before", v.clone()));
    }
    if let Some(v) = &options.after {
        query.push(("after", v.clone()));
    }
    if let Some(v) = &options.name_filter {
        query.push(("nameFilter", v.clone()));
    }
    query
}

/// Query pairs for `GET /policy/{id}/sponsorships`
fn list_sponsorships_query(options: &ListSponsorshipsOptions) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    if let Some(v) = options.limit {
        query.push(("limit", v.to_string()));
    }
    if let Some(v) = &options.before {
        query.push(("before", v.clone()));
    }
    if let Some(v) = &options.after {
        query.push(("after", v.clone()));
    }
    query
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{Config, Network};

    fn test_client() -> Client {
        Client::with_config(Config::new("test-key", Network::EthMainnet).with_access_key("acc"))
            .unwrap()
    }

    #[test]
    fn list_policies_request_has_paging_query() {
        let client = test_client();
        let api = GasManagerApi::new(&client);
        let options = ListPoliciesOptions {
            app_id: Some("app-1".into()),
            limit: Some(25),
            after: Some("cur".into()),
            ..Default::default()
        };
        let req = api
            .build_admin_request(
                reqwest::Method::GET,
                "/policies",
                &list_policies_query(&options),
                None::<&()>,
            )
            .unwrap();
        assert_eq!(
            req.url().as_str(),
            "https://manage.g.alchemy.com/api/gasManager/policies?appId=app-1&limit=25&after=cur"
        );
        assert_eq!(req.headers().get("authorization").unwrap(), "Bearer acc");
    }

    #[test]
    fn list_sponsorships_query_pairs() {
        let options = ListSponsorshipsOptions {
            limit: Some(100),
            before: Some("b".into()),
            after: None,
        };
        assert_eq!(
            list_sponsorships_query(&options),
            vec![("limit", "100".to_string()), ("before", "b".to_string())]
        );
        assert!(list_sponsorships_query(&ListSponsorshipsOptions::default()).is_empty());
    }
}
