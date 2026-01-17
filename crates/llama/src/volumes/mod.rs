//! Trading volume data (DEX, Options, Derivatives)
//!
//! Access trading volumes across DEXes, options protocols, and derivatives platforms.
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> llama::error::Result<()> {
//! let client = llama::Client::new()?;
//!
//! // Get DEX volume overview
//! let dex = client.volumes().dex_overview().await?;
//!
//! // Get specific protocol volume
//! let uniswap = client.volumes().dex_protocol("uniswap").await?;
//! # Ok(())
//! # }
//! ```

mod api;
mod types;

pub use api::VolumesApi;
pub use types::*;
