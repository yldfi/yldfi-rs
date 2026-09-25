//! Types for the Usage API

use serde::{Deserialize, Serialize};

/// Request body for `POST /v1/usage`
#[derive(Debug, Clone, Default, Serialize)]
pub struct UsageRequest {
    /// Optional start date (YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    /// Optional end date (YYYY-MM-DD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
}

impl UsageRequest {
    /// Create an empty usage request (current billing period)
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the start date (YYYY-MM-DD)
    #[must_use]
    pub fn start_date(mut self, date: &str) -> Self {
        self.start_date = Some(date.to_string());
        self
    }

    /// Set the end date (YYYY-MM-DD)
    #[must_use]
    pub fn end_date(mut self, date: &str) -> Self {
        self.end_date = Some(date.to_string());
        self
    }
}

/// Billing period
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BillingPeriod {
    /// Start date (YYYY-MM-DD)
    pub start_date: Option<String>,
    /// End date (YYYY-MM-DD)
    pub end_date: Option<String>,
    /// Credits included in plan
    pub credits_included: Option<f64>,
    /// Credits used
    pub credits_used: Option<f64>,
}

/// Usage response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsageResponse {
    /// Billing periods
    #[serde(default)]
    pub billing_periods: Vec<BillingPeriod>,
    /// Bytes allowed
    pub bytes_allowed: Option<i64>,
    /// Bytes used
    pub bytes_used: Option<i64>,
    /// Number of private dashboards
    pub private_dashboards: Option<i64>,
    /// Number of private queries
    pub private_queries: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_usage_request_serializes_to_empty_object() {
        let body = serde_json::to_string(&UsageRequest::new()).unwrap();
        assert_eq!(body, "{}");
    }

    #[test]
    fn usage_request_with_dates() {
        let req = UsageRequest::new()
            .start_date("2026-01-01")
            .end_date("2026-01-31");
        let body = serde_json::to_value(&req).unwrap();
        assert_eq!(body["start_date"], "2026-01-01");
        assert_eq!(body["end_date"], "2026-01-31");
    }
}
