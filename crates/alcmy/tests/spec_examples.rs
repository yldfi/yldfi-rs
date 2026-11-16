//! Deserializes the example responses in Alchemy's OpenAPI specs (beacon,
//! NFT and prices), plus responses built from each endpoint's schema, into
//! the type this crate reads that endpoint's response as, so a type that no
//! longer matches the spec fails here instead of at runtime.
//!
//! Refresh a fixture with `python3 scripts/spec-check/examples.py
//! https://raw.githubusercontent.com/alchemyplatform/docs/main/src/openapi/<api>/<api>.yaml
//! crates/alcmy/tests/spec_examples/alchemy-<api>.json` (the beacon spec is
//! `eth-beacon/eth-beacon.yaml`, written to `alchemy-beacon.json`).
//!
//! `OwnedBaseNftsResponse` (getNFTsForOwner with `withMetadata=false`) is not
//! checked: the spec describes only the full response. The portfolio spec has
//! no response schemas, so it has no fixture.

mod beacon {
    yldfi_spec_test::spec_tests! {
        file: "spec_examples/alchemy-beacon.json",
        checks: {
            "GET /eth/v1/beacon/blob_sidecars/{block_id}" => alcmy::beacon::BeaconListResponse<alcmy::beacon::BlobSidecar>,
            "GET /eth/v1/beacon/blocks/{block_id}/root" => alcmy::beacon::BeaconResponse<alcmy::beacon::RootResponse>,
            "GET /eth/v1/beacon/genesis" => alcmy::beacon::BeaconResponse<alcmy::beacon::GenesisInfo>,
            "GET /eth/v1/beacon/headers" => alcmy::beacon::BeaconListResponse<alcmy::beacon::BlockHeaderResponse>,
            "GET /eth/v1/beacon/headers/{block_id}" => alcmy::beacon::BeaconResponse<alcmy::beacon::BlockHeaderResponse>,
            "GET /eth/v1/beacon/pool/voluntary_exits" => alcmy::beacon::BeaconListResponse<alcmy::beacon::SignedVoluntaryExit>,
            "GET /eth/v1/beacon/rewards/blocks/{block_id}" => alcmy::beacon::BeaconResponse<alcmy::beacon::BlockReward>,
            "GET /eth/v1/beacon/states/{state_id}/finality_checkpoints" => alcmy::beacon::BeaconResponse<alcmy::beacon::FinalityCheckpoints>,
            "GET /eth/v1/beacon/states/{state_id}/fork" => alcmy::beacon::BeaconResponse<alcmy::beacon::ForkInfo>,
            "GET /eth/v1/beacon/states/{state_id}/randao" => alcmy::beacon::BeaconResponse<alcmy::beacon::RandaoResponse>,
            "GET /eth/v1/beacon/states/{state_id}/root" => alcmy::beacon::BeaconResponse<alcmy::beacon::RootResponse>,
            "GET /eth/v1/beacon/states/{state_id}/sync_committees" => alcmy::beacon::BeaconResponse<alcmy::beacon::SyncCommittee>,
            "GET /eth/v1/beacon/states/{state_id}/validator_balances" => alcmy::beacon::BeaconListResponse<alcmy::beacon::ValidatorBalance>,
            "GET /eth/v1/beacon/states/{state_id}/validators" => alcmy::beacon::BeaconListResponse<alcmy::beacon::ValidatorInfo>,
            "GET /eth/v1/beacon/states/{state_id}/validators/{validator_id}" => alcmy::beacon::BeaconResponse<alcmy::beacon::ValidatorInfo>,
            "GET /eth/v1/config/deposit_contract" => alcmy::beacon::BeaconResponse<alcmy::beacon::DepositContract>,
            "GET /eth/v1/config/fork_schedule" => alcmy::beacon::BeaconListResponse<alcmy::beacon::ForkScheduleEntry>,
            "GET /eth/v1/config/spec" => alcmy::beacon::BeaconResponse<serde_json::Value>,
            "GET /eth/v1/node/peer_count" => alcmy::beacon::BeaconResponse<alcmy::beacon::PeerCount>,
            "GET /eth/v1/node/peers" => alcmy::beacon::BeaconListResponse<alcmy::beacon::PeerInfo>,
            "GET /eth/v1/node/syncing" => alcmy::beacon::BeaconResponse<alcmy::beacon::SyncStatus>,
            "GET /eth/v1/node/version" => alcmy::beacon::BeaconResponse<alcmy::beacon::NodeVersion>,
            "GET /eth/v1/validator/duties/proposer/{epoch}" => alcmy::beacon::BeaconListResponse<alcmy::beacon::ProposerDuty>,
            "GET /eth/v2/beacon/blocks/{block_id}" => alcmy::beacon::BeaconResponse<serde_json::Value>,
            "GET /eth/v2/beacon/blocks/{block_id}/attestations" => alcmy::beacon::BeaconListResponse<alcmy::beacon::Attestation>,
            "GET /eth/v2/beacon/pool/attestations" => alcmy::beacon::BeaconListResponse<alcmy::beacon::Attestation>,
            "POST /eth/v1/validator/duties/attester/{epoch}" => alcmy::beacon::BeaconListResponse<alcmy::beacon::AttesterDuty>,
            "POST /eth/v1/validator/duties/sync/{epoch}" => alcmy::beacon::BeaconListResponse<alcmy::beacon::SyncDuty>,
        },
        not_implemented: [
            "GET /eth/v1/beacon/blinded_blocks/{block_id}",
            "GET /eth/v1/beacon/states/{state_id}/committees",
            "GET /eth/v1/beacon/states/{state_id}/pending_consolidations",
            "GET /eth/v1/debug/fork_choice",
            "GET /eth/v1/validator/sync_committee_contribution",
            "GET /eth/v2/debug/beacon/states/{state_id}",
            "GET /eth/v2/validator/aggregate_attestation",
            "POST /eth/v1/beacon/rewards/attestations/{epoch}",
            "POST /eth/v1/beacon/rewards/sync_committee/{block_id}",
        ],
    }
}

mod nft {
    yldfi_spec_test::spec_tests! {
        file: "spec_examples/alchemy-nft.json",
        checks: {
            "getContractMetadata-v3" => alcmy::nft::ContractMetadataResponse,
            "getContractMetadataBatch-v3" => alcmy::nft::ContractMetadataBatchResponse,
            "getContractsForOwner-v3" => alcmy::nft::ContractsForOwnerResponse,
            "getFloorPrice-v3" => alcmy::nft::FloorPriceResponse,
            "getNFTMetadata-v3" => alcmy::nft::Nft,
            "getNFTMetadataBatch-v3" => Vec<alcmy::nft::Nft>,
            "getNFTsForCollection-v3" => alcmy::nft::NftsForCollectionResponse,
            "getNFTsForContract-v3" => alcmy::nft::NftsForContractResponse,
                "getNFTsForOwner-v3" => alcmy::nft::OwnedNftsResponse,
            "getOwnersForContract-v3" => alcmy::nft::OwnersForContractResponse,
            "getOwnersForNFT-v3" => alcmy::nft::OwnersForNftResponse,
            "isSpamContract-v3" => alcmy::nft::IsSpamResponse,
            "refreshNftMetadata-v3" => alcmy::nft::RefreshMetadataResponse,
        },
        not_implemented: [
            "reportSpam-v3",
        ],
    }
}

mod prices {
    yldfi_spec_test::spec_tests! {
        file: "spec_examples/alchemy-prices.json",
        checks: {
            "get-historical-token-prices" => alcmy::prices::HistoricalPriceResponse,
            "get-token-prices-by-address" => alcmy::prices::TokenPricesByAddressResponse,
            "get-token-prices-by-symbol" => alcmy::prices::TokenPricesBySymbolResponse,
        },
        not_implemented: [
        ],
    }
}
