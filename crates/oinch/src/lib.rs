//! Rust client for the 1inch DEX Aggregator Swap API v6.0
//!
//! 1inch is one of the most popular DEX aggregators, offering optimal swap routes
//! across hundreds of liquidity sources on multiple chains. This crate provides
//! a type-safe Rust interface to the 1inch Swap API.
//!
//! # Features
//!
//! - Multi-chain support (Ethereum, BSC, Polygon, Arbitrum, Optimism, Base, etc.)
//! - Advanced Pathfinder algorithm for optimal routing
//! - Type-safe request/response handling
//! - Automatic rate limiting awareness
//! - Token approval helpers
//!
//! # Authentication
//!
//! The 1inch API requires an API key for authentication. Get your API key at
//! [https://portal.1inch.dev](https://portal.1inch.dev).
//!
//! # Rate Limits
//!
//! - **Free tier**: 1 request per second, 100,000 calls per month
//! - Higher tiers available for production use
//!
//! # Quick Start
//!
//! ```no_run
//! use oinch::{Client, Chain, QuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), oinch::Error> {
//!     // Create client with your API key
//!     let client = Client::new("your-api-key")?;
//!
//!     // Get a quote for swapping 1 ETH to USDC on Ethereum
//!     let request = QuoteRequest::new(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // Native ETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 ETH in wei
//!     )
//!     .with_tokens_info()
//!     .with_protocols_info();
//!
//!     let quote = client.get_quote(Chain::Ethereum, &request).await?;
//!     println!("You will receive: {} USDC (in minimal units)", quote.to_amount);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Getting Transaction Data
//!
//! To execute a swap, use `get_swap` which returns complete transaction data:
//!
//! ```no_run
//! use oinch::{Client, Chain, SwapRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), oinch::Error> {
//!     let client = Client::new("your-api-key")?;
//!
//!     let request = SwapRequest::new(
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE", // ETH
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "1000000000000000000", // 1 ETH
//!         "0xYourWalletAddress",
//!         1.0, // 1% slippage tolerance
//!     );
//!
//!     let swap = client.get_swap(Chain::Ethereum, &request).await?;
//!
//!     // Use with ethers/alloy to send the transaction
//!     println!("To: {}", swap.tx.to);
//!     println!("Data: {}", swap.tx.data);
//!     println!("Value: {}", swap.tx.value);
//!
//!     Ok(())
//! }
//! ```
//!
//! # Token Approval
//!
//! Before swapping ERC20 tokens (not native ETH), you need to approve the
//! 1inch router to spend your tokens:
//!
//! ```no_run
//! use oinch::{Client, Chain};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), oinch::Error> {
//!     let client = Client::new("your-api-key")?;
//!
//!     // Check current allowance
//!     let allowance = client.get_approve_allowance(
//!         Chain::Ethereum,
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
//!         "0xYourWalletAddress",
//!     ).await?;
//!
//!     if allowance == "0" {
//!         // Get approval transaction data
//!         let approval_tx = client.get_approve_transaction(
//!             Chain::Ethereum,
//!             "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
//!             None, // unlimited approval
//!         ).await?;
//!
//!         println!("Send approval to: {}", approval_tx.to);
//!         println!("Data: {}", approval_tx.data);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Supported Chains
//!
//! The client supports the following chains:
//!
//! | Chain | Chain ID |
//! |-------|----------|
//! | Ethereum | 1 |
//! | BNB Smart Chain | 56 |
//! | Polygon | 137 |
//! | Optimism | 10 |
//! | Arbitrum One | 42161 |
//! | Gnosis Chain | 100 |
//! | Avalanche | 43114 |
//! | Fantom | 250 |
//! | Klaytn | 8217 |
//! | Aurora | 1313161554 |
//! | zkSync Era | 324 |
//! | Base | 8453 |
//! | Linea | 59144 |
//!
//! See [`Chain`] for the full list.
//!
//! # Error Handling
//!
//! The crate provides detailed error types for different failure modes:
//!
//! - [`Error::Http`] - Network/HTTP errors
//! - [`Error::Json`] - JSON parsing errors
//! - [`Error::Api`] - API-level errors (invalid params, etc.)
//! - [`Error::RateLimited`] - Rate limit exceeded
//! - [`Error::ServerError`] - 1inch server errors
//!
//! All errors implement the [`RetryableError`] trait for use with retry utilities.

pub mod client;
pub mod error;
pub mod types;

pub use client::{Client, Config, DEFAULT_BASE_URL};
pub use error::{Error, Result};
pub use types::{
    AllowanceResponse, ApiErrorResponse, ApprovalTransaction, Chain, LiquiditySource,
    LiquiditySourcesResponse, ParseChainError, ProtocolInfo, QuoteRequest, QuoteResponse,
    SpenderResponse, SwapRequest, SwapResponse, TokenInfo, TokenListResponse, TransactionData,
};

// Re-export common utilities
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Create a config with an API key
#[must_use]
pub fn config_with_api_key(api_key: impl Into<String>) -> Config {
    Config::new(api_key)
}

/// Common token addresses across chains
pub mod tokens {
    /// Native token address (used for ETH, MATIC, BNB, etc.)
    pub const NATIVE: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

    /// Ethereum mainnet tokens
    pub mod ethereum {
        /// Native ETH
        pub const ETH: &str = super::NATIVE;
        /// USDC (Circle)
        pub const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
        /// USDT (Tether)
        pub const USDT: &str = "0xdAC17F958D2ee523a2206206994597C13D831ec7";
        /// WETH (Wrapped ETH)
        pub const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
        /// DAI
        pub const DAI: &str = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
        /// WBTC (Wrapped Bitcoin)
        pub const WBTC: &str = "0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599";
    }

    /// Polygon tokens
    pub mod polygon {
        /// Native MATIC
        pub const MATIC: &str = super::NATIVE;
        /// USDC (Circle)
        pub const USDC: &str = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";
        /// USDC.e (Bridged USDC)
        pub const USDC_E: &str = "0x2791Bca1f2de4661ED88A30C99A7a9449Aa84174";
        /// USDT (Tether)
        pub const USDT: &str = "0xc2132D05D31c914a87C6611C10748AEb04B58e8F";
        /// WMATIC (Wrapped MATIC)
        pub const WMATIC: &str = "0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270";
        /// WETH
        pub const WETH: &str = "0x7ceB23fD6bC0adD59E62ac25578270cFf1b9f619";
    }

    /// Arbitrum One tokens
    pub mod arbitrum {
        /// Native ETH
        pub const ETH: &str = super::NATIVE;
        /// USDC (Circle - native)
        pub const USDC: &str = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831";
        /// USDC.e (Bridged USDC)
        pub const USDC_E: &str = "0xFF970A61A04b1cA14834A43f5dE4533eBDDB5CC8";
        /// USDT (Tether)
        pub const USDT: &str = "0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9";
        /// WETH
        pub const WETH: &str = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
        /// ARB (Arbitrum token)
        pub const ARB: &str = "0x912CE59144191C1204E64559FE8253a0e49E6548";
    }

    /// Base tokens
    pub mod base {
        /// Native ETH
        pub const ETH: &str = super::NATIVE;
        /// USDC (Circle - native)
        pub const USDC: &str = "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913";
        /// USDbC (Bridged USDC)
        pub const USDBC: &str = "0xd9aAEc86B65D86f6A7B5B1b0c42FFA531710b6CA";
        /// WETH
        pub const WETH: &str = "0x4200000000000000000000000000000000000006";
    }

    /// BNB Smart Chain tokens
    pub mod bsc {
        /// Native BNB
        pub const BNB: &str = super::NATIVE;
        /// USDC
        pub const USDC: &str = "0x8AC76a51cc950d9822D68b83fE1Ad97B32Cd580d";
        /// USDT
        pub const USDT: &str = "0x55d398326f99059fF775485246999027B3197955";
        /// WBNB
        pub const WBNB: &str = "0xbb4CdB9CBd36B01bD1cBaEBF2De08d9173bc095c";
        /// BUSD
        pub const BUSD: &str = "0xe9e7CEA3DedcA5984780Bafc599bD69ADd087D56";
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reexports() {
        // Verify all main types are accessible
        let _chain = Chain::Ethereum;
        let _request = QuoteRequest::new("0x", "0x", "1000");
        let _swap_request = SwapRequest::new("0x", "0x", "1000", "0x", 1.0);
    }

    #[test]
    fn test_token_constants() {
        assert_eq!(tokens::NATIVE, "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE");
        assert_eq!(tokens::ethereum::ETH, tokens::NATIVE);
        assert_eq!(
            tokens::ethereum::USDC,
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
        );
    }
}
