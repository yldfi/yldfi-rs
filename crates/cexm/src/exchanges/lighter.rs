//! Lighter: <https://apidocs.lighter.xyz>
//!
//! Markets are addressed by a numeric `market_id`, resolved from
//! `/api/v1/orderBooks`. Perpetuals are listed by coin (`ETH`) and are
//! quoted and settled in USDC, so their unified symbol is `ETH/USDC:USDC`;
//! spot markets are listed as `BASE/QUOTE` (so `ETH/USDC` is spot and
//! `ETH/USDC:USDC` the perpetual). Responses carry `"code": 200` on success,
//! and numbers in details and candles are JSON floats. Lighter refuses some
//! regions with HTTP 403, and rate-limits bursts with HTTP 429.

use super::{change_24h, now_millis, unsupported_timeframe, with_cost, DAY_MILLIS};
use crate::client::Ctx;
use crate::error::{Error, Result};
use crate::http::{array, field, get_json, int_field, parse_error, require};
use crate::types::{
    as_f64, Candle, Level, Market, MarketType, OrderBook, Side, Symbol, Ticker, Timeframe, Trade,
};
use serde_json::Value;

const PERP_QUOTE: &str = "USDC";
const MAX_BOOK_ORDERS: u32 = 250;
const MAX_TRADES: u32 = 100;
const MAX_CANDLES: u32 = 500;

async fn get(ctx: &Ctx<'_>, path: &str, query: &[(&str, String)]) -> Result<Value> {
    let body = get_json(ctx.http, ctx.exchange, &ctx.url(path), query).await?;
    match body.get("code").and_then(Value::as_i64) {
        Some(200) | None => Ok(body),
        Some(code) => Err(Error::Api {
            exchange: ctx.exchange,
            code: code.to_string(),
            message: field(&body, "message").unwrap_or_default(),
        }),
    }
}

/// A Lighter market as listed by `/api/v1/orderBooks`
struct LighterMarket {
    id: i64,
    symbol: Symbol,
    market_type: MarketType,
    active: bool,
}

fn parse_market(ctx: &Ctx<'_>, m: &Value) -> Result<LighterMarket> {
    let name = require(ctx.exchange, m, "symbol")?;
    let spot = match field(m, "market_type").as_deref() {
        Some("spot") => true,
        Some("perp") => false,
        // Older responses have no market_type: spot names contain a '/'
        _ => name.contains('/'),
    };
    let (symbol, market_type) = if spot {
        let symbol = Symbol::parse(&name).ok_or_else(|| {
            parse_error(
                ctx.exchange,
                format!("spot market with an odd name {name:?}"),
            )
        })?;
        (symbol, MarketType::Spot)
    } else {
        (
            Symbol::swap(name.to_ascii_uppercase(), PERP_QUOTE, PERP_QUOTE),
            MarketType::Swap,
        )
    };
    Ok(LighterMarket {
        id: int_field(m, "market_id")
            .ok_or_else(|| parse_error(ctx.exchange, "market without a market_id"))?,
        symbol,
        market_type,
        active: field(m, "status").as_deref() == Some("active"),
    })
}

async fn list_markets(ctx: &Ctx<'_>) -> Result<Vec<LighterMarket>> {
    let body = get(ctx, "/api/v1/orderBooks", &[]).await?;
    array(ctx.exchange, &body, "order_books")?
        .iter()
        .map(|m| parse_market(ctx, m))
        .collect()
}

/// The `market_id` for a symbol; `COIN/USDC` also finds the perpetual
async fn market_id(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<i64> {
    let markets = list_markets(ctx).await?;
    let perp = Symbol::swap(&symbol.base, PERP_QUOTE, PERP_QUOTE);
    markets
        .iter()
        .find(|m| m.symbol == *symbol)
        .or_else(|| {
            (symbol.settle.is_none() && symbol.quote == PERP_QUOTE)
                .then(|| markets.iter().find(|m| m.symbol == perp))
                .flatten()
        })
        .map(|m| m.id)
        .ok_or_else(|| Error::InvalidSymbol {
            exchange: ctx.exchange,
            symbol: symbol.to_string(),
            reason: "not listed (perpetuals are COIN/USDC:USDC)".into(),
        })
}

pub(super) async fn ticker(ctx: &Ctx<'_>, symbol: &Symbol) -> Result<Ticker> {
    let id = market_id(ctx, symbol).await?;
    let body = get(
        ctx,
        "/api/v1/orderBookDetails",
        &[("market_id", id.to_string())],
    )
    .await?;
    // Perpetual details and spot details come back under different keys
    let d = ["order_book_details", "spot_order_book_details"]
        .iter()
        .find_map(|k| body.get(*k)?.as_array()?.first())
        .ok_or_else(|| parse_error(ctx.exchange, "no order book details"))?;
    let last = field(d, "last_trade_price");
    // `daily_price_change` matches neither the 24h change nor the change
    // since 00:00 UTC (checked against Lighter's own candles), so the change
    // comes from the 5-minute candle covering 24 hours ago
    let end = now_millis();
    let recent = candles_by_id(ctx, id, "5m", end - DAY_MILLIS - 10 * 60_000, end).await?;
    let book = book_by_id(ctx, id, 1).await?;
    let best = |levels: &[Level]| levels.first().map(|l| l.price.clone());
    Ok(Ticker {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        change_pct: change_24h(&recent, last.as_deref()),
        last,
        bid: best(&book.0),
        ask: best(&book.1),
        // Rolling 24h, matching the candles
        high: field(d, "daily_price_high"),
        low: field(d, "daily_price_low"),
        base_volume: field(d, "daily_base_token_volume"),
        timestamp: None,
    })
}

/// Sum individual resting orders into price levels, keeping their order
fn aggregate(ctx: &Ctx<'_>, orders: &[Value], limit: usize) -> Result<Vec<Level>> {
    let mut levels: Vec<Level> = Vec::new();
    for o in orders {
        let price = require(ctx.exchange, o, "price")?;
        let amount = require(ctx.exchange, o, "remaining_base_amount")?;
        let same_price = levels
            .last()
            .is_some_and(|l| as_f64(&l.price) == as_f64(&price));
        if same_price {
            let level = levels.last_mut().expect("checked above");
            level.amount = add_decimal(&level.amount, &amount)
                .ok_or_else(|| parse_error(ctx.exchange, "invalid order amount"))?;
        } else if levels.len() == limit {
            break;
        } else {
            levels.push(Level { price, amount });
        }
    }
    Ok(levels)
}

/// Exact sum of two non-negative decimal strings, e.g. `("0.1", "0.25")` → `"0.35"`
fn add_decimal(a: &str, b: &str) -> Option<String> {
    let scale = |s: &str| s.split_once('.').map_or(0, |(_, f)| f.len());
    let digits = scale(a).max(scale(b));
    // Both as integers at the shared scale, e.g. "0.1" → 10 at scale 2
    let as_int = |s: &str| -> Option<u128> {
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        format!("{int}{frac:0<digits$}").parse().ok()
    };
    let sum = format!(
        "{:0>width$}",
        as_int(a)?.checked_add(as_int(b)?)?,
        width = digits + 1
    );
    let (int, frac) = sum.split_at(sum.len() - digits);
    let frac = frac.trim_end_matches('0');
    Some(if frac.is_empty() {
        int.to_string()
    } else {
        format!("{int}.{frac}")
    })
}

/// Bids and asks for a market, summed into at most `limit` levels each
async fn book_by_id(ctx: &Ctx<'_>, id: i64, limit: usize) -> Result<(Vec<Level>, Vec<Level>)> {
    let body = get(
        ctx,
        "/api/v1/orderBookOrders",
        &[
            ("market_id", id.to_string()),
            ("limit", MAX_BOOK_ORDERS.to_string()),
        ],
    )
    .await?;
    Ok((
        aggregate(ctx, array(ctx.exchange, &body, "bids")?, limit)?,
        aggregate(ctx, array(ctx.exchange, &body, "asks")?, limit)?,
    ))
}

pub(super) async fn order_book(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<OrderBook> {
    let id = market_id(ctx, symbol).await?;
    let (bids, asks) = book_by_id(ctx, id, limit as usize).await?;
    Ok(OrderBook {
        exchange: ctx.exchange,
        symbol: symbol.to_string(),
        bids,
        asks,
        timestamp: None,
    })
}

fn resolution(timeframe: Timeframe) -> Option<&'static str> {
    Some(match timeframe {
        Timeframe::M1 => "1m",
        Timeframe::M5 => "5m",
        Timeframe::M15 => "15m",
        Timeframe::M30 => "30m",
        Timeframe::H1 => "1h",
        Timeframe::H4 => "4h",
        Timeframe::H12 => "12h",
        Timeframe::D1 => "1d",
        // Checked live: 3m, 2h, 6h, 8h, 3d, 1w and 1M are rejected
        _ => return None,
    })
}

pub(super) async fn candles(
    ctx: &Ctx<'_>,
    symbol: &Symbol,
    timeframe: Timeframe,
    limit: u32,
) -> Result<Vec<Candle>> {
    let resolution =
        resolution(timeframe).ok_or_else(|| unsupported_timeframe(ctx.exchange, timeframe))?;
    let id = market_id(ctx, symbol).await?;
    let end = now_millis();
    // One extra period so the current, still-open candle doesn't cost a slot
    let start = end - timeframe.millis() * (i64::from(limit.min(MAX_CANDLES)) + 1);
    candles_by_id(ctx, id, resolution, start, end).await
}

/// `/api/v1/candles` returns `{"r": "1h", "c": [{"t", "o", "h", "l", "c", "v", ...}]}`
/// for the window, up to 500 candles (`count_back` is ignored)
async fn candles_by_id(
    ctx: &Ctx<'_>,
    id: i64,
    resolution: &str,
    start: i64,
    end: i64,
) -> Result<Vec<Candle>> {
    let body = get(
        ctx,
        "/api/v1/candles",
        &[
            ("market_id", id.to_string()),
            ("resolution", resolution.into()),
            ("start_timestamp", start.to_string()),
            ("end_timestamp", end.to_string()),
            ("count_back", MAX_CANDLES.to_string()),
        ],
    )
    .await?;
    array(ctx.exchange, &body, "c")?
        .iter()
        .map(|c| {
            Ok(Candle {
                timestamp: int_field(c, "t")
                    .ok_or_else(|| parse_error(ctx.exchange, "candle without a time"))?,
                open: require(ctx.exchange, c, "o")?,
                high: require(ctx.exchange, c, "h")?,
                low: require(ctx.exchange, c, "l")?,
                close: require(ctx.exchange, c, "c")?,
                // `v` is base volume, `V` quote volume
                volume: require(ctx.exchange, c, "v")?,
            })
        })
        .collect()
}

pub(super) async fn trades(ctx: &Ctx<'_>, symbol: &Symbol, limit: u32) -> Result<Vec<Trade>> {
    let id = market_id(ctx, symbol).await?;
    let body = get(
        ctx,
        "/api/v1/recentTrades",
        &[
            ("market_id", id.to_string()),
            ("limit", limit.min(MAX_TRADES).to_string()),
        ],
    )
    .await?;
    array(ctx.exchange, &body, "trades")?
        .iter()
        .map(|t| {
            Ok(with_cost(Trade {
                id: field(t, "trade_id_str").or_else(|| field(t, "trade_id")),
                timestamp: int_field(t, "timestamp")
                    .ok_or_else(|| parse_error(ctx.exchange, "trade without a timestamp"))?,
                // The ask was the maker, so the taker bought
                side: if t.get("is_maker_ask").and_then(Value::as_bool) == Some(true) {
                    Side::Buy
                } else {
                    Side::Sell
                },
                price: require(ctx.exchange, t, "price")?,
                amount: require(ctx.exchange, t, "size")?,
                cost: field(t, "usd_amount"),
            }))
        })
        .collect()
}

pub(super) async fn markets(ctx: &Ctx<'_>) -> Result<Vec<Market>> {
    Ok(list_markets(ctx)
        .await?
        .into_iter()
        .map(|m| Market {
            symbol: m.symbol.to_string(),
            id: m.id.to_string(),
            base: m.symbol.base,
            quote: m.symbol.quote,
            market_type: m.market_type,
            active: m.active,
        })
        .collect())
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
            exchange: Exchange::Lighter,
        }
    }

    #[test]
    fn orders_are_summed_into_levels() {
        let http = reqwest::Client::new();
        let orders = json!([
            {"price": "3025.00", "remaining_base_amount": "0.10"},
            {"price": "3025.00", "remaining_base_amount": "0.25"},
            {"price": "3025.50", "remaining_base_amount": "1"},
            {"price": "3026.00", "remaining_base_amount": "2"},
        ]);
        let levels = aggregate(&ctx(&http), orders.as_array().unwrap(), 2).unwrap();
        assert_eq!(levels.len(), 2);
        assert_eq!(levels[0].amount, "0.35");
        assert_eq!(levels[1].price, "3025.50");
    }

    #[test]
    fn add_decimal_is_exact() {
        assert_eq!(add_decimal("0.1", "0.2").as_deref(), Some("0.3"));
        assert_eq!(add_decimal("1", "2.50").as_deref(), Some("3.5"));
        assert_eq!(add_decimal("10", "5").as_deref(), Some("15"));
    }

    #[test]
    fn perps_and_spot_markets_parse() {
        let http = reqwest::Client::new();
        let perp = parse_market(
            &ctx(&http),
            &json!({"symbol": "ETH", "market_id": 0, "market_type": "perp", "status": "active"}),
        )
        .unwrap();
        assert_eq!(perp.symbol.to_string(), "ETH/USDC:USDC");
        assert_eq!(perp.market_type, MarketType::Swap);
        let spot = parse_market(
            &ctx(&http),
            &json!({"symbol": "ETH/USDC", "market_id": 2048, "market_type": "spot", "status": "inactive"}),
        )
        .unwrap();
        assert_eq!(spot.symbol.to_string(), "ETH/USDC");
        assert!(!spot.active);
    }
}
