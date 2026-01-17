//! Rust client for the CoW Protocol (CowSwap) API
//!
//! CoW Protocol is a fully permissionless trading protocol that leverages
//! Batch Auctions as its price finding mechanism. It provides:
//! - MEV protection through batch auctions
//! - Gasless trading (fees taken from output tokens)
//! - Coincidence of Wants (CoW) for better prices
//!
//! # Quick Start
//!
//! ```no_run
//! use cowswap::{Client, Chain, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cowswap::Error> {
//!     let client = Client::new()?;
//!
//!     // Get a sell quote (exact input amount)
//!     let request = QuoteRequest::sell(
//!         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 WETH
//!         "0xYourAddress",
//!     );
//!
//!     let quote = client.get_quote(None, &request).await?;
//!     println!("You will receive: {} USDC", quote.quote.buy_amount);
//!     println!("Fee: {} WETH", quote.quote.fee_amount);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Buy Orders
//!
//! For exact output swaps, use `QuoteRequest::buy`:
//!
//! ```no_run
//! use cowswap::{Client, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), cowswap::Error> {
//!     let client = Client::new()?;
//!
//!     // Get a buy quote (exact output amount)
//!     let request = QuoteRequest::buy(
//!         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000", // 1000 USDC (6 decimals)
//!         "0xYourAddress",
//!     );
//!
//!     let quote = client.get_quote(None, &request).await?;
//!     println!("You will pay: {} WETH", quote.quote.sell_amount);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Supported Chains
//!
//! - Ethereum Mainnet
//! - Gnosis Chain (xDai)
//! - Arbitrum One
//! - Sepolia (testnet)
//!
//! # Note on Order Submission
//!
//! Getting a quote is free and doesn't require signing. However, submitting
//! an order requires signing the order data with your wallet. This crate
//! provides the types for order submission, but signing must be done
//! externally (e.g., with ethers-rs or alloy).

pub mod client;
pub mod error;
pub mod types;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use types::{
    ApiError, Chain, Order, OrderCreation, OrderKind, OrderResponse, OrderStatus, PriceQuality,
    QuoteDetails, QuoteRequest, QuoteResponse, SigningScheme, Trade,
};

// Re-export common utilities
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Create a default CowSwap config
#[must_use]
pub fn default_config() -> Config {
    Config::default()
}

/// Create a config for a specific chain
#[must_use]
pub fn config_for_chain(chain: Chain) -> Config {
    Config::default().with_chain(chain)
}
