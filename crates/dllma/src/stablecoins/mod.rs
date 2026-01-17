//! Stablecoin data and market caps
//!
//! Access stablecoin supply, dominance, and historical data.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> dllma::error::Result<()> {
//! let client = dllma::Client::new()?;
//!
//! // List all stablecoins
//! let stables = client.stablecoins().list().await?;
//!
//! // Get chain breakdown
//! let chains = client.stablecoins().chains().await?;
//! # Ok(())
//! # }
//! ```

mod api;
mod types;

pub use api::StablecoinsApi;
pub use types::*;
