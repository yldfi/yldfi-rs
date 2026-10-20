//! Alerts API
//!
//! Monitor on-chain activity with customizable alerts. Get notifications
//! when specific events occur on your contracts or addresses.
//!
//! # Model
//!
//! An alert has a `simple_type` ([`AlertType`], e.g. `failed_tx`,
//! `log_emitted`, `function_called`), a list of [`AlertExpression`]s
//! (conditions such as `contract_address`, `network`, `tx_status`) and the
//! delivery channels to notify (see [`crate::delivery_channels`]).
//!
//! # Example
//!
//! ```ignore
//! use tndrly::Client;
//! use tndrly::alerts::{AlertExpression, AlertType, CreateAlertRequest};
//!
//! let client = Client::from_env()?;
//!
//! // Create an alert for failed transactions to a contract
//! let alert = client.alerts()
//!     .create(&CreateAlertRequest::new("Failed Txs", AlertType::FailedTx)
//!         .address("0xMyContract")
//!         .network("1")
//!         .expression(AlertExpression::tx_status(false))
//!         .delivery_channel("channel-id"))
//!     .await?;
//! ```
//!
//! Tenderly has no public API to attach/detach alert destinations
//! (`/alert/{id}/destinations`) or to fire a test webhook
//! (`/webhooks/{id}/test`); configure destinations in the dashboard and use
//! [`AlertsApi::test_alert`] to test an alert.

mod api;
mod types;

pub use api::AlertsApi;
pub use types::*;
