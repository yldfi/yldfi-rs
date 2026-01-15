//! Test VNetTransaction deserialization against real API
//!
//! Run with: TENDERLY_ACCESS_KEY=... TENDERLY_ACCOUNT=... TENDERLY_PROJECT=... cargo run --example test_vnet_transactions

use tndrly::vnets::{CreateVNetRequest, ListVNetTransactionsQuery};
use tndrly::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;

    println!("=== Testing VNetTransaction Fix (Issue #12) ===\n");

    // List existing VNets
    println!("1. Listing existing VNets...");
    let vnets = client.vnets().list(None).await?;
    println!("   Found {} VNets", vnets.len());

    // Find a VNet with transactions or create one
    let vnet_id = if let Some(vnet) = vnets.first() {
        println!(
            "   Using existing VNet: {} ({})",
            vnet.display_name, vnet.id
        );
        vnet.id.clone()
    } else {
        println!("   No VNets found, creating test VNet...");
        let request =
            CreateVNetRequest::new("test-issue-12", "Test Issue 12 Fix", 1).block_number(21000000);
        let vnet = client.vnets().create(&request).await?;
        println!("   Created VNet: {} ({})", vnet.display_name, vnet.id);
        vnet.id
    };

    // Test listing transactions (this is where the bug was - deserialization failed)
    println!("\n2. Listing transactions (raw array response)...");
    let txs = client.vnets().transactions(&vnet_id, None).await?;
    println!("   SUCCESS! Deserialized {} transactions", txs.len());

    if txs.is_empty() {
        println!("   No transactions yet. Let's create one via Admin RPC...");

        // Get admin RPC and send a transaction
        let admin = client.vnets().admin_rpc(&vnet_id).await?;

        // Fund an address
        let test_addr = "0x1234567890123456789012345678901234567890";
        println!("   Setting balance for {}...", test_addr);
        admin.set_balance(test_addr, "1000000000000000000").await?;

        // Send a simple transaction
        println!("   Sending test transaction...");
        use tndrly::vnets::SendTransactionParams;
        let tx_params = SendTransactionParams::new(test_addr)
            .to("0xabcdefabcdefabcdefabcdefabcdefabcdefabcd")
            .value("100000000000000000") // 0.1 ETH
            .gas("0x5208");
        let tx_hash = admin.send_transaction(&tx_params).await?;
        println!("   Transaction sent: {}", tx_hash);

        // Now list transactions again
        println!("\n3. Listing transactions after sending one...");
        let txs = client.vnets().transactions(&vnet_id, None).await?;
        println!("   Found {} transactions", txs.len());

        for (i, tx) in txs.iter().enumerate() {
            println!("\n   Transaction {}:", i + 1);
            println!("     tx_hash: {:?}", tx.tx_hash);
            println!("     kind: {:?}", tx.kind);
            println!(
                "     status: {:?} (is_success={}, is_failed={})",
                tx.status,
                tx.is_success(),
                tx.is_failed()
            );
            println!(
                "     block_number: {:?} -> {:?}",
                tx.block_number,
                tx.block_number_as_u64()
            );
            println!("     from: {:?}", tx.from);
            println!("     to: {:?}", tx.to);
            println!("     gas: {:?} -> {:?}", tx.gas, tx.gas_as_u64());
            println!(
                "     gas_used: {:?} -> {:?}",
                tx.gas_used,
                tx.gas_used_as_u64()
            );
            if tx.id.is_some() {
                println!("     id: {:?}", tx.id);
            }
            if tx.origin.is_some() {
                println!("     origin: {:?}", tx.origin);
            }
        }
    } else {
        // Show existing transactions
        println!("\n3. Showing existing transactions:");
        for (i, tx) in txs.iter().take(5).enumerate() {
            println!("\n   Transaction {}:", i + 1);
            println!("     tx_hash: {:?}", tx.tx_hash);
            println!("     kind: {:?}", tx.kind);
            println!(
                "     status: {:?} (is_success={}, is_failed={})",
                tx.status,
                tx.is_success(),
                tx.is_failed()
            );
            println!(
                "     block_number: {:?} -> {:?}",
                tx.block_number,
                tx.block_number_as_u64()
            );
            println!("     from: {:?}", tx.from);
            println!("     to: {:?}", tx.to);
            println!("     gas: {:?} -> {:?}", tx.gas, tx.gas_as_u64());
            println!(
                "     gas_used: {:?} -> {:?}",
                tx.gas_used,
                tx.gas_used_as_u64()
            );
            if tx.id.is_some() {
                println!("     id: {:?}", tx.id);
            }
            if tx.origin.is_some() {
                println!("     origin: {:?}", tx.origin);
            }
        }
    }

    // Test query with status filter
    println!("\n4. Testing query with status filter...");
    let query = ListVNetTransactionsQuery::new().success();
    let success_txs = client.vnets().transactions(&vnet_id, Some(query)).await?;
    println!("   Found {} successful transactions", success_txs.len());

    let query = ListVNetTransactionsQuery::new().failed();
    let failed_txs = client.vnets().transactions(&vnet_id, Some(query)).await?;
    println!("   Found {} failed transactions", failed_txs.len());

    println!("\n=== All tests passed! Issue #12 is fixed. ===");
    Ok(())
}
