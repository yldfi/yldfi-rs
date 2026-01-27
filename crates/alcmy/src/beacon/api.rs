//! Beacon API implementation

use serde::{Deserialize, Serialize};

use super::types::{
    Attestation, AttesterDuty, BeaconListResponse, BeaconResponse, BlobSidecar,
    BlockHeaderResponse, BlockReward, DepositContract, FinalityCheckpoints, ForkInfo,
    ForkScheduleEntry, GenesisInfo, NodeVersion, PeerCount, PeerInfo, ProposerDuty,
    SignedVoluntaryExit, SyncCommittee, SyncDuty, SyncStatus, ValidatorInfo,
};
use crate::client::Client;
use crate::error::{Error, Result};

/// Beacon API for Ethereum consensus layer
pub struct BeaconApi<'a> {
    client: &'a Client,
}

impl<'a> BeaconApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    fn beacon_url(&self) -> String {
        format!(
            "https://{}.g.alchemy.com/eth/v1",
            self.client.network().slug()
        )
    }

    async fn get<R>(&self, path: &str) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.beacon_url(), path);
        let response = self.client.http().get(&url).send().await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    async fn post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = format!("{}{}", self.beacon_url(), path);
        let response = self.client.http().post(&url).json(body).send().await?;

        if response.status() == 429 {
            return Err(Error::rate_limited(None));
        }

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    // ========== Genesis & Config ==========

    /// Get genesis info
    pub async fn get_genesis(&self) -> Result<BeaconResponse<GenesisInfo>> {
        self.get("/beacon/genesis").await
    }

    /// Get fork schedule
    pub async fn get_fork_schedule(&self) -> Result<BeaconListResponse<ForkScheduleEntry>> {
        self.get("/config/fork_schedule").await
    }

    /// Get deposit contract info
    pub async fn get_deposit_contract(&self) -> Result<BeaconResponse<DepositContract>> {
        self.get("/config/deposit_contract").await
    }

    /// Get spec/config values
    pub async fn get_spec(&self) -> Result<BeaconResponse<serde_json::Value>> {
        self.get("/config/spec").await
    }

    // ========== Blocks & Headers ==========

    /// Get block headers
    pub async fn get_headers(&self) -> Result<BeaconListResponse<BlockHeaderResponse>> {
        self.get("/beacon/headers").await
    }

    /// Get block header by ID
    pub async fn get_header(&self, block_id: &str) -> Result<BeaconResponse<BlockHeaderResponse>> {
        self.get(&format!("/beacon/headers/{block_id}")).await
    }

    /// Get block by ID
    pub async fn get_block(&self, block_id: &str) -> Result<BeaconResponse<serde_json::Value>> {
        // Use v2 endpoint for full block
        let url = format!(
            "https://{}.g.alchemy.com/eth/v2/beacon/blocks/{}",
            self.client.network().slug(),
            block_id
        );
        let response = self.client.http().get(&url).send().await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get block root
    pub async fn get_block_root(&self, block_id: &str) -> Result<BeaconResponse<RootResponse>> {
        self.get(&format!("/beacon/blocks/{block_id}/root")).await
    }

    /// Get block attestations
    pub async fn get_block_attestations(
        &self,
        block_id: &str,
    ) -> Result<BeaconListResponse<Attestation>> {
        let url = format!(
            "https://{}.g.alchemy.com/eth/v2/beacon/blocks/{}/attestations",
            self.client.network().slug(),
            block_id
        );
        let response = self.client.http().get(&url).send().await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get blob sidecars
    pub async fn get_blob_sidecars(
        &self,
        block_id: &str,
    ) -> Result<BeaconListResponse<BlobSidecar>> {
        self.get(&format!("/beacon/blob_sidecars/{block_id}")).await
    }

    // ========== State ==========

    /// Get state root
    pub async fn get_state_root(&self, state_id: &str) -> Result<BeaconResponse<RootResponse>> {
        self.get(&format!("/beacon/states/{state_id}/root")).await
    }

    /// Get fork info for state
    pub async fn get_state_fork(&self, state_id: &str) -> Result<BeaconResponse<ForkInfo>> {
        self.get(&format!("/beacon/states/{state_id}/fork")).await
    }

    /// Get finality checkpoints
    pub async fn get_finality_checkpoints(
        &self,
        state_id: &str,
    ) -> Result<BeaconResponse<FinalityCheckpoints>> {
        self.get(&format!("/beacon/states/{state_id}/finality_checkpoints"))
            .await
    }

    /// Get validators
    pub async fn get_validators(
        &self,
        state_id: &str,
    ) -> Result<BeaconListResponse<ValidatorInfo>> {
        self.get(&format!("/beacon/states/{state_id}/validators"))
            .await
    }

    /// Get specific validator
    pub async fn get_validator(
        &self,
        state_id: &str,
        validator_id: &str,
    ) -> Result<BeaconResponse<ValidatorInfo>> {
        self.get(&format!(
            "/beacon/states/{state_id}/validators/{validator_id}"
        ))
        .await
    }

    /// Get validator balances
    pub async fn get_validator_balances(
        &self,
        state_id: &str,
    ) -> Result<BeaconListResponse<ValidatorBalance>> {
        self.get(&format!("/beacon/states/{state_id}/validator_balances"))
            .await
    }

    /// Get sync committees
    pub async fn get_sync_committees(
        &self,
        state_id: &str,
    ) -> Result<BeaconResponse<SyncCommittee>> {
        self.get(&format!("/beacon/states/{state_id}/sync_committees"))
            .await
    }

    /// Get RANDAO
    pub async fn get_randao(&self, state_id: &str) -> Result<BeaconResponse<RandaoResponse>> {
        self.get(&format!("/beacon/states/{state_id}/randao")).await
    }

    // ========== Pool ==========

    /// Get pool attestations
    pub async fn get_pool_attestations(&self) -> Result<BeaconListResponse<Attestation>> {
        let url = format!(
            "https://{}.g.alchemy.com/eth/v2/beacon/pool/attestations",
            self.client.network().slug()
        );
        let response = self.client.http().get(&url).send().await?;
        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            let status = response.status().as_u16();
            let message = response.text().await.unwrap_or_default();
            Err(Error::api(status, message))
        }
    }

    /// Get voluntary exits
    pub async fn get_voluntary_exits(&self) -> Result<BeaconListResponse<SignedVoluntaryExit>> {
        self.get("/beacon/pool/voluntary_exits").await
    }

    // ========== Rewards ==========

    /// Get block rewards
    pub async fn get_block_rewards(&self, block_id: &str) -> Result<BeaconResponse<BlockReward>> {
        self.get(&format!("/beacon/rewards/blocks/{block_id}"))
            .await
    }

    // ========== Node ==========

    /// Get sync status
    pub async fn get_syncing(&self) -> Result<BeaconResponse<SyncStatus>> {
        self.get("/node/syncing").await
    }

    /// Get node version
    pub async fn get_version(&self) -> Result<BeaconResponse<NodeVersion>> {
        self.get("/node/version").await
    }

    /// Get peers
    pub async fn get_peers(&self) -> Result<BeaconListResponse<PeerInfo>> {
        self.get("/node/peers").await
    }

    /// Get peer count
    pub async fn get_peer_count(&self) -> Result<BeaconResponse<PeerCount>> {
        self.get("/node/peer_count").await
    }

    // ========== Validator Duties ==========

    /// Get attester duties
    pub async fn get_attester_duties(
        &self,
        epoch: &str,
        validator_indices: &[&str],
    ) -> Result<BeaconListResponse<AttesterDuty>> {
        self.post(
            &format!("/validator/duties/attester/{epoch}"),
            &validator_indices,
        )
        .await
    }

    /// Get proposer duties
    pub async fn get_proposer_duties(
        &self,
        epoch: &str,
    ) -> Result<BeaconListResponse<ProposerDuty>> {
        self.get(&format!("/validator/duties/proposer/{epoch}"))
            .await
    }

    /// Get sync duties
    pub async fn get_sync_duties(
        &self,
        epoch: &str,
        validator_indices: &[&str],
    ) -> Result<BeaconListResponse<SyncDuty>> {
        self.post(
            &format!("/validator/duties/sync/{epoch}"),
            &validator_indices,
        )
        .await
    }
}

/// Root response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RootResponse {
    pub root: String,
}

/// Validator balance
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ValidatorBalance {
    pub index: String,
    pub balance: String,
}

/// RANDAO response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RandaoResponse {
    pub randao: String,
}
