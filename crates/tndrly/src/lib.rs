//! # tndrly
//!
//! Unofficial Rust client for the [Tenderly](https://tenderly.co) API.
//!
//! Provides access to:
//! - **Simulation API** - Simulate transactions without broadcasting
//! - **Virtual TestNets API** - Create isolated blockchain environments
//! - **Alerts API** - Monitor on-chain activity with notifications
//! - **Contract API** - Manage and verify smart contracts
//! - **Web3 Actions API** - Deploy serverless functions
//! - **Wallets API** - Track and monitor wallet addresses
//!
//! ## Quick Start
//!
//! ```ignore
//! use tndrly::{Client, Config};
//! use tndrly::simulation::SimulationRequest;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), tndrly::Error> {
//!     // Create client from environment variables
//!     // Requires: TENDERLY_ACCESS_KEY, TENDERLY_ACCOUNT, TENDERLY_PROJECT
//!     let client = Client::from_env()?;
//!
//!     // Or configure manually
//!     let client = Client::new(Config::new(
//!         "your-access-key",
//!         "your-account",
//!         "your-project"
//!     ))?;
//!
//!     // Simulate a transaction
//!     let request = SimulationRequest::new(
//!         "0x0000000000000000000000000000000000000000",
//!         "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
//!         "0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045"
//!     );
//!
//!     let result = client.simulation().simulate(&request).await?;
//!     println!("Gas used: {}", result.simulation.gas_used);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Environment Variables
//!
//! The client can be configured via environment variables:
//!
//! - `TENDERLY_ACCESS_KEY` - Your Tenderly API access key
//! - `TENDERLY_ACCOUNT` - Your account slug (username or organization)
//! - `TENDERLY_PROJECT` - Your project slug
//!
//! ## API Modules
//!
//! All API modules are always available:
//! - [`simulation`] - Transaction simulation
//! - [`vnets`] - Virtual TestNets
//! - [`alerts`] - Alert monitoring
//! - [`contracts`] - Contract management
//! - [`actions`] - Web3 Actions
//! - [`wallets`] - Wallet monitoring
//! - [`delivery_channels`] - Notification delivery channels
//! - [`networks`] - Supported networks

mod client;
mod error;

pub mod actions;
pub mod alerts;
pub mod contracts;
pub mod delivery_channels;
pub mod networks;
pub mod simulation;
pub mod utils;
pub mod vnets;
pub mod wallets;

pub use client::{Client, Config, API_BASE_URL};
pub use error::{Error, Result};

// Re-export commonly used types at the crate root
pub use actions::{ActionTrigger, CreateActionRequest};
pub use alerts::{AlertType, CreateAlertRequest};
pub use contracts::{AddContractRequest, Contract};
pub use simulation::{SimulationRequest, SimulationResponse, SimulationType};
pub use vnets::{CreateVNetRequest, VNet};
pub use wallets::{AddWalletRequest, AddWalletResponse, WalletOnNetwork};

impl Client {
    /// Access the Simulation API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = client.simulation().simulate(&request).await?;
    /// ```
    pub fn simulation(&self) -> simulation::SimulationApi<'_> {
        simulation::SimulationApi::new(self)
    }

    /// Access the Virtual TestNets API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let vnets = client.vnets().list(None).await?;
    /// ```
    pub fn vnets(&self) -> vnets::VNetsApi<'_> {
        vnets::VNetsApi::new(self)
    }

    /// Access the Alerts API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let alerts = client.alerts().list().await?;
    /// ```
    pub fn alerts(&self) -> alerts::AlertsApi<'_> {
        alerts::AlertsApi::new(self)
    }

    /// Access the Contract API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let contracts = client.contracts().list(None).await?;
    /// ```
    pub fn contracts(&self) -> contracts::ContractsApi<'_> {
        contracts::ContractsApi::new(self)
    }

    /// Access the Web3 Actions API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let actions = client.actions().list().await?;
    /// ```
    pub fn actions(&self) -> actions::ActionsApi<'_> {
        actions::ActionsApi::new(self)
    }

    /// Access the Wallets API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let wallets = client.wallets().list().await?;
    /// ```
    pub fn wallets(&self) -> wallets::WalletsApi<'_> {
        wallets::WalletsApi::new(self)
    }

    /// Access the Delivery Channels API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let channels = client.delivery_channels().list_project().await?;
    /// ```
    pub fn delivery_channels(&self) -> delivery_channels::DeliveryChannelsApi<'_> {
        delivery_channels::DeliveryChannelsApi::new(self)
    }

    /// Access the Networks API
    ///
    /// # Example
    ///
    /// ```ignore
    /// let networks = client.networks().supported().await?;
    /// ```
    pub fn networks(&self) -> networks::NetworksApi<'_> {
        networks::NetworksApi::new(self)
    }
}
