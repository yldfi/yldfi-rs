//! Types for the Contracts API

use serde::{Deserialize, Serialize};

/// One contract to submit for decoding
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct ContractSubmission {
    /// Blockchain, e.g. `ethereum`
    pub blockchain_name: String,
    /// Contract address
    pub address: String,
    /// Project (namespace) name the decoded tables go under
    pub project_name: String,
    /// Contract name
    pub contract_name: String,
    /// Contract ABI: a JSON array, or a string holding the JSON array
    pub abi: serde_json::Value,
    /// `new` (Dune's default), `upgrade`, `rename`, `delete` or `other`;
    /// `upgrade` always goes to manual review
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submission_type: Option<String>,
    /// Makes the item safe to retry: unique per account (at most 128
    /// characters); reusing a key returns the existing submission with
    /// `replayed` set instead of creating a new one
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
    /// Whether the contract is a proxy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_proxy: Option<bool>,
    /// Whether the contract is deployed by a factory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_created_by_factory: Option<bool>,
    /// Whether other contracts share this ABI
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_multiple_instances: Option<bool>,
    /// Whether the ABI was supplied by hand rather than from a verified
    /// source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_manual_abi: Option<bool>,
    /// Current project name, for `rename` submissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_project_name: Option<String>,
    /// Current contract name, for `rename` submissions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_contract_name: Option<String>,
    /// Why the contract is resubmitted
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resubmission_reason: Option<String>,
}

impl ContractSubmission {
    /// A `new` submission of the contract at `address`
    #[must_use]
    pub fn new(
        blockchain_name: impl Into<String>,
        address: impl Into<String>,
        project_name: impl Into<String>,
        contract_name: impl Into<String>,
        abi: serde_json::Value,
    ) -> Self {
        Self {
            blockchain_name: blockchain_name.into(),
            address: address.into(),
            project_name: project_name.into(),
            contract_name: contract_name.into(),
            abi,
            submission_type: None,
            idempotency_key: None,
            is_proxy: None,
            is_created_by_factory: None,
            has_multiple_instances: None,
            is_manual_abi: None,
            old_project_name: None,
            old_contract_name: None,
            resubmission_reason: None,
        }
    }

    /// Set the idempotency key
    #[must_use]
    pub fn idempotency_key(mut self, key: impl Into<String>) -> Self {
        self.idempotency_key = Some(key.into());
        self
    }
}

/// Request to submit contracts for decoding (`POST /v1/contracts/decode`)
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct DecodeContractsRequest {
    /// 1 to 100 contracts
    pub submissions: Vec<ContractSubmission>,
}

impl DecodeContractsRequest {
    /// Submit these contracts
    #[must_use]
    pub fn new(submissions: Vec<ContractSubmission>) -> Self {
        Self { submissions }
    }
}

/// Result for one submitted contract, matched to the request by `index`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DecodeResult {
    /// Index of the item in the request
    pub index: Option<i64>,
    /// Submission ID, when the item was queued
    pub submission_id: Option<String>,
    /// `pending` when the item was queued
    pub status: Option<String>,
    /// Why the item was rejected
    pub error: Option<String>,
    /// `true` when the idempotency key matched an earlier submission;
    /// nothing new was created
    pub replayed: Option<bool>,
}

/// Response from submitting contracts for decoding
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct DecodeContractsResponse {
    /// One result per submitted item
    #[serde(default)]
    pub results: Vec<DecodeResult>,
}

/// A contract decoding submission
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ContractSubmissionStatus {
    /// Submission ID
    pub id: Option<String>,
    /// Blockchain
    pub blockchain_name: Option<String>,
    /// Contract address
    pub address: Option<String>,
    /// Project (namespace) name
    pub project_name: Option<String>,
    /// Contract name
    pub contract_name: Option<String>,
    /// `pending`, `approved`, `rejected`, `processed`, `in_progress`,
    /// `cancelled` or `needs_manual_review`
    pub status: Option<String>,
    /// `new`, `upgrade`, `rename`, `delete` or `other`
    pub submission_type: Option<String>,
    /// Reviewer comment
    pub comment: Option<String>,
    /// Idempotency key the submission was made with
    pub idempotency_key: Option<String>,
    /// Creation timestamp
    pub created_at: Option<String>,
    /// Last update timestamp
    pub updated_at: Option<String>,
}

/// Response from listing contract submissions
#[derive(Debug, Clone, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ListSubmissionsResponse {
    /// Submissions, newest first
    #[serde(default)]
    pub submissions: Vec<ContractSubmissionStatus>,
    /// Total number of matching submissions
    pub total: Option<i64>,
    /// Cursor of the next page; `None` on the last page
    pub next_cursor: Option<String>,
}

/// Options for listing contract submissions
#[derive(Debug, Clone, Default)]
#[non_exhaustive]
pub struct ListSubmissionsOptions {
    /// Only this blockchain, e.g. `ethereum`
    pub blockchain_name: Option<String>,
    /// Only this contract address
    pub address: Option<String>,
    /// Only this project (namespace)
    pub project_name: Option<String>,
    /// Only this contract name
    pub contract_name: Option<String>,
    /// Only this status
    pub status: Option<String>,
    /// Maximum number of results (default 50, max 250)
    pub limit: Option<u32>,
    /// `next_cursor` from the previous page
    pub cursor: Option<String>,
}

impl ListSubmissionsOptions {
    /// Options with nothing set
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Query pairs for the request (values are URL-encoded by the HTTP client)
    #[must_use]
    pub fn to_query_pairs(&self) -> Vec<(&'static str, String)> {
        let mut params = Vec::new();
        let strings = [
            ("blockchain_name", &self.blockchain_name),
            ("address", &self.address),
            ("project_name", &self.project_name),
            ("contract_name", &self.contract_name),
            ("status", &self.status),
        ];
        for (key, value) in strings {
            if let Some(value) = value {
                params.push((key, value.clone()));
            }
        }
        if let Some(limit) = self.limit {
            params.push(("limit", limit.to_string()));
        }
        if let Some(ref cursor) = self.cursor {
            params.push(("cursor", cursor.clone()));
        }
        params
    }
}
