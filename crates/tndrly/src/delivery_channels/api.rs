//! Delivery Channels API operations

use super::types::*;
use crate::client::Client;
use crate::error::Result;

/// Delivery Channels API client
pub struct DeliveryChannelsApi<'a> {
    client: &'a Client,
}

impl<'a> DeliveryChannelsApi<'a> {
    /// Create a new Delivery Channels API client
    pub fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all delivery channels for the account
    ///
    /// Returns delivery channels configured at the account level.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let channels = client.delivery_channels().list_account().await?;
    /// for channel in channels {
    ///     println!("{}: {:?}", channel.id, channel.channel_type);
    /// }
    /// ```
    pub async fn list_account(&self) -> Result<ListDeliveryChannelsResponse> {
        self.client.get_account("/delivery-channels").await
    }

    /// List delivery channels for the current project
    ///
    /// Returns delivery channels configured at the project level.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let channels = client.delivery_channels().list_project().await?;
    /// for channel in channels {
    ///     println!("{}: {:?}", channel.id, channel.channel_type);
    /// }
    /// ```
    pub async fn list_project(&self) -> Result<ListDeliveryChannelsResponse> {
        self.client.get("/delivery-channels").await
    }

    /// List all delivery channels (both account and project level)
    ///
    /// Convenience method that combines account and project channels.
    /// Returns an error if both requests fail; returns partial results if one succeeds.
    pub async fn list_all(&self) -> Result<Vec<DeliveryChannel>> {
        let account_result = self.list_account().await;
        let project_result = self.list_project().await;

        match (account_result, project_result) {
            (Ok(account), Ok(project)) => {
                let mut channels = account.delivery_channels;
                channels.extend(project.delivery_channels);
                Ok(channels)
            }
            (Ok(account), Err(_)) => Ok(account.delivery_channels),
            (Err(_), Ok(project)) => Ok(project.delivery_channels),
            (Err(e), Err(_)) => Err(e), // Return the first error
        }
    }
}
