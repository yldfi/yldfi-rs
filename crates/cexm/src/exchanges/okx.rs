//! OKX spot, v5 API: <https://www.okx.com/docs-v5/en/#order-book-trading-market-data>
//!
//! Responses are wrapped as `{"code": "0", "msg": "", "data": [...]}`. OKX
//! reports most errors (e.g. an unknown instrument) with HTTP 200 and a
//! non-zero code.

use super::{require_spot, unsupported_timeframe, with_cost};
use crate::client::Ctx;
use crate::error::{Error, Result};
use crate::http::{array, as_i64, cell, field, get_json, int_field, levels, parse_error, require};
use crate::types::{
    pct_change, Candle, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe, Trade,
};
use serde_json::Value;

const MAX_BOOK: u32 = 400;
const MAX_CANDLES: u32 = 300;
const MAX_TRADES: u32 = 500;

fn inst_id(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<String> {
    require_spot(ctx.exchange, symbol)?;
    Ok(format!("{}-{}", symbol.base, symbol.quote))
}

/// GET and unwrap `data`, turning a non-`0` code into [`Error::Api`]
async fn data(ctx: &Ctx<'_>, path: &str, query: &[(&str, String)]) -> Result<Vec<Value>> {
    let mut body = get_json(ctx.http, ctx.exchange, &ctx.url(path), query).await?;
    let code = field(&body, "code").unwrap_or_default();
    if code != "0" {
        return Err(Error::Api {
            exchange: ctx.exchange,
            code,
            message: field(&body, "msg").unwrap_or_default(),
        });
    }
    match body["data"].take() {
        Value::Array(rows) => Ok(rows),
        _ => Err(parse_error(ctx.exchange, "data is not an array")),
    }
}

fn first(ctx: &Ctx<'_>, rows: Vec<Value>) -> Result<Value> {
    rows.into_iter()
        .next()
        .ok_or_else(|| parse_error(ctx.exchange, "empty data"))
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let id = inst_id(ctx, symbol)?;
    let t = first(
        ctx,
        data(ctx, "/api/v5/market/ticker", &[("instId", id)]).await?,
    )?;
    let last = field(&t, "last");
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        change_pct: match (field(&t, "open24h"), &last) {
            (Some(open), Some(last)) => pct_change(&open, last),
            _ => None,
        },
        last,
        bid: field(&t, "bidPx"),
        ask: field(&t, "askPx"),
        high: field(&t, "high24h"),
        low: field(&t, "low24h"),
        // `vol24h` is in the base currency for spot
        base_volume: field(&t, "vol24h"),
        timestamp: int_field(&t, "ts"),
    })
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let id = inst_id(ctx, symbol)?;
    let sz = limit.min(MAX_BOOK).to_string();
    let book = first(
        ctx,
        data(ctx, "/api/v5/market/books", &[("instId", id), ("sz", sz)]).await?,
    )?;
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        bids: levels(ctx.exchange, array(ctx.exchange, &book, "bids")?)?,
        asks: levels(ctx.exchange, array(ctx.exchange, &book, "asks")?)?,
        timestamp: int_field(&book, "ts"),
    })
}

/// OKX bar names. Bars of 6h and longer use the `utc` variants so they align
/// to UTC like the other venues (the plain ones align to Hong Kong time).
fn bar(timeframe: Timeframe) -> Option<&'static str> {
    Some(match timeframe {
        Timeframe::M1 => "1m",
        Timeframe::M3 => "3m",
        Timeframe::M5 => "5m",
        Timeframe::M15 => "15m",
        Timeframe::M30 => "30m",
        Timeframe::H1 => "1H",
        Timeframe::H2 => "2H",
        Timeframe::H4 => "4H",
        Timeframe::H6 => "6Hutc",
        Timeframe::H12 => "12Hutc",
        Timeframe::D1 => "1Dutc",
        Timeframe::D3 => "3Dutc",
        Timeframe::W1 => "1Wutc",
        Timeframe::Mon1 => "1Mutc",
        Timeframe::H8 => return None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    limit: u32,
) -> Result<Vec<Candle>> {
    let id = inst_id(ctx, symbol)?;
    let bar = bar(timeframe).ok_or_else(|| unsupported_timeframe(ctx.exchange, timeframe))?;
    let rows = data(
        ctx,
        "/api/v5/market/candles",
        &[
            ("instId", id),
            ("bar", bar.into()),
            ("limit", limit.min(MAX_CANDLES).to_string()),
        ],
    )
    .await?;
    parse_candles(ctx, &rows)
}

/// `[[ts, o, h, l, c, vol, volCcy, volCcyQuote, confirm], ...]`, newest first
fn parse_candles(ctx: &Ctx<'_>, rows: &[Value]) -> Result<Vec<Candle>> {
    let ex = ctx.exchange;
    rows.iter()
        .map(|row| {
            Ok(Candle {
                timestamp: row
                    .get(0)
                    .and_then(as_i64)
                    .ok_or_else(|| parse_error(ex, "candle without a timestamp"))?,
                open: cell(ex, row, 1)?,
                high: cell(ex, row, 2)?,
                low: cell(ex, row, 3)?,
                close: cell(ex, row, 4)?,
                volume: cell(ex, row, 5)?,
            })
        })
        .collect()
}

pub(super) async fn trades(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<Vec<Trade>> {
    let id = inst_id(ctx, symbol)?;
    let rows = data(
        ctx,
        "/api/v5/market/trades",
        &[("instId", id), ("limit", limit.min(MAX_TRADES).to_string())],
    )
    .await?;
    rows.iter()
        .map(|t| {
            Ok(with_cost(Trade {
                id: field(t, "tradeId"),
                timestamp: int_field(t, "ts")
                    .ok_or_else(|| parse_error(ctx.exchange, "trade without a timestamp"))?,
                side: match field(t, "side").as_deref() {
                    Some("buy") => Side::Buy,
                    _ => Side::Sell,
                },
                price: require(ctx.exchange, t, "px")?,
                amount: require(ctx.exchange, t, "sz")?,
                cost: None,
            }))
        })
        .collect()
}

pub(super) async fn markets(ctx: &Ctx<'_>) -> Result<Vec<Market>> {
    let rows = data(
        ctx,
        "/api/v5/public/instruments",
        &[("instType", "SPOT".into())],
    )
    .await?;
    rows.iter()
        .map(|m| {
            let base = require(ctx.exchange, m, "baseCcy")?;
            let quote = require(ctx.exchange, m, "quoteCcy")?;
            Ok(Market {
                symbol: Symbol::spot(&base, &quote).to_string(),
                id: require(ctx.exchange, m, "instId")?,
                base,
                quote,
                market_type: MarketType::Spot,
                active: field(m, "state").as_deref() == Some("live"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bars_match_what_okx_accepts() {
        // Checked live: 1H/2H/4H valid, 1Hutc and 8H rejected (code 51000)
        assert_eq!(bar(Timeframe::H1), Some("1H"));
        assert_eq!(bar(Timeframe::D1), Some("1Dutc"));
        assert_eq!(bar(Timeframe::Mon1), Some("1Mutc"));
        assert_eq!(bar(Timeframe::H8), None);
    }
}
