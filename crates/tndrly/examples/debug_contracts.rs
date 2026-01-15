//! Debug Contracts API responses
//!
//! Run with: cargo run --example debug_contracts
//!
//! Requires: TENDERLY_ACCESS_KEY, TENDERLY_ACCOUNT, TENDERLY_PROJECT

use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("Testing raw /contracts endpoint...");
    match client.get_raw("/contracts").await {
        Ok(json) => {
            println!("Raw response:\n{}", serde_json::to_string_pretty(&json)?);
        }
        Err(e) => {
            println!("Error getting raw: {:?}", e);
        }
    }

    println!("\nTesting raw /actions endpoint...");
    match client.get_raw("/actions").await {
        Ok(json) => {
            println!("Raw response:\n{}", serde_json::to_string_pretty(&json)?);
        }
        Err(e) => {
            println!("Error getting raw: {:?}", e);
        }
    }

    Ok(())
}
