# ethcli-mcp

MCP (Model Context Protocol) server that wraps ethcli commands as tools for AI assistants.

## Build Commands

```bash
# Development build
cargo build -p ethcli-mcp

# Release build
cargo build -p ethcli-mcp --release

# Run tests (requires ethcli binary)
ETHCLI_PATH=/absolute/path/to/ethcli cargo test -p ethcli-mcp

# Run unit tests only (no ethcli required)
cargo test -p ethcli-mcp --lib

# Run with logging
RUST_LOG=debug ./target/debug/ethcli-mcp
```

## Architecture

```
┌─────────────┐     MCP/STDIO     ┌─────────────┐    subprocess    ┌─────────┐
│  AI Client  │ ◄───────────────► │ ethcli-mcp  │ ◄──────────────► │ ethcli  │
└─────────────┘                   └─────────────┘                  └─────────┘
```

The MCP server:
1. Receives JSON-RPC requests via STDIO
2. Validates inputs using JSON Schema
3. Spawns ethcli as a subprocess
4. Returns results as MCP tool responses

By default the server is read-only for operations that mutate local files, remote service state, or submitted orders. Set `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1` only for trusted clients that need those tools.

## Project Structure

```
src/
├── main.rs      # MCP server, tool registration (rmcp macros)
├── executor.rs  # Subprocess execution, rate limiting, validation
├── tools.rs     # Tool implementations (ArgsBuilder wrappers)
└── types.rs     # Input type definitions with JsonSchema
tests/
└── integration.rs  # MCP protocol integration tests
```

## Key Patterns

### Adding a New Tool

1. **Define input type** in `types.rs`:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MyToolInput {
    /// Required parameter
    pub param: String,
    /// Optional with default
    #[serde(default = "default_chain")]
    pub chain: String,
}
```

2. **Implement tool function** in `tools.rs`:
```rust
pub async fn my_tool(param: &str, chain: Option<&str>) -> Result<String, ToolError> {
    ArgsBuilder::new("my-command")
        .arg(param)
        .chain(chain)
        .format_json()
        .execute()
        .await
        .map_err(ToolError::from)
}
```

3. **Register tool** in `main.rs`:
```rust
#[tool(description = "Description of my tool")]
async fn my_tool(&self, Parameters(input): Parameters<MyToolInput>) -> CallToolResult {
    tools::my_tool(&input.param, Some(&input.chain)).await.to_response()
}
```

Note: The `ToResponse` trait (from `tools.rs`) converts `Result<String, E>` to a `CallToolResult`: `Ok` is a (size-capped, see `MAX_RESPONSE_BYTES`) success and `Err` sets `isError: true`. A static test (`crates/ethcli/tests/mcp_tool_paths.rs`) checks every `ArgsBuilder` path/flag against the ethcli clap tree.

When adding a tool, decide whether it is safe in default read-only mode:
- Mutating local or remote state must call `require_write_tools_enabled(...)` in `tools.rs` and be covered by `validate_command_policy(...)` in `executor.rs`.
- File-output arguments should be rejected for MCP tools unless they are constrained to a safe app-owned directory.
- Direct secret-bearing arguments and headers should not be forwarded to subprocess argv; require env/config credentials instead.

### Error Handling

- Use `ToolError` enum for tool-level errors
- Use `ExecutionError` for subprocess errors
- Use `ValidationError` for input validation
- Always sanitize error messages before returning
- Tools that pass `-o json` get ethcli's JSON error line on stderr;
  `sanitize_error` turns it into `Error [<code>]: <message> (hint: <hint>)`
  so the stable code reaches the client

### `fields` on every tool

`fields` is handled once, not per tool. `add_fields_parameter` (main.rs) adds
it at startup to the published input schema of every read-only tool whose
ethcli command has JSON output; write tools, `health` and `TEXT_TOOLS` do not
get it, and a call that passes it anyway is refused before anything runs.
`TEXT_TOOLS` is checked against the ethcli command definitions by
`crates/ethcli/tests/mcp_tool_paths.rs`, which prints the expected list when
a tool changes. The hand-written
`call_tool` takes it out of the arguments before they are parsed into the
input struct (so `deny_unknown_fields` structs are fine) and sets the
`executor::FIELDS` task-local for the call. `ArgsBuilder::execute` then adds
`--fields=<spec>` (before any `--`) to every ethcli run the tool makes, and
ethcli does the projection. Do not add a `fields` field to an input struct.

### Security Controls

- **Rate limiting**: Max 10 concurrent subprocesses (semaphore). A call waits up to 30s (`SLOT_WAIT`) for a free slot, then fails with "Too many concurrent requests"
- **Timeouts**: 10s for local commands (cast, config, address, most blacklist/endpoints), 30s for network commands, 330s for Dune executions, 600s for `endpoints optimize` (it saves after each endpoint, so a kill keeps finished results)
- **Input validation**: Argument length limits, null byte detection; `ArgsBuilder` rejects positional/subcommand values starting with `-` and passes dash-led option values as `--flag=value`. Use `.end_of_options()` before positionals that may legitimately start with `-`
- **Stdin**: closed for every subprocess (stdin is the MCP transport); `ArgsBuilder::stdin()` pipes a secret instead of argv (`config_set_*`)
- **Read-only default**: mutating tools require `ETHCLI_MCP_ENABLE_WRITE_TOOLS=1`. Every `#[tool]` sets `annotations(...)`: `read_only_hint = true` for read tools; write tools set `read_only_hint = false`, `destructive_hint`, `idempotent_hint` and say "Requires ETHCLI_MCP_ENABLE_WRITE_TOOLS=1." in the description (`annotation_tests` in main.rs enforces this)
- **Config safety**: `config_show` returns safe status only, not raw config contents
- **File-output policy**: `contract_abi`, `contract_source`, and `address_export` reject output paths over MCP
- **Secret argv policy**: reject `--show-secrets`, direct API-key flags, and RPC/fork header flags
- **Success redaction**: redact credential-bearing URLs and secret-looking header/config lines before returning tool responses
- **Process cleanup**: subprocesses run in a process group and are killed on timeout
- **Error sanitization**: Filter API keys, paths, tokens from error messages

### Testing

```bash
# Unit tests (no external dependencies)
cargo test -p ethcli-mcp --lib

# Integration tests (requires ethcli binary)
ETHCLI_PATH=/absolute/path/to/ethcli cargo test -p ethcli-mcp --test integration

# Network tests (requires API keys)
cargo test -p ethcli-mcp --test integration -- --include-ignored
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ETHCLI_PATH` | Absolute path to ethcli binary when it is not next to ethcli-mcp |
| `ETHCLI_MCP_ENABLE_WRITE_TOOLS` | Set to `1` to enable mutating MCP tools for trusted clients |
| `ETHCLI_MCP_TOOLSETS` | Comma-separated `core`, group prefixes or tool names to offer; unset or `all` offers every tool. `CORE_TOOLS` in `main.rs` lists the core set |
| `RUST_LOG` | Logging level (debug, info, warn, error) |

## Tool Categories

| Prefix | Count | Description |
|--------|-------|-------------|
| `cast_*` | 15 | Unit conversions, hashing, mapping slots |
| `rpc_*` | 14 | Direct RPC calls, batch reads, block by time |
| `contract_*` | 9 | Address overview, ABI, source, bytecode analysis, security |
| `account_*` | 10 | Balance, transactions, first funder, labels |
| `lifi_*` | 19 | Cross-chain aggregator |
| `cowswap_*` | 16 | MEV-protected swaps |
| `tenderly_*` | 34 | VNets, wallets, contracts, alerts, simulation |
| ... | ... | See README for full list |

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
| `sig_lookup` | A signature from a 4-byte selector or 32-byte topic |
| `sig_error` | Decoded revert data: `Error(string)`, `Panic(uint256)` or a custom error |
| `cast_index` | The storage slot of a mapping entry |

The `block` input of `rpc_balance`, `rpc_multicall` and `contract_call` also
accepts a date, `@timestamp` or age. A plain date is 00:00 UTC at the start of
that day; for an end-of-day figure pass the next day's date or a full time. A reverted call's result carries a `revert`
object; its `source` field is `signature_database` when the error name is a
guess from the public signature list.

## Important Notes

1. **STDIO only**: Never write to stdout except MCP responses. Use stderr for logging.
2. **Subprocess model**: All ethcli calls spawn a new process. No shared state.
3. **Read-only default**: New mutating tools must be gated in both `tools.rs` and `executor.rs`.
4. **No secrets in logs or argv**: sanitize_error() filters sensitive data, and MCP blocks secret-bearing subprocess flags.
5. **Binary verification**: `ETHCLI_PATH` must be absolute and executable; otherwise ethcli must be next to ethcli-mcp.

## Future Enhancements

These are documented improvements that could be made:

1. **Split types.rs into modules** - Group input types by category (core, defi, data, security, config) for better organization. Currently a single 1274-line file.

2. **Add mock layer for testing** - Inject mock executor to test tool logic without spawning real subprocesses. Would require trait abstraction in executor.rs.

3. **Structured error responses** - Errors now carry ethcli's stable code (`Error [code]: ...`); returning them as JSON objects in the tool result is still open.
