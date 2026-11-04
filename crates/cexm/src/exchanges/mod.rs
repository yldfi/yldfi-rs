//! One module per exchange, each mapping its REST API onto the shared types.
//!
//! Every module exposes the same five functions (`ticker`, `order_book`,
//! `candles`, `trades`, `markets`); the functions here route a call to the
//! right one.

mod binance;
mod bitget;
mod bybit;
mod coinbase;
mod hyperliquid;
mod kraken;
mod lighter;
mod okx;

use crate::client::Ctx;
use crate::error::{Error, Result};
use crate::types::{Candle, Exchange, Market, OrderBook, Symbol, Ticker, Timeframe, Trade};

macro_rules! route {
    ($ctx:expr, $func:ident ( $($arg:expr),* )) => {
        match $ctx.exchange {
            Exchange::Binance => binance::$func($ctx, $($arg),*).await,
            Exchange::Bitget => bitget::$func($ctx, $($arg),*).await,
            Exchange::Okx => okx::$func($ctx, $($arg),*).await,
            Exchange::Hyperliquid => hyperliquid::$func($ctx, $($arg),*).await,
            Exchange::Lighter => lighter::$func($ctx, $($arg),*).await,
            Exchange::Bybit => bybit::$func($ctx, $($arg),*).await,
            Exchange::Coinbase => coinbase::$func($ctx, $($arg),*).await,
            Exchange::Kraken => kraken::$func($ctx, $($arg),*).await,
        }
    };
}

pub(crate) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    route!(ctx, ticker(symbol))
}

pub(crate) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    route!(ctx, order_book(symbol, limit))
}

pub(crate) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    limit: u32,
) -> Result<Vec<Candle>> {
    route!(ctx, candles(symbol, timeframe, limit))
}

pub(crate) async fn trades(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<Vec<Trade>> {
    route!(ctx, trades(symbol, limit))
}

pub(crate) async fn markets(ctx: &Ctx<'_>) -> Result<Vec<Market>> {
    route!(ctx, markets())
}

/// Reject a swap symbol on a spot-only venue
pub(crate) fn require_spot(exchange: Exchange, symbol: &Symbol) -> Result<()> {
    match symbol.settle {
        None => Ok(()),
        Some(_) => Err(Error::InvalidSymbol {
            exchange,
            symbol: symbol.to_string(),
            reason: format!("{exchange} support covers spot markets only; use BASE/QUOTE"),
        }),
    }
}

/// Error for a timeframe the venue has no candles for
pub(crate) fn unsupported_timeframe(exchange: Exchange, timeframe: Timeframe) -> Error {
    Error::UnsupportedTimeframe {
        exchange,
        timeframe: timeframe.to_string(),
    }
}

/// `trade` with the cost filled in as `price * amount`
pub(crate) fn with_cost(mut trade: Trade) -> Trade {
    if trade.cost.is_none() {
        trade.cost = crate::types::mul_decimal(&trade.price, &trade.amount);
    }
    trade
}

/// Milliseconds in a day
pub(crate) const DAY_MILLIS: i64 = 86_400_000;

/// The open of the candle covering the moment 24 hours ago, for venues whose
/// ticker has no trustworthy 24h open. Candle order doesn't matter.
pub(crate) fn open_24h_ago(candles: &[crate::types::Candle]) -> Option<String> {
    let at = now_millis() - DAY_MILLIS;
    candles
        .iter()
        .filter(|c| c.timestamp <= at)
        .max_by_key(|c| c.timestamp)
        .map(|c| c.open.clone())
}

/// Percent change since 24 hours ago, from the venue's own 5-minute candles
pub(crate) fn change_24h(candles: &[crate::types::Candle], last: Option<&str>) -> Option<f64> {
    crate::types::pct_change(&open_24h_ago(candles)?, last?)
}

/// Current time in milliseconds since the Unix epoch
pub(crate) fn now_millis() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as i64)
}
