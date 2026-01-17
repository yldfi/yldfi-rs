//! Types for the Beacon API (Ethereum consensus layer)

use serde::{Deserialize, Serialize};

/// Genesis info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GenesisInfo {
    pub genesis_time: String,
    pub genesis_validators_root: String,
    pub genesis_fork_version: String,
}

/// Block header
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeader {
    pub slot: String,
    pub proposer_index: String,
    pub parent_root: String,
    pub state_root: String,
    pub body_root: String,
}

/// Signed block header
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SignedBlockHeader {
    pub message: BlockHeader,
    pub signature: String,
}

/// Block header response
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockHeaderResponse {
    pub root: String,
    pub canonical: bool,
    pub header: SignedBlockHeader,
}

/// Fork info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ForkInfo {
    pub previous_version: String,
    pub current_version: String,
    pub epoch: String,
}

/// Finality checkpoints
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct FinalityCheckpoints {
    pub previous_justified: Checkpoint,
    pub current_justified: Checkpoint,
    pub finalized: Checkpoint,
}

/// Checkpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Checkpoint {
    pub epoch: String,
    pub root: String,
}

/// Validator info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorInfo {
    pub index: String,
    pub balance: String,
    pub status: String,
    pub validator: ValidatorData,
}

/// Validator data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ValidatorData {
    pub pubkey: String,
    pub withdrawal_credentials: String,
    pub effective_balance: String,
    pub slashed: bool,
    pub activation_eligibility_epoch: String,
    pub activation_epoch: String,
    pub exit_epoch: String,
    pub withdrawable_epoch: String,
}

/// Sync committee info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncCommittee {
    pub validators: Vec<String>,
    pub validator_aggregates: Vec<Vec<String>>,
}

/// Attestation
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Attestation {
    pub aggregation_bits: String,
    pub data: AttestationData,
    pub signature: String,
}

/// Attestation data
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AttestationData {
    pub slot: String,
    pub index: String,
    pub beacon_block_root: String,
    pub source: Checkpoint,
    pub target: Checkpoint,
}

/// Voluntary exit
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct VoluntaryExit {
    pub epoch: String,
    pub validator_index: String,
}

/// Signed voluntary exit
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: String,
}

/// Block reward
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BlockReward {
    pub proposer_index: String,
    pub total: String,
    pub attestations: String,
    pub sync_aggregate: String,
    pub proposer_slashings: String,
    pub attester_slashings: String,
}

/// Sync status
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncStatus {
    pub head_slot: String,
    pub sync_distance: String,
    pub is_syncing: bool,
    pub is_optimistic: Option<bool>,
    pub el_offline: Option<bool>,
}

/// Node version
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct NodeVersion {
    pub version: String,
}

/// Peer info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PeerInfo {
    pub peer_id: String,
    pub enr: Option<String>,
    pub last_seen_p2p_address: Option<String>,
    pub state: String,
    pub direction: String,
}

/// Peer count
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct PeerCount {
    pub disconnected: String,
    pub connecting: String,
    pub connected: String,
    pub disconnecting: String,
}

/// Deposit contract info
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct DepositContract {
    pub chain_id: String,
    pub address: String,
}

/// Fork schedule
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ForkScheduleEntry {
    pub previous_version: String,
    pub current_version: String,
    pub epoch: String,
}

/// Blob sidecar
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct BlobSidecar {
    pub index: String,
    pub blob: String,
    pub kzg_commitment: String,
    pub kzg_proof: String,
    pub signed_block_header: SignedBlockHeader,
    pub kzg_commitment_inclusion_proof: Vec<String>,
}

/// Attester duty
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct AttesterDuty {
    pub pubkey: String,
    pub validator_index: String,
    pub committee_index: String,
    pub committee_length: String,
    pub committees_at_slot: String,
    pub validator_committee_index: String,
    pub slot: String,
}

/// Proposer duty
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ProposerDuty {
    pub pubkey: String,
    pub validator_index: String,
    pub slot: String,
}

/// Sync duty
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct SyncDuty {
    pub pubkey: String,
    pub validator_index: String,
    pub validator_sync_committee_indices: Vec<String>,
}

/// Beacon API response wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeaconResponse<T> {
    pub data: T,
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
}

/// Beacon API list response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BeaconListResponse<T> {
    pub data: Vec<T>,
    pub execution_optimistic: Option<bool>,
    pub finalized: Option<bool>,
}
