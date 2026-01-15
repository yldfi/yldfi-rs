//! CLI integration tests
//!
//! Tests the ethcli binary end-to-end for offline commands

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn ethcli() -> Command {
    Command::cargo_bin("ethcli").unwrap()
}

/// Create a temp config directory with a minimal config file
/// Returns the temp dir (must be kept alive for the duration of the test)
fn setup_temp_config() -> TempDir {
    let temp_dir = TempDir::new().unwrap();

    let config_content = r#"
[settings]
concurrency = 5
timeout_seconds = 30

[[endpoints]]
url = "https://eth.example.com/rpc"
max_block_range = 100000
max_logs = 10000
priority = 10
enabled = true
chain = "ethereum"
node_type = "archive"
has_debug = true
has_trace = false

[[endpoints]]
url = "https://polygon.example.com/rpc"
max_block_range = 50000
priority = 8
enabled = true
chain = "polygon"
node_type = "full"
has_debug = false
has_trace = true

[[endpoints]]
url = "https://disabled.example.com/rpc"
max_block_range = 10000
priority = 5
enabled = false
chain = "ethereum"
"#;

    fs::write(temp_dir.path().join("config.toml"), config_content).unwrap();
    temp_dir
}

/// Create ethcli command with temp config directory
fn ethcli_with_config(temp_dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ethcli").unwrap();
    cmd.env("ETHCLI_CONFIG_DIR", temp_dir.path());
    cmd
}

// ==================== Basic CLI tests ====================

#[test]
fn test_version() {
    ethcli()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ethcli"));
}

#[test]
fn test_help() {
    ethcli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Comprehensive Ethereum CLI"));
}

#[test]
fn test_cast_help() {
    ethcli()
        .args(["cast", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("to-wei"));
}

// ==================== Cast conversion tests ====================

#[test]
fn test_cast_to_hex() {
    ethcli()
        .args(["cast", "to-hex", "255"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0xff"));
}

#[test]
fn test_cast_to_hex_large() {
    ethcli()
        .args(["cast", "to-hex", "1000000"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0xf4240"));
}

#[test]
fn test_cast_to_dec() {
    ethcli()
        .args(["cast", "to-dec", "0xff"])
        .assert()
        .success()
        .stdout(predicate::str::contains("255"));
}

#[test]
fn test_cast_to_dec_without_prefix() {
    ethcli()
        .args(["cast", "to-dec", "ff"])
        .assert()
        .success()
        .stdout(predicate::str::contains("255"));
}

#[test]
fn test_cast_to_wei_eth() {
    ethcli()
        .args(["cast", "to-wei", "1", "eth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1000000000000000000"));
}

#[test]
fn test_cast_to_wei_decimal() {
    ethcli()
        .args(["cast", "to-wei", "1.5", "eth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1500000000000000000"));
}

#[test]
fn test_cast_to_wei_gwei() {
    ethcli()
        .args(["cast", "to-wei", "1", "gwei"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1000000000"));
}

#[test]
fn test_cast_from_wei_eth() {
    ethcli()
        .args(["cast", "from-wei", "1000000000000000000", "eth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0"));
}

#[test]
fn test_cast_from_wei_gwei() {
    ethcli()
        .args(["cast", "from-wei", "1000000000", "gwei"])
        .assert()
        .success()
        .stdout(predicate::str::contains("1.0"));
}

// ==================== Cast hashing tests ====================

#[test]
fn test_cast_keccak_string() {
    ethcli()
        .args(["cast", "keccak", "hello"])
        .assert()
        .success()
        // keccak256("hello") = 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
        .stdout(predicate::str::contains(
            "1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8",
        ));
}

#[test]
fn test_cast_sig_transfer() {
    ethcli()
        .args(["cast", "sig", "transfer(address,uint256)"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0xa9059cbb"));
}

#[test]
fn test_cast_sig_approve() {
    ethcli()
        .args(["cast", "sig", "approve(address,uint256)"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0x095ea7b3"));
}

#[test]
fn test_cast_sig_balance_of() {
    ethcli()
        .args(["cast", "sig", "balanceOf(address)"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0x70a08231"));
}

#[test]
fn test_cast_topic_transfer() {
    ethcli()
        .args(["cast", "topic", "Transfer(address,address,uint256)"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
        ));
}

// ==================== Cast address tests ====================

#[test]
fn test_cast_checksum() {
    ethcli()
        .args([
            "cast",
            "checksum",
            "0xd8da6bf26964af9d7eed9e03e53415d37aa96045",
        ])
        .assert()
        .success()
        // EIP-55 checksummed address
        .stdout(predicate::str::contains(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        ));
}

#[test]
fn test_cast_checksum_already_valid() {
    ethcli()
        .args([
            "cast",
            "checksum",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        ));
}

// ==================== Cast bytes32 tests ====================

#[test]
fn test_cast_to_bytes32() {
    ethcli()
        .args(["cast", "to-bytes32", "0x01"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ));
}

#[test]
fn test_cast_to_bytes32_larger() {
    ethcli()
        .args(["cast", "to-bytes32", "0xabcd"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0x000000000000000000000000000000000000000000000000000000000000abcd",
        ));
}

// ==================== Cast concat tests ====================

#[test]
fn test_cast_concat() {
    ethcli()
        .args(["cast", "concat", "0xaa", "0xbb", "0xcc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0xaabbcc"));
}

// ==================== ENS tests (offline) ====================

#[test]
fn test_ens_namehash() {
    ethcli()
        .args(["ens", "namehash", "vitalik.eth"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0xee6c4522aab0003e8d14cd40a6af439055fd2577951148c14b6cea9a53475835",
        ));
}

#[test]
fn test_ens_namehash_eth() {
    ethcli()
        .args(["ens", "namehash", "eth"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "0x93cdeb708b7545dc668eb9280176169d1c33cfd8ed6f04690a0bcc88a93fc4ae",
        ));
}

// ==================== Error handling tests ====================

#[test]
fn test_cast_to_wei_invalid_unit() {
    ethcli()
        .args(["cast", "to-wei", "1", "invalid"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown unit"));
}

#[test]
fn test_cast_to_wei_invalid_value() {
    ethcli()
        .args(["cast", "to-wei", "not_a_number", "eth"])
        .assert()
        .failure();
}

#[test]
fn test_cast_to_dec_invalid_hex() {
    ethcli()
        .args(["cast", "to-dec", "0xZZZ"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid hex"));
}

#[test]
fn test_cast_checksum_invalid_address() {
    ethcli()
        .args(["cast", "checksum", "not_an_address"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Invalid address"));
}

// ==================== ABI encode/decode tests ====================

#[test]
fn test_cast_abi_encode_no_args() {
    ethcli()
        .args(["cast", "abi-encode", "totalSupply()"])
        .assert()
        .success()
        .stdout(predicate::str::contains("0x18160ddd"));
}

#[test]
fn test_cast_abi_encode_with_args() {
    ethcli()
        .args([
            "cast",
            "abi-encode",
            "transfer(address,uint256)",
            "0x0000000000000000000000000000000000000001",
            "1000",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("0xa9059cbb"));
}

// ==================== Endpoints CLI tests ====================

#[test]
fn test_endpoints_help() {
    ethcli()
        .args(["endpoints", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage RPC endpoints"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("add"))
        .stdout(predicate::str::contains("remove"))
        .stdout(predicate::str::contains("optimize"));
}

#[test]
fn test_endpoints_list_help() {
    ethcli()
        .args(["endpoints", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("List all configured endpoints"))
        .stdout(predicate::str::contains("--archive"))
        .stdout(predicate::str::contains("--debug"))
        .stdout(predicate::str::contains("--chain"));
}

#[test]
fn test_endpoints_add_help() {
    ethcli()
        .args(["endpoints", "add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Add a new RPC endpoint"));
}

#[test]
fn test_endpoints_remove_help() {
    ethcli()
        .args(["endpoints", "remove", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Remove an RPC endpoint"));
}

#[test]
fn test_endpoints_optimize_help() {
    ethcli()
        .args(["endpoints", "optimize", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Optimize endpoint"));
}

#[test]
fn test_endpoints_list_with_temp_config() {
    let temp_dir = setup_temp_config();

    // Default list shows ethereum endpoints (default chain)
    ethcli_with_config(&temp_dir)
        .args(["endpoints", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://eth.example.com/rpc"))
        .stdout(predicate::str::contains("ETHEREUM"));
}

#[test]
fn test_endpoints_list_filter_by_chain() {
    let temp_dir = setup_temp_config();

    // Filter for ethereum - should show eth endpoint
    ethcli_with_config(&temp_dir)
        .args(["endpoints", "list", "--chain", "ethereum"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://eth.example.com/rpc"));

    // Filter for polygon - should show polygon endpoint
    ethcli_with_config(&temp_dir)
        .args(["endpoints", "list", "--chain", "polygon"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://polygon.example.com/rpc"));
}

#[test]
fn test_endpoints_list_filter_archive() {
    let temp_dir = setup_temp_config();

    ethcli_with_config(&temp_dir)
        .args(["endpoints", "list", "--archive"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://eth.example.com/rpc"));
}

#[test]
fn test_endpoints_list_filter_debug() {
    let temp_dir = setup_temp_config();

    ethcli_with_config(&temp_dir)
        .args(["endpoints", "list", "--debug"])
        .assert()
        .success()
        .stdout(predicate::str::contains("https://eth.example.com/rpc"));
}

#[test]
fn test_endpoints_list_detailed() {
    let temp_dir = setup_temp_config();

    ethcli_with_config(&temp_dir)
        .args(["endpoints", "list", "--detailed"])
        .assert()
        .success()
        // Detailed view shows block range info
        .stdout(predicate::str::contains("Block range"))
        .stdout(predicate::str::contains("100,000"));
}

#[test]
fn test_endpoints_enable_disable() {
    let temp_dir = setup_temp_config();

    // Disable an endpoint
    ethcli_with_config(&temp_dir)
        .args(["endpoints", "disable", "https://eth.example.com/rpc"])
        .assert()
        .success();

    // Verify it's disabled by checking the config file
    let config_path = temp_dir.path().join("config.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();
    // After disable, the endpoint should have enabled = false
    assert!(config_content.contains("enabled = false"));

    // Re-enable it
    ethcli_with_config(&temp_dir)
        .args(["endpoints", "enable", "https://eth.example.com/rpc"])
        .assert()
        .success();
}

#[test]
fn test_endpoints_remove() {
    let temp_dir = setup_temp_config();

    // Remove an endpoint
    ethcli_with_config(&temp_dir)
        .args(["endpoints", "remove", "https://polygon.example.com/rpc"])
        .assert()
        .success();

    // Verify it's removed
    let config_path = temp_dir.path().join("config.toml");
    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(!config_content.contains("https://polygon.example.com/rpc"));
}

#[test]
fn test_endpoints_remove_nonexistent() {
    let temp_dir = setup_temp_config();

    // Try to remove a nonexistent endpoint - returns success with message
    ethcli_with_config(&temp_dir)
        .args(["endpoints", "remove", "https://nonexistent.example.com/rpc"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not found"));
}

// ==================== Config CLI tests ====================

#[test]
fn test_config_help() {
    ethcli()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage configuration"));
}

#[test]
fn test_config_path() {
    let temp_dir = setup_temp_config();

    ethcli_with_config(&temp_dir)
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("config.toml"));
}
