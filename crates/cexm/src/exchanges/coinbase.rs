//! Coinbase Exchange spot: <https://docs.cdp.coinbase.com/exchange/reference/exchangerestapi_getproducts>
//!
//! Product ids are `BASE-QUOTE` (e.g. `BTC-USD`). Errors come back as
//! `{"message": "..."}` with a non-2xx status.

use super::{require_spot, unsupported_timeframe, with_cost};
use crate::client::Ctx;
use crate::error::Result;
use crate::http::{array, cell, field, get_json, levels, parse_error, require};
use crate::types::{
    pct_change, Candle, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe, Trade,
};
use serde_json::Value;

const MAX_TRADES: u32 = 1000;

fn product_id(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<String> {
    require_spot(ctx.exchange, symbol)?;
    Ok(format!("{}-{}", symbol.base, symbol.quote))
}

async fn get(ctx: &Ctx<'_>, path: &str, query: &[(&str, String)]) -> Result<Value> {
    get_json(ctx.http, ctx.exchange, &ctx.url(path), query).await
}

/// RFC 3339 time (`2026-11-04T10:30:03.123456Z`) in milliseconds
fn millis(v: Option<String>) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(v.as_deref()?)
        .ok()
        .map(|t| t.timestamp_millis())
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let id = product_id(ctx, symbol)?;
    // `ticker` has the top of book; `stats` has the 24h open/high/low
    let t = get(ctx, &format!("/products/{id}/ticker"), &[]).await?;
    let s = get(ctx, &format!("/products/{id}/stats"), &[]).await?;
    let last = field(&t, "price");
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        change_pct: match (field(&s, "open"), &last) {
            (Some(open), Some(last)) => pct_change(&open, last),
            _ => None,
        },
        last,
        bid: field(&t, "bid"),
        ask: field(&t, "ask"),
        high: field(&s, "high"),
        low: field(&s, "low"),
        base_volume: field(&s, "volume"),
        timestamp: millis(field(&t, "time")),
    })
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let id = product_id(ctx, symbol)?;
    // Level 2: aggregated price levels, [[price, size, num_orders], ...]
    let book = get(
        ctx,
        &format!("/products/{id}/book"),
        &[("level", "2".into())],
    )
    .await?;
    let side = |key| -> Result<_> {
        let rows = array(ctx.exchange, &book, key)?;
        levels(ctx.exchange, &rows[..rows.len().min(limit as usize)])
    };
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        bids: side("bids")?,
        asks: side("asks")?,
        timestamp: millis(field(&book, "time")),
    })
}

/// Candle granularity in seconds
fn granularity(timeframe: Timeframe) -> Option<u32> {
    Some(match timeframe {
        Timeframe::M1 => 60,
        Timeframe::M5 => 300,
        Timeframe::M15 => 900,
        Timeframe::H1 => 3_600,
        Timeframe::H6 => 21_600,
        Timeframe::D1 => 86_400,
        _ => return None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    _limit: u32,
) -> Result<Vec<Candle>> {
    // Without start/end Coinbase returns its latest 300 candles; the client
    // trims them to the requested limit
    let id = product_id(ctx, symbol)?;
    let g = granularity(timeframe).ok_or_else(|| unsupported_timeframe(ctx.exchange, timeframe))?;
    let rows = get(
        ctx,
        &format!("/products/{id}/candles"),
        &[("granularity", g.to_string())],
    )
    .await?;
    let ex = ctx.exchange;
    // [[time (s), low, high, open, close, volume], ...], newest first
    rows.as_array()
        .ok_or_else(|| parse_error(ex, "candles are not an array"))?
        .iter()
        .map(|row| {
            Ok(Candle {
                timestamp: row
                    .get(0)
                    .and_then(Value::as_i64)
                    .ok_or_else(|| parse_error(ex, "candle without a time"))?
                    * 1000,
                low: cell(ex, row, 1)?,
                high: cell(ex, row, 2)?,
                open: cell(ex, row, 3)?,
                close: cell(ex, row, 4)?,
                volume: cell(ex, row, 5)?,
            })
        })
        .collect()
}

pub(super) async fn trades(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<Vec<Trade>> {
    let id = product_id(ctx, symbol)?;
    let rows = get(
        ctx,
        &format!("/products/{id}/trades"),
        &[("limit", limit.min(MAX_TRADES).to_string())],
    )
    .await?;
    rows.as_array()
        .ok_or_else(|| parse_error(ctx.exchange, "trades are not an array"))?
        .iter()
        .map(|t| {
            Ok(with_cost(Trade {
                id: field(t, "trade_id"),
                timestamp: millis(field(t, "time"))
                    .ok_or_else(|| parse_error(ctx.exchange, "trade without a valid time"))?,
                // `side` is the maker's side; the taker did the opposite
                side: match field(t, "side").as_deref() {
                    Some("buy") => Side::Sell,
                    _ => Side::Buy,
                },
                price: require(ctx.exchange, t, "price")?,
                amount: require(ctx.exchange, t, "size")?,
                cost: None,
            }))
        })
        .collect()
}

pub(super) async fn markets(ctx: &Ctx<'_>) -> Result<Vec<Market>> {
    let rows = get(ctx, "/products", &[]).await?;
    rows.as_array()
        .ok_or_else(|| parse_error(ctx.exchange, "products are not an array"))?
        .iter()
        .map(|m| {
            let base = require(ctx.exchange, m, "base_currency")?;
            let quote = require(ctx.exchange, m, "quote_currency")?;
            let disabled = m.get("trading_disabled").and_then(Value::as_bool) == Some(true);
            Ok(Market {
                symbol: Symbol::spot(&base, &quote).to_string(),
                id: require(ctx.exchange, m, "id")?,
                base,
                quote,
                market_type: MarketType::Spot,
                active: field(m, "status").as_deref() == Some("online") && !disabled,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rfc3339_with_fraction() {
        assert_eq!(
            millis(Some("2026-11-04T10:30:03.123456Z".into())),
            Some(1_793_788_203_123)
        );
        assert_eq!(millis(Some("not a time".into())), None);
    }
}
