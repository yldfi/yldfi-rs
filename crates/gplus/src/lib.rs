//! # gplus
//!
//! Unofficial Rust client for the GoPlus Security API.
//!
//! GoPlus provides real-time token security analysis including honeypot detection,
//! tax analysis, ownership checks, and more.
//!
//! ## Quick Start
//!
//! ```no_run
//! # async fn example() -> gplus::error::Result<()> {
//! let client = gplus::Client::new()?;
//!
//! // Check token security (USDT on Ethereum)
//! let security = client.token_security(1, "0xdac17f958d2ee523a2206206994597c13d831ec7").await?;
//!
//! println!("Token: {}", security.token_symbol.as_deref().unwrap_or("Unknown"));
//! println!("Is honeypot: {}", security.is_honeypot());
//! println!("Is verified: {}", security.is_verified());
//! # Ok(())
//! # }
//! ```
//!
//! ## Authenticated Access
//!
//! For batch queries and higher rate limits, use authentication:
//!
//! ```no_run
//! # async fn example() -> gplus::error::Result<()> {
//! // From environment variables (GOPLUS_APP_KEY, GOPLUS_APP_SECRET)
//! let client = gplus::Client::from_env()?;
//!
//! // Or explicitly
//! let client = gplus::Client::with_credentials("your_app_key", "your_app_secret")?;
//!
//! // Batch queries (up to 100 tokens)
//! let addresses = &["0xdac17f958d2ee523a2206206994597c13d831ec7", "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"];
//! let results = client.token_security_batch(1, addresses).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Available APIs
//!
//! - **Token Security** - Honeypot detection, tax analysis, ownership
//! - **Address Security** - Malicious address detection
//! - **NFT Security** - NFT collection verification
//! - **Approval Security** - ERC20/721/1155 approval risks
//!
//! ## Supported Chains
//!
//! Ethereum (1), BSC (56), Polygon (137), Arbitrum (42161), Base (8453),
//! Avalanche (43114), Optimism (10), Fantom (250), and more.
//!
//! ## API Reference
//!
//! See <https://docs.gopluslabs.io/reference/api-overview> for full API documentation.

pub mod client;
pub mod error;
pub mod types;

pub use client::{Client, Config, Credentials, RateLimitInfo, BASE_URL};
pub use error::{Error, Result};
pub use types::{
    AddressSecurity, ApprovalSecurity, Chain, NftSecurity, TokenSecurity, TokenSecurityResponse,
};

/// Create a new GoPlus client without authentication (limited access)
pub fn new_client() -> Result<Client> {
    Client::new()
}

/// Create a new GoPlus client from environment variables
/// Uses `GOPLUS_APP_KEY` and `GOPLUS_APP_SECRET`
pub fn client_from_env() -> Result<Client> {
    Client::from_env()
}

/// Check if a chain ID is supported by GoPlus
pub fn is_chain_supported(chain_id: u64) -> bool {
    Chain::is_supported(chain_id)
}
