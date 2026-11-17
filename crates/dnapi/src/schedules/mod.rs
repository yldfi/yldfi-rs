//! Schedules API for running saved queries and refreshing dashboards on a
//! schedule
//!
//! Every scheduled run is charged credits at the schedule's performance
//! tier. Materialized view refreshes show up in [`SchedulesApi::list`] but
//! are managed through [`crate::matviews`].

mod api;
mod types;

pub use api::SchedulesApi;
pub use types::*;
