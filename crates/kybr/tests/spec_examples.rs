//! Deserializes the example responses in KyberSwap's aggregator's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://openapi.gitbook.com/o/2RCW3YGZHRsvcw6VtGeF/spec/aggregator-api.yaml
//! crates/kybr/tests/spec_examples/kyberswap.json`.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/kyberswap.json",
    checks: {
        "get-route" => kybr::RoutesResponse,
        "post-route-encoded" => kybr::BuildRouteResponse,
    },
    not_implemented: [
        "get-route-encode",
        "get-swap",
    ],
}
