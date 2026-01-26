//! Integration tests for unswp client
//!
//! These tests verify the client can be created and configured correctly.
//! Tests marked with #[ignore] require network access.

use unswp::{Client, Config};

#[test]
fn test_mainnet_client_creation() {
    let client = Client::mainnet("https://eth.llamarpc.com");
    assert!(client.is_ok(), "Mainnet client should be created: {:?}", client.err());
}

#[test]
fn test_client_with_config() {
    let config = Config::new("https://eth.llamarpc.com")
        .with_subgraph_key("test-api-key");
    let client = Client::new(config);
    assert!(client.is_ok(), "Client with config should be created: {:?}", client.err());
}

#[test]
fn test_client_with_subgraph() {
    let client = Client::mainnet_with_subgraph(
        "https://eth.llamarpc.com",
        "test-api-key"
    );
    assert!(client.is_ok(), "Client with subgraph should be created: {:?}", client.err());

    let client = client.unwrap();
    assert!(client.has_subgraph(), "Should have subgraph configured");
}

#[test]
fn test_client_without_subgraph() {
    let client = Client::mainnet("https://eth.llamarpc.com")
        .expect("Client creation failed");

    assert!(!client.has_subgraph(), "Should not have subgraph by default");
}

#[test]
fn test_well_known_pools() {
    use unswp::pools;

    // Verify well-known pool addresses are non-zero
    assert!(!pools::MAINNET_WETH_USDC_005.is_zero());
    assert!(!pools::MAINNET_WETH_USDC_030.is_zero());
    assert!(!pools::MAINNET_WETH_USDT_005.is_zero());
    assert!(!pools::MAINNET_WBTC_WETH_030.is_zero());
}

#[test]
fn test_well_known_tokens() {
    use unswp::tokens;

    // Verify well-known token addresses are non-zero
    assert!(!tokens::MAINNET_WETH.is_zero());
    assert!(!tokens::MAINNET_USDC.is_zero());
    assert!(!tokens::MAINNET_USDT.is_zero());
    assert!(!tokens::MAINNET_WBTC.is_zero());
    assert!(!tokens::MAINNET_DAI.is_zero());
}

#[test]
fn test_factory_addresses() {
    use unswp::factories;

    // Verify factory addresses are non-zero
    assert!(!factories::MAINNET.is_zero());
    assert!(!factories::BASE.is_zero());
    assert!(!factories::v2::MAINNET.is_zero());
    assert!(!factories::v3::MAINNET.is_zero());
    assert!(!factories::v4::MAINNET.is_zero());
}

#[test]
fn test_sdk_reexports() {
    // Verify SDK crates are re-exported and accessible
    // These just need to exist and compile to verify re-exports work
    let _ = std::any::type_name::<unswp::sdk_core::prelude::Currency>();
    let _ = std::any::type_name::<unswp::v2_sdk::prelude::Pair>();
    let _ = std::any::type_name::<unswp::v3_sdk::prelude::Pool>();
}

// Integration tests that require network access
// Run with: cargo test -p unswp --test integration -- --ignored

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_block_number() {
    let client = Client::mainnet("https://eth.llamarpc.com")
        .expect("Client creation failed");

    let block = client.get_block_number().await;
    assert!(block.is_ok(), "Should fetch block number: {:?}", block.err());
    assert!(block.unwrap() > 0, "Block number should be positive");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_pool_state() {
    use unswp::pools;

    let client = Client::mainnet("https://eth.llamarpc.com")
        .expect("Client creation failed");

    let state = client.get_pool_state(pools::MAINNET_WETH_USDC_005).await;
    assert!(state.is_ok(), "Should fetch pool state: {:?}", state.err());

    let state = state.unwrap();
    assert!(state.tick != 0, "Tick should be non-zero for active pool");
}

#[tokio::test]
#[ignore = "requires network access"]
async fn test_fetch_liquidity() {
    use unswp::pools;

    let client = Client::mainnet("https://eth.llamarpc.com")
        .expect("Client creation failed");

    let liquidity = client.get_liquidity(pools::MAINNET_WETH_USDC_005).await;
    assert!(liquidity.is_ok(), "Should fetch liquidity: {:?}", liquidity.err());
    assert!(liquidity.unwrap() > 0, "Liquidity should be positive for active pool");
}
