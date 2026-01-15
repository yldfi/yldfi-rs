//! Comprehensive integration tests for all API endpoints
//!
//! Run with: cargo test -p tndrly --test integration_test -- --ignored

use tndrly::simulation::SimulationRequest;
use tndrly::wallets::AddWalletRequest;
use tndrly::Client;

#[tokio::test]
#[ignore]
async fn test_networks_api() {
    let client = Client::from_env().expect("Failed to create client");

    let networks = client.networks().supported().await.unwrap();
    assert!(!networks.is_empty());
    println!("supported() - {} networks", networks.len());

    let mainnets = client.networks().mainnets().await.unwrap();
    assert!(!mainnets.is_empty());
    println!("mainnets() - {} mainnets", mainnets.len());

    let testnets = client.networks().testnets().await.unwrap();
    assert!(!testnets.is_empty());
    println!("testnets() - {} testnets", testnets.len());

    let mainnet = client.networks().get("1").await.unwrap();
    assert!(mainnet.is_some());
    println!("get(\"1\") - {}", mainnet.unwrap().network_name);
}

#[tokio::test]
#[ignore]
async fn test_wallets_api() {
    let client = Client::from_env().expect("Failed to create client");

    let wallets = client.wallets().list().await.unwrap();
    println!("list() - {} wallets", wallets.len());

    // Try to add a wallet (may already exist)
    let add_request = AddWalletRequest::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .network("1")
        .display_name("vitalik.eth");

    match client.wallets().add(&add_request).await {
        Ok(resp) => println!("add() - added wallet, {} contracts", resp.contracts.len()),
        Err(e) => println!("add() - {} (expected if exists)", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_delivery_channels_api() {
    let client = Client::from_env().expect("Failed to create client");

    let project = client.delivery_channels().list_project().await.unwrap();
    println!("list_project() - {} channels", project.delivery_channels.len());

    let account = client.delivery_channels().list_account().await.unwrap();
    println!("list_account() - {} channels", account.delivery_channels.len());
}

#[tokio::test]
#[ignore]
async fn test_actions_api() {
    let client = Client::from_env().expect("Failed to create client");

    let resp = client.actions().list().await.unwrap();
    println!("list() - {} actions", resp.actions.len());
}

#[tokio::test]
#[ignore]
async fn test_alerts_api() {
    let client = Client::from_env().expect("Failed to create client");

    let resp = client.alerts().list().await.unwrap();
    println!("list() - {} alerts", resp.alerts.len());
}

#[tokio::test]
#[ignore]
async fn test_contracts_api() {
    let client = Client::from_env().expect("Failed to create client");

    let contracts = client.contracts().list(None).await.unwrap();
    println!("list() - {} contracts", contracts.len());
}

#[tokio::test]
#[ignore]
async fn test_vnets_api() {
    let client = Client::from_env().expect("Failed to create client");

    let vnets = client.vnets().list(None).await.unwrap();
    println!("list() - {} vnets", vnets.len());
}

#[tokio::test]
#[ignore]
async fn test_simulation_api() {
    let client = Client::from_env().expect("Failed to create client");

    // Simple ETH transfer simulation
    let sim_request = SimulationRequest::new(
        "0xd8da6bf26964af9d7eed9e03e53415d37aa96045", // from: vitalik.eth
        "0x0000000000000000000000000000000000000000", // to: zero address
        "",                                           // input: empty calldata
    )
    .network_id("1")
    .value("0");

    let result = client.simulation().simulate(&sim_request).await.unwrap();
    println!("simulate() - status: {}", result.simulation.status);
}
