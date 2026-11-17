//! Dashboards API for creating, reading, updating and archiving dashboards
//!
//! Scheduled dashboard refreshes are in [`crate::schedules`].

mod api;
mod types;

pub use api::DashboardsApi;
pub use types::*;
