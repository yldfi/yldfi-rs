//! `contract twins --submit-similarity` against a mock Sourcify: the real
//! endpoint publishes a verification, so it is never called from tests.

use ethcli::etherscan::fallback::{sourcify_lookup, sourcify_submit_similarity};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

const ADDR: &str = "0x6b75d8AF000000e20B7a7DDf000Ba900b4009A80";

#[tokio::test]
async fn similarity_submission_and_lookup_use_the_configured_server() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path(format!("/v2/verify/similarity/1/{ADDR}")))
        .respond_with(
            ResponseTemplate::new(202)
                .set_body_json(serde_json::json!({"verificationId": "job-1"})),
        )
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/verify/job-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "isJobCompleted": true,
            "error": {"customCode": "no_similar_match_found"}
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path(format!("/v2/contract/1/{ADDR}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "match": "match", "runtimeMatch": "match", "creationMatch": null
        })))
        .mount(&server)
        .await;

    // Only this test in this binary reads the variable
    std::env::set_var("SOURCIFY_SERVER_URL", server.uri());
    let http = reqwest::Client::new();

    let job = sourcify_submit_similarity(&http, 1, ADDR, Duration::from_secs(10))
        .await
        .expect("submitted");
    assert_eq!(job["verificationId"], "job-1");
    assert_eq!(job["isJobCompleted"], true);

    let m = sourcify_lookup(&http, 1, ADDR).await.expect("answered");
    assert_eq!(m.match_type, "partial");
}
