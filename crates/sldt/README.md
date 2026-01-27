# sldt

Unofficial Rust client for the [Solodit](https://solodit.cyfrin.io) smart contract vulnerability database.

## Features

- Search vulnerability findings by keywords
- Filter by impact level (HIGH, MEDIUM, LOW, GAS)
- Filter by audit firm, tags, protocol, language
- Filter by quality and rarity scores
- Pagination support with rate limit tracking
- Full response metadata and rate limit info

## Installation

```toml
[dependencies]
sldt = "0.1"
```

## Getting an API Key

1. Create an account on [solodit.cyfrin.io](https://solodit.cyfrin.io)
2. Open the dropdown menu in the top right corner
3. Open API Keys modal and generate an API Key

## Usage

### Basic Search

```rust
use sldt::Client;

#[tokio::main]
async fn main() -> sldt::Result<()> {
    let client = Client::new("sk_your_api_key_here");

    let results = client.search("reentrancy").await?;

    println!("Found {} total results", results.total);
    println!("Rate limit: {}/{}", results.rate_limit.remaining, results.rate_limit.limit);

    for finding in results.findings {
        println!("[{}] {}",
            finding.impact_level(),
            finding.title.unwrap_or_default()
        );
    }

    Ok(())
}
```

### Filtered Search

```rust
use sldt::{Client, SearchFilter, Impact};

#[tokio::main]
async fn main() -> sldt::Result<()> {
    let client = Client::new("sk_your_api_key");

    let filter = SearchFilter::new("flash loan")
        .impact(Impact::High)
        .impact(Impact::Medium)
        .firm("Cyfrin")
        .tag("Oracle")
        .page_size(50)
        .sort_by_quality();

    let results = client.search_with_filter(filter).await?;

    for finding in results.findings {
        println!("{}: {}",
            finding.firm().unwrap_or("Unknown"),
            finding.title.unwrap_or_default()
        );
    }

    Ok(())
}
```

### Pagination

```rust
use sldt::{Client, SearchFilter};

#[tokio::main]
async fn main() -> sldt::Result<()> {
    let client = Client::new("sk_your_api_key");

    let mut paginator = client.paginate(
        SearchFilter::new("oracle")
            .impact(sldt::Impact::High)
            .page_size(100)
    );

    while let Some(findings) = paginator.next_page().await? {
        for finding in findings {
            println!("{}", finding.title.unwrap_or_default());
        }

        // Respect rate limits (20 req/60s)
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }

    Ok(())
}
```

### Advanced Filtering

```rust
use sldt::{Client, SearchFilter, Impact, ReportedPeriod};

#[tokio::main]
async fn main() -> sldt::Result<()> {
    let client = Client::new("sk_your_api_key");

    let filter = SearchFilter::new("price manipulation")
        // Impact levels
        .impact(Impact::High)
        .impact(Impact::Medium)

        // Audit firms
        .firm("Cyfrin")
        .firm("Sherlock")

        // Tags
        .tag("Oracle")
        .tag("DeFi")

        // Protocol filters
        .protocol_category("DeFi")
        .language("Solidity")

        // Date filter
        .reported(ReportedPeriod::Days30)

        // Quality filters
        .min_quality(3)
        .min_rarity(2)

        // Sorting
        .sort_by_quality()
        .descending()

        // Pagination
        .page(1)
        .page_size(50);

    let results = client.search_with_filter(filter).await?;
    println!("Found {} results across {} pages", results.total, results.total_pages);

    Ok(())
}
```

## Rate Limiting

The API has a rate limit of **20 requests per 60-second window**. The response includes rate limit information:

```rust
let results = client.search("test").await?;
println!("Remaining: {}", results.rate_limit.remaining);
println!("Limit: {}", results.rate_limit.limit);
println!("Resets at: {}", results.rate_limit.reset); // Unix timestamp
```

## Error Handling

```rust
use sldt::{Client, Error};

async fn example() {
    let client = Client::new("sk_your_api_key");

    match client.search("test").await {
        Ok(results) => println!("Found {} results", results.total),
        Err(Error::Unauthorized) => println!("Invalid API key"),
        Err(Error::RateLimited) => println!("Rate limit exceeded, wait and retry"),
        Err(Error::NotFound(slug)) => println!("Finding not found: {}", slug),
        Err(e) => println!("Error: {}", e),
    }
}
```

## Terms of Service

This is an **unofficial** client. When using Solodit data, you must comply with [Solodit's Terms of Service](https://solodit.cyfrin.io/terms-of-service):

| Permitted | Prohibited |
|-----------|------------|
| Integrating data into products | Raw data redistribution |
| Derivative works | Database mirroring |
| Commercial use | Competing services |

**Attribution required** when publishing research or derivative works based on Solodit data.

## Disclaimer

This crate is not affiliated with or endorsed by Cyfrin or Solodit.

## License

MIT
