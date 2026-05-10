<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">ethcli-mcp</h1>

<p align="center">
  MCP server exposing <a href="../ethcli">ethcli</a> functionality as tools for AI assistants
</p>

<p align="center">
  <a href="https://crates.io/crates/ethcli-mcp"><img src="https://img.shields.io/crates/v/ethcli-mcp.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/ethcli-mcp/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Overview

Exposes 200+ Ethereum tools via the [Model Context Protocol](https://modelcontextprotocol.io/) (MCP), enabling AI assistants like Claude to query blockchains, analyze transactions, fetch DeFi data, and interact with 50+ data sources. The default MCP surface is read-only for local and remote state changes.

## Features

- **230+ MCP tools** - Broad ethcli functionality exposed as typed tools, with mutating tools gated by policy
- **JSON Schema validation** - All tool inputs have schemas for LLM structured output
- **Multi-chain support** - Ethereum, Polygon, Arbitrum, Optimism, Base, and more
- **DEX aggregators** - 1inch, CowSwap, LI.FI, KyberSwap, OpenOcean, 0x, Velora, Enso
- **Data sources** - CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth, Dune
- **Security tools** - GoPlus token analysis, Solodit vulnerability search, bytecode security analysis with dispatcher and handler guard heuristics

## Installation

### Prerequisites

**ethcli must be installed separately.** The MCP server spawns ethcli as a subprocess.

```bash
cargo install ethcli
ethcli --version
```

`ethcli-mcp` looks for `ethcli` next to the MCP binary. If you use a custom binary location, set `ETHCLI_PATH` to an absolute path.

### Install ethcli-mcp

```bash
cargo install ethcli-mcp
```

## Quick Start

### Claude Desktop

Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "ethcli": {
      "command": "ethcli-mcp"
    }
  }
}
```

With a custom ethcli binary:

```json
{
  "mcpServers": {
    "ethcli": {
      "command": "ethcli-mcp",
      "env": {
        "ETHCLI_PATH": "/absolute/path/to/ethcli"
      }
    }
  }
}
```

### Manual Testing

```bash
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"test","version":"1.0.0"}}}' | ethcli-mcp
```

## Tool Categories

| Category | Count | Examples |
|----------|-------|----------|
| `cast_*` | 14 | Unit conversions, hashing, ABI encoding |
| `lifi_*` | 12 | Cross-chain quotes, bridges, chains |
| `curve_*` | 10 | Pools, routing, lending |
| `uniswap_*` | 9 | V2/V3/V4 pool queries |
| `rpc_*` | 9 | Direct blockchain calls |
| `tenderly_*` | 34 | Simulation, VNets (CRUD + Admin RPC + RPC Simulation) |
| `cowswap_*` | 8 | MEV-protected trading |
| `account_*` | 8 | Balance, transactions |
| `oneinch_*` | 7 | DEX aggregator |
| `goplus_*` | 6 | Security analysis |
| Other | 175 | See full list below |

<details>
<summary>All 42 tool categories</summary>

| Category | Count |
|----------|-------|
| cast_* | 14 |
| config_* | 14 |
| lifi_* | 12 |
| curve_* | 10 |
| uniswap_* | 9 |
| rpc_* | 9 |
| tenderly_* | 34 |
| simulate_* | 8 |
| endpoints_* | 8 |
| cowswap_* | 8 |
| account_* | 8 |
| oneinch_* | 7 |
| dsim_* | 7 |
| ccxt_* | 7 |
| blacklist_* | 7 |
| address_* | 7 |
| llama_* | 6 |
| goplus_* | 6 |
| alchemy_* | 6 |
| solodit_* | 5 |
| openocean_* | 5 |
| moralis_* | 5 |
| kong_* | 5 |
| gecko_* | 5 |
| sig_* | 4 |
| pyth_* | 4 |
| ens_* | 4 |
| contract_* | 8 |
| chainlink_* | 4 |
| zerox_* | 3 |
| velora_* | 3 |
| token_* | 3 |
| quote_* | 3 |
| kyberswap_* | 3 |
| enso_* | 3 |
| dune_* | 3 |
| gas_* | 2 |
| yields | 1 |
| tx | 1 |
| price | 1 |
| portfolio | 1 |
| nfts | 1 |
| logs | 1 |
| doctor | 1 |

</details>

## Environment Variables

| Variable | Required For |
|----------|-------------|
| `ETHCLI_PATH` | Absolute custom ethcli binary path |
| `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1` | Opt in to MCP tools that mutate local/remote state |
| `ETHERSCAN_API_KEY` | Higher rate limits (optional) |
| `ALCHEMY_API_KEY` | `alchemy_*` tools |
| `TENDERLY_ACCESS_KEY` | `tenderly_*` tools |
| `ONEINCH_API_KEY` | `oneinch_*` tools |
| `ENSO_API_KEY` | `enso_*` tools |
| `DUNE_API_KEY` | `dune_*` tools |
| `MORALIS_API_KEY` | `moralis_*` tools |
| `SOLODIT_API_KEY` | `solodit_*` tools |
| `THEGRAPH_API_KEY` | `uniswap_top_pools`, etc. |

## Unverified Contract Analysis

The `contract_analyze` MCP tool exposes ethcli's native unverified-contract workflow. Set `follow_proxy`, `lookup`, `dispatcher`, and `checks` to inspect proxy implementations, resolve selectors, map selector handlers, and flag side-effecting handler bodies that appear to lack caller gates or calldata hash validation. `contract_selectors` also supports `follow_proxy` when you want implementation selectors rather than proxy wrapper selectors.

Equivalent CLI:

```bash
ethcli contract analyze 0x... --follow-proxy --lookup --dispatcher --checks --format json
```

For endpoint diagnostics, `endpoints_health` returns JSON and defaults to `probes: 1` so MCP calls stay bounded. Increase `probes` when you need a deeper endpoint check.

## Architecture

```
┌─────────────────┐     JSON-RPC      ┌─────────────────┐    subprocess    ┌─────────────┐
│  Claude/LLM     │ ◄──── STDIO ────► │   ethcli-mcp    │ ◄─────────────► │   ethcli    │
└─────────────────┘                   └─────────────────┘                  └─────────────┘
```

## Security

- **Read-only by default** - Local/remote mutating tools such as `config_set_*`, `address_add`, `blacklist_add`, `endpoints_add`, `chainlist_add`, `simulate_share`, Dune write APIs, Tenderly VNet/admin mutations, and CowSwap create/cancel order are blocked unless `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1` is set.
- **No raw config disclosure** - `config_show` returns safe config status only. It does not return config file contents because they may contain API keys or private RPC URLs.
- **No arbitrary file output** - MCP rejects file-output paths for tools such as `contract_abi`, `contract_source`, and `address_export`; omit `output` to receive data in the tool response.
- **No secret-bearing subprocess args** - MCP rejects direct credential/header flags such as `--show-secrets`, `--tenderly-key`, `--alchemy-key`, `--etherscan-api-key`, `--rpc-header`, and `--fork-header`. Configure credentials via environment variables or ethcli config instead.
- **Success-output redaction** - Credential-bearing URLs and secret-looking header/config lines are redacted before returning MCP tool responses.
- **Process-group cleanup** - Timed-out subprocesses are killed as a process group so child tools such as `anvil` do not remain running.
- **Rate limiting** - Max 10 concurrent subprocesses
- **Timeouts** - 10 seconds for fast local commands, 30 seconds for network commands
- **Input validation** - Argument length limits, null byte detection
- **Error sanitization** - API keys filtered from error messages

## Development

See [CONTRIBUTING.md](https://github.com/yldfi/yldfi-rs/blob/main/CONTRIBUTING.md) for guidelines.

```bash
cargo build -p ethcli-mcp
cargo test -p ethcli-mcp --test integration
```

## License

MIT
