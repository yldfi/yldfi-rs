//! Pipelines API implementation

use super::types::{
    ExecutePipelineRequest, ExecutePipelineResponse, ExecuteQueryPipelineRequest,
    PipelineExecutionStatus, QueryPipeline, QueryPipelineOptions,
};
use crate::client::Client;
use crate::error::{self, Result};

/// Pipelines API
pub struct PipelinesApi<'a> {
    client: &'a Client,
}

impl<'a> PipelinesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Execute a pipeline
    pub async fn execute(
        &self,
        request: &ExecutePipelineRequest,
    ) -> Result<ExecutePipelineResponse> {
        let url = format!("{}/v1/pipelines/execute", self.client.base_url());
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 402 {
            Err(error::insufficient_credits())
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Get pipeline execution status
    pub async fn status(&self, pipeline_execution_id: &str) -> Result<PipelineExecutionStatus> {
        let url = format!(
            "{}/v1/pipelines/executions/{}/status",
            self.client.base_url(),
            pipeline_execution_id
        );
        let response = self.client.http().get(&url).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!(
                "Pipeline execution {pipeline_execution_id}"
            )))
        } else {
            Err(error::handle_error_response(response).await)
        }
    }

    /// Build the pipeline of a query: the query plus every materialized view
    /// it depends on, in dependency order
    ///
    /// Calls `GET /v1/query/{query_id}/pipeline`. Nothing runs; pass the
    /// returned pipeline to [`execute`](Self::execute), or use
    /// [`execute_for_query`](Self::execute_for_query) to do both.
    pub async fn for_query(
        &self,
        query_id: i64,
        options: &QueryPipelineOptions,
    ) -> Result<QueryPipeline> {
        let url = self.client.url(&format!("/v1/query/{query_id}/pipeline"));
        let request = self
            .client
            .http()
            .get(&url)
            .query(&options.to_query_pairs());
        self.client
            .send_json(request, Some(format!("Query {query_id}")))
            .await
    }

    /// Build a query's pipeline and execute it
    ///
    /// Calls `POST /v1/query/{query_id}/pipeline/execute`. Every node runs,
    /// and is charged credits, at `request.performance`. Follow it with
    /// [`status`](Self::status).
    pub async fn execute_for_query(
        &self,
        query_id: i64,
        request: &ExecuteQueryPipelineRequest,
    ) -> Result<ExecutePipelineResponse> {
        let url = self
            .client
            .url(&format!("/v1/query/{query_id}/pipeline/execute"));
        let response = self.client.http().post(&url).json(request).send().await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else if response.status() == 404 {
            Err(error::not_found(format!("Query {query_id}")))
        } else if response.status() == 402 {
            Err(error::insufficient_credits())
        } else {
            Err(error::handle_error_response(response).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::test_util::{failing, serve};
    use serde_json::json;
    use std::collections::HashMap;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::Mock;

    #[tokio::test]
    async fn for_query_sends_options_and_reads_pipeline() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/query/1234/pipeline"))
                .and(query_param("performance", "medium"))
                .and(query_param("query_parameters", r#"{"chain":"ethereum"}"#)),
            json!({"pipeline": {"nodes": [
                {"id": 1, "materialized_view_refresh": {"name": "dune.test_team.result_mv", "performance": "medium"}},
                {"id": 2, "dependencies": [1], "query_execution": {"query_id": 1234, "performance": "medium"}}
            ]}}),
        )
        .await;
        let mut options = QueryPipelineOptions::new();
        options.performance = Some("medium".into());
        options.query_parameters = Some(HashMap::from([("chain".to_string(), json!("ethereum"))]));
        let r = client.pipelines().for_query(1234, &options).await.unwrap();
        let nodes = &r.pipeline.nodes;
        assert_eq!(
            nodes[0].materialized_view_refresh.as_ref().unwrap().name,
            "dune.test_team.result_mv"
        );
        assert_eq!(nodes[1].dependencies, vec![1]);
        assert_eq!(nodes[1].query_execution.as_ref().unwrap().query_id, 1234);
    }

    #[tokio::test]
    async fn execute_for_query_sends_body() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/query/1234/pipeline/execute"))
                .and(body_json(json!({
                    "performance": "large",
                    "query_parameters": {"chain": "ethereum"}
                }))),
            json!({"pipeline_execution_id": "01TESTPIPE"}),
        )
        .await;
        let request = ExecuteQueryPipelineRequest::new("large").param("chain", json!("ethereum"));
        let r = client
            .pipelines()
            .execute_for_query(1234, &request)
            .await
            .unwrap();
        assert_eq!(r.pipeline_execution_id.as_deref(), Some("01TESTPIPE"));
    }

    #[tokio::test]
    async fn execute_for_query_maps_402() {
        let (_server, client) = failing(402).await;
        let err = client
            .pipelines()
            .execute_for_query(1234, &ExecuteQueryPipelineRequest::new("small"))
            .await
            .unwrap_err();
        assert!(
            matches!(
                err,
                crate::error::ApiError::Domain(crate::error::DomainError::InsufficientCredits)
            ),
            "{err}"
        );
    }
}
