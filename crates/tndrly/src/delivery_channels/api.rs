//! Delivery Channels API operations

use super::types::{DeliveryChannel, ListDeliveryChannelsResponse};
use crate::client::Client;
use crate::error::Result;

/// Delivery Channels API client
pub struct DeliveryChannelsApi<'a> {
    client: &'a Client,
}

impl<'a> DeliveryChannelsApi<'a> {
    /// Create a new Delivery Channels API client
    #[must_use]
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
    /// Fails if either request fails, so a missing half is never mistaken
    /// for "no channels".
    pub async fn list_all(&self) -> Result<Vec<DeliveryChannel>> {
        let mut channels = self.list_account().await?.delivery_channels;
        channels.extend(self.list_project().await?.delivery_channels);
        Ok(channels)
    }
}

#[cfg(test)]
mod tests {
    use crate::client::{Client, Config};
    use wiremock::matchers::{method, path_regex};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_list_all_propagates_partial_failure() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path_regex("/account/acct/delivery-channels$"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({ "delivery_channels": [] })),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path_regex("/project/proj/delivery-channels$"))
            .respond_with(ResponseTemplate::new(503).set_body_string("down"))
            .mount(&server)
            .await;

        let config = Config::new("key", "acct", "proj").with_base_url(server.uri());
        let client = Client::new(config).unwrap();
        let err = client.delivery_channels().list_all().await.unwrap_err();
        assert!(err.is_retryable(), "{err}");
    }
}
