# dune

Rust client for the Dune Analytics API.

## Overview

An unofficial Rust client for the [Dune Analytics API](https://docs.dune.com/api-reference), providing access to queries, executions, tables, and more.

## Features

- **Queries** - Create, read, update, archive/unarchive queries
- **Executions** - Execute queries, run raw SQL, get results
- **Tables** - Manage custom tables, upload/insert data
- **Materialized Views** - Create and manage materialized views
- **Pipelines** - Set up data pipelines
- **Usage** - Get API usage statistics

## Quick Start

```rust
use dune::Client;

#[tokio::main]
async fn main() -> Result<(), dune::Error> {
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

## Installation

```toml
[dependencies]
dune = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Environment Variables

- `DUNE_API_KEY` - Your Dune API key (required)

## License

MIT
