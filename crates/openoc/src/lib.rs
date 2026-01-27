//! Rust client for the `OpenOcean` DEX Aggregator API
//!
//! `OpenOcean` is a multi-chain DEX aggregator that provides optimal swap routes
//! across 40+ chains and hundreds of DEXs.
//!
//! # Quick Start
//!
//! ```no_run
//! use openoc::{Client, Chain, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), openoc::Error> {
//!     let client = Client::new()?;
//!
//!     // Get a quote for swapping 1 ETH to USDC
//!     let request = QuoteRequest::new(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 ETH in wei
//!     ).with_slippage(1.0);
//!
//!     let quote = client.get_quote(Chain::Eth, &request).await?;
//!     println!("You will receive: {} USDC", quote.out_amount);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Getting Transaction Data
//!
//! To execute a swap, use `get_swap_quote` which returns transaction data:
//!
//! ```no_run
//! use openoc::{Client, Chain, SwapRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), openoc::Error> {
//!     let client = Client::new()?;
//!
//!     let request = SwapRequest::new(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
//!         "1000000000000000000",
//!         "0xYourWalletAddress",
//!     ).with_slippage(1.0);
//!
//!     let swap = client.get_swap_quote(Chain::Eth, &request).await?;
//!
//!     // Use with ethers/alloy to send transaction
//!     println!("To: {}", swap.to);
//!     println!("Data: {}", swap.data);
//!     println!("Value: {}", swap.value);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Supported Chains
//!
//! `OpenOcean` supports 40+ chains including:
//! - Ethereum, BSC, Polygon, Arbitrum, Optimism, Base
//! - Avalanche, Fantom, Gnosis, zkSync, Scroll
//! - Solana, Sui (non-EVM)
//!
//! See [`Chain`] for the full list.

pub mod client;
pub mod error;
pub mod types;

pub use client::Client;
pub use error::{Error, Result};
pub use types::{
    Chain, DexInfo, QuoteData, QuoteRequest, QuoteResponse, RoutePath, RouteSegment, SubRoute,
    SwapData, SwapRequest, SwapResponse, TokenInfo,
};

// Re-export common utilities
pub use yldfi_common::api::{ApiConfig, BaseClient};
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the `OpenOcean` API
pub const DEFAULT_BASE_URL: &str = "https://open-api.openocean.finance/v4";

/// Configuration for the `OpenOcean` API client
pub type Config = ApiConfig;

/// Create a default `OpenOcean` config
#[must_use]
pub fn default_config() -> Config {
    ApiConfig::new(DEFAULT_BASE_URL)
}
