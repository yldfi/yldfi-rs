//! Schedules API implementation

use super::types::{
    DeleteScheduleResponse, ListSchedulesResponse, ScheduleRequest, SetScheduleResponse,
};
use crate::client::Client;
use crate::error::Result;
use crate::PageOptions;

/// Schedules API
pub struct SchedulesApi<'a> {
    client: &'a Client,
}

impl<'a> SchedulesApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List the schedules owned by the API key's account: scheduled query
    /// runs, dashboard refreshes and materialized view refreshes
    ///
    /// Calls `GET /v1/schedules` (default page size 20, max 100).
    pub async fn list(&self, page: &PageOptions) -> Result<ListSchedulesResponse> {
        let url = self.client.url("/v1/schedules");
        let request = self.client.http().get(&url).query(&page.to_query_pairs());
        self.client.send_json(request, None).await
    }

    /// Run a saved query on a schedule, creating the schedule or replacing
    /// all settings of the existing one
    ///
    /// Calls `PUT /v1/queries/{query_id}/schedule`. **Every run is charged
    /// credits** at `request.performance`. Needs the query's owner, or an
    /// editor or admin of the owning team.
    pub async fn set_query_schedule(
        &self,
        query_id: i64,
        request: &ScheduleRequest,
    ) -> Result<SetScheduleResponse> {
        let url = self.client.url(&format!("/v1/queries/{query_id}/schedule"));
        self.client
            .send_json(
                self.client.http().put(&url).json(request),
                Some(format!("Query {query_id}")),
            )
            .await
    }

    /// Stop running a saved query on a schedule
    ///
    /// Calls `DELETE /v1/queries/{query_id}/schedule`. **Alerts that depend
    /// on the schedule are deleted with it.** `deleted` is `false` when the
    /// query had no schedule.
    pub async fn delete_query_schedule(&self, query_id: i64) -> Result<DeleteScheduleResponse> {
        let url = self.client.url(&format!("/v1/queries/{query_id}/schedule"));
        self.client
            .send_json(
                self.client.http().delete(&url),
                Some(format!("Query {query_id}")),
            )
            .await
    }

    /// Refresh a dashboard's queries on a schedule, creating the schedule or
    /// replacing all settings of the existing one
    ///
    /// Calls `PUT /v1/dashboards/{dashboard_id}/schedule`. **Every refresh is
    /// charged credits** at `request.performance`. Needs the dashboard's
    /// owner, or an editor or admin of the owning team.
    pub async fn set_dashboard_schedule(
        &self,
        dashboard_id: i64,
        request: &ScheduleRequest,
    ) -> Result<SetScheduleResponse> {
        let url = self
            .client
            .url(&format!("/v1/dashboards/{dashboard_id}/schedule"));
        self.client
            .send_json(
                self.client.http().put(&url).json(request),
                Some(format!("Dashboard {dashboard_id}")),
            )
            .await
    }

    /// Stop refreshing a dashboard on a schedule
    ///
    /// Calls `DELETE /v1/dashboards/{dashboard_id}/schedule`. `deleted` is
    /// `false` when the dashboard had no schedule.
    pub async fn delete_dashboard_schedule(
        &self,
        dashboard_id: i64,
    ) -> Result<DeleteScheduleResponse> {
        let url = self
            .client
            .url(&format!("/v1/dashboards/{dashboard_id}/schedule"));
        self.client
            .send_json(
                self.client.http().delete(&url),
                Some(format!("Dashboard {dashboard_id}")),
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
    use wiremock::matchers::{body_json, method, path, query_param, query_param_is_missing};
    use wiremock::Mock;

    fn set_json(kind: &str) -> serde_json::Value {
        json!({
            "created": true,
            "type": kind,
            "query_id": 1234,
            "cron_expression": "0 */6 * * *",
            "performance": "medium",
            "next_execution_time": "2026-01-01T06:00:00Z"
        })
    }

    #[tokio::test]
    async fn list_reads_schedules() {
        let (_server, client) = serve(
            Mock::given(method("GET"))
                .and(path("/v1/schedules"))
                .and(query_param("limit", "50"))
                .and(query_param_is_missing("offset")),
            json!({
                "schedules": [
                    {"type": "query", "query_id": 1234, "cron_expression": "0 * * * *", "performance": "small"},
                    {"type": "dashboard", "dashboard_id": 42, "cron_expression": "30 8 * * 1",
                     "performance": "medium", "expires_at": "2026-06-01T00:00:00Z"}
                ],
                "total": 2
            }),
        )
        .await;
        let r = client
            .schedules()
            .list(&PageOptions::new(Some(50), None))
            .await
            .unwrap();
        assert_eq!(r.total, Some(2));
        assert_eq!(r.schedules[0].query_id, Some(1234));
        assert_eq!(r.schedules[1].schedule_type.as_deref(), Some("dashboard"));
        assert_eq!(r.schedules[1].dashboard_id, Some(42));
    }

    #[tokio::test]
    async fn set_query_schedule_puts_body() {
        let (_server, client) = serve(
            Mock::given(method("PUT"))
                .and(path("/v1/queries/1234/schedule"))
                .and(body_json(json!({
                    "cron_expression": "0 */6 * * *",
                    "performance": "medium",
                    "expires_at": "2026-06-01T00:00:00Z"
                }))),
            set_json("query"),
        )
        .await;
        let request =
            ScheduleRequest::new("0 */6 * * *", "medium").expires_at("2026-06-01T00:00:00Z");
        let r = client
            .schedules()
            .set_query_schedule(1234, &request)
            .await
            .unwrap();
        assert_eq!(r.created, Some(true));
        assert_eq!(r.query_id, Some(1234));
    }

    #[tokio::test]
    async fn set_query_schedule_maps_404() {
        let (_server, client) = failing(404).await;
        let err = client
            .schedules()
            .set_query_schedule(1234, &ScheduleRequest::new("0 * * * *", "small"))
            .await
            .unwrap_err();
        assert!(is_not_found(&err), "{err}");
    }

    #[tokio::test]
    async fn delete_query_schedule_reads_deleted() {
        let (_server, client) = serve(
            Mock::given(method("DELETE")).and(path("/v1/queries/1234/schedule")),
            json!({"deleted": false}),
        )
        .await;
        let r = client
            .schedules()
            .delete_query_schedule(1234)
            .await
            .unwrap();
        assert_eq!(r.deleted, Some(false));
    }

    #[tokio::test]
    async fn set_dashboard_schedule_puts_body() {
        let (_server, client) = serve(
            Mock::given(method("PUT"))
                .and(path("/v1/dashboards/42/schedule"))
                .and(body_json(json!({
                    "cron_expression": "0 * * * *",
                    "performance": "small"
                }))),
            json!({
                "created": false, "type": "dashboard", "dashboard_id": 42,
                "cron_expression": "0 * * * *", "performance": "small"
            }),
        )
        .await;
        let r = client
            .schedules()
            .set_dashboard_schedule(42, &ScheduleRequest::new("0 * * * *", "small"))
            .await
            .unwrap();
        assert_eq!(r.created, Some(false));
        assert_eq!(r.dashboard_id, Some(42));
    }

    #[tokio::test]
    async fn delete_dashboard_schedule_reads_deleted() {
        let (_server, client) = serve(
            Mock::given(method("DELETE")).and(path("/v1/dashboards/42/schedule")),
            json!({"deleted": true}),
        )
        .await;
        let r = client
            .schedules()
            .delete_dashboard_schedule(42)
            .await
            .unwrap();
        assert_eq!(r.deleted, Some(true));
    }
}
