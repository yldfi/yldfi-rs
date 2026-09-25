//! Per-token analytics and scores (the `/discovery/*` endpoints were removed by Moralis on 2026-06-04)

mod api;
mod types;

pub use api::{DiscoveryApi, DiscoveryQuery};
pub use types::*;
