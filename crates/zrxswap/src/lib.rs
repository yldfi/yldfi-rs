//! Rust client for the 0x (ZeroEx) DEX Aggregator API v2
//!
//! This crate provides a type-safe Rust client for the 0x Swap API, a professional-grade
//! DEX aggregator that finds optimal swap routes across multiple DEXs on Ethereum and
//! EVM-compatible chains.
//!
//! # Features
//!
//! - Multi-chain support (Ethereum, Polygon, Arbitrum, Optimism, Base, BSC, and more)
//! - Professional-grade liquidity aggregation across 100+ DEXs
//! - Permit2 integration for efficient token approvals
//! - Gasless trading support
//! - MEV protection options
//! - Type-safe request/response handling
//!
//! # API Key
//!
//! An API key is required for production use. Get one at:
//! <https://0x.org/docs/introduction/getting-started>
//!
//! # Quick Start
//!
//! ```no_run
//! use zrxswap::{Client, Chain, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), zrxswap::Error> {
//!     // Create a client with your API key
//!     let client = Client::with_api_key("your-api-key")?;
//!
//!     // Get an indicative price for swapping 1 ETH to USDC
//!     let request = QuoteRequest::sell(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 ETH in wei
//!     );
//!
//!     let price = client.get_price(Chain::Ethereum, &request).await?;
//!     println!("You would receive: {} USDC", price.buy_amount);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Getting a Full Quote with Transaction Data
//!
//! To execute a swap, you need a full quote with transaction data.
//! This requires a taker address:
//!
//! ```no_run
//! use zrxswap::{Client, Chain, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), zrxswap::Error> {
//!     let client = Client::with_api_key("your-api-key")?;
//!
//!     let request = QuoteRequest::sell(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
//!         "1000000000000000000",
//!     )
//!     .with_taker("0xYourWalletAddress")
//!     .with_slippage_bps(100); // 1% slippage tolerance
//!
//!     let quote = client.get_quote(Chain::Ethereum, &request).await?;
//!
//!     // The quote contains transaction data ready for signing
//!     if let Some(tx) = quote.transaction {
//!         println!("To: {}", tx.to);
//!         println!("Data: {}", tx.data);
//!         println!("Value: {}", tx.value);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Exact Output Swaps
//!
//! You can also specify the exact amount you want to receive:
//!
//! ```no_run
//! use zrxswap::{Client, Chain, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), zrxswap::Error> {
//!     let client = Client::with_api_key("your-api-key")?;
//!
//!     // Buy exactly 1000 USDC with ETH
//!     let request = QuoteRequest::buy(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000", // 1000 USDC (6 decimals)
//!     )
//!     .with_taker("0xYourWalletAddress");
//!
//!     let quote = client.get_quote(Chain::Ethereum, &request).await?;
//!     println!("Cost: {} ETH", quote.sell_amount);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Excluding Liquidity Sources
//!
//! You can exclude specific DEXs from the route:
//!
//! ```no_run
//! use zrxswap::{Client, Chain, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), zrxswap::Error> {
//!     let client = Client::with_api_key("your-api-key")?;
//!
//!     // First, get available sources
//!     let sources = client.get_sources(Chain::Ethereum).await?;
//!     println!("Available sources: {:?}", sources.iter().map(|s| &s.name).collect::<Vec<_>>());
//!
//!     // Then exclude specific sources
//!     let request = QuoteRequest::sell(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
//!         "1000000000000000000",
//!     )
//!     .with_excluded_sources("Uniswap_V2,SushiSwap");
//!
//!     let price = client.get_price(Chain::Ethereum, &request).await?;
//!     Ok(())
//! }
//! ```
//!
//! # Supported Chains
//!
//! 0x supports swaps on the following chains:
//!
//! | Chain | Chain ID | Native Token |
//! |-------|----------|--------------|
//! | Ethereum | 1 | ETH |
//! | Polygon | 137 | MATIC |
//! | Arbitrum | 42161 | ETH |
//! | Optimism | 10 | ETH |
//! | Base | 8453 | ETH |
//! | BSC | 56 | BNB |
//! | Avalanche | 43114 | AVAX |
//! | Fantom | 250 | FTM |
//! | Celo | 42220 | CELO |
//! | Blast | 81457 | ETH |
//! | Linea | 59144 | ETH |
//! | Scroll | 534352 | ETH |
//! | Mantle | 5000 | MNT |
//!
//! See [`Chain`] for the complete list including testnets.
//!
//! # Error Handling with Retry
//!
//! The crate provides retry utilities for handling transient failures:
//!
//! ```no_run
//! use zrxswap::{Client, Chain, QuoteRequest, with_retry, RetryConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let client = Client::with_api_key("your-api-key").unwrap();
//!     let config = RetryConfig::default();
//!
//!     let request = QuoteRequest::sell(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
//!         "1000000000000000000",
//!     );
//!
//!     // Automatically retry on transient failures
//!     let price = with_retry(&config, || async {
//!         client.get_price(Chain::Ethereum, &request).await
//!     }).await.unwrap();
//!
//!     println!("Price: {}", price.buy_amount);
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::{Client, Config, DEFAULT_BASE_URL};
pub use error::{Error, Result};
pub use types::{
    AllowanceIssue, ApiError, BalanceIssue, Chain, LiquiditySource, Permit2Data, PriceRequest,
    PriceResponse, QuoteIssues, QuoteRequest, QuoteResponse, Route, RouteFill, RouteToken, Source,
    SourcesResponse, TokenMetadata, Transaction, ValidationError,
};

// Re-export common utilities
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Create a default 0x config
#[must_use]
pub fn default_config() -> Config {
    Config::default()
}

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    Config::with_api_key(api_key)
}

/// Native token address (used for ETH and other native tokens)
///
/// Use this address when swapping to/from the native token on any chain.
pub const NATIVE_TOKEN_ADDRESS: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

/// Wrapped ETH address on Ethereum mainnet
pub const WETH_ETHEREUM: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

/// USDC address on Ethereum mainnet
pub const USDC_ETHEREUM: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

/// USDT address on Ethereum mainnet
pub const USDT_ETHEREUM: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";

/// DAI address on Ethereum mainnet
pub const DAI_ETHEREUM: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";

/// WBTC address on Ethereum mainnet
pub const WBTC_ETHEREUM: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
