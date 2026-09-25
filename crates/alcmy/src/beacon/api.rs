//! Beacon API implementation

use serde::{Deserialize, Serialize};

use super::types::{
    Attestation, AttesterDuty, BeaconListResponse, BeaconResponse, BlobSidecar,
    BlockHeaderResponse, BlockReward, DepositContract, FinalityCheckpoints, ForkInfo,
    ForkScheduleEntry, GenesisInfo, NodeVersion, PeerCount, PeerInfo, ProposerDuty,
    SignedVoluntaryExit, SyncCommittee, SyncDuty, SyncStatus, ValidatorInfo,
};
use crate::client::{Client, Network};
use crate::error::{DomainError, Error, Result};

/// Get the Beacon (consensus layer) host for a network.
///
/// Alchemy serves the Beacon API from dedicated `{network}beacon` hosts,
/// e.g. `https://eth-mainnetbeacon.g.alchemy.com/v2/{apiKey}/eth/v1/beacon/genesis`.
/// Only Ethereum mainnet and its testnets have beacon hosts.
fn beacon_host(network: Network) -> Result<String> {
    match network {
        Network::EthMainnet | Network::EthSepolia | Network::EthHolesky => {
            Ok(format!("{}beacon.g.alchemy.com", network.slug()))
        }
        other => Err(Error::domain(DomainError::UnsupportedBeaconNetwork(
            other.slug(),
        ))),
    }
}

/// Build a full Beacon API URL.
///
/// `version` is the Beacon API version segment (`v1` / `v2`) and `path` is
/// the endpoint path after `/eth/{version}` (e.g. `/beacon/genesis`).
fn beacon_url(network: Network, api_key: &str, version: &str, path: &str) -> Result<String> {
    Ok(format!(
        "https://{}/v2/{}/eth/{}{}",
        beacon_host(network)?,
        api_key,
        version,
        path
    ))
}

/// Beacon API for Ethereum consensus layer
pub struct BeaconApi<'a> {
    client: &'a Client,
}

impl<'a> BeaconApi<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    async fn get<R>(&self, path: &str) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        self.get_versioned("v1", path).await
    }

    async fn get_versioned<R>(&self, version: &str, path: &str) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let url = beacon_url(self.client.network(), self.client.api_key(), version, path)?;
        let response = self.client.http().get(&url).send().await?;
        self.client.handle_response(response).await
    }

    async fn post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = beacon_url(self.client.network(), self.client.api_key(), "v1", path)?;
        let response = self.client.http().post(&url).json(body).send().await?;
        self.client.handle_response(response).await
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
        self.get_versioned("v2", &format!("/beacon/blocks/{block_id}"))
            .await
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
        self.get_versioned("v2", &format!("/beacon/blocks/{block_id}/attestations"))
            .await
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
        self.get_versioned("v2", "/beacon/pool/attestations").await
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn beacon_url_mainnet_matches_docs() {
        assert_eq!(
            beacon_url(Network::EthMainnet, "KEY", "v1", "/beacon/genesis").unwrap(),
            "https://eth-mainnetbeacon.g.alchemy.com/v2/KEY/eth/v1/beacon/genesis"
        );
    }

    #[test]
    fn beacon_url_v2_and_testnets() {
        assert_eq!(
            beacon_url(Network::EthSepolia, "KEY", "v2", "/beacon/blocks/head").unwrap(),
            "https://eth-sepoliabeacon.g.alchemy.com/v2/KEY/eth/v2/beacon/blocks/head"
        );
        assert_eq!(
            beacon_url(Network::EthHolesky, "KEY", "v1", "/node/version").unwrap(),
            "https://eth-holeskybeacon.g.alchemy.com/v2/KEY/eth/v1/node/version"
        );
    }

    #[test]
    fn beacon_url_rejects_non_ethereum_networks() {
        let err = beacon_url(Network::BaseMainnet, "KEY", "v1", "/beacon/genesis").unwrap_err();
        assert!(matches!(
            err,
            Error::Domain(DomainError::UnsupportedBeaconNetwork("base-mainnet"))
        ));
    }
}
