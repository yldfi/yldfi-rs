//! Chain ID and name mappings for EVM-compatible networks

use std::fmt;

/// Common EVM chain identifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Chain {
    /// Ethereum Mainnet (1)
    Ethereum,
    /// Goerli Testnet (5) - deprecated
    Goerli,
    /// Sepolia Testnet (11155111)
    Sepolia,
    /// Holesky Testnet (17000)
    Holesky,
    /// Optimism (10)
    Optimism,
    /// Optimism Sepolia (11155420)
    OptimismSepolia,
    /// BNB Smart Chain (56)
    Bsc,
    /// BNB Smart Chain Testnet (97)
    BscTestnet,
    /// Gnosis/xDai (100)
    Gnosis,
    /// Polygon/Matic (137)
    Polygon,
    /// Polygon Mumbai (80001) - deprecated
    Mumbai,
    /// Polygon Amoy (80002)
    Amoy,
    /// Fantom Opera (250)
    Fantom,
    /// Fantom Testnet (4002)
    FantomTestnet,
    /// Moonbeam (1284)
    Moonbeam,
    /// Moonriver (1285)
    Moonriver,
    /// Arbitrum One (42161)
    Arbitrum,
    /// Arbitrum Nova (42170)
    ArbitrumNova,
    /// Arbitrum Sepolia (421614)
    ArbitrumSepolia,
    /// Avalanche C-Chain (43114)
    Avalanche,
    /// Avalanche Fuji (43113)
    AvalancheFuji,
    /// Celo (42220)
    Celo,
    /// Base (8453)
    Base,
    /// Base Sepolia (84532)
    BaseSepolia,
    /// Linea (59144)
    Linea,
    /// Linea Testnet (59140)
    LineaTestnet,
    /// zkSync Era (324)
    ZkSync,
    /// zkSync Sepolia (300)
    ZkSyncSepolia,
    /// Scroll (534352)
    Scroll,
    /// Scroll Sepolia (534351)
    ScrollSepolia,
    /// Blast (81457)
    Blast,
    /// Blast Sepolia (168587773)
    BlastSepolia,
    /// Mantle (5000)
    Mantle,
    /// Mode (34443)
    Mode,
    /// Fraxtal (252)
    Fraxtal,
    /// Klaytn (8217)
    Klaytn,
    /// Aurora (1313161554)
    Aurora,
    /// Polygon zkEVM (1101)
    PolygonZkEvm,
    /// Unknown chain with custom ID
    Other(u64),
}

impl Chain {
    /// Get the chain ID
    #[must_use]
    pub const fn id(&self) -> u64 {
        match self {
            Self::Ethereum => 1,
            Self::Goerli => 5,
            Self::Sepolia => 11_155_111,
            Self::Holesky => 17000,
            Self::Optimism => 10,
            Self::OptimismSepolia => 11_155_420,
            Self::Bsc => 56,
            Self::BscTestnet => 97,
            Self::Gnosis => 100,
            Self::Polygon => 137,
            Self::Mumbai => 80001,
            Self::Amoy => 80002,
            Self::Fantom => 250,
            Self::FantomTestnet => 4002,
            Self::Moonbeam => 1284,
            Self::Moonriver => 1285,
            Self::Arbitrum => 42161,
            Self::ArbitrumNova => 42170,
            Self::ArbitrumSepolia => 421_614,
            Self::Avalanche => 43114,
            Self::AvalancheFuji => 43113,
            Self::Celo => 42220,
            Self::Base => 8453,
            Self::BaseSepolia => 84532,
            Self::Linea => 59144,
            Self::LineaTestnet => 59140,
            Self::ZkSync => 324,
            Self::ZkSyncSepolia => 300,
            Self::Scroll => 534_352,
            Self::ScrollSepolia => 534_351,
            Self::Blast => 81457,
            Self::BlastSepolia => 168_587_773,
            Self::Mantle => 5000,
            Self::Mode => 34443,
            Self::Fraxtal => 252,
            Self::Klaytn => 8217,
            Self::Aurora => 1_313_161_554,
            Self::PolygonZkEvm => 1101,
            Self::Other(id) => *id,
        }
    }

    /// Get the chain name (lowercase, API-friendly)
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Ethereum => "ethereum",
            Self::Goerli => "goerli",
            Self::Sepolia => "sepolia",
            Self::Holesky => "holesky",
            Self::Optimism => "optimism",
            Self::OptimismSepolia => "optimism-sepolia",
            Self::Bsc => "bsc",
            Self::BscTestnet => "bsc-testnet",
            Self::Gnosis => "gnosis",
            Self::Polygon => "polygon",
            Self::Mumbai => "mumbai",
            Self::Amoy => "amoy",
            Self::Fantom => "fantom",
            Self::FantomTestnet => "fantom-testnet",
            Self::Moonbeam => "moonbeam",
            Self::Moonriver => "moonriver",
            Self::Arbitrum => "arbitrum",
            Self::ArbitrumNova => "arbitrum-nova",
            Self::ArbitrumSepolia => "arbitrum-sepolia",
            Self::Avalanche => "avalanche",
            Self::AvalancheFuji => "avalanche-fuji",
            Self::Celo => "celo",
            Self::Base => "base",
            Self::BaseSepolia => "base-sepolia",
            Self::Linea => "linea",
            Self::LineaTestnet => "linea-testnet",
            Self::ZkSync => "zksync",
            Self::ZkSyncSepolia => "zksync-sepolia",
            Self::Scroll => "scroll",
            Self::ScrollSepolia => "scroll-sepolia",
            Self::Blast => "blast",
            Self::BlastSepolia => "blast-sepolia",
            Self::Mantle => "mantle",
            Self::Mode => "mode",
            Self::Fraxtal => "fraxtal",
            Self::Klaytn => "klaytn",
            Self::Aurora => "aurora",
            Self::PolygonZkEvm => "polygon-zkevm",
            Self::Other(_) => "unknown",
        }
    }

    /// Get the display name (human-readable)
    #[must_use]
    pub const fn display_name(&self) -> &'static str {
        match self {
            Self::Ethereum => "Ethereum",
            Self::Goerli => "Goerli",
            Self::Sepolia => "Sepolia",
            Self::Holesky => "Holesky",
            Self::Optimism => "Optimism",
            Self::OptimismSepolia => "Optimism Sepolia",
            Self::Bsc => "BNB Smart Chain",
            Self::BscTestnet => "BNB Smart Chain Testnet",
            Self::Gnosis => "Gnosis",
            Self::Polygon => "Polygon",
            Self::Mumbai => "Mumbai",
            Self::Amoy => "Amoy",
            Self::Fantom => "Fantom",
            Self::FantomTestnet => "Fantom Testnet",
            Self::Moonbeam => "Moonbeam",
            Self::Moonriver => "Moonriver",
            Self::Arbitrum => "Arbitrum One",
            Self::ArbitrumNova => "Arbitrum Nova",
            Self::ArbitrumSepolia => "Arbitrum Sepolia",
            Self::Avalanche => "Avalanche",
            Self::AvalancheFuji => "Avalanche Fuji",
            Self::Celo => "Celo",
            Self::Base => "Base",
            Self::BaseSepolia => "Base Sepolia",
            Self::Linea => "Linea",
            Self::LineaTestnet => "Linea Testnet",
            Self::ZkSync => "zkSync Era",
            Self::ZkSyncSepolia => "zkSync Sepolia",
            Self::Scroll => "Scroll",
            Self::ScrollSepolia => "Scroll Sepolia",
            Self::Blast => "Blast",
            Self::BlastSepolia => "Blast Sepolia",
            Self::Mantle => "Mantle",
            Self::Mode => "Mode",
            Self::Fraxtal => "Fraxtal",
            Self::Klaytn => "Klaytn",
            Self::Aurora => "Aurora",
            Self::PolygonZkEvm => "Polygon zkEVM",
            Self::Other(_) => "Unknown",
        }
    }

    /// Get the native currency symbol
    #[must_use]
    pub const fn native_currency(&self) -> &'static str {
        match self {
            Self::Ethereum | Self::Goerli | Self::Sepolia | Self::Holesky => "ETH",
            Self::Optimism | Self::OptimismSepolia => "ETH",
            Self::Bsc | Self::BscTestnet => "BNB",
            Self::Gnosis => "xDAI",
            Self::Polygon | Self::Mumbai | Self::Amoy => "MATIC",
            Self::Fantom | Self::FantomTestnet => "FTM",
            Self::Moonbeam => "GLMR",
            Self::Moonriver => "MOVR",
            Self::Arbitrum | Self::ArbitrumNova | Self::ArbitrumSepolia => "ETH",
            Self::Avalanche | Self::AvalancheFuji => "AVAX",
            Self::Celo => "CELO",
            Self::Base | Self::BaseSepolia => "ETH",
            Self::Linea | Self::LineaTestnet => "ETH",
            Self::ZkSync | Self::ZkSyncSepolia => "ETH",
            Self::Scroll | Self::ScrollSepolia => "ETH",
            Self::Blast | Self::BlastSepolia => "ETH",
            Self::Mantle => "MNT",
            Self::Mode => "ETH",
            Self::Fraxtal => "frxETH",
            Self::Klaytn => "KLAY",
            Self::Aurora => "ETH",
            Self::PolygonZkEvm => "ETH",
            Self::Other(_) => "ETH",
        }
    }

    /// Check if this is a testnet
    #[must_use]
    pub const fn is_testnet(&self) -> bool {
        matches!(
            self,
            Self::Goerli
                | Self::Sepolia
                | Self::Holesky
                | Self::OptimismSepolia
                | Self::BscTestnet
                | Self::Mumbai
                | Self::Amoy
                | Self::FantomTestnet
                | Self::ArbitrumSepolia
                | Self::AvalancheFuji
                | Self::BaseSepolia
                | Self::LineaTestnet
                | Self::ZkSyncSepolia
                | Self::ScrollSepolia
                | Self::BlastSepolia
        )
    }

    /// Check if this is a mainnet
    #[must_use]
    pub const fn is_mainnet(&self) -> bool {
        !self.is_testnet()
    }

    /// Get chain from ID
    #[must_use]
    pub const fn from_id(id: u64) -> Self {
        match id {
            1 => Self::Ethereum,
            5 => Self::Goerli,
            11_155_111 => Self::Sepolia,
            17000 => Self::Holesky,
            10 => Self::Optimism,
            11_155_420 => Self::OptimismSepolia,
            56 => Self::Bsc,
            97 => Self::BscTestnet,
            100 => Self::Gnosis,
            137 => Self::Polygon,
            80001 => Self::Mumbai,
            80002 => Self::Amoy,
            250 => Self::Fantom,
            4002 => Self::FantomTestnet,
            1284 => Self::Moonbeam,
            1285 => Self::Moonriver,
            42161 => Self::Arbitrum,
            42170 => Self::ArbitrumNova,
            421_614 => Self::ArbitrumSepolia,
            43114 => Self::Avalanche,
            43113 => Self::AvalancheFuji,
            42220 => Self::Celo,
            8453 => Self::Base,
            84532 => Self::BaseSepolia,
            59144 => Self::Linea,
            59140 => Self::LineaTestnet,
            324 => Self::ZkSync,
            300 => Self::ZkSyncSepolia,
            534_352 => Self::Scroll,
            534_351 => Self::ScrollSepolia,
            81457 => Self::Blast,
            168_587_773 => Self::BlastSepolia,
            5000 => Self::Mantle,
            34443 => Self::Mode,
            252 => Self::Fraxtal,
            8217 => Self::Klaytn,
            1_313_161_554 => Self::Aurora,
            1101 => Self::PolygonZkEvm,
            _ => Self::Other(id),
        }
    }

    /// Get chain from name (case-insensitive)
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        let name_lower = name.to_lowercase();
        match name_lower.as_str() {
            "ethereum" | "eth" | "mainnet" => Some(Self::Ethereum),
            "goerli" => Some(Self::Goerli),
            "sepolia" => Some(Self::Sepolia),
            "holesky" => Some(Self::Holesky),
            "optimism" | "op" => Some(Self::Optimism),
            "optimism-sepolia" | "op-sepolia" => Some(Self::OptimismSepolia),
            "bsc" | "bnb" | "binance" => Some(Self::Bsc),
            "bsc-testnet" | "bnb-testnet" => Some(Self::BscTestnet),
            "gnosis" | "xdai" => Some(Self::Gnosis),
            "polygon" | "matic" => Some(Self::Polygon),
            "mumbai" => Some(Self::Mumbai),
            "amoy" => Some(Self::Amoy),
            "fantom" | "ftm" => Some(Self::Fantom),
            "fantom-testnet" => Some(Self::FantomTestnet),
            "moonbeam" => Some(Self::Moonbeam),
            "moonriver" => Some(Self::Moonriver),
            "arbitrum" | "arb" => Some(Self::Arbitrum),
            "arbitrum-nova" | "arb-nova" => Some(Self::ArbitrumNova),
            "arbitrum-sepolia" | "arb-sepolia" => Some(Self::ArbitrumSepolia),
            "avalanche" | "avax" => Some(Self::Avalanche),
            "avalanche-fuji" | "fuji" => Some(Self::AvalancheFuji),
            "celo" => Some(Self::Celo),
            "base" => Some(Self::Base),
            "base-sepolia" => Some(Self::BaseSepolia),
            "linea" => Some(Self::Linea),
            "linea-testnet" => Some(Self::LineaTestnet),
            "zksync" | "zksync-era" | "era" => Some(Self::ZkSync),
            "zksync-sepolia" => Some(Self::ZkSyncSepolia),
            "scroll" => Some(Self::Scroll),
            "scroll-sepolia" => Some(Self::ScrollSepolia),
            "blast" => Some(Self::Blast),
            "blast-sepolia" => Some(Self::BlastSepolia),
            "mantle" => Some(Self::Mantle),
            "mode" => Some(Self::Mode),
            "fraxtal" => Some(Self::Fraxtal),
            "klaytn" | "klay" => Some(Self::Klaytn),
            "aurora" => Some(Self::Aurora),
            "polygon-zkevm" | "polygonzkevm" | "zkevm" => Some(Self::PolygonZkEvm),
            _ => None,
        }
    }

    /// List all known mainnets
    #[must_use]
    pub const fn mainnets() -> &'static [Chain] {
        &[
            Self::Ethereum,
            Self::Optimism,
            Self::Bsc,
            Self::Gnosis,
            Self::Polygon,
            Self::Fantom,
            Self::Moonbeam,
            Self::Moonriver,
            Self::Arbitrum,
            Self::ArbitrumNova,
            Self::Avalanche,
            Self::Celo,
            Self::Base,
            Self::Linea,
            Self::ZkSync,
            Self::Scroll,
            Self::Blast,
            Self::Mantle,
            Self::Mode,
            Self::Fraxtal,
            Self::Klaytn,
            Self::Aurora,
            Self::PolygonZkEvm,
        ]
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl From<u64> for Chain {
    fn from(id: u64) -> Self {
        Self::from_id(id)
    }
}

impl From<Chain> for u64 {
    fn from(chain: Chain) -> Self {
        chain.id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_id_roundtrip() {
        assert_eq!(Chain::from_id(1), Chain::Ethereum);
        assert_eq!(Chain::Ethereum.id(), 1);

        assert_eq!(Chain::from_id(137), Chain::Polygon);
        assert_eq!(Chain::Polygon.id(), 137);

        assert_eq!(Chain::from_id(8453), Chain::Base);
        assert_eq!(Chain::Base.id(), 8453);
    }

    #[test]
    fn test_chain_from_name() {
        assert_eq!(Chain::from_name("ethereum"), Some(Chain::Ethereum));
        assert_eq!(Chain::from_name("ETH"), Some(Chain::Ethereum));
        assert_eq!(Chain::from_name("mainnet"), Some(Chain::Ethereum));

        assert_eq!(Chain::from_name("polygon"), Some(Chain::Polygon));
        assert_eq!(Chain::from_name("MATIC"), Some(Chain::Polygon));

        assert_eq!(Chain::from_name("base"), Some(Chain::Base));
        assert_eq!(Chain::from_name("unknown-chain"), None);
    }

    #[test]
    fn test_is_testnet() {
        assert!(!Chain::Ethereum.is_testnet());
        assert!(Chain::Sepolia.is_testnet());
        assert!(!Chain::Base.is_testnet());
        assert!(Chain::BaseSepolia.is_testnet());
    }

    #[test]
    fn test_native_currency() {
        assert_eq!(Chain::Ethereum.native_currency(), "ETH");
        assert_eq!(Chain::Polygon.native_currency(), "MATIC");
        assert_eq!(Chain::Bsc.native_currency(), "BNB");
        assert_eq!(Chain::Avalanche.native_currency(), "AVAX");
    }

    #[test]
    fn test_unknown_chain() {
        let chain = Chain::from_id(999999);
        assert_eq!(chain, Chain::Other(999999));
        assert_eq!(chain.id(), 999999);
        assert_eq!(chain.name(), "unknown");
    }
}
