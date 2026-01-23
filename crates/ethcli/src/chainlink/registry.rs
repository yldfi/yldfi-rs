//! Chainlink Feed Registry queries
//!
//! The Feed Registry allows querying prices by token address
//! instead of needing to know specific oracle addresses.
//! Only available on Ethereum mainnet.

use super::constants::FEED_REGISTRY;
use super::types::{ChainlinkError, PriceData};
use alloy::eips::BlockId;
use alloy::primitives::{Address, Uint};
use alloy::providers::Provider;
use alloy::sol;

// Define the Feed Registry interface
sol! {
    #[sol(rpc)]
    interface IFeedRegistry {
        /// Get the latest price data for a base/quote pair
        function latestRoundData(address base, address quote)
            external
            view
            returns (
                uint80 roundId,
                int256 answer,
                uint256 startedAt,
                uint256 updatedAt,
                uint80 answeredInRound
            );

        /// Get price data for a specific round
        function getRoundData(address base, address quote, uint80 _roundId)
            external
            view
            returns (
                uint80 roundId,
                int256 answer,
                uint256 startedAt,
                uint256 updatedAt,
                uint80 answeredInRound
            );

        /// Get the number of decimals for this pair
        function decimals(address base, address quote) external view returns (uint8);

        /// Get the aggregator address for a pair (returns address(0) if not found)
        function getFeed(address base, address quote) external view returns (address);

        /// Get the latest round ID for a pair
        function latestRound(address base, address quote) external view returns (uint80);

        /// Get the description of a pair
        function description(address base, address quote) external view returns (string);
    }
}

/// Chainlink Feed Registry client
pub struct FeedRegistry<P: Provider + Clone> {
    provider: P,
}

impl<P: Provider + Clone> FeedRegistry<P> {
    /// Create a new Feed Registry client
    pub fn new(provider: P) -> Self {
        Self { provider }
    }

    /// Check if a feed exists for a base/quote pair
    pub async fn has_feed(&self, base: Address, quote: Address) -> Result<bool, ChainlinkError> {
        let contract = IFeedRegistry::new(FEED_REGISTRY, &self.provider);

        match contract.getFeed(base, quote).call().await {
            Ok(result) => Ok(result != Address::ZERO),
            Err(e) => {
                // Some implementations revert instead of returning address(0)
                let msg = e.to_string();
                if msg.contains("Feed not found") || msg.contains("revert") {
                    Ok(false)
                } else {
                    Err(ChainlinkError::from(e))
                }
            }
        }
    }

    /// Get the aggregator address for a pair
    pub async fn get_feed(&self, base: Address, quote: Address) -> Result<Address, ChainlinkError> {
        let contract = IFeedRegistry::new(FEED_REGISTRY, &self.provider);

        let result = contract.getFeed(base, quote).call().await?;

        if result == Address::ZERO {
            return Err(ChainlinkError::NoFeed);
        }

        Ok(result)
    }

    /// Get the latest price for a base/quote pair
    pub async fn latest_price(
        &self,
        base: Address,
        quote: Address,
    ) -> Result<PriceData, ChainlinkError> {
        let contract = IFeedRegistry::new(FEED_REGISTRY, &self.provider);

        // First check if feed exists and get its address
        let feed_address = match contract.getFeed(base, quote).call().await {
            Ok(result) if result != Address::ZERO => result,
            Ok(_) => return Err(ChainlinkError::NoFeed),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("Feed not found") {
                    return Err(ChainlinkError::NoFeed);
                }
                return Err(ChainlinkError::from(e));
            }
        };

        // Get decimals
        let decimals = contract.decimals(base, quote).call().await?;

        // Get latest round data
        let round_data = contract.latestRoundData(base, quote).call().await?;

        PriceData::from_round_data(
            round_data.answer,
            decimals,
            round_data.roundId,
            round_data.updatedAt,
            round_data.answeredInRound,
            Some(feed_address),
        )
    }

    /// Get price at a specific block (requires archive node)
    ///
    /// Important: This resolves the feed address at the target block,
    /// ensuring historical accuracy even if the feed was upgraded.
    pub async fn price_at_block(
        &self,
        base: Address,
        quote: Address,
        block: BlockId,
    ) -> Result<PriceData, ChainlinkError> {
        let contract = IFeedRegistry::new(FEED_REGISTRY, &self.provider);

        // Resolve feed address at the target block
        let feed_result = contract.getFeed(base, quote).block(block).call().await;

        let feed_address = match feed_result {
            Ok(result) if result != Address::ZERO => result,
            Ok(_) => return Err(ChainlinkError::NoFeed),
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("missing trie node") || msg.contains("state not available") {
                    return Err(ChainlinkError::ArchiveNodeRequired);
                }
                if msg.contains("Feed not found") {
                    return Err(ChainlinkError::NoFeed);
                }
                return Err(ChainlinkError::from(e));
            }
        };

        // Get decimals at that block
        let decimals = contract.decimals(base, quote).block(block).call().await?;

        // Get round data at that block
        let round_data = contract
            .latestRoundData(base, quote)
            .block(block)
            .call()
            .await?;

        PriceData::from_round_data(
            round_data.answer,
            decimals,
            round_data.roundId,
            round_data.updatedAt,
            round_data.answeredInRound,
            Some(feed_address),
        )
    }

    /// Get price at a specific round
    pub async fn price_at_round(
        &self,
        base: Address,
        quote: Address,
        round_id: u64,
    ) -> Result<PriceData, ChainlinkError> {
        let contract = IFeedRegistry::new(FEED_REGISTRY, &self.provider);

        // Get feed address
        let feed_address = match contract.getFeed(base, quote).call().await {
            Ok(result) if result != Address::ZERO => result,
            Ok(_) => return Err(ChainlinkError::NoFeed),
            Err(e) => return Err(ChainlinkError::from(e)),
        };

        // Get decimals
        let decimals = contract.decimals(base, quote).call().await?;

        // Get round data - convert to Uint<80, 2> (u80)
        let round_id_uint: Uint<80, 2> = Uint::from(round_id);
        let round_data = contract
            .getRoundData(base, quote, round_id_uint)
            .call()
            .await?;

        PriceData::from_round_data(
            round_data.answer,
            decimals,
            round_data.roundId,
            round_data.updatedAt,
            round_data.answeredInRound,
            Some(feed_address),
        )
    }

    /// Get the description of a pair (e.g., "ETH / USD")
    pub async fn description(
        &self,
        base: Address,
        quote: Address,
    ) -> Result<String, ChainlinkError> {
        let contract = IFeedRegistry::new(FEED_REGISTRY, &self.provider);
        let result = contract.description(base, quote).call().await?;
        Ok(result)
    }
}
