//! Mock API tests for the Pyth client using wiremock
//!
//! These tests verify client behavior without hitting the real Pyth API.

use pythc::{Client, Config};
use wiremock::matchers::{header, header_exists, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_API_KEY: &str = "test-pyth-key-123";

/// Create an authenticated client configured to use the mock server.
///
/// Every mock below matches on the `Authorization: Bearer <key>` header, so a
/// request without the header would not match and the test would fail.
fn mock_client(server: &MockServer) -> Client {
    Client::with_config(
        Config::default()
            .with_base_url(server.uri())
            .with_api_key(TEST_API_KEY),
    )
    .unwrap()
}

fn latest_eth_body() -> serde_json::Value {
    serde_json::json!({
        "binary": { "encoding": "hex", "data": [] },
        "parsed": [{
            "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "price": { "price": "325000000000", "conf": "100000000", "expo": -8, "publish_time": 1704067200 },
            "ema_price": { "price": "324500000000", "conf": "150000000", "expo": -8, "publish_time": 1704067200 }
        }]
    })
}

#[tokio::test]
async fn test_get_latest_price_success() {
    let mock_server = MockServer::start().await;

    // Mock response for ETH/USD price
    let response_body = serde_json::json!({
        "binary": {
            "encoding": "hex",
            "data": ["deadbeef"]
        },
        "parsed": [{
            "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "price": {
                "price": "325000000000",
                "conf": "100000000",
                "expo": -8,
                "publish_time": 1704067200
            },
            "ema_price": {
                "price": "324500000000",
                "conf": "150000000",
                "expo": -8,
                "publish_time": 1704067200
            }
        }]
    });

    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .and(query_param(
            "ids[]",
            "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let result = client.get_latest_price(pythc::feed_ids::ETH_USD).await;

    assert!(result.is_ok());
    let feed = result.unwrap();
    assert!(feed.is_some());

    let feed = feed.unwrap();
    assert_eq!(
        feed.id,
        "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace"
    );
    assert_eq!(feed.price.price, "325000000000");
    assert_eq!(feed.price.expo, -8);

    // Test price_f64 conversion: 325000000000 * 10^(-8) = 3250.0
    let price_f64 = feed.price_f64();
    assert!(price_f64.is_some());
    let price = price_f64.unwrap();
    assert!((price - 3250.0).abs() < 0.001);
}

#[tokio::test]
async fn test_get_latest_prices_multiple() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "binary": {
            "encoding": "hex",
            "data": ["deadbeef"]
        },
        "parsed": [
            {
                "id": "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
                "price": {
                    "price": "4300000000000",
                    "conf": "500000000",
                    "expo": -8,
                    "publish_time": 1704067200
                },
                "ema_price": {
                    "price": "4295000000000",
                    "conf": "600000000",
                    "expo": -8,
                    "publish_time": 1704067200
                }
            },
            {
                "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
                "price": {
                    "price": "325000000000",
                    "conf": "100000000",
                    "expo": -8,
                    "publish_time": 1704067200
                },
                "ema_price": {
                    "price": "324500000000",
                    "conf": "150000000",
                    "expo": -8,
                    "publish_time": 1704067200
                }
            }
        ]
    });

    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let result = client
        .get_latest_prices(&[pythc::feed_ids::BTC_USD, pythc::feed_ids::ETH_USD])
        .await;

    assert!(result.is_ok());
    let feeds = result.unwrap();
    assert_eq!(feeds.len(), 2);

    // BTC price: 4300000000000 * 10^(-8) = 43000.0
    let btc = &feeds[0];
    let btc_price = btc.price_f64().unwrap();
    assert!((btc_price - 43000.0).abs() < 0.001);

    // ETH price: 325000000000 * 10^(-8) = 3250.0
    let eth = &feeds[1];
    let eth_price = eth.price_f64().unwrap();
    assert!((eth_price - 3250.0).abs() < 0.001);
}

#[tokio::test]
async fn test_get_latest_prices_empty_input() {
    let mock_server = MockServer::start().await;

    let client = mock_client(&mock_server);
    let result = client.get_latest_prices(&[]).await;

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[tokio::test]
async fn test_invalid_feed_id_rejected() {
    let mock_server = MockServer::start().await;

    let client = mock_client(&mock_server);

    // Too short
    let result = client.get_latest_price("0x1234").await;
    assert!(result.is_err());

    // Invalid characters
    let result = client
        .get_latest_price("0xgg61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace")
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_rate_limiting_with_retry() {
    let mock_server = MockServer::start().await;

    // First two requests get rate limited, third succeeds
    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "0"))
        .up_to_n_times(2)
        .mount(&mock_server)
        .await;

    let response_body = serde_json::json!({
        "binary": { "encoding": "hex", "data": [] },
        "parsed": [{
            "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "price": { "price": "325000000000", "conf": "100000000", "expo": -8, "publish_time": 1704067200 },
            "ema_price": { "price": "324500000000", "conf": "150000000", "expo": -8, "publish_time": 1704067200 }
        }]
    });

    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let result = client.get_latest_price(pythc::feed_ids::ETH_USD).await;

    // Should succeed after retries
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_server_error_retry() {
    let mock_server = MockServer::start().await;

    // First request gets 500, second succeeds
    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .up_to_n_times(1)
        .mount(&mock_server)
        .await;

    let response_body = serde_json::json!({
        "binary": { "encoding": "hex", "data": [] },
        "parsed": [{
            "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "price": { "price": "325000000000", "conf": "100000000", "expo": -8, "publish_time": 1704067200 },
            "ema_price": { "price": "324500000000", "conf": "150000000", "expo": -8, "publish_time": 1704067200 }
        }]
    });

    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let result = client.get_latest_price(pythc::feed_ids::ETH_USD).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_search_feeds() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!([
        {
            "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "attributes": {
                "asset_type": "Crypto",
                "base": "ETH",
                "quote_currency": "USD",
                "symbol": "Crypto.ETH/USD"
            }
        }
    ]);

    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/price_feeds"))
        .and(query_param("query", "ETH"))
        .and(query_param("asset_type", "crypto"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let result = client.search_feeds("ETH").await;

    assert!(result.is_ok());
    let feeds = result.unwrap();
    assert_eq!(feeds.len(), 1);
    assert_eq!(
        feeds[0].id,
        "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace"
    );
    assert_eq!(feeds[0].attributes.base, Some("ETH".to_string()));
}

#[tokio::test]
async fn test_get_price_feed_ids() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!([
        {
            "id": "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43",
            "attributes": { "base": "BTC", "quote_currency": "USD", "symbol": "Crypto.BTC/USD" }
        },
        {
            "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "attributes": { "base": "ETH", "quote_currency": "USD", "symbol": "Crypto.ETH/USD" }
        }
    ]);

    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/price_feeds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let result = client.get_price_feed_ids().await;

    assert!(result.is_ok());
    let feeds = result.unwrap();
    assert_eq!(feeds.len(), 2);
}

#[tokio::test]
async fn test_404_error() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .respond_with(ResponseTemplate::new(404).set_body_string("Not found"))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let result = client.get_latest_price(pythc::feed_ids::ETH_USD).await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("404"));
}

#[tokio::test]
async fn test_feed_id_normalization() {
    let mock_server = MockServer::start().await;

    let response_body = serde_json::json!({
        "binary": { "encoding": "hex", "data": [] },
        "parsed": [{
            "id": "ff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
            "price": { "price": "325000000000", "conf": "100000000", "expo": -8, "publish_time": 1704067200 },
            "ema_price": { "price": "324500000000", "conf": "150000000", "expo": -8, "publish_time": 1704067200 }
        }]
    });

    // Test that uppercase feed IDs are normalized to lowercase
    Mock::given(method("GET"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .and(path("/v2/updates/price/latest"))
        .and(query_param(
            "ids[]",
            "0xff61491a931112ddf1bd8147cd1b641375f79f5825126d665480874634fd0ace",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(&response_body))
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);

    // Pass uppercase feed ID - should be normalized
    let result = client
        .get_latest_price("0xFF61491A931112DDF1BD8147CD1B641375F79F5825126D665480874634FD0ACE")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_symbol_to_feed_id() {
    // Test symbol lookup
    assert_eq!(
        pythc::symbol_to_feed_id("ETH"),
        Some(pythc::feed_ids::ETH_USD)
    );
    assert_eq!(
        pythc::symbol_to_feed_id("eth"),
        Some(pythc::feed_ids::ETH_USD)
    );
    assert_eq!(
        pythc::symbol_to_feed_id("ETHEREUM"),
        Some(pythc::feed_ids::ETH_USD)
    );
    assert_eq!(
        pythc::symbol_to_feed_id("BTC"),
        Some(pythc::feed_ids::BTC_USD)
    );
    assert_eq!(
        pythc::symbol_to_feed_id("BITCOIN"),
        Some(pythc::feed_ids::BTC_USD)
    );
    assert_eq!(
        pythc::symbol_to_feed_id("USDC"),
        Some(pythc::feed_ids::USDC_USD)
    );
    assert_eq!(
        pythc::symbol_to_feed_id("LINK"),
        Some(pythc::feed_ids::LINK_USD)
    );

    // Unknown symbol
    assert_eq!(pythc::symbol_to_feed_id("UNKNOWN_TOKEN_XYZ"), None);
}

#[tokio::test]
async fn test_default_base_url_is_upgraded_endpoint() {
    assert_eq!(
        pythc::base_urls::MAINNET,
        "https://pyth.dourolabs.app/hermes"
    );
    assert_eq!(pythc::base_urls::LEGACY, "https://hermes.pyth.network");
}

#[tokio::test]
async fn test_base_url_path_prefix_preserved() {
    // The upgraded endpoint lives under a path prefix (/hermes); make sure
    // request paths are appended rather than replacing it.
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/hermes/v2/updates/price/latest"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .respond_with(ResponseTemplate::new(200).set_body_json(latest_eth_body()))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = Client::with_config(
        Config::default()
            .with_base_url(format!("{}/hermes", mock_server.uri()))
            .with_api_key(TEST_API_KEY),
    )
    .unwrap();
    let feed = client
        .get_latest_price(pythc::feed_ids::ETH_USD)
        .await
        .unwrap();
    assert!(feed.is_some());
}

#[tokio::test]
async fn test_no_auth_header_without_key() {
    let mock_server = MockServer::start().await;

    // Any request carrying an Authorization header would hit this mock
    Mock::given(method("GET"))
        .and(header_exists("authorization"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&mock_server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v2/price_feeds"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = Client::with_config(Config::default().with_base_url(mock_server.uri())).unwrap();
    assert!(!client.has_api_key());
    let feeds = client.get_price_feed_ids().await.unwrap();
    assert!(feeds.is_empty());
}

#[tokio::test]
async fn test_401_without_key_is_actionable_and_not_retried() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/updates/price/latest"))
        .respond_with(ResponseTemplate::new(401).set_body_string("Unauthorized"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = Client::with_config(Config::default().with_base_url(mock_server.uri())).unwrap();
    let err = client
        .get_latest_price(pythc::feed_ids::ETH_USD)
        .await
        .unwrap_err();

    assert!(pythc::is_unauthorized(&err));
    let msg = err.to_string();
    assert!(msg.contains("401"), "{msg}");
    assert!(msg.contains("API key is required"), "{msg}");
    assert!(msg.contains("PYTH_API_KEY"), "{msg}");
}

#[tokio::test]
async fn test_401_with_key_reports_rejected_key_without_leaking_it() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v2/updates/price/latest"))
        .and(header("authorization", "Bearer test-pyth-key-123"))
        .respond_with(ResponseTemplate::new(401).set_body_string("invalid token test-pyth-key-123"))
        .expect(1)
        .mount(&mock_server)
        .await;

    let client = mock_client(&mock_server);
    let err = client
        .get_latest_price(pythc::feed_ids::ETH_USD)
        .await
        .unwrap_err();

    assert!(pythc::is_unauthorized(&err));
    let msg = err.to_string();
    assert!(msg.contains("rejected"), "{msg}");
    assert!(!msg.contains(TEST_API_KEY), "{msg}");
    assert!(!format!("{err:?}").contains(TEST_API_KEY));
}

#[tokio::test]
async fn test_api_key_redacted_in_debug() {
    let config = Config::mainnet().with_api_key(TEST_API_KEY);
    assert!(config.has_api_key());
    assert!(!format!("{config:?}").contains(TEST_API_KEY));

    let client = Client::with_config(config).unwrap();
    assert!(client.has_api_key());
    assert!(!format!("{client:?}").contains(TEST_API_KEY));

    // Blank keys are treated as "no key"
    assert!(!Config::mainnet().with_api_key("   ").has_api_key());
    assert!(!Config::mainnet().with_optional_api_key(None).has_api_key());
}
