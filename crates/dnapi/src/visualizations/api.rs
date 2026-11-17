//! Visualizations API implementation

use super::types::{
    DeleteVisualizationResponse, ListVisualizationsResponse, Visualization,
    VisualizationIdResponse, VisualizationRequest,
};
use crate::client::Client;
use crate::error::Result;
use crate::PageOptions;

/// Visualizations API
pub struct VisualizationsApi<'a> {
    client: &'a Client,
}

impl<'a> VisualizationsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List the visualizations attached to a query
    ///
    /// Calls `GET /v1/queries/{query_id}/visualizations` (default page size
    /// 25, max 100).
    pub async fn list_for_query(
        &self,
        query_id: i64,
        page: &PageOptions,
    ) -> Result<ListVisualizationsResponse> {
        let url = self
            .client
            .url(&format!("/v1/queries/{query_id}/visualizations"));
        let request = self.client.http().get(&url).query(&page.to_query_pairs());
        self.client
            .send_json(request, Some(format!("Query {query_id}")))
            .await
    }

    /// Create a visualization on an existing query
    ///
    /// Calls `POST /v1/queries/{query_id}/visualizations` and returns the new
    /// visualization's ID.
    pub async fn create(
        &self,
        query_id: i64,
        request: &VisualizationRequest,
    ) -> Result<VisualizationIdResponse> {
        let url = self
            .client
            .url(&format!("/v1/queries/{query_id}/visualizations"));
        self.client
            .send_json(
                self.client.http().post(&url).json(request),
                Some(format!("Query {query_id}")),
            )
            .await
    }

    /// Get a visualization by ID
    ///
    /// Calls `GET /v1/visualizations/{visualization_id}`.
    pub async fn get(&self, visualization_id: i64) -> Result<Visualization> {
        let url = self
            .client
            .url(&format!("/v1/visualizations/{visualization_id}"));
        self.client
            .send_json(
                self.client.http().get(&url),
                Some(format!("Visualization {visualization_id}")),
            )
            .await
    }

    /// Replace a visualization's name, type, description and options
    ///
    /// Calls `PUT /v1/visualizations/{visualization_id}`. This is a full
    /// update: a description or options left `None` are not sent.
    pub async fn update(
        &self,
        visualization_id: i64,
        request: &VisualizationRequest,
    ) -> Result<VisualizationIdResponse> {
        let url = self
            .client
            .url(&format!("/v1/visualizations/{visualization_id}"));
        self.client
            .send_json(
                self.client.http().put(&url).json(request),
                Some(format!("Visualization {visualization_id}")),
            )
            .await
    }

    /// Permanently delete a visualization
    ///
    /// Calls `DELETE /v1/visualizations/{visualization_id}`. Dashboard
    /// widgets showing it lose their content.
    pub async fn delete(&self, visualization_id: i64) -> Result<DeleteVisualizationResponse> {
        let url = self
            .client
            .url(&format!("/v1/visualizations/{visualization_id}"));
        self.client
            .send_json(
                self.client.http().delete(&url),
                Some(format!("Visualization {visualization_id}")),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::test_util::{failing, is_not_found, serve};
    use crate::PageOptions;
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::Mock;

    fn options() -> serde_json::Map<String, serde_json::Value> {
        json!({"columns": {"amount": {"type": "normal"}}})
            .as_object()
            .unwrap()
            .clone()
    }

    #[tokio::test]
    async fn list_for_query_pages() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/queries/1234/visualizations"))
                .and(query_param("limit", "10"))
                .and(query_param("offset", "10")),
            json!({
                "results": [{"id": 99, "type": "table", "name": "Results",
                             "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"}],
                "total_count": 11
            }),
        )
        .await;
        let r = client
            .visualizations()
            .list_for_query(1234, &PageOptions::new(Some(10), Some(10)))
            .await
            .unwrap();
        assert_eq!(r.results[0].id, 99);
        assert_eq!(r.results[0].visualization_type.as_deref(), Some("table"));
        assert_eq!(r.total_count, Some(11));
    }

    #[tokio::test]
    async fn create_sends_body() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/queries/1234/visualizations"))
                .and(body_json(json!({
                    "name": "Results",
                    "type": "table",
                    "description": "all rows",
                    "options": {"columns": {"amount": {"type": "normal"}}}
                }))),
            json!({"id": 99}),
        )
        .await;
        let request = VisualizationRequest::new("Results", "table")
            .description("all rows")
            .options(options());
        let r = client
            .visualizations()
            .create(1234, &request)
            .await
            .unwrap();
        assert_eq!(r.id, 99);
    }

    #[tokio::test]
    async fn get_reads_visualization() {
        let (_server, client) = serve(
            Mock::given(method("GET")).and(path("/v1/visualizations/99")),
            json!({
                "id": 99, "query_id": 1234, "type": "table", "name": "Results",
                "description": "", "options": {"columns": {}},
                "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"
            }),
        )
        .await;
        let v = client.visualizations().get(99).await.unwrap();
        assert_eq!(v.query_id, Some(1234));
        assert!(v.options.unwrap().contains_key("columns"));
    }

    #[tokio::test]
    async fn get_maps_404_to_not_found() {
        let (_server, client) = failing(404).await;
        let err = client.visualizations().get(99).await.unwrap_err();
        assert!(is_not_found(&err), "{err}");
    }

    #[tokio::test]
    async fn update_puts_body() {
        let (_server, client) = serve(
            Mock::given(method("PUT"))
                .and(path("/v1/visualizations/99"))
                .and(body_json(json!({"name": "Renamed", "type": "chart"}))),
            json!({"id": 99}),
        )
        .await;
        let request = VisualizationRequest::new("Renamed", "chart");
        let r = client.visualizations().update(99, &request).await.unwrap();
        assert_eq!(r.id, 99);
    }

    #[tokio::test]
    async fn delete_reads_ok() {
        let (_server, client) = serve(
            Mock::given(method("DELETE")).and(path("/v1/visualizations/99")),
            json!({"ok": true}),
        )
        .await;
        let r = client.visualizations().delete(99).await.unwrap();
        assert_eq!(r.ok, Some(true));
    }
}
