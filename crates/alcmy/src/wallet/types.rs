//! Types for the Wallet API

use serde::{Deserialize, Serialize};

/// Account type for smart wallets
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AccountType {
    /// Modular Account (default)
    ModularAccountV2,
    /// Light Account
    LightAccount,
    /// Multi-owner Light Account
    MultiOwnerLightAccount,
}

/// Key type for sessions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum KeyType {
    Secp256k1,
    Ecdsa,
    Contract,
}

/// WebAuthn public key
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnPublicKey {
    /// X coordinate (hex)
    pub x: String,
    /// Y coordinate (hex)
    pub y: String,
    /// Key type
    #[serde(rename = "type")]
    pub key_type: String,
}

/// Account creation hint
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreationHint {
    /// Account type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_type: Option<AccountType>,
    /// Salt for account creation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub salt: Option<String>,
}

/// Request account parameters
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestAccountParams {
    /// Signer address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_address: Option<String>,
    /// WebAuthn public key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signer_public_key: Option<WebAuthnPublicKey>,
    /// Existing account address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub account_address: Option<String>,
    /// Account ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Creation hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub creation_hint: Option<CreationHint>,
    /// Include counterfactual info
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_counterfactual_info: Option<bool>,
}

/// Account response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountResponse {
    /// Account address
    pub address: String,
    /// Account ID
    pub id: Option<String>,
    /// Counterfactual info
    pub counterfactual: Option<CounterfactualInfo>,
}

/// Counterfactual deployment info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CounterfactualInfo {
    /// Factory address
    pub factory_address: String,
    /// Factory data
    pub factory_data: String,
    /// Whether already deployed
    pub is_deployed: bool,
}

/// Call for wallet_prepareCalls
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletCall {
    /// Target address
    pub to: String,
    /// Call data (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// Value (hex)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// Capabilities for prepare calls
#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareCallsCapabilities {
    /// Paymaster service
    #[serde(skip_serializing_if = "Option::is_none")]
    pub paymaster_service: Option<PaymasterServiceCapability>,
    /// Gas overrides
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gas_overrides: Option<serde_json::Value>,
}

/// Paymaster service capability
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymasterServiceCapability {
    /// Policy ID
    pub policy_id: String,
}

/// Prepare calls request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PrepareCallsRequest {
    /// Calls to prepare
    pub calls: Vec<WalletCall>,
    /// Account address
    pub from: String,
    /// Chain ID (hex)
    pub chain_id: String,
    /// Capabilities
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<PrepareCallsCapabilities>,
}

/// Prepared calls response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreparedCallsResponse {
    /// Prepared call ID
    pub prepared_call_id: String,
    /// User operations
    pub user_operations: Vec<serde_json::Value>,
    /// Signature requests
    pub signature_requests: Vec<SignatureRequest>,
    /// Fee info
    pub fee_info: Option<FeeInfo>,
}

/// Signature request
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureRequest {
    /// Type of signature
    #[serde(rename = "type")]
    pub sig_type: String,
    /// Data to sign
    pub data: serde_json::Value,
}

/// Fee info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeInfo {
    /// Estimated gas (wei)
    pub estimated_gas: String,
    /// Estimated gas in USD
    pub estimated_gas_usd: Option<String>,
}

/// Send prepared calls request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPreparedCallsRequest {
    /// Prepared call ID
    pub prepared_call_id: String,
    /// Signatures
    pub signatures: Vec<Signature>,
}

/// Signature
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    /// Signature type
    #[serde(rename = "type")]
    pub sig_type: String,
    /// Signature data (hex)
    pub data: String,
}

/// Send prepared calls response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPreparedCallsResponse {
    /// Call ID
    pub call_id: String,
    /// User operation hashes
    pub user_op_hashes: Vec<String>,
}

/// Calls status response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallsStatusResponse {
    /// Status code (100-600)
    pub status: u16,
    /// Status message
    pub message: Option<String>,
    /// Transaction receipts
    pub receipts: Vec<serde_json::Value>,
}

/// Wallet capabilities
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WalletCapabilities {
    /// Atomic batch support
    pub atomic_batch: Option<AtomicBatchCapability>,
    /// Paymaster support
    pub paymaster: Option<PaymasterCapability>,
}

/// Atomic batch capability
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AtomicBatchCapability {
    pub supported: bool,
}

/// Paymaster capability
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PaymasterCapability {
    pub supported: bool,
}

/// List accounts response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListAccountsResponse {
    pub accounts: Vec<AccountInfo>,
    pub total_count: u64,
    pub cursor: Option<String>,
}

/// Account info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub address: String,
    pub id: String,
}

/// Session permission
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionPermission {
    /// Permission type
    #[serde(rename = "type")]
    pub permission_type: String,
    /// Permission data
    pub data: serde_json::Value,
}

/// Create session request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionRequest {
    /// Account address
    pub account: String,
    /// Chain ID (hex)
    pub chain_id: String,
    /// Session key
    pub key: SessionKey,
    /// Permissions
    pub permissions: Vec<SessionPermission>,
    /// Expiry (unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiry_sec: Option<u64>,
}

/// Session key
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionKey {
    /// Key type
    #[serde(rename = "type")]
    pub key_type: KeyType,
    /// Public key (hex)
    pub public_key: String,
}

/// Create session response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSessionResponse {
    /// Session ID
    pub session_id: String,
    /// Chain ID
    pub chain_id: String,
    /// Signature request
    pub signature_request: SignatureRequest,
}
