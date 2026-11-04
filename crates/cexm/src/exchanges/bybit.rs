//! Bybit spot, v5 API: <https://bybit-exchange.github.io/docs/v5/market/tickers>
//!
//! Responses are wrapped as `{"retCode": 0, "retMsg": "OK", "result": ...}`.
//! Bybit refuses some regions (including the US) with HTTP 403.

use super::{require_spot, unsupported_timeframe, with_cost};
use crate::client::Ctx;
use crate::error::{Error, Result};
use crate::http::{array, as_i64, cell, field, get_json, int_field, levels, parse_error, require};
use crate::types::{
    as_f64, Candle, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe, Trade,
};
use serde_json::Value;

const MAX_BOOK: u32 = 200;
const MAX_CANDLES: u32 = 1000;
const MAX_TRADES: u32 = 60;

fn market_id(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<String> {
    require_spot(ctx.exchange, symbol)?;
    Ok(format!("{}{}", symbol.base, symbol.quote))
}

/// GET a spot endpoint and return the whole body after checking `retCode`
async fn get(ctx: &Ctx<'_>, path: &str, query: &[(&str, String)]) -> Result<Value> {
    let mut query = query.to_vec();
    query.push(("category", "spot".into()));
    let body = get_json(ctx.http, ctx.exchange, &ctx.url(path), &query).await?;
    let code = field(&body, "retCode").unwrap_or_default();
    if code != "0" {
        return Err(Error::Api {
            exchange: ctx.exchange,
            code,
            message: field(&body, "retMsg").unwrap_or_default(),
        });
    }
    Ok(body)
}

/// `result.list`
fn list<'a>(ctx: &Ctx<'_>, body: &'a Value) -> Result<&'a Vec<Value>> {
    array(ctx.exchange, &body["result"], "list")
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let id = market_id(ctx, symbol)?;
    let body = get(ctx, "/v5/market/tickers", &[("symbol", id)]).await?;
    let t = list(ctx, &body)?
        .first()
        .ok_or_else(|| not_listed(ctx, symbol))?;
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        last: field(t, "lastPrice"),
        bid: field(t, "bid1Price"),
        ask: field(t, "ask1Price"),
        high: field(t, "highPrice24h"),
        low: field(t, "lowPrice24h"),
        base_volume: field(t, "volume24h"),
        // `price24hPcnt` is a ratio (0.03 = +3%)
        change_pct: field(t, "price24hPcnt")
            .as_deref()
            .and_then(as_f64)
            .map(|r| r * 100.0),
        timestamp: int_field(&body, "time"),
    })
}

fn not_listed(ctx: &Ctx<'_>, symbol: &Symbol) -> Error {
    Error::InvalidSymbol {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        reason: "not listed".into(),
    }
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let id = market_id(ctx, symbol)?;
    let body = get(
        ctx,
        "/v5/market/orderbook",
        &[("symbol", id), ("limit", limit.min(MAX_BOOK).to_string())],
    )
    .await?;
    let book = &body["result"];
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        bids: levels(ctx.exchange, array(ctx.exchange, book, "b")?)?,
        asks: levels(ctx.exchange, array(ctx.exchange, book, "a")?)?,
        timestamp: int_field(book, "ts"),
    })
}

fn interval(timeframe: Timeframe) -> Option<&'static str> {
    Some(match timeframe {
        Timeframe::M1 => "1",
        Timeframe::M3 => "3",
        Timeframe::M5 => "5",
        Timeframe::M15 => "15",
        Timeframe::M30 => "30",
        Timeframe::H1 => "60",
        Timeframe::H2 => "120",
        Timeframe::H4 => "240",
        Timeframe::H6 => "360",
        Timeframe::H12 => "720",
        Timeframe::D1 => "D",
        Timeframe::W1 => "W",
        Timeframe::Mon1 => "M",
        Timeframe::H8 | Timeframe::D3 => return None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    limit: u32,
) -> Result<Vec<Candle>> {
    let id = market_id(ctx, symbol)?;
    let interval =
        interval(timeframe).ok_or_else(|| unsupported_timeframe(ctx.exchange, timeframe))?;
    let body = get(
        ctx,
        "/v5/market/kline",
        &[
            ("symbol", id),
            ("interval", interval.into()),
            ("limit", limit.min(MAX_CANDLES).to_string()),
        ],
    )
    .await?;
    let ex = ctx.exchange;
    // [[start, open, high, low, close, volume, turnover], ...], newest first
    list(ctx, &body)?
        .iter()
        .map(|row| {
            Ok(Candle {
                timestamp: row
                    .get(0)
                    .and_then(as_i64)
                    .ok_or_else(|| parse_error(ex, "kline without a start time"))?,
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
    let id = market_id(ctx, symbol)?;
    let body = get(
        ctx,
        "/v5/market/recent-trade",
        &[("symbol", id), ("limit", limit.min(MAX_TRADES).to_string())],
    )
    .await?;
    list(ctx, &body)?
        .iter()
        .map(|t| {
            Ok(with_cost(Trade {
                id: field(t, "execId"),
                timestamp: int_field(t, "time")
                    .ok_or_else(|| parse_error(ctx.exchange, "trade without a time"))?,
                side: match field(t, "side").as_deref() {
                    Some("Buy") => Side::Buy,
                    _ => Side::Sell,
                },
                price: require(ctx.exchange, t, "price")?,
                amount: require(ctx.exchange, t, "size")?,
                cost: None,
            }))
        })
        .collect()
}

pub(super) async fn markets(ctx: &Ctx<'_>) -> Result<Vec<Market>> {
    let mut markets = Vec::new();
    let mut cursor = String::new();
    loop {
        let mut query = vec![("limit", "1000".to_string())];
        if !cursor.is_empty() {
            query.push(("cursor", cursor.clone()));
        }
        let body = get(ctx, "/v5/market/instruments-info", &query).await?;
        for m in list(ctx, &body)? {
            let base = require(ctx.exchange, m, "baseCoin")?;
            let quote = require(ctx.exchange, m, "quoteCoin")?;
            markets.push(Market {
                symbol: Symbol::spot(&base, &quote).to_string(),
                id: require(ctx.exchange, m, "symbol")?,
                base,
                quote,
                market_type: MarketType::Spot,
                active: field(m, "status").as_deref() == Some("Trading"),
            });
        }
        match field(&body["result"], "nextPageCursor") {
            Some(next) if next != cursor => cursor = next,
            _ => return Ok(markets),
        }
    }
}
