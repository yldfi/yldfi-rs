//! Error types for the prop AMM data client
//!
//! Built on the shared `ApiError` infrastructure.

use thiserror::Error;
pub use yldfi_common::api::ApiError;

/// Domain-specific errors for prop AMM data
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DomainError {
    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParam(String),

    /// No prop AMM quotes the requested pair
    #[error("No prop AMM quotes this pair")]
    UnknownPair,

    /// The data endpoint answered with a JSON-RPC error
    #[error("JSON-RPC error {code}: {message}")]
    Rpc {
        /// JSON-RPC error code
        code: i64,
        /// Message from the endpoint
        message: String,
    },

    /// The data endpoint answered with neither a result nor an error
    #[error("JSON-RPC response had neither a result nor an error")]
    EmptyResponse,
}

/// Error type for prop AMM data operations
pub type Error = ApiError<DomainError>;

/// Result type for prop AMM data operations
pub type Result<T> = std::result::Result<T, Error>;

/// Create an invalid parameter error
pub fn invalid_param(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InvalidParam(message.into()))
}

/// Map a JSON-RPC error object to an [`Error`]
pub(crate) fn rpc_error(code: i64, message: String) -> Error {
    if message.eq_ignore_ascii_case("unknown pair") {
        ApiError::domain(DomainError::UnknownPair)
    } else {
        ApiError::domain(DomainError::Rpc { code, message })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_pair_is_its_own_error() {
        let err = rpc_error(-32000, "unknown pair".to_string());
        assert!(matches!(err, ApiError::Domain(DomainError::UnknownPair)));
        assert!(err.to_string().contains("No prop AMM quotes this pair"));
    }

    #[test]
    fn test_other_rpc_errors_keep_code_and_message() {
        let err = rpc_error(-32601, "Method not found".to_string());
        let text = err.to_string();
        assert!(text.contains("-32601"), "{text}");
        assert!(text.contains("Method not found"), "{text}");
    }

    #[test]
    fn test_invalid_param_error() {
        let text = invalid_param("amount is not a number").to_string();
        assert!(text.contains("Invalid parameter"));
        assert!(text.contains("amount is not a number"));
    }
}
