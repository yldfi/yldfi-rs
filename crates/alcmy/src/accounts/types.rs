//! Types for the Accounts/Signer API
//!
//! Alchemy is sunsetting the Signer APIs: new user signup and wallet
//! pregeneration stopped on 2026-06-16; existing users can still log in.

use serde::{Deserialize, Serialize};

/// Authentication type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    Email,
    Passkey,
}

/// Auth request (`/auth`: sends a magic link / OTP email)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthRequest {
    /// Email address
    pub email: String,
    /// Client HPKE public key the credential bundle is encrypted to
    pub target_public_key: String,
    /// Session lifetime in seconds (default 900)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_seconds: Option<String>,
    /// Extra parameters appended to the magic-link redirect
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_params: Option<String>,
}

/// Auth response; `otp_id` is needed for [`OtpRequest`]
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthResponse {
    /// Org ID
    pub org_id: String,
    /// OTP ID (present for OTP-based email auth)
    #[serde(default)]
    pub otp_id: Option<String>,
}

/// Stamp over a Turnkey request body
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnkeyStamp {
    /// `X-Stamp-Webauthn` for WebAuthn stamps, otherwise `X-Stamp`
    pub stamp_header_name: String,
    /// The stamp value
    pub stamp_header_value: String,
}

/// A stamped Turnkey request (used by `/whoami` and `/sign-payload`)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StampedRequest {
    /// Turnkey URL the request targets
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    /// JSON-encoded request body that was stamped
    pub body: String,
    /// Stamp over `body`
    pub stamp: TurnkeyStamp,
}

/// Whoami response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhoamiResponse {
    /// Org ID
    pub org_id: String,
    /// User ID
    pub user_id: String,
    /// EVM address
    pub address: String,
    /// Solana address
    #[serde(default)]
    pub solana_address: Option<String>,
    /// Email
    #[serde(default)]
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
    /// Org ID of the user, `None` when no account exists
    #[serde(default)]
    pub org_id: Option<String>,
}

impl LookupResponse {
    /// Whether an account exists for the email
    pub fn exists(&self) -> bool {
        self.org_id.is_some()
    }
}

/// OTP verification request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpRequest {
    /// Org ID (from [`AuthResponse`])
    pub org_id: String,
    /// OTP ID (from [`AuthResponse`])
    pub otp_id: String,
    /// OTP code from the email
    pub otp_code: String,
    /// Client HPKE public key
    pub target_public_key: String,
    /// Session lifetime in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_seconds: Option<String>,
}

/// OTP verification response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OtpResponse {
    /// Encrypted credential bundle
    pub credential_bundle: String,
}

/// JWT auth request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtAuthRequest {
    /// OIDC ID token
    pub jwt: String,
    /// Client HPKE public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_public_key: Option<String>,
    /// Session lifetime in seconds
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_seconds: Option<String>,
}

/// JWT auth response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JwtAuthResponse {
    /// Whether the call created a new user
    #[serde(default, alias = "isSignUp")]
    pub is_signup: bool,
    /// Org ID
    pub org_id: String,
    /// Encrypted credential bundle
    #[serde(default)]
    pub credential_bundle: Option<String>,
    /// User ID
    #[serde(default)]
    pub user_id: Option<String>,
    /// EVM address
    #[serde(default)]
    pub address: Option<String>,
    /// Solana address
    #[serde(default)]
    pub solana_address: Option<String>,
}

/// Sign payload request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignPayloadRequest {
    /// Stamped Turnkey `ACTIVITY_TYPE_SIGN_RAW_PAYLOAD_V2` request
    pub stamped_request: StampedRequest,
}

/// Sign payload response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignPayloadResponse {
    /// Signature (hex)
    pub signature: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_and_otp_requests_match_spec() {
        let auth = AuthRequest {
            email: "user@example.com".into(),
            target_public_key: "04ab".into(),
            expiration_seconds: Some("900".into()),
            redirect_params: None,
        };
        assert_eq!(
            serde_json::to_value(&auth).unwrap(),
            serde_json::json!({"email": "user@example.com", "targetPublicKey": "04ab", "expirationSeconds": "900"})
        );
        let resp: AuthResponse =
            serde_json::from_str(r#"{"orgId":"org-1","otpId":"otp-1"}"#).unwrap();
        assert_eq!(resp.otp_id.as_deref(), Some("otp-1"));

        let otp = OtpRequest {
            org_id: resp.org_id,
            otp_id: resp.otp_id.unwrap(),
            otp_code: "123456".into(),
            target_public_key: "04ab".into(),
            expiration_seconds: None,
        };
        assert_eq!(
            serde_json::to_value(&otp).unwrap(),
            serde_json::json!({"orgId": "org-1", "otpId": "otp-1", "otpCode": "123456", "targetPublicKey": "04ab"})
        );
        let bundle: OtpResponse = serde_json::from_str(r#"{"credentialBundle":"bundle"}"#).unwrap();
        assert_eq!(bundle.credential_bundle, "bundle");
    }

    #[test]
    fn lookup_response_without_account_parses() {
        let none: LookupResponse = serde_json::from_str(r#"{"orgId":null}"#).unwrap();
        assert!(!none.exists());
        let some: LookupResponse = serde_json::from_str(r#"{"orgId":"org-1"}"#).unwrap();
        assert!(some.exists());
    }

    #[test]
    fn stamped_requests_and_jwt_response() {
        let req = SignPayloadRequest {
            stamped_request: StampedRequest {
                url: None,
                body: "{}".into(),
                stamp: TurnkeyStamp {
                    stamp_header_name: "X-Stamp".into(),
                    stamp_header_value: "stamp".into(),
                },
            },
        };
        assert_eq!(
            serde_json::to_value(&req).unwrap(),
            serde_json::json!({"stampedRequest": {"body": "{}", "stamp": {"stampHeaderName": "X-Stamp", "stampHeaderValue": "stamp"}}})
        );
        let jwt: JwtAuthResponse =
            serde_json::from_str(r#"{"isSignUp":false,"orgId":"org-1","userId":"u"}"#).unwrap();
        assert!(!jwt.is_signup);
        let whoami: WhoamiResponse = serde_json::from_str(
            r#"{"orgId":"org-1","userId":"u","address":"0x0000000000000000000000000000000000000003","solanaAddress":"Sol111"}"#,
        )
        .unwrap();
        assert_eq!(whoami.solana_address.as_deref(), Some("Sol111"));
    }
}
