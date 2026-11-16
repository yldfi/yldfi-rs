//! Deserializes the example responses in Enso's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://api.enso.finance/api-json
//! crates/ensof/tests/spec_examples/enso.json`.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/enso.json",
    checks: {
        "AccountController_accountId" => String,
        "ActionsController_findAll" => Vec<ensof::Action>,
        "ActionsController_getActionsBySlug" => Vec<ensof::Action>,
        "AggregatorsController_aggregators" => Vec<String>,
        "BundleController_bundleShortcutTransaction" => ensof::BundleResponse,
        "NetworksController_networks" => Vec<ensof::Network>,
        "PricesController_getPrice" => ensof::TokenPrice,
        "PricesController_getPrices" => Vec<Option<ensof::TokenPrice>>,
        "ProjectsController_getStandardsByProject" => Vec<ensof::Protocol>,
        "ProjectsController_standards" => Vec<ensof::Project>,
        "ProtocolsController_findAll" => Vec<ensof::Protocol>,
        "QuoteController_quote" => ensof::QuoteResponse,
        // GET form of the route endpoint; the client uses the POST form,
        // which returns the same schema
        "RouterController_routeShortcutTransaction" => ensof::RouteResponse,
        "RouterController_postRouteShortcutTransaction" => ensof::RouteResponse,
        "StandardsController_getProtocolBySlug" => ensof::Protocol,
        "StandardsController_standards" => Vec<ensof::Standard>,
        "TokensController_tokens" => ensof::TokensPage,
        "WalletController_createApproveTransaction" => ensof::ApproveTransaction,
        "WalletController_wallet" => ensof::Wallet,
        "WalletController_walletBalances" => Vec<ensof::TokenBalance>,
    },
    not_implemented: [
        "ArbitrumNativeController_checkBridgeTransaction",
        "CcipController_checkBridgeTransaction",
        "CcipController_getRouter",
        "CctpController_checkBridgeTransaction",
        "CctpController_getClaimTx",
        "CctpController_getTokenMessenger",
        "LayerZeroController_checkBridgeTransaction",
        "LayerZeroController_getPoolAddress",
        "MayanController_checkBridgeTransaction",
        "NonTokenizedController_nontokenizedPositions",
        "NontokenizedController_routeNontokenizedShorcutTransaction",
        "RelayController_checkBridgeTransaction",
    ],
}
