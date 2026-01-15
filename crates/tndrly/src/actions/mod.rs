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
//! use tndrly::actions::{CreateActionRequest, ActionTrigger, TriggerConfig, InvokeActionRequest};
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
//! // Manually invoke for testing
//! let result = client.actions()
//!     .invoke(&action.id, &InvokeActionRequest::with_payload(serde_json::json!({
//!         "test": true
//!     })))
//!     .await?;
//!
//! // Check execution logs
//! let logs = client.actions().logs(&action.id).await?;
//! for log in logs.logs {
//!     println!("{:?}: {:?}", log.status, log.output);
//! }
//! ```

mod api;
mod types;

pub use api::ActionsApi;
pub use types::*;
