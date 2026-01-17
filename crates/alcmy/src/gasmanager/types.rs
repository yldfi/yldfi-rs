//! Types for the Gas Manager API

use serde::{Deserialize, Serialize};

/// Partial UserOperation for gas sponsorship (v0.6)
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialUserOperationV06 {
    pub sender: String,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub init_code: Option<String>,
    pub call_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_gas_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_verification_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster_and_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Partial UserOperation for gas sponsorship (v0.7)
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialUserOperationV07 {
    pub sender: String,
    pub nonce: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub factory_data: Option<String>,
    pub call_data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_gas_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_verification_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<String>,
}

/// ERC-20 context for token payment
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20Context {
    /// Token address
    pub token_address: String,
    /// Token amount (hex)
    pub amount: String,
}

/// Gas/fee overrides
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_gas_limit: Option<GasOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<GasOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_verification_gas: Option<GasOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_fee_per_gas: Option<GasOverride>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_priority_fee_per_gas: Option<GasOverride>,
}

/// Single gas override value
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum GasOverride {
    /// Fixed value (hex)
    Fixed(String),
    /// Multiplier
    Multiplier { multiplier: f64 },
}

/// Response for gas sponsorship request (v0.6)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasSponsorshipResponseV06 {
    /// Paymaster and data
    pub paymaster_and_data: String,
    /// Call gas limit (hex)
    pub call_gas_limit: String,
    /// Verification gas limit (hex)
    pub verification_gas_limit: String,
    /// Pre-verification gas (hex)
    pub pre_verification_gas: String,
    /// Max fee per gas (hex)
    pub max_fee_per_gas: String,
    /// Max priority fee per gas (hex)
    pub max_priority_fee_per_gas: String,
}

/// Response for gas sponsorship request (v0.7)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasSponsorshipResponseV07 {
    /// Paymaster address
    pub paymaster: String,
    /// Paymaster data (hex)
    pub paymaster_data: String,
    /// Paymaster verification gas limit (hex)
    pub paymaster_verification_gas_limit: String,
    /// Paymaster post-op gas limit (hex)
    pub paymaster_post_op_gas_limit: String,
    /// Call gas limit (hex)
    pub call_gas_limit: String,
    /// Verification gas limit (hex)
    pub verification_gas_limit: String,
    /// Pre-verification gas (hex)
    pub pre_verification_gas: String,
    /// Max fee per gas (hex)
    pub max_fee_per_gas: String,
    /// Max priority fee per gas (hex)
    pub max_priority_fee_per_gas: String,
}

/// Response for requestPaymasterAndData (no gas estimation)
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPaymasterAndDataResponse {
    /// Paymaster and data (v0.6 format)
    pub paymaster_and_data: Option<String>,
    /// Paymaster address (v0.7 format)
    pub paymaster: Option<String>,
    /// Paymaster data (v0.7 format)
    pub paymaster_data: Option<String>,
}

/// Paymaster stub data response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymasterStubDataResponse {
    /// Sponsor info
    pub sponsor: Option<SponsorInfo>,
    /// Paymaster (v0.7)
    pub paymaster: Option<String>,
    /// Paymaster data (v0.7)
    pub paymaster_data: Option<String>,
    /// Paymaster and data (v0.6)
    pub paymaster_and_data: Option<String>,
    /// Paymaster verification gas limit (v0.7)
    pub paymaster_verification_gas_limit: Option<String>,
    /// Paymaster post-op gas limit (v0.7)
    pub paymaster_post_op_gas_limit: Option<String>,
}

/// Sponsor info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SponsorInfo {
    /// Sponsor name
    pub name: String,
    /// Sponsor icon URL
    pub icon: Option<String>,
}

/// Token quote response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenQuoteResponse {
    /// Tokens per ETH
    pub tokens_per_eth: String,
    /// Estimated token amount
    pub estimated_token_amount: String,
    /// Estimated USD value
    pub estimated_usd: Option<String>,
}

// ========== Admin API Types ==========

/// Gas manager policy
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasPolicy {
    /// Policy ID
    pub id: String,
    /// Policy name
    pub name: String,
    /// Policy status
    pub status: PolicyStatus,
    /// Policy rules
    pub rules: PolicyRules,
    /// Created at timestamp
    pub created_at: Option<String>,
    /// Updated at timestamp
    pub updated_at: Option<String>,
}

/// Policy status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PolicyStatus {
    Active,
    Inactive,
}

/// Policy rules
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRules {
    /// Sender allowlist
    #[serde(default)]
    pub sender_allowlist: Vec<String>,
    /// Sender blocklist
    #[serde(default)]
    pub sender_blocklist: Vec<String>,
    /// Spending limits
    #[serde(default)]
    pub spending_limits: Option<SpendingLimits>,
    /// Start time (ISO 8601)
    pub start_time: Option<String>,
    /// End time (ISO 8601)
    pub end_time: Option<String>,
    /// Webhook URL for custom rules
    pub webhook_url: Option<String>,
}

/// Spending limits
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpendingLimits {
    /// Max spend per user operation (wei)
    pub max_spend_per_uo: Option<String>,
    /// Max spend per sender (wei)
    pub max_spend_per_sender: Option<String>,
    /// Max count per sender
    pub max_count_per_sender: Option<u64>,
}

/// Request to create a policy
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePolicyRequest {
    /// Policy name
    pub name: String,
    /// Policy rules
    pub rules: PolicyRules,
    /// App ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
}

/// Request to update a policy
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePolicyRequest {
    /// Policy rules
    pub rules: PolicyRules,
}

/// Policy statistics
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyStats {
    /// Total sponsored operations
    pub total_sponsored: u64,
    /// Total spent (wei)
    pub total_spent: String,
    /// Unique senders
    pub unique_senders: u64,
}

/// Sponsorship record
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sponsorship {
    /// UserOperation hash
    pub user_op_hash: String,
    /// Sender address
    pub sender: String,
    /// Transaction hash
    pub tx_hash: Option<String>,
    /// Sponsored amount (wei)
    pub amount: String,
    /// Timestamp
    pub timestamp: String,
}

/// List policies response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPoliciesResponse {
    pub data: Vec<GasPolicy>,
    pub page_info: Option<PageInfo>,
}

/// Page info for pagination
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub has_next_page: bool,
    pub end_cursor: Option<String>,
}

/// List sponsorships response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSponsorshipsResponse {
    pub data: Vec<Sponsorship>,
    pub page_info: Option<PageInfo>,
}
