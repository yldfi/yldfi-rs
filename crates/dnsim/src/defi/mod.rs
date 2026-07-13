//! `DeFi` positions module (Beta)
//!
//! Dune deprecated the `DeFi` Positions endpoints on 2026-06-01 and the Sim
//! platform shuts down on 2026-08-01. The API handlers are short-circuited
//! and always return an error; the types remain for deserializing archived
//! responses. See <https://github.com/yldfi/yldfi-rs/issues/64>.

mod api;
mod types;

pub use api::{DefiApi, DEFI_POSITIONS_SUNSET_MESSAGE};
pub use types::*;
