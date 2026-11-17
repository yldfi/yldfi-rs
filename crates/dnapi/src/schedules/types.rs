//! Types for the Schedules API

use serde::{Deserialize, Serialize};

/// A scheduled query run, dashboard refresh or materialized view refresh
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct Schedule {
    /// What the schedule runs: `query`, `dashboard` or `materialized_view`
    #[serde(rename = "type")]
    pub schedule_type: Option<String>,
    /// Query ID, for query and materialized view schedules
    pub query_id: Option<i64>,
    /// Dashboard ID, for dashboard schedules
    pub dashboard_id: Option<i64>,
    /// 5-part cron expression in UTC
    pub cron_expression: Option<String>,
    /// Engine tier each run uses: `small`, `medium` or `large`
    pub performance: Option<String>,
    /// When the schedule stops; `None` when it does not expire
    pub expires_at: Option<String>,
    /// When the next run starts
    pub next_execution_time: Option<String>,
}

/// Response from listing schedules
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ListSchedulesResponse {
    /// Schedules
    #[serde(default)]
    pub schedules: Vec<Schedule>,
    /// Number of schedules across all pages
    pub total: Option<i64>,
}

/// Request to set a query or dashboard schedule
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ScheduleRequest {
    /// 5-part cron expression in UTC. Dune supports every 15, 20 or 30
    /// minutes (`*/15 * * * *`), hourly (`0 * * * *`), every 2, 3, 4, 6, 8 or
    /// 12 hours (`0 */6 * * *`), daily (`M H * * *`) and weekly
    /// (`M H * * D`), where M is 0 or 30.
    pub cron_expression: String,
    /// Engine tier each run uses (`small`, `medium` or `large`); must be
    /// available on your plan, and every run is charged at this tier
    pub performance: String,
    /// Stop the schedule after this time; `None` means no expiration (the
    /// free plan caps it at 3 months)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

impl ScheduleRequest {
    /// Run on `cron_expression` (UTC) at the `performance` tier
    #[must_use]
    pub fn new(cron_expression: impl Into<String>, performance: impl Into<String>) -> Self {
        Self {
            cron_expression: cron_expression.into(),
            performance: performance.into(),
            expires_at: None,
        }
    }

    /// Stop the schedule after `expires_at` (an RFC 3339 timestamp)
    #[must_use]
    pub fn expires_at(mut self, expires_at: impl Into<String>) -> Self {
        self.expires_at = Some(expires_at.into());
        self
    }
}

/// Response from setting a schedule
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct SetScheduleResponse {
    /// `true` when the request created the schedule, `false` when it
    /// replaced one
    pub created: Option<bool>,
    /// What the schedule runs: `query` or `dashboard`
    #[serde(rename = "type")]
    pub schedule_type: Option<String>,
    /// Query ID, for query schedules
    pub query_id: Option<i64>,
    /// Dashboard ID, for dashboard schedules
    pub dashboard_id: Option<i64>,
    /// 5-part cron expression in UTC
    pub cron_expression: Option<String>,
    /// Engine tier each run uses
    pub performance: Option<String>,
    /// When the schedule stops; `None` when it does not expire
    pub expires_at: Option<String>,
    /// When the next run starts
    pub next_execution_time: Option<String>,
}

/// Response from deleting a schedule
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DeleteScheduleResponse {
    /// `false` when there was no schedule to delete
    pub deleted: Option<bool>,
}
