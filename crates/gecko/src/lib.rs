//! # gecko
//!
//! Unofficial Rust client for the CoinGecko API.
//!
//! ## Quick Start
//!
//! ```no_run
//! # async fn example() -> gecko::error::Result<()> {
//! let client = gecko::Client::new()?;
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
//! # async fn example() -> gecko::error::Result<()> {
//! let client = gecko::Client::pro("your-api-key")?;
//! let markets = client.coins().markets("usd").await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## GeckoTerminal (Onchain)
//!
//! ```no_run
//! # async fn example() -> gecko::error::Result<()> {
//! let client = gecko::Client::new()?;
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

/// Base URLs for the CoinGecko API
pub mod base_urls {
    /// Pro API
    pub const PRO: &str = "https://pro-api.coingecko.com/api/v3";
    /// Demo/Public API
    pub const DEMO: &str = "https://api.coingecko.com/api/v3";
}

/// Create a default CoinGecko config (demo mode)
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
    pub fn simple(&self) -> simple::SimpleApi<'_> {
        simple::SimpleApi::new(self)
    }

    /// Access coins endpoints
    pub fn coins(&self) -> coins::CoinsApi<'_> {
        coins::CoinsApi::new(self)
    }

    /// Access categories endpoints
    pub fn categories(&self) -> categories::CategoriesApi<'_> {
        categories::CategoriesApi::new(self)
    }

    /// Access exchanges endpoints
    pub fn exchanges(&self) -> exchanges::ExchangesApi<'_> {
        exchanges::ExchangesApi::new(self)
    }

    /// Access derivatives endpoints
    pub fn derivatives(&self) -> derivatives::DerivativesApi<'_> {
        derivatives::DerivativesApi::new(self)
    }

    /// Access NFT endpoints
    pub fn nfts(&self) -> nfts::NftsApi<'_> {
        nfts::NftsApi::new(self)
    }

    /// Access global/general endpoints
    pub fn global(&self) -> global::GlobalApi<'_> {
        global::GlobalApi::new(self)
    }

    /// Access onchain/GeckoTerminal endpoints
    pub fn onchain(&self) -> onchain::OnchainApi<'_> {
        onchain::OnchainApi::new(self)
    }

    /// Access treasury endpoints (public companies/governments holding crypto)
    pub fn treasury(&self) -> treasury::TreasuryApi<'_> {
        treasury::TreasuryApi::new(self)
    }
}
