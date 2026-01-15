//! Test Networks API

use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("Testing Networks API...\n");

    let networks = client.networks().supported().await?;
    println!("Found {} supported networks\n", networks.len());

    // Print first 10
    println!("Sample networks:");
    for network in networks.iter().take(10) {
        println!("  {} (chain {})", network.network_name, network.chain_id);
        println!("    - Slug: {}", network.slug());
        println!("    - Simulation: {}", network.simulation_supported());
        println!("    - VNet: {}", network.vnet_supported());
        println!("    - Testnet: {}", network.is_testnet());
    }

    // Test get by chain ID
    println!("\n--- Get Mainnet (chain 1) ---");
    if let Some(mainnet) = client.networks().get("1").await? {
        println!("Found: {} (slug: {})", mainnet.network_name, mainnet.slug());
    }

    // Test mainnets filter
    println!("\n--- Mainnets ---");
    let mainnets = client.networks().mainnets().await?;
    println!("Found {} mainnets", mainnets.len());

    // Test testnets filter
    println!("\n--- Testnets ---");
    let testnets = client.networks().testnets().await?;
    println!("Found {} testnets", testnets.len());

    println!("\n✓ Networks API: SUCCESS");
    Ok(())
}
