//! Alerts API
//!
//! Monitor on-chain activity with customizable alerts. Get notifications
//! when specific events occur on your contracts or addresses.
//!
//! # Alert Types
//!
//! - `SuccessfulTransaction` - Transaction succeeded
//! - `FailedTransaction` - Transaction failed
//! - `FunctionCall` - Specific function was called
//! - `EventEmitted` - Specific event was emitted
//! - `Erc20Transfer` - ERC20 token transfer
//! - `Erc721Transfer` - NFT transfer
//! - `StateChange` - Contract state variable changed
//! - `BalanceChange` - ETH balance changed
//! - `WhaleAlert` - Large value transfer
//!
//! # Destinations
//!
//! - Email
//! - Slack
//! - Discord
//! - Telegram
//! - PagerDuty
//! - Webhooks
//! - Web3 Actions
//!
//! # Example
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::alerts::{
//!     CreateAlertRequest, AlertType, AlertTarget,
//!     CreateWebhookRequest, AddDestinationRequest
//! };
//!
//! let client = Client::from_env()?;
//!
//! // Create a webhook
//! let webhook = client.alerts()
//!     .create_webhook(CreateWebhookRequest::new(
//!         "My Webhook",
//!         "https://myapp.com/tenderly-webhook"
//!     ))
//!     .await?;
//!
//! // Create an alert for failed transactions
//! let alert = client.alerts()
//!     .create(&CreateAlertRequest::new(
//!         "Failed Txs",
//!         AlertType::FailedTransaction,
//!         "1",
//!         AlertTarget::Address,
//!     ).address("0xMyContract"))
//!     .await?;
//!
//! // Connect the webhook to the alert
//! client.alerts()
//!     .add_destination(&alert.id, &AddDestinationRequest::webhook(&webhook.id))
//!     .await?;
//! ```

mod api;
mod types;

pub use api::AlertsApi;
pub use types::*;
