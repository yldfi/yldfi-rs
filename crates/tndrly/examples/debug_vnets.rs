//! Debug VNets API responses
//!
//! Run with: cargo run --example debug_vnets
//!
//! Requires: TENDERLY_ACCESS_KEY, TENDERLY_ACCOUNT, TENDERLY_PROJECT

use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("Testing raw /vnets endpoint...");
    match client.get_raw("/vnets").await {
        Ok(json) => {
            println!("Raw response:\n{}", serde_json::to_string_pretty(&json)?);
        }
        Err(e) => {
            println!("Error getting raw: {:?}", e);
        }
    }

    Ok(())
}
