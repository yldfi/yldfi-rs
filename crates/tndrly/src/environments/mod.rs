//! Environments API
//!
//! Multi-chain environments: a set of Virtual `TestNets` (one per network)
//! created together, optionally with cross-chain bridge relaying. Each
//! environment has an active *instance* holding its `VNets`; transactions are
//! listed per instance and sent or simulated on one `VNet` of an instance.
//!
//! These are the `/api/public/v1/account/{account}/project/{project}/environments`
//! routes. Single Virtual `TestNets` remain available through
//! [`vnets`](crate::vnets), whose request and transaction types are reused
//! here.
//!
//! # Example
//!
//! ```ignore
//! use tndrly::Client;
//! use tndrly::environments::{BridgeConfig, CreateEnvironmentRequest, EnvironmentNetworkConfig};
//! use tndrly::vnets::SendVNetTransactionRequest;
//!
//! let client = Client::from_env()?;
//!
//! let request = CreateEnvironmentRequest::new("My Multi-Chain Environment")
//!     .slug("my-multi-chain-env")
//!     .network(EnvironmentNetworkConfig::new("1").block_number("0x12c50f0"))
//!     .network(EnvironmentNetworkConfig::new("8453"))
//!     .bridge_config(BridgeConfig::auto_relay());
//! let env = client.environments().create(&request).await?;
//!
//! let instance = env.active_instance.as_ref().expect("active instance");
//! let vnet = &instance.vnets[0];
//! let tx = SendVNetTransactionRequest::new("0x...from", "0x...to", "0x");
//! client
//!     .environments()
//!     .send_transaction(&env.id, &instance.id, &vnet.id, &tx)
//!     .await?;
//! ```

mod api;
mod types;

pub use api::EnvironmentsApi;
pub use types::*;
