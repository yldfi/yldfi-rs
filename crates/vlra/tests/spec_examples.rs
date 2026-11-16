//! Deserializes the example responses in Velora's market's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://raw.githubusercontent.com/VeloraDEX/docs/main/api-reference/specs/market.json
//! crates/vlra/tests/spec_examples/velora.json`.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/velora.json",
    checks: {
        "marketPrices" => vlra::PriceResponse,
        "marketSwap" => vlra::SwapResponse,
        "marketTransactions" => vlra::TransactionResponse,
    },
    not_implemented: [],
}
