//! Transfers API implementation

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Transfers API for historical transaction data
pub struct TransfersApi<'a> {
    client: &'a Client,
}

impl<'a> TransfersApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// Get asset transfers with custom options
    ///
    /// # Arguments
    /// * `options` - Transfer query options
    ///
    /// # Example
    /// ```ignore
    /// let options = AssetTransfersOptions::from_address("0x123...")
    ///     .with_metadata()
    ///     .exclude_zero_value();
    /// let transfers = client.transfers().get_asset_transfers(&options).await?;
    /// ```
    pub async fn get_asset_transfers(
        &self,
        options: &AssetTransfersOptions,
    ) -> Result<AssetTransfersResponse> {
        self.client
            .rpc("alchemy_getAssetTransfers", vec![options])
            .await
    }

    /// Get transfers sent from an address
    ///
    /// # Arguments
    /// * `address` - Sender address
    pub async fn get_transfers_from(&self, address: &str) -> Result<AssetTransfersResponse> {
        let options = AssetTransfersOptions::from_address(address).with_metadata();
        self.get_asset_transfers(&options).await
    }

    /// Get transfers sent to an address
    ///
    /// # Arguments
    /// * `address` - Receiver address
    pub async fn get_transfers_to(&self, address: &str) -> Result<AssetTransfersResponse> {
        let options = AssetTransfersOptions::to_address(address).with_metadata();
        self.get_asset_transfers(&options).await
    }

    /// Get all transfers for an address (both sent and received)
    ///
    /// This makes two API calls - one for sent, one for received.
    ///
    /// # Arguments
    /// * `address` - Wallet address
    pub async fn get_all_transfers(
        &self,
        address: &str,
    ) -> Result<(AssetTransfersResponse, AssetTransfersResponse)> {
        let from = self.get_transfers_from(address).await?;
        let to = self.get_transfers_to(address).await?;
        Ok((from, to))
    }

    /// Get next page of transfers
    ///
    /// # Arguments
    /// * `options` - Original options used for the first query
    /// * `page_key` - Page key from previous response
    pub async fn get_next_page(
        &self,
        options: &AssetTransfersOptions,
        page_key: &str,
    ) -> Result<AssetTransfersResponse> {
        let mut new_options = options.clone();
        new_options.page_key = Some(page_key.to_string());
        self.get_asset_transfers(&new_options).await
    }
}
