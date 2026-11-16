//! Deserializes the example responses in LI.FI's OpenAPI spec, plus
//! responses built from each endpoint's schema, into the type this crate
//! reads that endpoint's response as, so a type that no longer matches the
//! spec fails here instead of at runtime.
//!
//! Refresh the fixture with `python3 scripts/spec-check/examples.py
//! https://docs.li.fi/openapi.yaml crates/lfi/tests/spec_examples/lifi.json`.
//!
//! The skips below were checked against live li.quest responses.

yldfi_spec_test::spec_tests! {
    file: "spec_examples/lifi.json",
    checks: {
        "GET /v1/chains" => lfi::ChainsResponse,
        "GET /v1/connections" => lfi::ConnectionsResponse,
        "GET /v1/gas/prices" => lfi::GasPricesResponse,
        "GET /v1/gas/prices/{chainId}" => lfi::GasPrice,
        "GET /v1/quote" => lfi::Quote,
        "GET /v1/status" => lfi::StatusResponse,
        "GET /v1/tokens" => lfi::TokensResponse,
        "GET /v1/tools" => lfi::ToolsResponse,
        "POST /v1/advanced/routes" => lfi::RoutesResponse,
        "POST /v1/advanced/stepTransaction" => lfi::Step,
    },
    not_implemented: [
        "GET /v1/analytics/transfers",
        "GET /v1/analytics/transfers/summary",
        "GET /v1/gas/refetch",
        "GET /v1/gas/status",
        "GET /v1/integrators/{integratorId}",
        "GET /v1/integrators/{integratorId}/withdraw/{chainId}",
        "GET /v1/quote/toAmount",
        "GET /v1/relayer/quote",
        "GET /v1/relayer/status/{taskId}",
        "GET /v1/token",
        "GET /v2/analytics/transfers",
        "POST /v1/advanced/possibilities",
        "POST /v1/quote/contractCall",
        "POST /v1/quote/contractCalls",
        "POST /v1/relayer/relay",
    ],
    skip: [
        (
            "GET /v1/gas/prices",
            "sample",
            "the spec describes one chain's prices; live /gas/prices maps chain IDs to prices",
        ),
        (
            "GET /v1/tokens",
            "sample",
            "the spec omits the tokens wrapper that live /tokens responses have",
        ),
        (
            "GET /v1/tools",
            "sample",
            "the spec types supportedChains chain IDs as strings; live responses send integers",
        ),
        (
            "GET /v1/quote",
            "sample",
            "the spec's transactionRequest also covers Solana (SVM) transactions, which this crate does not support; the EVM example is checked",
        ),
        (
            "POST /v1/advanced/stepTransaction",
            "sample",
            "the spec's transactionRequest also covers Solana (SVM) transactions, which this crate does not support",
        ),
        (
            "POST /v1/advanced/routes",
            "sample",
            "the spec's transactionRequest also covers Solana (SVM) transactions, which this crate does not support",
        ),
        (
            "POST /v1/advanced/routes",
            "example",
            "the spec's example route omits fromAddress, which live routes include (the request requires it)",
        ),
    ],
}
