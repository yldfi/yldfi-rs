//! Shared helpers for the API tests

use crate::Client;
use wiremock::matchers::{any, header};
use wiremock::{Mock, MockBuilder, MockServer, ResponseTemplate};

/// Serve `body` as JSON to the request `mock` matches, expected exactly once
/// (checked when the server drops), and return a client pointed at it. The
/// server must outlive the request, so keep it bound for the whole test.
pub(crate) async fn serve(mock: MockBuilder, body: serde_json::Value) -> (MockServer, Client) {
    let server = MockServer::start().await;
    mock.and(header("x-dune-api-key", "test-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(body))
        .expect(1)
        .mount(&server)
        .await;
    let client = Client::with_base_url("test-key", &server.uri()).unwrap();
    (server, client)
}

/// A client pointed at a server that answers every request with `status`
pub(crate) async fn failing(status: u16) -> (MockServer, Client) {
    let server = MockServer::start().await;
    Mock::given(any())
        .respond_with(ResponseTemplate::new(status))
        .mount(&server)
        .await;
    let client = Client::with_base_url("test-key", &server.uri()).unwrap();
    (server, client)
}

/// Whether `err` is the crate's not-found error
pub(crate) fn is_not_found(err: &crate::Error) -> bool {
    matches!(
        err,
        crate::error::ApiError::Domain(crate::error::DomainError::NotFound(_))
    )
}
