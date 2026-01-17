//! Example: Query data from Moralis Web3 API
//!
//! This example demonstrates various Moralis API endpoints.
//!
//! **Requirements:**
//! - API key from https://moralis.io/
//! - Set MORALIS_API_KEY environment variable

use mrls::Client;

// Example wallet address (Vitalik)
const WALLET: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    // Test 1: Get native balance
    println!("=== Native Balance ===");
    match client.wallet().get_native_balance(WALLET, Some("eth")).await {
        Ok(balance) => println!("Balance: {} wei", balance.balance),
        Err(e) => println!("Error: {}", e),
    }

    // Test 2: Get token price
    println!("\n=== Token Price (WETH) ===");
    match client.token().get_price(WETH, Some("eth")).await {
        Ok(price) => {
            println!("WETH Price: ${:?}", price.usd_price);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 3: Get token metadata
    println!("\n=== Token Metadata (WETH) ===");
    match client.token().get_metadata(WETH, Some("eth")).await {
        Ok(metadata) => {
            println!("Name: {:?}", metadata.name);
            println!("Symbol: {:?}", metadata.symbol);
            println!("Decimals: {:?}", metadata.decimals);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test 4: Resolve ENS domain
    println!("\n=== ENS Resolution ===");
    match client.resolve().resolve_domain("vitalik.eth").await {
        Ok(resolved) => println!("vitalik.eth => {:?}", resolved.address),
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
