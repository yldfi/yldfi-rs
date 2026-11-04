//! Bitget spot, v2 API: <https://www.bitget.com/api-doc/spot/market/Get-Tickers>
//!
//! Responses are wrapped as `{"code": "00000", "msg": ..., "data": ...}`.

use super::{require_spot, unsupported_timeframe, with_cost};
use crate::client::Ctx;
use crate::error::{Error, Result};
use crate::http::{array, as_i64, cell, field, get_json, int_field, levels, parse_error, require};
use crate::types::{
    as_f64, Candle, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe, Trade,
};
use serde_json::Value;

const MAX_BOOK: u32 = 150;
const MAX_CANDLES: u32 = 1000;
const MAX_TRADES: u32 = 500;

fn market_id(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<String> {
    require_spot(ctx.exchange, symbol)?;
    Ok(format!("{}{}", symbol.base, symbol.quote))
}

/// GET and unwrap `data`, turning a non-`00000` code into [`Error::Api`]
async fn data(ctx: &Ctx<'_>, path: &str, query: &[(&str, String)]) -> Result<Value> {
    let mut body = get_json(ctx.http, ctx.exchange, &ctx.url(path), query).await?;
    let code = field(&body, "code").unwrap_or_default();
    if code != "00000" {
        return Err(Error::Api {
            exchange: ctx.exchange,
            code,
            message: field(&body, "msg").unwrap_or_default(),
        });
    }
    Ok(body["data"].take())
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let id = market_id(ctx, symbol)?;
    let data = data(ctx, "/api/v2/spot/market/tickers", &[("symbol", id)]).await?;
    let t = data
        .as_array()
        .and_then(|rows| rows.first())
        .ok_or_else(|| parse_error(ctx.exchange, "empty ticker list"))?;
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        last: field(t, "lastPr"),
        bid: field(t, "bidPr"),
        ask: field(t, "askPr"),
        high: field(t, "high24h"),
        low: field(t, "low24h"),
        base_volume: field(t, "baseVolume"),
        // `change24h` is a ratio (0.03 = +3%)
        change_pct: field(t, "change24h")
            .as_deref()
            .and_then(as_f64)
            .map(|r| r * 100.0),
        timestamp: int_field(t, "ts"),
    })
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let id = market_id(ctx, symbol)?;
    let limit = limit.min(MAX_BOOK).to_string();
    let book = data(
        ctx,
        "/api/v2/spot/market/orderbook",
        &[("symbol", id), ("type", "step0".into()), ("limit", limit)],
    )
    .await?;
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        bids: levels(ctx.exchange, array(ctx.exchange, &book, "bids")?)?,
        asks: levels(ctx.exchange, array(ctx.exchange, &book, "asks")?)?,
        timestamp: int_field(&book, "ts"),
    })
}

/// Bitget granularities. Bars of 6h and longer use the `utc` variants: the
/// plain ones (`1day`, `1week`, ...) align to UTC+8.
fn granularity(timeframe: Timeframe) -> Option<&'static str> {
    Some(match timeframe {
        Timeframe::M1 => "1min",
        Timeframe::M3 => "3min",
        Timeframe::M5 => "5min",
        Timeframe::M15 => "15min",
        Timeframe::M30 => "30min",
        Timeframe::H1 => "1h",
        Timeframe::H4 => "4h",
        Timeframe::H6 => "6Hutc",
        Timeframe::H12 => "12Hutc",
        Timeframe::D1 => "1Dutc",
        Timeframe::D3 => "3Dutc",
        Timeframe::W1 => "1Wutc",
        Timeframe::Mon1 => "1Mutc",
        Timeframe::H2 | Timeframe::H8 => return None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    limit: u32,
) -> Result<Vec<Candle>> {
    let id = market_id(ctx, symbol)?;
    let g = granularity(timeframe).ok_or_else(|| unsupported_timeframe(ctx.exchange, timeframe))?;
    let rows = data(
        ctx,
        "/api/v2/spot/market/candles",
        &[
            ("symbol", id),
            ("granularity", g.into()),
            ("limit", limit.min(MAX_CANDLES).to_string()),
        ],
    )
    .await?;
    parse_candles(ctx, &rows)
}

/// `[[ts, open, high, low, close, baseVol, usdtVol, quoteVol], ...]`
fn parse_candles(ctx: &Ctx<'_>, rows: &Value) -> Result<Vec<Candle>> {
    let ex = ctx.exchange;
    rows.as_array()
        .ok_or_else(|| parse_error(ex, "candles are not an array"))?
        .iter()
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
    let id = market_id(ctx, symbol)?;
    let rows = data(
        ctx,
        "/api/v2/spot/market/fills",
        &[("symbol", id), ("limit", limit.min(MAX_TRADES).to_string())],
    )
    .await?;
    rows.as_array()
        .ok_or_else(|| parse_error(ctx.exchange, "trades are not an array"))?
        .iter()
        .map(|t| {
            Ok(with_cost(Trade {
                id: field(t, "tradeId"),
                timestamp: int_field(t, "ts")
                    .ok_or_else(|| parse_error(ctx.exchange, "trade without a timestamp"))?,
                side: match field(t, "side").as_deref() {
                    Some("buy") => Side::Buy,
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
    let rows = data(ctx, "/api/v2/spot/public/symbols", &[]).await?;
    rows.as_array()
        .ok_or_else(|| parse_error(ctx.exchange, "symbols are not an array"))?
        .iter()
        .map(|m| {
            let base = require(ctx.exchange, m, "baseCoin")?.to_ascii_uppercase();
            let quote = require(ctx.exchange, m, "quoteCoin")?.to_ascii_uppercase();
            Ok(Market {
                symbol: Symbol::spot(&base, &quote).to_string(),
                id: require(ctx.exchange, m, "symbol")?,
                base,
                quote,
                market_type: MarketType::Spot,
                active: field(m, "status").as_deref() == Some("online"),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Exchange;
    use serde_json::json;

    fn ctx(http: &reqwest::Client) -> Ctx<'_> {
        Ctx {
            http,
            base: "",
            exchange: Exchange::Bitget,
        }
    }

    #[test]
    fn candles_parse_live_shape() {
        let http = reqwest::Client::new();
        let rows = json!([[
            "1793790000000",
            "2328.54",
            "2337.68",
            "2326.7",
            "2335.37",
            "2359.02",
            "5.5e6",
            "5.5e6"
        ]]);
        let c = parse_candles(&ctx(&http), &rows).unwrap();
        assert_eq!(c[0].timestamp, 1_793_790_000_000);
        assert_eq!(c[0].close, "2335.37");
    }

    #[test]
    fn granularity_uses_v2_names() {
        assert_eq!(granularity(Timeframe::H1), Some("1h"));
        assert_eq!(granularity(Timeframe::D1), Some("1Dutc"));
        assert_eq!(granularity(Timeframe::M5), Some("5min"));
        assert_eq!(granularity(Timeframe::Mon1), Some("1Mutc"));
        assert_eq!(granularity(Timeframe::H2), None);
    }
}
