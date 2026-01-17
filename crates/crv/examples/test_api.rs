//! Example: Query Curve Finance API
//!
//! This example demonstrates various Curve API endpoints.
//! No API key required - public API.

use crv::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Test 1: Get all pools on Ethereum
    println!("=== Curve Pools on Ethereum ===");
    match client.pools().get_all_on_chain("ethereum").await {
        Ok(response) => {
            let pools = &response.data.pool_data;
            println!("Found {} pools", pools.len());
            println!("First 3 pools:");
            for pool in pools.iter().take(3) {
                println!("  - {:?}", pool.address());
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 2: Get lending vaults
    println!("\n=== Curve Lending Vaults ===");
    match client.lending().get_all().await {
        Ok(response) => {
            println!(
                "Found {} lending vaults",
                response.data.lending_vault_data.len()
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 3: Get crvUSD total supply
    println!("\n=== crvUSD Supply ===");
    match client.crvusd().get_total_supply_number().await {
        Ok(supply) => {
            println!("Total crvUSD supply: ${:.2}M", supply / 1_000_000.0);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
