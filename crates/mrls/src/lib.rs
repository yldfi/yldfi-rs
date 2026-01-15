//! # mrls - Moralis Web3 API Client
//!
//! A Rust client for the [Moralis Web3 API](https://docs.moralis.io/).
//!
//! ## Features
//!
//! - **Token API** - ERC20 token metadata, prices, transfers, holders
//! - **Wallet API** - Native balances, token balances, transactions, net worth
//!
//! ## Quick Start
//!
//! ```no_run
//! use mrls::Client;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), mrls::Error> {
//!     // Create client from MORALIS_API_KEY env var
//!     let client = Client::from_env()?;
//!
//!     // Get native balance
//!     let balance = client.wallet().get_native_balance(
//!         "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
//!         Some("eth"),
//!     ).await?;
//!     println!("Balance: {} wei", balance.balance);
//!
//!     // Get token price
//!     let price = client.token().get_price(
//!         "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
//!         Some("eth"),
//!     ).await?;
//!     println!("WETH Price: ${:?}", price.usd_price);
//!
//!     Ok(())
//! }
//! ```

mod client;
pub mod error;
pub mod token;
pub mod wallet;

pub use client::Client;
pub use error::Error;
pub use token::TokenApi;
pub use wallet::{WalletApi, WalletQuery};

/// Result type alias for this crate
pub type Result<T> = std::result::Result<T, Error>;
