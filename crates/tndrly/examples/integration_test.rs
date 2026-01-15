//! Comprehensive integration test for all API endpoints
//!
//! Tests each endpoint against the live Tenderly API.
//! Run with: cargo run --example integration_test

use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    let mut passed = 0;
    let mut failed = 0;

    println!("=== COMPREHENSIVE API INTEGRATION TEST ===\n");

    // ============ NETWORKS API ============
    println!("## Networks API");

    // supported()
    match client.networks().supported().await {
        Ok(networks) => {
            println!("  ✅ supported() - {} networks", networks.len());
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ supported() - {}", e);
            failed += 1;
        }
    }

    // mainnets()
    match client.networks().mainnets().await {
        Ok(nets) => {
            println!("  ✅ mainnets() - {} mainnets", nets.len());
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ mainnets() - {}", e);
            failed += 1;
        }
    }

    // testnets()
    match client.networks().testnets().await {
        Ok(nets) => {
            println!("  ✅ testnets() - {} testnets", nets.len());
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ testnets() - {}", e);
            failed += 1;
        }
    }

    // get()
    match client.networks().get("1").await {
        Ok(Some(net)) => {
            println!("  ✅ get(\"1\") - {}", net.network_name);
            passed += 1;
        }
        Ok(None) => {
            println!("  ❌ get(\"1\") - Network not found");
            failed += 1;
        }
        Err(e) => {
            println!("  ❌ get(\"1\") - {}", e);
            failed += 1;
        }
    }

    // ============ WALLETS API ============
    println!("\n## Wallets API");

    // list() - list all wallets in project
    match client.wallets().list().await {
        Ok(wallets) => {
            println!("  ✅ list() - {} wallets", wallets.len());
            for w in wallets.iter().take(3) {
                println!(
                    "      - {} on network {}",
                    w.address().unwrap_or("unknown"),
                    w.network_id().unwrap_or("unknown")
                );
            }
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ list() - {}", e);
            failed += 1;
        }
    }

    // add() - try to add a wallet (may already exist or hit quota)
    use tndrly::wallets::AddWalletRequest;
    let add_request = AddWalletRequest::new("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045")
        .network("1")
        .display_name("vitalik.eth");

    match client.wallets().add(&add_request).await {
        Ok(resp) => {
            println!(
                "  ✅ add() - added wallet, {} contracts",
                resp.contracts.len()
            );
            passed += 1;
        }
        Err(e) => {
            let err_str = e.to_string();
            if err_str.contains("quota")
                || err_str.contains("limit")
                || err_str.contains("already")
                || err_str.contains("conflict")
            {
                println!("  ✅ add() - endpoint works (wallet exists or quota limit)");
                passed += 1;
            } else {
                println!("  ❌ add() - {}", e);
                failed += 1;
            }
        }
    }

    // get() - test with public address (will 404 if not in project, which is expected)
    match client
        .wallets()
        .get("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", "1")
        .await
    {
        Ok(wallet) => {
            println!(
                "  ✅ get() - {} (balance: {})",
                wallet.address().unwrap_or("unknown"),
                wallet.balance().unwrap_or("unknown")
            );
            passed += 1;
        }
        Err(e) => {
            if e.to_string().contains("not found") {
                println!("  ✅ get() - endpoint works (wallet not in project)");
                passed += 1;
            } else {
                println!("  ❌ get() - {}", e);
                failed += 1;
            }
        }
    }

    // ============ DELIVERY CHANNELS API ============
    println!("\n## Delivery Channels API");

    // list_project()
    match client.delivery_channels().list_project().await {
        Ok(resp) => {
            println!(
                "  ✅ list_project() - {} channels",
                resp.delivery_channels.len()
            );
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ list_project() - {}", e);
            failed += 1;
        }
    }

    // list_account()
    match client.delivery_channels().list_account().await {
        Ok(resp) => {
            println!(
                "  ✅ list_account() - {} channels",
                resp.delivery_channels.len()
            );
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ list_account() - {}", e);
            failed += 1;
        }
    }

    // ============ ACTIONS API ============
    println!("\n## Actions API");

    // list()
    match client.actions().list().await {
        Ok(resp) => {
            println!("  ✅ list() - {} actions", resp.actions.len());
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ list() - {}", e);
            failed += 1;
        }
    }

    // ============ ALERTS API ============
    println!("\n## Alerts API");

    // list()
    match client.alerts().list().await {
        Ok(resp) => {
            println!("  ✅ list() - {} alerts", resp.alerts.len());
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ list() - {}", e);
            failed += 1;
        }
    }

    // ============ CONTRACTS API ============
    println!("\n## Contracts API");

    // list()
    match client.contracts().list(None).await {
        Ok(contracts) => {
            println!("  ✅ list() - {} contracts", contracts.len());
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ list() - {}", e);
            failed += 1;
        }
    }

    // ============ VNETS API ============
    println!("\n## VNets API");

    // list()
    match client.vnets().list(None).await {
        Ok(vnets) => {
            println!("  ✅ list() - {} vnets", vnets.len());
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ list() - {}", e);
            failed += 1;
        }
    }

    // ============ SIMULATION API ============
    println!("\n## Simulation API");

    // simulate() - simple ETH transfer
    use tndrly::simulation::SimulationRequest;
    let sim_request = SimulationRequest::new(
        "0xd8da6bf26964af9d7eed9e03e53415d37aa96045", // from: vitalik.eth
        "0x0000000000000000000000000000000000000000", // to: zero address
        "",                                           // input: empty calldata
    )
    .network_id("1")
    .value("0");

    match client.simulation().simulate(&sim_request).await {
        Ok(result) => {
            println!("  ✅ simulate() - status: {}", result.simulation.status);
            passed += 1;
        }
        Err(e) => {
            println!("  ❌ simulate() - {}", e);
            failed += 1;
        }
    }

    // ============ SUMMARY ============
    println!("\n=== SUMMARY ===");
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Total:  {}", passed + failed);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
