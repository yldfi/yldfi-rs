//! Error types for the 1inch API client
//!
//! This module provides the error types for the 1inch API client,
//! built on top of the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for 1inch API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// Unsupported chain
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    /// No route found for the swap
    #[error("No route found for swap")]
    NoRouteFound,

    /// Insufficient liquidity
    #[error("Insufficient liquidity for swap")]
    InsufficientLiquidity,

    /// Token not found
    #[error("Token not found: {0}")]
    TokenNotFound(String),

    /// Missing API key
    #[error("API key is required for 1inch API")]
    MissingApiKey,

    /// HTTP 402: 1inch answers requests without a valid API key with an x402
    /// pay-per-call challenge (since 2026-08-14) instead of 401
    #[error(
        "1inch returned 402 Payment Required: the request has no valid API key, so 1inch \
         asked for an x402 pay-per-call payment. Set an API key from \
         https://business.1inch.com (ONEINCH_API_KEY){}",
        if .0.is_empty() { String::new() } else { format!(" ({})", .0) }
    )]
    PaymentRequired(String),
}

/// Error type for 1inch API operations
pub type Error = ApiError<DomainError>;

/// Result type for 1inch API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors
/// Create an invalid parameter error
pub fn invalid_param(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidParam(message.into()))
}

/// Create an unsupported chain error
pub fn unsupported_chain(chain: impl Into<String>) -> Error {
    ApiError::domain(DomainError::UnsupportedChain(chain.into()))
}

/// Create a no route found error
#[must_use]
pub fn no_route_found() -> Error {
    ApiError::domain(DomainError::NoRouteFound)
}

/// Create an insufficient liquidity error
#[must_use]
pub fn insufficient_liquidity() -> Error {
    ApiError::domain(DomainError::InsufficientLiquidity)
}

/// Create a token not found error
pub fn token_not_found(token: impl Into<String>) -> Error {
    ApiError::domain(DomainError::TokenNotFound(token.into()))
}

/// Create a missing API key error
#[must_use]
pub fn missing_api_key() -> Error {
    ApiError::domain(DomainError::MissingApiKey)
}

/// Create from HTTP response status and body
///
/// This function includes special handling for 1inch-specific error messages
/// that indicate insufficient liquidity or no route found.
#[must_use]
pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    // Check for 1inch-specific error patterns
    if status == 400 {
        if body.contains("insufficient liquidity") {
            return insufficient_liquidity();
        }
        if body.contains("cannot find route") || body.contains("No route found") {
            return no_route_found();
        }
    }

    if status == 402 {
        return ApiError::domain(DomainError::PaymentRequired(body.trim().to_string()));
    }

    // Fall back to generic ApiError handling
    ApiError::from_response(status, body, retry_after)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payment_required_is_explained() {
        let err = from_response(402, "null", None);
        assert!(matches!(
            err,
            ApiError::Domain(DomainError::PaymentRequired(_))
        ));
        let msg = err.to_string();
        assert!(msg.contains("402") && msg.contains("API key"), "{msg}");
    }
}
