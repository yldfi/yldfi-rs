//! Rust client for the Alchemy API
//!
//! This crate provides a comprehensive type-safe interface to Alchemy's blockchain APIs:
//!
//! ## Core APIs
//! - **NFT API**: NFT ownership, metadata, sales, and spam detection
//! - **Prices API**: Token prices by symbol/address and historical data
//! - **Portfolio API**: Multi-chain wallet balances and NFT holdings
//! - **Token API**: ERC-20 token balances, metadata, and allowances
//! - **Transfers API**: Historical asset transfers via `alchemy_getAssetTransfers`
//!
//! ## Debugging & Tracing
//! - **Debug API**: Transaction and block tracing (debug_* methods)
//! - **Trace API**: Parity-style tracing (trace_* methods)
//! - **Simulation API**: Transaction simulation and asset change prediction
//!
//! ## Account Abstraction (ERC-4337)
//! - **Bundler API**: `UserOperation` submission and gas estimation
//! - **Gas Manager API**: Gas sponsorship policies and paymaster integration
//! - **Wallet API**: Smart wallet operations and session management
//! - **Accounts API**: Smart wallet authentication (email, passkey, JWT)
//!
//! ## Notifications & Webhooks
//! - **Notify API**: Webhook management for address activity, NFTs, and custom GraphQL
//!
//! ## Chain-Specific APIs
//! - **Beacon API**: Ethereum consensus layer (validators, blocks, attestations)
//! - **Solana API**: Digital Asset Standard (DAS) for NFTs and tokens
//!
//! # Example
//!
//! ```no_run
//! use alcmy::{Client, Network};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a client for Ethereum mainnet
//!     let client = Client::new("your-api-key", Network::EthMainnet)?;
//!
//!     // Get NFTs owned by an address
//!     let nfts = client.nft().get_nfts_for_owner("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").await?;
//!     println!("Found {} NFTs", nfts.total_count);
//!
//!     // Get token balances
//!     let balances = client.token().get_token_balances("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045").await?;
//!     println!("Found {} token balances", balances.token_balances.len());
//!
//!     // Trace a transaction
//!     let trace = client.debug().trace_transaction("0x...").await?;
//!     println!("Trace: {:?}", trace);
//!
//!     // Simulate a transaction
//!     let sim = client.simulation().simulate_asset_changes(&Default::default()).await?;
//!     println!("Asset changes: {:?}", sim.changes);
//!
//!     Ok(())
//! }
//! ```

mod client;
mod error;

// Core APIs
pub mod nft;
pub mod portfolio;
pub mod prices;
pub mod token;
pub mod transfers;

// Debugging & Tracing
pub mod debug;
pub mod simulation;
pub mod trace;

// Account Abstraction (ERC-4337)
pub mod accounts;
pub mod bundler;
pub mod gasmanager;
pub mod wallet;

// Notifications
pub mod notify;

// Chain-Specific
pub mod beacon;
pub mod solana;

pub use client::{Client, Config, Network};
pub use error::{Error, Result};
pub use yldfi_common::http::HttpClientConfig;
pub use yldfi_common::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

/// Create a config for a specific network
#[must_use]
pub fn config_for_network(api_key: impl Into<String>, network: Network) -> Config {
    Config::new(api_key, network)
}

/// Create a config for Ethereum mainnet
#[must_use]
pub fn config_eth_mainnet(api_key: impl Into<String>) -> Config {
    Config::new(api_key, Network::EthMainnet)
}

impl Client {
    // ========== Core APIs ==========

    /// Access the NFT API
    #[must_use] 
    pub fn nft(&self) -> nft::NftApi<'_> {
        nft::NftApi::new(self)
    }

    /// Access the Prices API
    #[must_use] 
    pub fn prices(&self) -> prices::PricesApi<'_> {
        prices::PricesApi::new(self)
    }

    /// Access the Portfolio/Data API
    #[must_use] 
    pub fn portfolio(&self) -> portfolio::PortfolioApi<'_> {
        portfolio::PortfolioApi::new(self)
    }

    /// Access the Token API (RPC methods)
    #[must_use] 
    pub fn token(&self) -> token::TokenApi<'_> {
        token::TokenApi::new(self)
    }

    /// Access the Transfers API
    #[must_use] 
    pub fn transfers(&self) -> transfers::TransfersApi<'_> {
        transfers::TransfersApi::new(self)
    }

    // ========== Debugging & Tracing ==========

    /// Access the Debug API (debug_* methods)
    #[must_use] 
    pub fn debug(&self) -> debug::DebugApi<'_> {
        debug::DebugApi::new(self)
    }

    /// Access the Trace API (Parity-style trace_* methods)
    #[must_use] 
    pub fn trace(&self) -> trace::TraceApi<'_> {
        trace::TraceApi::new(self)
    }

    /// Access the Simulation API
    #[must_use] 
    pub fn simulation(&self) -> simulation::SimulationApi<'_> {
        simulation::SimulationApi::new(self)
    }

    // ========== Account Abstraction (ERC-4337) ==========

    /// Access the Bundler API for ERC-4337 operations
    #[must_use] 
    pub fn bundler(&self) -> bundler::BundlerApi<'_> {
        bundler::BundlerApi::new(self)
    }

    /// Access the Gas Manager API for gas sponsorship
    #[must_use] 
    pub fn gas_manager(&self) -> gasmanager::GasManagerApi<'_> {
        gasmanager::GasManagerApi::new(self)
    }

    /// Access the Wallet API for smart wallet operations
    #[must_use] 
    pub fn wallet(&self) -> wallet::WalletApi<'_> {
        wallet::WalletApi::new(self)
    }

    /// Access the Accounts API for smart wallet authentication
    #[must_use] 
    pub fn accounts(&self) -> accounts::AccountsApi<'_> {
        accounts::AccountsApi::new(self)
    }

    // ========== Notifications ==========

    /// Access the Notify API for webhook management
    #[must_use] 
    pub fn notify(&self) -> notify::NotifyApi<'_> {
        notify::NotifyApi::new(self)
    }

    // ========== Chain-Specific ==========

    /// Access the Beacon API (Ethereum consensus layer)
    #[must_use] 
    pub fn beacon(&self) -> beacon::BeaconApi<'_> {
        beacon::BeaconApi::new(self)
    }

    /// Access the Solana DAS API
    #[must_use] 
    pub fn solana(&self) -> solana::SolanaApi<'_> {
        solana::SolanaApi::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_slug() {
        assert_eq!(Network::EthMainnet.slug(), "eth-mainnet");
        assert_eq!(Network::PolygonMainnet.slug(), "polygon-mainnet");
        assert_eq!(Network::ArbitrumMainnet.slug(), "arb-mainnet");
        assert_eq!(Network::BaseMainnet.slug(), "base-mainnet");
    }

    #[test]
    fn test_client_urls() {
        let client = Client::new("test-key", Network::EthMainnet).unwrap();
        assert!(client.rpc_url().contains("eth-mainnet"));
        assert!(client.rpc_url().contains("test-key"));
        assert!(client.nft_url().contains("nft/v3"));
        assert!(client.prices_url().contains("prices/v1"));
        assert!(client.data_url().contains("data/v1"));
    }

    #[test]
    fn test_asset_transfers_options() {
        let opts = transfers::AssetTransfersOptions::from_address("0x123")
            .with_metadata()
            .exclude_zero_value()
            .with_max_count(100);

        assert_eq!(opts.from_address, Some("0x123".to_string()));
        assert_eq!(opts.with_metadata, Some(true));
        assert_eq!(opts.exclude_zero_value, Some(true));
        assert_eq!(opts.max_count, Some("0x64".to_string()));
    }

    #[test]
    fn test_all_apis_accessible() {
        let client = Client::new("test-key", Network::EthMainnet).unwrap();
        // Just verify all APIs can be accessed
        let _ = client.nft();
        let _ = client.prices();
        let _ = client.portfolio();
        let _ = client.token();
        let _ = client.transfers();
        let _ = client.debug();
        let _ = client.trace();
        let _ = client.simulation();
        let _ = client.bundler();
        let _ = client.gas_manager();
        let _ = client.wallet();
        let _ = client.accounts();
        let _ = client.notify();
        let _ = client.beacon();
        let _ = client.solana();
    }
}
