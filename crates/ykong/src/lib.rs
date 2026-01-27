//! Rust client for Yearn's Kong GraphQL API
//!
//! This crate provides a typed client for accessing Yearn Finance vault and strategy
//! data through the Kong GraphQL API at <https://kong.yearn.farm/api/gql>.
//!
//! # Example
//!
//! ```no_run
//! use ykong::Client;
//!
//! # async fn example() -> ykong::Result<()> {
//! let client = Client::new()?;
//!
//! // Get all vaults on Ethereum mainnet
//! let vaults = client.vaults().by_chain(1).await?;
//! println!("Found {} vaults", vaults.len());
//!
//! // Get v3 vaults only
//! let v3_vaults = client.vaults().v3_vaults().await?;
//!
//! // Get strategies for a vault
//! let strategies = client.strategies().by_vault(1, "0x...").await?;
//!
//! // Get token price
//! let price = client.prices().usd(1, "0x...").await?;
//!
//! // Get TVL history
//! let tvls = client.tvls().daily(1, "0x...", 30).await?;
//!
//! // Get vault reports
//! let reports = client.reports().vault_reports(1, "0x...").await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod prices;
pub mod reports;
pub mod strategies;
pub mod tvls;
pub mod types;
pub mod vaults;

pub use client::{Client, Config, BASE_URL};
pub use error::{Error, Result};
pub use prices::PricesApi;
pub use reports::ReportsApi;
pub use strategies::{StrategiesApi, StrategyFilter};
pub use tvls::{TvlPeriod, TvlsApi};
pub use types::*;
pub use vaults::{VaultFilter, VaultsApi};

impl Client {
    /// Access the vaults API
    #[must_use]
    pub fn vaults(&self) -> VaultsApi<'_> {
        VaultsApi::new(self)
    }

    /// Access the strategies API
    #[must_use]
    pub fn strategies(&self) -> StrategiesApi<'_> {
        StrategiesApi::new(self)
    }

    /// Access the prices API
    #[must_use]
    pub fn prices(&self) -> PricesApi<'_> {
        PricesApi::new(self)
    }

    /// Access the TVLs API
    #[must_use]
    pub fn tvls(&self) -> TvlsApi<'_> {
        TvlsApi::new(self)
    }

    /// Access the reports API
    #[must_use]
    pub fn reports(&self) -> ReportsApi<'_> {
        ReportsApi::new(self)
    }
}
