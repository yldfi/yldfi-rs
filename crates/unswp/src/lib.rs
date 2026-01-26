#![warn(missing_docs)]

//! # unswp
//!
//! Unofficial Rust client for Uniswap V2, V3, and V4, combining on-chain lens queries
//! and subgraph historical data.
//!
//! ## Features
//!
//! - **Multi-version support** - V2, V3, and V4 protocols
//! - **On-chain queries** via ephemeral lens contracts (no API key required)
//! - **Historical data** via The Graph subgraph (optional, requires API key)
//! - **Unified client** that combines both data sources
//!
//! ## Quick Start
//!
//! ### On-chain only (no API key needed)
//!
//! ```no_run
//! # async fn example() -> unswp::error::Result<()> {
//! use unswp::Client;
//! use unswp::lens::pools;
//!
//! let client = Client::mainnet("https://eth.llamarpc.com")?;
//!
//! // Get current pool state
//! let state = client.get_pool_state(pools::MAINNET_WETH_USDC_005).await?;
//! println!("Current tick: {}", state.tick);
//! println!("Liquidity: {}", client.get_liquidity(pools::MAINNET_WETH_USDC_005).await?);
//! # Ok(())
//! # }
//! ```
//!
//! ### With historical data (requires The Graph API key)
//!
//! ```no_run
//! # async fn example() -> unswp::error::Result<()> {
//! use unswp::Client;
//!
//! let client = Client::mainnet_with_subgraph(
//!     "https://eth.llamarpc.com",
//!     "your-graph-api-key"
//! )?;
//!
//! // On-chain query
//! let block = client.get_block_number().await?;
//! println!("Current block: {}", block);
//!
//! // Historical query (requires API key)
//! let eth_price = client.get_eth_price().await?;
//! println!("ETH price: ${:.2}", eth_price);
//!
//! let top_pools = client.get_top_pools(10).await?;
//! for pool in top_pools {
//!     println!("{}/{}: ${}", pool.token0.symbol, pool.token1.symbol, pool.total_value_locked_usd);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Module Organization
//!
//! - [`client`] - Unified client combining on-chain and subgraph
//! - [`lens`] - On-chain queries via ephemeral contracts
//! - [`subgraph`] - Historical data via The Graph
//! - [`types`] - Data types for pools, swaps, etc.
//! - [`error`] - Error types

pub mod client;
pub mod error;
pub mod lens;
pub mod subgraph;
pub mod types;

// Re-export main types at crate root
pub use client::{Client, Config};
pub use error::{Error, Result};
pub use types::{
    PairDataV2, Pool, PoolData, PoolDataV4, PoolDayData, PoolState, Quote, Swap, Token,
};

// Re-export commonly used items from submodules
pub use lens::{factories, pools, tokens, LensClient};
pub use subgraph::{subgraph_ids, SubgraphClient, SubgraphConfig, UniswapVersion};

// Re-export SDK crates for direct access
pub use uniswap_sdk_core as sdk_core;
pub use uniswap_v2_sdk as v2_sdk;
pub use uniswap_v3_sdk as v3_sdk;
pub use uniswap_v4_sdk as v4_sdk;
