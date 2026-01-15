//! Test the Curve API clients against live endpoints

use curve_api::{Client, PricesClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test main API client
    println!("=== Testing api.curve.finance ===");
    let client = Client::new()?;

    // Get pools on ethereum
    let pools = client.pools().get_all_on_chain("ethereum").await?;
    println!(
        "✓ Found {} pools on Ethereum",
        pools.data.pool_data.len()
    );

    // Get lending vaults
    let vaults = client.lending().get_all().await?;
    println!(
        "✓ Found {} lending vaults",
        vaults.data.lending_vault_data.len()
    );

    // Test prices API client
    println!("\n=== Testing prices.curve.finance ===");
    let prices = PricesClient::new()?;

    // Ping
    let ping = prices.ping().await?;
    println!("✓ Ping response: {}", ping);

    // Get chains
    let _chains = prices.get_chains().await?;
    println!("✓ Got chains data");

    // Get USD price for WETH on ethereum
    let weth = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";
    let price = prices.get_usd_price("ethereum", weth).await?;
    println!("✓ WETH price: {}", price);

    println!("\n✅ All tests passed!");
    Ok(())
}
