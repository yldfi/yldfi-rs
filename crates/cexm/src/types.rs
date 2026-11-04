//! Exchange-independent market data types
//!
//! Prices and amounts are kept as the decimal strings the exchange returned,
//! so no precision is lost. Use [`as_f64`] when an approximate value is enough.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A supported exchange
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Exchange {
    /// Binance spot (`data-api.binance.vision`, its market-data host)
    Binance,
    /// Bitget spot, v2 API (`api.bitget.com`)
    Bitget,
    /// OKX spot, v5 API (`www.okx.com`)
    Okx,
    /// Hyperliquid perpetuals (`api.hyperliquid.xyz`)
    Hyperliquid,
    /// Lighter perpetuals (`mainnet.zklighter.elliot.ai`)
    Lighter,
    /// Bybit spot, v5 API (`api.bybit.com`)
    Bybit,
    /// Coinbase Exchange spot (`api.exchange.coinbase.com`)
    Coinbase,
    /// Kraken spot (`api.kraken.com`)
    Kraken,
}

impl Exchange {
    /// Every supported exchange
    pub const ALL: [Exchange; 8] = [
        Exchange::Binance,
        Exchange::Bitget,
        Exchange::Okx,
        Exchange::Hyperliquid,
        Exchange::Lighter,
        Exchange::Bybit,
        Exchange::Coinbase,
        Exchange::Kraken,
    ];

    /// Lowercase identifier, e.g. `"binance"`
    pub fn as_str(self) -> &'static str {
        match self {
            Exchange::Binance => "binance",
            Exchange::Bitget => "bitget",
            Exchange::Okx => "okx",
            Exchange::Hyperliquid => "hyperliquid",
            Exchange::Lighter => "lighter",
            Exchange::Bybit => "bybit",
            Exchange::Coinbase => "coinbase",
            Exchange::Kraken => "kraken",
        }
    }
}

impl fmt::Display for Exchange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl FromStr for Exchange {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Exchange::ALL
            .into_iter()
            .find(|e| e.as_str().eq_ignore_ascii_case(s))
            .ok_or_else(|| {
                let names: Vec<_> = Exchange::ALL.iter().map(|e| e.as_str()).collect();
                format!(
                    "unknown exchange {s:?} (expected one of: {})",
                    names.join(", ")
                )
            })
    }
}

/// A unified market symbol: `BASE/QUOTE` for spot, `BASE/QUOTE:SETTLE` for
/// perpetual swaps (e.g. `BTC/USDT`, `BTC/USDC:USDC`)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    /// Base asset, e.g. `BTC`
    pub base: String,
    /// Quote asset, e.g. `USDT`
    pub quote: String,
    /// Settlement asset for swaps, `None` for spot
    pub settle: Option<String>,
}

impl Symbol {
    /// A spot symbol
    pub fn spot(base: impl Into<String>, quote: impl Into<String>) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
            settle: None,
        }
    }

    /// A swap symbol (`BASE/QUOTE:SETTLE`)
    pub fn swap(
        base: impl Into<String>,
        quote: impl Into<String>,
        settle: impl Into<String>,
    ) -> Self {
        Self {
            base: base.into(),
            quote: quote.into(),
            settle: Some(settle.into()),
        }
    }

    /// Parse `BASE/QUOTE` or `BASE/QUOTE:SETTLE`, upper-casing each part
    pub fn parse(s: &str) -> Option<Self> {
        let (pair, settle) = match s.split_once(':') {
            Some((pair, settle)) => (pair, Some(settle)),
            None => (s, None),
        };
        let (base, quote) = pair.split_once('/')?;
        let part = |p: &str| {
            let p = p.trim();
            (!p.is_empty() && p.chars().all(|c| c.is_ascii_alphanumeric()))
                .then(|| p.to_ascii_uppercase())
        };
        Some(Self {
            base: part(base)?,
            quote: part(quote)?,
            settle: match settle {
                Some(s) => Some(part(s)?),
                None => None,
            },
        })
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.base, self.quote)?;
        if let Some(settle) = &self.settle {
            write!(f, ":{settle}")?;
        }
        Ok(())
    }
}

/// Candle timeframe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Timeframe {
    /// 1 minute
    M1,
    /// 3 minutes
    M3,
    /// 5 minutes
    M5,
    /// 15 minutes
    M15,
    /// 30 minutes
    M30,
    /// 1 hour
    H1,
    /// 2 hours
    H2,
    /// 4 hours
    H4,
    /// 6 hours
    H6,
    /// 8 hours
    H8,
    /// 12 hours
    H12,
    /// 1 day
    D1,
    /// 3 days
    D3,
    /// 1 week
    W1,
    /// 1 month
    Mon1,
}

impl Timeframe {
    /// Canonical label: `1m`, `1h`, `1d`, `1w`, `1M` (month)
    pub fn as_str(self) -> &'static str {
        match self {
            Timeframe::M1 => "1m",
            Timeframe::M3 => "3m",
            Timeframe::M5 => "5m",
            Timeframe::M15 => "15m",
            Timeframe::M30 => "30m",
            Timeframe::H1 => "1h",
            Timeframe::H2 => "2h",
            Timeframe::H4 => "4h",
            Timeframe::H6 => "6h",
            Timeframe::H8 => "8h",
            Timeframe::H12 => "12h",
            Timeframe::D1 => "1d",
            Timeframe::D3 => "3d",
            Timeframe::W1 => "1w",
            Timeframe::Mon1 => "1M",
        }
    }

    /// Length in milliseconds (a month counts as 30 days)
    pub fn millis(self) -> i64 {
        const MIN: i64 = 60_000;
        MIN * match self {
            Timeframe::M1 => 1,
            Timeframe::M3 => 3,
            Timeframe::M5 => 5,
            Timeframe::M15 => 15,
            Timeframe::M30 => 30,
            Timeframe::H1 => 60,
            Timeframe::H2 => 120,
            Timeframe::H4 => 240,
            Timeframe::H6 => 360,
            Timeframe::H8 => 480,
            Timeframe::H12 => 720,
            Timeframe::D1 => 1_440,
            Timeframe::D3 => 4_320,
            Timeframe::W1 => 10_080,
            Timeframe::Mon1 => 43_200,
        }
    }
}

impl fmt::Display for Timeframe {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(self.as_str())
    }
}

impl FromStr for Timeframe {
    type Err = String;

    /// Parse `1m` … `1w`, case-insensitively, except that `1M` / `M` mean a
    /// month (lower-case `1m` is a minute). `d` and `w` alone mean `1d` / `1w`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if matches!(s, "1M" | "M") {
            return Ok(Timeframe::Mon1);
        }
        Ok(match s.to_ascii_lowercase().as_str() {
            "1m" => Timeframe::M1,
            "3m" => Timeframe::M3,
            "5m" => Timeframe::M5,
            "15m" => Timeframe::M15,
            "30m" => Timeframe::M30,
            "1h" => Timeframe::H1,
            "2h" => Timeframe::H2,
            "4h" => Timeframe::H4,
            "6h" => Timeframe::H6,
            "8h" => Timeframe::H8,
            "12h" => Timeframe::H12,
            "1d" | "d" => Timeframe::D1,
            "3d" => Timeframe::D3,
            "1w" | "w" => Timeframe::W1,
            _ => {
                return Err(format!(
                    "invalid timeframe {s:?}. Valid: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M"
                ))
            }
        })
    }
}

/// 24-hour ticker
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticker {
    /// Exchange it came from
    pub exchange: Exchange,
    /// Unified symbol, e.g. `BTC/USDT`
    pub symbol: String,
    /// Last traded price (Hyperliquid: mid price)
    pub last: Option<String>,
    /// Best bid
    pub bid: Option<String>,
    /// Best ask
    pub ask: Option<String>,
    /// 24h high
    pub high: Option<String>,
    /// 24h low
    pub low: Option<String>,
    /// 24h volume in the base asset
    pub base_volume: Option<String>,
    /// 24h price change in percent (e.g. `1.25` for +1.25%)
    pub change_pct: Option<f64>,
    /// Exchange timestamp in milliseconds, if the exchange gives one
    pub timestamp: Option<i64>,
}

/// One order book price level
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Level {
    /// Price
    pub price: String,
    /// Amount in the base asset
    pub amount: String,
}

/// Order book snapshot (bids best-first, asks best-first)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderBook {
    /// Exchange it came from
    pub exchange: Exchange,
    /// Unified symbol
    pub symbol: String,
    /// Bids, highest price first
    pub bids: Vec<Level>,
    /// Asks, lowest price first
    pub asks: Vec<Level>,
    /// Exchange timestamp in milliseconds, if the exchange gives one
    pub timestamp: Option<i64>,
}

/// OHLCV candle
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Candle {
    /// Open time in milliseconds
    pub timestamp: i64,
    /// Open price
    pub open: String,
    /// High price
    pub high: String,
    /// Low price
    pub low: String,
    /// Close price
    pub close: String,
    /// Volume in the base asset
    pub volume: String,
}

/// Taker side of a trade
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    /// Taker bought
    Buy,
    /// Taker sold
    Sell,
}

impl fmt::Display for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            Side::Buy => "buy",
            Side::Sell => "sell",
        })
    }
}

/// A public trade
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trade {
    /// Exchange trade id
    pub id: Option<String>,
    /// Trade time in milliseconds
    pub timestamp: i64,
    /// Taker side
    pub side: Side,
    /// Price
    pub price: String,
    /// Amount in the base asset
    pub amount: String,
    /// Value in the quote asset (`price * amount`, exact)
    pub cost: Option<String>,
}

/// Market type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MarketType {
    /// Spot market
    Spot,
    /// Perpetual swap
    Swap,
}

impl fmt::Display for MarketType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.pad(match self {
            MarketType::Spot => "spot",
            MarketType::Swap => "swap",
        })
    }
}

/// A tradable market
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Market {
    /// Unified symbol
    pub symbol: String,
    /// The exchange's own id for it, e.g. `BTCUSDT`, `BTC-USDT`, `BTC`
    pub id: String,
    /// Base asset
    pub base: String,
    /// Quote asset
    pub quote: String,
    /// Spot or swap
    pub market_type: MarketType,
    /// Whether it is currently trading
    pub active: bool,
}

/// Parse a decimal string as `f64` (approximate)
pub fn as_f64(s: &str) -> Option<f64> {
    s.trim().parse::<f64>().ok().filter(|v| v.is_finite())
}

/// Multiply two decimal strings exactly, e.g. `("1.5", "2.25")` → `"3.375"`.
///
/// Returns `None` for malformed input or if the product overflows 38 digits.
pub fn mul_decimal(a: &str, b: &str) -> Option<String> {
    fn parse(s: &str) -> Option<(bool, u128, u32)> {
        let s = s.trim();
        let (neg, s) = match s.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s.strip_prefix('+').unwrap_or(s)),
        };
        let (int, frac) = s.split_once('.').unwrap_or((s, ""));
        if int.is_empty() && frac.is_empty()
            || !int.chars().chain(frac.chars()).all(|c| c.is_ascii_digit())
        {
            return None;
        }
        let digits = format!("{int}{frac}");
        let value = if digits.is_empty() {
            0
        } else {
            digits.parse::<u128>().ok()?
        };
        Some((neg, value, frac.len() as u32))
    }
    let (na, va, sa) = parse(a)?;
    let (nb, vb, sb) = parse(b)?;
    let product = va.checked_mul(vb)?;
    let scale = (sa + sb) as usize;
    let mut digits = product.to_string();
    if digits.len() <= scale {
        digits = format!("{}{digits}", "0".repeat(scale + 1 - digits.len()));
    }
    let (int, frac) = digits.split_at(digits.len() - scale);
    let frac = frac.trim_end_matches('0');
    let neg = na != nb && product != 0;
    Some(format!(
        "{}{int}{}{frac}",
        if neg { "-" } else { "" },
        if frac.is_empty() { "" } else { "." }
    ))
}

/// Percent change from `from` to `to`, e.g. `("100", "101.5")` → `1.5`
pub(crate) fn pct_change(from: &str, to: &str) -> Option<f64> {
    let (from, to) = (as_f64(from)?, as_f64(to)?);
    (from != 0.0).then(|| (to - from) / from * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symbol_parse_and_display() {
        assert_eq!(Symbol::parse("btc/usdt"), Some(Symbol::spot("BTC", "USDT")));
        assert_eq!(
            Symbol::parse("BTC/USDC:USDC"),
            Some(Symbol::swap("BTC", "USDC", "USDC"))
        );
        assert_eq!(Symbol::spot("ETH", "USDT").to_string(), "ETH/USDT");
        assert_eq!(
            Symbol::swap("BTC", "USDC", "USDC").to_string(),
            "BTC/USDC:USDC"
        );
        for bad in ["BTCUSDT", "BTC/", "/USDT", "BTC/US-DT", "BTC/USDT:", ""] {
            assert_eq!(Symbol::parse(bad), None, "{bad}");
        }
    }

    #[test]
    fn timeframe_parse_keeps_month_and_minute_apart() {
        assert_eq!("1m".parse::<Timeframe>(), Ok(Timeframe::M1));
        assert_eq!("1M".parse::<Timeframe>(), Ok(Timeframe::Mon1));
        assert_eq!("1H".parse::<Timeframe>(), Ok(Timeframe::H1));
        assert_eq!("d".parse::<Timeframe>(), Ok(Timeframe::D1));
        assert!("2d".parse::<Timeframe>().is_err());
        assert_eq!(Timeframe::H4.millis(), 4 * 3_600_000);
    }

    #[test]
    fn display_respects_width() {
        assert_eq!(format!("{:<8}|", Exchange::Okx), "okx     |");
        assert_eq!(format!("{:>4}", Side::Buy), " buy");
        assert_eq!(format!("{:<4}|", Timeframe::H1), "1h  |");
    }

    #[test]
    fn exchange_round_trips() {
        for e in Exchange::ALL {
            assert_eq!(e.to_string().parse::<Exchange>(), Ok(e));
        }
        assert_eq!("OKX".parse::<Exchange>(), Ok(Exchange::Okx));
        assert!("kucoin".parse::<Exchange>().is_err());
    }

    #[test]
    fn mul_decimal_is_exact() {
        assert_eq!(mul_decimal("1.5", "2.25").as_deref(), Some("3.375"));
        assert_eq!(mul_decimal("0.1", "0.2").as_deref(), Some("0.02"));
        assert_eq!(
            mul_decimal("2695.85000000", "0.01230000").as_deref(),
            Some("33.158955")
        );
        assert_eq!(mul_decimal("100", "3").as_deref(), Some("300"));
        assert_eq!(mul_decimal("-2", "0.5").as_deref(), Some("-1"));
        assert_eq!(mul_decimal("0", "-5").as_deref(), Some("0"));
        assert_eq!(mul_decimal(".5", "4").as_deref(), Some("2"));
        assert_eq!(mul_decimal("1e3", "2"), None);
        assert_eq!(mul_decimal("", "2"), None);
    }

    #[test]
    fn pct_change_handles_zero_base() {
        assert_eq!(pct_change("100", "101.5"), Some(1.5));
        assert_eq!(pct_change("0", "1"), None);
    }
}
