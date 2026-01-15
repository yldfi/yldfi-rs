//! Chain definitions and utilities

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Chain ID type
pub type ChainId = u64;

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Chain {
    #[default]
    Ethereum,
    Polygon,
    Arbitrum,
    Optimism,
    Base,
    Bsc,
    Avalanche,
    /// Custom chain with ID
    #[serde(untagged)]
    Custom(ChainId),
}

impl Chain {
    /// Get the chain ID
    pub fn chain_id(&self) -> ChainId {
        match self {
            Chain::Ethereum => 1,
            Chain::Polygon => 137,
            Chain::Arbitrum => 42161,
            Chain::Optimism => 10,
            Chain::Base => 8453,
            Chain::Bsc => 56,
            Chain::Avalanche => 43114,
            Chain::Custom(id) => *id,
        }
    }

    /// Get the chain name
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Polygon => "polygon",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
            Chain::Bsc => "bsc",
            Chain::Avalanche => "avalanche",
            Chain::Custom(_) => "custom",
        }
    }

    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Chain::Ethereum => "Ethereum",
            Chain::Polygon => "Polygon",
            Chain::Arbitrum => "Arbitrum One",
            Chain::Optimism => "Optimism",
            Chain::Base => "Base",
            Chain::Bsc => "BNB Smart Chain",
            Chain::Avalanche => "Avalanche C-Chain",
            Chain::Custom(_) => "Custom Chain",
        }
    }

    /// Get native currency symbol
    pub fn native_symbol(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ETH",
            Chain::Polygon => "MATIC",
            Chain::Arbitrum => "ETH",
            Chain::Optimism => "ETH",
            Chain::Base => "ETH",
            Chain::Bsc => "BNB",
            Chain::Avalanche => "AVAX",
            Chain::Custom(_) => "???",
        }
    }

    /// Get block explorer URL
    pub fn explorer_url(&self) -> Option<&'static str> {
        match self {
            Chain::Ethereum => Some("https://etherscan.io"),
            Chain::Polygon => Some("https://polygonscan.com"),
            Chain::Arbitrum => Some("https://arbiscan.io"),
            Chain::Optimism => Some("https://optimistic.etherscan.io"),
            Chain::Base => Some("https://basescan.org"),
            Chain::Bsc => Some("https://bscscan.com"),
            Chain::Avalanche => Some("https://snowtrace.io"),
            Chain::Custom(_) => None,
        }
    }

    /// Get average block time in seconds (approximate)
    /// Used for converting relative time to block numbers
    pub fn avg_block_time_secs(&self) -> f64 {
        match self {
            Chain::Ethereum => 12.0,  // ~12 seconds post-merge
            Chain::Polygon => 2.0,    // ~2 seconds
            Chain::Arbitrum => 0.25,  // ~250ms (L2)
            Chain::Optimism => 2.0,   // ~2 seconds (L2)
            Chain::Base => 2.0,       // ~2 seconds (L2, OP stack)
            Chain::Bsc => 3.0,        // ~3 seconds
            Chain::Avalanche => 2.0,  // ~2 seconds
            Chain::Custom(_) => 12.0, // Default to Ethereum-like
        }
    }

    /// Calculate approximate blocks for a given duration in seconds
    pub fn blocks_for_duration(&self, duration_secs: f64) -> u64 {
        (duration_secs / self.avg_block_time_secs()).ceil() as u64
    }

    /// Create from chain ID
    pub fn from_chain_id(id: ChainId) -> Self {
        match id {
            1 => Chain::Ethereum,
            137 => Chain::Polygon,
            42161 => Chain::Arbitrum,
            10 => Chain::Optimism,
            8453 => Chain::Base,
            56 => Chain::Bsc,
            43114 => Chain::Avalanche,
            _ => Chain::Custom(id),
        }
    }

    /// Parse from string (name or chain ID)
    pub fn from_str_or_id(s: &str) -> Result<Self, ConfigError> {
        // Try parsing as chain ID first
        if let Ok(id) = s.parse::<ChainId>() {
            return Ok(Self::from_chain_id(id));
        }

        // Try parsing as name
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Ok(Chain::Ethereum),
            "polygon" | "matic" => Ok(Chain::Polygon),
            "arbitrum" | "arb" | "arbitrum-one" => Ok(Chain::Arbitrum),
            "optimism" | "op" => Ok(Chain::Optimism),
            "base" => Ok(Chain::Base),
            "bsc" | "bnb" | "binance" => Ok(Chain::Bsc),
            "avalanche" | "avax" => Ok(Chain::Avalanche),
            _ => Err(ConfigError::InvalidChain(s.to_string())),
        }
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl std::str::FromStr for Chain {
    type Err = ConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_or_id(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id() {
        assert_eq!(Chain::Ethereum.chain_id(), 1);
        assert_eq!(Chain::Polygon.chain_id(), 137);
        assert_eq!(Chain::Custom(12345).chain_id(), 12345);
    }

    #[test]
    fn test_from_chain_id() {
        assert_eq!(Chain::from_chain_id(1), Chain::Ethereum);
        assert_eq!(Chain::from_chain_id(137), Chain::Polygon);
        assert_eq!(Chain::from_chain_id(99999), Chain::Custom(99999));
    }

    #[test]
    fn test_from_str() {
        assert_eq!("ethereum".parse::<Chain>().unwrap(), Chain::Ethereum);
        assert_eq!("1".parse::<Chain>().unwrap(), Chain::Ethereum);
        assert_eq!("polygon".parse::<Chain>().unwrap(), Chain::Polygon);
        assert_eq!("137".parse::<Chain>().unwrap(), Chain::Polygon);
    }
}
