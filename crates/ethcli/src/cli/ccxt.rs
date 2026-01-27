//! CCXT Cryptocurrency Exchange CLI commands
//!
//! Provides unified access to multiple cryptocurrency exchanges.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

/// Supported exchanges
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum ExchangeId {
    Binance,
    Bitget,
    Okx,
    // Note: Bybit doesn't implement the unified Exchange trait
    Hyperliquid,
}

impl std::fmt::Display for ExchangeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExchangeId::Binance => write!(f, "binance"),
            ExchangeId::Bitget => write!(f, "bitget"),
            ExchangeId::Okx => write!(f, "okx"),
            ExchangeId::Hyperliquid => write!(f, "hyperliquid"),
        }
    }
}

#[derive(Args)]
pub struct CcxtArgs {
    /// Exchange to query
    #[arg(long, short, default_value = "binance")]
    pub exchange: ExchangeId,

    /// Use testnet/sandbox mode
    #[arg(long)]
    pub testnet: bool,

    /// Output format
    #[arg(long, short = 'o', default_value = "table")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum CcxtCommands {
    /// Get ticker for a trading pair
    Ticker {
        /// Trading pair symbol (e.g., BTC/USDT)
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

        /// Timeframe (1m, 5m, 15m, 1h, 4h, 1d)
        #[arg(long, short, default_value = "1h")]
        timeframe: String,

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
        /// Trading pair symbol
        symbol: String,

        /// Output format
        #[arg(long, short = 'o', default_value = "table")]
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
    timestamp: i64,
}

/// Order book entry
#[derive(Debug, Serialize)]
struct OrderBookOutput {
    exchange: String,
    symbol: String,
    bids: Vec<(String, String)>,
    asks: Vec<(String, String)>,
    timestamp: i64,
}

#[derive(Debug, Serialize)]
struct OhlcvOutput {
    exchange: String,
    symbol: String,
    timeframe: String,
    candles: Vec<CandleOutput>,
}

#[derive(Debug, Serialize)]
struct CandleOutput {
    timestamp: i64,
    open: String,
    high: String,
    low: String,
    close: String,
    volume: String,
}

#[derive(Debug, Serialize)]
struct TradeOutput {
    id: Option<String>,
    timestamp: i64,
    side: String,
    price: String,
    amount: String,
    cost: Option<String>,
}

#[derive(Debug, Serialize)]
struct TradesOutput {
    exchange: String,
    symbol: String,
    trades: Vec<TradeOutput>,
}

#[derive(Debug, Serialize)]
struct MarketOutput {
    symbol: String,
    base: String,
    quote: String,
    market_type: String,
    active: bool,
}

/// Create an exchange and load markets
macro_rules! create_exchange {
    ($exchange_type:ty, $sandbox:expr) => {{
        let exchange = <$exchange_type>::builder()
            .sandbox($sandbox)
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to create exchange: {:?}", e))?;
        exchange
            .load_markets(false)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to load markets: {:?}", e))?;
        exchange
    }};
}

/// Handle CCXT commands
pub async fn handle(command: &CcxtCommands, quiet: bool) -> anyhow::Result<()> {
    use ccxt_rust::prelude::{Binance, Bitget, Exchange as ExchangeTrait, HyperLiquid, Okx};

    match command {
        CcxtCommands::Ticker { symbol, args } => {
            if !quiet {
                eprintln!("Fetching ticker for {} from {}...", symbol, args.exchange);
            }

            let ticker = match args.exchange {
                ExchangeId::Binance => {
                    let exchange = create_exchange!(Binance, args.testnet);
                    ExchangeTrait::fetch_ticker(&exchange, symbol)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch ticker: {:?}", e))?
                }
                ExchangeId::Bitget => {
                    let exchange = create_exchange!(Bitget, args.testnet);
                    ExchangeTrait::fetch_ticker(&exchange, symbol)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch ticker: {:?}", e))?
                }
                ExchangeId::Okx => {
                    let exchange = create_exchange!(Okx, args.testnet);
                    ExchangeTrait::fetch_ticker(&exchange, symbol)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch ticker: {:?}", e))?
                }
                ExchangeId::Hyperliquid => {
                    let exchange = create_exchange!(HyperLiquid, args.testnet);
                    ExchangeTrait::fetch_ticker(&exchange, symbol)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch ticker: {:?}", e))?
                }
            };

            let output = TickerOutput {
                exchange: args.exchange.to_string(),
                symbol: ticker.symbol.to_string(),
                last: ticker.last.map(|p| p.to_string()),
                bid: ticker.bid.map(|p| p.to_string()),
                ask: ticker.ask.map(|p| p.to_string()),
                high: ticker.high.map(|p| p.to_string()),
                low: ticker.low.map(|p| p.to_string()),
                volume: ticker.base_volume.map(|v| v.to_string()),
                change_24h: ticker.percentage.map(|p| format!("{}%", p)),
                timestamp: ticker.timestamp,
            };

            print_output(&output, args.format)?;
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

            // Helper to fetch multiple tickers using a created exchange
            macro_rules! fetch_tickers {
                ($exchange:expr) => {{
                    for symbol in symbols {
                        if let Ok(t) = ExchangeTrait::fetch_ticker(&$exchange, symbol).await {
                            outputs.push(TickerOutput {
                                exchange: args.exchange.to_string(),
                                symbol: t.symbol.to_string(),
                                last: t.last.map(|p| p.to_string()),
                                bid: t.bid.map(|p| p.to_string()),
                                ask: t.ask.map(|p| p.to_string()),
                                high: t.high.map(|p| p.to_string()),
                                low: t.low.map(|p| p.to_string()),
                                volume: t.base_volume.map(|v| v.to_string()),
                                change_24h: t.percentage.map(|p| format!("{}%", p)),
                                timestamp: t.timestamp,
                            });
                        }
                    }
                }};
            }

            match args.exchange {
                ExchangeId::Binance => {
                    let exchange = create_exchange!(Binance, args.testnet);
                    fetch_tickers!(exchange);
                }
                ExchangeId::Bitget => {
                    let exchange = create_exchange!(Bitget, args.testnet);
                    fetch_tickers!(exchange);
                }
                ExchangeId::Okx => {
                    let exchange = create_exchange!(Okx, args.testnet);
                    fetch_tickers!(exchange);
                }
                ExchangeId::Hyperliquid => {
                    let exchange = create_exchange!(HyperLiquid, args.testnet);
                    fetch_tickers!(exchange);
                }
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

            let orderbook = match args.exchange {
                ExchangeId::Binance => {
                    let exchange = create_exchange!(Binance, args.testnet);
                    ExchangeTrait::fetch_order_book(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch order book: {:?}", e))?
                }
                ExchangeId::Bitget => {
                    let exchange = create_exchange!(Bitget, args.testnet);
                    ExchangeTrait::fetch_order_book(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch order book: {:?}", e))?
                }
                ExchangeId::Okx => {
                    let exchange = create_exchange!(Okx, args.testnet);
                    ExchangeTrait::fetch_order_book(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch order book: {:?}", e))?
                }
                ExchangeId::Hyperliquid => {
                    let exchange = create_exchange!(HyperLiquid, args.testnet);
                    ExchangeTrait::fetch_order_book(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch order book: {:?}", e))?
                }
            };

            let output = OrderBookOutput {
                exchange: args.exchange.to_string(),
                symbol: orderbook.symbol.to_string(),
                bids: orderbook
                    .bids
                    .iter()
                    .map(|e| (e.price.to_string(), e.amount.to_string()))
                    .collect(),
                asks: orderbook
                    .asks
                    .iter()
                    .map(|e| (e.price.to_string(), e.amount.to_string()))
                    .collect(),
                timestamp: orderbook.timestamp,
            };

            match args.format {
                OutputFormat::Table => {
                    println!("\nOrder Book for {} on {}", symbol, args.exchange);
                    println!("{}", "=".repeat(60));
                    println!("{:<30} {:>30}", "BIDS (Buy Orders)", "ASKS (Sell Orders)");
                    println!("{}", "-".repeat(60));

                    let max_len = output.bids.len().max(output.asks.len());
                    for i in 0..max_len {
                        let bid = output
                            .bids
                            .get(i)
                            .map(|(p, a)| format!("{} @ {}", a, p))
                            .unwrap_or_default();
                        let ask = output
                            .asks
                            .get(i)
                            .map(|(p, a)| format!("{} @ {}", a, p))
                            .unwrap_or_default();
                        println!("{:<30} {:>30}", bid, ask);
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

            let tf = parse_timeframe(timeframe)?;

            let candles = match args.exchange {
                ExchangeId::Binance => {
                    let exchange = create_exchange!(Binance, args.testnet);
                    ExchangeTrait::fetch_ohlcv(&exchange, symbol, tf, None, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV: {:?}", e))?
                }
                ExchangeId::Bitget => {
                    let exchange = create_exchange!(Bitget, args.testnet);
                    ExchangeTrait::fetch_ohlcv(&exchange, symbol, tf, None, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV: {:?}", e))?
                }
                ExchangeId::Okx => {
                    let exchange = create_exchange!(Okx, args.testnet);
                    ExchangeTrait::fetch_ohlcv(&exchange, symbol, tf, None, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV: {:?}", e))?
                }
                ExchangeId::Hyperliquid => {
                    let exchange = create_exchange!(HyperLiquid, args.testnet);
                    ExchangeTrait::fetch_ohlcv(&exchange, symbol, tf, None, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV: {:?}", e))?
                }
            };

            let output = OhlcvOutput {
                exchange: args.exchange.to_string(),
                symbol: symbol.clone(),
                timeframe: timeframe.clone(),
                candles: candles
                    .into_iter()
                    .map(|c| CandleOutput {
                        timestamp: c.timestamp,
                        open: c.open.to_string(),
                        high: c.high.to_string(),
                        low: c.low.to_string(),
                        close: c.close.to_string(),
                        volume: c.volume.to_string(),
                    })
                    .collect(),
            };

            match args.format {
                OutputFormat::Table => {
                    println!("\nOHLCV for {} on {} ({})", symbol, args.exchange, timeframe);
                    println!("{}", "=".repeat(80));
                    println!(
                        "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12}",
                        "Time", "Open", "High", "Low", "Close", "Volume"
                    );
                    println!("{}", "-".repeat(80));

                    for c in &output.candles {
                        let time = chrono::DateTime::from_timestamp(c.timestamp / 1000, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| c.timestamp.to_string());
                        println!(
                            "{:<20} {:>12} {:>12} {:>12} {:>12} {:>12}",
                            time,
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

            let trades = match args.exchange {
                ExchangeId::Binance => {
                    let exchange = create_exchange!(Binance, args.testnet);
                    ExchangeTrait::fetch_trades(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch trades: {:?}", e))?
                }
                ExchangeId::Bitget => {
                    let exchange = create_exchange!(Bitget, args.testnet);
                    ExchangeTrait::fetch_trades(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch trades: {:?}", e))?
                }
                ExchangeId::Okx => {
                    let exchange = create_exchange!(Okx, args.testnet);
                    ExchangeTrait::fetch_trades(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch trades: {:?}", e))?
                }
                ExchangeId::Hyperliquid => {
                    let exchange = create_exchange!(HyperLiquid, args.testnet);
                    ExchangeTrait::fetch_trades(&exchange, symbol, Some(*limit))
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch trades: {:?}", e))?
                }
            };

            let output = TradesOutput {
                exchange: args.exchange.to_string(),
                symbol: symbol.clone(),
                trades: trades
                    .into_iter()
                    .map(|t| TradeOutput {
                        id: t.id.map(|id| id.to_string()),
                        timestamp: t.timestamp,
                        side: format!("{:?}", t.side).to_lowercase(),
                        price: t.price.to_string(),
                        amount: t.amount.to_string(),
                        cost: t.cost.map(|c| c.to_string()),
                    })
                    .collect(),
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
                        let time = chrono::DateTime::from_timestamp(t.timestamp / 1000, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| t.timestamp.to_string());
                        println!(
                            "{:<20} {:>8} {:>15} {:>15} {:>15}",
                            time,
                            t.side,
                            truncate_num(&t.price, 15),
                            truncate_num(&t.amount, 15),
                            t.cost.as_deref().map(|c| truncate_num(c, 15)).unwrap_or_else(|| "-".to_string())
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

            let markets = match args.exchange {
                ExchangeId::Binance => {
                    let exchange = create_exchange!(Binance, args.testnet);
                    ExchangeTrait::fetch_markets(&exchange)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch markets: {:?}", e))?
                }
                ExchangeId::Bitget => {
                    let exchange = create_exchange!(Bitget, args.testnet);
                    ExchangeTrait::fetch_markets(&exchange)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch markets: {:?}", e))?
                }
                ExchangeId::Okx => {
                    let exchange = create_exchange!(Okx, args.testnet);
                    ExchangeTrait::fetch_markets(&exchange)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch markets: {:?}", e))?
                }
                ExchangeId::Hyperliquid => {
                    let exchange = create_exchange!(HyperLiquid, args.testnet);
                    ExchangeTrait::fetch_markets(&exchange)
                        .await
                        .map_err(|e| anyhow::anyhow!("Failed to fetch markets: {:?}", e))?
                }
            };

            // Filter markets
            let mut output: Vec<MarketOutput> = markets
                .into_iter()
                .filter(|m| {
                    let quote_match = quote
                        .as_ref()
                        .map(|q| m.quote.eq_ignore_ascii_case(q))
                        .unwrap_or(true);
                    let base_match = base
                        .as_ref()
                        .map(|b| m.base.eq_ignore_ascii_case(b))
                        .unwrap_or(true);
                    quote_match && base_match
                })
                .map(|m| MarketOutput {
                    symbol: m.symbol.to_string(),
                    base: m.base.to_string(),
                    quote: m.quote.to_string(),
                    market_type: format!("{:?}", m.market_type).to_lowercase(),
                    active: m.active,
                })
                .collect();

            // Sort by symbol
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
                            base.as_ref().map(|b| format!("base={}", b)).unwrap_or_default(),
                            quote.as_ref().map(|q| format!(" quote={}", q)).unwrap_or_default()
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

            let mut results: Vec<TickerOutput> = Vec::new();

            // Fetch from each exchange, collecting successes
            macro_rules! try_fetch {
                ($exchange_type:ty, $name:expr) => {{
                    match <$exchange_type>::builder().build() {
                        Ok(exchange) => {
                            if exchange.load_markets(false).await.is_ok() {
                                if let Ok(t) = ExchangeTrait::fetch_ticker(&exchange, symbol).await
                                {
                                    results.push(TickerOutput {
                                        exchange: $name.to_string(),
                                        symbol: t.symbol.to_string(),
                                        last: t.last.map(|p| p.to_string()),
                                        bid: t.bid.map(|p| p.to_string()),
                                        ask: t.ask.map(|p| p.to_string()),
                                        high: t.high.map(|p| p.to_string()),
                                        low: t.low.map(|p| p.to_string()),
                                        volume: t.base_volume.map(|v| v.to_string()),
                                        change_24h: t.percentage.map(|p| format!("{}%", p)),
                                        timestamp: t.timestamp,
                                    });
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }};
            }

            try_fetch!(Binance, "binance");
            try_fetch!(Bitget, "bitget");
            try_fetch!(Okx, "okx");
            try_fetch!(HyperLiquid, "hyperliquid");

            match format {
                OutputFormat::Table => {
                    println!("\nPrice Comparison for {}", symbol);
                    println!("{}", "=".repeat(80));
                    println!(
                        "{:<15} {:>15} {:>15} {:>15} {:>15}",
                        "Exchange", "Last", "Bid", "Ask", "24h Change"
                    );
                    println!("{}", "-".repeat(80));

                    for r in &results {
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
                _ => print_output(&results, *format)?,
            }
        }
    }

    Ok(())
}

fn print_output<T: serde::Serialize>(data: &T, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(data)?);
        }
        OutputFormat::Table => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}

/// Truncate a numeric string to fit in a column width
fn truncate_num(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        // Try to preserve meaningful digits by trimming trailing zeros after decimal
        let truncated = &s[..max_len.saturating_sub(2)];
        format!("{}…", truncated)
    }
}

/// Parse a timeframe string into the ccxt Timeframe enum
fn parse_timeframe(s: &str) -> anyhow::Result<ccxt_rust::Timeframe> {
    use ccxt_rust::Timeframe;
    match s.to_lowercase().as_str() {
        "1m" => Ok(Timeframe::M1),
        "3m" => Ok(Timeframe::M3),
        "5m" => Ok(Timeframe::M5),
        "15m" => Ok(Timeframe::M15),
        "30m" => Ok(Timeframe::M30),
        "1h" => Ok(Timeframe::H1),
        "2h" => Ok(Timeframe::H2),
        "4h" => Ok(Timeframe::H4),
        "6h" => Ok(Timeframe::H6),
        "8h" => Ok(Timeframe::H8),
        "12h" => Ok(Timeframe::H12),
        "1d" | "d" => Ok(Timeframe::D1),
        "3d" => Ok(Timeframe::D3),
        "1w" | "w" => Ok(Timeframe::W1),
        "1M" | "M" => Ok(Timeframe::Mon1),
        _ => anyhow::bail!("Invalid timeframe: {}. Valid: 1m, 5m, 15m, 30m, 1h, 4h, 1d, 1w", s),
    }
}
