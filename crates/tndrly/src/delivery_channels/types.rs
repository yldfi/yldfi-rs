//! Types for Delivery Channels API

use serde::{Deserialize, Serialize};

/// Delivery channel type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum DeliveryChannelType {
    /// Email notification
    Email,
    /// Slack webhook
    Slack,
    /// Discord webhook
    Discord,
    /// Telegram bot
    Telegram,
    /// `PagerDuty` integration
    PagerDuty,
    /// Custom webhook
    Webhook,
    /// Sentry integration
    Sentry,
    /// Datadog integration
    Datadog,
    /// Unknown type (for forward compatibility)
    #[serde(other)]
    Unknown,
}

impl DeliveryChannelType {
    /// Get the string representation
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Slack => "slack",
            Self::Discord => "discord",
            Self::Telegram => "telegram",
            Self::PagerDuty => "pager_duty",
            Self::Webhook => "webhook",
            Self::Sentry => "sentry",
            Self::Datadog => "datadog",
            Self::Unknown => "unknown",
        }
    }
}

impl std::fmt::Display for DeliveryChannelType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for DeliveryChannelType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "email" => Ok(Self::Email),
            "slack" => Ok(Self::Slack),
            "discord" => Ok(Self::Discord),
            "telegram" => Ok(Self::Telegram),
            "pagerduty" | "pager_duty" => Ok(Self::PagerDuty),
            "webhook" => Ok(Self::Webhook),
            "sentry" => Ok(Self::Sentry),
            "datadog" => Ok(Self::Datadog),
            _ => Err(format!("Invalid delivery channel type: {s}")),
        }
    }
}

/// Delivery channel details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryChannel {
    /// Channel ID
    pub id: String,

    /// Human-readable label (e.g. "Email: ops@example.com")
    #[serde(default)]
    pub label: Option<String>,

    /// Channel type
    #[serde(rename = "type")]
    pub channel_type: DeliveryChannelType,

    /// Owner (user) ID
    #[serde(default)]
    pub owner_id: Option<String>,

    /// Project ID (`null` for account-level channels)
    #[serde(default)]
    pub project_id: Option<String>,

    /// Reference ID (e.g. the email address or webhook ID)
    #[serde(default)]
    pub reference_id: Option<String>,

    /// Whether the channel is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Channel-specific details (which fields are set depends on the type)
    #[serde(default)]
    pub information: DeliveryChannelInformation,
}

/// Type-specific delivery channel details
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeliveryChannelInformation {
    /// Email address (email channels)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,

    /// Email addresses (email channels)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emails: Option<Vec<String>>,

    /// Team/workspace name (Slack channels)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,

    /// Channel name (Slack/Discord channels)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub channel_name: Option<String>,

    /// Webhook URL (webhook channels)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook: Option<String>,

    /// Chat ID (Telegram channels)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_id: Option<i64>,
}

/// Response for listing delivery channels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListDeliveryChannelsResponse {
    /// List of delivery channels
    #[serde(default)]
    pub delivery_channels: Vec<DeliveryChannel>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_delivery_channels_response_spec_shape() {
        let json = serde_json::json!({
            "delivery_channels": [{
                "id": "00000000-0000-0000-0000-000000000001",
                "label": "Email: alerts@example.com",
                "type": "email",
                "owner_id": "00000000-0000-0000-0000-000000000002",
                "project_id": null,
                "reference_id": "alerts@example.com",
                "enabled": true,
                "created_at": "2024-01-01T00:00:00Z",
                "information": {
                    "email": "alerts@example.com",
                    "emails": ["alerts@example.com"]
                }
            }]
        });
        let resp: ListDeliveryChannelsResponse = serde_json::from_value(json).unwrap();
        let ch = &resp.delivery_channels[0];
        assert_eq!(ch.label.as_deref(), Some("Email: alerts@example.com"));
        assert_eq!(ch.channel_type, DeliveryChannelType::Email);
        assert!(ch.project_id.is_none());
        assert_eq!(ch.reference_id.as_deref(), Some("alerts@example.com"));
        assert_eq!(ch.information.email.as_deref(), Some("alerts@example.com"));
        assert_eq!(ch.information.emails.as_ref().map(Vec::len), Some(1));
    }
}
