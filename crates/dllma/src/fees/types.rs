//! Types for fees and revenue data

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Options for fees overview endpoints
#[derive(Debug, Clone, Default)]
pub struct FeesOverviewOptions {
    /// Exclude totalDataChart from response (reduces payload size)
    pub exclude_total_data_chart: bool,
    /// Exclude totalDataChartBreakdown from response (reduces payload size)
    pub exclude_total_data_chart_breakdown: bool,
    /// Data type filter: "dailyFees" or "totalFees"
    pub data_type: Option<String>,
}

impl FeesOverviewOptions {
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

/// Fees overview response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeesOverview {
    /// Total 24h fees
    pub total24h: Option<f64>,
    /// Total 48h to 24h fees
    pub total48h_to24h: Option<f64>,
    /// Total 7d fees
    pub total7d: Option<f64>,
    /// Total 30d fees
    pub total30d: Option<f64>,
    /// Total all-time fees
    pub total_all_time: Option<f64>,
    /// Average 1d fees
    pub average1d: Option<f64>,
    /// Fee change 1d (percentage)
    pub change_1d: Option<f64>,
    /// Fee change 7d (percentage)
    pub change_7d: Option<f64>,
    /// Fee change 30d (percentage)
    pub change_30d: Option<f64>,
    /// Fee change 1m (percentage)
    pub change_1m: Option<f64>,
    /// Historical chart data
    #[serde(default)]
    pub total_data_chart: Vec<Vec<serde_json::Value>>,
    /// Historical breakdown
    #[serde(default)]
    pub total_data_chart_breakdown: Vec<serde_json::Value>,
    /// List of protocols
    #[serde(default)]
    pub protocols: Vec<FeesProtocol>,
    /// All chains
    #[serde(default)]
    pub all_chains: Vec<String>,
}

/// Protocol fees data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeesProtocol {
    /// Protocol name
    pub name: String,
    /// Display name
    pub display_name: Option<String>,
    /// Protocol ID
    pub id: Option<String>,
    /// Logo URL
    pub logo: Option<String>,
    /// Chains
    #[serde(default)]
    pub chains: Vec<String>,
    /// Category
    pub category: Option<String>,
    /// Module name
    pub module: Option<String>,
    /// 24h fees
    pub total24h: Option<f64>,
    /// 48h to 24h fees
    pub total48h_to24h: Option<f64>,
    /// 7d fees
    pub total7d: Option<f64>,
    /// 30d fees
    pub total30d: Option<f64>,
    /// All-time fees
    pub total_all_time: Option<f64>,
    /// 24h revenue
    pub revenue24h: Option<f64>,
    /// 24h user fees
    pub user_fees24h: Option<f64>,
    /// 24h holder revenue
    pub holder_revenue24h: Option<f64>,
    /// 24h creator revenue
    pub creator_revenue24h: Option<f64>,
    /// 24h supply-side revenue
    pub supply_side_revenue24h: Option<f64>,
    /// 24h protocol revenue
    pub protocol_revenue24h: Option<f64>,
    /// 1d change
    pub change_1d: Option<f64>,
    /// 7d change
    pub change_7d: Option<f64>,
    /// 30d change
    pub change_30d: Option<f64>,
    /// Chain breakdown
    #[serde(default)]
    pub breakdown24h: Option<HashMap<String, HashMap<String, f64>>>,
    /// `DefiLlama` ID
    pub defi_llama_id: Option<String>,
    /// Parent protocol
    pub parent_protocol: Option<String>,
    /// Methodology URL
    pub methodology_url: Option<String>,
}

/// Protocol fees summary
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProtocolFeesSummary {
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
    /// Historical fees data
    #[serde(default)]
    pub total_data_chart: Vec<Vec<serde_json::Value>>,
    /// Historical breakdown
    #[serde(default)]
    pub total_data_chart_breakdown: Vec<serde_json::Value>,
    /// 24h fees
    pub total24h: Option<f64>,
    /// 7d fees
    pub total7d: Option<f64>,
    /// 30d fees
    pub total30d: Option<f64>,
    /// All-time fees
    pub total_all_time: Option<f64>,
    /// 24h revenue
    pub revenue24h: Option<f64>,
    /// All-time revenue
    pub total_all_time_revenue: Option<f64>,
    /// 1d change
    pub change_1d: Option<f64>,
}
