//! Dune SIM API client for Rust
//!
//! An unofficial Rust client for the [Dune SIM API](https://docs.sim.dune.com/).
//!
//! # Example
//!
//! ```no_run
//! # async fn example() -> dsim::error::Result<()> {
//! let client = dsim::Client::new("your-api-key")?;
//!
//! // Get supported chains
//! let chains = client.chains().list().await?;
//! for chain in chains.chains {
//!     println!("{}: {}", chain.chain_id, chain.name);
//! }
//!
//! // Get wallet balances
//! let balances = client.balances().get("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").await?;
//! for balance in balances.balances {
//!     println!("{}: {} {}", balance.chain, balance.amount, balance.symbol);
//! }
//! # Ok(())
//! # }
//! ```

mod client;
pub mod error;

pub mod activity;
pub mod balances;
pub mod chains;
pub mod collectibles;
pub mod defi;
pub mod holders;
pub mod tokens;
pub mod transactions;
pub mod webhooks;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the Dune SIM API
pub const DEFAULT_BASE_URL: &str = "https://api.sim.dune.com";

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    Config::new(api_key)
}

#[cfg(test)]
mod tests;
