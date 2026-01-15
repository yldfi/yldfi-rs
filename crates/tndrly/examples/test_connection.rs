//! Example demonstrating tndrly connection and simulation
//!
//! Run with: cargo run --example test_connection
//!
//! Requires: TENDERLY_ACCESS_KEY, TENDERLY_ACCOUNT, TENDERLY_PROJECT

use tndrly::simulation::{SimulationRequest, SimulationType};
use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("Testing tndrly connection...");
    println!("Account: {}", client.account());
    println!("Project: {}", client.project());

    // Test: List simulations
    println!("\n--- Listing saved simulations ---");
    match client.simulation().list(1, 10).await {
        Ok(response) => {
            println!("Found {} simulations", response.simulations.len());
            for (i, sim) in response.simulations.iter().take(5).enumerate() {
                println!("  {}. ID: {} (status: {:?})", i + 1, sim.id, sim.status);
            }
        }
        Err(e) => println!("Error listing simulations: {}", e),
    }

    // Test: Simple simulation (USDC balanceOf on vitalik.eth)
    println!("\n--- Running test simulation ---");
    let request = SimulationRequest::new(
        "0x0000000000000000000000000000000000000000",
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045", // balanceOf(vitalik.eth)
    )
    .network_id("1")
    .simulation_type(SimulationType::Quick);

    match client.simulation().simulate(&request).await {
        Ok(result) => {
            println!("Simulation successful!");
            println!("  Status: {}", result.simulation.status);
            println!("  Gas used: {}", result.simulation.gas_used);
            println!("  Block: {}", result.simulation.block_number);
        }
        Err(e) => println!("Simulation error: {}", e),
    }

    Ok(())
}
