//! Test all APIs example

use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("Testing Networks API...");
    match client.networks().supported().await {
        Ok(networks) => {
            println!("Found {} supported networks", networks.len());
            for network in networks.iter().take(5) {
                println!(
                    "  - {} (chain {}, testnet: {})",
                    network.network_name,
                    network.chain_id,
                    network.is_testnet()
                );
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nTesting Wallets API (get specific wallet)...");
    // Note: Wallets API only has add() and get(address, network) - no list endpoint
    match client
        .wallets()
        .get("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "1")
        .await
    {
        Ok(wallet) => {
            println!(
                "  - {} (balance: {})",
                wallet.address().unwrap_or("unknown"),
                wallet.balance().unwrap_or("unknown")
            );
        }
        Err(e) => println!("Error (expected if wallet not in project): {}", e),
    }

    println!("\nTesting Delivery Channels API...");
    match client.delivery_channels().list_project().await {
        Ok(response) => {
            println!(
                "Found {} delivery channels",
                response.delivery_channels.len()
            );
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\nTesting Actions API...");
    match client.actions().list().await {
        Ok(response) => {
            println!("Found {} actions", response.actions.len());
        }
        Err(e) => println!("Error: {}", e),
    }

    Ok(())
}
