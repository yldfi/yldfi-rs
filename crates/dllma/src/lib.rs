//! # llama
//!
//! Unofficial Rust client for the [DefiLlama API](https://defillama.com/).
//!
//! Provides access to `DeFi` protocol data including TVL, prices, yields, volumes,
//! fees, stablecoins, bridges, and more.
//!
//! ## Quick Start
//!
//! ```no_run
//! # async fn example() -> dllma::error::Result<()> {
//! use dllma::Client;
//!
//! // Create a free-tier client
//! let client = Client::new()?;
//!
//! // Get all protocols
//! let protocols = client.tvl().protocols().await?;
//! println!("Found {} protocols", protocols.len());
//!
//! // Get current ETH price
//! use dllma::coins::Token;
//! let tokens = vec![Token::coingecko("ethereum")];
//! let prices = client.coins().current(&tokens).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Pro API Access
//!
//! Some endpoints require a Pro API key. Get one at <https://defillama.com/subscription>
//!
//! ```no_run
//! # async fn example() -> dllma::error::Result<()> {
//! use dllma::Client;
//!
//! // Create client with Pro API key
//! let client = Client::with_api_key("your-api-key")?;
//!
//! // Or from environment variable
//! let client = Client::from_env()?; // reads DEFILLAMA_API_KEY
//!
//! // Now you can access Pro endpoints
//! let yields = client.yields().pools().await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Available Modules
//!
//! ### Free Endpoints
//!
//! - [`tvl`] - Protocol TVL and chain data
//! - [`coins`] - Token prices (current, historical, charts)
//! - [`stablecoins`] - Stablecoin supply and dominance
//! - [`volumes`] - DEX and options trading volumes
//! - [`fees`] - Protocol fees and revenue
//!
//! ### Free and Pro Endpoints
//!
//! - [`yields`] - Yield pools and charts (free), lending rates, perps (Pro)
//! - [`bridges`] - Cross-chain bridge volumes and transactions
//! - [`ecosystem`] - Categories, forks, oracles, treasuries, hacks, raises
//! - [`emissions`] - Token unlock schedules
//! - [`etf`] - Bitcoin and Ethereum ETF data
//! - [`dat`] - Digital Asset Treasury (institutional holdings)

pub mod bridges;
pub mod client;
pub mod coins;
pub mod dat;
pub mod ecosystem;
pub mod emissions;
pub mod error;
pub mod etf;
pub mod fees;
pub mod stablecoins;
pub mod tvl;
pub mod volumes;
pub mod yields;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Base URLs for the `DefiLlama` API
pub mod base_urls {
    /// Main API (TVL, protocols, fees, volumes) - Free tier
    pub const MAIN: &str = "https://api.llama.fi";
    /// Pro API (includes all endpoints)
    pub const PRO: &str = "https://pro-api.llama.fi";
    /// Coins/prices API
    pub const COINS: &str = "https://coins.llama.fi";
    /// Stablecoins API
    pub const STABLECOINS: &str = "https://stablecoins.llama.fi";
    /// Yields API (free tier)
    pub const YIELDS: &str = "https://yields.llama.fi";
}

/// Create a default `DefiLlama` config (free tier)
#[must_use]
pub fn default_config() -> Config {
    Config::default()
}

/// Create a config with a Pro API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    Config::with_api_key(api_key)
}

impl Client {
    /// Access TVL and protocol data endpoints
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let protocols = client.tvl().protocols().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn tvl(&self) -> tvl::TvlApi<'_> {
        tvl::TvlApi::new(self)
    }

    /// Access coin/price endpoints
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// use dllma::coins::Token;
    /// let client = dllma::Client::new()?;
    /// let prices = client.coins().current(&[Token::coingecko("ethereum")]).await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn coins(&self) -> coins::CoinsApi<'_> {
        coins::CoinsApi::new(self)
    }

    /// Access stablecoin endpoints
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let stables = client.stablecoins().list().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn stablecoins(&self) -> stablecoins::StablecoinsApi<'_> {
        stablecoins::StablecoinsApi::new(self)
    }

    /// Access trading volume endpoints (DEX, Options, Derivatives)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let dex_volume = client.volumes().dex_overview().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn volumes(&self) -> volumes::VolumesApi<'_> {
        volumes::VolumesApi::new(self)
    }

    /// Access fees and revenue endpoints
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let fees = client.fees().overview().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn fees(&self) -> fees::FeesApi<'_> {
        fees::FeesApi::new(self)
    }

    /// Access yield farming and lending endpoints
    ///
    /// Some endpoints are free (`pools`, `chart`), others require Pro API key.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::new()?;
    /// let pools = client.yields().pools().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn yields(&self) -> yields::YieldsApi<'_> {
        yields::YieldsApi::new(self)
    }

    /// Access bridge endpoints (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-key")?;
    /// let bridges = client.bridges().list().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn bridges(&self) -> bridges::BridgesApi<'_> {
        bridges::BridgesApi::new(self)
    }

    /// Access ecosystem data endpoints (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-key")?;
    /// let hacks = client.ecosystem().hacks().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn ecosystem(&self) -> ecosystem::EcosystemApi<'_> {
        ecosystem::EcosystemApi::new(self)
    }

    /// Access token emissions/unlock endpoints (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-key")?;
    /// let emissions = client.emissions().list().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn emissions(&self) -> emissions::EmissionsApi<'_> {
        emissions::EmissionsApi::new(self)
    }

    /// Access ETF data endpoints (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-key")?;
    /// let btc_etfs = client.etf().overview().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn etf(&self) -> etf::EtfApi<'_> {
        etf::EtfApi::new(self)
    }

    /// Access Digital Asset Treasury (DAT) endpoints (Pro)
    ///
    /// **Requires Pro API key**
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> dllma::error::Result<()> {
    /// let client = dllma::Client::with_api_key("your-key")?;
    /// let institutions = client.dat().institutions().await?;
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn dat(&self) -> dat::DatApi<'_> {
        dat::DatApi::new(self)
    }
}
