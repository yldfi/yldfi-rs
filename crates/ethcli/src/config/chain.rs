//! Chain definitions and utilities

use crate::error::ConfigError;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Chain ID type
pub type ChainId = u64;

/// Supported blockchain networks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
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

    /// Get the chain name (delegates to yldfi_common for known custom chains)
    pub fn name(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ethereum",
            Chain::Polygon => "polygon",
            Chain::Arbitrum => "arbitrum",
            Chain::Optimism => "optimism",
            Chain::Base => "base",
            Chain::Bsc => "bsc",
            Chain::Avalanche => "avalanche",
            Chain::Custom(id) => {
                let common = yldfi_common::Chain::from_id(*id);
                if matches!(common, yldfi_common::Chain::Other(_)) {
                    "custom"
                } else {
                    common.name()
                }
            }
        }
    }

    /// Get display name (delegates to yldfi_common for known custom chains)
    pub fn display_name(&self) -> &'static str {
        match self {
            Chain::Ethereum => "Ethereum",
            Chain::Polygon => "Polygon",
            Chain::Arbitrum => "Arbitrum One",
            Chain::Optimism => "Optimism",
            Chain::Base => "Base",
            Chain::Bsc => "BNB Smart Chain",
            Chain::Avalanche => "Avalanche C-Chain",
            Chain::Custom(id) => {
                let common = yldfi_common::Chain::from_id(*id);
                if matches!(common, yldfi_common::Chain::Other(_)) {
                    "Custom Chain"
                } else {
                    common.display_name()
                }
            }
        }
    }

    /// Get native currency symbol (delegates to yldfi_common for known custom chains)
    pub fn native_symbol(&self) -> &'static str {
        match self {
            Chain::Ethereum => "ETH",
            Chain::Polygon => "MATIC",
            Chain::Arbitrum => "ETH",
            Chain::Optimism => "ETH",
            Chain::Base => "ETH",
            Chain::Bsc => "BNB",
            Chain::Avalanche => "AVAX",
            Chain::Custom(id) => {
                let common = yldfi_common::Chain::from_id(*id);
                if matches!(common, yldfi_common::Chain::Other(_)) {
                    "???"
                } else {
                    common.native_currency()
                }
            }
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
            Chain::Custom(id) => match yldfi_common::Chain::from_id(*id) {
                yldfi_common::Chain::Gnosis => Some("https://gnosisscan.io"),
                yldfi_common::Chain::Fantom => Some("https://ftmscan.com"),
                yldfi_common::Chain::Linea => Some("https://lineascan.build"),
                yldfi_common::Chain::ZkSync => Some("https://explorer.zksync.io"),
                yldfi_common::Chain::Scroll => Some("https://scrollscan.com"),
                yldfi_common::Chain::Blast => Some("https://blastscan.io"),
                yldfi_common::Chain::Mantle => Some("https://mantlescan.xyz"),
                yldfi_common::Chain::Moonbeam => Some("https://moonscan.io"),
                yldfi_common::Chain::Celo => Some("https://celoscan.io"),
                yldfi_common::Chain::PolygonZkEvm => Some("https://zkevm.polygonscan.com"),
                yldfi_common::Chain::Aurora => Some("https://explorer.aurora.dev"),
                yldfi_common::Chain::Mode => Some("https://explorer.mode.network"),
                _ => None,
            },
        }
    }

    /// Get average block time in seconds (approximate)
    /// Used for converting relative time to block numbers
    pub fn avg_block_time_secs(&self) -> f64 {
        match self {
            Chain::Ethereum => 12.0,
            Chain::Polygon => 2.0,
            Chain::Arbitrum => 0.25,
            Chain::Optimism => 2.0,
            Chain::Base => 2.0,
            Chain::Bsc => 3.0,
            Chain::Avalanche => 2.0,
            Chain::Custom(id) => match yldfi_common::Chain::from_id(*id) {
                yldfi_common::Chain::Gnosis => 5.0,
                yldfi_common::Chain::Fantom => 1.0,
                yldfi_common::Chain::Linea => 2.0,
                yldfi_common::Chain::ZkSync => 1.0,
                yldfi_common::Chain::Scroll => 3.0,
                yldfi_common::Chain::Blast => 2.0,
                yldfi_common::Chain::Mantle => 2.0,
                yldfi_common::Chain::Moonbeam => 12.0,
                yldfi_common::Chain::Moonriver => 12.0,
                yldfi_common::Chain::Celo => 5.0,
                yldfi_common::Chain::Mode => 2.0,
                yldfi_common::Chain::Fraxtal => 2.0,
                yldfi_common::Chain::Klaytn => 1.0,
                yldfi_common::Chain::Aurora => 1.0,
                yldfi_common::Chain::PolygonZkEvm => 2.0,
                _ => 12.0,
            },
        }
    }

    /// Calculate approximate blocks for a given duration in seconds
    ///
    /// Returns 0 for non-positive durations. This handles negative inputs
    /// safely without panicking.
    pub fn blocks_for_duration(&self, duration_secs: f64) -> u64 {
        // Guard against non-positive durations (including NaN)
        if duration_secs <= 0.0 || !duration_secs.is_finite() {
            return 0;
        }

        let block_time = self.avg_block_time_secs();
        // Guard against zero/invalid block time (shouldn't happen with current impl)
        debug_assert!(block_time > 0.0, "avg_block_time_secs must be positive");
        if block_time <= 0.0 {
            return 0;
        }

        (duration_secs / block_time).ceil() as u64
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
    ///
    /// Supports all chains known to yldfi_common (44+ chains including gnosis,
    /// fantom, linea, zksync, scroll, blast, mantle, etc.) plus any numeric
    /// chain ID.
    pub fn from_str_or_id(s: &str) -> Result<Self, ConfigError> {
        // Try parsing as chain ID first
        if let Ok(id) = s.parse::<ChainId>() {
            return Ok(Self::from_chain_id(id));
        }

        // Try parsing as name (primary variants first)
        match s.to_lowercase().as_str() {
            "ethereum" | "eth" | "mainnet" => Ok(Chain::Ethereum),
            "polygon" | "matic" => Ok(Chain::Polygon),
            "arbitrum" | "arb" | "arbitrum-one" => Ok(Chain::Arbitrum),
            "optimism" | "op" => Ok(Chain::Optimism),
            "base" => Ok(Chain::Base),
            "bsc" | "bnb" | "binance" => Ok(Chain::Bsc),
            "avalanche" | "avax" => Ok(Chain::Avalanche),
            _ => {
                // Fallback: delegate to yldfi_common for 44+ known chains
                // (gnosis, fantom, linea, zksync, scroll, blast, mantle, etc.)
                if let Some(common_chain) = yldfi_common::Chain::from_name(s) {
                    Ok(Chain::Custom(common_chain.id()))
                } else {
                    Err(ConfigError::InvalidChain(s.to_string()))
                }
            }
        }
    }

    /// Parse from string, defaulting to Ethereum if parsing fails
    pub fn from_str_or_default(s: &str) -> Self {
        Self::from_str_or_id(s).unwrap_or_default()
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

impl From<yldfi_common::Chain> for Chain {
    fn from(chain: yldfi_common::Chain) -> Self {
        match chain {
            yldfi_common::Chain::Ethereum => Self::Ethereum,
            yldfi_common::Chain::Polygon => Self::Polygon,
            yldfi_common::Chain::Arbitrum => Self::Arbitrum,
            yldfi_common::Chain::Optimism => Self::Optimism,
            yldfi_common::Chain::Base => Self::Base,
            yldfi_common::Chain::Bsc => Self::Bsc,
            yldfi_common::Chain::Avalanche => Self::Avalanche,
            yldfi_common::Chain::Other(id) => Self::Custom(id),
            // All other chains become Custom with their chain ID
            other => Self::Custom(other.id()),
        }
    }
}

impl From<Chain> for yldfi_common::Chain {
    fn from(chain: Chain) -> Self {
        match chain {
            Chain::Ethereum => Self::Ethereum,
            Chain::Polygon => Self::Polygon,
            Chain::Arbitrum => Self::Arbitrum,
            Chain::Optimism => Self::Optimism,
            Chain::Base => Self::Base,
            Chain::Bsc => Self::Bsc,
            Chain::Avalanche => Self::Avalanche,
            Chain::Custom(id) => Self::from_id(id),
        }
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

    #[test]
    fn test_from_common_chain() {
        assert_eq!(Chain::from(yldfi_common::Chain::Ethereum), Chain::Ethereum);
        assert_eq!(Chain::from(yldfi_common::Chain::Polygon), Chain::Polygon);
        assert_eq!(Chain::from(yldfi_common::Chain::Arbitrum), Chain::Arbitrum);
        assert_eq!(Chain::from(yldfi_common::Chain::Base), Chain::Base);
        // Chains not in ethcli become Custom with their chain ID
        assert_eq!(Chain::from(yldfi_common::Chain::Fantom), Chain::Custom(250));
        assert_eq!(
            Chain::from(yldfi_common::Chain::Other(99999)),
            Chain::Custom(99999)
        );
    }

    #[test]
    fn test_to_common_chain() {
        assert_eq!(
            yldfi_common::Chain::from(Chain::Ethereum),
            yldfi_common::Chain::Ethereum
        );
        assert_eq!(
            yldfi_common::Chain::from(Chain::Polygon),
            yldfi_common::Chain::Polygon
        );
        assert_eq!(
            yldfi_common::Chain::from(Chain::Custom(250)),
            yldfi_common::Chain::Fantom
        );
        assert_eq!(
            yldfi_common::Chain::from(Chain::Custom(99999)),
            yldfi_common::Chain::Other(99999)
        );
    }

    #[test]
    fn test_from_str_extended_chains() {
        // Gnosis
        let chain = "gnosis".parse::<Chain>().unwrap();
        assert_eq!(chain, Chain::Custom(100));
        assert_eq!(chain.name(), "gnosis");
        assert_eq!(chain.display_name(), "Gnosis");
        assert_eq!(chain.native_symbol(), "xDAI");

        // Also works with alias
        assert_eq!("xdai".parse::<Chain>().unwrap(), Chain::Custom(100));

        // Fantom
        let chain = "fantom".parse::<Chain>().unwrap();
        assert_eq!(chain, Chain::Custom(250));
        assert_eq!(chain.name(), "fantom");
        assert_eq!(chain.display_name(), "Fantom");
        assert_eq!(chain.native_symbol(), "FTM");
        assert_eq!("ftm".parse::<Chain>().unwrap(), Chain::Custom(250));

        // Linea
        let chain = "linea".parse::<Chain>().unwrap();
        assert_eq!(chain, Chain::Custom(59144));
        assert_eq!(chain.name(), "linea");

        // zkSync
        let chain = "zksync".parse::<Chain>().unwrap();
        assert_eq!(chain, Chain::Custom(324));
        assert_eq!(chain.name(), "zksync");

        // Scroll
        let chain = "scroll".parse::<Chain>().unwrap();
        assert_eq!(chain, Chain::Custom(534352));

        // Blast
        let chain = "blast".parse::<Chain>().unwrap();
        assert_eq!(chain, Chain::Custom(81457));

        // Mantle
        let chain = "mantle".parse::<Chain>().unwrap();
        assert_eq!(chain, Chain::Custom(5000));
    }

    #[test]
    fn test_custom_unknown_still_works() {
        let chain = Chain::Custom(99999);
        assert_eq!(chain.name(), "custom");
        assert_eq!(chain.display_name(), "Custom Chain");
        assert_eq!(chain.native_symbol(), "???");
        assert_eq!(chain.explorer_url(), None);
    }

    #[test]
    fn test_custom_known_explorer() {
        assert_eq!(
            Chain::Custom(100).explorer_url(),
            Some("https://gnosisscan.io")
        );
        assert_eq!(
            Chain::Custom(250).explorer_url(),
            Some("https://ftmscan.com")
        );
        assert_eq!(
            Chain::Custom(324).explorer_url(),
            Some("https://explorer.zksync.io")
        );
    }

    #[test]
    fn test_invalid_chain_name() {
        assert!("totally-fake-chain".parse::<Chain>().is_err());
    }
}
