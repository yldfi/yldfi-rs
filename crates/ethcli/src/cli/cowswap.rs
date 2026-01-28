//! Direct CowSwap (CoW Protocol) API commands
//!
//! Provides 1:1 access to CoW Protocol API endpoints for MEV-protected trading.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use cowp::{Client, OrderKind, QuoteRequest};

#[derive(Args, Clone)]
pub struct CowSwapArgs {
    #[command(subcommand)]
    pub action: CowSwapCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
    pub format: OutputFormat,
}

#[derive(Subcommand, Clone)]
pub enum CowSwapCommands {
    /// Get swap quote (MEV-protected)
    Quote {
        /// Source token address
        sell_token: String,
        /// Destination token address
        buy_token: String,
        /// Order kind: sell or buy
        #[arg(long, default_value = "sell")]
        kind: String,
        /// Amount in smallest units (sell amount if kind=sell, buy amount if kind=buy)
        amount: String,
        /// From address (order creator)
        from: String,
        /// Chain (ethereum, gnosis, arbitrum, sepolia)
        #[arg(long, default_value = "ethereum")]
        chain: String,
        /// Receiver address (defaults to from address)
        #[arg(long)]
        receiver: Option<String>,
    },

    /// Get order by UID
    Order {
        /// Order UID
        uid: String,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get orders for an address
    Orders {
        /// Owner address
        owner: String,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get trades for an address
    Trades {
        /// Owner address
        owner: String,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get trades for an order
    OrderTrades {
        /// Order UID
        uid: String,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get current auction
    Auction {
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get solver competition for an auction
    Competition {
        /// Auction ID
        auction_id: u64,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get native token price for a token
    NativePrice {
        /// Token address
        token: String,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },
}

pub async fn run(args: CowSwapArgs, _chain: &str) -> anyhow::Result<()> {
    let client = Client::new()?;

    match args.action {
        CowSwapCommands::Quote {
            sell_token,
            buy_token,
            kind,
            amount,
            from,
            chain,
            receiver,
        } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let order_kind = match kind.to_lowercase().as_str() {
                "sell" => OrderKind::Sell,
                "buy" => OrderKind::Buy,
                _ => anyhow::bail!("Invalid order kind: {}. Use 'sell' or 'buy'", kind),
            };

            let mut request = match order_kind {
                OrderKind::Sell => QuoteRequest::sell(&sell_token, &buy_token, &amount, &from),
                OrderKind::Buy => QuoteRequest::buy(&sell_token, &buy_token, &amount, &from),
            };
            request.receiver = receiver;

            let quote = client.get_quote(Some(cow_chain), &request).await?;
            output_json(&quote, args.format)?;
        }

        CowSwapCommands::Order { uid, chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let order = client.get_order(Some(cow_chain), &uid).await?;
            output_json(&order, args.format)?;
        }

        CowSwapCommands::Orders { owner, chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let orders = client.get_orders_by_owner(Some(cow_chain), &owner).await?;
            output_json(&orders, args.format)?;
        }

        CowSwapCommands::Trades { owner, chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let trades = client.get_trades_by_owner(Some(cow_chain), &owner).await?;
            output_json(&trades, args.format)?;
        }

        CowSwapCommands::OrderTrades { uid, chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let trades = client.get_trades_by_order(Some(cow_chain), &uid).await?;
            output_json(&trades, args.format)?;
        }

        CowSwapCommands::Auction { chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let auction = client.get_auction(Some(cow_chain)).await?;
            output_json(&auction, args.format)?;
        }

        CowSwapCommands::Competition { auction_id, chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let competition = client
                .get_solver_competition(Some(cow_chain), auction_id)
                .await?;
            output_json(&competition, args.format)?;
        }

        CowSwapCommands::NativePrice { token, chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let price = client.get_native_price(Some(cow_chain), &token).await?;
            output_json(&price, args.format)?;
        }
    }

    Ok(())
}

fn chain_name_to_cow_chain(name: &str) -> anyhow::Result<cowp::Chain> {
    match name.to_lowercase().as_str() {
        "ethereum" | "eth" | "mainnet" => Ok(cowp::Chain::Mainnet),
        "gnosis" | "xdai" => Ok(cowp::Chain::Gnosis),
        "arbitrum" | "arb" => Ok(cowp::Chain::Arbitrum),
        "sepolia" => Ok(cowp::Chain::Sepolia),
        _ => anyhow::bail!(
            "Unsupported chain: {}. Supported: ethereum, gnosis, arbitrum, sepolia",
            name
        ),
    }
}

fn output_json<T: serde::Serialize>(value: &T, format: OutputFormat) -> anyhow::Result<()> {
    let json = if format.is_table() {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    };
    println!("{}", json);
    Ok(())
}
