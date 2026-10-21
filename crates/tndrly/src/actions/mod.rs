//! Web3 Actions API
//!
//! Deploy serverless JavaScript/TypeScript functions that execute in response
//! to on-chain events, alerts, webhooks, or schedules.
//!
//! # Trigger Types
//!
//! - `Alert` - Triggered by a Tenderly alert
//! - `Webhook` - Triggered by an HTTP call
//! - `Periodic` - Triggered on a cron schedule / interval
//! - `Block` - Triggered when a block is mined
//! - `Transaction` / `TransactionSimple` - Triggered by matching transactions
//!
//! # Example
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::actions::{CreateActionRequest, ActionTrigger, TriggerConfig};
//!
//! let client = Client::from_env()?;
//!
//! let source = r#"
//! const actionFn = async (context, blockEvent) => {
//!     console.log(blockEvent);
//! };
//! module.exports = { actionFn };
//! "#;
//!
//! // Publish and deploy an action that runs every 10th block on mainnet
//! let request = CreateActionRequest::new("block-action", ActionTrigger::Block, source)
//!     .trigger_config(TriggerConfig::block(vec!["1".into()], 10));
//! let published = client.actions().create(&request).await?;
//! let action_id = published.actions["block-action"].action_id.clone().unwrap();
//!
//! // Pause / resume it
//! client.actions().stop(&action_id).await?;
//! client.actions().resume(&action_id).await?;
//!
//! // Inspect executions
//! let calls = client.actions().calls(&action_id, None).await?;
//! ```
//!
//! Tenderly's public API has no `PATCH /actions/action/{id}` (update/enable/
//! disable), `/invoke`, `/logs` or `/source` endpoints. Use
//! [`ActionsApi::stop`] / [`ActionsApi::resume`] instead of disable/enable,
//! [`ActionsApi::calls`] / [`ActionsApi::get_call`] instead of logs, and
//! re-publish with [`ActionsApi::create`] to change code.

mod api;
mod types;

pub use api::ActionsApi;
pub use types::*;
