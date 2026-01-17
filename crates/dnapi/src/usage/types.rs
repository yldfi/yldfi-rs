//! Types for the Usage API

use serde::{Deserialize, Serialize};

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
