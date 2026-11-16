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
        "GET /v1/gas/status" => lfi::GasStatusResponse,
        "GET /v1/quote" => lfi::Quote,
        "GET /v1/quote/toAmount" => lfi::Quote,
        "GET /v1/status" => lfi::StatusResponse,
        "GET /v1/token" => lfi::Token,
        "GET /v1/tokens" => lfi::TokensResponse,
        "GET /v1/tools" => lfi::ToolsResponse,
        "GET /v2/analytics/transfers" => lfi::TransfersResponse,
        "POST /v1/advanced/routes" => lfi::RoutesResponse,
        "POST /v1/advanced/stepTransaction" => lfi::Step,
        "POST /v1/quote/contractCalls" => lfi::Quote,
    },
    not_implemented: [
        "GET /v1/analytics/transfers",
        "GET /v1/analytics/transfers/summary",
        "GET /v1/gas/refetch",
        "GET /v1/integrators/{integratorId}",
        "GET /v1/integrators/{integratorId}/withdraw/{chainId}",
        "GET /v1/relayer/quote",
        "GET /v1/relayer/status/{taskId}",
        "POST /v1/advanced/possibilities",
        "POST /v1/quote/contractCall",
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
            "GET /v1/quote/toAmount",
            "sample",
            "the spec's transactionRequest also covers Solana (SVM) transactions, which this crate does not support; the EVM example is checked",
        ),
        (
            "GET /v1/token",
            "sample",
            "the spec's schema is an array of tokens; live /token returns one token object (as the spec's example shows)",
        ),
        (
            "POST /v1/quote/contractCalls",
            "sample",
            "the spec gives the /tools response schema; live /quote/contractCalls returns a step, like /quote",
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
