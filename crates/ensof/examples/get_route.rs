//! Example: Get swap routes from Enso Finance
//!
//! This example demonstrates getting routes for token swaps.
//!
//! **Requirements:**
//! - API key from https://www.enso.finance/
//! - Set ENSO_API_KEY environment variable

use ensof::{Client, Chain, RouteRequest};
use std::env;

// Token addresses on Ethereum mainnet
const NATIVE_ETH: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

// Example wallet address
const WALLET: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("ENSO_API_KEY")
        .expect("ENSO_API_KEY environment variable must be set");

    let client = Client::with_api_key(&api_key)?;

    // Get route for swapping 1 ETH to USDC
    println!("=== Route: 1 ETH -> USDC ===");
    let request = RouteRequest::new(
        Chain::Ethereum.chain_id(),
        WALLET,
        NATIVE_ETH,
        USDC,
        "1000000000000000000", // 1 ETH in wei
        100, // 1% slippage in basis points
    );

    match client.get_route(&request).await {
        Ok(response) => {
            println!("Output amount: {}", response.amount_out);
            println!("Transaction to: {}", response.tx.to);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get token price
    println!("\n=== Token Price: USDC ===");
    match client.get_token_price(1, USDC).await {
        Ok(price) => {
            println!("USDC price: ${}", price.price);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
