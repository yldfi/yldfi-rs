//! Deserializes the example responses in CoW Protocol's orderbook OpenAPI
//! spec, plus responses built from each endpoint's schema, into the type
//! this crate reads that endpoint's response as, so a type that no longer
//! matches the spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://raw.githubusercontent.com/cowprotocol/services/main/crates/orderbook/openapi.yml
//! crates/cowp/tests/spec_examples/cowswap.json`.
//!
//! Only the endpoints this crate reads into its own types are checked; the
//! auction, solver competition and native price calls return untyped JSON.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/cowswap.json",
    checks: {
        "createOrder" => String,
        "getOrder" => cowp::Order,
        "getTradesV2" => Vec<cowp::Trade>,
        "getUserOrdersPaginated" => Vec<cowp::Order>,
        "quote" => cowp::QuoteResponse,
    },
    not_implemented: [
        "debugOrder",
        "debugSimulation",
        "debugSimulationPost",
        "getAddressTotalSurplus",
        "getAppDataByHash",
        "getCurrentBatchAuction",
        "getOrderStatus",
        "getOrders",
        "getOrdersByTxHash",
        "getSolverCompetitionByAuctionIdV2",
        "getSolverCompetitionByTxHashV2",
        "getSolverCompetitionLatestV2",
        "getTokenNativePrice",
        "registerAppData",
        "registerAppDataByHash",
    ],
}
