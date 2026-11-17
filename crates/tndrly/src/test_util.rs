//! Shared helpers for the crate's unit tests

use crate::client::{Client, Config};
use wiremock::MockServer;

/// A mock server and a client pointed at it the way production is laid out:
/// base URL `{server}/api/v1`, so `/v2/...` and `/public/v1/...` routes land
/// under `{server}/api`. Account `acct`, project `proj`.
pub(crate) async fn mock_client() -> (MockServer, Client) {
    let server = MockServer::start().await;
    let config =
        Config::new("key", "acct", "proj").with_base_url(format!("{}/api/v1", server.uri()));
    (server, Client::new(config).unwrap())
}
