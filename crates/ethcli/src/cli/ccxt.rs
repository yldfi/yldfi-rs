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
    #[arg(long, short = 'o', visible_alias = "output", default_value = "table")]
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

    if let Some(args) = command.exchange_args() {
        if args.exchange == ExchangeId::Hyperliquid {
            anyhow::bail!("{}", HYPERLIQUID_UNSUPPORTED);
        }
    }

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
                change_24h: ticker
                    .percentage
                    .map(|p| format_change_pct(args.exchange, p)),
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
                                change_24h: t
                                    .percentage
                                    .map(|p| format_change_pct(args.exchange, p)),
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

            let candles: Vec<CandleOutput> = match args.exchange {
                ExchangeId::Binance => {
                    let exchange = create_exchange!(Binance, args.testnet);
                    candles_to_output(
                        ExchangeTrait::fetch_ohlcv(&exchange, symbol, tf, None, Some(*limit))
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV: {:?}", e))?,
                    )
                }
                ExchangeId::Bitget => {
                    // ccxt-rust 0.1.5 sends v1-style granularities ("1H", "1D")
                    // that Bitget's v2 API rejects (code 400171); query v2 directly.
                    if args.testnet {
                        anyhow::bail!("Bitget OHLCV is not available in --testnet mode");
                    }
                    fetch_bitget_ohlcv(symbol, timeframe, *limit).await?
                }
                ExchangeId::Okx => {
                    let exchange = create_exchange!(Okx, args.testnet);
                    candles_to_output(
                        ExchangeTrait::fetch_ohlcv(&exchange, symbol, tf, None, Some(*limit))
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV: {:?}", e))?,
                    )
                }
                ExchangeId::Hyperliquid => {
                    let exchange = create_exchange!(HyperLiquid, args.testnet);
                    candles_to_output(
                        ExchangeTrait::fetch_ohlcv(&exchange, symbol, tf, None, Some(*limit))
                            .await
                            .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV: {:?}", e))?,
                    )
                }
            };

            let output = OhlcvOutput {
                exchange: args.exchange.to_string(),
                symbol: symbol.clone(),
                timeframe: timeframe.clone(),
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

            let mut results: Vec<TickerOutput> = Vec::new();

            // Fetch from each exchange, collecting successes
            macro_rules! try_fetch {
                ($exchange_type:ty, $name:expr, $id:expr) => {{
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
                                        change_24h: t.percentage.map(|p| format_change_pct($id, p)),
                                        timestamp: t.timestamp,
                                    });
                                }
                            }
                        }
                        Err(_) => {}
                    }
                }};
            }

            try_fetch!(Binance, "binance", ExchangeId::Binance);
            try_fetch!(Bitget, "bitget", ExchangeId::Bitget);
            try_fetch!(Okx, "okx", ExchangeId::Okx);
            // Hyperliquid is skipped: see HYPERLIQUID_UNSUPPORTED.
            if !quiet {
                eprintln!("Note: hyperliquid skipped ({})", HYPERLIQUID_UNSUPPORTED);
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

/// Why Hyperliquid is disabled.
///
/// ccxt-rust 0.1.5 (latest release) posts JSON bodies without a
/// `Content-Type: application/json` header (ccxt-core http_client/request.rs),
/// which Hyperliquid's /info endpoint rejects with HTTP 415 for every call.
/// Upstream issue to file against ccxt-rust: set the JSON content type in
/// `HttpClient::fetch_once` when a body is present.
const HYPERLIQUID_UNSUPPORTED: &str =
    "Hyperliquid is temporarily unsupported: the ccxt-rust 0.1.5 \
     HTTP client omits the JSON Content-Type header and Hyperliquid rejects every request with \
     HTTP 415. Use --exchange binance, okx or bitget instead.";

impl CcxtCommands {
    /// Shared exchange args, if the command targets a single exchange.
    fn exchange_args(&self) -> Option<&CcxtArgs> {
        match self {
            CcxtCommands::Ticker { args, .. }
            | CcxtCommands::Tickers { args, .. }
            | CcxtCommands::OrderBook { args, .. }
            | CcxtCommands::Ohlcv { args, .. }
            | CcxtCommands::Trades { args, .. }
            | CcxtCommands::Markets { args, .. } => Some(args),
            CcxtCommands::Compare { .. } => None,
        }
    }
}

/// Normalize a ccxt ticker `percentage` to a percent value.
///
/// ccxt-rust 0.1.5 maps Bitget's `changeUtc24h` (a ratio, e.g. `0.00301`)
/// straight into `percentage`, so Bitget values are 100x too small.
pub(crate) fn change_pct_value(exchange: ExchangeId, raw: f64) -> f64 {
    match exchange {
        ExchangeId::Bitget => raw * 100.0,
        _ => raw,
    }
}

fn format_change_pct<D>(exchange: ExchangeId, raw: D) -> String
where
    D: num_traits::ToPrimitive + std::fmt::Display,
{
    match raw.to_f64() {
        Some(v) => format!("{:.4}%", change_pct_value(exchange, v)),
        None => format!("{}%", raw),
    }
}

fn candles_to_output(candles: Vec<ccxt_rust::prelude::Ohlcv>) -> Vec<CandleOutput> {
    candles
        .into_iter()
        .map(|c| CandleOutput {
            timestamp: c.timestamp,
            open: c.open.to_string(),
            high: c.high.to_string(),
            low: c.low.to_string(),
            close: c.close.to_string(),
            volume: c.volume.to_string(),
        })
        .collect()
}

/// Map a CLI timeframe to a Bitget v2 spot candle granularity.
fn bitget_v2_granularity(timeframe: &str) -> anyhow::Result<&'static str> {
    Ok(match timeframe {
        "1M" => "1M",
        tf => match tf.to_lowercase().as_str() {
            "1m" => "1min",
            "3m" => "3min",
            "5m" => "5min",
            "15m" => "15min",
            "30m" => "30min",
            "1h" => "1h",
            "4h" => "4h",
            "6h" => "6h",
            "12h" => "12h",
            "1d" | "d" => "1day",
            "3d" => "3day",
            "1w" | "w" => "1week",
            other => anyhow::bail!(
                "Timeframe {} is not supported by Bitget. Valid: 1m, 3m, 5m, 15m, 30m, 1h, 4h, 6h, 12h, 1d, 3d, 1w, 1M",
                other
            ),
        },
    })
}

/// Parse Bitget v2 candles: `[[ts, open, high, low, close, baseVol, usdtVol, quoteVol], ...]`
fn parse_bitget_candles(body: &serde_json::Value) -> anyhow::Result<Vec<CandleOutput>> {
    let code = body.get("code").and_then(|c| c.as_str()).unwrap_or("");
    if code != "00000" {
        anyhow::bail!(
            "Bitget error {}: {}",
            code,
            body.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
        );
    }
    let rows = body
        .get("data")
        .and_then(|d| d.as_array())
        .ok_or_else(|| anyhow::anyhow!("Bitget response missing data array"))?;
    let field = |row: &[serde_json::Value], i: usize| -> String {
        row.get(i)
            .map(|v| {
                v.as_str()
                    .map(str::to_string)
                    .unwrap_or_else(|| v.to_string())
            })
            .unwrap_or_default()
    };
    let mut candles = rows
        .iter()
        .filter_map(|r| r.as_array())
        .map(|row| {
            let timestamp = field(row, 0)
                .parse::<i64>()
                .map_err(|e| anyhow::anyhow!("invalid Bitget candle timestamp: {e}"))?;
            Ok(CandleOutput {
                timestamp,
                open: field(row, 1),
                high: field(row, 2),
                low: field(row, 3),
                close: field(row, 4),
                volume: field(row, 5),
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    candles.sort_by_key(|c| c.timestamp);
    Ok(candles)
}

/// Fetch OHLCV directly from Bitget's v2 spot API.
async fn fetch_bitget_ohlcv(
    symbol: &str,
    timeframe: &str,
    limit: u32,
) -> anyhow::Result<Vec<CandleOutput>> {
    let granularity = bitget_v2_granularity(timeframe)?;
    let market_id: String = symbol
        .split(':')
        .next()
        .unwrap_or(symbol)
        .chars()
        .filter(|c| *c != '/')
        .collect::<String>()
        .to_uppercase();
    let limit = limit.clamp(1, 1000).to_string();
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let body: serde_json::Value = client
        .get("https://api.bitget.com/api/v2/spot/market/candles")
        .query(&[
            ("symbol", market_id.as_str()),
            ("granularity", granularity),
            ("limit", limit.as_str()),
        ])
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch OHLCV from Bitget: {e}"))?
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Invalid Bitget OHLCV response: {e}"))?;
    parse_bitget_candles(&body)
}

/// Parse a timeframe string into the ccxt Timeframe enum
fn parse_timeframe(s: &str) -> anyhow::Result<ccxt_rust::Timeframe> {
    use ccxt_rust::Timeframe;
    // Match monthly timeframes before lowercasing, since "1M"/"M" (month)
    // would become "1m"/"m" (minute) after to_lowercase()
    match s {
        "1M" | "M" => return Ok(Timeframe::Mon1),
        _ => {}
    }
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
        _ => anyhow::bail!(
            "Invalid timeframe: {}. Valid: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M",
            s
        ),
    }
}

#[cfg(test)]
mod workaround_tests {
    use super::*;

    #[test]
    fn bitget_change_is_scaled_from_ratio_to_percent() {
        assert!((change_pct_value(ExchangeId::Bitget, 0.00301) - 0.301).abs() < 1e-12);
        assert_eq!(change_pct_value(ExchangeId::Binance, 1.5), 1.5);
        assert_eq!(change_pct_value(ExchangeId::Okx, -2.0), -2.0);
    }

    #[test]
    fn bitget_granularity_uses_v2_names() {
        assert_eq!(bitget_v2_granularity("1h").unwrap(), "1h");
        assert_eq!(bitget_v2_granularity("1d").unwrap(), "1day");
        assert_eq!(bitget_v2_granularity("5m").unwrap(), "5min");
        assert_eq!(bitget_v2_granularity("1w").unwrap(), "1week");
        assert_eq!(bitget_v2_granularity("1M").unwrap(), "1M");
        assert!(bitget_v2_granularity("2h").is_err());
    }

    #[test]
    fn parse_bitget_candles_live_shape() {
        let body = serde_json::json!({"code":"00000","msg":"success","data":[
            ["1790352000000","2687.09","2699.47","2678.26","2695.85","6578.5623","1.7e7","1.7e7"],
            ["1790265600000","2683.63","2742.76","2660.7","2687.09","55500.7331","1.4e8","1.4e8"]]});
        let c = parse_bitget_candles(&body).unwrap();
        assert_eq!(c.len(), 2);
        assert_eq!(c[0].timestamp, 1_790_265_600_000); // sorted ascending
        assert_eq!(c[1].close, "2695.85");
        let err =
            parse_bitget_candles(&serde_json::json!({"code":"400171","msg":"bad granularity"}))
                .unwrap_err()
                .to_string();
        assert!(err.contains("400171"));
    }

    #[test]
    fn hyperliquid_is_rejected_with_clear_error() {
        assert!(HYPERLIQUID_UNSUPPORTED.contains("415"));
    }
}
