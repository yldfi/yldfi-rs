//! Yield farming and lending data (Pro)
//!
//! Access yield pools, APY data, lending/borrowing rates, and more.
//!
//! **All endpoints require a Pro API key.**
//!
//! The `/yields/v1/*` methods ([`YieldsApi::pools`], [`YieldsApi::chart`],
//! [`YieldsApi::pools_borrow`], ...) return `DefiLlama`'s legacy format,
//! which is frozen; the v2 methods ([`YieldsApi::earn`],
//! [`YieldsApi::borrow_markets`], [`YieldsApi::borrow_routes`],
//! [`YieldsApi::loops`], ...) return the current one.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> dllma::error::Result<()> {
//! let client = dllma::Client::with_api_key("your-api-key")?;
//!
//! // Get all yield pools
//! let pools = client.yields().pools().await?;
//!
//! // Get borrowing rates
//! let borrow = client.yields().pools_borrow().await?;
//!
//! // v2: earn pools and borrow markets
//! let earn = client.yields().earn().await?;
//! let markets = client.yields().borrow_markets().await?;
//! # Ok(())
//! # }
//! ```

mod api;
mod types;
mod v2_types;

pub use api::YieldsApi;
pub use types::*;
pub use v2_types::*;
