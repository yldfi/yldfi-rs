//! Rust client for the Curve Finance APIs
//!
//! This crate provides clients for both Curve APIs:
//! - **Curve API** (`api.curve.finance`) - Pools, gauges, volumes, lending vaults
//! - **Curve Prices API** (`prices.curve.finance`) - Detailed pricing, OHLC, trades, DAO
//!
//! # Quick Start
//!
//! ```no_run
//! use crv::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), crv::Error> {
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
//! use crv::PricesClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), crv::Error> {
//!     let client = PricesClient::new()?;
//!
//!     // Get USD price for a token
//!     let price = client.get_usd_price("ethereum", "0x...").await?;
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod crvusd;
pub mod error;
pub mod lending;
pub mod pools;
pub mod prices;
pub mod router;
pub mod tokens;
pub mod volumes;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the Curve Finance API
pub const DEFAULT_BASE_URL: &str = "https://api.curve.finance/v1";

/// Create a default Curve config
#[must_use]
pub fn default_config() -> Config {
    Config::default()
}

// Re-export common types
pub use lending::{LendingVault, LendingVaultsResponse};
pub use pools::{Pool, PoolsResponse};
pub use router::{Route, RouteGraph, RouterApi, RouterStats};
pub use tokens::{TokenInfo, TokensResponse};
pub use volumes::{GaugesResponse, VolumesResponse};

// API accessors on Client
impl Client {
    /// Access the Pools API
    #[must_use] 
    pub fn pools(&self) -> pools::PoolsApi<'_> {
        pools::PoolsApi::new(self)
    }

    /// Access the Volumes and APYs API
    #[must_use] 
    pub fn volumes(&self) -> volumes::VolumesApi<'_> {
        volumes::VolumesApi::new(self)
    }

    /// Access the crvUSD API
    #[must_use] 
    pub fn crvusd(&self) -> crvusd::CrvUsdApi<'_> {
        crvusd::CrvUsdApi::new(self)
    }

    /// Access the Lending API
    #[must_use] 
    pub fn lending(&self) -> lending::LendingApi<'_> {
        lending::LendingApi::new(self)
    }

    /// Access the Tokens API
    #[must_use] 
    pub fn tokens(&self) -> tokens::TokensApi<'_> {
        tokens::TokensApi::new(self)
    }

    /// Build a router from pool data
    ///
    /// This fetches all pools for a chain and builds a route graph.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use crv::Client;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), crv::Error> {
    ///     let client = Client::new()?;
    ///     let router = client.build_router("ethereum").await?;
    ///
    ///     let routes = router.find_routes("0xdai...", "0xusdc...");
    ///     Ok(())
    /// }
    /// ```
    pub async fn build_router(&self, chain: &str) -> Result<RouterApi> {
        let pools_response = self.pools().get_all_on_chain(chain).await?;
        Ok(RouterApi::new(chain, &pools_response.data.pool_data))
    }
}

// Prices API client (separate base URL)
pub use prices::PricesClient;
