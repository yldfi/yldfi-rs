//! Dashboards API implementation

use super::types::{
    ArchiveDashboardResponse, CreateDashboardRequest, Dashboard, UpdateDashboardRequest,
};
use crate::client::Client;
use crate::error::Result;

/// Dashboards API
pub struct DashboardsApi<'a> {
    client: &'a Client,
}

impl<'a> DashboardsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get a dashboard, with its widgets, by ID
    ///
    /// Calls `GET /v1/dashboards/{dashboard_id}`.
    pub async fn get(&self, dashboard_id: i64) -> Result<Dashboard> {
        let url = self.client.url(&format!("/v1/dashboards/{dashboard_id}"));
        self.client
            .send_json(
                self.client.http().get(&url),
                Some(format!("Dashboard {dashboard_id}")),
            )
            .await
    }

    /// Get a dashboard by its owner's handle and URL slug, as in
    /// `dune.com/{owner_handle}/{slug}`
    ///
    /// Calls `GET /v1/dashboards/by-slug/{owner_handle}/{slug}`.
    pub async fn get_by_slug(&self, owner_handle: &str, slug: &str) -> Result<Dashboard> {
        let url = self
            .client
            .url(&format!("/v1/dashboards/by-slug/{owner_handle}/{slug}"));
        self.client
            .send_json(
                self.client.http().get(&url),
                Some(format!("Dashboard {owner_handle}/{slug}")),
            )
            .await
    }

    /// Create a dashboard, optionally with visualization and text widgets
    ///
    /// Calls `POST /v1/dashboards` and returns the new dashboard.
    pub async fn create(&self, request: &CreateDashboardRequest) -> Result<Dashboard> {
        let url = self.client.url("/v1/dashboards");
        self.client
            .send_json(self.client.http().post(&url).json(request), None)
            .await
    }

    /// Update a dashboard's metadata and/or **replace** its widgets
    ///
    /// Calls `PATCH /v1/dashboards/{dashboard_id}`. Each widget list set in
    /// `request` replaces all widgets of that kind, so a partial list drops
    /// the widgets it leaves out. Returns the updated dashboard.
    pub async fn update(
        &self,
        dashboard_id: i64,
        request: &UpdateDashboardRequest,
    ) -> Result<Dashboard> {
        let url = self.client.url(&format!("/v1/dashboards/{dashboard_id}"));
        self.client
            .send_json(
                self.client.http().patch(&url).json(request),
                Some(format!("Dashboard {dashboard_id}")),
            )
            .await
    }

    /// Archive a dashboard
    ///
    /// Calls `POST /v1/dashboards/{dashboard_id}/archive`. The dashboard is
    /// hidden from the library but not deleted; it can be restored in the
    /// Dune app.
    pub async fn archive(&self, dashboard_id: i64) -> Result<ArchiveDashboardResponse> {
        let url = self
            .client
            .url(&format!("/v1/dashboards/{dashboard_id}/archive"));
        self.client
            .send_json(
                self.client.http().post(&url),
                Some(format!("Dashboard {dashboard_id}")),
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use crate::test_util::{failing, is_not_found, serve};
    use serde_json::json;
    use wiremock::matchers::{body_json, method, path};
    use wiremock::Mock;

    fn dashboard_json() -> serde_json::Value {
        json!({
            "dashboard_id": 42,
            "name": "Test dashboard",
            "slug": "test-dashboard",
            "dashboard_url": "https://dune.com/test_user/test-dashboard",
            "is_private": false,
            "tags": ["dex"],
            "visualization_widgets": [{
                "widget_id": 7, "visualization_id": 99,
                "position": {"col": 0, "row": 0, "size_x": 6, "size_y": 8}
            }],
            "text_widgets": [{"widget_id": 8, "text": "# Hello"}],
            "param_widgets": [{
                "widget_id": "p1", "key": "chain", "query_id": 1234,
                "visualization_widget_id": 7
            }]
        })
    }

    #[tokio::test]
    async fn get_reads_dashboard() {
        let (_server, client) = serve(
            Mock::given(method("GET")).and(path("/v1/dashboards/42")),
            dashboard_json(),
        )
        .await;
        let d = client.dashboards().get(42).await.unwrap();
        assert_eq!(d.dashboard_id, 42);
        assert_eq!(d.slug.as_deref(), Some("test-dashboard"));
        assert_eq!(d.visualization_widgets[0].visualization_id, Some(99));
        assert_eq!(
            d.visualization_widgets[0].position,
            Some(WidgetPosition::new(0, 0, 6, 8))
        );
        assert_eq!(d.text_widgets[0].text.as_deref(), Some("# Hello"));
        assert_eq!(d.param_widgets[0].widget_id.as_deref(), Some("p1"));
    }

    #[tokio::test]
    async fn get_maps_404_to_not_found() {
        let (_server, client) = failing(404).await;
        let err = client.dashboards().get(42).await.unwrap_err();
        assert!(is_not_found(&err), "{err}");
    }

    #[tokio::test]
    async fn get_by_slug_uses_owner_and_slug() {
        let (_server, client) = serve(
            Mock::given(method("GET")).and(path("/v1/dashboards/by-slug/test_user/test-dashboard")),
            dashboard_json(),
        )
        .await;
        let d = client
            .dashboards()
            .get_by_slug("test_user", "test-dashboard")
            .await
            .unwrap();
        assert_eq!(d.name.as_deref(), Some("Test dashboard"));
    }

    #[tokio::test]
    async fn create_sends_widgets() {
        let (_server, client) = serve(
            Mock::given(method("POST"))
                .and(path("/v1/dashboards"))
                .and(body_json(json!({
                    "name": "Test dashboard",
                    "is_private": true,
                    "visualization_ids": [99],
                    "text_widgets": [{
                        "text": "# Hello",
                        "position": {"col": 0, "row": 0, "size_x": 6, "size_y": 2}
                    }]
                }))),
            dashboard_json(),
        )
        .await;
        let request = CreateDashboardRequest::new("Test dashboard")
            .private(true)
            .visualizations(vec![99])
            .text_widgets(vec![
                TextWidgetInput::new("# Hello").at(WidgetPosition::new(0, 0, 6, 2))
            ]);
        let d = client.dashboards().create(&request).await.unwrap();
        assert_eq!(d.dashboard_id, 42);
    }

    #[tokio::test]
    async fn update_patches_only_set_fields() {
        let (_server, client) = serve(
            Mock::given(method("PATCH"))
                .and(path("/v1/dashboards/42"))
                .and(body_json(json!({
                    "name": "Renamed",
                    "tags": ["dex", "nft"],
                    "visualization_widgets": [{"visualization_id": 99}],
                    "param_widgets": [{"key": "chain", "query_id": 1234}]
                }))),
            dashboard_json(),
        )
        .await;
        let mut request = UpdateDashboardRequest::new()
            .name("Renamed")
            .tags(vec!["dex".into(), "nft".into()]);
        request.visualization_widgets = Some(vec![VisualizationWidgetInput::new(99)]);
        request.param_widgets = Some(vec![ParamWidgetInput::new("chain", 1234)]);
        let d = client.dashboards().update(42, &request).await.unwrap();
        assert_eq!(d.tags, vec!["dex".to_string()]);
    }

    #[tokio::test]
    async fn archive_posts_to_archive() {
        let (_server, client) = serve(
            Mock::given(method("POST")).and(path("/v1/dashboards/42/archive")),
            json!({"dashboard_id": 42, "ok": true}),
        )
        .await;
        let r = client.dashboards().archive(42).await.unwrap();
        assert_eq!(r.dashboard_id, Some(42));
        assert_eq!(r.ok, Some(true));
    }
}
