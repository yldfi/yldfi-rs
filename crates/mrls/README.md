<p align="center">
  <img src="https://raw.githubusercontent.com/yldfi/yldfi-rs/main/logo-128.png" alt="yld_fi" width="128" height="128">
</p>

<h1 align="center">mrls</h1>

<p align="center">
  Unofficial Rust client for the <a href="https://docs.moralis.io/">Moralis Web3</a> API
</p>

<p align="center">
  <a href="https://crates.io/crates/mrls"><img src="https://img.shields.io/crates/v/mrls.svg" alt="crates.io"></a>
  <a href="https://github.com/yldfi/yldfi-rs/blob/main/crates/mrls/LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="MIT License"></a>
</p>

## Features

- **Wallet API** - Native balances, token balances, transactions, approvals, net worth, profitability
- **Token API** - Metadata, prices, transfers, swaps, pairs, holders, search, trending
- **NFT API** - NFT metadata, transfers, owners, trades, floor prices, collections
- **DeFi API** - Positions and protocol summaries (legacy pair price/reserves/address helpers are deprecated)
- **Block API** - Block data, timestamps, date-to-block lookups
- **Transaction API** - Transaction details, decoded calls, internal transactions
- **Resolve API** - ENS, Unstoppable Domains, domain resolution
- **Token Analytics API** - Batch and timeseries analytics (`client.analytics()`), per-token analytics and scores (`client.discovery()`)
- **Entities API** - Wallet/protocol/exchange labels and categories

## Removed Moralis Endpoints

Moralis removed the following endpoints (they now return `404`), and the
corresponding `mrls` wrappers have been deleted:

| Removed on | Endpoints | Replacement |
|------------|-----------|-------------|
| 2026-06-04 | Discovery API: `/discovery/tokens/*`, `POST /discovery/tokens`, `/discovery/token` | Token Search `GET /tokens/search` (`TokenApi::search`), `GET /tokens/trending` (`TokenApi::get_trending`) |
| 2026-06-04 | Volume API: `/volume/chains`, `/volume/categories`, `/volume/timeseries[/{category}]` | Token Analytics `POST /tokens/analytics[/timeseries]` (`AnalyticsApi`) |
| 2026-06-04 | Market Data API: `/market-data/*` | Token Search / Trending, Token Analytics |
| 2026-06-04 | `/erc20/{address}/stats`, `/erc20/{address}/pairs/stats` | Token Analytics `GET /tokens/{address}/analytics`, `GET /pairs/{address}/stats` |
| 2026-06-04 | `/erc20/exchange/{exchange}/new\|bonding\|graduated`, `/erc20/{address}/bondingStatus`, `/pairs/{address}/snipers` | None documented |
| 2026-06-04 | `/erc20/metadata/symbols` | Token Search `GET /tokens/search` |
| 2026-07-31 | `/erc20/{address}/holders/historical` | `GET /erc20/{address}/holders` (`TokenApi::get_holders_summary`), `GET /erc20/{token}/owners` (`TokenApi::get_holders`) |

`GET /tokens/{address}/analytics` and `GET /tokens/{address}/score` remain
available via `client.discovery()`; token scores are EVM-only.

Moralis removed Fantom support on 2026-05-29.

### Deprecated (still routed, not in the OpenAPI spec)

These methods still work but the endpoints were dropped from the Moralis v2.2
OpenAPI spec, so they are marked `#[deprecated]`:

- `DefiApi::get_pair_price` (`GET /{token0}/{token1}/price`)
- `DefiApi::get_pair_reserves` (`GET /{pair}/reserves`)
- `DefiApi::get_pair_address` (`GET /{token0}/{token1}/pairAddress`)
- `UtilsApi::get_contract_events` (`POST /{address}/events`)
- `UtilsApi::get_contract_logs` (`GET /{address}/logs`)
- `TransactionApi::get_wallet_transactions` duplicates `WalletApi::get_transactions` (`GET /{address}`)

## Installation

```toml
[dependencies]
mrls = "0.1"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use mrls::Client;

#[tokio::main]
async fn main() -> Result<(), mrls::Error> {
    // Create client from MORALIS_API_KEY env var
    let client = Client::from_env()?;

    // Get native balance
    let balance = client.wallet().get_native_balance(
        "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
        Some("eth"),
    ).await?;
    println!("Balance: {} wei", balance.balance);

    // Get token price
    let price = client.token().get_price(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        Some("eth"),
    ).await?;
    println!("WETH Price: ${:?}", price.usd_price);

    Ok(())
}
```

## Environment Variables

- `MORALIS_API_KEY` - Your Moralis API key (required)

## Terms of Service

This is an **unofficial** client. By using this library, you agree to comply with [Moralis Terms of Service](https://moralis.io/terms/).

## Disclaimer

This crate is not affiliated with or endorsed by Moralis.

## License

MIT
