//! Example: Query Alchemy API
//!
//! This example demonstrates various Alchemy API endpoints.
//!
//! **Requirements:**
//! - API key from https://www.alchemy.com/
//! - Set ALCHEMY_API_KEY environment variable

use alcmy::{Client, Network};
use std::env;

// Example wallet address (Vitalik)
const WALLET: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY environment variable must be set");

    let client = Client::new(&api_key, Network::EthMainnet)?;

    // Test 1: Get NFTs for owner
    println!("=== NFTs for {} ===", WALLET);
    match client.nft().get_nfts_for_owner(WALLET).await {
        Ok(response) => {
            println!(
                "Found {} NFTs (total: {})",
                response.owned_nfts.len(),
                response.total_count
            );
            for nft in response.owned_nfts.iter().take(3) {
                println!("  - {:?}", nft.name);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 2: Get token balances
    println!("\n=== Token Balances ===");
    match client.token().get_token_balances(WALLET).await {
        Ok(response) => {
            println!("Found {} token balances", response.token_balances.len());
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
