//! Deserializes the example responses in Tenderly's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://docs.tenderly.co/api-reference/openapi.json
//! crates/tndrly/tests/spec_examples/tenderly.json`.
//!
//! Not checked: `getSimulatedTransactionInfoById` (read as untyped JSON) and
//! the environment endpoints (this crate's Virtual TestNet calls use the
//! `/vnets` API instead).

yldfi_spec_test::spec_tests! {
    file: "spec_examples/tenderly.json",
    checks: {
        "addContractToProject" => tndrly::contracts::Contract,
        "addWalletsToProject" => tndrly::wallets::AddWalletResponse,
        "checkSupportedNetworks" => Vec<tndrly::networks::Network>,
        "createAlert" => tndrly::alerts::AlertResponse,
        "createWeb3Action" => tndrly::actions::PublishActionResponse,
        "encodeContractStateOverrides" => tndrly::contracts::EncodeStateResponse,
        "getAlertHistory" => tndrly::alerts::AlertHistoryResponse,
        "getContractsForProject" => Vec<tndrly::contracts::Contract>,
        "getContractsForProject" => Vec<tndrly::wallets::WalletOnNetwork>,
        "getSimulatedTransactionById" => tndrly::simulation::SimulationResponse,
        "getSimulatedTransactions" => tndrly::simulation::SimulationListResponse,
        "getWalletForProject" => tndrly::wallets::WalletOnNetwork,
        "getWeb3ActionExecutionForProject" => tndrly::actions::ActionCall,
        "getWeb3ActionExecutionsForProject" => Option<Vec<tndrly::actions::ActionCall>>,
        "getWeb3ActionForProject" => tndrly::actions::Action,
        "getWeb3ActionsForProject" => Option<Vec<tndrly::actions::Action>>,
        "listAlerts" => tndrly::alerts::ListAlertsResponse,
        "listDeliveryChannels" => tndrly::delivery_channels::ListDeliveryChannelsResponse,
        "listDeliveryChannelsForProject" => tndrly::delivery_channels::ListDeliveryChannelsResponse,
        "simulateTransaction" => tndrly::simulation::SimulationResponse,
        "simulateTransactionBundle" => tndrly::simulation::BundleSimulationResponse,
        "tagContracts" => tndrly::contracts::BulkTagResponse,
        "updateAlert" => tndrly::alerts::AlertResponse,
    },
    not_implemented: [
        "bulkAddWalletsToProject",
        "createEnvironment",
        "createProject",
        "createSimulationDraft",
        "getEnvironment",
        "getEnvironmentInstanceTransaction",
        "getEnvironmentInstanceTransactions",
        "getProject",
        "getSimulatedTransactionInfoById",
        "getSimulationDraftById",
        "listEnvironments",
        "listProjects",
        "sendEnvironmentVnetTransaction",
        "sendTestAlert",
        "simulateEnvironmentVnetTransaction",
        "updateProject",
    ],
}
