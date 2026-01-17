//! Example: Get a swap quote from CoW Protocol
//!
//! This example demonstrates getting a quote for swapping WETH to USDC.
//! No API key required.

use cowp::{Client, Chain, QuoteRequest};

// Example wallet address (Vitalik's public address)
const EXAMPLE_WALLET: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";

// Token addresses on Ethereum mainnet
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Test 1: Sell quote (exact input)
    println!("=== Sell Quote (1 WETH -> USDC) ===");
    let request = QuoteRequest::sell(
        WETH,
        USDC,
        "1000000000000000000", // 1 WETH
        EXAMPLE_WALLET,
    );

    match client.get_quote(Some(Chain::Mainnet), &request).await {
        Ok(quote) => {
            println!("You will receive: {} USDC (minimal units)", quote.quote.buy_amount);
            println!("Fee amount: {} WETH", quote.quote.fee_amount);
        }
        Err(e) => println!("Error getting quote: {}", e),
    }

    // Test 2: Buy quote (exact output)
    println!("\n=== Buy Quote (WETH -> 1000 USDC) ===");
    let request = QuoteRequest::buy(
        WETH,
        USDC,
        "1000000000", // 1000 USDC (6 decimals)
        EXAMPLE_WALLET,
    );

    match client.get_quote(Some(Chain::Mainnet), &request).await {
        Ok(quote) => {
            println!("You will pay: {} WETH (minimal units)", quote.quote.sell_amount);
        }
        Err(e) => println!("Error getting quote: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
