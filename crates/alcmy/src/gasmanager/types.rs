//! Types for the Gas Manager API

use serde::{Deserialize, Serialize};

/// Partial `UserOperation` for gas sponsorship (v0.6)
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

/// Partial `UserOperation` for gas sponsorship (v0.7)
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
//
// Shapes follow the Gas Manager Admin API (`https://manage.g.alchemy.com`).
// Every response wraps its payload in a `data` object; the API methods
// unwrap it, so these types describe the inner payload.

/// Response envelope used by every Gas Manager Admin endpoint
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct DataEnvelope<T> {
    pub data: T,
}

/// `data` payload of the single-policy endpoints (create, get, replace, status)
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PolicyData {
    pub policy: GasPolicy,
}

/// Gas manager policy
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GasPolicy {
    /// Policy ID
    pub policy_id: String,
    /// App ID the policy belongs to
    pub app_id: String,
    /// Policy status
    pub status: PolicyStatus,
    /// Policy name
    #[serde(default)]
    pub policy_name: Option<String>,
    /// Policy type
    #[serde(default)]
    pub policy_type: Option<PolicyType>,
    /// Rules (`sponsorship` / `bundler_sponsorship` policies)
    #[serde(default)]
    pub rules: Option<PolicyRules>,
    /// ERC-20 rules (`erc20` policies)
    #[serde(default)]
    pub erc20_rules: Option<Erc20Rules>,
    /// Solana rules (`solana` policies)
    #[serde(default)]
    pub solana_rules: Option<SolanaRules>,
    /// Networks the policy is active on (e.g. `ETH_MAINNET`)
    #[serde(default)]
    pub networks: Vec<String>,
    /// Policy state (e.g. `ongoing`)
    #[serde(default)]
    pub policy_state: Option<String>,
    /// Policy version
    #[serde(default)]
    pub policy_version: Option<u64>,
    /// Unix timestamp (string) of the last update
    #[serde(default)]
    pub last_updated_unix: Option<String>,
    /// Gas pump ID
    #[serde(default)]
    pub gas_pump_id: Option<String>,
}

/// Policy status
///
/// Only `Active` and `Inactive` can be set via
/// [`GasManagerApi::set_policy_status`](super::GasManagerApi::set_policy_status).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyStatus {
    Active,
    Inactive,
    PendingApproval,
    Rejected,
    /// Any status not known to this client
    #[serde(other)]
    Unknown,
}

/// Policy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyType {
    /// Sponsor gas on EVM networks
    Sponsorship,
    /// Sponsor gas on EVM networks via the bundler
    BundlerSponsorship,
    /// Let users pay gas with an ERC-20 token
    Erc20,
    /// Sponsor fees and rent on Solana
    Solana,
    /// Any type not known to this client
    #[serde(other)]
    Unknown,
}

/// Rules for `sponsorship` / `bundler_sponsorship` policies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyRules {
    /// Maximum USD that can be sponsored
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_spend_usd: Option<f64>,
    /// Maximum USD that can be sponsored for a single sender
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_spend_per_sender_usd: Option<f64>,
    /// Maximum USD that can be sponsored for a single user operation
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_spend_per_uo_usd: Option<f64>,
    /// Maximum number of user operations that can be sponsored
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u64>,
    /// Maximum number of user operations per sender
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_count_per_sender: Option<u64>,
    /// Addresses eligible for sponsorship
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sender_allowlist: Vec<String>,
    /// Addresses banned from sponsorship
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sender_blocklist: Vec<String>,
    /// Unix timestamp (string) when the policy starts; required when creating
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time_unix: Option<String>,
    /// Unix timestamp (string) when the policy ends
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time_unix: Option<String>,
    /// Milliseconds (string) after issuance that a sponsorship expires
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sponsorship_expiry_ms: Option<String>,
    /// Webhook for conditional sponsorship
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub webhook_rules: Option<WebhookRules>,
}

/// Webhook rules for conditional gas sponsorship
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookRules {
    /// URL called to verify sponsorship eligibility
    pub webhook_url: String,
    /// Sponsor the user operation if the webhook errors or times out
    #[serde(default)]
    pub approve_on_failure: bool,
}

/// Rules for `solana` policies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SolanaRules {
    /// Maximum USD that can be sponsored
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_spend_usd: Option<f64>,
    /// Maximum USD that can be sponsored per transaction
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_spend_per_txn_usd: Option<f64>,
    /// Maximum number of transactions that can be sponsored
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_count: Option<u64>,
    /// Unix timestamp (string) when the policy starts; required when creating
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time_unix: Option<String>,
    /// Unix timestamp (string) when the policy ends
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time_unix: Option<String>,
}

/// Rules for `erc20` policies (the base rules plus token settings)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20Rules {
    /// Base spending / time rules
    #[serde(flatten)]
    pub rules: PolicyRules,
    /// ERC-20 tokens accepted under this policy
    #[serde(default)]
    pub tokens: Vec<Erc20Token>,
    /// Address receiving the tokens paid by users
    #[serde(default)]
    pub recipient_address: String,
    /// Charge the user after execution (`true`) or before (`false`)
    #[serde(default)]
    pub use_post_op: bool,
    /// Multiplier applied to the ERC-20 amount the user pays
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_multiplier: Option<f64>,
}

/// ERC-20 token accepted by an `erc20` policy
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20Token {
    /// Network of the token (e.g. `ETH_MAINNET`)
    pub network: String,
    /// Token contract address
    pub token_address: String,
    /// Network of the price-reference token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_reference_network: Option<String>,
    /// Contract address of the price-reference token
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_reference_token_address: Option<String>,
}

/// Request to create a policy (the new policy starts `inactive`)
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePolicyRequest {
    /// Policy name
    pub policy_name: String,
    /// Policy type
    pub policy_type: PolicyType,
    /// App ID the policy belongs to
    pub app_id: String,
    /// Networks (required for `sponsorship`, `bundler_sponsorship` and `solana`)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub networks: Vec<String>,
    /// Rules for `sponsorship` / `bundler_sponsorship` policies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<PolicyRules>,
    /// Rules for `solana` policies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solana_rules: Option<SolanaRules>,
    /// Rules for `erc20` policies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erc20_rules: Option<Erc20Rules>,
}

/// Request to replace a policy (`PUT /policy/{id}` replaces all rules)
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePolicyRequest {
    /// Policy name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
    /// Networks
    #[serde(skip_serializing_if = "Option::is_none")]
    pub networks: Option<Vec<String>>,
    /// Rules for `sponsorship` / `bundler_sponsorship` policies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rules: Option<PolicyRules>,
    /// Rules for `solana` policies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solana_rules: Option<SolanaRules>,
    /// Rules for `erc20` policies
    #[serde(skip_serializing_if = "Option::is_none")]
    pub erc20_rules: Option<Erc20Rules>,
}

/// Query options for listing policies
#[derive(Debug, Clone, Default)]
pub struct ListPoliciesOptions {
    /// Only return policies of this app
    pub app_id: Option<String>,
    /// Results per page (max 50)
    pub limit: Option<u32>,
    /// Cursor for the previous page (`before` from a response)
    pub before: Option<String>,
    /// Cursor for the next page (`after` from a response)
    pub after: Option<String>,
    /// Case-insensitive policy name filter
    pub name_filter: Option<String>,
}

/// Query options for listing sponsorships
#[derive(Debug, Clone, Default)]
pub struct ListSponsorshipsOptions {
    /// Results per page (max 100)
    pub limit: Option<u32>,
    /// Cursor for the previous page (`before` from a response)
    pub before: Option<String>,
    /// Cursor for the next page (`after` from a response)
    pub after: Option<String>,
}

/// Policy statistics (`GET /policy/{id}/stats/detailed`)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyStats {
    /// Totals across all networks
    #[serde(default)]
    pub policy_stats: PolicyStatsSummary,
    /// Per-network breakdown
    #[serde(default)]
    pub policy_network_stats: Vec<PolicyNetworkStats>,
}

/// Policy totals
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyStatsSummary {
    #[serde(default)]
    pub signatures_mined: u64,
    #[serde(default)]
    pub signatures_expired: u64,
    #[serde(default)]
    pub signatures_pending: u64,
    #[serde(default)]
    pub usd_pending: f64,
    #[serde(default)]
    pub usd_mined: f64,
}

/// Per-network policy statistics
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyNetworkStats {
    /// Network (e.g. `ETH_MAINNET`)
    #[serde(default)]
    pub network: Option<String>,
    #[serde(default)]
    pub signatures_mined: u64,
    #[serde(default)]
    pub signatures_expired: u64,
    #[serde(default)]
    pub signatures_pending: u64,
    #[serde(default)]
    pub usd_pending: f64,
    #[serde(default)]
    pub usd_mined: f64,
    /// Pending amount in the native token (kept as a JSON number to avoid
    /// overflow / precision loss)
    #[serde(default)]
    pub pending_native_token: Option<serde_json::Number>,
    /// Mined amount in the native token
    #[serde(default)]
    pub mined_native_token: Option<serde_json::Number>,
    /// Native currency symbol
    #[serde(default)]
    pub currency: Option<String>,
    /// ERC-20 amounts collected (`erc20` policies)
    #[serde(default)]
    pub erc20_stats: Vec<Erc20NetworkStats>,
}

/// ERC-20 amount collected on one network
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Erc20NetworkStats {
    #[serde(default)]
    pub token_address: Option<String>,
    #[serde(default)]
    pub token_amount: Option<String>,
}

/// Sponsorship status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum SponsorshipStatus {
    Pending,
    Mined,
    Expired,
    /// Any status not known to this client
    #[serde(other)]
    Unknown,
}

/// Sponsorship record
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sponsorship {
    /// Sender address
    pub sender: String,
    /// Unix timestamp (string) when the sponsorship was granted
    #[serde(default)]
    pub granted_at: Option<String>,
    /// Sponsorship status
    pub status: SponsorshipStatus,
    /// `UserOperation` hash
    pub uo_hash: String,
    /// Explorer URL for the user operation
    #[serde(default)]
    pub uo_explorer_url: Option<String>,
    /// Total USD sponsored (null until mined)
    #[serde(default)]
    pub confirmed_total_usd: Option<f64>,
    /// Bundle transaction hash (null until mined)
    #[serde(default)]
    pub txn_hash: Option<String>,
    /// Explorer URL for the bundle transaction
    #[serde(default)]
    pub txn_explorer_url: Option<String>,
    /// Network the user operation was submitted on
    #[serde(default)]
    pub network: Option<String>,
    /// ERC-20 token address (ERC-20 sponsorships only)
    #[serde(default)]
    pub token_address: Option<String>,
    /// ERC-20 token amount paid (ERC-20 sponsorships only)
    #[serde(default)]
    pub token_amount: Option<String>,
}

/// List policies response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPoliciesResponse {
    /// Policies on this page
    #[serde(default)]
    pub policies: Vec<GasPolicy>,
    /// Cursor for the previous page
    #[serde(default)]
    pub before: Option<String>,
    /// Cursor for the next page
    #[serde(default)]
    pub after: Option<String>,
}

/// List sponsorships response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListSponsorshipsResponse {
    /// Sponsorships on this page
    #[serde(default)]
    pub sponsorships: Vec<Sponsorship>,
    /// Cursor for the previous page
    #[serde(default)]
    pub before: Option<String>,
    /// Cursor for the next page
    #[serde(default)]
    pub after: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_policy_response_parses() {
        let json = r#"{"data":{"policy":{
            "policyId":"pol-1","appId":"app-1","status":"pending_approval",
            "policyName":"Test","policyType":"sponsorship","policyState":"ongoing",
            "policyVersion":2,"lastUpdatedUnix":"1700000000","networks":["ETH_SEPOLIA"],
            "rules":{"maxSpendUsd":100.5,"maxCountPerSender":2,
                     "senderAllowlist":["0x0000000000000000000000000000000000000001"],
                     "startTimeUnix":"1700000000","sponsorshipExpiryMs":"600000",
                     "webhookRules":{"webhookUrl":"https://example.com/hook","approveOnFailure":true}}
        }}}"#;
        let env: DataEnvelope<PolicyData> = serde_json::from_str(json).unwrap();
        let p = env.data.policy;
        assert_eq!(p.policy_id, "pol-1");
        assert_eq!(p.status, PolicyStatus::PendingApproval);
        assert_eq!(p.policy_type, Some(PolicyType::Sponsorship));
        let rules = p.rules.unwrap();
        assert_eq!(rules.max_spend_usd, Some(100.5));
        assert_eq!(rules.max_count_per_sender, Some(2));
        assert!(rules.webhook_rules.unwrap().approve_on_failure);
    }

    #[test]
    fn erc20_and_solana_policies_parse() {
        let json = r#"{"policyId":"p","appId":"a","status":"active","policyType":"erc20",
            "erc20Rules":{"startTimeUnix":"1","tokens":[{"network":"BASE_MAINNET","tokenAddress":"0xabc"}],
                          "recipientAddress":"0xdef","usePostOp":true,"priceMultiplier":1.1},
            "solanaRules":{"maxSpendPerTxnUsd":3.0,"startTimeUnix":"1"}}"#;
        let p: GasPolicy = serde_json::from_str(json).unwrap();
        let erc20 = p.erc20_rules.unwrap();
        assert_eq!(erc20.rules.start_time_unix.as_deref(), Some("1"));
        assert_eq!(erc20.tokens[0].token_address, "0xabc");
        assert!(erc20.use_post_op);
        assert_eq!(p.solana_rules.unwrap().max_spend_per_txn_usd, Some(3.0));
    }

    #[test]
    fn create_policy_request_serializes_spec_fields() {
        let req = CreatePolicyRequest {
            policy_name: "My Policy".into(),
            policy_type: PolicyType::BundlerSponsorship,
            app_id: "app-1".into(),
            networks: vec!["ETH_SEPOLIA".into()],
            rules: Some(PolicyRules {
                max_spend_per_uo_usd: Some(20.0),
                start_time_unix: Some("1700000000".into()),
                ..Default::default()
            }),
            solana_rules: None,
            erc20_rules: None,
        };
        let v = serde_json::to_value(&req).unwrap();
        assert_eq!(
            v,
            serde_json::json!({
                "policyName": "My Policy",
                "policyType": "bundler_sponsorship",
                "appId": "app-1",
                "networks": ["ETH_SEPOLIA"],
                "rules": {"maxSpendPerUoUsd": 20.0, "startTimeUnix": "1700000000"}
            })
        );
    }

    #[test]
    fn list_and_stats_responses_parse() {
        let json = r#"{"data":{"policies":[{"policyId":"p","appId":"a","status":"inactive"}],
                       "before":null,"after":"cursor-2"}}"#;
        let env: DataEnvelope<ListPoliciesResponse> = serde_json::from_str(json).unwrap();
        assert_eq!(env.data.policies.len(), 1);
        assert_eq!(env.data.after.as_deref(), Some("cursor-2"));

        let json = r#"{"data":{"policyStats":{"signaturesMined":3,"signaturesExpired":1,
                         "signaturesPending":0,"usdPending":0.0,"usdMined":1.25},
                       "policyNetworkStats":[{"network":"ETH_SEPOLIA","signaturesMined":3,
                         "minedNativeToken":123456789012345678901234,"currency":"ETH",
                         "erc20Stats":[{"tokenAddress":"0xabc","tokenAmount":"10"}]}]}}"#;
        let env: DataEnvelope<PolicyStats> = serde_json::from_str(json).unwrap();
        assert_eq!(env.data.policy_stats.signatures_mined, 3);
        assert_eq!(env.data.policy_stats.usd_mined, 1.25);
        let net = &env.data.policy_network_stats[0];
        assert!(net.mined_native_token.is_some());
        assert_eq!(net.erc20_stats[0].token_amount.as_deref(), Some("10"));

        let json = r#"{"data":{"before":null,"after":null,"sponsorships":[{
            "sender":"0x0000000000000000000000000000000000000002","grantedAt":"1700000000",
            "confirmedTotalUsd":null,"status":"PENDING","uoHash":"0x01",
            "uoExplorerUrl":"https://example.com/uo/0x01","txnHash":null,"network":"ETH_SEPOLIA"}]}}"#;
        let env: DataEnvelope<ListSponsorshipsResponse> = serde_json::from_str(json).unwrap();
        let s = &env.data.sponsorships[0];
        assert_eq!(s.status, SponsorshipStatus::Pending);
        assert!(s.confirmed_total_usd.is_none());
    }
}
