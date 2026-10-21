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
        #[allow(deprecated)]
        Network::EthMainnet | Network::EthSepolia | Network::EthHoodi | Network::EthHolesky => {
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
        self.get_with_query(version, path, &[]).await
    }

    /// Build a GET request with optional query parameters
    fn build_get(
        &self,
        version: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<reqwest::Request> {
        let url = beacon_url(self.client.network(), self.client.api_key(), version, path)?;
        let mut request = self.client.http().get(&url);
        if !query.is_empty() {
            request = request.query(query);
        }
        request.build().map_err(crate::error::http)
    }

    async fn get_with_query<R>(
        &self,
        version: &str,
        path: &str,
        query: &[(&str, String)],
    ) -> Result<R>
    where
        R: serde::de::DeserializeOwned,
    {
        let request = self.build_get(version, path, query)?;
        let response = self
            .client
            .http()
            .execute(request)
            .await
            .map_err(crate::error::http)?;
        self.client.handle_response(response).await
    }

    async fn post<B, R>(&self, path: &str, body: &B) -> Result<R>
    where
        B: serde::Serialize,
        R: serde::de::DeserializeOwned,
    {
        let url = beacon_url(self.client.network(), self.client.api_key(), "v1", path)?;
        let response = self
            .client
            .http()
            .post(&url)
            .json(body)
            .send()
            .await
            .map_err(crate::error::http)?;
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
    ///
    /// Beacon API v4.0.0 deprecated `blob_sidecars` in favour of
    /// [`Self::get_blobs`].
    #[deprecated(
        since = "0.1.5",
        note = "Beacon API v4 deprecated /beacon/blob_sidecars; use get_blobs()"
    )]
    pub async fn get_blob_sidecars(
        &self,
        block_id: &str,
    ) -> Result<BeaconListResponse<BlobSidecar>> {
        self.get(&format!("/beacon/blob_sidecars/{block_id}")).await
    }

    /// Get the blobs in a block (`GET /eth/v1/beacon/blobs/{block_id}`)
    ///
    /// Each entry is a hex-encoded blob. Replaces the deprecated
    /// `blob_sidecars` endpoint (Beacon API v4.0.0).
    pub async fn get_blobs(&self, block_id: &str) -> Result<BeaconListResponse<String>> {
        self.get(&format!("/beacon/blobs/{block_id}")).await
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

    /// Get validators, filtered by `ids` (indices or pubkeys) and/or
    /// `statuses` (e.g. `active_ongoing`, `pending_queued`, `exited`)
    ///
    /// With both filters empty the node returns every validator (over a
    /// million on mainnet, hundreds of MB), so pass at least one filter
    /// unless that is really what you want.
    pub async fn get_validators(
        &self,
        state_id: &str,
        ids: &[&str],
        statuses: &[&str],
    ) -> Result<BeaconListResponse<ValidatorInfo>> {
        self.get_with_query(
            "v1",
            &format!("/beacon/states/{state_id}/validators"),
            &validators_query(ids, statuses),
        )
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

    /// Get validator balances, filtered by `ids` (indices or pubkeys)
    ///
    /// An empty `ids` returns the balance of every validator.
    pub async fn get_validator_balances(
        &self,
        state_id: &str,
        ids: &[&str],
    ) -> Result<BeaconListResponse<ValidatorBalance>> {
        self.get_with_query(
            "v1",
            &format!("/beacon/states/{state_id}/validator_balances"),
            &validators_query(ids, &[]),
        )
        .await
    }

    /// Get sync committees, optionally for a specific `epoch`
    pub async fn get_sync_committees(
        &self,
        state_id: &str,
        epoch: Option<u64>,
    ) -> Result<BeaconResponse<SyncCommittee>> {
        self.get_with_query(
            "v1",
            &format!("/beacon/states/{state_id}/sync_committees"),
            &epoch_query(epoch),
        )
        .await
    }

    /// Get RANDAO, optionally for a specific `epoch` (defaults to the
    /// state's epoch)
    pub async fn get_randao(
        &self,
        state_id: &str,
        epoch: Option<u64>,
    ) -> Result<BeaconResponse<RandaoResponse>> {
        self.get_with_query(
            "v1",
            &format!("/beacon/states/{state_id}/randao"),
            &epoch_query(epoch),
        )
        .await
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

/// `id` / `status` query pairs (comma-separated, as the Beacon API accepts)
fn validators_query(ids: &[&str], statuses: &[&str]) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    if !ids.is_empty() {
        query.push(("id", ids.join(",")));
    }
    if !statuses.is_empty() {
        query.push(("status", statuses.join(",")));
    }
    query
}

/// Optional `epoch` query pair
fn epoch_query(epoch: Option<u64>) -> Vec<(&'static str, String)> {
    epoch
        .map(|e| ("epoch", e.to_string()))
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validators_request_carries_filters() {
        let client = Client::new("KEY", Network::EthMainnet).unwrap();
        let api = BeaconApi::new(&client);
        let req = api
            .build_get(
                "v1",
                "/beacon/states/head/validators",
                &validators_query(&["1", "0xabc"], &["active_ongoing"]),
            )
            .unwrap();
        assert_eq!(
            req.url().as_str(),
            "https://eth-mainnetbeacon.g.alchemy.com/v2/KEY/eth/v1/beacon/states/head/validators?id=1%2C0xabc&status=active_ongoing"
        );
        assert!(validators_query(&[], &[]).is_empty());
    }

    #[test]
    fn randao_epoch_query() {
        assert_eq!(epoch_query(Some(42)), vec![("epoch", "42".to_string())]);
        assert!(epoch_query(None).is_empty());
    }

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
            beacon_url(Network::EthHoodi, "KEY", "v1", "/node/version").unwrap(),
            "https://eth-hoodibeacon.g.alchemy.com/v2/KEY/eth/v1/node/version"
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
