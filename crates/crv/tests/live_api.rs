//! Integration tests against live Curve APIs
//!
//! These tests hit the real APIs and may be skipped in CI.
//! Run with: cargo test -p crv --test live_api

use crv::{Client, PricesClient};

#[tokio::test]
async fn test_pools_api() {
    let client = Client::new().expect("Failed to create client");

    let pools = client.pools().get_all_on_chain("ethereum").await.unwrap();
    assert!(pools.success);
    assert!(!pools.data.pool_data.is_empty());
    println!("Found {} pools on Ethereum", pools.data.pool_data.len());
}

#[tokio::test]
async fn test_lending_api() {
    let client = Client::new().expect("Failed to create client");

    let vaults = client.lending().get_all().await.unwrap();
    assert!(vaults.success);
    assert!(!vaults.data.lending_vault_data.is_empty());
    println!(
        "Found {} lending vaults",
        vaults.data.lending_vault_data.len()
    );
}

#[tokio::test]
async fn test_prices_api_ping() {
    let client = PricesClient::new().expect("Failed to create client");

    let ping = client.ping().await.unwrap();
    assert_eq!(ping["status"], "ok");
}

#[tokio::test]
async fn test_prices_api_usd_price() {
    let client = PricesClient::new().expect("Failed to create client");

    // WETH on Ethereum
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let price = client.get_usd_price("ethereum", weth).await.unwrap();

    assert!(price["data"]["usd_price"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
async fn test_prices_api_chains() {
    let client = PricesClient::new().expect("Failed to create client");

    let chains = client.get_chains().await.unwrap();
    assert!(chains["data"].is_array() || chains["data"].is_object());
}
