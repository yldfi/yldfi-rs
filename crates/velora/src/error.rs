//! Error types for the Velora (ParaSwap) API client
//!
//! This module re-exports the shared `ApiError` type from `yldfi_common`.

pub use yldfi_common::api::{ApiError, ApiResult};

/// Error type for Velora API operations
pub type Error = ApiError;

/// Result type for Velora API operations
pub type Result<T> = ApiResult<T>;
