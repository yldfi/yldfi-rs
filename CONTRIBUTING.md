# Contributing to yldfi-rs

Thank you for considering contributing to yldfi-rs!

## Repository Structure

This is a Cargo workspace containing multiple crates:

| Crate | Description |
|-------|-------------|
| `ethcli` | Main Ethereum CLI tool |
| `ethcli-mcp` | MCP server wrapping ethcli |
| `yldfi-common` | Shared utilities |
| `openoc`, `cowp`, `kybr`, `zrxswap`, `oinch`, `vlra`, `ensof`, `lfi` | DEX aggregator clients |
| `tndrly`, `dllma`, `cgko`, `dnsim`, `dnapi`, `alcmy`, `pythc`, `unswp`, `ykong`, `gplus`, `sldt` | Various API clients |
| `crv`, `mrls` | Curve Finance and Moralis clients |

## Development Setup

```bash
# Clone the repository
git clone https://github.com/yldfi/yldfi-rs.git
cd yldfi-rs

# Build all crates
cargo build --workspace

# Build specific crate
cargo build -p ethcli --release

# Run tests for all crates
cargo test --workspace

# Run tests for specific crate
cargo test -p ethcli
```

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy --workspace` and fix any warnings
- Follow existing patterns in the codebase
- Add doc comments for public items

## Adding a New Crate

1. Create the crate directory under `crates/`
2. Add to workspace members in root `Cargo.toml`
3. Add to `release-please-config.json` if publishing to crates.io
4. Add to `.github/workflows/publish-crate.yml` if auto-publishing

## Adding Features to ethcli

1. **Add the command** in `crates/ethcli/src/cli/mod.rs`:
   ```rust
   MyCommand {
       #[clap(long)]
       param: String,
   }
   ```

2. **Implement the handler** in `crates/ethcli/src/main.rs`:
   ```rust
   Commands::MyCommand { param } => {
       // Implementation
   }
   ```

3. **Add tests** in `crates/ethcli/tests/`

## Adding Tools to ethcli-mcp

See `crates/ethcli-mcp/README.md` for MCP-specific instructions.

1. **Define the input type** in `src/types.rs`
2. **Add the tool handler** in `src/main.rs`
3. **Add integration test** in `tests/integration.rs`

## Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run ignored tests (require network)
cargo test --workspace -- --include-ignored

# Run specific test
cargo test -p ethcli test_name
```

## Pull Request Process

1. Create a feature branch from `main`
2. Make your changes
3. Ensure `cargo fmt` and `cargo clippy` pass
4. Ensure tests pass
5. Update documentation if needed
6. Submit a pull request

## Release Process

This repository uses [release-please](https://github.com/googleapis/release-please) for automated releases:

1. Commits to `main` are analyzed for conventional commit messages
2. Release PRs are automatically created
3. Merging a release PR triggers crate publishing

## Environment Variables

Many crates require API keys for full functionality:

| Variable | Used By |
|----------|---------|
| `ETHERSCAN_API_KEY` | ethcli (Etherscan API) |
| `ALCHEMY_API_KEY` | ethcli (RPC provider) |
| `TENDERLY_API_KEY` | tndrly |
| `DUNE_API_KEY` | dnapi |
| `COINGECKO_API_KEY` | cgko |

## Questions?

Open an issue on GitHub or reach out to the maintainers.
