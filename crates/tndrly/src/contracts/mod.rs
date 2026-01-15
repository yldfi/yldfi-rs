//! Contract API
//!
//! Add, manage, and verify smart contracts in your Tenderly project.
//! Organize contracts with tags, set display names, and verify source code.
//!
//! # Example
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::contracts::{AddContractRequest, VerifyContractRequest, ListContractsQuery};
//!
//! let client = Client::from_env()?;
//!
//! // Add a contract
//! let request = AddContractRequest::new("1", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
//!     .display_name("USDC")
//!     .tag("stablecoin");
//!
//! let contract = client.contracts().add(&request).await?;
//!
//! // List contracts by tag
//! let query = ListContractsQuery::new().tag("stablecoin");
//! let contracts = client.contracts().list(Some(query)).await?;
//!
//! // Verify a contract
//! let verify_request = VerifyContractRequest::new(
//!     "1",
//!     "0xMyContract",
//!     "MyContract",
//!     include_str!("MyContract.sol"),
//!     "v0.8.19+commit.7dd6d404",
//! )
//! .optimization(true, 200);
//!
//! let result = client.contracts().verify(&verify_request).await?;
//! if result.success {
//!     println!("Contract verified!");
//! }
//! ```

mod api;
mod types;

pub use api::ContractsApi;
pub use types::*;
