//! Pre-IPO company data (Pro)
//!
//! Valuation estimates, funding rounds and tradeable markets of private
//! companies. See [`PreIpoApi`].
//!
//! **All endpoints require a Pro API key.**

mod api;
mod types;

pub use api::PreIpoApi;
pub use types::*;
