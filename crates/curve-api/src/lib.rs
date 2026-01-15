//! Rust client for the Curve Finance APIs
//!
//! This crate provides clients for both Curve APIs:
//! - **Curve API** (`api.curve.finance`) - Pools, gauges, volumes, lending vaults
//! - **Curve Prices API** (`prices.curve.finance`) - Detailed pricing, OHLC, trades, DAO
//!
//! # Quick Start
//!
//! ```no_run
//! use curve_api::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), curve_api::Error> {
//!     // Create client for the main Curve API
//!     let client = Client::new()?;
//!
//!     // Get all pools on Ethereum
//!     let pools = client.pools().get_all_on_chain("ethereum").await?;
//!     println!("Found {} pools", pools.data.pool_data.len());
//!
//!     // Get lending vaults
//!     let vaults = client.lending().get_all().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Using the Prices API
//!
//! ```no_run
//! use curve_api::PricesClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), curve_api::Error> {
//!     let client = PricesClient::new()?;
//!
//!     // Get USD price for a token
//!     let price = client.get_usd_price("ethereum", "0x...").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod pools;
pub mod volumes;
pub mod crvusd;
pub mod lending;
pub mod tokens;
pub mod prices;

pub use client::{Client, Config};
pub use error::{Error, Result};

// Re-export common types
pub use pools::{Pool, PoolsResponse};
pub use volumes::{VolumesResponse, GaugesResponse};
pub use lending::{LendingVault, LendingVaultsResponse};
pub use tokens::{TokenInfo, TokensResponse};

// API accessors on Client
impl Client {
    /// Access the Pools API
    pub fn pools(&self) -> pools::PoolsApi<'_> {
        pools::PoolsApi::new(self)
    }

    /// Access the Volumes and APYs API
    pub fn volumes(&self) -> volumes::VolumesApi<'_> {
        volumes::VolumesApi::new(self)
    }

    /// Access the crvUSD API
    pub fn crvusd(&self) -> crvusd::CrvUsdApi<'_> {
        crvusd::CrvUsdApi::new(self)
    }

    /// Access the Lending API
    pub fn lending(&self) -> lending::LendingApi<'_> {
        lending::LendingApi::new(self)
    }

    /// Access the Tokens API
    pub fn tokens(&self) -> tokens::TokensApi<'_> {
        tokens::TokensApi::new(self)
    }
}

// Prices API client (separate base URL)
pub use prices::PricesClient;
