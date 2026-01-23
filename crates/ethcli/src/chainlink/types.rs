//! Chainlink types and error handling

use alloy::primitives::{I256, U256};
use serde::Serialize;
use thiserror::Error;

/// Chainlink price data from an aggregator
#[derive(Debug, Clone, Serialize)]
pub struct PriceData {
    /// Raw answer from the oracle (can be negative for some feeds)
    #[serde(serialize_with = "serialize_i256")]
    pub answer: I256,
    /// Decimals for this feed (call decimals() on aggregator)
    pub decimals: u8,
    /// Round ID
    pub round_id: u64,
    /// Timestamp when the answer was computed
    pub updated_at: u64,
    /// Round ID in which the answer was computed
    pub answered_in_round: u64,
    /// Oracle/feed address that provided this price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_address: Option<alloy::primitives::Address>,
}

fn serialize_i256<S>(val: &I256, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&val.to_string())
}

impl PriceData {
    /// Create PriceData from Chainlink round data response
    ///
    /// Handles the u80 -> u64 conversion for round IDs and timestamps.
    /// Note: roundId uses a composite format (phaseId << 64 | aggregatorRoundId),
    /// so values above u64::MAX are technically possible but extremely rare.
    ///
    /// # Errors
    /// Returns `ChainlinkError::ContractError` if any field fails u64 conversion.
    pub fn from_round_data(
        answer: I256,
        decimals: u8,
        round_id: impl TryInto<u64>,
        updated_at: impl TryInto<u64>,
        answered_in_round: impl TryInto<u64>,
        feed_address: Option<alloy::primitives::Address>,
    ) -> Result<Self, ChainlinkError> {
        let round_id = round_id
            .try_into()
            .map_err(|_| ChainlinkError::ContractError("roundId overflow".to_string()))?;
        let updated_at = updated_at
            .try_into()
            .map_err(|_| ChainlinkError::ContractError("updatedAt overflow".to_string()))?;
        let answered_in_round = answered_in_round
            .try_into()
            .map_err(|_| ChainlinkError::ContractError("answeredInRound overflow".to_string()))?;

        Ok(Self {
            answer,
            decimals,
            round_id,
            updated_at,
            answered_in_round,
            feed_address,
        })
    }

    /// Check if the price data is stale
    ///
    /// Stale data occurs when answeredInRound < roundId
    pub fn is_stale(&self) -> bool {
        self.answered_in_round < self.round_id
    }

    /// Check if the answer is valid (positive and not stale)
    pub fn is_valid(&self) -> bool {
        self.answer > I256::ZERO && !self.is_stale()
    }

    /// Check if the price is fresh (not older than max_age_seconds)
    ///
    /// This provides timestamp-based staleness detection, complementing
    /// the round-based `is_stale()` check. Use both for robust validation.
    pub fn is_fresh(&self, current_timestamp: u64, max_age_seconds: u64) -> bool {
        self.age_seconds(current_timestamp) <= max_age_seconds
    }

    /// Convert to f64 price, normalized by decimals
    ///
    /// Returns None if the price is invalid (negative, zero, or stale).
    ///
    /// # Precision Warning
    /// f64 has ~15-17 significant decimal digits. Chainlink prices with
    /// more than 15 significant digits will lose precision. For exact
    /// calculations, use `to_u256()` with fixed-point math instead.
    pub fn to_f64(&self) -> Option<f64> {
        if !self.is_valid() {
            return None;
        }

        // Convert I256 to f64
        // I256 can be very large, but Chainlink prices are reasonable
        let answer_str = self.answer.to_string();
        let answer_f64: f64 = answer_str.parse().ok()?;

        // Divide by 10^decimals
        Some(answer_f64 / 10f64.powi(self.decimals as i32))
    }

    /// Get the price as a U256 (for on-chain calculations)
    ///
    /// Returns None if negative
    pub fn to_u256(&self) -> Option<U256> {
        if self.answer <= I256::ZERO {
            return None;
        }
        // Safe conversion since we checked it's positive
        // I256 and U256 are both 32 bytes
        Some(U256::from_be_bytes::<32>(
            self.answer.into_raw().to_be_bytes::<32>(),
        ))
    }

    /// Age of the price data in seconds
    pub fn age_seconds(&self, current_timestamp: u64) -> u64 {
        current_timestamp.saturating_sub(self.updated_at)
    }

    /// Check if price is older than the given threshold
    pub fn is_older_than(&self, current_timestamp: u64, max_age_seconds: u64) -> bool {
        self.age_seconds(current_timestamp) > max_age_seconds
    }
}

/// Chainlink-specific errors
#[derive(Debug, Error)]
pub enum ChainlinkError {
    #[error("No Chainlink feed found for this token/pair")]
    NoFeed,

    #[error("No feed found at block {0}")]
    NoFeedAtBlock(u64),

    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    #[error("Unknown token symbol: {0}")]
    UnknownSymbol(String),

    #[error("Stale price data (answeredInRound < roundId)")]
    StalePrice,

    #[error("Invalid price (zero or negative)")]
    InvalidPrice,

    #[error("Archive node required for historical queries")]
    ArchiveNodeRequired,

    #[error("Chain not supported: {0}")]
    UnsupportedChain(String),

    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Contract call failed: {0}")]
    ContractError(String),
}

impl From<alloy::contract::Error> for ChainlinkError {
    fn from(e: alloy::contract::Error) -> Self {
        let msg = e.to_string();
        // Detect archive node errors
        if msg.contains("missing trie node") || msg.contains("state not available") {
            ChainlinkError::ArchiveNodeRequired
        } else {
            ChainlinkError::ContractError(msg)
        }
    }
}

impl From<alloy::transports::TransportError> for ChainlinkError {
    fn from(e: alloy::transports::TransportError) -> Self {
        ChainlinkError::RpcError(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_data_to_f64() {
        // 100000000 with 8 decimals = 1.0
        let price = PriceData {
            answer: I256::try_from(100_000_000i64).unwrap(),
            decimals: 8,
            round_id: 100,
            updated_at: 1000,
            answered_in_round: 100,
            feed_address: None,
        };
        assert_eq!(price.to_f64(), Some(1.0));

        // 250000000000 with 8 decimals = 2500.0
        let price2 = PriceData {
            answer: I256::try_from(250_000_000_000i64).unwrap(),
            decimals: 8,
            round_id: 100,
            updated_at: 1000,
            answered_in_round: 100,
            feed_address: None,
        };
        assert_eq!(price2.to_f64(), Some(2500.0));
    }

    #[test]
    fn test_stale_detection() {
        let stale = PriceData {
            answer: I256::try_from(100_000_000i64).unwrap(),
            decimals: 8,
            round_id: 100,
            updated_at: 1000,
            answered_in_round: 99, // < roundId
            feed_address: None,
        };
        assert!(stale.is_stale());
        assert!(!stale.is_valid());
        assert_eq!(stale.to_f64(), None);
    }

    #[test]
    fn test_negative_price() {
        let negative = PriceData {
            answer: I256::try_from(-100_000_000i64).unwrap(),
            decimals: 8,
            round_id: 100,
            updated_at: 1000,
            answered_in_round: 100,
            feed_address: None,
        };
        assert!(!negative.is_valid());
        assert_eq!(negative.to_f64(), None);
    }

    #[test]
    fn test_from_round_data() {
        let price = PriceData::from_round_data(
            I256::try_from(100_000_000i64).unwrap(),
            8,
            100u64,
            1000u64,
            100u64,
            None,
        )
        .unwrap();
        assert_eq!(price.decimals, 8);
        assert_eq!(price.round_id, 100);
        assert_eq!(price.updated_at, 1000);
        assert_eq!(price.answered_in_round, 100);
        assert!(price.is_valid());
        assert_eq!(price.to_f64(), Some(1.0));
    }

    #[test]
    fn test_is_fresh() {
        let price = PriceData::from_round_data(
            I256::try_from(100_000_000i64).unwrap(),
            8,
            100u64,
            1000u64, // updated_at = 1000
            100u64,
            None,
        )
        .unwrap();

        // Current time = 1500, max age = 600 -> fresh (age = 500)
        assert!(price.is_fresh(1500, 600));

        // Current time = 2000, max age = 600 -> stale (age = 1000)
        assert!(!price.is_fresh(2000, 600));

        // Edge case: exactly at threshold
        assert!(price.is_fresh(1600, 600)); // age = 600 == max_age
        assert!(!price.is_fresh(1601, 600)); // age = 601 > max_age
    }

    #[test]
    fn test_age_seconds() {
        let price = PriceData::from_round_data(
            I256::try_from(100_000_000i64).unwrap(),
            8,
            100u64,
            1000u64,
            100u64,
            None,
        )
        .unwrap();

        assert_eq!(price.age_seconds(1500), 500);
        assert_eq!(price.age_seconds(1000), 0);
        // Saturating sub: if current < updated, return 0
        assert_eq!(price.age_seconds(500), 0);
    }
}
