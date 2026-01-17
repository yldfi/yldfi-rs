//! Utils/Contract API client

use super::types::*;
use crate::client::Client;
use crate::error::Result;
use serde::Serialize;

/// Query parameters for utils endpoints
#[derive(Debug, Default, Serialize)]
pub struct UtilsQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_block: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_block: Option<i64>,
}

impl UtilsQuery {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn chain(mut self, chain: impl Into<String>) -> Self {
        self.chain = Some(chain.into());
        self
    }

    #[must_use]
    pub fn function_name(mut self, name: impl Into<String>) -> Self {
        self.function_name = Some(name.into());
        self
    }

    #[must_use]
    pub fn cursor(mut self, cursor: impl Into<String>) -> Self {
        self.cursor = Some(cursor.into());
        self
    }

    #[must_use]
    pub fn limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    #[must_use]
    pub fn from_block(mut self, block: i64) -> Self {
        self.from_block = Some(block);
        self
    }

    #[must_use]
    pub fn to_block(mut self, block: i64) -> Self {
        self.to_block = Some(block);
        self
    }
}

/// API for utility and contract operations
pub struct UtilsApi<'a> {
    client: &'a Client,
}

impl<'a> UtilsApi<'a> {
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Run a read-only contract function
    pub async fn run_contract_function(
        &self,
        address: &str,
        request: &RunContractFunctionRequest,
        query: Option<&UtilsQuery>,
    ) -> Result<serde_json::Value> {
        let path = format!("/{}/function", address);
        if let Some(q) = query {
            self.client.post_with_query(&path, request, q).await
        } else {
            self.client.post(&path, request).await
        }
    }

    /// Get web3 API version
    pub async fn get_web3_version(&self) -> Result<Web3Version> {
        self.client.get("/web3/version").await
    }

    /// Get endpoint weights/costs
    pub async fn get_endpoint_weights(&self) -> Result<Vec<EndpointWeight>> {
        self.client.get("/info/endpointWeights").await
    }

    /// Get contract events by topic
    pub async fn get_contract_events(
        &self,
        address: &str,
        request: &GetContractEventsRequest,
        query: Option<&UtilsQuery>,
    ) -> Result<ContractEventsResponse> {
        let path = format!("/{}/events", address);
        if let Some(q) = query {
            self.client.post_with_query(&path, request, q).await
        } else {
            self.client.post(&path, request).await
        }
    }

    /// Get contract logs
    pub async fn get_contract_logs(
        &self,
        address: &str,
        query: Option<&UtilsQuery>,
    ) -> Result<ContractEventsResponse> {
        let path = format!("/{}/logs", address);
        if let Some(q) = query {
            self.client.get_with_query(&path, q).await
        } else {
            self.client.get(&path).await
        }
    }

    /// Review contracts for security and verification status
    pub async fn review_contracts(
        &self,
        request: &ContractReviewRequest,
    ) -> Result<Vec<ContractReview>> {
        self.client.post("/contracts-review", request).await
    }
}
