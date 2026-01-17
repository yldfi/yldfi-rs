//! Yield farming and lending data (Pro)
//!
//! Access yield pools, APY data, lending/borrowing rates, and more.
//!
//! **All endpoints require a Pro API key.**
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> llama::error::Result<()> {
//! let client = llama::Client::with_api_key("your-api-key")?;
//!
//! // Get all yield pools
//! let pools = client.yields().pools().await?;
//!
//! // Get borrowing rates
//! let borrow = client.yields().pools_borrow().await?;
//! # Ok(())
//! # }
//! ```

mod api;
mod types;

pub use api::YieldsApi;
pub use types::*;
