//! Virtual TestNets API
//!
//! Create and manage Virtual TestNets - isolated blockchain environments
//! that fork from mainnet or other networks for development and testing.
//!
//! # Features
//!
//! - Create VNets forked from any EVM network at any block
//! - Infinite faucet for test accounts
//! - State sync with parent network
//! - Fork VNets from other VNets
//! - CI/CD integration with bulk operations
//! - Admin RPC for state manipulation (time warping, balance setting, snapshots)
//!
//! # Example
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::vnets::{CreateVNetRequest, ListVNetsQuery};
//!
//! let client = Client::from_env()?;
//!
//! // Create a new VNet forked from Ethereum mainnet
//! let request = CreateVNetRequest::new("my-dev-env", "Development Environment", 1)
//!     .block_number(18000000)
//!     .sync_state(true);
//!
//! let vnet = client.vnets().create(&request).await?;
//! println!("VNet ID: {}", vnet.id);
//! println!("Public RPC: {:?}", vnet.rpcs.as_ref().and_then(|r| r.public()));
//!
//! // Use Admin RPC to manipulate state
//! let admin = client.vnets().admin_rpc("vnet-id").await?;
//! admin.set_balance("0x1234...", "1000000000000000000").await?;
//! admin.increase_time(3600).await?; // Advance 1 hour
//!
//! // List VNets matching a PR number (useful for CI)
//! let query = ListVNetsQuery::new().slug("pr-123");
//! let vnets = client.vnets().list(Some(query)).await?;
//!
//! // Clean up old VNets
//! let old_ids: Vec<String> = vnets.iter().map(|v| v.id.clone()).collect();
//! client.vnets().delete_many(old_ids).await?;
//! ```

pub mod admin_rpc;
mod api;
mod types;

pub use admin_rpc::{
    AccessListEntry, AccessListResult, AdminRpc, LatestBlock, SendTransactionParams,
};
pub use api::VNetsApi;
pub use types::*;
