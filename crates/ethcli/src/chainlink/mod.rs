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
mod streams;
mod types;

pub use aggregator::Aggregator;
pub use constants::{denominations, oracles, symbol_to_address, tokens, FEED_REGISTRY};
pub use registry::FeedRegistry;
pub use streams::{
    DataStreamsClient, DataStreamsError, DataStreamsFeed, DEFAULT_STREAMS_REST_URL,
    DEFAULT_STREAMS_WS_URL,
};
pub use types::{ChainlinkError, PriceData, MAX_PRICE_AGE_SECS};

/// Fetch Chainlink price via RPC (registry or direct oracle)
///
/// Tries Feed Registry first (mainnet), falls back to known oracles.
pub async fn fetch_price<P: alloy::providers::Provider + Clone>(
    provider: P,
    token: &str,
    chain: &str,
    denomination: alloy::primitives::Address,
) -> Result<PriceData, ChainlinkError> {
    let chain = normalize_chain(chain);

    // Feed Registry on mainnet, for any base/quote pair it knows
    if chain == "ethereum" {
        if let Some(base) = registry_base(token)? {
            let registry = FeedRegistry::new(provider.clone());
            match registry.latest_price(base, denomination).await {
                Ok(price) => return Ok(price),
                Err(ChainlinkError::NoFeed) => {}
                Err(e) => return Err(e),
            }
        }
    }

    // Known direct oracles are all X/USD: never use one for another quote
    if let Some(oracle) = usd_oracle(token, &chain, denomination) {
        return Aggregator::new(oracle, provider).latest_price().await;
    }

    no_feed(token, &chain)
}

/// Fetch historical Chainlink price at a specific block
pub async fn fetch_price_at_block<P: alloy::providers::Provider + Clone>(
    provider: P,
    token: &str,
    chain: &str,
    block: alloy::eips::BlockId,
    denomination: alloy::primitives::Address,
) -> Result<PriceData, ChainlinkError> {
    let chain = normalize_chain(chain);

    // Feed Registry approach (mainnet) - resolve feed at the same block
    if chain == "ethereum" {
        if let Some(base) = registry_base(token)? {
            let registry = FeedRegistry::new(provider.clone());
            match registry.price_at_block(base, denomination, block).await {
                Ok(price) => return Ok(price),
                Err(ChainlinkError::NoFeed) => {}
                Err(e) => return Err(e),
            }
        }
    }

    if let Some(oracle) = usd_oracle(token, &chain, denomination) {
        return Aggregator::new(oracle, provider)
            .price_at_block(block)
            .await;
    }

    no_feed(token, &chain)
}

/// Canonical chain name ("1", "eth", "mainnet" -> "ethereum"; "arb" -> "arbitrum")
pub fn normalize_chain(chain: &str) -> String {
    use std::str::FromStr;
    match crate::config::Chain::from_str(chain.trim()) {
        Ok(c) if c.name() != "custom" => c.name().to_string(),
        _ => chain.trim().to_lowercase(),
    }
}

/// Base asset address for the Feed Registry: the token address, or a known
/// symbol's address. `None` for symbols the registry doesn't know (L2-only
/// tokens such as ARB or GMX), which are left to the direct oracles.
fn registry_base(token: &str) -> Result<Option<alloy::primitives::Address>, ChainlinkError> {
    if token.starts_with("0x") {
        return token
            .parse()
            .map(Some)
            .map_err(|_| ChainlinkError::InvalidAddress(token.to_string()));
    }
    Ok(constants::symbol_to_address(token))
}

/// The known X/USD oracle for `token` on `chain`, only when USD is requested
fn usd_oracle(
    token: &str,
    chain: &str,
    denomination: alloy::primitives::Address,
) -> Option<alloy::primitives::Address> {
    if denomination != constants::denominations::USD {
        return None;
    }
    constants::get_oracle_for_token(token, chain)
}

/// NoFeed, or UnknownSymbol for a symbol nothing on this chain recognises
fn no_feed<T>(token: &str, chain: &str) -> Result<T, ChainlinkError> {
    let known_symbol = token.starts_with("0x")
        || constants::symbol_to_address(token).is_some()
        || constants::get_oracle_for_token(token, chain).is_some();
    if known_symbol {
        Err(ChainlinkError::NoFeed)
    } else {
        Err(ChainlinkError::UnknownSymbol(token.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_chain_maps_aliases() {
        assert_eq!(normalize_chain("1"), "ethereum");
        assert_eq!(normalize_chain("Mainnet"), "ethereum");
        assert_eq!(normalize_chain("arb"), "arbitrum");
        assert_eq!(normalize_chain("42161"), "arbitrum");
    }

    #[test]
    fn direct_oracles_serve_usd_only() {
        let usd = constants::denominations::USD;
        assert!(usd_oracle("ETH", "arbitrum", usd).is_some());
        assert!(usd_oracle("ETH", "arbitrum", constants::denominations::BTC).is_none());
        assert!(usd_oracle("ETH", "arbitrum", constants::denominations::ETH).is_none());
    }

    #[test]
    fn l2_only_symbols_are_not_unknown() {
        // ARB has no mainnet registry address but an Arbitrum oracle
        assert!(registry_base("ARB").unwrap().is_none());
        assert!(matches!(
            no_feed::<()>("ARB", "arbitrum"),
            Err(ChainlinkError::NoFeed)
        ));
        assert!(matches!(
            no_feed::<()>("NOTATOKEN", "arbitrum"),
            Err(ChainlinkError::UnknownSymbol(_))
        ));
    }
}
