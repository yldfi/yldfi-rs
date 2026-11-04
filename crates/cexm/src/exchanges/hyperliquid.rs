//! Hyperliquid perpetuals: <https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint>
//!
//! Everything is a `POST /info` with a JSON body. Markets are quoted and
//! settled in USDC, so the unified symbol is `COIN/USDC:USDC`; `COIN/USDC`
//! is accepted too. An unknown coin is answered with HTTP 500 and `null`.

use super::{now_millis, unsupported_timeframe, with_cost};
use crate::client::Ctx;
use crate::error::{Error, Result};
use crate::http::{array, field, int_field, parse_error, post_json, require};
use crate::types::{
    pct_change, Candle, Level, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe,
    Trade,
};
use serde_json::{json, Value};

const QUOTE: &str = "USDC";
/// Hyperliquid serves at most this many candles per request
const MAX_CANDLES: u32 = 5000;

/// The coin name (`BTC`) for a symbol, which must be quoted in USDC
fn coin(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<String> {
    let usdc = symbol.quote == QUOTE && symbol.settle.as_deref().is_none_or(|s| s == QUOTE);
    if !usdc {
        return Err(Error::InvalidSymbol {
            exchange: ctx.exchange,
            symbol: symbol.to_string(),
            reason: "Hyperliquid perpetuals are quoted in USDC, e.g. BTC/USDC:USDC".into(),
        });
    }
    Ok(symbol.base.clone())
}

fn unified(coin: &str) -> String {
    Symbol::swap(coin, QUOTE, QUOTE).to_string()
}

async fn info(ctx: &Ctx<'_>, symbol: Option<&Symbol>, body: Value) -> Result<Value> {
    let result = post_json(ctx.http, ctx.exchange, &ctx.url("/info"), &body).await;
    match (result, symbol) {
        // An unknown coin comes back as HTTP 500 with a `null` body
        (Err(Error::Api { code, message, .. }), Some(symbol))
            if code == "500" && message == "null" =>
        {
            Err(not_listed(ctx, symbol))
        }
        (Ok(Value::Null), Some(symbol)) => Err(not_listed(ctx, symbol)),
        (result, _) => result,
    }
}

fn not_listed(ctx: &Ctx<'_>, symbol: &Symbol) -> Error {
    Error::InvalidSymbol {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        reason: "not listed".into(),
    }
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let coin = coin(ctx, symbol)?;
    // [meta, assetCtxs]: contexts are in the same order as meta.universe
    let body = info(ctx, None, json!({"type": "metaAndAssetCtxs"})).await?;
    let universe = array(ctx.exchange, &body[0], "universe")?;
    let index = universe
        .iter()
        .position(|m| field(m, "name").as_deref() == Some(coin.as_str()))
        .ok_or_else(|| not_listed(ctx, symbol))?;
    let c = body[1]
        .get(index)
        .ok_or_else(|| parse_error(ctx.exchange, "asset context missing"))?;
    let mid = field(c, "midPx");
    // The asset context has no top of book; take it from the L2 book
    let book = order_book(ctx, symbol, 1).await?;
    let best = |levels: &[Level]| levels.first().map(|l| l.price.clone());
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: unified(&coin),
        change_pct: match (field(c, "prevDayPx"), &mid) {
            (Some(prev), Some(mid)) => pct_change(&prev, mid),
            _ => None,
        },
        // No last-trade price in the context; the mid is the closest equivalent
        last: mid,
        bid: best(&book.bids),
        ask: best(&book.asks),
        high: None,
        low: None,
        base_volume: field(c, "dayBaseVlm"),
        timestamp: book.timestamp,
    })
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let coin = coin(ctx, symbol)?;
    let book = info(ctx, Some(symbol), json!({"type": "l2Book", "coin": coin})).await?;
    let sides = array(ctx.exchange, &book, "levels")?;
    let side = |i: usize| -> Result<Vec<Level>> {
        sides
            .get(i)
            .and_then(Value::as_array)
            .ok_or_else(|| parse_error(ctx.exchange, "missing book side"))?
            .iter()
            .take(limit as usize)
            .map(|l| {
                Ok(Level {
                    price: require(ctx.exchange, l, "px")?,
                    amount: require(ctx.exchange, l, "sz")?,
                })
            })
            .collect()
    };
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: unified(&coin),
        bids: side(0)?,
        asks: side(1)?,
        timestamp: int_field(&book, "time"),
    })
}

fn interval(timeframe: Timeframe) -> Option<&'static str> {
    Some(match timeframe {
        Timeframe::M1 => "1m",
        Timeframe::M3 => "3m",
        Timeframe::M5 => "5m",
        Timeframe::M15 => "15m",
        Timeframe::M30 => "30m",
        Timeframe::H1 => "1h",
        Timeframe::H2 => "2h",
        Timeframe::H4 => "4h",
        Timeframe::H8 => "8h",
        Timeframe::H12 => "12h",
        Timeframe::D1 => "1d",
        Timeframe::D3 => "3d",
        Timeframe::W1 => "1w",
        Timeframe::Mon1 => "1M",
        Timeframe::H6 => return None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    limit: u32,
) -> Result<Vec<Candle>> {
    let coin = coin(ctx, symbol)?;
    let interval =
        interval(timeframe).ok_or_else(|| unsupported_timeframe(ctx.exchange, timeframe))?;
    let end = now_millis();
    // One extra period so the current, still-open candle doesn't cost a slot
    let start = end - timeframe.millis() * (i64::from(limit.min(MAX_CANDLES)) + 1);
    let rows = info(
        ctx,
        Some(symbol),
        json!({"type": "candleSnapshot", "req": {
            "coin": coin, "interval": interval, "startTime": start, "endTime": end,
        }}),
    )
    .await?;
    rows.as_array()
        .ok_or_else(|| parse_error(ctx.exchange, "candles are not an array"))?
        .iter()
        .map(|c| {
            Ok(Candle {
                timestamp: int_field(c, "t")
                    .ok_or_else(|| parse_error(ctx.exchange, "candle without a timestamp"))?,
                open: require(ctx.exchange, c, "o")?,
                high: require(ctx.exchange, c, "h")?,
                low: require(ctx.exchange, c, "l")?,
                close: require(ctx.exchange, c, "c")?,
                volume: require(ctx.exchange, c, "v")?,
            })
        })
        .collect()
}

pub(super) async fn trades(ctx: &Ctx<'_>, symbol: &Symbol, _limit: u32) -> Result<Vec<Trade>> {
    // `recentTrades` takes no limit; the client trims the result
    let coin = coin(ctx, symbol)?;
    let rows = info(
        ctx,
        Some(symbol),
        json!({"type": "recentTrades", "coin": coin}),
    )
    .await?;
    rows.as_array()
        .ok_or_else(|| parse_error(ctx.exchange, "trades are not an array"))?
        .iter()
        .map(|t| {
            Ok(with_cost(Trade {
                id: field(t, "tid"),
                timestamp: int_field(t, "time")
                    .ok_or_else(|| parse_error(ctx.exchange, "trade without a timestamp"))?,
                // "B" = buy (bid side took liquidity), "A" = sell
                side: match field(t, "side").as_deref() {
                    Some("B") => Side::Buy,
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
    let meta = info(ctx, None, json!({"type": "meta"})).await?;
    array(ctx.exchange, &meta, "universe")?
        .iter()
        .map(|m| {
            let coin = require(ctx.exchange, m, "name")?;
            Ok(Market {
                symbol: unified(&coin),
                id: coin.clone(),
                base: coin,
                quote: QUOTE.into(),
                market_type: MarketType::Swap,
                active: !m
                    .get("isDelisted")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Exchange;

    #[test]
    fn coin_accepts_usdc_symbols_only() {
        let http = reqwest::Client::new();
        let ctx = Ctx {
            http: &http,
            base: "",
            exchange: Exchange::Hyperliquid,
        };
        let s = |x: &str| Symbol::parse(x).unwrap();
        assert_eq!(coin(&ctx, &s("BTC/USDC:USDC")).unwrap(), "BTC");
        assert_eq!(coin(&ctx, &s("ETH/USDC")).unwrap(), "ETH");
        assert!(coin(&ctx, &s("BTC/USDT")).is_err());
        assert!(coin(&ctx, &s("BTC/USDC:USDT")).is_err());
    }
}
