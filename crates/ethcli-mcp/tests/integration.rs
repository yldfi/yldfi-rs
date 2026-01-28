//! Integration tests for ethcli-mcp
//!
//! These tests spawn the actual ethcli-mcp binary and communicate via MCP JSON-RPC.
//!
//! Run all tests (including network tests):
//!   cargo test -p ethcli-mcp --test integration -- --include-ignored
//!
//! Run only offline tests:
//!   cargo test -p ethcli-mcp --test integration

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Child, Command, Stdio};

// Test addresses
const VITALIK: &str = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045";
const USDC: &str = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48";
const WETH: &str = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2";

/// MCP client for testing
struct McpClient {
    proc: Child,
    request_id: u64,
}

impl McpClient {
    fn new() -> Self {
        // Use CARGO_BIN_EXE_ethcli-mcp which cargo sets during `cargo test`
        // This ensures the binary is built and available
        let binary = std::path::PathBuf::from(
            std::env::var("CARGO_BIN_EXE_ethcli-mcp").unwrap_or_else(|_| {
                // Fallback for manual test runs
                let manifest_dir =
                    std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
                let workspace_root = std::path::Path::new(&manifest_dir)
                    .parent()
                    .and_then(|p| p.parent())
                    .unwrap_or(std::path::Path::new("."));

                let release_path = workspace_root.join("target/release/ethcli-mcp");
                let debug_path = workspace_root.join("target/debug/ethcli-mcp");

                if release_path.exists() {
                    release_path.to_string_lossy().to_string()
                } else if debug_path.exists() {
                    debug_path.to_string_lossy().to_string()
                } else {
                    panic!(
                        "ethcli-mcp binary not found. Run 'cargo build -p ethcli-mcp' first.\n\
                         Searched: {:?}, {:?}",
                        release_path, debug_path
                    );
                }
            }),
        );

        let proc = Command::new(&binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap_or_else(|e| panic!("Failed to spawn ethcli-mcp at {:?}: {}", binary, e));

        Self {
            proc,
            request_id: 0,
        }
    }

    fn send_request(&mut self, method: &str, params: Option<Value>) -> Value {
        self.request_id += 1;
        let mut request = json!({
            "jsonrpc": "2.0",
            "id": self.request_id,
            "method": method
        });
        if let Some(p) = params {
            request["params"] = p;
        }

        let stdin = self.proc.stdin.as_mut().expect("Failed to get stdin");
        writeln!(stdin, "{}", request).expect("Failed to write request");
        stdin.flush().expect("Failed to flush stdin");

        let stdout = self.proc.stdout.as_mut().expect("Failed to get stdout");
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .expect("Failed to read response");

        serde_json::from_str(&line).unwrap_or_else(|_| json!({"error": "Failed to parse response"}))
    }

    fn send_notification(&mut self, method: &str, params: Option<Value>) {
        let mut notification = json!({
            "jsonrpc": "2.0",
            "method": method
        });
        if let Some(p) = params {
            notification["params"] = p;
        }

        let stdin = self.proc.stdin.as_mut().expect("Failed to get stdin");
        writeln!(stdin, "{}", notification).expect("Failed to write notification");
        stdin.flush().expect("Failed to flush stdin");
    }

    fn initialize(&mut self) -> bool {
        let resp = self.send_request(
            "initialize",
            Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {"name": "ethcli-mcp-test", "version": "1.0.0"}
            })),
        );

        if resp.get("error").is_some() {
            return false;
        }

        self.send_notification("notifications/initialized", None);
        true
    }

    fn call_tool(&mut self, name: &str, arguments: Value) -> Value {
        self.send_request(
            "tools/call",
            Some(json!({"name": name, "arguments": arguments})),
        )
    }

    fn list_tools(&mut self) -> Value {
        self.send_request("tools/list", None)
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.proc.kill();
        let _ = self.proc.wait();
    }
}

/// Helper to check if a tool call succeeded
fn is_tool_success(response: &Value) -> bool {
    if response.get("error").is_some() {
        return false;
    }

    if let Some(result) = response.get("result") {
        if let Some(content) = result.get("content") {
            if let Some(arr) = content.as_array() {
                if let Some(first) = arr.first() {
                    if let Some(text) = first.get("text").and_then(|t| t.as_str()) {
                        // Check for error patterns
                        return !text.starts_with("Error:")
                            && !text.contains("\nError:")
                            && !text.contains("Command failed");
                    }
                }
            }
        }
    }

    true
}

/// Get the text content from a tool response
fn get_tool_text(response: &Value) -> Option<String> {
    response
        .get("result")?
        .get("content")?
        .as_array()?
        .first()?
        .get("text")?
        .as_str()
        .map(String::from)
}

/// Retry a tool call up to `max_retries` times with a delay between attempts.
/// Useful for flaky tests that may fail due to rate limiting.
fn call_tool_with_retry(
    client: &mut McpClient,
    name: &str,
    arguments: Value,
    max_retries: u32,
) -> Value {
    let mut last_response = json!({});
    for attempt in 0..=max_retries {
        let response = client.call_tool(name, arguments.clone());
        if is_tool_success(&response) {
            return response;
        }
        last_response = response;
        if attempt < max_retries {
            std::thread::sleep(std::time::Duration::from_millis(500 * (attempt as u64 + 1)));
        }
    }
    last_response
}

// =============================================================================
// MCP Protocol Tests (no network required)
// =============================================================================

#[test]
fn test_mcp_initialize() {
    let mut client = McpClient::new();
    assert!(client.initialize(), "MCP initialization should succeed");
}

#[test]
fn test_mcp_list_tools() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.list_tools();
    assert!(
        response.get("error").is_none(),
        "tools/list should not error"
    );

    let tools = response["result"]["tools"]
        .as_array()
        .expect("Should have tools array");
    assert!(
        tools.len() > 100,
        "Should have many tools, got {}",
        tools.len()
    );

    // Check for some expected tools
    let tool_names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();

    assert!(
        tool_names.contains(&"cast_to_wei"),
        "Should have cast_to_wei"
    );
    assert!(
        tool_names.contains(&"cast_keccak"),
        "Should have cast_keccak"
    );
    assert!(
        tool_names.contains(&"rpc_block_number"),
        "Should have rpc_block_number"
    );
    assert!(
        tool_names.contains(&"lifi_chains"),
        "Should have lifi_chains"
    );
}

// =============================================================================
// Input Validation Tests (no network required, not ignored)
// =============================================================================

#[test]
fn test_invalid_tool_name() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("nonexistent_tool_xyz", json!({}));
    assert!(
        response.get("error").is_some(),
        "Should error on unknown tool"
    );
}

#[test]
fn test_missing_required_params() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    // cast_to_wei requires 'amount' parameter
    let response = client.call_tool("cast_to_wei", json!({}));
    // Should either error or have isError in result
    let has_error = response.get("error").is_some()
        || response
            .get("result")
            .and_then(|r| r.get("isError"))
            .and_then(|e| e.as_bool())
            .unwrap_or(false);
    assert!(has_error, "Should error on missing required params");
}

#[test]
// Requires ethcli binary (built in CI)
fn test_empty_string_param() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    // Empty string should be handled gracefully
    let response = client.call_tool("cast_keccak", json!({"value": ""}));
    // Empty keccak is valid (hash of empty string)
    assert!(
        is_tool_success(&response),
        "Empty string keccak should succeed"
    );
}

// =============================================================================
// Pure Conversion Tests (no network required, use ethcli cast)
// =============================================================================

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_to_wei() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("cast_to_wei", json!({"amount": "1", "unit": "eth"}));
    assert!(is_tool_success(&response), "cast_to_wei should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("1000000000000000000"),
        "1 ETH should be 1e18 wei, got: {}",
        text
    );
}

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_from_wei() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool(
        "cast_from_wei",
        json!({"wei": "1000000000000000000", "unit": "eth"}),
    );
    assert!(is_tool_success(&response), "cast_from_wei should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("1"),
        "1e18 wei should be 1 ETH, got: {}",
        text
    );
}

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_to_hex() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("cast_to_hex", json!({"value": "255"}));
    assert!(is_tool_success(&response), "cast_to_hex should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.to_lowercase().contains("0xff") || text.contains("ff"),
        "255 should be 0xff, got: {}",
        text
    );
}

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_to_dec() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("cast_to_dec", json!({"value": "0xff"}));
    assert!(is_tool_success(&response), "cast_to_dec should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(text.contains("255"), "0xff should be 255, got: {}", text);
}

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_keccak() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("cast_keccak", json!({"value": "hello"}));
    assert!(is_tool_success(&response), "cast_keccak should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    // keccak256("hello") = 0x1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8
    assert!(
        text.to_lowercase()
            .contains("1c8aff950685c2ed4bc3174f3472287b56d9517b9c948127319a09a7a36deac8"),
        "keccak256('hello') mismatch, got: {}",
        text
    );
}

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_sig() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool(
        "cast_sig",
        json!({"signature": "transfer(address,uint256)"}),
    );
    assert!(is_tool_success(&response), "cast_sig should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    // transfer(address,uint256) selector = 0xa9059cbb
    assert!(
        text.to_lowercase().contains("a9059cbb"),
        "transfer selector should be a9059cbb, got: {}",
        text
    );
}

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_checksum() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool(
        "cast_checksum",
        json!({"value": "0xd8da6bf26964af9d7eed9e03e53415d37aa96045"}),
    );
    assert!(is_tool_success(&response), "cast_checksum should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains(VITALIK),
        "Should return checksummed address, got: {}",
        text
    );
}

#[test]
// Requires ethcli binary (built in CI)
fn test_cast_abi_encode() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool(
        "cast_abi_encode",
        json!({
            "sig": "transfer(address,uint256)",
            "args": [VITALIK, "1000000"]
        }),
    );
    assert!(is_tool_success(&response), "cast_abi_encode should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.starts_with("0x") || text.contains("0x"),
        "Should return hex data"
    );
}

// =============================================================================
// Signature Lookup Tests (uses 4byte.directory, may need network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_sig_fn() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("sig_fn", json!({"selector": "0xa9059cbb"}));
    assert!(is_tool_success(&response), "sig_fn should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("transfer"),
        "0xa9059cbb should be transfer, got: {}",
        text
    );
}

#[test]
#[ignore] // Requires network
fn test_sig_event() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool(
        "sig_event",
        json!({"selector": "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"}),
    );
    assert!(is_tool_success(&response), "sig_event should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("Transfer"),
        "Should be Transfer event, got: {}",
        text
    );
}

// =============================================================================
// ENS Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_ens_resolve() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("ens_resolve", json!({"name": "vitalik.eth"}));
    assert!(is_tool_success(&response), "ens_resolve should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.to_lowercase().contains(&VITALIK.to_lowercase()),
        "vitalik.eth should resolve to Vitalik's address, got: {}",
        text
    );
}

#[test]
#[ignore] // Requires network
fn test_ens_lookup() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("ens_lookup", json!({"address": VITALIK}));
    assert!(is_tool_success(&response), "ens_lookup should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("vitalik.eth"),
        "Vitalik's address should resolve to vitalik.eth, got: {}",
        text
    );
}

#[test]
// Requires ethcli binary (built in CI) (pure computation)
fn test_ens_namehash() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("ens_namehash", json!({"name": "vitalik.eth"}));
    assert!(is_tool_success(&response), "ens_namehash should succeed");
}

// =============================================================================
// RPC Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_rpc_block_number() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("rpc_block_number", json!({}));
    assert!(
        is_tool_success(&response),
        "rpc_block_number should succeed"
    );

    let text = get_tool_text(&response).expect("Should have text");
    // Block number should be a large number (>15M for mainnet)
    let has_number = text.chars().any(|c| c.is_ascii_digit());
    assert!(has_number, "Should contain block number, got: {}", text);
}

#[test]
#[ignore] // Requires network
fn test_rpc_chain_id() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("rpc_chain_id", json!({}));
    assert!(is_tool_success(&response), "rpc_chain_id should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("1"),
        "Mainnet chain ID should be 1, got: {}",
        text
    );
}

#[test]
#[ignore] // Requires network
fn test_rpc_gas_price() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("rpc_gas_price", json!({}));
    assert!(is_tool_success(&response), "rpc_gas_price should succeed");
}

#[test]
#[ignore] // Requires network
fn test_rpc_nonce() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("rpc_nonce", json!({"address": VITALIK}));
    assert!(is_tool_success(&response), "rpc_nonce should succeed");
}

#[test]
#[ignore] // Requires network
fn test_rpc_code() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("rpc_code", json!({"address": USDC}));
    assert!(is_tool_success(&response), "rpc_code should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.starts_with("0x") && text.len() > 10,
        "USDC should have bytecode, got: {}",
        text
    );
}

#[test]
#[ignore] // Requires network
fn test_rpc_block() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("rpc_block", json!({"block": "latest"}));
    assert!(is_tool_success(&response), "rpc_block should succeed");
}

// =============================================================================
// Account Tests (requires network + Etherscan API)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_account_balance() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("account_balance", json!({"address": VITALIK}));
    assert!(is_tool_success(&response), "account_balance should succeed");
}

#[test]
#[ignore] // Requires network
fn test_account_info() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("account_info", json!({"address": VITALIK}));
    assert!(is_tool_success(&response), "account_info should succeed");
}

// =============================================================================
// Gas Oracle Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_gas_oracle() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    // Use retry logic - gas oracle aggregates multiple sources which may rate limit
    let response = call_tool_with_retry(&mut client, "gas_oracle", json!({}), 2);
    assert!(is_tool_success(&response), "gas_oracle should succeed");
}

// =============================================================================
// Contract Tests (requires network + Etherscan API)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_contract_abi() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("contract_abi", json!({"address": USDC}));
    assert!(is_tool_success(&response), "contract_abi should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    // USDC is a proxy contract, so check for valid ABI structure
    assert!(
        text.contains("type")
            && (text.contains("function")
                || text.contains("constructor")
                || text.contains("fallback")),
        "Should return valid ABI, got: {}",
        &text[..200.min(text.len())]
    );
}

#[test]
#[ignore] // Requires network
fn test_contract_creation() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("contract_creation", json!({"address": USDC}));
    assert!(
        is_tool_success(&response),
        "contract_creation should succeed"
    );
}

// =============================================================================
// Token Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_token_info() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("token_info", json!({"address": USDC}));
    assert!(is_tool_success(&response), "token_info should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("USDC") || text.contains("USD Coin"),
        "Should contain USDC info, got: {}",
        text
    );
}

#[test]
#[ignore] // Requires network
fn test_token_balance() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("token_balance", json!({"token": USDC, "address": VITALIK}));
    assert!(is_tool_success(&response), "token_balance should succeed");
}

// =============================================================================
// Chainlink Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_chainlink_price() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("chainlink_price", json!({"token": "ETH"}));
    assert!(is_tool_success(&response), "chainlink_price should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    // ETH price should be > $100
    assert!(
        text.chars().any(|c| c.is_ascii_digit()),
        "Should contain price, got: {}",
        text
    );
}

#[test]
#[ignore] // Requires network
fn test_chainlink_oracles() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("chainlink_oracles", json!({}));
    assert!(
        is_tool_success(&response),
        "chainlink_oracles should succeed"
    );
}

// =============================================================================
// LI.FI Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_lifi_chains() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("lifi_chains", json!({}));
    assert!(is_tool_success(&response), "lifi_chains should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("Ethereum") || text.contains("ethereum"),
        "Should list Ethereum, got: {}",
        &text[..500.min(text.len())]
    );
}

#[test]
#[ignore] // Requires network
fn test_lifi_gas() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    // Use retry logic - LI.FI API may rate limit
    let response = call_tool_with_retry(&mut client, "lifi_gas", json!({"chain_id": "1"}), 2);
    assert!(is_tool_success(&response), "lifi_gas should succeed");

    let text = get_tool_text(&response).expect("Should have text");
    assert!(
        text.contains("standard") || text.contains("fast"),
        "Should have gas prices, got: {}",
        text
    );
}

#[test]
#[ignore] // Requires network
fn test_lifi_tools() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("lifi_tools", json!({}));
    assert!(is_tool_success(&response), "lifi_tools should succeed");
}

#[test]
#[ignore] // Requires network
fn test_lifi_bridges() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("lifi_bridges", json!({}));
    assert!(is_tool_success(&response), "lifi_bridges should succeed");
}

#[test]
#[ignore] // Requires network
fn test_lifi_exchanges() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("lifi_exchanges", json!({}));
    assert!(is_tool_success(&response), "lifi_exchanges should succeed");
}

// =============================================================================
// Curve Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_curve_pools() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("curve_pools", json!({"chain": "ethereum"}));
    assert!(is_tool_success(&response), "curve_pools should succeed");
}

#[test]
#[ignore] // Requires network
fn test_curve_crvusd() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("curve_crvusd", json!({}));
    assert!(is_tool_success(&response), "curve_crvusd should succeed");
}

// =============================================================================
// CCXT Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_ccxt_ticker() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool(
        "ccxt_ticker",
        json!({"exchange": "binance", "symbol": "BTC/USDT"}),
    );
    assert!(is_tool_success(&response), "ccxt_ticker should succeed");
}

#[test]
#[ignore] // Requires network
fn test_ccxt_markets() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("ccxt_markets", json!({"exchange": "binance"}));
    assert!(is_tool_success(&response), "ccxt_markets should succeed");
}

// =============================================================================
// Pyth Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_pyth_price() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("pyth_price", json!({"symbols": "BTC/USD"}));
    assert!(is_tool_success(&response), "pyth_price should succeed");
}

#[test]
#[ignore] // Requires network
fn test_pyth_search() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("pyth_search", json!({"query": "ETH"}));
    assert!(is_tool_success(&response), "pyth_search should succeed");
}

// =============================================================================
// KyberSwap Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_kyberswap_routes() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool(
        "kyberswap_routes",
        json!({
            "token_in": WETH,
            "token_out": USDC,
            "amount_in": "1000000000000000000"
        }),
    );
    assert!(
        is_tool_success(&response),
        "kyberswap_routes should succeed"
    );
}

// =============================================================================
// OpenOcean Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_openocean_tokens() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("openocean_tokens", json!({}));
    assert!(
        is_tool_success(&response),
        "openocean_tokens should succeed"
    );
}

#[test]
#[ignore] // Requires network
fn test_openocean_dexes() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("openocean_dexes", json!({}));
    assert!(is_tool_success(&response), "openocean_dexes should succeed");
}

// =============================================================================
// Velora Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_velora_tokens() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("velora_tokens", json!({}));
    assert!(is_tool_success(&response), "velora_tokens should succeed");
}

// =============================================================================
// GoPlus Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_goplus_chains() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("goplus_chains", json!({}));
    assert!(is_tool_success(&response), "goplus_chains should succeed");
}

#[test]
#[ignore] // Requires network
fn test_goplus_token() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("goplus_token", json!({"address": USDC, "chain_id": 1}));
    assert!(is_tool_success(&response), "goplus_token should succeed");
}

// =============================================================================
// DefiLlama Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_llama_stablecoins() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("llama_stablecoins", json!({}));
    assert!(
        is_tool_success(&response),
        "llama_stablecoins should succeed"
    );
}

// =============================================================================
// Config/Endpoints Tests (local config, no network for some)
// =============================================================================

#[test]
// Requires ethcli binary (built in CI)
fn test_config_path() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("config_path", json!({}));
    assert!(is_tool_success(&response), "config_path should succeed");
}

#[test]
// Requires ethcli binary (built in CI)
fn test_config_show() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("config_show", json!({}));
    assert!(is_tool_success(&response), "config_show should succeed");
}

#[test]
// Requires ethcli binary (built in CI)
fn test_endpoints_list() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("endpoints_list", json!({}));
    assert!(is_tool_success(&response), "endpoints_list should succeed");
}

#[test]
#[ignore] // Requires network
fn test_endpoints_health() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("endpoints_health", json!({}));
    assert!(
        is_tool_success(&response),
        "endpoints_health should succeed"
    );
}

// =============================================================================
// Address Book Tests (local storage)
// =============================================================================

#[test]
// Requires ethcli binary (built in CI)
fn test_address_list() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("address_list", json!({}));
    assert!(is_tool_success(&response), "address_list should succeed");
}

#[test]
// Requires ethcli binary (built in CI)
fn test_address_search() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("address_search", json!({"query": "vitalik"}));
    assert!(is_tool_success(&response), "address_search should succeed");
}

// =============================================================================
// Blacklist Tests (local storage)
// =============================================================================

#[test]
// Requires ethcli binary (built in CI)
fn test_blacklist_list() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("blacklist_list", json!({}));
    assert!(is_tool_success(&response), "blacklist_list should succeed");
}

#[test]
// Requires ethcli binary (built in CI)
fn test_blacklist_check() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("blacklist_check", json!({"address": USDC}));
    assert!(is_tool_success(&response), "blacklist_check should succeed");
}

// =============================================================================
// Doctor Test (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_doctor() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("doctor", json!({}));
    // Doctor may report warnings, which is fine
    assert!(
        response.get("error").is_none(),
        "doctor should not return JSON-RPC error"
    );
}

// =============================================================================
// Kong (Yearn) Tests (requires network)
// =============================================================================

#[test]
#[ignore] // Requires network
fn test_kong_vaults() {
    let mut client = McpClient::new();
    assert!(client.initialize());

    let response = client.call_tool("kong_vaults", json!({}));
    assert!(is_tool_success(&response), "kong_vaults should succeed");
}
