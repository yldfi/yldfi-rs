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

/// Contract review request (`POST /contracts-review`): report contracts as
/// spam or not spam
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractReviewRequest {
    /// Contracts to report
    pub contracts: Vec<ContractInput>,
}

/// One contract report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractInput {
    /// Contract address
    pub contract_address: String,
    /// Report type (e.g. `spam`, `not_spam`)
    pub report_type: String,
    /// Contract type (e.g. `ERC20`, `NFT`)
    pub contract_type: String,
    /// Reason for the report
    pub reason: String,
}

/// Contract review response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractReviewResponse {
    /// Confirmation message
    pub message: Option<String>,
}
