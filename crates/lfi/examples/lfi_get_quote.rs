//! Example: Get cross-chain swap quotes from LI.FI
//!
//! This example demonstrates getting quotes for cross-chain swaps.
//! An integrator ID is recommended but not strictly required.

use lfi::{chains, Client, QuoteRequest};

// Token addresses
const NATIVE_ETH: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";
const USDC_ARB: &str = "0xaf88d065e77c8cC2239327C5EDb3A432268e5831"; // USDC on Arbitrum

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client with integrator identifier
    let client = Client::with_integrator("example-app")?;

    // Get quote for cross-chain swap: ETH on Ethereum -> USDC on Arbitrum
    println!("=== Cross-chain Quote: ETH (Ethereum) -> USDC (Arbitrum) ===");
    let request = QuoteRequest::new(
        chains::ETHEREUM,
        chains::ARBITRUM,
        NATIVE_ETH,
        USDC_ARB,
        "100000000000000000",                         // 0.1 ETH
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", // Example wallet
    )
    .with_slippage(0.5); // 0.5% slippage

    match client.get_quote(&request).await {
        Ok(quote) => {
            println!(
                "From: {} {} on chain {}",
                quote.action.from_amount,
                quote.action.from_token.symbol,
                quote.action.from_chain_id
            );
            println!(
                "To: {} {} on chain {}",
                quote.estimate.to_amount, quote.action.to_token.symbol, quote.action.to_chain_id
            );
            println!(
                "Estimated time: {} seconds",
                quote.estimate.execution_duration.unwrap_or(0)
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    // Same-chain swap on Arbitrum
    println!("\n=== Same-chain Quote: ETH -> USDC on Arbitrum ===");
    let arb_eth = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

    let request = QuoteRequest::new(
        chains::ARBITRUM,
        chains::ARBITRUM,
        arb_eth,
        USDC_ARB,
        "100000000000000000", // 0.1 ETH
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
    );

    match client.get_quote(&request).await {
        Ok(quote) => {
            println!("Output: {} USDC", quote.estimate.to_amount);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nDone!");
    Ok(())
}
