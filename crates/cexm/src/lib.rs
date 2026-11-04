#![warn(missing_docs)]

//! # cexm
//!
//! Public market data from centralised exchanges and perp venues, through one
//! client and one set of types: Binance, Bitget, OKX, Bybit, Coinbase and
//! Kraken (spot), plus Hyperliquid and Lighter (perpetuals).
//!
//! Only public, unauthenticated endpoints are used: tickers, order books,
//! candles, recent trades and market lists. No API keys, no trading.
//!
//! Prices and amounts are the exchange's own decimal strings, so nothing is
//! lost to floating point; [`as_f64`] converts when an approximation is fine.
//!
//! ## Symbols
//!
//! Symbols are unified: `BASE/QUOTE` for spot (`BTC/USDT`, `ETH/USD`) and
//! `BASE/QUOTE:SETTLE` for perpetuals (`BTC/USDC:USDC` on Hyperliquid and
//! Lighter). Each exchange module maps them to its own ids (`BTCUSDT`,
//! `BTC-USDT`, `XBTUSD`, ...).
//!
//! ## Example
//!
//! ```no_run
//! # async fn example() -> cexm::Result<()> {
//! use cexm::{Client, Exchange, Timeframe};
//!
//! let client = Client::new()?;
//! let ticker = client.ticker(Exchange::Okx, "ETH/USDT").await?;
//! println!("ETH on OKX: {:?} ({:?}% 24h)", ticker.last, ticker.change_pct);
//!
//! let candles = client
//!     .candles(Exchange::Hyperliquid, "BTC/USDC:USDC", Timeframe::H1, 24)
//!     .await?;
//! println!("{} hourly candles", candles.len());
//! # Ok(())
//! # }
//! ```
//!
//! ## Regions
//!
//! Some venues refuse some regions. Binance is queried through its
//! market-data host `data-api.binance.vision`, which isn't region-locked;
//! for Bybit and Lighter use [`Config::with_proxy`] if needed.

pub mod client;
pub mod error;
mod exchanges;
mod http;
pub mod types;

pub use client::{default_base_url, Client, Config};
pub use error::{Error, Result};
pub use types::{
    as_f64, mul_decimal, Candle, Exchange, Level, Market, MarketType, OrderBook, Side, Symbol,
    Ticker, Timeframe, Trade,
};
