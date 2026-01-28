#!/usr/bin/env python3
"""
Full MCP server test suite for ethcli-mcp.
Tests representative tools from each category.
"""

import json
import subprocess
import sys
from dataclasses import dataclass
from typing import Optional, List, Tuple

# Test addresses and values
VITALIK = "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045"
USDC = "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"
WETH = "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2"
TX_HASH = "0x5c504ed432cb51138bcf09aa5e8a410dd4a1e204ef84bfed1be16dfba1b22060"

MCP_BINARY = "./target/release/ethcli-mcp"

@dataclass
class TestResult:
    tool: str
    success: bool
    error: Optional[str] = None

class MCPClient:
    def __init__(self):
        self.proc = None
        self.request_id = 0

    def start(self):
        self.proc = subprocess.Popen(
            [MCP_BINARY],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            bufsize=1
        )

    def stop(self):
        if self.proc:
            self.proc.terminate()
            self.proc.wait()
            self.proc = None

    def send_request(self, method: str, params: dict = None) -> dict:
        self.request_id += 1
        request = {"jsonrpc": "2.0", "id": self.request_id, "method": method}
        if params is not None:
            request["params"] = params

        try:
            self.proc.stdin.write(json.dumps(request) + "\n")
            self.proc.stdin.flush()
            line = self.proc.stdout.readline()
            return json.loads(line) if line else {"error": "No response"}
        except Exception as e:
            return {"error": str(e)}

    def send_notification(self, method: str, params: dict = None):
        notification = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            notification["params"] = params
        self.proc.stdin.write(json.dumps(notification) + "\n")
        self.proc.stdin.flush()

    def initialize(self) -> bool:
        resp = self.send_request("initialize", {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {"name": "ethcli-mcp-test", "version": "1.0.0"}
        })
        if "error" in resp:
            return False
        self.send_notification("notifications/initialized")
        return True

    def call_tool(self, name: str, arguments: dict) -> dict:
        return self.send_request("tools/call", {"name": name, "arguments": arguments})


def test_tool(client: MCPClient, name: str, arguments: dict, expect_api_error: bool = False) -> TestResult:
    resp = client.call_tool(name, arguments)

    if "error" in resp:
        return TestResult(name, False, error=str(resp["error"])[:100])

    if "result" in resp:
        result = resp["result"]
        if isinstance(result, dict) and "content" in result:
            content = result["content"]
            if isinstance(content, list) and len(content) > 0:
                text = content[0].get("text", "")
                # Check for command execution errors (Error: at start of line indicates actual error)
                # Don't flag informational output that happens to contain "error" in other contexts
                is_error = text.startswith("Error:") or "\nError:" in text or "Command failed" in text
                if is_error:
                    # If we expected an API error, this is OK
                    if expect_api_error and ("HTTP error" in text or "API error" in text or "403" in text or "GraphQL error" in text):
                        return TestResult(name, True)
                    return TestResult(name, False, error=text[:100])
                return TestResult(name, True)
        return TestResult(name, True)

    return TestResult(name, False, error="Unknown response format")


# Tests organized by category
# (name, arguments, expect_api_error)
TESTS: List[Tuple[str, dict, bool]] = [
    # CAST - Pure conversions, no network
    ("cast_to_wei", {"amount": "1", "unit": "eth"}, False),
    ("cast_from_wei", {"wei": "1000000000000000000", "unit": "eth"}, False),
    ("cast_to_hex", {"value": "255"}, False),
    ("cast_to_dec", {"value": "0xff"}, False),
    ("cast_keccak", {"value": "hello"}, False),
    ("cast_sig", {"signature": "transfer(address,uint256)"}, False),
    ("cast_topic", {"signature": "Transfer(address,address,uint256)"}, False),
    ("cast_checksum", {"value": VITALIK.lower()}, False),
    ("cast_to_bytes32", {"value": "0x1234"}, False),
    ("cast_abi_encode", {"sig": "transfer(address,uint256)", "args": [VITALIK, "1000000"]}, False),

    # SIG - Signature lookup
    ("sig_fn", {"selector": "0xa9059cbb"}, False),
    ("sig_event", {"selector": "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"}, False),
    ("sig_cache_stats", {}, False),

    # ENS
    ("ens_resolve", {"name": "vitalik.eth"}, False),
    ("ens_lookup", {"address": VITALIK}, False),
    ("ens_namehash", {"name": "vitalik.eth"}, False),

    # RPC
    ("rpc_block_number", {}, False),
    ("rpc_chain_id", {}, False),
    ("rpc_gas_price", {}, False),
    ("rpc_nonce", {"address": VITALIK}, False),
    ("rpc_code", {"address": USDC}, False),
    ("rpc_block", {"block": "latest"}, False),

    # ACCOUNT
    ("account_balance", {"address": VITALIK}, False),
    ("account_info", {"address": VITALIK}, False),

    # GAS
    ("gas_oracle", {}, False),

    # CONTRACT
    ("contract_abi", {"address": USDC}, False),
    ("contract_creation", {"address": USDC}, False),

    # TOKEN
    ("token_info", {"address": USDC}, False),
    ("token_balance", {"token": USDC, "address": VITALIK}, False),

    # ADDRESS
    ("address_list", {}, False),
    ("address_search", {"query": "vitalik"}, False),

    # BLACKLIST
    ("blacklist_list", {}, False),
    ("blacklist_check", {"address": USDC}, False),

    # CONFIG
    ("config_path", {}, False),
    ("config_show", {}, False),
    ("config_validate", {}, False),

    # ENDPOINTS
    ("endpoints_list", {}, False),
    ("endpoints_health", {}, False),

    # CHAINLINK
    ("chainlink_price", {"token": "ETH"}, False),
    ("chainlink_oracles", {}, False),

    # GOPLUS
    ("goplus_chains", {}, False),
    ("goplus_token", {"address": USDC, "chain_id": 1}, False),

    # LLAMA (only stablecoins implemented, others not in MCP)
    ("llama_stablecoins", {}, False),

    # CURVE
    ("curve_pools", {}, False),
    ("curve_crvusd", {}, False),
    ("curve_dao", {}, False),
    ("curve_volumes", {}, False),

    # CCXT
    ("ccxt_ticker", {"exchange": "binance", "symbol": "BTC/USDT"}, False),
    ("ccxt_markets", {"exchange": "binance"}, False),

    # PYTH
    ("pyth_price", {"symbols": "BTC/USD"}, False),
    ("pyth_search", {"query": "ETH"}, False),
    ("pyth_known_feeds", {}, False),

    # KYBERSWAP
    ("kyberswap_routes", {"token_in": WETH, "token_out": USDC, "amount_in": "1000000000000000000"}, False),

    # COWSWAP - Some endpoints have API issues
    ("cowswap_native_price", {"token": USDC}, False),
    ("cowswap_auction", {}, True),  # Known 403 error

    # LIFI - Fixed API format
    ("lifi_chains", {}, False),
    ("lifi_gas", {"chain_id": "1"}, False),  # Gas endpoint now uses /gas/prices
    ("lifi_tools", {}, False),
    ("lifi_bridges", {}, False),
    ("lifi_exchanges", {}, False),

    # OPENOCEAN - API format issues (fixed with flexible deserialization)
    ("openocean_tokens", {}, False),
    ("openocean_dexes", {}, False),
    ("openocean_quote", {"in_token": WETH, "out_token": USDC, "amount": "1000000000000000000"}, True),  # May have rate limiting

    # VELORA - API format fixed
    ("velora_tokens", {}, False),

    # DOCTOR (warnings are expected, not errors)
    ("doctor", {}, False),

    # KONG/YEARN
    ("kong_vaults", {}, False),
    ("kong_strategies", {}, True),  # GraphQL API may have issues

    # UNISWAP (requires THEGRAPH_API_KEY)
    ("uniswap_eth_price", {}, True),  # Subgraph requires API key
    ("uniswap_top_pools", {"limit": 5}, True),  # Subgraph requires API key
]


def run_tests():
    client = MCPClient()
    client.start()

    print("Initializing MCP connection...")
    if not client.initialize():
        print("Failed to initialize MCP connection")
        client.stop()
        return 1

    results = []
    passed = 0
    failed = 0
    expected_failures = 0

    print(f"\nRunning {len(TESTS)} tests...\n")

    for name, args, expect_api_error in TESTS:
        result = test_tool(client, name, args, expect_api_error)
        results.append((result, expect_api_error))

        if result.success:
            passed += 1
            print(f"  OK: {name}")
        else:
            if expect_api_error:
                expected_failures += 1
                print(f"  EXPECTED FAIL: {name} (external API issue)")
            else:
                failed += 1
                print(f"  FAIL: {name}")
                print(f"        {result.error}")

    client.stop()

    print("\n" + "=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"Passed: {passed}")
    print(f"Failed: {failed}")
    print(f"Expected failures (external API): {expected_failures}")
    print(f"Total: {len(results)}")
    print(f"Success rate (excluding API issues): {passed}/{passed + failed} ({100*passed/(passed+failed) if passed+failed > 0 else 0:.1f}%)")

    if failed > 0:
        print("\nUnexpected failures:")
        for result, expected in results:
            if not result.success and not expected:
                print(f"  - {result.tool}: {result.error}")

    return 0 if failed == 0 else 1


if __name__ == "__main__":
    sys.exit(run_tests())
