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
//!     // Get NFT collection floor price
//!     let floor = client.nft().get_floor_price(
//!         "0xBC4CA0EdA7647A8aB7C2061c2E118A18a936f13D", // BAYC
//!         Some("eth"),
//!     ).await?;
//!     println!("BAYC Floor: ${:?}", floor.floor_price_usd);
//!
//!     // Resolve ENS domain
//!     let resolved = client.resolve().resolve_domain("vitalik.eth").await?;
//!     println!("vitalik.eth = {:?}", resolved.address);
//!
//!     Ok(())
//! }
//! ```

mod client;
pub mod error;

// API modules
pub mod block;
pub mod defi;
pub mod discovery;
pub mod entities;
pub mod market;
pub mod nft;
pub mod resolve;
pub mod token;
pub mod transaction;
pub mod wallet;

// Re-exports
pub use client::Client;
pub use error::Error;

// API re-exports
pub use block::{BlockApi, BlockQuery};
pub use defi::{DefiApi, DefiQuery};
pub use discovery::{DiscoveryApi, DiscoveryQuery};
pub use entities::{EntitiesApi, EntityQuery};
pub use market::{MarketApi, MarketQuery};
pub use nft::{NftApi, NftQuery};
pub use resolve::ResolveApi;
pub use token::TokenApi;
pub use transaction::{TransactionApi, TransactionQuery};
pub use wallet::{WalletApi, WalletQuery};

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, Error>;
