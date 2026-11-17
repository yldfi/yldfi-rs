//! Real-world asset (RWA) data (Pro)
//!
//! Tokenized real-world assets with their Onchain AUM, Active AUM and
//! `DeFi` Active TVL by chain, aggregates, and charts. See [`RwaApi`].
//!
//! **All endpoints require a Pro API key.**

mod api;
mod types;

pub use api::RwaApi;
pub use types::*;
