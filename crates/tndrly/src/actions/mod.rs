//! Web3 Actions API
//!
//! Deploy serverless JavaScript/TypeScript functions that execute in response
//! to on-chain events, alerts, webhooks, or schedules.
//!
//! # Trigger Types
//!
//! - `Alert` - Triggered by a Tenderly alert
//! - `Webhook` - Triggered by an HTTP call
//! - `Periodic` - Triggered on a cron schedule
//! - `Block` - Triggered when a block is mined
//! - `Transaction` - Triggered by transactions to an address
//!
//! # Example
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::actions::{CreateActionRequest, ActionTrigger, TriggerConfig};
//!
//! let client = Client::from_env()?;
//!
//! // Create an action triggered by an alert
//! let source = r#"
//! module.exports = async (context, event) => {
//!     const { alertId, transaction } = event;
//!     console.log(`Alert ${alertId} triggered for tx ${transaction.hash}`);
//!
//!     // Send to Discord, Telegram, etc.
//!     await fetch(context.secrets.DISCORD_WEBHOOK, {
//!         method: 'POST',
//!         body: JSON.stringify({ content: `Transaction failed: ${transaction.hash}` })
//!     });
//!
//!     return { notified: true };
//! };
//! "#;
//!
//! let request = CreateActionRequest::new("Notify Discord", ActionTrigger::Alert, source)
//!     .trigger_config(TriggerConfig::alert("alert-123"))
//!     .secret("DISCORD_WEBHOOK", "https://discord.com/api/webhooks/...");
//!
//! let action = client.actions().create(&request).await?;
//!
//! // Pause / resume it
//! client.actions().stop(&action.id).await?;
//! client.actions().resume(&action.id).await?;
//!
//! // Inspect executions
//! let calls = client.actions().calls(&action.id, None).await?;
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
