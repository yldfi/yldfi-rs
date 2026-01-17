//! Rust client for the KyberSwap Aggregator API
//!
//! KyberSwap is a multi-chain DEX aggregator that provides optimal swap routes.
//!
//! # Quick Start
//!
//! ```no_run
//! use kybr::{Client, Chain, RouteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), kybr::Error> {
//!     let client = Client::new()?;
//!
//!     let request = RouteRequest::new(
//!         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 WETH
//!     );
//!
//!     let route = client.get_routes(Chain::Ethereum, &request).await?;
//!     println!("Output: {} USDC", route.amount_out);
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::Client;
pub use error::{Error, Result};
pub use types::{
    BuildRouteRequest, BuildRouteResponse, Chain, RouteRequest, RouteSummary, RoutesResponse,
    SwapStep, TokenInfo,
};

// Re-export common utilities
pub use yldfi_common::api::{ApiConfig, BaseClient};
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Default base URL for the KyberSwap API
pub const DEFAULT_BASE_URL: &str = "https://aggregator-api.kyberswap.com";

/// Configuration for the KyberSwap API client
pub type Config = ApiConfig;

/// Create a default Kyber config
#[must_use]
pub fn default_config() -> Config {
    ApiConfig::new(DEFAULT_BASE_URL)
}
