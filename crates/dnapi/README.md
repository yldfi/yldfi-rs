<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">dune-api</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://docs.dune.com/api-reference">Dune Analytics</a> API
</p>

<p align="center">
  <a href="https://crates.io/crates/dnapi"><img src="https://img.shields.io/crates/v/dune-api.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/dune-api/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Queries** - Create, read, update, archive/unarchive queries
- **Executions** - Execute queries, run raw SQL, get results
- **Tables** - Manage custom tables, upload/insert data
- **Materialized Views** - Create and manage materialized views
- **Pipelines** - Set up data pipelines
- **Usage** - Get API usage statistics

## Installation

```toml
[dependencies]
dnapi = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use dnapi::Client;

#[tokio::main]
async fn main() -> Result<(), dnapi::Error> {
    let client = Client::new("your-api-key")?;

    // Execute a query and wait for results
    let result = client.executions().run_query(1234, None).await?;
    for row in result.result.unwrap().rows {
        println!("{:?}", row);
    }

    // Execute raw SQL
    let result = client.executions().run_sql("SELECT 1 as value", None).await?;
    println!("{:?}", result.result);

    // Get cached results for a query
    let result = client.executions().query_results(1234).await?;
    println!("{:?}", result.result);

    Ok(())
}
```

## Environment Variables

- `DUNE_API_KEY` - Your Dune API key (required)

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [Dune's Terms of Service](https://dune.com/terms).

## Disclaimer

This crate is not affiliated with or endorsed by Dune Analytics.

## License

MIT
