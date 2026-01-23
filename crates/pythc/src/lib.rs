#![warn(missing_docs)]

//! # pyth
//!
//! Rust client for the Pyth Network Hermes API.
//!
//! Pyth Network provides real-time price feeds for crypto, equities, FX, and commodities.
//! This crate interfaces with the Hermes REST API to fetch price data.
//!
//! ## Quick Start
//!
//! ```no_run
//! # async fn example() -> pyth::error::Result<()> {
//! use pyth::Client;
//!
//! let client = Client::new()?;
//!
//! // Get ETH/USD price
//! let eth = client.get_latest_price(pyth::feed_ids::ETH_USD).await?;
//! if let Some(feed) = eth {
//!     println!("ETH/USD: ${:.2}", feed.price_f64().unwrap_or(0.0));
//! }
//!
//! // Get multiple prices at once
//! let feeds = client.get_latest_prices(&[
//!     pyth::feed_ids::BTC_USD,
//!     pyth::feed_ids::ETH_USD,
//! ]).await?;
//!
//! for feed in feeds {
//!     println!("{}: ${:.2}", feed.id, feed.price_f64().unwrap_or(0.0));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Symbol Lookup
//!
//! ```no_run
//! # async fn example() -> pyth::error::Result<()> {
//! use pyth::{Client, symbol_to_feed_id};
//!
//! let client = Client::new()?;
//!
//! if let Some(feed_id) = symbol_to_feed_id("ETH") {
//!     let price = client.get_latest_price(feed_id).await?;
//!     println!("{:?}", price);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Search for Feeds
//!
//! ```no_run
//! # async fn example() -> pyth::error::Result<()> {
//! use pyth::Client;
//!
//! let client = Client::new()?;
//!
//! // Search for feeds matching a query
//! let feeds = client.search_feeds("BTC").await?;
//! for feed in feeds {
//!     println!("{}: {:?}", feed.id, feed.attributes.symbol);
//! }
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::{base_urls, feed_ids, symbol_to_feed_id, Client, Config};
pub use error::{feed_not_found, invalid_feed_id, stale_price, Error, Result};
pub use types::{LatestPriceResponse, ParsedPriceFeed, PriceData, PriceFeedId};
pub use yldfi_common::http::HttpClientConfig;
