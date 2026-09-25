//! Direct CowSwap (CoW Protocol) API commands
//!
//! Provides 1:1 access to CoW Protocol API endpoints for MEV-protected trading.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use cowp::{
    Client, EcdsaSigningScheme, OrderCancellations, OrderCreation, OrderKind, QuoteRequest,
    SigningScheme, TradesQuery,
};

#[derive(Args, Clone)]
pub struct CowSwapArgs {
    #[command(subcommand)]
    pub action: CowSwapCommands,

    /// Output format
    #[arg(
        long,
        short = 'o',
        visible_alias = "output",
        default_value = "json",
        global = true
    )]
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

    /// Get trades for an address (paginated, newest first)
    Trades {
        /// Owner address
        owner: String,
        /// Pagination offset
        #[arg(long)]
        offset: Option<u64>,
        /// Max trades to return, 1-1000 (API default: 10)
        #[arg(long)]
        limit: Option<u32>,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Get trades for an order (paginated, newest first)
    OrderTrades {
        /// Order UID
        uid: String,
        /// Pagination offset
        #[arg(long)]
        offset: Option<u64>,
        /// Max trades to return, 1-1000 (API default: 10)
        #[arg(long)]
        limit: Option<u32>,
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

    /// Get solver competition (by auction ID, by settlement tx hash, or latest)
    Competition {
        /// Auction ID (omit for the latest competition)
        #[arg(conflicts_with = "tx_hash")]
        auction_id: Option<u64>,
        /// Settlement transaction hash
        #[arg(long)]
        tx_hash: Option<String>,
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

    /// Create an order (requires pre-signed order data)
    CreateOrder {
        /// Source token address
        sell_token: String,
        /// Destination token address
        buy_token: String,
        /// Sell amount in smallest units
        sell_amount: String,
        /// Buy amount in smallest units
        buy_amount: String,
        /// Order expiration (Unix timestamp)
        valid_to: u64,
        /// Order creator address
        from: String,
        /// Receiver address
        receiver: String,
        /// Signed order data (hex)
        signature: String,
        /// Order kind: sell or buy
        #[arg(long, default_value = "sell")]
        kind: String,
        /// Signing scheme: eip712, eip1271, presign
        #[arg(long, default_value = "eip712")]
        signing_scheme: String,
        /// Fee amount in sell token (smallest units)
        #[arg(long, default_value = "0")]
        fee_amount: String,
        /// App data hash
        #[arg(
            long,
            default_value = "0x0000000000000000000000000000000000000000000000000000000000000000"
        )]
        app_data: String,
        /// Allow partial fills
        #[arg(long)]
        partially_fillable: bool,
        /// Quote ID reference
        #[arg(long)]
        quote_id: Option<i64>,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Cancel a single order (deprecated API; prefer cancel-orders)
    ///
    /// Signature must be over `OrderCancellation(bytes orderUid)`.
    CancelOrder {
        /// Order UID to cancel
        uid: String,
        /// EIP-712 signature proving ownership
        signature: String,
        /// Chain
        #[arg(long, default_value = "ethereum")]
        chain: String,
    },

    /// Cancel one or more orders (up to 128)
    ///
    /// Signature must be over `OrderCancellations(bytes[] orderUids)`.
    CancelOrders {
        /// Order UIDs to cancel (comma-separated or repeated)
        #[arg(long = "uid", required = true, value_delimiter = ',')]
        uids: Vec<String>,
        /// Signature of `OrderCancellations` from the orders' owner
        signature: String,
        /// Signing scheme: eip712 or ethsign
        #[arg(long, default_value = "eip712")]
        signing_scheme: String,
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

        CowSwapCommands::Trades {
            owner,
            offset,
            limit,
            chain,
        } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let query = paginate(TradesQuery::by_owner(owner), offset, limit);
            let trades = client.get_trades(Some(cow_chain), &query).await?;
            output_json(&trades, args.format)?;
        }

        CowSwapCommands::OrderTrades {
            uid,
            offset,
            limit,
            chain,
        } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let query = paginate(TradesQuery::by_order(uid), offset, limit);
            let trades = client.get_trades(Some(cow_chain), &query).await?;
            output_json(&trades, args.format)?;
        }

        CowSwapCommands::Auction { chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let auction = client.get_auction(Some(cow_chain)).await?;
            output_json(&auction, args.format)?;
        }

        CowSwapCommands::Competition {
            auction_id,
            tx_hash,
            chain,
        } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let competition = match (auction_id, tx_hash) {
                (Some(id), _) => client.get_solver_competition(Some(cow_chain), id).await?,
                (None, Some(tx)) => {
                    client
                        .get_solver_competition_by_tx_hash(Some(cow_chain), &tx)
                        .await?
                }
                (None, None) => {
                    client
                        .get_latest_solver_competition(Some(cow_chain))
                        .await?
                }
            };
            output_json(&competition, args.format)?;
        }

        CowSwapCommands::NativePrice { token, chain } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let price = client.get_native_price(Some(cow_chain), &token).await?;
            output_json(&price, args.format)?;
        }

        CowSwapCommands::CreateOrder {
            sell_token,
            buy_token,
            sell_amount,
            buy_amount,
            valid_to,
            from,
            receiver,
            signature,
            kind,
            signing_scheme,
            fee_amount,
            app_data,
            partially_fillable,
            quote_id,
            chain,
        } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let order_kind = match kind.to_lowercase().as_str() {
                "sell" => OrderKind::Sell,
                "buy" => OrderKind::Buy,
                _ => anyhow::bail!("Invalid order kind: {}. Use 'sell' or 'buy'", kind),
            };
            let scheme = match signing_scheme.to_lowercase().as_str() {
                "eip712" => SigningScheme::Eip712,
                "eip1271" => SigningScheme::Eip1271,
                "presign" => SigningScheme::PreSign,
                _ => anyhow::bail!(
                    "Invalid signing scheme: {}. Use 'eip712', 'eip1271', or 'presign'",
                    signing_scheme
                ),
            };
            let order = OrderCreation {
                sell_token,
                buy_token,
                sell_amount,
                buy_amount,
                valid_to,
                app_data,
                fee_amount,
                kind: order_kind,
                partially_fillable,
                receiver,
                signature,
                signing_scheme: scheme,
                from,
                quote_id,
            };
            let uid = client.create_order(Some(cow_chain), &order).await?;
            output_json(&uid, args.format)?;
        }

        CowSwapCommands::CancelOrder {
            uid,
            signature,
            chain,
        } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            // The single-order endpoint is deprecated upstream but still served;
            // keep it for signatures produced over `OrderCancellation(bytes)`.
            #[allow(deprecated)]
            client
                .cancel_order(Some(cow_chain), &uid, &signature)
                .await?;
            println!("Order {} cancelled successfully", uid);
        }

        CowSwapCommands::CancelOrders {
            uids,
            signature,
            signing_scheme,
            chain,
        } => {
            let cow_chain = chain_name_to_cow_chain(&chain)?;
            let scheme = match signing_scheme.to_lowercase().as_str() {
                "eip712" => EcdsaSigningScheme::Eip712,
                "ethsign" => EcdsaSigningScheme::EthSign,
                _ => anyhow::bail!(
                    "Invalid signing scheme: {}. Use 'eip712' or 'ethsign'",
                    signing_scheme
                ),
            };
            if uids.len() > 128 {
                anyhow::bail!("At most 128 orders can be cancelled per request");
            }
            let count = uids.len();
            let cancellations =
                OrderCancellations::new(uids, signature).with_signing_scheme(scheme);
            client
                .cancel_orders(Some(cow_chain), &cancellations)
                .await?;
            println!("{} order(s) cancelled successfully", count);
        }
    }

    Ok(())
}

fn paginate(mut query: TradesQuery, offset: Option<u64>, limit: Option<u32>) -> TradesQuery {
    if let Some(offset) = offset {
        query = query.offset(offset);
    }
    if let Some(limit) = limit {
        query = query.limit(limit);
    }
    query
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
