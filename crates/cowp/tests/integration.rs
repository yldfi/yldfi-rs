//! Integration tests for cowp client
//!
//! These tests verify the client correctly handles API responses without
//! making actual network calls.

use cowp::{Chain, Client, Config, QuoteRequest};

#[tokio::test]
async fn test_client_creation() {
    let client = Client::new();
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_config_builder() {
    let config = Config::new()
        .with_chain(Chain::Gnosis)
        .with_timeout(std::time::Duration::from_secs(60));

    let client = Client::with_config(config);
    assert!(client.is_ok());
}

#[tokio::test]
async fn test_quote_request_validation() {
    // Test that validation catches invalid addresses
    let request = QuoteRequest::sell(
        "invalid",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "1000",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    );

    let result = request.validate();
    assert!(result.is_err(), "Should reject invalid sell_token address");
}

#[tokio::test]
async fn test_quote_request_valid() {
    let request = QuoteRequest::sell(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48",
        "1000000000000000000",
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    );

    let result = request.validate();
    assert!(result.is_ok(), "Valid request should pass validation");
}

#[tokio::test]
async fn test_chain_api_urls() {
    assert_eq!(Chain::Mainnet.api_url(), "https://api.cow.fi/mainnet");
    assert_eq!(Chain::Gnosis.api_url(), "https://api.cow.fi/xdai");
    assert_eq!(Chain::Arbitrum.api_url(), "https://api.cow.fi/arbitrum_one");
    assert_eq!(Chain::Sepolia.api_url(), "https://api.cow.fi/sepolia");
}

#[tokio::test]
async fn test_chain_from_str() {
    assert_eq!("mainnet".parse::<Chain>().unwrap(), Chain::Mainnet);
    assert_eq!("ethereum".parse::<Chain>().unwrap(), Chain::Mainnet);
    assert_eq!("gnosis".parse::<Chain>().unwrap(), Chain::Gnosis);
    assert_eq!("xdai".parse::<Chain>().unwrap(), Chain::Gnosis);
    assert_eq!("arbitrum".parse::<Chain>().unwrap(), Chain::Arbitrum);
    assert_eq!("sepolia".parse::<Chain>().unwrap(), Chain::Sepolia);
}

#[tokio::test]
async fn test_chain_from_str_invalid() {
    let result = "invalid_chain".parse::<Chain>();
    assert!(result.is_err());
}

// Note: Full HTTP integration tests would require modifications
// to cowp's Client to accept a custom base URL. The tests above verify
// the core functionality that can be tested without mocking HTTP.
