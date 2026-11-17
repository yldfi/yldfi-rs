//! Contracts API implementation

use super::types::{
    DecodeContractsRequest, DecodeContractsResponse, ListSubmissionsOptions,
    ListSubmissionsResponse,
};
use crate::client::Client;
use crate::error::Result;

/// Contracts API
pub struct ContractsApi<'a> {
    client: &'a Client,
}

impl<'a> ContractsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Submit up to 100 contracts, with their ABIs, for decoding
    ///
    /// Calls `POST /v1/contracts/decode`. Each item is validated and queued
    /// on its own; the response has one result per item, matched by index.
    /// Submissions are attributed to the user who created the API key, and
    /// `upgrade` submissions and protected namespaces go to manual review.
    pub async fn decode(
        &self,
        request: &DecodeContractsRequest,
    ) -> Result<DecodeContractsResponse> {
        let url = self.client.url("/v1/contracts/decode");
        self.client
            .send_json(self.client.http().post(&url).json(request), None)
            .await
    }

    /// List the account's decoding submissions, newest first
    ///
    /// Calls `GET /v1/contracts/submissions`. Page with `limit` and the
    /// `next_cursor` of the previous response.
    pub async fn submissions(
        &self,
        options: &ListSubmissionsOptions,
    ) -> Result<ListSubmissionsResponse> {
        let url = self.client.url("/v1/contracts/submissions");
        let request = self
            .client
            .http()
            .get(&url)
            .query(&options.to_query_pairs());
        self.client.send_json(request, None).await
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::test_util::serve;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
    use wiremock::Mock;

    #[tokio::test]
    async fn decode_sends_submissions() {
        let abi = json!([{"type": "event", "name": "Transfer", "inputs": []}]);
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/contracts/decode"))
                .and(body_json(json!({
                    "submissions": [{
                        "blockchain_name": "ethereum",
                        "address": "0x1111111111111111111111111111111111111111",
                        "project_name": "test_project",
                        "contract_name": "Token",
                        "abi": abi.clone(),
                        "idempotency_key": "k1"
                    }]
                }))),
            json!({"results": [
                {"index": 0, "submission_id": "s1", "status": "pending", "replayed": false}
            ]}),
        )
        .await;
        let request = DecodeContractsRequest::new(vec![ContractSubmission::new(
            "ethereum",
            "0x1111111111111111111111111111111111111111",
            "test_project",
            "Token",
            abi,
        )
        .idempotency_key("k1")]);
        let r = client.contracts().decode(&request).await.unwrap();
        assert_eq!(r.results[0].index, Some(0));
        assert_eq!(r.results[0].submission_id.as_deref(), Some("s1"));
        assert_eq!(r.results[0].status.as_deref(), Some("pending"));
    }

    #[tokio::test]
    async fn submissions_sends_filters() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/contracts/submissions"))
                .and(query_param("blockchain_name", "ethereum"))
                .and(query_param("status", "pending"))
                .and(query_param("limit", "5"))
                .and(query_param("cursor", "c1"))
                .and(query_param_is_missing("address")),
            json!({
                "submissions": [{
                    "id": "s1", "blockchain_name": "ethereum",
                    "address": "0x1111111111111111111111111111111111111111",
                    "project_name": "test_project", "contract_name": "Token",
                    "status": "pending", "submission_type": "new",
                    "created_at": "2026-01-01T00:00:00Z"
                }],
                "total": 6,
                "next_cursor": "c2"
            }),
        )
        .await;
        let mut options = ListSubmissionsOptions::new();
        options.blockchain_name = Some("ethereum".into());
        options.status = Some("pending".into());
        options.limit = Some(5);
        options.cursor = Some("c1".into());
        let r = client.contracts().submissions(&options).await.unwrap();
        assert_eq!(r.submissions[0].id.as_deref(), Some("s1"));
        assert_eq!(r.total, Some(6));
        assert_eq!(r.next_cursor.as_deref(), Some("c2"));
    }
}
