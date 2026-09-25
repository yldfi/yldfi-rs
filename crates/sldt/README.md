# sldt

Unofficial Rust client for the [Solodit](https://solodit.cyfrin.io) smart contract vulnerability database.

## Features

- Search vulnerability findings by keywords
- Filter by impact level (HIGH, MEDIUM, LOW, GAS)
- Filter by audit firm, tags, protocol, protocol category, forked protocol, language
- Filter by finder handle / finder count and report date (`30`/`60`/`90` days, `after` a date)
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
    let client = Client::new("sk_your_api_key_here")?;

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
    let client = Client::new("sk_your_api_key")?;

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
    let client = Client::new("sk_your_api_key")?;

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
    let client = Client::new("sk_your_api_key")?;

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
        .forked("Compound")
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

### Fetching a Single Finding

The official API only has `POST /findings` (no get-by-id endpoint).
`Client::get_by_slug` is therefore a **best-effort** lookup: it searches using the
slug text as keywords (then with dashes replaced by spaces) and returns the finding
whose `slug` or `id` matches exactly. It accepts a slug, an ID, or a full
`https://solodit.cyfrin.io/issues/<slug>` URL, costs 1-2 requests, and may return
`Error::NotFound` for findings whose text does not match their slug.

## Rate Limiting

The API has a rate limit of **20 requests per 60-second window**. Rate limit info is
returned both in the body (`rateLimit`) and in `X-RateLimit-Limit` /
`X-RateLimit-Remaining` / `X-RateLimit-Reset` headers:

```rust
let results = client.search("test").await?;
println!("Remaining: {}", results.rate_limit.remaining); // body, falls back to headers
println!("Limit: {}", results.rate_limit.limit);
println!("Resets at: {}", results.rate_limit.reset); // Unix timestamp
println!("Headers: {:?}", results.rate_limit_headers); // raw X-RateLimit-* values
```

## Error Handling

Server error messages (`{"message": ...}`) are preserved. The API key is never
included in error messages.

```rust
use sldt::{Client, Error};

async fn example() -> sldt::Result<()> {
    let client = Client::new("sk_your_api_key")?;

    match client.search("test").await {
        Ok(results) => println!("Found {} results", results.total),
        // 401: "Missing API key" or "Invalid API key". Keys are managed at
        // solodit.cyfrin.io (Profile > API Keys); regenerating invalidates the old key.
        Err(e) if e.is_invalid_api_key() => println!("Key rejected: {e}"),
        Err(Error::Unauthorized { message }) => println!("Unauthorized: {message}"),
        // 429: includes X-RateLimit-Reset when the server sends it
        Err(Error::RateLimited { reset, .. }) => println!("Rate limited, resets at {reset:?}"),
        // 400 and others: server message in `message`
        Err(Error::Api { status, message }) => println!("{status}: {message}"),
        Err(e) => println!("Error: {e}"),
    }
    Ok(())
}
```

## Response Tolerance and Known Upstream Quirks

Response parsing is deliberately lenient so that one odd field cannot fail a whole
page: IDs are accepted as JSON strings or numbers (the API serializes BigInts),
numeric fields accept numeric strings, malformed nested objects become `None`,
and array entries that fail to parse are skipped.

Known upstream quirks ([solodit/solodit_content#153](https://github.com/solodit/solodit_content/issues/153)):

- `report_date` is sometimes returned as `{}` (mapped to `None`)
- `protocols_protocol.protocols_protocolcategoryscore` is currently always `[]`
- The tags field is `issues_issuetagscore` (some docs call it `issues_issuetags`)

## Testing

`cargo test -p sldt` runs offline tests against a wiremock server using a fixture
built from the spec's response schema. A live smoke test is available:

```bash
SOLODIT_API_KEY=sk_... cargo test -p sldt --test mock_api -- --ignored
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
