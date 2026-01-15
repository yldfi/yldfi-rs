//! Transaction simulation API
//!
//! Simulate individual transactions or bundles of transactions without
//! broadcasting them to the network. Get detailed execution traces,
//! gas usage, state changes, and event logs.
//!
//! # Example
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::simulation::{SimulationRequest, SimulationType};
//!
//! let client = Client::from_env()?;
//!
//! // Simulate a simple transfer
//! let request = SimulationRequest::new(
//!     "0xSenderAddress",
//!     "0xRecipientAddress",
//!     "0x" // empty calldata for ETH transfer
//! )
//! .value_wei(1_000_000_000_000_000_000u128) // 1 ETH
//! .gas(21000)
//! .save(true);
//!
//! let result = client.simulation().simulate(&request).await?;
//! println!("Simulation ID: {}", result.simulation.id);
//! println!("Gas used: {}", result.simulation.gas_used);
//! ```

mod api;
mod types;

pub use api::SimulationApi;
pub use types::*;
