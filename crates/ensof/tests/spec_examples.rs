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
        "BundleController_bundleShortcutTransaction" => ensof::BundleResponse,
        "PricesController_getPrice" => ensof::TokenPrice,
        "RouterController_postRouteShortcutTransaction" => ensof::RouteResponse,
        "WalletController_walletBalances" => Vec<ensof::TokenBalance>,
    },
    not_implemented: [
        "AccountController_accountId",
        "ActionsController_findAll",
        "ActionsController_getActionsBySlug",
        "AggregatorsController_aggregators",
        "ArbitrumNativeController_checkBridgeTransaction",
        "CcipController_checkBridgeTransaction",
        "CcipController_getRouter",
        "CctpController_checkBridgeTransaction",
        "CctpController_getClaimTx",
        "CctpController_getTokenMessenger",
        "LayerZeroController_checkBridgeTransaction",
        "LayerZeroController_getPoolAddress",
        "MayanController_checkBridgeTransaction",
        "NetworksController_networks",
        "NonTokenizedController_nontokenizedPositions",
        "NontokenizedController_routeNontokenizedShorcutTransaction",
        "PricesController_getPrices",
        "ProjectsController_getStandardsByProject",
        "ProjectsController_standards",
        "ProtocolsController_findAll",
        "QuoteController_quote",
        "RelayController_checkBridgeTransaction",
        "RouterController_routeShortcutTransaction",
        "StandardsController_getProtocolBySlug",
        "StandardsController_standards",
        "TokensController_tokens",
        "WalletController_createApproveTransaction",
        "WalletController_wallet",
    ],
}
