<p align="center">
  <img src="../../logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">ethcli-mcp</h1>

<p align="center">
  MCP server exposing <a href="https://github.com/yldfi/yldfi-rs/tree/main/crates/ethcli">ethcli</a> functionality as tools for AI assistants
</p>

<p align="center">
  <a href="https://crates.io/crates/ethcli-mcp"><img src="https://img.shields.io/crates/v/ethcli-mcp.svg" alt="crates.io"></a>
  <a href="https://opensource.org/license/mit"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Overview

Exposes 580+ Ethereum tools via the [Model Context Protocol](https://modelcontextprotocol.io/) (MCP), enabling AI assistants like Claude to query blockchains, analyze transactions, fetch DeFi data, and interact with 50+ data sources. The default MCP surface is read-only for local and remote state changes.

## Features

`account_txs` accepts optional inclusive `start_block` and `end_block` bounds,
alongside `page`, `limit`, and `sort`. For example:

```json
{"address":"0x0000000000000000000000000000000000000000","start_block":25893052,"end_block":26108081,"page":1,"limit":1000,"sort":"asc"}
```

Omitting bounds preserves the CLI defaults (0 through 99999999). Explorers can
cap a result window even when requests stay below their rate limit. If that
happens, split the history into smaller block ranges and paginate each range.
This tool does not automatically split ranges or bypass provider limits.

- **580+ MCP tools** - Broad ethcli functionality exposed as typed tools, with mutating tools gated by policy
- **JSON Schema validation** - All tool inputs have schemas for LLM structured output
- **Multi-chain support** - Ethereum, Polygon, Arbitrum, Optimism, Base, and more
- **DEX aggregators** - 1inch, CowSwap, LI.FI, KyberSwap, OpenOcean, 0x, Velora, Enso
- **Data sources** - CoinGecko, DefiLlama, Alchemy, Moralis, Chainlink, Pyth, Dune
- **Security tools** - GoPlus token analysis, Solodit vulnerability search, bytecode security analysis with dispatcher and handler guard heuristics

## Installation

### Prerequisites

**ethcli must be installed separately.** The MCP server spawns ethcli as a subprocess.

```bash
cargo install --locked ethcli
ethcli --version
```

`ethcli-mcp` looks for `ethcli` next to the MCP binary. If you use a custom binary location, set `ETHCLI_PATH` to an absolute path. If `ethcli` and `ethcli-mcp` are wrapper scripts (for example, to inject secrets), point `ETHCLI_PATH` at the real `ethcli` binary rather than the wrapper, and have the wrappers exec real binaries by absolute path. A wrapper that resolves back to `ethcli-mcp` would make the server start itself recursively.

Built on [rmcp](https://crates.io/crates/rmcp) 3.x over stdio. It negotiates the MCP protocol version the client requests (tested with `2024-11-05` and `2025-06-18`).

### Install ethcli-mcp

```bash
cargo install --locked ethcli-mcp
```

### Agent skill

[`skills/ethcli/SKILL.md`](https://github.com/yldfi/yldfi-rs/blob/main/skills/ethcli/SKILL.md)
tells a coding agent (Claude Code, Codex and others that read skills) which
ethcli command answers which question, and which output fields to check
before trusting a result: partial balance changes, guessed error names,
incomplete histories. Copy the `skills/ethcli` folder into your agent's skills
directory, for example `~/.claude/skills/ethcli/` for Claude Code.

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
| `cast_*` | 15 | Unit conversions, hashing, ABI encoding |
| `lifi_*` | 19 | Cross-chain quotes, bridges, chains, transfers |
| `curve_*` | 10 | Pools, routing, lending |
| `uniswap_*` | 9 | V2/V3/V4 pool queries |
| `rpc_*` | 15 | Direct blockchain calls |
| `tenderly_*` | 34 | Simulation, VNets (CRUD + Admin RPC + RPC Simulation) |
| `cowswap_*` | 16 | MEV-protected trading |
| `account_*` | 10 | Balance, transactions |
| `oneinch_*` | 7 | DEX aggregator |
| `goplus_*` | 6 | Security analysis |
| Other | 452 | See full list below |

<details>
<summary>All 42 tool categories</summary>

| Category | Count |
|----------|-------|
| cast_* | 14 |
| config_* | 15 |
| lifi_* | 19 |
| curve_* | 10 |
| uniswap_* | 9 |
| rpc_* | 9 |
| tenderly_* | 34 |
| simulate_* | 8 |
| endpoints_* | 8 |
| cowswap_* | 16 |
| account_* | 8 |
| oneinch_* | 7 |
| cex_* | 7 |
| blacklist_* | 7 |
| address_* | 7 |
| llama_* | 15 |
| goplus_* | 6 |
| alchemy_* | 6 |
| solodit_* | 5 |
| openocean_* | 5 |
| moralis_* | 82 |
| kong_* | 5 |
| gecko_* | 54 |
| sig_* | 6 |
| pyth_* | 5 |
| ens_* | 4 |
| contract_* | 9 |
| chainlink_* | 4 |
| zerox_* | 7 |
| velora_* | 4 |
| token_* | 3 |
| quote_* | 3 |
| kyberswap_* | 4 |
| enso_* | 11 |
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

### Read tools that save calls

| Tool | Use it for |
|------|------------|
| `rpc_multicall` | Many contract reads at one block in one request, decoded, in input order. Also a time series with `block_range`. Replaces repeated `contract_call` / `rpc_call` |
| `contract_inspect` | Everything about an address in one call: account kind, proxy and implementation, verified name, creator, token details, Safe owners, and the function list. Use it first for an unknown address |
| `contract_call` | One read by function name, using the contract's ABI |
| `rpc_balance` | Native balance in wei and ether at a block or time |
| `rpc_tx` | A transaction by hash |
| `rpc_code_summary` | Whether an address has code, its size and code hash, and any EIP-7702 delegate, without the bytecode |
| `rpc_block_at` | The block for a date, Unix timestamp (`@1706700000`) or age (`7d`) |
| `rpc_find_change` | The block where a call result, balance, storage slot, code or nonce changed (binary search) |
| `sig_lookup` | A signature from a 4-byte selector or 32-byte topic |
| `sig_error` | Decoded revert data: `Error(string)`, `Panic(uint256)` or a custom error |
| `safe_info` | Safe owners, threshold, nonce, version, modules, guard and fallback handler, read on chain |
| `safe_queue` | Pending Safe transactions with who has and has not signed |
| `safe_tx` | One Safe transaction, decoded, with MultiSend batches unpacked |
| `safe_history` | Executed Safe transactions, paged |
| `cast_index` | The storage slot of a mapping entry |

The `block` input of `rpc_balance`, `rpc_multicall` and `contract_call` also
accepts a date, `@timestamp` or age. A plain date is 00:00 UTC at the start of
that day; for an end-of-day figure pass the next day's date or a full time. A reverted call's result carries a `revert`
object; its `source` field is `signature_database` when the error name is a
guess from the public signature list.

### Smaller results and error codes

- Read-only tools accept an optional `fields` input: a comma-separated list
  of field paths (`"hash,value"`, `"balance.wei"`), applied to each element
  of an array. Only those fields of the JSON result are returned; a tool
  whose result is plain text returns `invalid_argument`. Tools that write do
  not list it and refuse it before running.
- A failed call's text starts with a stable code, for example
  `Error [not_verified]: ... (hint: ...)`. The codes are `invalid_argument`,
  `not_verified`, `missing_api_key`, `unauthorized`, `rate_limited`,
  `unsupported_chain`, `not_found`, `rpc_error` and `internal`.

## Choosing which tools to offer

The server offers every tool by default (about 590). Some clients load every
tool definition into the model's context, and model providers advise keeping
the number of tools small, so `ETHCLI_MCP_TOOLSETS` limits what is offered. It
takes a comma-separated list:

| Entry | Offers |
|-------|--------|
| `core` | About 20 tools that answer most questions in one call: `contract_inspect`, `contract_call`, `tx_analyze`, `rpc_multicall`, `rpc_call`, `rpc_balance`, `rpc_block_at`, `logs`, `token_info`, `account_txs`, `simulate_call`, `price`, `quote_best`, `sig_error`, `safe_info`, `safe_queue`, `safe_tx` and a few more |
| a group (`curve`, `tenderly`, `rpc`, `account`, ...) | Every tool whose name starts with that group |
| a tool name (`uniswap_pool`) | That tool |
| `all`, or unset | Every tool |

```json
{
  "mcpServers": {
    "ethcli": {
      "command": "ethcli-mcp",
      "env": { "ETHCLI_MCP_TOOLSETS": "core,curve" }
    }
  }
}
```

Clients that load tools on demand (Claude Code's tool search, for example) do
not need this. Entries that match no tool are logged and ignored. Write tools
still need `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1`.

## Environment Variables

| Variable | Required For |
|----------|-------------|
| `ETHCLI_PATH` | Absolute custom ethcli binary path |
| `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1` | Opt in to MCP tools that mutate local/remote state |
| `ETHCLI_MCP_TOOLSETS` | Offer only some tools, e.g. `core,curve` (see above) |
| `ETHERSCAN_API_KEY` | Higher rate limits (optional) |
| `ALCHEMY_API_KEY` | `alchemy_*` tools |
| `TENDERLY_ACCESS_KEY` | `tenderly_*` tools |
| `ONEINCH_API_KEY` | `oneinch_*` tools |
| `ENSO_API_KEY` | `enso_*` tools |
| `DUNE_API_KEY` | `dune_*` tools |
| `MORALIS_API_KEY` | `moralis_*` tools |
| `SOLODIT_API_KEY` | `solodit_*` tools |
| `PYTH_API_KEY` | `pyth_price` (required since the Pyth Core upgrade; or `config_set_pyth`), sent with `pyth_search`/`pyth_feeds` when set |
| `DEFILLAMA_API_KEY` | DefiLlama data in `yields` from 2026-11-13, and Pro endpoints (or `config_set_defillama`) |
| `THEGRAPH_API_KEY` | `uniswap_top_pools`, etc. |

Moralis removed Fantom support on 2026-05-29 (the `moralis_*` tools inherit
ethcli's Fantom guard). Moralis deleted its Discovery, Volume and Market Data
APIs and several ERC20 helper endpoints on 2026-06-04, and
`GET /erc20/{address}/holders/historical` on 2026-07-31, so these MCP tools
were removed: `moralis_market_*`, `moralis_volume_*`, `moralis_discovery_*`
(except `moralis_discovery_token_analytics` and `moralis_discovery_token_score`),
`moralis_token_stats`, `moralis_token_exchange_*`, `moralis_token_bonding_status`,
`moralis_token_pairs_stats`, `moralis_token_by_symbols`,
`moralis_token_pair_snipers` and `moralis_token_holders_historical`. Use
`moralis_token_search` (Token Search), `moralis_token_trending`,
`moralis_analytics_batch` / `moralis_analytics_timeseries` /
`moralis_discovery_token_analytics` (Token Analytics),
`moralis_token_pair_stats` and `moralis_token_holders_summary` instead.

Tenderly's public API has no alert-destination or contract PATCH endpoints, so
`tenderly_alerts_add_destination`, `tenderly_alerts_remove_destination` and
`tenderly_contracts_update` were removed (use `tenderly_contracts_remove_tag` /
`tenderly_contracts_bulk_tag` and the Tenderly dashboard). Tenderly has renamed
Virtual TestNets to Virtual Environments (`/api/public/v1/.../environments`);
the `tenderly_vnets_*` tools still use the live `/vnets` routes.

Alchemy retired several NFT API endpoints on 2026-09-30, so these tools were
removed: `alchemy_nft_collections_for_owner` (use
`alchemy_nft_contracts_for_owner`), `alchemy_nft_collection_metadata` and
`alchemy_nft_search_contract_metadata` (use `alchemy_nft_contract_metadata`),
`alchemy_nft_spam_contracts` (use `alchemy_nft_is_spam`), and
`alchemy_nft_summarize_attributes`, `alchemy_nft_compute_rarity`,
`alchemy_nft_invalidate_contract`, `alchemy_nft_is_airdrop`,
`alchemy_nft_sales` (no replacement). `alchemy_nft_is_holder` remains and is
now backed by `getNFTsForOwner`. The Alchemy Transaction Simulation API was
retired on the same date, so `alchemy_sim_asset_changes` and
`alchemy_sim_execution` were removed; use `simulate_call` (Tenderly or
`debug_traceCall` backends) or `alchemy_debug_trace_call` instead.

Dune shut down the Sim platform on 2026-08-01, so all `dsim_*` tools and
`config_set_dune_sim` were removed, and `portfolio` no longer accepts
`source: "dsim"`. The `dune_*` (Dune Analytics) tools are unaffected. See
<https://github.com/yldfi/yldfi-rs/issues/64>.

## Unverified Contract Analysis

The `contract_analyze` MCP tool exposes ethcli's native unverified-contract workflow. Set `follow_proxy`, `lookup`, `dispatcher`, and `checks` to inspect proxy implementations, resolve selectors, map selector handlers, and flag side-effecting handler bodies that appear to lack caller gates or calldata hash validation. `contract_selectors` also supports `follow_proxy` when you want implementation selectors rather than proxy wrapper selectors.

Equivalent CLI:

```bash
ethcli contract analyze 0x... --follow-proxy --lookup --dispatcher --checks --format json
```

`contract_fingerprint` reads compiler facts from runtime code, `contract_decompile`
runs the optional external evmdec decompiler (cross-check first, output labelled
heuristic; returns install steps and ethcli's facts when evmdec is missing) and
`contract_match` compares a local forge or solc artifact with deployed code
(`exact`, `partial` = equal ignoring metadata and not a verification, or
`mismatch` localised to a function). All three are read-only. `contract_match` reads only `.json` files (up to 50 MB) under `ETHCLI_MCP_ARTIFACT_ROOT`. The server must be started with that variable set to a project or artifacts directory; without it, or set to `/` or the home directory, `contract_match` refuses. See
`skills/bytecode-to-source/SKILL.md` for the rebuild procedure.

For endpoint diagnostics, `endpoints_health` returns JSON and defaults to `probes: 1` so MCP calls stay bounded. Increase `probes` when you need a deeper endpoint check.

## Architecture

```
┌─────────────────┐     JSON-RPC      ┌─────────────────┐    subprocess    ┌─────────────┐
│  Claude/LLM     │ ◄──── STDIO ────► │   ethcli-mcp    │ ◄─────────────► │   ethcli    │
└─────────────────┘                   └─────────────────┘                  └─────────────┘
```

## Security

- **Read-only by default** - Local/remote mutating tools such as `config_set_*`, `address_add`, `blacklist_add`, `endpoints_add`, `chainlist_add`, `simulate_share`, Dune write APIs, Tenderly VNet/admin mutations, `simulate_call` with `save`, Alchemy NFT `report_spam`/`refresh_metadata`, Moralis NFT `sync_collection`/`resync_metadata`/`resync_traits`, and CowSwap create/cancel order(s) are blocked unless `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1` is set. Every tool carries MCP annotations: read tools have `readOnlyHint: true`; write tools have `readOnlyHint: false`, `destructiveHint`/`idempotentHint`, and say in their description that they need `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1`.
- **No raw config disclosure** - `config_show` returns safe config status only. It does not return config file contents because they may contain API keys or private RPC URLs.
- **No local file input** - `@file` values (`simulate_call.access_list`, `simulate_bundle.txs`) are rejected; pass inline JSON.
- **No arbitrary file output** - MCP rejects file-output paths for tools such as `contract_abi`, `contract_source`, and `address_export`; omit `output` to receive data in the tool response.
- **No secret-bearing subprocess args** - MCP rejects direct credential/header flags such as `--show-secrets`, `--tenderly-key`, `--alchemy-key`, `--etherscan-api-key`/`--etherscan-key`, `--rpc-header(s)`, and `--fork-header(s)`. Configure credentials via environment variables or ethcli config instead. `config_set_*` passes the key to ethcli on stdin, not in argv.
- **Success-output redaction** - Credential-bearing URLs and secret-looking header/config lines are redacted before returning MCP tool responses. JSON output is redacted inside its string values, so it stays valid JSON, including when a string quotes a URL from an upstream error.
- **Quote tools** - `quote_best`, `quote_compare` and `quote_from` take `sender` only as the user's own wallet. Without it, CowSwap, LI.FI and Enso are priced for a placeholder address, marked `placeholder_sender`, and return no transaction data. `aggregation.outliers` lists sources under half the median, which are left out of the summary figures.
- **Process-group cleanup** - Timed-out subprocesses are killed as a process group so child tools such as `anvil` do not remain running.
- **Rate limiting** - Max 10 concurrent subprocesses; a call waits up to 30 seconds for a free one before failing
- **Timeouts** - 10 seconds for fast local commands, 30 seconds for network commands, 330 seconds for `dune_sql`/`dune_query` (ethcli waits up to 300 seconds for the execution), 600 seconds for `endpoints_optimize` (results are saved after each endpoint)
- **Input validation** - Argument length limits, null byte detection. Tool values are never read as flags: positional values starting with `-` are rejected (or passed after `--` where they are legitimate, such as negative ABI arguments and SQL comments), and option values starting with `-` are passed as `--flag=value`
- **Closed stdin** - The ethcli subprocess gets no stdin (the MCP transport), except the secret piped to `config_set_*`
- **Bounded output** - At most 1 MB of stdout is kept; the rest is drained and the response notes the truncation
- **Error sanitization** - API keys filtered from error messages

## Development

See [CONTRIBUTING.md](https://github.com/yldfi/yldfi-rs/blob/main/CONTRIBUTING.md) for guidelines.

```bash
cargo build -p ethcli-mcp
cargo test -p ethcli-mcp --test integration
```

## License

MIT
