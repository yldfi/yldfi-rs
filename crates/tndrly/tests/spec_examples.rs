//! Deserializes the example responses in Tenderly's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://docs.tenderly.co/api-reference/openapi.json
//! crates/tndrly/tests/spec_examples/tenderly.json`.
//!
//! `getSimulatedTransactionInfoById` is checked against the type
//! `SimulationApi::get_info` reads (`SimulationApi::info` returns untyped
//! JSON), and `sendTestAlert` against the type `AlertsApi::send_test_alert`
//! reads.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/tenderly.json",
    checks: {
        "addContractToProject" => tndrly::contracts::Contract,
        "addWalletsToProject" => tndrly::wallets::AddWalletResponse,
        "bulkAddWalletsToProject" => tndrly::wallets::BulkAddWalletsResponse,
        "checkSupportedNetworks" => Vec<tndrly::networks::Network>,
        "createAlert" => tndrly::alerts::AlertResponse,
        "createEnvironment" => tndrly::environments::Environment,
        "createProject" => tndrly::projects::ProjectResponse,
        "createSimulationDraft" => tndrly::simulation::SimulationDraftCreated,
        "createWeb3Action" => tndrly::actions::PublishActionResponse,
        "encodeContractStateOverrides" => tndrly::contracts::EncodeStateResponse,
        "getAlertHistory" => tndrly::alerts::AlertHistoryResponse,
        "getContractsForProject" => Vec<tndrly::contracts::Contract>,
        "getContractsForProject" => Vec<tndrly::wallets::WalletOnNetwork>,
        "getEnvironment" => tndrly::environments::Environment,
        "getEnvironmentInstanceTransaction" => tndrly::vnets::VNetTransaction,
        "getEnvironmentInstanceTransactions" => Vec<tndrly::vnets::VNetTransaction>,
        "getProject" => tndrly::projects::ProjectResponse,
        "getSimulatedTransactionById" => tndrly::simulation::SimulationResponse,
        "getSimulatedTransactionInfoById" => tndrly::simulation::SimulationInfoResponse,
        "getSimulatedTransactions" => tndrly::simulation::SimulationListResponse,
        "getSimulationDraftById" => tndrly::simulation::SimulationDraft,
        "getWalletForProject" => tndrly::wallets::WalletOnNetwork,
        "getWeb3ActionExecutionForProject" => tndrly::actions::ActionCall,
        "getWeb3ActionExecutionsForProject" => Option<Vec<tndrly::actions::ActionCall>>,
        "getWeb3ActionForProject" => tndrly::actions::Action,
        "getWeb3ActionsForProject" => Option<Vec<tndrly::actions::Action>>,
        "listAlerts" => tndrly::alerts::ListAlertsResponse,
        "listDeliveryChannels" => tndrly::delivery_channels::ListDeliveryChannelsResponse,
        "listDeliveryChannelsForProject" => tndrly::delivery_channels::ListDeliveryChannelsResponse,
        "listEnvironments" => Vec<tndrly::environments::Environment>,
        "listProjects" => tndrly::projects::ListProjectsResponse,
        "sendEnvironmentVnetTransaction" => tndrly::vnets::VNetTransaction,
        "sendTestAlert" => tndrly::alerts::TestAlertResponse,
        "simulateEnvironmentVnetTransaction" => tndrly::environments::EnvironmentSimulation,
        "simulateTransaction" => tndrly::simulation::SimulationResponse,
        "simulateTransactionBundle" => tndrly::simulation::BundleSimulationResponse,
        "tagContracts" => tndrly::contracts::BulkTagResponse,
        "updateAlert" => tndrly::alerts::AlertResponse,
        "updateProject" => tndrly::projects::ProjectResponse,
    },
    not_implemented: [],
}
