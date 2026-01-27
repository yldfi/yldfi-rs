//! Types for volume data (DEX, Options, Derivatives)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for volume overview endpoints
#[derive(Debug, Clone, Default)]
pub struct VolumeOverviewOptions {
    /// Exclude totalDataChart from response (reduces payload size)
    pub exclude_total_data_chart: bool,
    /// Exclude totalDataChartBreakdown from response (reduces payload size)
    pub exclude_total_data_chart_breakdown: bool,
    /// Data type filter: "dailyVolume" or "totalVolume"
    pub data_type: Option<String>,
}

impl VolumeOverviewOptions {
    /// Create new options with defaults
    #[must_use] 
    pub fn new() -> Self {
        Self::default()
    }

    /// Exclude chart data from response
    #[must_use] 
    pub fn exclude_charts(mut self) -> Self {
        self.exclude_total_data_chart = true;
        self.exclude_total_data_chart_breakdown = true;
        self
    }

    /// Set data type filter
    pub fn data_type(mut self, data_type: impl Into<String>) -> Self {
        self.data_type = Some(data_type.into());
        self
    }

    /// Build query string
    #[must_use] 
    pub fn to_query_string(&self) -> String {
        let mut params = Vec::new();
        if self.exclude_total_data_chart {
            params.push("excludeTotalDataChart=true".to_string());
        }
        if self.exclude_total_data_chart_breakdown {
            params.push("excludeTotalDataChartBreakdown=true".to_string());
        }
        if let Some(ref dt) = self.data_type {
            params.push(format!("dataType={dt}"));
        }
        if params.is_empty() {
            String::new()
        } else {
            format!("?{}", params.join("&"))
        }
    }
}

/// Volume overview response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeOverview {
    /// Total 24h volume
    pub total24h: Option<f64>,
    /// Total 48h to 24h volume (previous day)
    pub total48h_to24h: Option<f64>,
    /// Total 7d volume
    pub total7d: Option<f64>,
    /// Total 30d volume
    pub total30d: Option<f64>,
    /// Total all-time volume
    pub total_all_time: Option<f64>,
    /// Average 1d volume
    pub average1d: Option<f64>,
    /// Volume change 1d (percentage)
    pub change_1d: Option<f64>,
    /// Volume change 7d (percentage)
    pub change_7d: Option<f64>,
    /// Volume change 30d (percentage)
    pub change_30d: Option<f64>,
    /// Volume change 7d over 7d (percentage)
    pub change_7d_over_7d: Option<f64>,
    /// Historical chart data: [[timestamp, volume], ...]
    #[serde(default)]
    pub total_data_chart: Vec<(u64, f64)>,
    /// Historical breakdown: [[timestamp, {protocol: volume}], ...]
    #[serde(default)]
    pub total_data_chart_breakdown: Vec<(u64, HashMap<String, f64>)>,
    /// List of protocols
    #[serde(default)]
    pub protocols: Vec<VolumeProtocol>,
    /// All chains
    #[serde(default)]
    pub all_chains: Vec<String>,
}

/// Protocol volume data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VolumeProtocol {
    /// Protocol ID
    pub id: Option<String>,
    /// Protocol name
    pub name: String,
    /// Protocol display name
    pub display_name: Option<String>,
    /// Logo URL
    pub logo: Option<String>,
    /// Chains the protocol is on
    #[serde(default)]
    pub chains: Vec<String>,
    /// Protocol category
    pub category: Option<String>,
    /// Module name
    pub module: Option<String>,
    /// 24h volume
    pub total24h: Option<f64>,
    /// 48h to 24h volume
    pub total48h_to24h: Option<f64>,
    /// 7d volume
    pub total7d: Option<f64>,
    /// 30d volume
    pub total30d: Option<f64>,
    /// All-time volume
    pub total_all_time: Option<f64>,
    /// 1d volume change
    pub change_1d: Option<f64>,
    /// 7d volume change
    pub change_7d: Option<f64>,
    /// 30d volume change
    pub change_30d: Option<f64>,
    /// Chain breakdown
    #[serde(default)]
    pub breakdown24h: Option<HashMap<String, HashMap<String, f64>>>,
    /// Method type
    pub methodology_url: Option<String>,
    /// `DefiLlama` ID
    pub defi_llama_id: Option<String>,
    /// Parent protocol
    pub parent_protocol: Option<String>,
}

/// Protocol volume summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolVolumeSummary {
    /// Protocol name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Protocol ID
    pub id: Option<String>,
    /// Logo URL
    pub logo: Option<String>,
    /// `DefiLlama` ID
    pub defi_llama_id: Option<String>,
    /// Protocol URL
    pub url: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Chains
    #[serde(default)]
    pub chains: Vec<String>,
    /// Category
    pub category: Option<String>,
    /// Historical volume data
    #[serde(default)]
    pub total_data_chart: Vec<Vec<serde_json::Value>>,
    /// Historical breakdown
    #[serde(default)]
    pub total_data_chart_breakdown: Vec<serde_json::Value>,
    /// 24h volume
    pub total24h: Option<f64>,
    /// 7d volume
    pub total7d: Option<f64>,
    /// 30d volume
    pub total30d: Option<f64>,
    /// All-time volume
    pub total_all_time: Option<f64>,
    /// 1d change
    pub change_1d: Option<f64>,
}

/// Volume type (DEX, Options, Derivatives)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VolumeType {
    /// DEX trading volume
    Dex,
    /// Options trading volume
    Options,
    /// Derivatives trading volume (Pro only)
    Derivatives,
}

impl VolumeType {
    /// Get the API path segment
    #[must_use] 
    pub fn path(&self) -> &'static str {
        match self {
            Self::Dex => "dexs",
            Self::Options => "options",
            Self::Derivatives => "derivatives",
        }
    }
}

/// Open interest overview response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterestOverview {
    /// List of protocols with open interest data
    #[serde(default)]
    pub protocols: Vec<OpenInterestProtocol>,
    /// Historical chart data: [[timestamp, `open_interest`], ...]
    #[serde(default)]
    pub total_data_chart: Vec<(u64, f64)>,
    /// All chains with open interest data
    #[serde(default)]
    pub all_chains: Vec<String>,
}

/// Protocol open interest data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenInterestProtocol {
    /// Protocol name
    pub name: Option<String>,
    /// Display name
    pub display_name: Option<String>,
    /// 24h open interest
    pub total24h: Option<f64>,
    /// 7d open interest
    pub total7d: Option<f64>,
    /// 1d change percentage
    pub change_1d: Option<f64>,
    /// 7d change percentage
    pub change_7d: Option<f64>,
    /// Chains the protocol is on
    #[serde(default)]
    pub chains: Vec<String>,
}
