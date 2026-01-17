//! Types for the Utils API

use serde::{Deserialize, Serialize};

/// Request for calling a contract function
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunContractFunctionRequest {
    /// ABI of the function to call
    pub abi: serde_json::Value,
    /// Function parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
}

/// Contract function result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractFunctionResult {
    /// Result data
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Web3 version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Web3Version {
    /// Version string
    pub version: Option<String>,
}

/// Endpoint weight info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointWeight {
    /// Endpoint name
    pub endpoint: Option<String>,
    /// Path
    pub path: Option<String>,
    /// Weight/cost
    pub weight: Option<i32>,
    /// Rate limit per minute
    pub rate_limit_per_minute: Option<i32>,
}

/// Endpoint weights response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointWeightsResponse {
    /// List of endpoints with weights
    pub endpoints: Vec<EndpointWeight>,
}

/// Contract events request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetContractEventsRequest {
    /// ABI of the events to decode
    pub abi: serde_json::Value,
    /// Topic (event signature hash)
    pub topic: String,
}

/// Contract event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    /// Transaction hash
    pub transaction_hash: Option<String>,
    /// Block number
    pub block_number: Option<String>,
    /// Block timestamp
    pub block_timestamp: Option<String>,
    /// Log index
    pub log_index: Option<i32>,
    /// Decoded data
    pub data: Option<serde_json::Value>,
}

/// Contract events response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEventsResponse {
    /// Cursor
    pub cursor: Option<String>,
    /// Page size
    pub page_size: Option<i32>,
    /// Results
    pub result: Vec<ContractEvent>,
}

/// Contract review request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractReviewRequest {
    /// Contract addresses to review
    pub contracts: Vec<ContractInput>,
}

/// Contract input for review
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInput {
    /// Contract address
    pub address: String,
    /// Chain (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain: Option<String>,
}

/// Contract review result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractReview {
    /// Contract address
    pub address: Option<String>,
    /// Chain
    pub chain: Option<String>,
    /// Is verified
    pub is_verified: Option<bool>,
    /// Is proxy
    pub is_proxy: Option<bool>,
    /// Implementation address (if proxy)
    pub implementation_address: Option<String>,
    /// Contract name
    pub name: Option<String>,
    /// Has source code
    pub has_source: Option<bool>,
    /// Security findings
    pub security_findings: Option<Vec<SecurityFinding>>,
}

/// Security finding from contract review
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SecurityFinding {
    /// Finding type/category
    pub finding_type: Option<String>,
    /// Severity level
    pub severity: Option<String>,
    /// Description
    pub description: Option<String>,
}
