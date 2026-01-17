//! # yldfi-common
//!
//! Shared utilities for yldfi-rs API clients.
//!
//! ## Modules
//!
//! - [`retry`] - Retry utilities with exponential backoff
//! - [`eth`] - Ethereum address and transaction hash validation
//! - [`chains`] - EVM chain ID and name mappings
//! - [`units`] - Wei/Gwei/Ether conversion utilities
//!
//! ## Retry Utilities
//!
//! ```no_run
//! use yldfi_common::{with_retry, RetryConfig, RetryableError};
//!
//! // Implement RetryableError for your error type
//! struct MyError;
//! impl RetryableError for MyError {
//!     fn is_retryable(&self) -> bool { true }
//! }
//!
//! async fn example() {
//!     let config = RetryConfig::default();
//!     let result = with_retry(&config, || async {
//!         Ok::<_, MyError>("success")
//!     }).await;
//! }
//! ```
//!
//! ## Ethereum Utilities
//!
//! ```
//! use yldfi_common::eth::{is_valid_address, normalize_address, is_valid_tx_hash};
//!
//! assert!(is_valid_address("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"));
//! assert_eq!(
//!     normalize_address("0xABC123..."),
//!     None // Invalid length
//! );
//! ```
//!
//! ## Chain Mappings
//!
//! ```
//! use yldfi_common::chains::Chain;
//!
//! let chain = Chain::from_id(1);
//! assert_eq!(chain, Chain::Ethereum);
//! assert_eq!(chain.name(), "ethereum");
//! assert_eq!(chain.native_currency(), "ETH");
//! ```
//!
//! ## Unit Conversions
//!
//! ```
//! use yldfi_common::units::{to_wei, from_wei, parse_units, format_units};
//!
//! // Convert 1.5 ETH to wei
//! let wei = to_wei("1.5", 18).unwrap();
//! assert_eq!(wei, "1500000000000000000");
//!
//! // Convert wei back to ETH
//! let eth = from_wei("1500000000000000000", 18);
//! assert_eq!(eth, "1.5");
//! ```

pub mod api;
pub mod chains;
pub mod eth;
pub mod http;
pub mod retry;
pub mod units;

pub use retry::{with_retry, with_simple_retry, RetryConfig, RetryError, RetryableError};

// Re-export HTTP utilities
pub use http::{
    build_client, build_client_with_proxy, build_default_client, HttpClientConfig, HttpError,
};

// Re-export commonly used eth utilities at crate root
pub use eth::{
    is_valid_address, is_valid_tx_hash, normalize_address, Address, AddressParseError,
    HttpStatusKind, TxHash, TxHashParseError,
};

// Re-export Chain at crate root for convenience
pub use chains::Chain;

// Re-export API utilities
pub use api::{
    extract_retry_after, handle_error_response, ApiConfig, ApiError, ApiResult, BaseClient,
    ConfigValidationError, NoDomainError, SecretApiKey,
};

// Re-export Wei amount type
pub use units::{Wei, WeiParseError};
