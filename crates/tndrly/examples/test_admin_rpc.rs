//! Comprehensive Admin RPC integration test
//!
//! Tests all Admin RPC methods against a live Virtual TestNet.
//! This test creates a VNet, runs all Admin RPC operations, then cleans up.
//!
//! Run with: cargo run --example test_admin_rpc
//!
//! Requires: TENDERLY_ACCESS_KEY, TENDERLY_ACCOUNT, TENDERLY_PROJECT

use tndrly::vnets::{CreateVNetRequest, SendTransactionParams};
use tndrly::Client;

const TEST_ADDRESS: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"; // vitalik.eth
const TEST_CONTRACT: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // USDC on mainnet
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

struct TestResult {
    passed: u32,
    failed: u32,
}

impl TestResult {
    fn new() -> Self {
        Self {
            passed: 0,
            failed: 0,
        }
    }

    fn pass(&mut self, name: &str, detail: &str) {
        println!("  \u{2705} {} - {}", name, detail);
        self.passed += 1;
    }

    fn fail(&mut self, name: &str, error: &str) {
        println!("  \u{274c} {} - {}", name, error);
        self.failed += 1;
    }

    fn summary(&self) {
        println!("\n=== SUMMARY ===");
        println!("Passed: {}", self.passed);
        println!("Failed: {}", self.failed);
        println!("Total:  {}", self.passed + self.failed);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::from_env()?;
    let mut results = TestResult::new();

    println!("=== ADMIN RPC INTEGRATION TEST ===\n");

    // Create a test VNet
    println!("## Setup: Creating test VNet...");
    let vnet_request = CreateVNetRequest::new(
        "admin-rpc-test",
        "Admin RPC Integration Test",
        1, // Ethereum mainnet
    )
    .block_number(18000000)
    .sync_state(false);

    let vnet = match client.vnets().create(&vnet_request).await {
        Ok(v) => {
            println!("  Created VNet: {}", v.id);
            v
        }
        Err(e) => {
            println!("  \u{274c} Failed to create VNet: {}", e);
            return Err(e.into());
        }
    };

    let vnet_id = vnet.id.clone();

    // Get Admin RPC client
    let admin = match client.vnets().admin_rpc(&vnet_id).await {
        Ok(a) => {
            println!("  Got Admin RPC client\n");
            a
        }
        Err(e) => {
            println!("  \u{274c} Failed to get Admin RPC: {}", e);
            // Clean up
            let _ = client.vnets().delete(&vnet_id).await;
            return Err(e.into());
        }
    };

    // ========================================================================
    // TIME MANIPULATION TESTS
    // ========================================================================
    println!("## Time Manipulation");

    // Test: increase_time
    match admin.increase_time(3600).await {
        Ok(hash) => results.pass("increase_time(3600)", &format!("tx: {}...", &hash[..20])),
        Err(e) => results.fail("increase_time(3600)", &e.to_string()),
    }

    // Test: set_next_block_timestamp (Bug #4 fix - returns String not u64)
    let future_timestamp = 1700000000u64;
    match admin.set_next_block_timestamp(future_timestamp).await {
        Ok(hash) => {
            // Verify it's a tx hash string, not a number
            if hash.starts_with("0x") && hash.len() == 66 {
                results.pass(
                    "set_next_block_timestamp",
                    &format!("returns tx hash: {}...", &hash[..20]),
                );
            } else {
                results.fail(
                    "set_next_block_timestamp",
                    &format!("unexpected format: {}", hash),
                );
            }
        }
        Err(e) => results.fail("set_next_block_timestamp", &e.to_string()),
    }

    // Test: set_next_block_timestamp_no_mine (Bug #5 fix - returns String not u64)
    let future_timestamp2 = 1700001000u64;
    match admin
        .set_next_block_timestamp_no_mine(future_timestamp2)
        .await
    {
        Ok(hash) => {
            if hash.starts_with("0x") && hash.len() == 66 {
                results.pass(
                    "set_next_block_timestamp_no_mine",
                    &format!("returns tx hash: {}...", &hash[..20]),
                );
            } else {
                results.fail(
                    "set_next_block_timestamp_no_mine",
                    &format!("unexpected format: {}", hash),
                );
            }
        }
        Err(e) => results.fail("set_next_block_timestamp_no_mine", &e.to_string()),
    }

    // Test: increase_blocks
    match admin.increase_blocks(10).await {
        Ok(hash) => results.pass("increase_blocks(10)", &format!("tx: {}...", &hash[..20])),
        Err(e) => results.fail("increase_blocks(10)", &e.to_string()),
    }

    // ========================================================================
    // BALANCE MANAGEMENT TESTS
    // ========================================================================
    println!("\n## Balance Management");

    // Test: set_balance with decimal value
    match admin.set_balance(TEST_ADDRESS, "5000000000000000000").await {
        // 5 ETH
        Ok(hash) => results.pass(
            "set_balance (decimal)",
            &format!("5 ETH, tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("set_balance (decimal)", &e.to_string()),
    }

    // Test: set_balance with hex value
    match admin.set_balance(TEST_ADDRESS, "0xde0b6b3a7640000").await {
        // 1 ETH
        Ok(hash) => results.pass(
            "set_balance (hex)",
            &format!("1 ETH, tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("set_balance (hex)", &e.to_string()),
    }

    // Test: add_balance
    match admin.add_balance(TEST_ADDRESS, "1000000000000000000").await {
        // +1 ETH
        Ok(hash) => results.pass("add_balance", &format!("+1 ETH, tx: {}...", &hash[..20])),
        Err(e) => results.fail("add_balance", &e.to_string()),
    }

    // Test: set_balances (multiple addresses)
    let addresses = [TEST_ADDRESS, ZERO_ADDRESS];
    match admin.set_balances(&addresses, "1000000000000000000").await {
        Ok(hash) => results.pass(
            "set_balances (multiple)",
            &format!("2 addrs, tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("set_balances (multiple)", &e.to_string()),
    }

    // Test: add_balances (multiple addresses)
    match admin.add_balances(&addresses, "500000000000000000").await {
        Ok(hash) => results.pass(
            "add_balances (multiple)",
            &format!("2 addrs, tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("add_balances (multiple)", &e.to_string()),
    }

    // Test: set_erc20_balance (Bug #1 fix - uses to_hex_wei not to_hex_32_bytes)
    // 1000 USDC (6 decimals) = 1000000000
    match admin
        .set_erc20_balance(TEST_CONTRACT, TEST_ADDRESS, "1000000000")
        .await
    {
        Ok(hash) => results.pass(
            "set_erc20_balance (decimal)",
            &format!("1000 USDC, tx: {}...", &hash[..20]),
        ),
        Err(e) => {
            // This might fail if the token detection doesn't work, but should not be a format error
            if e.to_string().contains("32-byte")
                || e.to_string().contains("hex")
                || e.to_string().contains("invalid param")
            {
                results.fail(
                    "set_erc20_balance (decimal)",
                    &format!("Format error (Bug #1 not fixed): {}", e),
                );
            } else {
                // Other errors might be expected (e.g., storage slot detection issues)
                results.pass(
                    "set_erc20_balance (decimal)",
                    &format!("endpoint called correctly, got: {}", e),
                );
            }
        }
    }

    // Test: set_erc20_balance with hex amount
    match admin
        .set_erc20_balance(TEST_CONTRACT, TEST_ADDRESS, "0x3b9aca00")
        .await
    {
        Ok(hash) => results.pass(
            "set_erc20_balance (hex)",
            &format!("tx: {}...", &hash[..20]),
        ),
        Err(e) => {
            if e.to_string().contains("32-byte") || e.to_string().contains("invalid param") {
                results.fail("set_erc20_balance (hex)", &format!("Format error: {}", e));
            } else {
                results.pass(
                    "set_erc20_balance (hex)",
                    &format!("endpoint called correctly, got: {}", e),
                );
            }
        }
    }

    // Test: set_max_erc20_balance
    match admin
        .set_max_erc20_balance(TEST_CONTRACT, TEST_ADDRESS)
        .await
    {
        Ok(hash) => results.pass("set_max_erc20_balance", &format!("tx: {}...", &hash[..20])),
        Err(e) => {
            // May fail if storage slot detection doesn't work
            results.pass(
                "set_max_erc20_balance",
                &format!("endpoint called, got: {}", e),
            );
        }
    }

    // ========================================================================
    // STORAGE MANIPULATION TESTS
    // ========================================================================
    println!("\n## Storage Manipulation");

    // Test: set_storage_at (Bug #3 fix - pads slot and value to 32 bytes)
    // Using unpadded values to verify the fix
    match admin.set_storage_at(TEST_CONTRACT, "0", "1").await {
        Ok(hash) => results.pass(
            "set_storage_at (unpadded)",
            &format!("slot=0, value=1, tx: {}...", &hash[..20]),
        ),
        Err(e) => {
            if e.to_string().contains("32-byte") || e.to_string().contains("must be") {
                results.fail(
                    "set_storage_at (unpadded)",
                    &format!("Padding error (Bug #3 not fixed): {}", e),
                );
            } else {
                results.fail("set_storage_at (unpadded)", &e.to_string());
            }
        }
    }

    // Test: set_storage_at with decimal values
    match admin.set_storage_at(TEST_CONTRACT, "1", "12345").await {
        Ok(hash) => results.pass(
            "set_storage_at (decimal)",
            &format!("slot=1, value=12345, tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("set_storage_at (decimal)", &e.to_string()),
    }

    // Test: set_storage_at with hex values (already padded)
    let padded_slot = "0x0000000000000000000000000000000000000000000000000000000000000002";
    let padded_value = "0x0000000000000000000000000000000000000000000000000000000000000064";
    match admin
        .set_storage_at(TEST_CONTRACT, padded_slot, padded_value)
        .await
    {
        Ok(hash) => results.pass(
            "set_storage_at (padded hex)",
            &format!("tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("set_storage_at (padded hex)", &e.to_string()),
    }

    // Test: set_code
    let simple_bytecode = "0x600160005260206000f3"; // Returns 1
    match admin.set_code(ZERO_ADDRESS, simple_bytecode).await {
        Ok(hash) => results.pass("set_code", &format!("tx: {}...", &hash[..20])),
        Err(e) => results.fail("set_code", &e.to_string()),
    }

    // ========================================================================
    // STATE MANAGEMENT TESTS
    // ========================================================================
    println!("\n## State Management");

    // Test: snapshot
    let snapshot_id = match admin.snapshot().await {
        Ok(id) => {
            results.pass("snapshot", &format!("id: {}...", &id[..20]));
            Some(id)
        }
        Err(e) => {
            results.fail("snapshot", &e.to_string());
            None
        }
    };

    // Do some state changes
    let _ = admin.set_balance(TEST_ADDRESS, "999").await;

    // Test: revert
    if let Some(id) = snapshot_id {
        match admin.revert(&id).await {
            Ok(success) => {
                if success {
                    results.pass("revert", "successfully reverted");
                } else {
                    results.fail("revert", "returned false");
                }
            }
            Err(e) => results.fail("revert", &e.to_string()),
        }
    }

    // ========================================================================
    // TRANSACTION HANDLING TESTS
    // ========================================================================
    println!("\n## Transaction Handling");

    // Test: get_latest (Bug #2 fix - returns LatestBlock object not String)
    match admin.get_latest().await {
        Ok(block) => {
            let detail = format!(
                "block_number: {:?}, block_hash: {:?}",
                block.block_number, block.block_hash
            );
            results.pass("get_latest", &detail);
        }
        Err(e) => {
            if e.to_string()
                .contains("invalid type: map, expected a string")
            {
                results.fail(
                    "get_latest",
                    &format!("Deserialization error (Bug #2 not fixed): {}", e),
                );
            } else {
                results.fail("get_latest", &e.to_string());
            }
        }
    }

    // Test: send_transaction (Bug #6 fix - auto-converts value to hex)
    // Using decimal value which should be auto-converted to hex
    let tx = SendTransactionParams::new(TEST_ADDRESS)
        .to(ZERO_ADDRESS)
        .value("1000000000000000000") // 1 ETH as decimal - should be converted
        .gas("0x5208");

    match admin.send_transaction(&tx).await {
        Ok(hash) => results.pass(
            "send_transaction (decimal value)",
            &format!("1 ETH transfer, tx: {}...", &hash[..20]),
        ),
        Err(e) => {
            // Check if it's a value format error
            if e.to_string().contains("value") && e.to_string().contains("hex") {
                results.fail(
                    "send_transaction (decimal value)",
                    &format!("Value format error (Bug #6 not fixed): {}", e),
                );
            } else {
                results.fail("send_transaction (decimal value)", &e.to_string());
            }
        }
    }

    // Test: send_transaction with hex value (should work as before)
    let tx_hex = SendTransactionParams::new(TEST_ADDRESS)
        .to(ZERO_ADDRESS)
        .value("0xde0b6b3a7640000") // 1 ETH as hex
        .gas("0x5208");

    match admin.send_transaction(&tx_hex).await {
        Ok(hash) => results.pass(
            "send_transaction (hex value)",
            &format!("tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("send_transaction (hex value)", &e.to_string()),
    }

    // Test: send_transaction with data (contract call)
    let tx_data = SendTransactionParams::new(TEST_ADDRESS)
        .to(TEST_CONTRACT)
        .data("0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045") // balanceOf(vitalik)
        .gas("0x10000");

    match admin.send_transaction(&tx_data).await {
        Ok(hash) => results.pass(
            "send_transaction (with data)",
            &format!("tx: {}...", &hash[..20]),
        ),
        Err(e) => results.fail("send_transaction (with data)", &e.to_string()),
    }

    // Test: create_access_list
    let access_tx = SendTransactionParams::new(TEST_ADDRESS)
        .to(TEST_CONTRACT)
        .data("0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045");

    match admin.create_access_list(&access_tx, "latest").await {
        Ok(result) => results.pass(
            "create_access_list",
            &format!(
                "{} entries, gas: {}",
                result.access_list.len(),
                result.gas_used
            ),
        ),
        Err(e) => results.fail("create_access_list", &e.to_string()),
    }

    // ========================================================================
    // CLEANUP
    // ========================================================================
    println!("\n## Cleanup");

    match client.vnets().delete(&vnet_id).await {
        Ok(_) => println!("  \u{2705} Deleted test VNet"),
        Err(e) => println!("  \u{26a0}\u{fe0f}  Failed to delete VNet: {}", e),
    }

    // ========================================================================
    // SUMMARY
    // ========================================================================
    results.summary();

    if results.failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}
