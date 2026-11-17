//! Custom Endpoints API implementation

use crate::client::Client;
use crate::error::Result;
use crate::executions::{ExecutionResult, GetResultsOptions};

/// Custom Endpoints API
pub struct EndpointsApi<'a> {
    client: &'a Client,
}

impl<'a> EndpointsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get the latest results of a custom endpoint
    ///
    /// Calls `GET /v1/endpoints/{handle}/{endpoint_slug}/results`, where
    /// `handle` is the user or team handle that owns the endpoint.
    pub async fn results(&self, handle: &str, endpoint_slug: &str) -> Result<ExecutionResult> {
        self.results_with_options(handle, endpoint_slug, &GetResultsOptions::default())
            .await
    }

    /// Get the latest results of a custom endpoint, with paging, filtering,
    /// sorting, sampling and column selection
    ///
    /// Calls `GET /v1/endpoints/{handle}/{endpoint_slug}/results`.
    pub async fn results_with_options(
        &self,
        handle: &str,
        endpoint_slug: &str,
        options: &GetResultsOptions,
    ) -> Result<ExecutionResult> {
        let url = self
            .client
            .url(&format!("/v1/endpoints/{handle}/{endpoint_slug}/results"));
        let request = self
            .client
            .http()
            .get(&url)
            .query(&options.to_query_pairs());
        self.client
            .send_json(request, Some(format!("Endpoint {handle}/{endpoint_slug}")))
            .await
    }
}

#[cfg(test)]
mod tests {
    use crate::executions::{ExecutionState, GetResultsOptions};
    use crate::test_util::{failing, is_not_found, serve};
    use serde_json::json;
    use wiremock::matchers::{method, path, query_param};
    use wiremock::Mock;

    #[tokio::test]
    async fn results_with_options_sends_query() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/endpoints/test_team/top-pools/results"))
                .and(query_param("limit", "2"))
                .and(query_param("filters", "tvl > 100")),
            json!({
                "execution_id": "01TESTEXEC",
                "query_id": 1234,
                "state": "QUERY_STATE_COMPLETED",
                "is_execution_finished": true,
                "result": {
                    "rows": [{"pool": "a", "tvl": 200}],
                    "metadata": {"column_names": ["pool", "tvl"], "row_count": 1}
                },
                "next_offset": 2
            }),
        )
        .await;
        let options = GetResultsOptions {
            limit: Some(2),
            filters: Some("tvl > 100".into()),
            ..GetResultsOptions::default()
        };
        let r = client
            .endpoints()
            .results_with_options("test_team", "top-pools", &options)
            .await
            .unwrap();
        assert_eq!(r.state, ExecutionState::Completed);
        assert_eq!(r.query_id, Some(1234));
        let data = r.result.unwrap();
        assert_eq!(data.rows[0]["pool"], json!("a"));
        assert_eq!(r.next_offset, Some(2));
    }

    #[tokio::test]
    async fn results_maps_404_to_not_found() {
        let (_server, client) = failing(404).await;
        let err = client
            .endpoints()
            .results("test_team", "missing")
            .await
            .unwrap_err();
        assert!(is_not_found(&err), "{err}");
    }
}
