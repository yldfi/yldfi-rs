//! Integration tests against live Curve APIs
//!
//! These tests hit the real APIs and are skipped in CI.
//! Run with: cargo test -p crv --test live_api -- --ignored

use crv::{Client, PricesClient, RouterApi};

#[tokio::test]
#[ignore]
async fn test_pools_api() {
    let client = Client::new().expect("Failed to create client");

    let pools = client.pools().get_all_on_chain("ethereum").await.unwrap();
    assert!(pools.success);
    assert!(!pools.data.pool_data.is_empty());
    println!("Found {} pools on Ethereum", pools.data.pool_data.len());
}

#[tokio::test]
#[ignore]
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
#[ignore]
async fn test_prices_api_ping() {
    let client = PricesClient::new().expect("Failed to create client");

    let ping = client.ping().await.unwrap();
    assert_eq!(ping["status"], "ok");
}

#[tokio::test]
#[ignore]
async fn test_prices_api_usd_price() {
    let client = PricesClient::new().expect("Failed to create client");

    // WETH on Ethereum
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let price = client.get_usd_price("ethereum", weth).await.unwrap();

    assert!(price["data"]["usd_price"].as_f64().unwrap() > 0.0);
}

#[tokio::test]
#[ignore]
async fn test_prices_api_chains() {
    let client = PricesClient::new().expect("Failed to create client");

    let chains = client.get_chains().await.unwrap();
    assert!(chains["data"].is_array() || chains["data"].is_object());
}

#[tokio::test]
#[ignore]
async fn test_router_api() {
    let client = Client::new().expect("Failed to create client");

    // Build router from Ethereum pools
    let pools = client.pools().get_all_on_chain("ethereum").await.unwrap();
    let router = RouterApi::new("ethereum", &pools.data.pool_data);

    let stats = router.stats();
    println!(
        "Router graph: {} tokens, {} edges",
        stats.token_count, stats.edge_count
    );

    assert!(stats.token_count > 100);
    assert!(stats.edge_count > 200);

    // Test finding routes between major stablecoins (3pool tokens)
    let dai = "0x6B175474E89094C44Da98b954EedeAC495271d0F";
    let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    let routes = router.find_routes(dai, usdc);
    println!("Found {} routes from DAI to USDC", routes.len());

    assert!(!routes.is_empty(), "Should find at least one route");

    // Best route should be direct (1 hop via 3pool)
    let best = &routes[0];
    println!(
        "Best route: {} hops, min TVL: ${:.0}",
        best.steps.len(),
        best.min_tvl
    );

    // Test calldata encoding
    let calldata = router
        .encode_swap(best, "1000000000000000000", "990000")
        .unwrap();
    assert_eq!(calldata.len(), 1220); // Expected calldata size
    println!("Generated {} bytes of calldata", calldata.len());
}

#[tokio::test]
#[ignore]
async fn test_router_via_client() {
    let client = Client::new().expect("Failed to create client");

    // Test the convenience method
    let router = client.build_router("ethereum").await.unwrap();

    // WETH to USDC via wrapped ETH pools
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let usdc = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

    let best = router.find_best_route(weth, usdc);
    assert!(best.is_some(), "Should find route from WETH to USDC");

    let best = best.unwrap();
    println!(
        "WETH -> USDC: {} hops via pools: {:?}",
        best.steps.len(),
        best.steps.iter().map(|s| &s.pool_id).collect::<Vec<_>>()
    );
}
