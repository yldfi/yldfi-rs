//! Rust client for prop AMM (proprietary AMM) data on Ethereum
//!
//! A prop AMM is an on-chain pool run by one market maker, who re-prices it
//! every block. The block builder applies the maker's latest price just before
//! any trade against the pool, so the maker can quote close to the live
//! market. The pools are the makers' own contracts; what a builder adds is
//! the ordering guarantee, and the taker-side data this crate reads:
//!
//! - the best quote across every prop AMM, or from one of them
//! - the list of venues with their pairs and current state overrides
//! - price levels per maker and market
//!
//! The client talks to Titan Builder's public data endpoint by default, the
//! only builder publishing this data at the time of writing. Pass another
//! base URL in the [`Config`] for a builder that serves the same methods.
//!
//! # A quote is conditional
//!
//! A quote is the maker's price in the builder's latest snapshot for the
//! block being built. A trade only gets it when it is included by a builder
//! that carries that maker's price update. In any other block the pool still
//! holds its last landed price. Treat the figure as an indication, not a firm
//! quote.
//!
//! # Quick Start
//!
//! ```no_run
//! #[tokio::main]
//! async fn main() -> Result<(), pamm::Error> {
//!     let client = pamm::Client::new()?;
//!
//!     let quote = client
//!         .quote(
//!             "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
//!             "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!             "1000000000000000000",                        // 1 WETH
//!         )
//!         .await?;
//!
//!     let venues = client.venues().await?;
//!     let maker = venues.maker_of(&quote.pamm).unwrap_or("unknown maker");
//!     println!("{} USDC units from {maker}", quote.amount_out);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::Client;
pub use error::{DomainError, Error, Result};
pub use types::{AssetPair, Book, BookSnapshot, Level, Quote, Region, Venue, VenueSnapshot};

// Re-export common utilities
pub use yldfi_common::api::{ApiConfig, BaseClient};
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL: Titan Builder's EU data endpoint
pub const DEFAULT_BASE_URL: &str = Region::Eu.titan_base_url();

/// Name of the builder whose data the default endpoints serve
pub const DEFAULT_BUILDER: &str = "titan";

/// Configuration for the prop AMM data client
pub type Config = ApiConfig;

/// Create a config for Titan Builder's default (EU) data endpoint
#[must_use]
pub fn default_config() -> Config {
    ApiConfig::new(DEFAULT_BASE_URL)
}

/// Create a config for Titan Builder's data endpoint in `region`
#[must_use]
pub fn config_for_region(region: Region) -> Config {
    ApiConfig::new(region.titan_base_url())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        assert_eq!(
            default_config().base_url,
            "https://eu.data.titanbuilder.xyz"
        );
        assert_eq!(
            config_for_region(Region::Ap).base_url,
            "https://ap.data.titanbuilder.xyz"
        );
        assert_eq!(DEFAULT_BUILDER, "titan");
    }
}
