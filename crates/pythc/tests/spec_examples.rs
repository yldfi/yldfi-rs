//! Deserializes the example responses in Pyth Hermes's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://raw.githubusercontent.com/pyth-network/pyth-crosschain/main/apps/hermes/client/js/schema.json
//! crates/pythc/tests/spec_examples/pyth-hermes.json`.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/pyth-hermes.json",
    checks: {
        "latest_price_updates" => pythc::LatestPriceResponse,
        "price_feeds_metadata" => Vec<pythc::PriceFeedId>,
    },
    not_implemented: [
        "get_price_feed",
        "get_vaa",
        "get_vaa_ccip",
        "latest_price_feeds",
        "latest_publisher_stake_caps",
        "latest_vaas",
        "price_feed_ids",
        "price_stream_sse_handler",
        "timestamp_price_updates",
    ],
}
