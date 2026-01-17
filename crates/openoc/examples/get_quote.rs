//! Example: Get swap quotes from OpenOcean
//!
//! This example demonstrates getting quotes for token swaps.
//! No API key required.

use openoc::{Client, Chain, QuoteRequest};

// Token addresses on Ethereum mainnet
const NATIVE_ETH: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Get quote for swapping 1 ETH to USDC
    println!("=== Quote: 1 ETH -> USDC ===");
    let request = QuoteRequest::new(
        NATIVE_ETH,
        USDC,
        "1000000000000000000", // 1 ETH in wei
    ).with_slippage(1.0); // 1% slippage

    match client.get_quote(Chain::Eth, &request).await {
        Ok(quote) => {
            println!("Input: {} ETH", quote.in_amount);
            println!("Output: {} USDC (minimal units)", quote.out_amount);
            if let Some(price_impact) = &quote.price_impact {
                println!("Price impact: {}%", price_impact);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get quote for WETH to USDC
    println!("\n=== Quote: 0.5 WETH -> USDC ===");
    let request = QuoteRequest::new(
        WETH,
        USDC,
        "500000000000000000", // 0.5 WETH
    );

    match client.get_quote(Chain::Eth, &request).await {
        Ok(quote) => {
            println!("Output: {} USDC", quote.out_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test on Polygon
    println!("\n=== Quote on Polygon: MATIC -> USDC ===");
    let matic = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
    let polygon_usdc = "0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359";

    let request = QuoteRequest::new(
        matic,
        polygon_usdc,
        "10000000000000000000", // 10 MATIC
    );

    match client.get_quote(Chain::Polygon, &request).await {
        Ok(quote) => {
            println!("Output: {} USDC", quote.out_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
