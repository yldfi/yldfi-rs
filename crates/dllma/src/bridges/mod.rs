//! Cross-chain bridge data (Pro)
//!
//! Access bridge volumes, transactions, and cross-chain flow data.
//!
//! **All endpoints require a Pro API key.**

mod api;
mod types;

pub use api::BridgesApi;
pub use types::*;
