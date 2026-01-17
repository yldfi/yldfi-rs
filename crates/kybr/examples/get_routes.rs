//! Example: Get optimal swap routes from KyberSwap
//!
//! This example demonstrates getting a swap route for WETH to USDC.
//! No API key required.

use kybr::{Client, Chain, RouteRequest};

// Token addresses on Ethereum mainnet
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new()?;

    // Get routes for swapping 1 WETH to USDC
    println!("=== Get Swap Routes (1 WETH -> USDC) ===");
    let request = RouteRequest::new(
        WETH,
        USDC,
        "1000000000000000000", // 1 WETH
    ).with_slippage_bps(50); // 0.5% slippage

    match client.get_routes(Chain::Ethereum, &request).await {
        Ok(route) => {
            println!("Route found!");
            println!("  Input: {} (WETH)", route.amount_in);
            println!("  Output: {} (USDC minimal units)", route.amount_out);
            if let Some(gas) = &route.gas {
                println!("  Gas estimate: {} wei", gas);
            }
        }
        Err(e) => println!("Error getting routes: {}", e),
    }

    // Test with different chains
    println!("\n=== Get Routes on Arbitrum ===");
    // Arbitrum WETH and USDC addresses
    let arb_weth = "0x82aF49447D8a07e3bd95BD0d56f35241523fBab1";
    let arb_usdc = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831";

    let request = RouteRequest::new(
        arb_weth,
        arb_usdc,
        "100000000000000000", // 0.1 WETH
    );

    match client.get_routes(Chain::Arbitrum, &request).await {
        Ok(route) => {
            println!("Route found on Arbitrum!");
            println!("  Output: {} USDC", route.amount_out);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
