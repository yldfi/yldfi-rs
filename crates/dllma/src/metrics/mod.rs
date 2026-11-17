//! v2 chart and metrics data (Pro)
//!
//! Time series and summaries from `/api/v2/chart/*` and
//! `/api/v2/metrics/*`: dimension metrics (fees, volumes, open interest,
//! active users, ...) at any scope, and protocol TVL, treasury, oracle,
//! fork and income statement data. See [`MetricsApi`].
//!
//! **All endpoints require a Pro API key.**

mod api;
mod types;

pub use api::MetricsApi;
pub use types::*;
