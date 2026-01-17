//! # mrls - Moralis Web3 API Client
//!
//! A comprehensive Rust client for the [Moralis Web3 API](https://docs.moralis.io/).
//!
//! ## Features
//!
//! - **Wallet API** - Native balances, token balances, transactions, approvals, net worth, profitability
//! - **Token API** - Metadata, prices, transfers, swaps, pairs, holders, stats, trending
//! - **NFT API** - NFT metadata, transfers, owners, trades, floor prices, collections
//! - **DeFi API** - Pair prices, reserves, positions, protocol summaries
//! - **Block API** - Block data, timestamps, date-to-block lookups
//! - **Transaction API** - Transaction details, decoded calls, internal transactions
//! - **Resolve API** - ENS, Unstoppable Domains, domain resolution
//! - **Market Data API** - Top tokens, movers, NFT collections, global stats
//! - **Discovery API** - Token discovery, trending, analytics, scores
//! - **Entities API** - Wallet/protocol/exchange labels and categories
//!
//! ## Quick Start
//!
//! ```no_run
//! use mrls::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), mrls::Error> {
//!     // Create client from MORALIS_API_KEY env var
//!     let client = Client::from_env()?;
//!
//!     // Get native balance
//!     let balance = client.wallet().get_native_balance(
//!         "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
//!         Some("eth"),
//!     ).await?;
//!     println!("Balance: {} wei", balance.balance);
//!
//!     // Get token price
//!     let price = client.token().get_price(
//!         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
//!         Some("eth"),
//!     ).await?;
//!     println!("WETH Price: ${:?}", price.usd_price);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The client provides specific error types for common API errors:
//!
//! ```no_run
//! use mrls::{Client, Error};
//! use mrls::error::DomainError;
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::from_env().unwrap();
//!
//!     // Premium endpoints require Starter or Pro plan
//!     match client.discovery().get_token_score("0x...", Some("eth")).await {
//!         Ok(score) => println!("Token score: {:?}", score),
//!         Err(Error::Domain(DomainError::PlanRequired { required_plan, message })) => {
//!             println!("Upgrade to {} plan: {}", required_plan, message);
//!         }
//!         Err(Error::RateLimited { retry_after, .. }) => {
//!             println!("Rate limited, retry after {:?}", retry_after);
//!         }
//!         Err(Error::Domain(DomainError::Unauthorized)) => {
//!             println!("Invalid API key");
//!         }
//!         Err(e) => println!("Other error: {}", e),
//!     }
//! }
//! ```
//!
//! ### Plan Tiers
//!
//! Some endpoints require specific plan tiers:
//!
//! | Tier | Endpoints |
//! |------|-----------|
//! | **Free** | Most basic endpoints |
//! | **Starter** | `get_token_score` |
//! | **Pro** | Volume stats, token discovery, analytics, search |
//!
//! ## Automatic Retries
//!
//! Use the retry utilities for resilient API calls:
//!
//! ```no_run
//! use mrls::{Client, with_retry, RetryConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = Client::from_env()?;
//!     let config = RetryConfig::default(); // 3 retries with exponential backoff
//!
//!     let result = with_retry(&config, || async {
//!         client.wallet().get_native_balance(
//!             "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
//!             Some("eth"),
//!         ).await
//!     }).await?;
//!
//!     println!("Balance: {}", result.balance);
//!     Ok(())
//! }
//! ```
//!
//! Preset configurations:
//! - `RetryConfig::default()` - 3 retries, 100ms initial delay
//! - `RetryConfig::quick()` - 2 retries, 50ms initial delay (interactive)
//! - `RetryConfig::batch()` - 5 retries, 200ms initial delay (batch jobs)
//! - `RetryConfig::none()` - No retries

mod client;
pub mod error;

// API modules
pub mod analytics;
pub mod block;
pub mod defi;
pub mod discovery;
pub mod entities;
pub mod market;
pub mod nft;
pub mod resolve;
pub mod token;
pub mod transaction;
pub mod utils;
pub mod volume;
pub mod wallet;

// Re-exports
pub use client::{Client, Config};
pub use error::{Error, PlanTier};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the Moralis API
pub const DEFAULT_BASE_URL: &str = "https://deep-index.moralis.io/api/v2.2";

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    Config::new(api_key)
}

// API re-exports
pub use analytics::{AnalyticsApi, AnalyticsQuery};
pub use block::{BlockApi, BlockQuery};
pub use defi::{DefiApi, DefiQuery};
pub use discovery::{DiscoveryApi, DiscoveryQuery};
pub use entities::{EntitiesApi, EntityQuery};
pub use market::{MarketApi, MarketQuery};
pub use nft::{NftApi, NftQuery};
pub use resolve::ResolveApi;
pub use token::TokenApi;
pub use transaction::{TransactionApi, TransactionQuery};
pub use utils::{UtilsApi, UtilsQuery};
pub use volume::{VolumeApi, VolumeQuery};
pub use wallet::{WalletApi, WalletQuery};

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, Error>;
