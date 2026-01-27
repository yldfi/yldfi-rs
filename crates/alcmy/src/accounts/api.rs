//! Accounts/Signer API implementation

use super::types::{
    AuthRequest, JwtAuthRequest, LookupRequest, LookupResponse, OtpRequest, SignPayloadRequest,
    SignPayloadResponse, SignupRequest, SignupResponse, WhoamiResponse,
};
use crate::client::Client;
use crate::error::{Error, Result};

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
            .await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Sign up a new user (email or passkey)
    pub async fn signup(&self, request: &SignupRequest) -> Result<SignupResponse> {
        self.post("/signup", request).await
    }

    /// Send authentication email with magic link
    pub async fn auth(&self, request: &AuthRequest) -> Result<()> {
        let _: serde_json::Value = self.post("/auth", request).await?;
        Ok(())
    }

    /// Get current user info after authentication
    pub async fn whoami(&self, credential: &str) -> Result<WhoamiResponse> {
        let body = serde_json::json!({ "credential": credential });
        self.post("/whoami", &body).await
    }

    /// Look up if a user exists by email
    pub async fn lookup(&self, email: &str) -> Result<LookupResponse> {
        let request = LookupRequest {
            email: email.to_string(),
        };
        self.post("/lookup", &request).await
    }

    /// Verify OTP code
    pub async fn verify_otp(&self, request: &OtpRequest) -> Result<WhoamiResponse> {
        self.post("/otp", request).await
    }

    /// Authenticate with JWT
    pub async fn auth_jwt(&self, request: &JwtAuthRequest) -> Result<WhoamiResponse> {
        self.post("/auth-jwt", request).await
    }

    /// Sign a payload
    pub async fn sign_payload(&self, request: &SignPayloadRequest) -> Result<SignPayloadResponse> {
        self.post("/sign-payload", request).await
    }
}
