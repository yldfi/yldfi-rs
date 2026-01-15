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
    /// PagerDuty integration
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
            _ => Err(format!("Invalid delivery channel type: {}", s)),
        }
    }
}

/// Delivery channel details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryChannel {
    /// Channel ID
    pub id: String,

    /// Channel name
    #[serde(default)]
    pub name: Option<String>,

    /// Channel type
    #[serde(rename = "type")]
    pub channel_type: DeliveryChannelType,

    /// Whether the channel is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Channel-specific configuration
    #[serde(default)]
    pub config: serde_json::Value,

    /// Creation timestamp
    #[serde(default)]
    pub created_at: Option<String>,

    /// Last updated timestamp
    #[serde(default)]
    pub updated_at: Option<String>,
}

/// Response for listing delivery channels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ListDeliveryChannelsResponse {
    /// List of delivery channels
    #[serde(default)]
    pub delivery_channels: Vec<DeliveryChannel>,
}
