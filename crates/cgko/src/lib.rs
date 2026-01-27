//! # gecko
//!
//! Unofficial Rust client for the `CoinGecko` API.
//!
//! ## Quick Start
//!
//! ```no_run
//! # async fn example() -> cgko::error::Result<()> {
//! let client = cgko::Client::new()?;
//!
//! // Get Bitcoin price
//! let prices = client.simple().price(&["bitcoin"], &["usd"]).await?;
//! println!("{:?}", prices);
//!
//! // Get trending coins
//! let trending = client.global().trending().await?;
//! for item in trending.coins.iter().take(5) {
//!     println!("{}: #{}", item.item.name, item.item.market_cap_rank.unwrap_or(0));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Pro API
//!
//! ```no_run
//! # async fn example() -> cgko::error::Result<()> {
//! let client = cgko::Client::pro("your-api-key")?;
//! let markets = client.coins().markets("usd").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## `GeckoTerminal` (Onchain)
//!
//! ```no_run
//! # async fn example() -> cgko::error::Result<()> {
//! let client = cgko::Client::new()?;
//! let pools = client.onchain().trending_pools().await?;
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;

pub mod categories;
pub mod coins;
pub mod derivatives;
pub mod exchanges;
pub mod global;
pub mod nfts;
pub mod onchain;
pub mod simple;
pub mod treasury;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Base URLs for the `CoinGecko` API
pub mod base_urls {
    /// Pro API
    pub const PRO: &str = "https://pro-api.coingecko.com/api/v3";
    /// Demo/Public API
    pub const DEMO: &str = "https://api.coingecko.com/api/v3";
}

/// Create a default `CoinGecko` config (demo mode)
#[must_use]
pub fn default_config() -> Config {
    Config::demo()
}

/// Create a demo config with API key
#[must_use]
pub fn demo_config_with_key(api_key: impl Into<String>) -> Config {
    Config::demo_with_key(api_key)
}

/// Create a Pro config with API key
#[must_use]
pub fn pro_config(api_key: impl Into<String>) -> Config {
    Config::pro(api_key)
}

impl Client {
    /// Access simple price endpoints
    #[must_use]
    pub fn simple(&self) -> simple::SimpleApi<'_> {
        simple::SimpleApi::new(self)
    }

    /// Access coins endpoints
    #[must_use]
    pub fn coins(&self) -> coins::CoinsApi<'_> {
        coins::CoinsApi::new(self)
    }

    /// Access categories endpoints
    #[must_use]
    pub fn categories(&self) -> categories::CategoriesApi<'_> {
        categories::CategoriesApi::new(self)
    }

    /// Access exchanges endpoints
    #[must_use]
    pub fn exchanges(&self) -> exchanges::ExchangesApi<'_> {
        exchanges::ExchangesApi::new(self)
    }

    /// Access derivatives endpoints
    #[must_use]
    pub fn derivatives(&self) -> derivatives::DerivativesApi<'_> {
        derivatives::DerivativesApi::new(self)
    }

    /// Access NFT endpoints
    #[must_use]
    pub fn nfts(&self) -> nfts::NftsApi<'_> {
        nfts::NftsApi::new(self)
    }

    /// Access global/general endpoints
    #[must_use]
    pub fn global(&self) -> global::GlobalApi<'_> {
        global::GlobalApi::new(self)
    }

    /// Access onchain/GeckoTerminal endpoints
    #[must_use]
    pub fn onchain(&self) -> onchain::OnchainApi<'_> {
        onchain::OnchainApi::new(self)
    }

    /// Access treasury endpoints (public companies/governments holding crypto)
    #[must_use]
    pub fn treasury(&self) -> treasury::TreasuryApi<'_> {
        treasury::TreasuryApi::new(self)
    }
}
