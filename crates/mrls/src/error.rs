//! Error types for the Moralis API client
//!
//! This module provides the error types for the Moralis API client,
//! built on top of the shared `ApiError` infrastructure.

use serde::Deserialize;
use thiserror::Error;
pub use yldfi_common::api::{sanitize_error_body, ApiError};

/// Moralis API error response structure
#[derive(Debug, Clone, Deserialize)]
pub struct ApiErrorResponse {
    /// Error message
    pub message: Option<String>,
    /// Error code or name
    #[serde(alias = "name")]
    pub code: Option<String>,
}

/// Required plan tier for an endpoint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum PlanTier {
    /// Free tier
    Free,
    /// Starter plan required
    Starter,
    /// Pro plan required
    Pro,
    /// Business plan required
    Business,
}

impl std::fmt::Display for PlanTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlanTier::Free => write!(f, "Free"),
            PlanTier::Starter => write!(f, "Starter"),
            PlanTier::Pro => write!(f, "Pro"),
            PlanTier::Business => write!(f, "Business"),
        }
    }
}

/// Domain-specific errors for Moralis API
#[derive(Error, Debug)]
pub enum DomainError {
    /// Invalid or missing API key (401)
    #[error("Unauthorized: Invalid or missing API key")]
    Unauthorized,

    /// Endpoint requires a higher plan tier (402/403)
    #[error("Plan upgrade required: This endpoint requires {required_plan} plan or higher")]
    PlanRequired {
        required_plan: PlanTier,
        message: String,
    },

    /// Resource not found (404)
    #[error("Not found: {0}")]
    NotFound(String),

    /// Invalid configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Missing API key
    #[error("Missing API key")]
    MissingApiKey,

    /// Insecure URL scheme (MED-002 fix)
    #[error("Insecure URL scheme: use HTTPS to protect API keys (got: {0})")]
    InsecureScheme(String),
}

/// Error type for Moralis API operations
pub type Error = ApiError<DomainError>;

/// Result type for Moralis API operations
pub type Result<T> = std::result::Result<T, Error>;

// Convenience constructors for domain errors

/// Create an unauthorized error
#[must_use]
pub fn unauthorized() -> Error {
    ApiError::domain(DomainError::Unauthorized)
}

/// Create a plan required error
pub fn plan_required(required_plan: PlanTier, message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::PlanRequired {
        required_plan,
        message: message.into(),
    })
}

/// Create a not found error
pub fn not_found(resource: impl Into<String>) -> Error {
    ApiError::domain(DomainError::NotFound(resource.into()))
}

/// Create a configuration error
pub fn config(message: impl Into<String>) -> Error {
    ApiError::domain(DomainError::Config(message.into()))
}

/// Create a missing API key error
#[must_use]
pub fn missing_api_key() -> Error {
    ApiError::domain(DomainError::MissingApiKey)
}

/// Create an insecure scheme error (MED-002 fix)
pub fn insecure_scheme(scheme: impl Into<String>) -> Error {
    ApiError::domain(DomainError::InsecureScheme(scheme.into()))
}

/// Create from HTTP response status and body
///
/// Handles Moralis-specific error patterns:
/// - 401 -> Unauthorized
/// - 402/403 -> `PlanRequired` (parses plan tier from message)
/// - 404 -> `NotFound`
/// - 429 -> `RateLimited`
/// - 5xx -> `ServerError`
///
/// MRLS-001 fix: All error messages are sanitized to prevent credential leaks
#[must_use]
pub fn from_response(status: u16, body: &str, retry_after: Option<u64>) -> Error {
    // MRLS-001 fix: Sanitize body before using in error messages
    // This prevents API keys/tokens from leaking if server echoes them
    let sanitized_body = sanitize_error_body(body);

    // Try to parse the error response (use original for parsing, sanitized for messages)
    let parsed: Option<ApiErrorResponse> = serde_json::from_str(body).ok();
    let message = parsed
        .as_ref()
        .and_then(|r| r.message.as_ref().map(|m| sanitize_error_body(m)))
        .unwrap_or_else(|| sanitized_body.clone());

    match status {
        401 => unauthorized(),
        402 | 403 => {
            // Determine required plan from message
            let required_plan = if message.to_lowercase().contains("business") {
                PlanTier::Business
            } else if message.to_lowercase().contains("pro") {
                PlanTier::Pro
            } else if message.to_lowercase().contains("starter") {
                PlanTier::Starter
            } else {
                PlanTier::Pro // Default assumption for premium endpoints
            };
            plan_required(required_plan, message)
        }
        404 => not_found(message),
        _ => ApiError::from_response(status, &sanitized_body, retry_after),
    }
}

/// Check if an error is due to plan restrictions
#[must_use]
pub fn is_plan_required(error: &Error) -> bool {
    matches!(error, ApiError::Domain(DomainError::PlanRequired { .. }))
}
