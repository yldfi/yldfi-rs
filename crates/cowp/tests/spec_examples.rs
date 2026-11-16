//! Deserializes the example responses in CoW Protocol's orderbook OpenAPI
//! spec, plus responses built from each endpoint's schema, into the type
//! this crate reads that endpoint's response as, so a type that no longer
//! matches the spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://raw.githubusercontent.com/cowprotocol/services/main/crates/orderbook/openapi.yml
//! crates/cowp/tests/spec_examples/cowswap.json`.
//!
//! The auction, solver competition and native price calls return untyped
//! JSON, so their checks only confirm the responses are valid JSON.
//!
//! `getOrders` (`POST /api/v1/orders/by_uids`) returns an array, but the
//! spec's response schema describes a single element, so it is checked
//! against the element type.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/cowswap.json",
    checks: {
        "createOrder" => String,
        "getAddressTotalSurplus" => cowp::TotalSurplus,
        "getAppDataByHash" => cowp::AppDataObject,
        "getCurrentBatchAuction" => serde_json::Value,
        "getOrder" => cowp::Order,
        "getOrderStatus" => cowp::CompetitionOrderStatus,
        "getOrders" => cowp::OrderLookup,
        "getOrdersByTxHash" => Vec<cowp::Order>,
        "getSolverCompetitionByAuctionIdV2" => serde_json::Value,
        "getSolverCompetitionByTxHashV2" => serde_json::Value,
        "getSolverCompetitionLatestV2" => serde_json::Value,
        "getTokenNativePrice" => serde_json::Value,
        "getTradesV2" => Vec<cowp::Trade>,
        "getUserOrdersPaginated" => Vec<cowp::Order>,
        "quote" => cowp::QuoteResponse,
    },
    not_implemented: [
        "debugOrder",
        "debugSimulation",
        "debugSimulationPost",
        "registerAppData",
        "registerAppDataByHash",
    ],
}
