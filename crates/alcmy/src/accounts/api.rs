//! Accounts/Signer API implementation

use super::types::{
    AuthRequest, AuthResponse, JwtAuthRequest, JwtAuthResponse, LookupRequest, LookupResponse,
    OtpRequest, OtpResponse, SignPayloadRequest, SignPayloadResponse, StampedRequest,
    WhoamiResponse,
};
use crate::client::Client;
use crate::error::Result;

const SIGNER_BASE_URL: &str = "https://api.g.alchemy.com/signer/v1";

/// Accounts API for smart wallet authentication
pub struct AccountsApi<'a> {
    client: &'a Client,
}

impl<'a> AccountsApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{SIGNER_BASE_URL}{path}");
        let response = self
            .client
            .http()
            .post(&url)
            .bearer_auth(self.client.api_key())
            .json(body)
            .send()
            .await
            .map_err(crate::error::http)?;

        if response.status().is_success() {
            response.json().await.map_err(crate::error::http)
        } else {
            Err(crate::error::handle_error_response(response).await)
        }
    }

    /// Send an authentication email (magic link or OTP)
    ///
    /// The returned `otp_id` and `org_id` feed [`Self::verify_otp`].
    pub async fn auth(&self, request: &AuthRequest) -> Result<AuthResponse> {
        self.post("/auth", request).await
    }

    /// Get the authenticated user's identity from a stamped Turnkey whoami request
    pub async fn whoami(&self, stamped_request: &StampedRequest) -> Result<WhoamiResponse> {
        let body = serde_json::json!({ "stampedRequest": stamped_request });
        self.post("/whoami", &body).await
    }

    /// Look up whether a user exists by email
    pub async fn lookup(&self, email: &str) -> Result<LookupResponse> {
        let request = LookupRequest {
            email: email.to_string(),
        };
        self.post("/lookup", &request).await
    }

    /// Verify an OTP code and get the encrypted credential bundle
    pub async fn verify_otp(&self, request: &OtpRequest) -> Result<OtpResponse> {
        self.post("/otp", request).await
    }

    /// Authenticate an existing user with an OIDC JWT
    pub async fn auth_jwt(&self, request: &JwtAuthRequest) -> Result<JwtAuthResponse> {
        self.post("/auth-jwt", request).await
    }

    /// Sign a payload with a stamped Turnkey sign-raw-payload request
    pub async fn sign_payload(&self, request: &SignPayloadRequest) -> Result<SignPayloadResponse> {
        self.post("/sign-payload", request).await
    }
}
