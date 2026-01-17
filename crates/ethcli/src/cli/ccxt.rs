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
        limit: usize,

        #[command(flatten)]
        args: CcxtArgs,
    },

    /// Get recent trades for a trading pair
    Trades {
        /// Trading pair symbol
        symbol: String,

        /// Number of trades to fetch
        #[arg(long, default_value = "50")]
        limit: usize,

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

            // OHLCV is similar but requires different handling per exchange
            // For now, just show a placeholder
            println!(
                "OHLCV data fetch for {} on {} not yet fully implemented",
                symbol, args.exchange
            );
            println!("Timeframe: {}, Limit: {}", timeframe, limit);
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

            // Trades endpoint - placeholder
            println!(
                "Trades fetch for {} on {} not yet fully implemented",
                symbol, args.exchange
            );
        }

        CcxtCommands::Markets { quote, base, args } => {
            if !quiet {
                eprintln!("Fetching markets from {}...", args.exchange);
            }

            // Markets listing - placeholder
            println!(
                "Markets listing for {} not yet fully implemented",
                args.exchange
            );
            if let Some(q) = quote {
                println!("Filter by quote: {}", q);
            }
            if let Some(b) = base {
                println!("Filter by base: {}", b);
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
