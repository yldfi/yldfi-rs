//! Direct Pyth Network API commands
//!
//! Provides 1:1 access to Pyth Network Hermes API endpoints for price feeds.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};
use pyth::{feed_ids, Client};

#[derive(Args, Clone)]
pub struct PythArgs {
    #[command(subcommand)]
    pub action: PythCommands,

    /// Output format
    #[arg(long, short = 'o', default_value = "json", global = true)]
    pub format: OutputFormat,
}

#[derive(Subcommand, Clone)]
pub enum PythCommands {
    /// Get latest price for one or more feeds
    Price {
        /// Feed IDs or symbols (e.g., "BTC/USD" or feed ID hex)
        feeds: Vec<String>,
    },

    /// Search for price feeds by query
    Search {
        /// Search query (e.g., "BTC", "ETH/USD")
        query: String,
    },

    /// List all available price feed IDs
    Feeds {
        /// Filter by asset type (crypto, equity, fx, metal, rates)
        #[arg(long)]
        asset_type: Option<String>,
    },

    /// Get well-known feed IDs
    KnownFeeds,
}

pub async fn run(args: PythArgs, _chain: &str) -> anyhow::Result<()> {
    let client = Client::new()?;

    match args.action {
        PythCommands::Price { feeds } => {
            // Convert symbols to feed IDs where needed
            let feed_ids: Vec<String> = feeds
                .iter()
                .map(|f| symbol_to_feed_id(f).map(|s| s.to_string()).unwrap_or_else(|| f.clone()))
                .collect();

            let feed_refs: Vec<&str> = feed_ids.iter().map(|s| s.as_str()).collect();

            if feed_refs.len() == 1 {
                let price = client.get_latest_price(feed_refs[0]).await?;
                output_json(&price, args.format)?;
            } else {
                let prices = client.get_latest_prices(&feed_refs).await?;
                output_json(&prices, args.format)?;
            }
        }

        PythCommands::Search { query } => {
            let feeds = client.search_feeds(&query).await?;
            output_json(&feeds, args.format)?;
        }

        PythCommands::Feeds { asset_type } => {
            if let Some(at) = asset_type {
                let feeds = client.get_feeds_by_asset_type(&at).await?;
                output_json(&feeds, args.format)?;
            } else {
                let feeds = client.get_price_feed_ids().await?;
                output_json(&feeds, args.format)?;
            }
        }

        PythCommands::KnownFeeds => {
            let known = get_known_feeds();
            output_json(&known, args.format)?;
        }
    }

    Ok(())
}

/// Convert common symbols to Pyth feed IDs
fn symbol_to_feed_id(symbol: &str) -> Option<&'static str> {
    match symbol.to_uppercase().as_str() {
        "BTC" | "BTC/USD" | "BTCUSD" => Some(feed_ids::BTC_USD),
        "ETH" | "ETH/USD" | "ETHUSD" => Some(feed_ids::ETH_USD),
        "SOL" | "SOL/USD" | "SOLUSD" => Some(feed_ids::SOL_USD),
        "USDC" | "USDC/USD" => Some(feed_ids::USDC_USD),
        "USDT" | "USDT/USD" => Some(feed_ids::USDT_USD),
        "DAI" | "DAI/USD" => Some(feed_ids::DAI_USD),
        "AVAX" | "AVAX/USD" => Some(feed_ids::AVAX_USD),
        "ARB" | "ARB/USD" => Some(feed_ids::ARB_USD),
        "OP" | "OP/USD" => Some(feed_ids::OP_USD),
        "LINK" | "LINK/USD" => Some(feed_ids::LINK_USD),
        "UNI" | "UNI/USD" => Some(feed_ids::UNI_USD),
        "AAVE" | "AAVE/USD" => Some(feed_ids::AAVE_USD),
        "CRV" | "CRV/USD" => Some(feed_ids::CRV_USD),
        "CVX" | "CVX/USD" => Some(feed_ids::CVX_USD),
        "SNX" | "SNX/USD" => Some(feed_ids::SNX_USD),
        "LDO" | "LDO/USD" => Some(feed_ids::LDO_USD),
        "DOGE" | "DOGE/USD" => Some(feed_ids::DOGE_USD),
        "ATOM" | "ATOM/USD" => Some(feed_ids::ATOM_USD),
        "DOT" | "DOT/USD" => Some(feed_ids::DOT_USD),
        _ => None,
    }
}

fn get_known_feeds() -> serde_json::Value {
    serde_json::json!({
        "crypto": {
            "BTC/USD": feed_ids::BTC_USD,
            "ETH/USD": feed_ids::ETH_USD,
            "SOL/USD": feed_ids::SOL_USD,
            "USDC/USD": feed_ids::USDC_USD,
            "USDT/USD": feed_ids::USDT_USD,
            "DAI/USD": feed_ids::DAI_USD,
            "AVAX/USD": feed_ids::AVAX_USD,
            "ARB/USD": feed_ids::ARB_USD,
            "OP/USD": feed_ids::OP_USD,
            "LINK/USD": feed_ids::LINK_USD,
            "UNI/USD": feed_ids::UNI_USD,
            "AAVE/USD": feed_ids::AAVE_USD,
            "CRV/USD": feed_ids::CRV_USD,
            "CVX/USD": feed_ids::CVX_USD,
            "SNX/USD": feed_ids::SNX_USD,
            "LDO/USD": feed_ids::LDO_USD,
            "DOGE/USD": feed_ids::DOGE_USD,
            "ATOM/USD": feed_ids::ATOM_USD,
            "DOT/USD": feed_ids::DOT_USD,
        },
        "note": "Use 'ethcli pyth search <symbol>' to find additional feeds"
    })
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
