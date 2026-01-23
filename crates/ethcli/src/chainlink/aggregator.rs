//! Direct Chainlink Aggregator queries
//!
//! For querying specific oracle addresses directly.
//! Works on any chain where you know the oracle address.

use super::types::{ChainlinkError, PriceData};
use alloy::eips::BlockId;
use alloy::primitives::{Address, Uint, I256};
use alloy::providers::Provider;
use alloy::sol;

// Define the AggregatorV3 interface
sol! {
    #[sol(rpc)]
    interface IAggregatorV3 {
        /// Get the latest price data
        function latestRoundData()
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
        function getRoundData(uint80 _roundId)
            external
            view
            returns (
                uint80 roundId,
                int256 answer,
                uint256 startedAt,
                uint256 updatedAt,
                uint80 answeredInRound
            );

        /// Get the number of decimals
        function decimals() external view returns (uint8);

        /// Get the description (e.g., "ETH / USD")
        function description() external view returns (string);

        /// Get the version of the aggregator
        function version() external view returns (uint256);

        /// Get the latest answer (simpler, but less info)
        function latestAnswer() external view returns (int256);

        /// Get the latest round ID
        function latestRound() external view returns (uint256);

        /// Get the timestamp of a round
        function getTimestamp(uint256 roundId) external view returns (uint256);

        /// Get the answer for a round (simpler, but less info)
        function getAnswer(uint256 roundId) external view returns (int256);
    }
}

/// Chainlink Aggregator client for direct oracle queries
pub struct Aggregator<P: Provider + Clone> {
    address: Address,
    provider: P,
}

impl<P: Provider + Clone> Aggregator<P> {
    /// Create a new Aggregator client
    pub fn new(address: Address, provider: P) -> Self {
        Self { address, provider }
    }

    /// Get the oracle address
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get the latest price
    pub async fn latest_price(&self) -> Result<PriceData, ChainlinkError> {
        let contract = IAggregatorV3::new(self.address, &self.provider);

        // Get decimals and round data
        let decimals = contract.decimals().call().await?;
        let round_data = contract.latestRoundData().call().await?;

        PriceData::from_round_data(
            round_data.answer,
            decimals,
            round_data.roundId,
            round_data.updatedAt,
            round_data.answeredInRound,
            Some(self.address),
        )
    }

    /// Get price at a specific block (requires archive node)
    pub async fn price_at_block(&self, block: BlockId) -> Result<PriceData, ChainlinkError> {
        let contract = IAggregatorV3::new(self.address, &self.provider);

        // Get decimals at that block
        let decimals_result = contract.decimals().block(block).call().await;

        let decimals = match decimals_result {
            Ok(result) => result,
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("missing trie node") || msg.contains("state not available") {
                    return Err(ChainlinkError::ArchiveNodeRequired);
                }
                return Err(ChainlinkError::from(e));
            }
        };

        // Get round data at that block
        let round_data = contract.latestRoundData().block(block).call().await?;

        PriceData::from_round_data(
            round_data.answer,
            decimals,
            round_data.roundId,
            round_data.updatedAt,
            round_data.answeredInRound,
            Some(self.address),
        )
    }

    /// Get price at a specific round
    pub async fn price_at_round(&self, round_id: u64) -> Result<PriceData, ChainlinkError> {
        let contract = IAggregatorV3::new(self.address, &self.provider);

        // Get decimals
        let decimals = contract.decimals().call().await?;

        // Get round data - convert to Uint<80, 2> (u80)
        let round_id_uint: Uint<80, 2> = Uint::from(round_id);
        let round_data = contract.getRoundData(round_id_uint).call().await?;

        PriceData::from_round_data(
            round_data.answer,
            decimals,
            round_data.roundId,
            round_data.updatedAt,
            round_data.answeredInRound,
            Some(self.address),
        )
    }

    /// Get the description (e.g., "ETH / USD")
    pub async fn description(&self) -> Result<String, ChainlinkError> {
        let contract = IAggregatorV3::new(self.address, &self.provider);
        let result = contract.description().call().await?;
        Ok(result)
    }

    /// Get the version of the aggregator
    pub async fn version(&self) -> Result<u64, ChainlinkError> {
        let contract = IAggregatorV3::new(self.address, &self.provider);
        let result = contract.version().call().await?;
        Ok(result.try_into().unwrap_or(0))
    }

    /// Get just the latest answer (simpler than latestRoundData)
    pub async fn latest_answer(&self) -> Result<I256, ChainlinkError> {
        let contract = IAggregatorV3::new(self.address, &self.provider);
        let result = contract.latestAnswer().call().await?;
        Ok(result)
    }

    /// Get just the decimals
    pub async fn decimals(&self) -> Result<u8, ChainlinkError> {
        let contract = IAggregatorV3::new(self.address, &self.provider);
        let result = contract.decimals().call().await?;
        Ok(result)
    }
}

// Integration tests are in tests/chainlink_live.rs (require RPC connection)
