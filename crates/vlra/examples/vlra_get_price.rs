//! Example: Get swap prices from Velora (ParaSwap)
//!
//! This example demonstrates getting optimal swap prices.
//! No API key required for basic usage.

use vlra::{Chain, Client, PriceRequest};

// Token addresses on Ethereum mainnet
const NATIVE_ETH: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Get price for swapping 1 ETH to USDC
    println!("=== Price: 1 ETH -> USDC ===");
    let request = PriceRequest::sell(
        NATIVE_ETH,
        USDC,
        "1000000000000000000", // 1 ETH in wei
    );

    match client.get_price(Chain::Ethereum, &request).await {
        Ok(response) => {
            println!("Source amount: {} ETH", response.price_route.src_amount);
            println!(
                "Dest amount: {} USDC (minimal units)",
                response.price_route.dest_amount
            );
            println!(
                "Gas cost: {} USD",
                response.price_route.gas_cost_usd.unwrap_or_default()
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    // Get price for WETH to USDC
    println!("\n=== Price: 0.5 WETH -> USDC ===");
    let request = PriceRequest::sell(
        WETH,
        USDC,
        "500000000000000000", // 0.5 WETH
    );

    match client.get_price(Chain::Ethereum, &request).await {
        Ok(response) => {
            println!("Output: {} USDC", response.price_route.dest_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    // Test on Arbitrum
    println!("\n=== Price on Arbitrum ===");
    let arb_weth = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
    let arb_usdc = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831";

    let request = PriceRequest::sell(
        arb_weth,
        arb_usdc,
        "100000000000000000", // 0.1 WETH
    );

    match client.get_price(Chain::Arbitrum, &request).await {
        Ok(response) => {
            println!("Output: {} USDC", response.price_route.dest_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
