//! Test the Contracts API
//!
//! Run with: cargo run --example test_contracts_api
//!
//! Requires environment variables:
//!   TENDERLY_ACCESS_KEY, TENDERLY_ACCOUNT, TENDERLY_PROJECT

use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("Testing contracts().list()...");
    match client.contracts().list(None).await {
        Ok(contracts) => {
            println!("SUCCESS! Found {} contracts", contracts.len());
            for c in contracts.iter().take(3) {
                println!("  - ID: {}", c.id);
                println!("    Address: {}", c.address().unwrap_or("unknown"));
                println!("    Network: {}", c.network_id().unwrap_or("unknown"));
                println!("    Verified: {}", c.is_verified());
            }
        }
        Err(e) => {
            println!("ERROR: {:?}", e);
        }
    }

    Ok(())
}
