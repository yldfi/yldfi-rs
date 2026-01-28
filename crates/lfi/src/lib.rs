//! Rust client for the LI.FI cross-chain bridge and DEX aggregator API
//!
//! LI.FI is a multi-chain liquidity aggregation protocol that integrates multiple
//! bridges and DEXs to provide optimal cross-chain swap routes. It offers a unified
//! API for accessing liquidity across 20+ chains.
//!
//! # Features
//!
//! - **Cross-chain swaps**: Swap tokens across different blockchains in a single transaction
//! - **Bridge aggregation**: Access multiple bridges (Stargate, Hop, Connext, Across, etc.)
//! - **DEX aggregation**: Optimal routing through DEXs on each chain
//! - **Route optimization**: Find the best route by price, speed, or security
//! - **Transaction tracking**: Monitor cross-chain transaction status
//!
//! # Quick Start
//!
//! ```no_run
//! use lfi::{Client, QuoteRequest, chains};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), lfi::Error> {
//!     // Create a client with an integrator identifier (recommended)
//!     let client = Client::with_integrator("my-app")?;
//!
//!     // Get a quote for swapping 1 ETH on Ethereum to USDC on Arbitrum
//!     let request = QuoteRequest::new(
//!         chains::ETHEREUM,                                  // Source chain
//!         chains::ARBITRUM,                                  // Destination chain
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",      // Native ETH
//!         "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",      // USDC on Arbitrum
//!         "1000000000000000000",                             // 1 ETH in wei
//!         "0xYourWalletAddress",
//!     ).with_slippage(0.5);
//!
//!     let quote = client.get_quote(&request).await?;
//!     println!("Estimated output: {}", quote.estimate.to_amount);
//!
//!     // Get transaction data to execute
//!     if let Some(tx) = quote.transaction_request {
//!         println!("Send transaction to: {}", tx.to);
//!         println!("Data: {}", tx.data);
//!         println!("Value: {}", tx.value);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Getting Multiple Routes
//!
//! Use the advanced routes API to get multiple route options:
//!
//! ```no_run
//! use lfi::{Client, RoutesRequest, RoutesOptions, RouteOrder, chains};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), lfi::Error> {
//!     let client = Client::with_integrator("my-app")?;
//!
//!     let options = RoutesOptions::new()
//!         .with_slippage(0.5)
//!         .with_order(RouteOrder::Cheapest);  // Sort by best output
//!
//!     let request = RoutesRequest::new(
//!         chains::ETHEREUM,
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",  // USDC on Ethereum
//!         "100000000",                                    // 100 USDC (6 decimals)
//!         "0xYourWalletAddress",
//!         chains::BASE,
//!         "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913",  // USDC on Base
//!     ).with_options(options);
//!
//!     let response = client.get_routes(&request).await?;
//!
//!     for route in &response.routes {
//!         println!("Route via {:?}", route.steps.iter().map(|s| &s.tool).collect::<Vec<_>>());
//!         println!("  Output: {}", route.to_amount);
//!         println!("  Gas cost: {:?}", route.gas_cost_usd);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Tracking Cross-Chain Transactions
//!
//! Monitor the status of cross-chain swaps:
//!
//! ```no_run
//! use lfi::{Client, StatusRequest, TransactionStatus};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), lfi::Error> {
//!     let client = Client::new()?;
//!
//!     let request = StatusRequest::new("0xYourTxHash")
//!         .with_bridge("stargate")
//!         .with_from_chain(lfi::chains::ETHEREUM)
//!         .with_to_chain(lfi::chains::ARBITRUM);
//!
//!     loop {
//!         let status = client.get_status(&request).await?;
//!
//!         match status.status {
//!             TransactionStatus::Done => {
//!                 println!("Transaction complete!");
//!                 if let Some(receiving) = status.receiving {
//!                     println!("Received tx: {}", receiving.tx_hash);
//!                 }
//!                 break;
//!             }
//!             TransactionStatus::Pending => {
//!                 println!("Still processing...");
//!                 tokio::time::sleep(std::time::Duration::from_secs(10)).await;
//!             }
//!             TransactionStatus::Failed => {
//!                 println!("Transaction failed: {:?}", status.substatus_message);
//!                 break;
//!             }
//!             _ => break,
//!         }
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Supported Chains
//!
//! LI.FI supports 20+ chains including:
//! - **EVM chains**: Ethereum, Polygon, Arbitrum, Optimism, Base, BSC, Avalanche, etc.
//! - **L2s**: zkSync, Scroll, Linea, Mantle, Blast, Mode
//! - **Non-EVM**: Solana (via specific bridges)
//!
//! Use the [`chains`] module for common chain IDs:
//!
//! ```
//! use lfi::chains;
//!
//! let eth = chains::ETHEREUM;      // 1
//! let arb = chains::ARBITRUM;      // 42161
//! let base = chains::BASE;         // 8453
//! let op = chains::OPTIMISM;       // 10
//! ```
//!
//! # Integrator String
//!
//! LI.FI recommends using an integrator string for production applications.
//! This helps track API usage and enables features like fee collection:
//!
//! ```no_run
//! use lfi::{Client, Config};
//!
//! // Simple way
//! let client = Client::with_integrator("my-dapp")?;
//!
//! // With full configuration
//! let config = Config::new()
//!     .with_integrator("my-dapp")
//!     .with_fee(0.3)  // 0.3% integrator fee
//!     .with_referrer("0xYourReferrerAddress");
//!
//! let client = Client::with_config(config)?;
//! # Ok::<(), lfi::Error>(())
//! ```
//!
//! # Bridge and Exchange Selection
//!
//! You can control which bridges and exchanges are used:
//!
//! ```no_run
//! use lfi::{QuoteRequest, chains};
//!
//! let request = QuoteRequest::new(
//!     chains::ETHEREUM,
//!     chains::ARBITRUM,
//!     "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
//!     "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
//!     "1000000000",
//!     "0xYourAddress",
//! )
//! .with_allowed_bridges(vec!["stargate".to_string(), "hop".to_string()])
//! .with_denied_exchanges(vec!["sushiswap".to_string()]);
//! ```
//!
//! # Error Handling
//!
//! All API methods return `Result<T, lfi::Error>`:
//!
//! ```no_run
//! use lfi::{Client, QuoteRequest, Error, chains};
//! use lfi::error::DomainError;
//!
//! async fn get_quote(client: &Client) -> Result<(), Error> {
//!     let request = QuoteRequest::new(
//!         chains::ETHEREUM,
//!         chains::ARBITRUM,
//!         "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE",
//!         "0xaf88d065e77c8cC2239327C5EDb3A432268e5831",
//!         "1000000000000000000",
//!         "0xYourAddress",
//!     );
//!
//!     match client.get_quote(&request).await {
//!         Ok(quote) => println!("Got quote: {}", quote.estimate.to_amount),
//!         Err(Error::Domain(DomainError::NoRouteFound)) => println!("No route available"),
//!         Err(Error::RateLimited { retry_after }) => {
//!             println!("Rate limited, retry after {:?}", retry_after);
//!         }
//!         Err(e) => println!("Error: {}", e),
//!     }
//!
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod error;
pub mod types;

pub use client::{Client, Config};
pub use error::{Error, Result};
pub use types::{
    // Chain types
    chains,
    // Step types
    Action,
    // Other types
    BridgeOptions,
    Chain,
    ChainId,
    ChainsResponse,
    // Connection types
    Connection,
    ConnectionsRequest,
    ConnectionsResponse,
    Estimate,
    ExchangeOptions,
    FeeCost,
    GasCost,
    GasPrice,
    GasPricesResponse,
    Insurance,
    // Quote types
    Quote,
    QuoteRequest,
    // Routes types
    Route,
    RouteOrder,
    RoutesOptions,
    RoutesRequest,
    RoutesResponse,
    // Status types
    StatusRequest,
    StatusResponse,
    Step,
    StepType,
    // Token types
    Token,
    TokenAmount,
    // Token list types
    TokensRequest,
    TokensResponse,
    // Tool types
    Tool,
    ToolDetails,
    ToolType,
    ToolsResponse,
    TransactionInfo,
    // Transaction types
    TransactionRequest,
    TransactionStatus,
};

/// Default base URL for the LI.FI API
pub const DEFAULT_BASE_URL: &str = "https://li.quest/v1";

// Re-export common utilities
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Create a default LI.FI config
#[must_use]
pub fn default_config() -> Config {
    Config::default()
}

/// Create a config with an integrator identifier
#[must_use]
pub fn config_with_integrator(integrator: impl Into<String>) -> Config {
    Config::default().with_integrator(integrator)
}
