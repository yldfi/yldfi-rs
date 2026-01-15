//! Connection and basic simulation tests
//!
//! Run with: cargo test -p tndrly --test test_connection -- --ignored
//!
//! Requires: TENDERLY_ACCESS_KEY, TENDERLY_ACCOUNT, TENDERLY_PROJECT

use tndrly::simulation::{SimulationRequest, SimulationType};
use tndrly::Client;

#[tokio::test]
#[ignore]
async fn test_connection_and_list_simulations() {
    let client = Client::from_env().expect("Failed to create client");

    println!("Testing tndrly connection...");
    println!("Account: {}", client.account());
    println!("Project: {}", client.project());

    let response = client.simulation().list(1, 10).await.unwrap();
    println!("Found {} simulations", response.simulations.len());
    for (i, sim) in response.simulations.iter().take(5).enumerate() {
        println!("  {}. ID: {} (status: {:?})", i + 1, sim.id, sim.status);
    }
}

#[tokio::test]
#[ignore]
async fn test_simple_simulation() {
    let client = Client::from_env().expect("Failed to create client");

    // Simple simulation: USDC balanceOf on vitalik.eth
    let request = SimulationRequest::new(
        "0x0000000000000000000000000000000000000000",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045", // balanceOf(vitalik.eth)
    )
    .network_id("1")
    .simulation_type(SimulationType::Quick);

    let result = client.simulation().simulate(&request).await.unwrap();
    println!("Simulation successful!");
    println!("  Status: {}", result.simulation.status);
    println!("  Gas used: {}", result.simulation.gas_used);
    println!("  Block: {}", result.simulation.block_number);
    assert!(result.simulation.status);
}
