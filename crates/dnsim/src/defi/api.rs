//! `DeFi` positions API endpoints

use super::types::{DefiPositionsOptions, DefiPositionsResponse};
use crate::client::Client;
use crate::error::Result;

/// Error message returned for the removed `DeFi` Positions endpoints
pub const DEFI_POSITIONS_SUNSET_MESSAGE: &str = "Dune Sim DeFi Positions was deprecated 2026-06-01 and the Sim platform shuts down 2026-08-01. See https://github.com/yldfi/yldfi-rs/issues/64";

/// `DeFi` API
pub struct DefiApi<'a> {
    client: &'a Client,
}

impl<'a> DefiApi<'a> {
    #[must_use]
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get `DeFi` positions for a wallet (Beta)
    ///
    /// Note: Dune deprecated this endpoint on 2026-06-01; the call is
    /// short-circuited and always returns an error.
    ///
    /// # Arguments
    /// * `address` - Wallet address
    ///
    /// # Errors
    /// Always returns [`DEFI_POSITIONS_SUNSET_MESSAGE`]
    #[deprecated(note = "Dune Sim DeFi Positions was deprecated 2026-06-01 (yldfi-rs issue #64)")]
    pub async fn positions(&self, address: &str) -> Result<DefiPositionsResponse> {
        // Endpoint removed upstream; short-circuit instead of issuing the request.
        let _ = (self.client, address);
        Err(crate::error::deprecated(DEFI_POSITIONS_SUNSET_MESSAGE))
    }

    /// Get `DeFi` positions with options (Beta)
    ///
    /// Note: Dune deprecated this endpoint on 2026-06-01; the call is
    /// short-circuited and always returns an error.
    ///
    /// # Arguments
    /// * `address` - Wallet address
    /// * `options` - Query options
    ///
    /// # Errors
    /// Always returns [`DEFI_POSITIONS_SUNSET_MESSAGE`]
    #[deprecated(note = "Dune Sim DeFi Positions was deprecated 2026-06-01 (yldfi-rs issue #64)")]
    pub async fn positions_with_options(
        &self,
        address: &str,
        options: &DefiPositionsOptions,
    ) -> Result<DefiPositionsResponse> {
        // Endpoint removed upstream; short-circuit instead of issuing the request.
        let _ = (self.client, address, options);
        Err(crate::error::deprecated(DEFI_POSITIONS_SUNSET_MESSAGE))
    }
}
