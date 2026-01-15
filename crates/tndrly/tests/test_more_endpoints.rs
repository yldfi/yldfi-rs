//! Tests for additional API endpoints
//!
//! Run with: cargo test -p tndrly --test test_more_endpoints -- --ignored

use tndrly::Client;

#[tokio::test]
#[ignore]
async fn test_contracts_get() {
    let client = Client::from_env().expect("Failed to create client");

    // USDC on mainnet
    let contract = client
        .contracts()
        .get("1", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
        .await
        .unwrap();

    println!(
        "Contract: {} ({})",
        contract.contract_name().unwrap_or("unknown"),
        contract.address().unwrap_or("unknown")
    );
}

#[tokio::test]
#[ignore]
async fn test_vnets_list_and_get() {
    let client = Client::from_env().expect("Failed to create client");

    let vnets = client.vnets().list(None).await.unwrap();
    println!("Found {} vnets", vnets.len());

    if let Some(vnet) = vnets.first() {
        println!("First vnet: {}", vnet.id);
        let v = client.vnets().get(&vnet.id).await.unwrap();
        println!("Got vnet: {}", v.display_name);
    }
}

#[tokio::test]
#[ignore]
async fn test_alerts_list_webhooks() {
    let client = Client::from_env().expect("Failed to create client");

    let resp = client.alerts().list_webhooks().await.unwrap();
    println!("Found {} webhooks", resp.webhooks.len());
}

#[tokio::test]
#[ignore]
async fn test_simulation_list() {
    let client = Client::from_env().expect("Failed to create client");

    let resp = client.simulation().list(1, 10).await.unwrap();
    println!("Found {} simulations", resp.simulations.len());
}

#[tokio::test]
#[ignore]
async fn test_actions_list_and_get() {
    let client = Client::from_env().expect("Failed to create client");

    let resp = client.actions().list().await.unwrap();
    println!("Found {} actions", resp.actions.len());

    if let Some(action) = resp.actions.first() {
        let a = client.actions().get(&action.id).await.unwrap();
        println!("Got action: {}", a.name);
    }
}
