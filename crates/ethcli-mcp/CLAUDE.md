# ethcli-mcp

MCP (Model Context Protocol) server that wraps ethcli commands as tools for AI assistants.

## Build Commands

```bash
# Development build
cargo build -p ethcli-mcp

# Release build
cargo build -p ethcli-mcp --release

# Run tests (requires ethcli binary)
ETHCLI_PATH=/path/to/ethcli cargo test -p ethcli-mcp

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
async fn my_tool(&self, Parameters(input): Parameters<MyToolInput>) -> String {
    tools::my_tool(&input.param, Some(&input.chain)).await.to_response()
}
```

Note: The `ToResponse` trait (from `tools.rs`) converts `Result<String, E>` to `String`.

### Error Handling

- Use `ToolError` enum for tool-level errors
- Use `ExecutionError` for subprocess errors
- Use `ValidationError` for input validation
- Always sanitize error messages before returning

### Security Controls

- **Rate limiting**: Max 10 concurrent subprocesses (semaphore)
- **Timeouts**: 10s for fast commands (cast, ens), 30s for network commands
- **Input validation**: Argument length limits, null byte detection
- **Error sanitization**: Filter API keys, paths, tokens from error messages

### Testing

```bash
# Unit tests (no external dependencies)
cargo test -p ethcli-mcp --lib

# Integration tests (requires ethcli binary)
ETHCLI_PATH=/path/to/ethcli cargo test -p ethcli-mcp --test integration

# Network tests (requires API keys)
cargo test -p ethcli-mcp --test integration -- --include-ignored
```

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ETHCLI_PATH` | Path to ethcli binary (optional) |
| `RUST_LOG` | Logging level (debug, info, warn, error) |

## Tool Categories

| Prefix | Count | Description |
|--------|-------|-------------|
| `cast_*` | 14 | Unit conversions, hashing |
| `rpc_*` | 9 | Direct RPC calls |
| `account_*` | 8 | Balance, transactions |
| `lifi_*` | 12 | Cross-chain aggregator |
| `cowswap_*` | 8 | MEV-protected swaps |
| ... | ... | See README for full list |

## Important Notes

1. **STDIO only**: Never write to stdout except MCP responses. Use stderr for logging.
2. **Subprocess model**: All ethcli calls spawn a new process. No shared state.
3. **No secrets in logs**: sanitize_error() filters sensitive data.
4. **Binary verification**: Checks ETHCLI_PATH exists and is executable.

## Future Enhancements

These are documented improvements that could be made:

1. **Split types.rs into modules** - Group input types by category (core, defi, data, security, config) for better organization. Currently a single 1274-line file.

2. **Add mock layer for testing** - Inject mock executor to test tool logic without spawning real subprocesses. Would require trait abstraction in executor.rs.

3. **Structured error responses** - Return JSON error objects instead of "Error: ..." strings for better client parsing.
