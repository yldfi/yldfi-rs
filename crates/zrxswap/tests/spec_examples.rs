//! Deserializes the example responses in 0x's EVM's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://docs.0x.org/openapi/evm-apis.json
//! crates/zrxswap/tests/spec_examples/0x.json`.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/0x.json",
    checks: {
        "sources_getsources" => zrxswap::SourcesResponse,
        "swap_permit2Getprice" => zrxswap::PriceResponse,
        "swap_permit2Getquote" => zrxswap::QuoteResponse,
    },
    not_implemented: [
        "gasless_chains",
        "gasless_getprice",
        "gasless_getquote",
        "gasless_getstatus",
        "gasless_submit",
        "swap_allowanceholderGetprice",
        "swap_allowanceholderGetquote",
        "swap_chains",
        "tradeAnalytics_tradeAnalytics::gasless",
        "tradeAnalytics_tradeAnalytics::swap",
    ],
    skip: [
        (
            "swap_permit2Getprice",
            "sample2",
            "the no-liquidity response, which the client turns into an error before parsing",
        ),
        (
            "swap_permit2Getquote",
            "sample2",
            "the no-liquidity response, which the client turns into an error before parsing",
        ),
    ],
}
