//! Admin RPC Integration Tests
//!
//! These tests run against the live Tenderly API and require credentials.
//! They are marked with #[ignore] and can be run with:
//!
//! ```bash
//! cargo test --test admin_rpc_integration -- --ignored
//! ```
//!
//! Or run a specific test:
//! ```bash
//! cargo test --test admin_rpc_integration test_set_erc20_balance_format -- --ignored
//! ```
//!
//! Required environment variables:
//! - TENDERLY_ACCESS_KEY
//! - TENDERLY_ACCOUNT
//! - TENDERLY_PROJECT

use tndrly::vnets::{CreateVNetRequest, SendTransactionParams};
use tndrly::Client;

const TEST_ADDRESS: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
const TEST_CONTRACT: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"; // USDC
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

/// Helper to create a VNet for testing
async fn create_test_vnet(client: &Client, name: &str) -> Result<String, tndrly::Error> {
    let request = CreateVNetRequest::new(name, format!("Test: {}", name), 1).block_number(18000000);

    let vnet = client.vnets().create(&request).await?;
    Ok(vnet.id)
}

/// Helper to cleanup a VNet
async fn cleanup_vnet(client: &Client, vnet_id: &str) {
    let _ = client.vnets().delete(vnet_id).await;
}

// ============================================================================
// Bug #1: set_erc20_balance format
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_set_erc20_balance_decimal_format() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "erc20-decimal-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Bug #1: Previously used to_hex_32_bytes which padded to 64 chars
    // API expects simple hex like "0x3b9aca00", not "0x000...3b9aca00"
    let result = admin
        .set_erc20_balance(TEST_CONTRACT, TEST_ADDRESS, "1000000000")
        .await;

    cleanup_vnet(&client, &vnet_id).await;

    // The call should succeed or fail with a token-specific error, NOT a format error
    match result {
        Ok(hash) => {
            assert!(hash.starts_with("0x"), "Should return tx hash");
            assert_eq!(hash.len(), 66, "Tx hash should be 66 chars");
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            // These errors indicate Bug #1 is NOT fixed
            assert!(
                !err_str.contains("32-byte"),
                "Bug #1 not fixed: API rejected 32-byte format"
            );
            assert!(
                !err_str.contains("invalid hex"),
                "Bug #1 not fixed: Invalid hex format"
            );
            // Storage slot detection errors are acceptable
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_set_erc20_balance_hex_format() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "erc20-hex-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Hex value should pass through without extra padding
    let result = admin
        .set_erc20_balance(TEST_CONTRACT, TEST_ADDRESS, "0x3b9aca00")
        .await;

    cleanup_vnet(&client, &vnet_id).await;

    match result {
        Ok(hash) => {
            assert!(hash.starts_with("0x"));
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            assert!(
                !err_str.contains("32-byte"),
                "Hex value should not be padded"
            );
        }
    }
}

// ============================================================================
// Bug #2: get_latest return type
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_get_latest_returns_block_object() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "get-latest-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Bug #2: Previously expected String return but API returns JSON object
    let result = admin.get_latest().await;

    cleanup_vnet(&client, &vnet_id).await;

    match result {
        Ok(block) => {
            // Should successfully deserialize into LatestBlock
            // At minimum, block_number should be present
            assert!(
                block.block_number.is_some() || block.block_hash.is_some(),
                "LatestBlock should have some block info"
            );
        }
        Err(e) => {
            let err_str = e.to_string();
            // This specific error indicates Bug #2 is NOT fixed
            assert!(
                !err_str.contains("invalid type: map, expected a string"),
                "Bug #2 not fixed: API returns map but code expects string. Error: {}",
                e
            );
            panic!("get_latest failed: {}", e);
        }
    }
}

// ============================================================================
// Bug #3: set_storage_at padding
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_set_storage_at_unpadded_slot() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "storage-unpadded-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Bug #3: Previously passed unpadded values directly, but API requires 32-byte hex
    // Now the library should pad "0" to full 32-byte format
    let result = admin.set_storage_at(TEST_CONTRACT, "0", "1").await;

    cleanup_vnet(&client, &vnet_id).await;

    match result {
        Ok(hash) => {
            assert!(hash.starts_with("0x"), "Should return tx hash");
            assert_eq!(hash.len(), 66, "Tx hash should be 66 chars");
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            // These errors indicate Bug #3 is NOT fixed
            assert!(
                !err_str.contains("32-byte") && !err_str.contains("must be 32"),
                "Bug #3 not fixed: API requires 32-byte format but got unpadded. Error: {}",
                e
            );
            panic!("set_storage_at failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_set_storage_at_decimal_values() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "storage-decimal-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Decimal values should be converted to 32-byte hex
    let result = admin.set_storage_at(TEST_CONTRACT, "1", "12345").await;

    cleanup_vnet(&client, &vnet_id).await;

    result.expect("set_storage_at with decimal values should succeed");
}

#[tokio::test]
#[ignore]
async fn test_set_storage_at_padded_hex() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "storage-padded-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Already padded values should work
    let slot = "0x0000000000000000000000000000000000000000000000000000000000000002";
    let value = "0x0000000000000000000000000000000000000000000000000000000000000064";

    let result = admin.set_storage_at(TEST_CONTRACT, slot, value).await;

    cleanup_vnet(&client, &vnet_id).await;

    result.expect("set_storage_at with padded hex should succeed");
}

// ============================================================================
// Bug #4 & #5: Timestamp methods return type
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_set_next_block_timestamp_returns_hash() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "timestamp-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Bug #4: Previously expected u64 return but API returns tx hash string
    let result = admin.set_next_block_timestamp(1700000000).await;

    cleanup_vnet(&client, &vnet_id).await;

    match result {
        Ok(hash) => {
            // Should be a transaction hash, not a timestamp
            assert!(
                hash.starts_with("0x"),
                "Should return tx hash starting with 0x"
            );
            assert_eq!(hash.len(), 66, "Tx hash should be 66 chars (0x + 64 hex)");
        }
        Err(e) => {
            let err_str = e.to_string();
            assert!(
                !err_str.contains("invalid type"),
                "Bug #4 not fixed: Deserialization error. {}",
                e
            );
            panic!("set_next_block_timestamp failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_set_next_block_timestamp_no_mine_returns_hash() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "timestamp-no-mine-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Bug #5: Same issue as Bug #4
    let result = admin.set_next_block_timestamp_no_mine(1700001000).await;

    cleanup_vnet(&client, &vnet_id).await;

    match result {
        Ok(hash) => {
            assert!(hash.starts_with("0x"));
            assert_eq!(hash.len(), 66);
        }
        Err(e) => {
            let err_str = e.to_string();
            assert!(
                !err_str.contains("invalid type"),
                "Bug #5 not fixed: Deserialization error. {}",
                e
            );
            panic!("set_next_block_timestamp_no_mine failed: {}", e);
        }
    }
}

// ============================================================================
// Bug #6: send_transaction value conversion
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_send_transaction_decimal_value_conversion() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "tx-decimal-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // First fund the test address
    admin
        .set_balance(TEST_ADDRESS, "10000000000000000000")
        .await
        .expect("Failed to set balance");

    // Bug #6: Previously value was passed as-is, causing issues with decimal values
    // Now the builder auto-converts to hex
    let tx = SendTransactionParams::new(TEST_ADDRESS)
        .to(ZERO_ADDRESS)
        .value("1000000000000000000") // 1 ETH as decimal
        .gas("0x5208");

    let result = admin.send_transaction(&tx).await;

    cleanup_vnet(&client, &vnet_id).await;

    match result {
        Ok(hash) => {
            assert!(hash.starts_with("0x"));
            assert_eq!(hash.len(), 66);
        }
        Err(e) => {
            let err_str = e.to_string().to_lowercase();
            // These errors indicate Bug #6 is NOT fixed
            assert!(
                !err_str.contains("value") || !err_str.contains("hex"),
                "Bug #6 not fixed: Value format rejected. Error: {}",
                e
            );
            // Insufficient funds errors are acceptable (means format was correct)
            if !err_str.contains("insufficient") && !err_str.contains("balance") {
                panic!("send_transaction failed unexpectedly: {}", e);
            }
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_send_transaction_hex_value_passthrough() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "tx-hex-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Fund the test address
    admin
        .set_balance(TEST_ADDRESS, "10000000000000000000")
        .await
        .expect("Failed to set balance");

    // Hex value should still work
    let tx = SendTransactionParams::new(TEST_ADDRESS)
        .to(ZERO_ADDRESS)
        .value("0xde0b6b3a7640000") // 1 ETH as hex
        .gas("0x5208");

    let result = admin.send_transaction(&tx).await;

    cleanup_vnet(&client, &vnet_id).await;

    result.expect("send_transaction with hex value should succeed");
}

// ============================================================================
// Working methods (sanity checks)
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_increase_time() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "increase-time-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    let result = admin.increase_time(3600).await;

    cleanup_vnet(&client, &vnet_id).await;

    let hash = result.expect("increase_time should succeed");
    assert!(hash.starts_with("0x"));
    assert_eq!(hash.len(), 66);
}

#[tokio::test]
#[ignore]
async fn test_increase_blocks() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "increase-blocks-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    let result = admin.increase_blocks(10).await;

    cleanup_vnet(&client, &vnet_id).await;

    let hash = result.expect("increase_blocks should succeed");
    assert!(hash.starts_with("0x"));
}

#[tokio::test]
#[ignore]
async fn test_set_balance() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "set-balance-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Decimal value
    let result1 = admin.set_balance(TEST_ADDRESS, "5000000000000000000").await;
    result1.expect("set_balance with decimal should succeed");

    // Hex value
    let result2 = admin.set_balance(TEST_ADDRESS, "0xde0b6b3a7640000").await;
    result2.expect("set_balance with hex should succeed");

    cleanup_vnet(&client, &vnet_id).await;
}

#[tokio::test]
#[ignore]
async fn test_add_balance() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "add-balance-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    let result = admin.add_balance(TEST_ADDRESS, "1000000000000000000").await;

    cleanup_vnet(&client, &vnet_id).await;

    result.expect("add_balance should succeed");
}

#[tokio::test]
#[ignore]
async fn test_snapshot_and_revert() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "snapshot-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Create snapshot
    let snapshot_id = admin.snapshot().await.expect("snapshot should succeed");
    assert!(snapshot_id.starts_with("0x"));

    // Make changes
    admin
        .set_balance(TEST_ADDRESS, "999")
        .await
        .expect("set_balance should succeed");

    // Revert
    let success = admin
        .revert(&snapshot_id)
        .await
        .expect("revert should succeed");
    assert!(success, "revert should return true");

    cleanup_vnet(&client, &vnet_id).await;
}

#[tokio::test]
#[ignore]
async fn test_set_code() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "set-code-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // Simple bytecode that returns 1
    let bytecode = "0x600160005260206000f3";
    let result = admin.set_code(ZERO_ADDRESS, bytecode).await;

    cleanup_vnet(&client, &vnet_id).await;

    result.expect("set_code should succeed");
}

#[tokio::test]
#[ignore]
async fn test_create_access_list() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "access-list-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    let tx = SendTransactionParams::new(TEST_ADDRESS)
        .to(TEST_CONTRACT)
        .data("0x70a08231000000000000000000000000d8da6bf26964af9d7eed9e03e53415d37aa96045");

    let result = admin.create_access_list(&tx, "latest").await;

    cleanup_vnet(&client, &vnet_id).await;

    let access_list = result.expect("create_access_list should succeed");
    // Should have some entries for reading USDC balanceOf
    assert!(!access_list.gas_used.is_empty());
}

// ============================================================================
// Full integration test
// ============================================================================

#[tokio::test]
#[ignore]
async fn test_full_admin_rpc_workflow() {
    let client = Client::from_env().expect("Failed to create client");
    let vnet_id = create_test_vnet(&client, "full-workflow-test")
        .await
        .expect("Failed to create VNet");

    let admin = client
        .vnets()
        .admin_rpc(&vnet_id)
        .await
        .expect("Failed to get admin RPC");

    // 1. Create snapshot
    let snapshot_id = admin.snapshot().await.expect("snapshot failed");

    // 2. Set balance (decimal)
    admin
        .set_balance(TEST_ADDRESS, "10000000000000000000")
        .await
        .expect("set_balance failed");

    // 3. Add balance
    admin
        .add_balance(TEST_ADDRESS, "5000000000000000000")
        .await
        .expect("add_balance failed");

    // 4. Increase time
    admin
        .increase_time(3600)
        .await
        .expect("increase_time failed");

    // 5. Set timestamp (Bug #4 - should return hash)
    let ts_hash = admin
        .set_next_block_timestamp(1700000000)
        .await
        .expect("set_next_block_timestamp failed");
    assert!(ts_hash.starts_with("0x") && ts_hash.len() == 66);

    // 6. Get latest (Bug #2 - should return LatestBlock)
    let latest = admin.get_latest().await.expect("get_latest failed");
    assert!(latest.block_number.is_some() || latest.block_hash.is_some());

    // 7. Set storage (Bug #3 - should pad unpadded values)
    admin
        .set_storage_at(TEST_CONTRACT, "5", "100")
        .await
        .expect("set_storage_at failed");

    // 8. Send transaction with decimal value (Bug #6 - should auto-convert)
    let tx = SendTransactionParams::new(TEST_ADDRESS)
        .to(ZERO_ADDRESS)
        .value("1000000000000000000")
        .gas("0x5208");
    admin
        .send_transaction(&tx)
        .await
        .expect("send_transaction failed");

    // 9. Revert to snapshot
    let success = admin.revert(&snapshot_id).await.expect("revert failed");
    assert!(success);

    cleanup_vnet(&client, &vnet_id).await;
}
