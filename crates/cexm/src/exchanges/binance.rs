//! Binance spot: <https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints>
//!
//! The default base URL is `https://data-api.binance.vision`, Binance's
//! market-data-only host. It serves the same `/api/v3` endpoints as
//! `api.binance.com`, which answers HTTP 451 from restricted regions.

use super::{require_spot, with_cost};
use crate::client::Ctx;
use crate::error::Result;
use crate::http::{array, as_i64, cell, field, get_json, int_field, levels, parse_error, require};
use crate::types::{
    as_f64, Candle, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe, Trade,
};
use serde_json::Value;

const MAX_BOOK: u32 = 5000;
const MAX_CANDLES: u32 = 1000;
const MAX_TRADES: u32 = 1000;

fn market_id(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<String> {
    require_spot(ctx.exchange, symbol)?;
    Ok(format!("{}{}", symbol.base, symbol.quote))
}

async fn get(ctx: &Ctx<'_>, path: &str, query: &[(&str, String)]) -> Result<Value> {
    get_json(ctx.http, ctx.exchange, &ctx.url(path), query).await
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let id = market_id(ctx, symbol)?;
    let t = get(ctx, "/api/v3/ticker/24hr", &[("symbol", id)]).await?;
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        last: field(&t, "lastPrice"),
        bid: field(&t, "bidPrice"),
        ask: field(&t, "askPrice"),
        high: field(&t, "highPrice"),
        low: field(&t, "lowPrice"),
        base_volume: field(&t, "volume"),
        change_pct: field(&t, "priceChangePercent").as_deref().and_then(as_f64),
        timestamp: int_field(&t, "closeTime"),
    })
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let id = market_id(ctx, symbol)?;
    let book = get(
        ctx,
        "/api/v3/depth",
        &[("symbol", id), ("limit", limit.min(MAX_BOOK).to_string())],
    )
    .await?;
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        bids: levels(ctx.exchange, array(ctx.exchange, &book, "bids")?)?,
        asks: levels(ctx.exchange, array(ctx.exchange, &book, "asks")?)?,
        // The depth snapshot carries an update id, not a time
        timestamp: None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    limit: u32,
) -> Result<Vec<Candle>> {
    let id = market_id(ctx, symbol)?;
    // Binance accepts every Timeframe label as-is
    let rows = get(
        ctx,
        "/api/v3/klines",
        &[
            ("symbol", id),
            ("interval", timeframe.as_str().into()),
            ("limit", limit.min(MAX_CANDLES).to_string()),
        ],
    )
    .await?;
    let ex = ctx.exchange;
    rows.as_array()
        .ok_or_else(|| parse_error(ex, "klines are not an array"))?
        .iter()
        .map(|row| {
            Ok(Candle {
                timestamp: row
                    .get(0)
                    .and_then(as_i64)
                    .ok_or_else(|| parse_error(ex, "kline without an open time"))?,
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
    let rows = get(
        ctx,
        "/api/v3/trades",
        &[("symbol", id), ("limit", limit.min(MAX_TRADES).to_string())],
    )
    .await?;
    rows.as_array()
        .ok_or_else(|| parse_error(ctx.exchange, "trades are not an array"))?
        .iter()
        .map(|t| {
            Ok(with_cost(Trade {
                id: field(t, "id"),
                timestamp: int_field(t, "time")
                    .ok_or_else(|| parse_error(ctx.exchange, "trade without a time"))?,
                // The buyer was the maker, so the taker sold
                side: if t.get("isBuyerMaker").and_then(Value::as_bool) == Some(true) {
                    Side::Sell
                } else {
                    Side::Buy
                },
                price: require(ctx.exchange, t, "price")?,
                amount: require(ctx.exchange, t, "qty")?,
                cost: field(t, "quoteQty"),
            }))
        })
        .collect()
}

pub(super) async fn markets(ctx: &Ctx<'_>) -> Result<Vec<Market>> {
    let info = get(ctx, "/api/v3/exchangeInfo", &[]).await?;
    array(ctx.exchange, &info, "symbols")?
        .iter()
        .map(|m| {
            let base = require(ctx.exchange, m, "baseAsset")?;
            let quote = require(ctx.exchange, m, "quoteAsset")?;
            Ok(Market {
                symbol: Symbol::spot(&base, &quote).to_string(),
                id: require(ctx.exchange, m, "symbol")?,
                base,
                quote,
                market_type: MarketType::Spot,
                active: field(m, "status").as_deref() == Some("TRADING"),
            })
        })
        .collect()
}
