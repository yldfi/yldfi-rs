//! Public company (equities) data (Pro)
//!
//! Financial statements, prices, filings, onchain markets, sector and
//! industry aggregates, and the earnings calendar. See [`EquitiesApi`].
//!
//! **All endpoints require a Pro API key.**

mod api;
mod types;

pub use api::EquitiesApi;
pub use types::*;
