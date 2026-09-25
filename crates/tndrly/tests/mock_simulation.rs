//! Offline (wiremock) tests for simulation endpoints.

use tndrly::{Client, Config};
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Client {
    Client::new(Config::new("test-key", "acc", "proj").with_base_url(server.uri())).unwrap()
}

#[tokio::test]
async fn list_sends_page_then_per_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/account/acc/project/proj/simulations"))
        .and(query_param("page", "2"))
        .and(query_param("perPage", "20"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "simulations": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    let resp = client(&server).simulation().list(2, 20).await.unwrap();
    assert!(resp.simulations.is_empty());
}

#[tokio::test]
async fn list_clamps_invalid_page_and_per_page() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/account/acc/project/proj/simulations"))
        .and(query_param("page", "1"))
        .and(query_param("perPage", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "simulations": []
        })))
        .expect(1)
        .mount(&server)
        .await;

    client(&server).simulation().list(0, 0).await.unwrap();
}

#[tokio::test]
async fn info_uses_documented_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/account/acc/project/proj/simulations/sim1/info"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({ "transaction_info": {} })),
        )
        .expect(1)
        .mount(&server)
        .await;

    let resp = client(&server).simulation().info("sim1").await.unwrap();
    assert!(resp.get("transaction_info").is_some());
}

#[tokio::test]
#[allow(deprecated)]
async fn contract_abi_is_a_clear_error_not_none() {
    let server = MockServer::start().await;
    let err = client(&server)
        .contracts()
        .abi("1", "0x0000000000000000000000000000000000000001")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("ABI"), "{err}");
    // No request should have been made.
    assert!(server.received_requests().await.unwrap().is_empty());
}
