//! TradFi crypto ETF data (Pro)
//!
//! Access Bitcoin and Ethereum ETF data, flows, and performance metrics.
//!
//! **All endpoints require a Pro API key.**

mod api;
mod types;

pub use api::EtfApi;
pub use types::*;
