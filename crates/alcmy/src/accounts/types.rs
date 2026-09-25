//! Types for the Accounts/Signer API

use serde::{Deserialize, Serialize};

/// Authentication type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Email,
    Passkey,
}

/// Auth request (email magic link)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    /// Email address
    pub email: String,
    /// Redirect URL after auth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_url: Option<String>,
    /// Target public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_public_key: Option<String>,
}

/// Whoami response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiResponse {
    /// Org ID
    pub org_id: String,
    /// User ID
    pub user_id: String,
    /// Address
    pub address: String,
    /// Email
    pub email: Option<String>,
}

/// Lookup request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupRequest {
    /// Email to look up
    pub email: String,
}

/// Lookup response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LookupResponse {
    /// Whether the user exists
    pub exists: bool,
    /// Org ID (if exists)
    pub org_id: Option<String>,
}

/// OTP verification request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpRequest {
    /// Email address
    pub email: String,
    /// OTP code
    pub otp: String,
    /// Target public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_public_key: Option<String>,
}

/// JWT auth request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtAuthRequest {
    /// JWT token
    pub jwt: String,
    /// Target public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_public_key: Option<String>,
}

/// Sign payload request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignPayloadRequest {
    /// Payload to sign (hex)
    pub payload: String,
    /// Stamp token
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stamp_token: Option<String>,
}

/// Sign payload response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignPayloadResponse {
    /// Signature (hex)
    pub signature: String,
}
