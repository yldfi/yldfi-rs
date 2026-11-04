//! Centralised exchange market data commands (`ethcli ccxt`, alias `cex`)
//!
//! Backed by the `cexm` crate: Binance, Bitget, OKX, Bybit, Coinbase and
//! Kraken spot, plus Hyperliquid and Lighter perpetuals.

use crate::cli::OutputFormat;
use cexm::{Client, Exchange, Ticker, Timeframe};
use clap::builder::{PossibleValuesParser, TypedValueParser};
use clap::{Args, Subcommand};
use serde::Serialize;

/// Parses `--exchange` against every exchange `cexm` supports
fn exchange_parser() -> impl TypedValueParser<Value = Exchange> {
    PossibleValuesParser::new(Exchange::ALL.map(Exchange::as_str)).map(|s| {
        s.parse::<Exchange>()
            .expect("value is one of Exchange::ALL")
    })
}

#[derive(Args)]
pub struct CcxtArgs {
    /// Exchange to query
    #[arg(long, short, default_value = "binance", value_parser = exchange_parser())]
    pub exchange: Exchange,

    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum CcxtCommands {
    /// Get ticker for a trading pair
    Ticker {
        /// Trading pair symbol (e.g., BTC/USDT; perpetuals: BTC/USDC:USDC)
        symbol: String,

        #[command(flatten)]
        args: CcxtArgs,
    },

    /// Get tickers for multiple pairs
    Tickers {
        /// Trading pair symbols (comma-separated, e.g., BTC/USDT,ETH/USDT)
        #[arg(value_delimiter = ',')]
        symbols: Vec<String>,

        #[command(flatten)]
        args: CcxtArgs,
    },

    /// Get order book for a trading pair
    OrderBook {
        /// Trading pair symbol
        symbol: String,

        /// Depth limit (number of levels)
        #[arg(long, default_value = "10")]
        limit: u32,

        #[command(flatten)]
        args: CcxtArgs,
    },

    /// Get OHLCV candlestick data
    Ohlcv {
        /// Trading pair symbol
        symbol: String,

        /// Timeframe (1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M)
        #[arg(long, short, default_value = "1h")]
        timeframe: Timeframe,

        /// Number of candles to fetch
        #[arg(long, default_value = "100")]
        limit: u32,

        #[command(flatten)]
        args: CcxtArgs,
    },

    /// Get recent trades for a trading pair
    Trades {
        /// Trading pair symbol
        symbol: String,

        /// Number of trades to fetch
        #[arg(long, default_value = "50")]
        limit: u32,

        #[command(flatten)]
        args: CcxtArgs,
    },

    /// List available markets/trading pairs
    Markets {
        /// Filter by quote currency (e.g., USDT)
        #[arg(long)]
        quote: Option<String>,

        /// Filter by base currency (e.g., BTC)
        #[arg(long)]
        base: Option<String>,

        #[command(flatten)]
        args: CcxtArgs,
    },

    /// Compare prices across exchanges
    Compare {
        /// Trading pair symbol (each exchange is asked for exactly this pair)
        symbol: String,

        /// Output format
        #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
        format: OutputFormat,
    },
}

/// Ticker output for display
#[derive(Debug, Serialize)]
struct TickerOutput {
    exchange: String,
    symbol: String,
    last: Option<String>,
    bid: Option<String>,
    ask: Option<String>,
    high: Option<String>,
    low: Option<String>,
    volume: Option<String>,
    change_24h: Option<String>,
    timestamp: Option<i64>,
}

impl From<Ticker> for TickerOutput {
    fn from(t: Ticker) -> Self {
        Self {
            exchange: t.exchange.to_string(),
            symbol: t.symbol,
            last: t.last,
            bid: t.bid,
            ask: t.ask,
            high: t.high,
            low: t.low,
            volume: t.base_volume,
            change_24h: t.change_pct.map(|p| format!("{p:.4}%")),
            timestamp: t.timestamp,
        }
    }
}

/// Order book entry
#[derive(Debug, Serialize)]
struct OrderBookOutput {
    exchange: String,
    symbol: String,
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
    timestamp: Option<i64>,
}

#[derive(Debug, Serialize)]
struct OhlcvOutput {
    exchange: String,
    symbol: String,
    timeframe: String,
    candles: Vec<cexm::Candle>,
}

#[derive(Debug, Serialize)]
struct TradesOutput {
    exchange: String,
    symbol: String,
    trades: Vec<cexm::Trade>,
}

#[derive(Debug, Serialize)]
struct MarketOutput {
    symbol: String,
    base: String,
    quote: String,
    market_type: String,
    active: bool,
}

/// Handle exchange commands
pub async fn handle(command: &CcxtCommands, quiet: bool) -> anyhow::Result<()> {
    let client = Client::new()?;

    match command {
        CcxtCommands::Ticker { symbol, args } => {
            if !quiet {
                eprintln!("Fetching ticker for {} from {}...", symbol, args.exchange);
            }
            let ticker = client.ticker(args.exchange, symbol).await?;
            print_output(&TickerOutput::from(ticker), args.format)?;
        }

        CcxtCommands::Tickers { symbols, args } => {
            if !quiet {
                eprintln!(
                    "Fetching {} tickers from {}...",
                    symbols.len(),
                    args.exchange
                );
            }
            let mut outputs = Vec::new();
            let mut failures: Vec<String> = Vec::new();
            for symbol in symbols {
                match client.ticker(args.exchange, symbol).await {
                    Ok(t) => outputs.push(TickerOutput::from(t)),
                    Err(e) => {
                        eprintln!("Warning: {} failed: {}", symbol, e);
                        failures.push(symbol.to_string());
                    }
                }
            }
            if outputs.is_empty() && !failures.is_empty() {
                anyhow::bail!("No ticker could be fetched ({})", failures.join(", "));
            }
            print_output(&outputs, args.format)?;
        }

        CcxtCommands::OrderBook {
            symbol,
            limit,
            args,
        } => {
            if !quiet {
                eprintln!(
                    "Fetching order book for {} from {} (depth: {})...",
                    symbol, args.exchange, limit
                );
            }
            let book = client.order_book(args.exchange, symbol, *limit).await?;
            let pairs = |levels: Vec<cexm::Level>| -> Vec<(String, String)> {
                levels.into_iter().map(|l| (l.price, l.amount)).collect()
            };
            let output = OrderBookOutput {
                exchange: args.exchange.to_string(),
                symbol: book.symbol,
                bids: pairs(book.bids),
                asks: pairs(book.asks),
                timestamp: book.timestamp,
            };

            match args.format {
                OutputFormat::Table => {
                    println!("\nOrder Book for {} on {}", symbol, args.exchange);
                    println!("{}", "=".repeat(60));
                    println!("{:<30} {:>30}", "BIDS (Buy Orders)", "ASKS (Sell Orders)");
                    println!("{}", "-".repeat(60));

                    let max_len = output.bids.len().max(output.asks.len());
                    for i in 0..max_len {
                        let side = |levels: &[(String, String)]| {
                            levels
                                .get(i)
                                .map(|(p, a)| format!("{} @ {}", a, p))
                                .unwrap_or_default()
                        };
                        println!("{:<30} {:>30}", side(&output.bids), side(&output.asks));
                    }
                }
                _ => print_output(&output, args.format)?,
            }
        }

        CcxtCommands::Ohlcv {
            symbol,
            timeframe,
            limit,
            args,
        } => {
            if !quiet {
                eprintln!(
                    "Fetching {} {} candles for {} from {}...",
                    limit, timeframe, symbol, args.exchange
                );
            }
            let candles = client
                .candles(args.exchange, symbol, *timeframe, *limit)
                .await?;
            let output = OhlcvOutput {
                exchange: args.exchange.to_string(),
                symbol: symbol.clone(),
                timeframe: timeframe.to_string(),
                candles,
            };

            match args.format {
                OutputFormat::Table => {
                    println!(
                        "\nOHLCV for {} on {} ({})",
                        symbol, args.exchange, timeframe
                    );
                    println!("{}", "=".repeat(80));
                    println!(
                        "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12}",
                        "Time", "Open", "High", "Low", "Close", "Volume"
                    );
                    println!("{}", "-".repeat(80));

                    for c in &output.candles {
                        println!(
                            "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12}",
                            format_time(c.timestamp, "%Y-%m-%d %H:%M"),
                            truncate_num(&c.open, 12),
                            truncate_num(&c.high, 12),
                            truncate_num(&c.low, 12),
                            truncate_num(&c.close, 12),
                            truncate_num(&c.volume, 12)
                        );
                    }
                }
                _ => print_output(&output, args.format)?,
            }
        }

        CcxtCommands::Trades {
            symbol,
            limit,
            args,
        } => {
            if !quiet {
                eprintln!(
                    "Fetching {} recent trades for {} from {}...",
                    limit, symbol, args.exchange
                );
            }
            let trades = client.trades(args.exchange, symbol, *limit).await?;
            let output = TradesOutput {
                exchange: args.exchange.to_string(),
                symbol: symbol.clone(),
                trades,
            };

            match args.format {
                OutputFormat::Table => {
                    println!("\nRecent Trades for {} on {}", symbol, args.exchange);
                    println!("{}", "=".repeat(80));
                    println!(
                        "{:<20} {:>8} {:>15} {:>15} {:>15}",
                        "Time", "Side", "Price", "Amount", "Cost"
                    );
                    println!("{}", "-".repeat(80));

                    for t in &output.trades {
                        println!(
                            "{:<20} {:>8} {:>15} {:>15} {:>15}",
                            format_time(t.timestamp, "%Y-%m-%d %H:%M:%S"),
                            t.side,
                            truncate_num(&t.price, 15),
                            truncate_num(&t.amount, 15),
                            t.cost
                                .as_deref()
                                .map(|c| truncate_num(c, 15))
                                .unwrap_or_else(|| "-".to_string())
                        );
                    }
                }
                _ => print_output(&output, args.format)?,
            }
        }

        CcxtCommands::Markets { quote, base, args } => {
            if !quiet {
                eprintln!("Fetching markets from {}...", args.exchange);
            }
            let matches = |filter: &Option<String>, value: &str| {
                filter
                    .as_ref()
                    .is_none_or(|f| value.eq_ignore_ascii_case(f))
            };
            let mut output: Vec<MarketOutput> = client
                .markets(args.exchange)
                .await?
                .into_iter()
                .filter(|m| matches(quote, &m.quote) && matches(base, &m.base))
                .map(|m| MarketOutput {
                    symbol: m.symbol,
                    base: m.base,
                    quote: m.quote,
                    market_type: m.market_type.to_string(),
                    active: m.active,
                })
                .collect();
            output.sort_by(|a, b| a.symbol.cmp(&b.symbol));

            if !quiet {
                eprintln!("Found {} markets", output.len());
            }

            match args.format {
                OutputFormat::Table => {
                    println!("\nMarkets on {}", args.exchange);
                    if quote.is_some() || base.is_some() {
                        println!(
                            "Filtered by: {}{}",
                            base.as_ref()
                                .map(|b| format!("base={}", b))
                                .unwrap_or_default(),
                            quote
                                .as_ref()
                                .map(|q| format!(" quote={}", q))
                                .unwrap_or_default()
                        );
                    }
                    println!("{}", "=".repeat(70));
                    println!(
                        "{:<20} {:>10} {:>10} {:>12} {:>10}",
                        "Symbol", "Base", "Quote", "Type", "Active"
                    );
                    println!("{}", "-".repeat(70));

                    for m in &output {
                        println!(
                            "{:<20} {:>10} {:>10} {:>12} {:>10}",
                            m.symbol,
                            m.base,
                            m.quote,
                            m.market_type,
                            if m.active { "Yes" } else { "No" }
                        );
                    }
                    println!("{}", "-".repeat(70));
                    println!("Total: {} markets", output.len());
                }
                _ => print_output(&output, args.format)?,
            }
        }

        CcxtCommands::Compare { symbol, format } => {
            if !quiet {
                eprintln!("Comparing {} prices across exchanges...", symbol);
            }
            let client = &client;
            let results = futures::future::join_all(
                Exchange::ALL.map(|ex| async move { (ex, client.ticker(ex, symbol).await) }),
            )
            .await;

            // Report failures instead of silently dropping the exchange
            let mut outputs: Vec<TickerOutput> = Vec::new();
            for (exchange, result) in results {
                match result {
                    Ok(t) => outputs.push(t.into()),
                    Err(e) if !quiet => eprintln!("Note: {} skipped: {}", exchange, e),
                    Err(_) => {}
                }
            }
            if outputs.is_empty() {
                anyhow::bail!("No exchange returned a ticker for {}", symbol);
            }

            match format {
                OutputFormat::Table => {
                    println!("\nPrice Comparison for {}", symbol);
                    println!("{}", "=".repeat(80));
                    println!(
                        "{:<15} {:>15} {:>15} {:>15} {:>15}",
                        "Exchange", "Last", "Bid", "Ask", "24h Change"
                    );
                    println!("{}", "-".repeat(80));

                    for r in &outputs {
                        println!(
                            "{:<15} {:>15} {:>15} {:>15} {:>15}",
                            r.exchange,
                            r.last.as_deref().unwrap_or("-"),
                            r.bid.as_deref().unwrap_or("-"),
                            r.ask.as_deref().unwrap_or("-"),
                            r.change_24h.as_deref().unwrap_or("-")
                        );
                    }
                }
                _ => print_output(&outputs, *format)?,
            }
        }
    }

    Ok(())
}

fn print_output<T: serde::Serialize>(data: &T, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json | OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(data)?);
        }
    }
    Ok(())
}

/// Millisecond timestamp as UTC time, or the raw number if out of range
fn format_time(millis: i64, fmt: &str) -> String {
    chrono::DateTime::from_timestamp_millis(millis)
        .map(|dt| dt.format(fmt).to_string())
        .unwrap_or_else(|| millis.to_string())
}

/// Truncate a numeric string to fit in a column width
fn truncate_num(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len.saturating_sub(2)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[derive(Parser)]
    struct Cli {
        #[command(subcommand)]
        command: CcxtCommands,
    }

    #[test]
    fn every_cexm_exchange_is_accepted() {
        for ex in Exchange::ALL {
            let cli = Cli::try_parse_from(["t", "ticker", "BTC/USDT", "-e", ex.as_str()]).unwrap();
            match cli.command {
                CcxtCommands::Ticker { args, .. } => assert_eq!(args.exchange, ex),
                _ => unreachable!(),
            }
        }
        assert!(Cli::try_parse_from(["t", "ticker", "BTC/USDT", "-e", "kucoin"]).is_err());
    }

    #[test]
    fn timeframe_keeps_month_and_minute_apart() {
        let tf = |s: &str| match Cli::try_parse_from(["t", "ohlcv", "BTC/USDT", "-t", s])
            .unwrap()
            .command
        {
            CcxtCommands::Ohlcv { timeframe, .. } => timeframe,
            _ => unreachable!(),
        };
        assert_eq!(tf("1M"), Timeframe::Mon1);
        assert_eq!(tf("1m"), Timeframe::M1);
    }

    #[test]
    fn testnet_flag_is_gone() {
        assert!(Cli::try_parse_from(["t", "ticker", "BTC/USDT", "--testnet"]).is_err());
    }

    #[test]
    fn ticker_output_formats_change() {
        let out = TickerOutput::from(Ticker {
            exchange: Exchange::Okx,
            symbol: "ETH/USDT".into(),
            last: Some("2335.37".into()),
            bid: None,
            ask: None,
            high: None,
            low: None,
            base_volume: None,
            change_pct: Some(3.09412),
            timestamp: None,
        });
        assert_eq!(out.change_24h.as_deref(), Some("3.0941%"));
        assert_eq!(out.exchange, "okx");
    }
}
