//! Tests for Networks API
//!
//! Run with: cargo test -p tndrly --test test_networks -- --ignored

use tndrly::Client;

#[tokio::test]
#[ignore]
async fn test_networks_supported() {
    let client = Client::from_env().expect("Failed to create client");

    let networks = client.networks().supported().await.unwrap();
    println!("Found {} supported networks", networks.len());
    assert!(!networks.is_empty());

    for network in networks.iter().take(10) {
        println!("  {} (chain {})", network.network_name, network.chain_id);
        println!("    - Slug: {}", network.slug());
        println!("    - Simulation: {}", network.simulation_supported());
        println!("    - VNet: {}", network.vnet_supported());
        println!("    - Testnet: {}", network.is_testnet());
    }
}

#[tokio::test]
#[ignore]
async fn test_networks_get_mainnet() {
    let client = Client::from_env().expect("Failed to create client");

    let mainnet = client.networks().get("1").await.unwrap();
    assert!(mainnet.is_some());
    let mainnet = mainnet.unwrap();
    println!("Found: {} (slug: {})", mainnet.network_name, mainnet.slug());
}

#[tokio::test]
#[ignore]
async fn test_networks_mainnets() {
    let client = Client::from_env().expect("Failed to create client");

    let mainnets = client.networks().mainnets().await.unwrap();
    println!("Found {} mainnets", mainnets.len());
    assert!(!mainnets.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_networks_testnets() {
    let client = Client::from_env().expect("Failed to create client");

    let testnets = client.networks().testnets().await.unwrap();
    println!("Found {} testnets", testnets.len());
    assert!(!testnets.is_empty());
}
