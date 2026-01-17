//! TVL and protocol data
//!
//! Access Total Value Locked data for DeFi protocols and chains.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> dllma::error::Result<()> {
//! let client = dllma::Client::new()?;
//!
//! // Get all protocols
//! let protocols = client.tvl().protocols().await?;
//!
//! // Get specific protocol details
//! let aave = client.tvl().protocol("aave").await?;
//!
//! // Get chain TVL
//! let chains = client.tvl().chains().await?;
//! # Ok(())
//! # }
//! ```

mod api;
mod types;

pub use api::TvlApi;
pub use types::*;
