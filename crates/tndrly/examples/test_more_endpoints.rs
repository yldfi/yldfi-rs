//! Test additional endpoints

use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("=== TESTING MORE ENDPOINTS ===\n");

    // ============ CONTRACTS API ============
    println!("## Contracts API");

    // get() - get a specific contract
    match client
        .contracts()
        .get("1", "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48")
        .await
    {
        Ok(contract) => {
            println!(
                "  ✅ get() - {} ({})",
                contract.contract_name().unwrap_or("unknown"),
                contract.address().unwrap_or("unknown")
            );
        }
        Err(e) => println!("  ❌ get() - {}", e),
    }

    // ============ VNETS API ============
    println!("\n## VNets API");

    // get() - get a specific vnet (need an ID first)
    match client.vnets().list(None).await {
        Ok(vnets) => {
            if let Some(vnet) = vnets.first() {
                println!("  ✅ list() - found vnet: {}", vnet.id);

                // Try to get it
                match client.vnets().get(&vnet.id).await {
                    Ok(v) => println!("  ✅ get() - {}", v.display_name),
                    Err(e) => println!("  ❌ get() - {}", e),
                }
            }
        }
        Err(e) => println!("  ❌ list() - {}", e),
    }

    // ============ ALERTS API ============
    println!("\n## Alerts API");

    // list webhooks
    match client.alerts().list_webhooks().await {
        Ok(resp) => println!("  ✅ list_webhooks() - {} webhooks", resp.webhooks.len()),
        Err(e) => println!("  ❌ list_webhooks() - {}", e),
    }

    // ============ SIMULATION API ============
    println!("\n## Simulation API");

    // list simulations
    match client.simulation().list(1, 10).await {
        Ok(resp) => println!("  ✅ list() - {} simulations", resp.simulations.len()),
        Err(e) => println!("  ❌ list() - {}", e),
    }

    // ============ ACTIONS API ============
    println!("\n## Actions API");

    // If there are actions, try to get one
    match client.actions().list().await {
        Ok(resp) => {
            println!("  ✅ list() - {} actions", resp.actions.len());
            if let Some(action) = resp.actions.first() {
                match client.actions().get(&action.id).await {
                    Ok(a) => println!("  ✅ get() - {}", a.name),
                    Err(e) => println!("  ❌ get() - {}", e),
                }
            }
        }
        Err(e) => println!("  ❌ list() - {}", e),
    }

    println!("\n=== DONE ===");
    Ok(())
}
