//! Example: Get swap quotes from 1inch
//!
//! This example demonstrates getting quotes for token swaps.
//!
//! **Requirements:**
//! - API key from https://portal.1inch.dev
//! - Set ONEINCH_API_KEY environment variable

use oinch::{Chain, Client, QuoteRequest};
use std::env;

// Token addresses on Ethereum mainnet
const NATIVE_ETH: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        env::var("ONEINCH_API_KEY").expect("ONEINCH_API_KEY environment variable must be set");

    let client = Client::new(&api_key)?;

    // Get quote for swapping 1 ETH to USDC
    println!("=== Quote: 1 ETH -> USDC ===");
    let request = QuoteRequest::new(
        NATIVE_ETH,
        USDC,
        "1000000000000000000", // 1 ETH in wei
    );

    match client.get_quote(Chain::Ethereum, &request).await {
        Ok(quote) => {
            println!("Output: {} USDC", quote.to_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get liquidity sources
    println!("\n=== Liquidity Sources ===");
    match client.get_liquidity_sources(Chain::Ethereum).await {
        Ok(sources) => {
            println!("Found {} liquidity sources", sources.len());
            for source in sources.iter().take(5) {
                println!("  {}: {}", source.id, source.title);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
