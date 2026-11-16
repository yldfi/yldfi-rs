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
        "gasless_chains" => zrxswap::ChainsResponse,
        "gasless_getprice" => zrxswap::GaslessPriceResponse,
        "gasless_getquote" => zrxswap::GaslessQuoteResponse,
        "gasless_getstatus" => zrxswap::GaslessStatusResponse,
        "sources_getsources" => zrxswap::SourcesResponse,
        "swap_allowanceholderGetprice" => zrxswap::PriceResponse,
        "swap_allowanceholderGetquote" => zrxswap::QuoteResponse,
        "swap_chains" => zrxswap::ChainsResponse,
        "swap_permit2Getprice" => zrxswap::PriceResponse,
        "swap_permit2Getquote" => zrxswap::QuoteResponse,
    },
    not_implemented: [
        "gasless_submit",
        "tradeAnalytics_tradeAnalytics::gasless",
        "tradeAnalytics_tradeAnalytics::swap",
    ],
    skip: [
        (
            "gasless_getprice",
            "sample2",
            "the no-liquidity response (the spec types its liquidityAvailable as any boolean), which the client turns into an error before parsing",
        ),
        (
            "gasless_getquote",
            "sample2",
            "the no-liquidity response (the spec types its liquidityAvailable as any boolean), which the client turns into an error before parsing",
        ),
        (
            "swap_allowanceholderGetprice",
            "sample2",
            "the no-liquidity response, which the client turns into an error before parsing",
        ),
        (
            "swap_allowanceholderGetquote",
            "sample2",
            "the no-liquidity response, which the client turns into an error before parsing",
        ),
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
