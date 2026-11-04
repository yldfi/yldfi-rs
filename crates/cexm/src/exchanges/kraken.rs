//! Kraken spot: <https://docs.kraken.com/api/docs/rest-api/get-ticker-information>
//!
//! Responses are `{"error": [...], "result": {...}}`, usually with HTTP 200
//! even on error. Results are keyed by Kraken's own pair name (e.g.
//! `XXBTZUSD`), and Kraken uses `XBT` for bitcoin and `XDG` for dogecoin.

use super::{change_24h, require_spot, unsupported_timeframe, with_cost};
use crate::client::Ctx;
use crate::error::{Error, Result};
use crate::http::{array, cell, field, get_json, parse_error, require, scalar};
use crate::types::{
    as_f64, Candle, Level, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe, Trade,
};
use serde_json::Value;

const MAX_BOOK: u32 = 500;
const MAX_TRADES: u32 = 1000;

/// Kraken's code for a unified asset (`BTC` → `XBT`)
fn kraken_asset(asset: &str) -> &str {
    match asset {
        "BTC" => "XBT",
        "DOGE" => "XDG",
        other => other,
    }
}

/// Unified asset for a Kraken code (`XBT` → `BTC`)
fn unified_asset(asset: &str) -> String {
    match asset {
        "XBT" => "BTC".into(),
        "XDG" => "DOGE".into(),
        other => other.to_ascii_uppercase(),
    }
}

fn pair(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<String> {
    require_spot(ctx.exchange, symbol)?;
    Ok(format!(
        "{}{}",
        kraken_asset(&symbol.base),
        kraken_asset(&symbol.quote)
    ))
}

/// GET and return `result`, turning a non-empty `error` list into an error
async fn result(ctx: &Ctx<'_>, path: &str, query: &[(&str, String)]) -> Result<Value> {
    let mut body = get_json(ctx.http, ctx.exchange, &ctx.url(path), query).await?;
    if let Some(message) = body
        .get("error")
        .and_then(Value::as_array)
        .and_then(|e| e.first())
        .and_then(scalar)
    {
        let (code, message) = match message.split_once(':') {
            Some((code, rest)) => (code.to_string(), rest.to_string()),
            None => (String::new(), message),
        };
        return Err(Error::Api {
            exchange: ctx.exchange,
            code,
            message,
        });
    }
    Ok(body["result"].take())
}

/// The single pair entry in a result keyed by pair name (skipping `last`)
fn only_pair<'a>(ctx: &Ctx<'_>, result: &'a Value) -> Result<&'a Value> {
    result
        .as_object()
        .and_then(|o| o.iter().find(|(k, _)| k.as_str() != "last"))
        .map(|(_, v)| v)
        .ok_or_else(|| parse_error(ctx.exchange, "result has no pair entry"))
}

/// Element `i` of the array at `obj[key]`
fn nth(obj: &Value, key: &str, i: usize) -> Option<String> {
    obj.get(key)?.get(i).and_then(scalar)
}

/// Kraken times are seconds, possibly fractional
fn seconds_to_millis(v: &Value) -> Option<i64> {
    let secs = match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => as_f64(s),
        _ => None,
    }?;
    Some((secs * 1000.0).round() as i64)
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let pair = pair(ctx, symbol)?;
    let res = result(ctx, "/0/public/Ticker", &[("pair", pair)]).await?;
    let t = only_pair(ctx, &res)?;
    let last = nth(t, "c", 0);
    // The ticker's `o` is today's 00:00 UTC open, not the price 24h ago, so
    // the change comes from the 5-minute candle covering 24 hours ago
    let recent = candles(ctx, symbol, Timeframe::M5, 0).await?;
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        change_pct: change_24h(&recent, last.as_deref()),
        last,
        bid: nth(t, "b", 0),
        ask: nth(t, "a", 0),
        // Index 1 is the rolling 24h value (index 0 is "today")
        high: nth(t, "h", 1),
        low: nth(t, "l", 1),
        base_volume: nth(t, "v", 1),
        timestamp: None,
    })
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let pair = pair(ctx, symbol)?;
    let res = result(
        ctx,
        "/0/public/Depth",
        &[("pair", pair), ("count", limit.min(MAX_BOOK).to_string())],
    )
    .await?;
    let book = only_pair(ctx, &res)?;
    // [[price, volume, timestamp], ...]
    let side = |key| -> Result<Vec<Level>> {
        array(ctx.exchange, book, key)?
            .iter()
            .map(|row| {
                Ok(Level {
                    price: cell(ctx.exchange, row, 0)?,
                    amount: cell(ctx.exchange, row, 1)?,
                })
            })
            .collect()
    };
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        bids: side("bids")?,
        asks: side("asks")?,
        timestamp: None,
    })
}

/// Candle interval in minutes
fn interval(timeframe: Timeframe) -> Option<u32> {
    Some(match timeframe {
        Timeframe::M1 => 1,
        Timeframe::M5 => 5,
        Timeframe::M15 => 15,
        Timeframe::M30 => 30,
        Timeframe::H1 => 60,
        Timeframe::H4 => 240,
        Timeframe::D1 => 1_440,
        Timeframe::W1 => 10_080,
        _ => return None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    _limit: u32,
) -> Result<Vec<Candle>> {
    // Kraken returns up to 720 of the latest candles; the client trims them
    let pair = pair(ctx, symbol)?;
    let minutes =
        interval(timeframe).ok_or_else(|| unsupported_timeframe(ctx.exchange, timeframe))?;
    let res = result(
        ctx,
        "/0/public/OHLC",
        &[("pair", pair), ("interval", minutes.to_string())],
    )
    .await?;
    let ex = ctx.exchange;
    // [[time (s), open, high, low, close, vwap, volume, count], ...]
    only_pair(ctx, &res)?
        .as_array()
        .ok_or_else(|| parse_error(ex, "candles are not an array"))?
        .iter()
        .map(|row| {
            Ok(Candle {
                timestamp: row
                    .get(0)
                    .and_then(seconds_to_millis)
                    .ok_or_else(|| parse_error(ex, "candle without a time"))?,
                open: cell(ex, row, 1)?,
                high: cell(ex, row, 2)?,
                low: cell(ex, row, 3)?,
                close: cell(ex, row, 4)?,
                volume: cell(ex, row, 6)?,
            })
        })
        .collect()
}

pub(super) async fn trades(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<Vec<Trade>> {
    let pair = pair(ctx, symbol)?;
    let res = result(
        ctx,
        "/0/public/Trades",
        &[("pair", pair), ("count", limit.min(MAX_TRADES).to_string())],
    )
    .await?;
    let ex = ctx.exchange;
    // [[price, volume, time (s), "b"/"s", "m"/"l", misc, trade_id], ...]
    only_pair(ctx, &res)?
        .as_array()
        .ok_or_else(|| parse_error(ex, "trades are not an array"))?
        .iter()
        .map(|row| {
            Ok(with_cost(Trade {
                id: row.get(6).and_then(scalar),
                timestamp: row
                    .get(2)
                    .and_then(seconds_to_millis)
                    .ok_or_else(|| parse_error(ex, "trade without a time"))?,
                side: match row.get(3).and_then(Value::as_str) {
                    Some("b") => Side::Buy,
                    _ => Side::Sell,
                },
                price: cell(ex, row, 0)?,
                amount: cell(ex, row, 1)?,
                cost: None,
            }))
        })
        .collect()
}

pub(super) async fn markets(ctx: &Ctx<'_>) -> Result<Vec<Market>> {
    let res = result(ctx, "/0/public/AssetPairs", &[]).await?;
    let pairs = res
        .as_object()
        .ok_or_else(|| parse_error(ctx.exchange, "asset pairs are not an object"))?;
    let mut markets = Vec::with_capacity(pairs.len());
    for p in pairs.values() {
        // `wsname` is "XBT/USD"; dark-pool pairs (".d") have none
        let Some(wsname) = field(p, "wsname") else {
            continue;
        };
        let Some((base, quote)) = wsname.split_once('/') else {
            continue;
        };
        let (base, quote) = (unified_asset(base), unified_asset(quote));
        markets.push(Market {
            symbol: Symbol::spot(&base, &quote).to_string(),
            id: require(ctx.exchange, p, "altname")?,
            base,
            quote,
            market_type: MarketType::Spot,
            active: field(p, "status").as_deref() == Some("online"),
        });
    }
    Ok(markets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn asset_codes_round_trip() {
        assert_eq!(kraken_asset("BTC"), "XBT");
        assert_eq!(kraken_asset("ETH"), "ETH");
        assert_eq!(unified_asset("XBT"), "BTC");
        assert_eq!(unified_asset("XDG"), "DOGE");
    }

    #[test]
    fn fractional_seconds_become_millis() {
        assert_eq!(
            seconds_to_millis(&json!(1793788203.1234)),
            Some(1_793_788_203_123)
        );
        assert_eq!(
            seconds_to_millis(&json!(1793788203)),
            Some(1_793_788_203_000)
        );
    }
}
