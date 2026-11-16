//! Onchain wallet endpoints: token balances, transfers, trades and PnL
//! (Analyst plan and above)

mod api;
mod types;

pub use api::WalletsApi;
pub use types::*;
