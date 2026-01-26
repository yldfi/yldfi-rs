//! On-chain lens queries for real-time Uniswap data
//!
//! This module uses ephemeral lens contracts to query pool state directly
//! from the blockchain without requiring any API keys.

use alloy::primitives::{Address, U256};
use alloy::providers::{Provider, ProviderBuilder};
use url::Url;

use crate::error::{lens_error, rpc_error, Result};
use crate::types::PoolState;

/// Well-known Uniswap factory addresses
pub mod factories {
    use alloy::primitives::address;

    /// V2 Factory addresses
    pub mod v2 {
        use super::*;

        /// Uniswap V2 Factory on Ethereum mainnet
        pub const MAINNET: alloy::primitives::Address =
            address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
        /// Uniswap V2 Factory on Arbitrum
        pub const ARBITRUM: alloy::primitives::Address =
            address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
        /// Uniswap V2 Factory on Optimism
        pub const OPTIMISM: alloy::primitives::Address =
            address!("0c3c1c532F1e39EdF36BE9Fe0bE1410313E074Bf");
        /// Uniswap V2 Factory on Polygon
        pub const POLYGON: alloy::primitives::Address =
            address!("5C69bEe701ef814a2B6a3EDD4B1652CB9cc5aA6f");
        /// Uniswap V2 Factory on Base
        pub const BASE: alloy::primitives::Address =
            address!("8909Dc15e40173Ff4699343b6eB8132c65e18eC6");
    }

    /// V3 Factory addresses
    pub mod v3 {
        use super::*;

        /// Uniswap V3 Factory on Ethereum mainnet
        pub const MAINNET: alloy::primitives::Address =
            address!("1F98431c8aD98523631AE4a59f267346ea31F984");
        /// Uniswap V3 Factory on Arbitrum
        pub const ARBITRUM: alloy::primitives::Address =
            address!("1F98431c8aD98523631AE4a59f267346ea31F984");
        /// Uniswap V3 Factory on Optimism
        pub const OPTIMISM: alloy::primitives::Address =
            address!("1F98431c8aD98523631AE4a59f267346ea31F984");
        /// Uniswap V3 Factory on Polygon
        pub const POLYGON: alloy::primitives::Address =
            address!("1F98431c8aD98523631AE4a59f267346ea31F984");
        /// Uniswap V3 Factory on Base
        pub const BASE: alloy::primitives::Address =
            address!("33128a8fC17869897dcE68Ed026d694621f6FDfD");
    }

    /// V4 Pool Manager addresses (V4 uses a single PoolManager instead of factories)
    pub mod v4 {
        use super::*;

        /// Uniswap V4 PoolManager on Ethereum mainnet
        pub const MAINNET: alloy::primitives::Address =
            address!("000000000004444c5dc75cB358380D2e3de08A90");
        /// Uniswap V4 PoolManager on Arbitrum
        pub const ARBITRUM: alloy::primitives::Address =
            address!("000000000004444c5dc75cB358380D2e3de08A90");
        /// Uniswap V4 PoolManager on Base
        pub const BASE: alloy::primitives::Address =
            address!("000000000004444c5dc75cB358380D2e3de08A90");
        /// Uniswap V4 PoolManager on Polygon
        pub const POLYGON: alloy::primitives::Address =
            address!("000000000004444c5dc75cB358380D2e3de08A90");
    }

    // Re-export V3 addresses at top level for backwards compatibility
    pub use v3::MAINNET;
    pub use v3::ARBITRUM;
    pub use v3::OPTIMISM;
    pub use v3::POLYGON;
    pub use v3::BASE;
}

/// Well-known pool addresses for common pairs
pub mod pools {
    use alloy::primitives::address;

    /// V2 pool addresses (pairs)
    pub mod v2 {
        use super::*;

        /// WETH/USDC pair on Ethereum mainnet (V2)
        pub const MAINNET_WETH_USDC: alloy::primitives::Address =
            address!("B4e16d0168e52d35CaCD2c6185b44281Ec28C9Dc");
        /// WETH/USDT pair on Ethereum mainnet (V2)
        pub const MAINNET_WETH_USDT: alloy::primitives::Address =
            address!("0d4a11d5EEaaC28EC3F61d100daF4d40471f1852");
        /// WBTC/WETH pair on Ethereum mainnet (V2)
        pub const MAINNET_WBTC_WETH: alloy::primitives::Address =
            address!("BB2b8038a1640196FbE3e38816F3e67Cba72D940");
        /// DAI/WETH pair on Ethereum mainnet (V2)
        pub const MAINNET_DAI_WETH: alloy::primitives::Address =
            address!("A478c2975Ab1Ea89e8196811F51A7B7Ade33eB11");
    }

    /// V3 pool addresses
    pub mod v3 {
        use super::*;

        /// WETH/USDC 0.05% on Ethereum mainnet
        pub const MAINNET_WETH_USDC_005: alloy::primitives::Address =
            address!("88e6A0c2dDD26FEEb64F039a2c41296FcB3f5640");
        /// WETH/USDC 0.3% on Ethereum mainnet
        pub const MAINNET_WETH_USDC_030: alloy::primitives::Address =
            address!("8ad599c3A0ff1De082011EFDDc58f1908eb6e6D8");
        /// WETH/USDT 0.05% on Ethereum mainnet
        pub const MAINNET_WETH_USDT_005: alloy::primitives::Address =
            address!("11b815efB8f581194ae79006d24E0d814B7697F6");
        /// WBTC/WETH 0.3% on Ethereum mainnet
        pub const MAINNET_WBTC_WETH_030: alloy::primitives::Address =
            address!("Cbcdf9626bC03E24f779434178A73a0B4bad62eD");
    }

    // Re-export V3 pools at top level for backwards compatibility
    pub use v3::MAINNET_WETH_USDC_005;
    pub use v3::MAINNET_WETH_USDC_030;
    pub use v3::MAINNET_WETH_USDT_005;
    pub use v3::MAINNET_WBTC_WETH_030;
}

/// Common token addresses
pub mod tokens {
    use alloy::primitives::address;

    /// WETH on Ethereum mainnet
    pub const MAINNET_WETH: alloy::primitives::Address =
        address!("C02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2");
    /// USDC on Ethereum mainnet
    pub const MAINNET_USDC: alloy::primitives::Address =
        address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
    /// USDT on Ethereum mainnet
    pub const MAINNET_USDT: alloy::primitives::Address =
        address!("dAC17F958D2ee523a2206206994597C13D831ec7");
    /// WBTC on Ethereum mainnet
    pub const MAINNET_WBTC: alloy::primitives::Address =
        address!("2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599");
    /// DAI on Ethereum mainnet
    pub const MAINNET_DAI: alloy::primitives::Address =
        address!("6B175474E89094C44Da98b954EedeAC495271d0F");
}

/// HTTP provider type alias
type HttpProvider = alloy::providers::fillers::FillProvider<
    alloy::providers::fillers::JoinFill<
        alloy::providers::Identity,
        alloy::providers::fillers::JoinFill<
            alloy::providers::fillers::GasFiller,
            alloy::providers::fillers::JoinFill<
                alloy::providers::fillers::BlobGasFiller,
                alloy::providers::fillers::JoinFill<
                    alloy::providers::fillers::NonceFiller,
                    alloy::providers::fillers::ChainIdFiller,
                >,
            >,
        >,
    >,
    alloy::providers::RootProvider,
>;

/// Client for on-chain Uniswap queries via lens contracts
pub struct LensClient {
    provider: HttpProvider,
    #[allow(dead_code)]
    factory: Address,
}

impl std::fmt::Debug for LensClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LensClient")
            .field("factory", &self.factory)
            .finish_non_exhaustive()
    }
}

impl LensClient {
    /// Create a new lens client for Ethereum mainnet
    pub fn mainnet(rpc_url: &str) -> Result<Self> {
        Self::new(rpc_url, factories::MAINNET)
    }

    /// Create a new lens client for Arbitrum
    pub fn arbitrum(rpc_url: &str) -> Result<Self> {
        Self::new(rpc_url, factories::ARBITRUM)
    }

    /// Create a new lens client for Optimism
    pub fn optimism(rpc_url: &str) -> Result<Self> {
        Self::new(rpc_url, factories::OPTIMISM)
    }

    /// Create a new lens client for Base
    pub fn base(rpc_url: &str) -> Result<Self> {
        Self::new(rpc_url, factories::BASE)
    }

    /// Create a new lens client with custom factory
    pub fn new(rpc_url: &str, factory: Address) -> Result<Self> {
        let url = Url::parse(rpc_url).map_err(|e| rpc_error(e.to_string()))?;
        let provider = ProviderBuilder::new().connect_http(url);
        Ok(Self { provider, factory })
    }

    /// Get pool slot0 (current state)
    pub async fn get_pool_state(&self, pool: Address) -> Result<PoolState> {
        // Call slot0() on the pool contract
        // slot0 returns: (sqrtPriceX96, tick, observationIndex, observationCardinality,
        //                 observationCardinalityNext, feeProtocol, unlocked)

        // Using raw eth_call since we don't want to generate full contract bindings
        let slot0_selector = [0x3850c7bd_u32.to_be_bytes()].concat(); // slot0()

        let call_request = alloy::rpc::types::TransactionRequest::default()
            .to(pool)
            .input(slot0_selector.into());

        let result = self
            .provider
            .call(call_request)
            .await
            .map_err(|e| lens_error(format!("slot0 call failed: {}", e)))?;

        // Decode the result (7 values: uint160, int24, uint16, uint16, uint16, uint8, bool)
        if result.len() < 224 {
            // 7 * 32 bytes
            return Err(lens_error("Invalid slot0 response length"));
        }

        let sqrt_price_x96 = U256::from_be_slice(&result[0..32]);
        let tick = i32::from_be_bytes(result[60..64].try_into().unwrap());
        let observation_index = u16::from_be_bytes(result[94..96].try_into().unwrap());
        let observation_cardinality = u16::from_be_bytes(result[126..128].try_into().unwrap());
        let observation_cardinality_next = u16::from_be_bytes(result[158..160].try_into().unwrap());
        let fee_protocol = result[191];
        let unlocked = result[223] != 0;

        Ok(PoolState {
            sqrt_price_x96,
            tick,
            observation_index,
            observation_cardinality,
            observation_cardinality_next,
            fee_protocol,
            unlocked,
        })
    }

    /// Get pool liquidity
    pub async fn get_liquidity(&self, pool: Address) -> Result<u128> {
        // liquidity() selector
        let selector = [0x1a686502_u32.to_be_bytes()].concat();

        let call_request = alloy::rpc::types::TransactionRequest::default()
            .to(pool)
            .input(selector.into());

        let result = self
            .provider
            .call(call_request)
            .await
            .map_err(|e| lens_error(format!("liquidity call failed: {}", e)))?;

        if result.len() < 32 {
            return Err(lens_error("Invalid liquidity response"));
        }

        let liquidity = u128::from_be_bytes(result[16..32].try_into().unwrap());
        Ok(liquidity)
    }

    /// Get token balance
    pub async fn get_token_balance(&self, token: Address, account: Address) -> Result<U256> {
        // balanceOf(address) selector: 0x70a08231
        let mut calldata = vec![0x70, 0xa0, 0x82, 0x31];
        calldata.extend_from_slice(&[0u8; 12]); // padding
        calldata.extend_from_slice(account.as_slice());

        let call_request = alloy::rpc::types::TransactionRequest::default()
            .to(token)
            .input(calldata.into());

        let result = self
            .provider
            .call(call_request)
            .await
            .map_err(|e| lens_error(format!("balanceOf call failed: {}", e)))?;

        if result.len() < 32 {
            return Err(lens_error("Invalid balance response"));
        }

        Ok(U256::from_be_slice(&result[0..32]))
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        self.provider
            .get_block_number()
            .await
            .map_err(|e| rpc_error(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_factory_addresses() {
        // Ensure factory addresses are valid
        assert!(!factories::MAINNET.is_zero());
        assert!(!factories::BASE.is_zero());
    }

    #[test]
    fn test_pool_addresses() {
        assert!(!pools::MAINNET_WETH_USDC_005.is_zero());
        assert!(!pools::MAINNET_WETH_USDC_030.is_zero());
    }
}
