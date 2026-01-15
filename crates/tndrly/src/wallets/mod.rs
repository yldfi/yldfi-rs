//! Wallet API
//!
//! Add and manage watched wallets in your Tenderly project.
//! Monitor wallet transactions, organize with tags, and set display names.
//!
//! # Example
//!
//! ```ignore
//! use tndrly::Client;
//! use tndrly::wallets::{AddWalletRequest, UpdateWalletRequest};
//!
//! let client = Client::from_env()?;
//!
//! // Add a wallet to monitor
//! let request = AddWalletRequest::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
//!     .display_name("vitalik.eth")
//!     .tag("whale");
//!
//! let wallet = client.wallets().add(&request).await?;
//!
//! // List all wallets
//! let wallets = client.wallets().list().await?;
//! for wallet in wallets {
//!     println!("{}: {:?}", wallet.address, wallet.display_name);
//! }
//!
//! // Update a wallet
//! let update = UpdateWalletRequest::new()
//!     .display_name("Vitalik Buterin")
//!     .tag("founder");
//!
//! client.wallets().update("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", &update).await?;
//!
//! // Remove a wallet
//! client.wallets().remove("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").await?;
//! ```

mod api;
mod types;

pub use api::WalletsApi;
pub use types::*;
