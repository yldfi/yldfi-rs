//! Chainlink addresses and constants

use alloy::primitives::{address, Address};

/// Chainlink Feed Registry on Ethereum mainnet
/// https://docs.chain.link/data-feeds/feed-registry
pub const FEED_REGISTRY: Address = address!("47Fb2585D2C56Fe188D0E6ec628a38b74fCeeeDf");

/// Denomination addresses for Feed Registry queries
pub mod denominations {
    use alloy::primitives::{address, Address};

    /// USD denomination (ISO 4217 code 840)
    pub const USD: Address = address!("0000000000000000000000000000000000000348");

    /// ETH denomination
    pub const ETH: Address = address!("EeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE");

    /// BTC denomination
    pub const BTC: Address = address!("bBbBBBBbbBBBbbbBbbBbbbbBBbBbbbbBbBbbBBbB");

    /// GBP denomination (ISO 4217 code 826)
    pub const GBP: Address = address!("000000000000000000000000000000000000033a");

    /// EUR denomination (ISO 4217 code 978)
    pub const EUR: Address = address!("00000000000000000000000000000000000003d2");
}

/// Known oracle addresses by chain
pub mod oracles {
    use alloy::primitives::{address, Address};

    /// Ethereum mainnet oracles
    pub mod ethereum {
        use super::*;

        pub const ETH_USD: Address = address!("5f4eC3Df9cbd43714FE2740f5E3616155c5b8419");
        pub const BTC_USD: Address = address!("F4030086522a5bEEa4988F8cA5B36dbC97BeE88c");
        pub const LINK_USD: Address = address!("2c1d072e956AFFC0D435Cb7AC38EF18d24d9127c");
        pub const USDC_USD: Address = address!("8fFfFfd4AfB6115b954Bd326cbe7B4BA576818f6");
        pub const USDT_USD: Address = address!("3E7d1eAB13ad0104d2750B8863b489D65364e32D");
        pub const DAI_USD: Address = address!("AED0c38402a5d19df6E4c03F4E2DceD6e29c1ee9");
        pub const CVX_USD: Address = address!("d962fC30A72A84cE50161031391756Bf2876Af5D");
        pub const CRV_USD: Address = address!("Cd627aA160A6fA45Eb793D19eF54F5062F20f33f");
        pub const CRVUSD_USD: Address = address!("EEf0C605546958c1f899b6fB336C20671f9cD49F");
        pub const AAVE_USD: Address = address!("547a514d5e3769680Ce22B2361c10Ea13619e8a9");
        pub const UNI_USD: Address = address!("553303d460EE0afB37EdFf9bE42922D8FF63220e");
        pub const COMP_USD: Address = address!("dbd020CAeF83eFd542f4De03e3cF0C28A4428bd5");
        pub const MKR_USD: Address = address!("ec1D1B3b0443256cc3860e24a46F108e699484Aa");
        pub const SNX_USD: Address = address!("DEc0a100EaD1FaA37407f0EDc76033426CF90b82");
        pub const YFI_USD: Address = address!("A027702dbb89fbd58938e4324ac03B58d812b0E1");
        pub const SUSHI_USD: Address = address!("Cc70F09A6CC17553b2E31954cD36E4A2d89501f7");
        /// Note: LDO only has an ETH-denominated feed on mainnet, not USD
        pub const LDO_ETH: Address = address!("4e844125952D32AcdF339BE976c98E22F6F318dB");
        pub const RPL_USD: Address = address!("4E155eD98aFe9034b7A5962f6C84c86d869daA9d");
        pub const STETH_USD: Address = address!("CfE54B5cD566aB89272946F602D76Ea879CAb4a8");
        /// stETH/ETH exchange rate (wstETH/ETH is calculated via stETH/wstETH rate)
        pub const STETH_ETH: Address = address!("86392dC19c0b719886221c78AB11eb8Cf5c52812");
        pub const RETH_ETH: Address = address!("536218f9E9Eb48863970252233c8F271f554C2d0");
        pub const CBETH_ETH: Address = address!("F017fcB346A1885194689bA23Eff2fE6fA5C483b");
    }

    /// Arbitrum oracles
    pub mod arbitrum {
        use super::*;

        pub const ETH_USD: Address = address!("639Fe6ab55C921f74e7fac1ee960C0B6293ba612");
        pub const BTC_USD: Address = address!("6ce185860a4963106506C203335A68A2B5E60900");
        pub const LINK_USD: Address = address!("86E53CF1B870786351Da77A57575e79CB55812CB");
        pub const USDC_USD: Address = address!("50834F3163758fcC1Df9973b6e91f0F0F0434aD3");
        pub const USDT_USD: Address = address!("3f3f5dF88dC9F13eac63DF89EC16ef6e7E25DdE7");
        pub const DAI_USD: Address = address!("c5C8E77B397E531B8EC06BFb0048328B30E9eCfB");
        pub const ARB_USD: Address = address!("b2A824043730FE05F3DA2efaFa1CBbe83fa548D6");
        pub const GMX_USD: Address = address!("DB98056FecFff59D032aB628337A4887110df3dB");
    }

    /// Polygon oracles
    pub mod polygon {
        use super::*;

        pub const ETH_USD: Address = address!("F9680D99D6C9589e2a93a78A04A279e509205945");
        pub const BTC_USD: Address = address!("c907E116054Ad103354f2D350FD2514433D57F6f");
        pub const MATIC_USD: Address = address!("AB594600376Ec9fD91F8e885dADF0CE036862dE0");
        pub const LINK_USD: Address = address!("d9FFdb71EbE7496cC440152d43986Aae0AB76665");
        pub const USDC_USD: Address = address!("fE4A8cc5b5B2366C1B58Bea3858e81843581b2F7");
        pub const USDT_USD: Address = address!("0A6513e40db6EB1b165753AD52E80663aeA50545");
        pub const DAI_USD: Address = address!("4746DeC9e833A82EC7C2C1356372CcF2cfcD2F3D");
        pub const AAVE_USD: Address = address!("72484B12719E23115761D5DA1646945632979bB6");
    }

    /// Optimism oracles
    pub mod optimism {
        use super::*;

        pub const ETH_USD: Address = address!("13e3Ee699D1909E989722E753853AE30b17e08c5");
        pub const BTC_USD: Address = address!("D702DD976Fb76Fffc2D3963D037dfDae5b04E593");
        pub const LINK_USD: Address = address!("CbC8EEA53a9575F05d4b18a4C5F0121c2a6bd035");
        pub const USDC_USD: Address = address!("16a9FA2FDa030272Ce99B29CF780dFA30361E0f3");
        pub const USDT_USD: Address = address!("ECef79E109e997bCA29c1c0897ec9d7b03647F5E");
        pub const DAI_USD: Address = address!("8dBa75e83DA73cc766A7e5a0ee71F656BAb470d6");
        pub const OP_USD: Address = address!("0D276FC14719f9292D5C1eA2198673d1f4269246");
    }

    /// Base oracles
    pub mod base {
        use super::*;

        pub const ETH_USD: Address = address!("71041dddad3595F9CEd3DcCFBe3D1F4b0a16Bb70");
        pub const USDC_USD: Address = address!("7e860098F58bBFC8648a4311b374B1D669a2bc6B");
        pub const CBETH_ETH: Address = address!("806b4Ac04501c29769051e42783cF04dCE41440b");
    }
}

/// Token addresses used for Feed Registry lookups
pub mod tokens {
    use alloy::primitives::{address, Address};

    // Stablecoins
    pub const USDC: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    pub const USDT: Address = address!("dAC17F958D2ee523a2206206994597C13D831ec7");
    pub const DAI: Address = address!("6B175474E89094C44Da98b954EedeAC495271d0F");

    // Major tokens
    pub const WETH: Address = address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    pub const WBTC: Address = address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599");
    pub const LINK: Address = address!("514910771AF9Ca656af840dff83E8264EcF986CA");

    // DeFi tokens
    pub const CVX: Address = address!("4e3FBD56CD56c3e72c1403e103b45Db9da5B9D2B");
    pub const CRV: Address = address!("D533a949740bb3306d119CC777fa900bA034cd52");
    pub const CRVUSD: Address = address!("f939E0A03FB07F59A73314E73794Be0E57ac1b4E");
    pub const AAVE: Address = address!("7Fc66500c84A76Ad7e9c93437bFc5Ac33E2DDaE9");
    pub const UNI: Address = address!("1f9840a85d5aF5bf1D1762F925BDADdC4201F984");
    pub const COMP: Address = address!("c00e94Cb662C3520282E6f5717214004A7f26888");
    pub const MKR: Address = address!("9f8F72aA9304c8B593d555F12eF6589cC3A579A2");
    pub const SNX: Address = address!("C011a73ee8576Fb46F5E1c5751cA3B9Fe0af2a6F");
    pub const YFI: Address = address!("0bc529c00C6401aEF6D220BE8C6Ea1667F6Ad93e");
    pub const SUSHI: Address = address!("6B3595068778DD592e39A122f4f5a5cF09C90fE2");
    pub const LDO: Address = address!("5A98FcBEA516Cf06857215779Fd812CA3beF1B32");
    pub const RPL: Address = address!("D33526068D116cE69F19A9ee46F0bd304F21A51f");

    // Liquid staking
    pub const STETH: Address = address!("ae7ab96520DE3A18E5e111B5EaAb095312D7fE84");
    pub const WSTETH: Address = address!("7f39C581F595B53c5cb19bD0b3f8dA6c935E2Ca0");
    pub const RETH: Address = address!("ae78736Cd615f374D3085123A210448E74Fc6393");
    pub const CBETH: Address = address!("Be9895146f7AF43049ca1c1AE358B0541Ea49704");
}

/// Map token symbol to address for Feed Registry lookup.
///
/// Supports common token symbols (case-insensitive) including:
/// - Major tokens: ETH, BTC, WETH, WBTC, LINK
/// - Stablecoins: USDC, USDT, DAI
/// - DeFi tokens: CVX, CRV, AAVE, UNI, COMP, MKR, SNX, YFI, SUSHI, LDO, RPL
/// - Liquid staking: STETH, WSTETH, RETH, CBETH
///
/// Returns `None` for unknown symbols.
pub fn symbol_to_address(symbol: &str) -> Option<Address> {
    match symbol.to_uppercase().as_str() {
        // Use ETH denomination for ETH
        "ETH" | "ETHEREUM" => Some(denominations::ETH),
        "BTC" | "BITCOIN" => Some(denominations::BTC),

        // Stablecoins
        "USDC" => Some(tokens::USDC),
        "USDT" | "TETHER" => Some(tokens::USDT),
        "DAI" => Some(tokens::DAI),

        // Major tokens
        "WETH" => Some(tokens::WETH),
        "WBTC" => Some(tokens::WBTC),
        "LINK" | "CHAINLINK" => Some(tokens::LINK),

        // DeFi tokens
        "CVX" | "CONVEX" => Some(tokens::CVX),
        "CRV" | "CURVE" => Some(tokens::CRV),
        "CRVUSD" => Some(tokens::CRVUSD),
        "AAVE" => Some(tokens::AAVE),
        "UNI" | "UNISWAP" => Some(tokens::UNI),
        "COMP" | "COMPOUND" => Some(tokens::COMP),
        "MKR" | "MAKER" => Some(tokens::MKR),
        "SNX" | "SYNTHETIX" => Some(tokens::SNX),
        "YFI" | "YEARN" => Some(tokens::YFI),
        "SUSHI" | "SUSHISWAP" => Some(tokens::SUSHI),
        "LDO" | "LIDO" => Some(tokens::LDO),
        "RPL" | "ROCKETPOOL" => Some(tokens::RPL),

        // Liquid staking
        "STETH" => Some(tokens::STETH),
        "WSTETH" => Some(tokens::WSTETH),
        "RETH" => Some(tokens::RETH),
        "CBETH" => Some(tokens::CBETH),

        _ => None,
    }
}

/// Get oracle address for a token on a specific chain.
///
/// This bypasses the Feed Registry and returns the direct oracle address.
/// Use this for L2 chains (which don't have a Feed Registry) or when you
/// want to query a specific oracle directly.
///
/// # Arguments
/// * `symbol` - Token symbol (case-insensitive), e.g., "ETH", "BTC", "USDC"
/// * `chain` - Chain name: "ethereum", "arbitrum", "polygon", "optimism", or "base"
///
/// # Returns
/// The oracle address if a known mapping exists, or `None` otherwise.
pub fn get_oracle_for_token(symbol: &str, chain: &str) -> Option<Address> {
    let sym = symbol.to_uppercase();
    match chain.to_lowercase().as_str() {
        "ethereum" | "mainnet" | "eth" => match sym.as_str() {
            "ETH" | "ETHEREUM" => Some(oracles::ethereum::ETH_USD),
            "BTC" | "BITCOIN" | "WBTC" => Some(oracles::ethereum::BTC_USD),
            "LINK" | "CHAINLINK" => Some(oracles::ethereum::LINK_USD),
            "USDC" => Some(oracles::ethereum::USDC_USD),
            "USDT" | "TETHER" => Some(oracles::ethereum::USDT_USD),
            "DAI" => Some(oracles::ethereum::DAI_USD),
            "CVX" | "CONVEX" => Some(oracles::ethereum::CVX_USD),
            "CRV" | "CURVE" => Some(oracles::ethereum::CRV_USD),
            "CRVUSD" => Some(oracles::ethereum::CRVUSD_USD),
            "AAVE" => Some(oracles::ethereum::AAVE_USD),
            "UNI" | "UNISWAP" => Some(oracles::ethereum::UNI_USD),
            "COMP" | "COMPOUND" => Some(oracles::ethereum::COMP_USD),
            "MKR" | "MAKER" => Some(oracles::ethereum::MKR_USD),
            "SNX" | "SYNTHETIX" => Some(oracles::ethereum::SNX_USD),
            "YFI" | "YEARN" => Some(oracles::ethereum::YFI_USD),
            "SUSHI" | "SUSHISWAP" => Some(oracles::ethereum::SUSHI_USD),
            // Note: LDO only has ETH-denominated feed on mainnet (not USD)
            // "LDO" | "LIDO" => Some(oracles::ethereum::LDO_ETH),
            "RPL" | "ROCKETPOOL" => Some(oracles::ethereum::RPL_USD),
            "STETH" => Some(oracles::ethereum::STETH_USD),
            _ => None,
        },
        "arbitrum" | "arb" => match sym.as_str() {
            "ETH" | "ETHEREUM" => Some(oracles::arbitrum::ETH_USD),
            "BTC" | "BITCOIN" | "WBTC" => Some(oracles::arbitrum::BTC_USD),
            "LINK" | "CHAINLINK" => Some(oracles::arbitrum::LINK_USD),
            "USDC" => Some(oracles::arbitrum::USDC_USD),
            "USDT" | "TETHER" => Some(oracles::arbitrum::USDT_USD),
            "DAI" => Some(oracles::arbitrum::DAI_USD),
            "ARB" | "ARBITRUM" => Some(oracles::arbitrum::ARB_USD),
            "GMX" => Some(oracles::arbitrum::GMX_USD),
            _ => None,
        },
        "polygon" | "matic" => match sym.as_str() {
            "ETH" | "ETHEREUM" => Some(oracles::polygon::ETH_USD),
            "BTC" | "BITCOIN" | "WBTC" => Some(oracles::polygon::BTC_USD),
            "MATIC" | "POLYGON" => Some(oracles::polygon::MATIC_USD),
            "LINK" | "CHAINLINK" => Some(oracles::polygon::LINK_USD),
            "USDC" => Some(oracles::polygon::USDC_USD),
            "USDT" | "TETHER" => Some(oracles::polygon::USDT_USD),
            "DAI" => Some(oracles::polygon::DAI_USD),
            "AAVE" => Some(oracles::polygon::AAVE_USD),
            _ => None,
        },
        "optimism" | "op" => match sym.as_str() {
            "ETH" | "ETHEREUM" => Some(oracles::optimism::ETH_USD),
            "BTC" | "BITCOIN" | "WBTC" => Some(oracles::optimism::BTC_USD),
            "LINK" | "CHAINLINK" => Some(oracles::optimism::LINK_USD),
            "USDC" => Some(oracles::optimism::USDC_USD),
            "DAI" => Some(oracles::optimism::DAI_USD),
            "OP" | "OPTIMISM" => Some(oracles::optimism::OP_USD),
            _ => None,
        },
        "base" => match sym.as_str() {
            "ETH" | "ETHEREUM" => Some(oracles::base::ETH_USD),
            "USDC" => Some(oracles::base::USDC_USD),
            _ => None,
        },
        _ => None,
    }
}

/// List all tokens with known oracle addresses for a chain.
///
/// Returns symbols that can be passed to `get_oracle_for_token()` for this chain.
/// Note: On Ethereum mainnet, more tokens may be available via the Feed Registry
/// than are listed here.
pub fn supported_tokens(chain: &str) -> Vec<&'static str> {
    match chain.to_lowercase().as_str() {
        "ethereum" | "mainnet" | "eth" => vec![
            "ETH", "BTC", "LINK", "USDC", "USDT", "DAI", "CVX", "CRV", "CRVUSD", "AAVE", "UNI",
            "COMP", "MKR", "SNX", "YFI", "SUSHI", "RPL", "STETH",
        ],
        "arbitrum" | "arb" => {
            vec!["ETH", "BTC", "LINK", "USDC", "USDT", "DAI", "ARB", "GMX"]
        }
        "polygon" | "matic" => {
            vec!["ETH", "BTC", "MATIC", "LINK", "USDC", "USDT", "DAI", "AAVE"]
        }
        "optimism" | "op" => vec!["ETH", "BTC", "LINK", "USDC", "DAI", "OP"],
        "base" => vec!["ETH", "USDC"],
        _ => vec![],
    }
}
