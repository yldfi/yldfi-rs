//! Tests for all API modules
//!
//! Run with: cargo test -p tndrly --test test_all_apis -- --ignored

use tndrly::Client;

#[tokio::test]
#[ignore]
async fn test_networks_api() {
    let client = Client::from_env().expect("Failed to create client");

    let networks = client.networks().supported().await.unwrap();
    println!("Found {} supported networks", networks.len());
    assert!(!networks.is_empty());

    for network in networks.iter().take(5) {
        println!(
            "  - {} (chain {}, testnet: {})",
            network.network_name,
            network.chain_id,
            network.is_testnet()
        );
    }
}

#[tokio::test]
#[ignore]
async fn test_wallets_api() {
    let client = Client::from_env().expect("Failed to create client");

    // Note: Wallets API only has add() and get(address, network) - no list endpoint
    // This may fail if wallet not in project
    let result = client
        .wallets()
        .get("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "1")
        .await;

    match result {
        Ok(wallet) => {
            println!(
                "  - {} (balance: {})",
                wallet.address().unwrap_or("unknown"),
                wallet.balance().unwrap_or("unknown")
            );
        }
        Err(e) => println!("Expected if wallet not in project: {}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_delivery_channels_api() {
    let client = Client::from_env().expect("Failed to create client");

    let response = client.delivery_channels().list_project().await.unwrap();
    println!(
        "Found {} delivery channels",
        response.delivery_channels.len()
    );
}

#[tokio::test]
#[ignore]
async fn test_actions_api() {
    let client = Client::from_env().expect("Failed to create client");

    let response = client.actions().list().await.unwrap();
    println!("Found {} actions", response.actions.len());
}
