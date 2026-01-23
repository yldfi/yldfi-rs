//! Chainlink price feed queries via RPC
//!
//! Two approaches:
//! 1. Feed Registry - query by token address (Ethereum mainnet only)
//! 2. Direct Aggregator - query specific oracle addresses (all chains)
//!
//! # Example
//!
//! ```ignore
//! use ethcli::chainlink::{FeedRegistry, Aggregator, denominations};
//!
//! // Via Feed Registry (mainnet)
//! let registry = FeedRegistry::new(provider);
//! let price = registry.latest_price(cvx_address, denominations::USD).await?;
//!
//! // Via direct aggregator (any chain)
//! let aggregator = Aggregator::new(oracle_address, provider);
//! let price = aggregator.latest_price().await?;
//!
//! // Historical price at block (requires archive node)
//! let price = registry.price_at_block(token, denominations::USD, block_id).await?;
//! ```

mod aggregator;
pub mod constants;
mod registry;
mod types;

pub use aggregator::Aggregator;
pub use constants::{denominations, oracles, symbol_to_address, tokens, FEED_REGISTRY};
pub use registry::FeedRegistry;
pub use types::{ChainlinkError, PriceData};

/// Fetch Chainlink price via RPC (registry or direct oracle)
///
/// Tries Feed Registry first (mainnet), falls back to known oracles.
pub async fn fetch_price<P: alloy::providers::Provider + Clone>(
    provider: P,
    token: &str,
    chain: &str,
) -> Result<PriceData, ChainlinkError> {
    use alloy::primitives::Address;
    use std::str::FromStr;

    // Resolve token to address
    let base = if token.starts_with("0x") {
        Address::from_str(token).map_err(|_| ChainlinkError::InvalidAddress(token.to_string()))?
    } else {
        constants::symbol_to_address(token)
            .ok_or_else(|| ChainlinkError::UnknownSymbol(token.to_string()))?
    };

    // Try Feed Registry on mainnet
    if chain == "ethereum" || chain == "mainnet" || chain == "eth" {
        let registry = FeedRegistry::new(provider.clone());
        match registry.latest_price(base, denominations::USD).await {
            Ok(price) => return Ok(price),
            Err(ChainlinkError::NoFeed) => {
                // Fall through to try direct oracle
            }
            Err(e) => return Err(e),
        }
    }

    // Try known oracle mapping
    if let Some(oracle) = constants::get_oracle_for_token(token, chain) {
        let aggregator = Aggregator::new(oracle, provider);
        return aggregator.latest_price().await;
    }

    Err(ChainlinkError::NoFeed)
}

/// Fetch historical Chainlink price at a specific block
pub async fn fetch_price_at_block<P: alloy::providers::Provider + Clone>(
    provider: P,
    token: &str,
    chain: &str,
    block: alloy::eips::BlockId,
) -> Result<PriceData, ChainlinkError> {
    use alloy::primitives::Address;
    use std::str::FromStr;

    let base = if token.starts_with("0x") {
        Address::from_str(token).map_err(|_| ChainlinkError::InvalidAddress(token.to_string()))?
    } else {
        constants::symbol_to_address(token)
            .ok_or_else(|| ChainlinkError::UnknownSymbol(token.to_string()))?
    };

    // Feed Registry approach (mainnet) - resolve feed at the same block
    if chain == "ethereum" || chain == "mainnet" || chain == "eth" {
        let registry = FeedRegistry::new(provider.clone());
        match registry
            .price_at_block(base, denominations::USD, block)
            .await
        {
            Ok(price) => return Ok(price),
            Err(ChainlinkError::NoFeed) => {
                // Fall through
            }
            Err(e) => return Err(e),
        }
    }

    // Direct oracle approach
    if let Some(oracle) = constants::get_oracle_for_token(token, chain) {
        let aggregator = Aggregator::new(oracle, provider);
        return aggregator.price_at_block(block).await;
    }

    Err(ChainlinkError::NoFeed)
}
