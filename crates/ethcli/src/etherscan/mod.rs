//! Etherscan API client module
//!
//! Wraps `foundry-block-explorers` with additional functionality:
//! - Signature cache for function selectors and event topics
//! - 4byte.directory integration for signature lookups
//! - Token metadata via eth_call

pub mod cache;
mod client;
pub mod support;

pub use cache::{
    CacheData, CacheEntry, CacheStats, SignatureCache, TokenCacheEntry, TokenMetadataCache,
};
pub use client::Client;
pub use support::{ensure_etherscan_supported, etherscan_sunset, EtherscanSunset};
