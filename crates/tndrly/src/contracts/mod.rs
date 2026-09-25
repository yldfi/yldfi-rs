//! Contract API
//!
//! Add and manage smart contracts in your Tenderly project.
//! Organize contracts with tags and set display names.
//!
//! Tenderly has no public `PATCH /contract/{network}/{address}` or
//! `POST /contract/verify` endpoint; use [`ContractsApi::rename`],
//! [`ContractsApi::add_tag`] / [`ContractsApi::bulk_tag`] and
//! [`ContractsApi::delete_tag`] instead, and verify contracts with the
//! Tenderly CLI / Foundry / Hardhat plugins.
//!
//! # Example
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::contracts::{AddContractRequest, ListContractsQuery};
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
//! // Tag it
//! client.contracts().add_tag("1", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", "v1").await?;
//! ```

mod api;
mod types;

pub use api::{contract_id, ContractsApi};
pub use types::*;
