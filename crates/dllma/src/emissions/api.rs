//! Emissions/Unlocks API endpoints (Pro)

use crate::client::Client;
use crate::error::Result;

use super::types::{EmissionDetail, EmissionsSummary};

/// Emissions API client (Pro only)
pub struct EmissionsApi<'a> {
    client: &'a Client,
}

impl<'a> EmissionsApi<'a> {
    /// Create a new Emissions API client
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get all tokens with unlock schedules
    ///
    /// **Requires Pro API key**
    pub async fn list(&self) -> Result<Vec<EmissionsSummary>> {
        self.client.get_pro("/emissions").await
    }

    /// Get detailed vesting schedule for a protocol
    ///
    /// **Requires Pro API key**
    ///
    /// # Arguments
    ///
    /// * `protocol` - Protocol slug
    pub async fn get(&self, protocol: &str) -> Result<EmissionDetail> {
        let path = format!("/emission/{protocol}");
        self.client.get_pro(&path).await
    }
}
