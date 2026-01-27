//! Integration tests for the GoPlus Security API client
//!
//! These tests use wiremock to mock API responses and verify client behavior.

use gplus::{Client, Config};
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Helper to create a client configured for the mock server
fn mock_client(server: &MockServer) -> Client {
    Client::with_config(Config::new().with_base_url(server.uri())).expect("failed to create client")
}

/// Helper to create an authenticated client for the mock server
fn mock_auth_client(server: &MockServer) -> Client {
    Client::with_config(
        Config::with_credentials("test_key", "test_secret").with_base_url(server.uri()),
    )
    .expect("failed to create client")
}

// ==================== Token Security Tests ====================

#[tokio::test]
async fn test_token_security_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .and(query_param(
            "contract_addresses",
            "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48": {
                    "token_name": "USD Coin",
                    "token_symbol": "USDC",
                    "is_honeypot": "0",
                    "is_open_source": "1",
                    "buy_tax": "0",
                    "sell_tax": "0",
                    "is_proxy": "1",
                    "is_mintable": "0"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client
        .token_security(1, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
        .await;

    assert!(result.is_ok());
    let security = result.unwrap();
    assert_eq!(security.token_symbol.as_deref(), Some("USDC"));
    assert_eq!(security.token_name.as_deref(), Some("USD Coin"));
    assert!(!security.is_honeypot());
    assert!(security.is_verified());
}

#[tokio::test]
async fn test_token_security_honeypot() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/56"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "0xscamtoken": {
                    "token_name": "Scam Token",
                    "token_symbol": "SCAM",
                    "is_honeypot": "1",
                    "is_open_source": "0",
                    "buy_tax": "0",
                    "sell_tax": "100"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.token_security(56, "0xSCAMTOKEN").await;

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(security.is_honeypot());
    assert!(!security.is_verified());
    assert!(security.has_high_sell_tax());
}

#[tokio::test]
async fn test_token_security_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {}
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.token_security(1, "0xnonexistent").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("not found"));
}

#[tokio::test]
async fn test_token_security_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 5006,
            "message": "Param error!",
            "result": null
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.token_security(1, "invalid").await;

    assert!(result.is_err());
}

// ==================== Batch Token Security Tests ====================

#[tokio::test]
async fn test_token_security_batch_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "0xtoken1": {
                    "token_name": "Token 1",
                    "token_symbol": "TK1",
                    "is_honeypot": "0"
                },
                "0xtoken2": {
                    "token_name": "Token 2",
                    "token_symbol": "TK2",
                    "is_honeypot": "1"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client
        .token_security_batch(1, &["0xTOKEN1", "0xTOKEN2"])
        .await;

    assert!(result.is_ok());
    let results = result.unwrap();
    assert_eq!(results.len(), 2);
    assert!(results.contains_key("0xtoken1"));
    assert!(results.contains_key("0xtoken2"));
}

#[tokio::test]
async fn test_token_security_batch_empty() {
    let server = MockServer::start().await;
    let client = mock_client(&server);

    let result = client.token_security_batch(1, &[]).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

// ==================== Address Security Tests ====================

#[tokio::test]
async fn test_address_security_clean() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/address_security/0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
        ))
        .and(query_param("chain_id", "1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "is_contract": "0",
                "phishing_activities": "0",
                "blackmail_activities": "0",
                "stealing_attack": "0",
                "fake_kyc": "0",
                "malicious_mining_activities": "0",
                "darkweb_transactions": "0",
                "cybercrime": "0",
                "money_laundering": "0",
                "financial_crime": "0"
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client
        .address_security(1, "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .await;

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(!security.is_malicious());
    assert!(security.get_issues().is_empty());
}

#[tokio::test]
async fn test_address_security_malicious() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/address_security/0xmalicious"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "is_contract": "1",
                "phishing_activities": "1",
                "stealing_attack": "1",
                "data_source": "GoPlus Security"
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.address_security(1, "0xMALICIOUS").await;

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(security.is_malicious());
    let issues = security.get_issues();
    assert!(!issues.is_empty());
}

// ==================== NFT Security Tests ====================

#[tokio::test]
async fn test_nft_security_verified() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/nft_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "nft_name": "Bored Ape Yacht Club",
                "nft_symbol": "BAYC",
                "nft_verified": 1,
                "nft_open_source": 1,
                "malicious_nft_contract": 0,
                "nft_erc": "erc721",
                "website_url": "http://www.boredapeyachtclub.com/",
                "twitter_url": "https://twitter.com/BoredApeYC"
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client
        .nft_security(1, "0xbc4ca0eda7647a8ab7c2061c2e118a18a936f13d")
        .await;

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(security.is_verified());
    assert!(security.is_open_source());
    assert!(!security.is_malicious());
    assert!(!security.has_risks());
}

// ==================== Approval Security Tests ====================

#[tokio::test]
async fn test_approval_security_trusted() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/approval_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "contract_name": "FiatTokenProxy",
                "is_contract": 1,
                "is_open_source": 1,
                "is_proxy": 1,
                "trust_list": 1,
                "doubt_list": 0,
                "malicious_behavior": []
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client
        .approval_security(1, "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
        .await;

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(security.is_trusted());
    assert!(!security.is_malicious());
}

#[tokio::test]
async fn test_approval_security_malicious() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/approval_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "contract_name": "Phishing Contract",
                "is_contract": 1,
                "trust_list": 0,
                "doubt_list": 1,
                "malicious_behavior": ["Approval phishing", "Drainer contract"],
                "tag": "Fake_Phishing"
            }
        })))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.approval_security(1, "0xphishing").await;

    assert!(result.is_ok());
    let security = result.unwrap();
    assert!(security.is_malicious());
    assert!(security.is_doubtful());
    assert!(!security.is_trusted());
}

// ==================== Authentication Tests ====================

#[tokio::test]
async fn test_auth_header_sent_when_authenticated() {
    let server = MockServer::start().await;

    // First mock the token endpoint
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "access_token": "test_access_token_123",
                "expires_in": 3600
            }
        })))
        .mount(&server)
        .await;

    // Then mock the actual request expecting auth header
    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .and(header("Authorization", "Bearer test_access_token_123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "0xtoken": {
                    "token_name": "Test",
                    "token_symbol": "TEST"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = mock_auth_client(&server);
    let result = client.token_security(1, "0xTOKEN").await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_clear_token_cache() {
    let server = MockServer::start().await;

    // Token endpoint
    Mock::given(method("POST"))
        .and(path("/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "result": {
                "access_token": "test_token",
                "expires_in": 3600
            }
        })))
        .mount(&server)
        .await;

    // API endpoint
    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "code": 1,
            "message": "ok",
            "result": {
                "0xtoken": {
                    "token_name": "Test",
                    "token_symbol": "TEST"
                }
            }
        })))
        .mount(&server)
        .await;

    let client = mock_auth_client(&server);

    // First request should succeed and cache token
    let result = client.token_security(1, "0xTOKEN").await;
    assert!(result.is_ok());

    // Clear the token cache
    client.clear_token_cache().await;

    // Verify we can still make requests (token will be re-fetched)
    let result = client.token_security(1, "0xTOKEN").await;
    assert!(result.is_ok());
}

// ==================== Rate Limit Tests ====================

#[tokio::test]
async fn test_rate_limit_info_extracted() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-ratelimit-remaining", "95")
                .insert_header("x-ratelimit-limit", "100")
                .set_body_json(serde_json::json!({
                    "code": 1,
                    "message": "ok",
                    "result": {
                        "0xtoken": {
                            "token_name": "Test",
                            "token_symbol": "TEST"
                        }
                    }
                })),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);

    // Before any request, rate limit should be None
    assert!(client.rate_limit_info().await.is_none());

    // Make a request
    let _ = client.token_security(1, "0xTOKEN").await;

    // After request, rate limit info should be available
    let rate_info = client.rate_limit_info().await;
    assert!(rate_info.is_some());
    let info = rate_info.unwrap();
    assert_eq!(info.remaining, Some(95));
    assert_eq!(info.limit, Some(100));
    assert!(!info.is_near_limit());
    assert!(!info.is_exhausted());
}

#[tokio::test]
async fn test_rate_limit_near_exhaustion() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("x-ratelimit-remaining", "5")
                .insert_header("x-ratelimit-limit", "100")
                .set_body_json(serde_json::json!({
                    "code": 1,
                    "message": "ok",
                    "result": {
                        "0xtoken": {
                            "token_name": "Test",
                            "token_symbol": "TEST"
                        }
                    }
                })),
        )
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let _ = client.token_security(1, "0xTOKEN").await;

    assert!(client.is_near_rate_limit().await);
}

// ==================== Error Handling Tests ====================

#[tokio::test]
async fn test_http_error_500() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.token_security(1, "0xtoken").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("500"));
}

#[tokio::test]
async fn test_malformed_json_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/token_security/1"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
        .mount(&server)
        .await;

    let client = mock_client(&server);
    let result = client.token_security(1, "0xtoken").await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Parse error"));
}
