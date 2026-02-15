//! Example: Get swap prices from 0x (ZeroEx)
//!
//! This example demonstrates getting indicative prices for token swaps.
//!
//! **Requirements:**
//! - API key from https://0x.org/docs/introduction/getting-started
//! - Set ZEROX_API_KEY environment variable

use std::env;
use zrxswap::{Chain, Client, QuoteRequest};

// Token addresses on Ethereum mainnet
const NATIVE_ETH: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        env::var("ZEROX_API_KEY").expect("ZEROX_API_KEY environment variable must be set");

    let client = Client::with_api_key(&api_key)?;

    // Get indicative price for swapping 1 ETH to USDC
    println!("=== Indicative Price: 1 ETH -> USDC ===");
    let request = QuoteRequest::sell(
        NATIVE_ETH,
        USDC,
        "1000000000000000000", // 1 ETH in wei
    );

    match client.get_price(Chain::Ethereum, &request).await {
        Ok(price) => {
            println!("Sell amount: {} ETH", price.sell_amount);
            println!("Buy amount: {} USDC", price.buy_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get price for WETH to USDC
    println!("\n=== Price: 0.5 WETH -> USDC ===");
    let request = QuoteRequest::sell(
        WETH,
        USDC,
        "500000000000000000", // 0.5 WETH
    );

    match client.get_price(Chain::Ethereum, &request).await {
        Ok(price) => {
            println!("Buy amount: {} USDC", price.buy_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
