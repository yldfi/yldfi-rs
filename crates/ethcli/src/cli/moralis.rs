//! Direct Moralis API commands
//!
//! Provides 1:1 access to Moralis Web3 API endpoints.

use crate::cli::OutputFormat;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct MoralisArgs {
    /// Chain (e.g., eth, polygon, bsc, arbitrum)
    #[arg(long, short, default_value = "eth")]
    pub chain: String,

    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum MoralisCommands {
    /// Wallet operations
    Wallet {
        #[command(subcommand)]
        action: WalletCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Token operations
    Token {
        #[command(subcommand)]
        action: TokenCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// NFT operations
    Nft {
        #[command(subcommand)]
        action: NftCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Resolve domains
    Resolve {
        #[command(subcommand)]
        action: ResolveCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Market data
    Market {
        #[command(subcommand)]
        action: MarketCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },
}

#[derive(Subcommand)]
pub enum WalletCommands {
    /// Get native balance (ETH, MATIC, etc.)
    Balance {
        /// Wallet address
        address: String,
    },

    /// Get token balances
    Tokens {
        /// Wallet address
        address: String,
    },

    /// Get transactions
    Transactions {
        /// Wallet address
        address: String,
    },

    /// Get net worth across all chains
    NetWorth {
        /// Wallet address
        address: String,
    },

    /// Get active chains
    ActiveChains {
        /// Wallet address
        address: String,
    },

    /// Get token approvals
    Approvals {
        /// Wallet address
        address: String,
    },

    /// Get wallet history (decoded)
    History {
        /// Wallet address
        address: String,
    },

    /// Get wallet stats
    Stats {
        /// Wallet address
        address: String,
    },

    /// Get profitability summary
    Profitability {
        /// Wallet address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Get token metadata
    Metadata {
        /// Token contract address
        address: String,
    },

    /// Get token price
    Price {
        /// Token contract address
        address: String,
    },

    /// Get token transfers for an address
    Transfers {
        /// Wallet address
        address: String,
    },

    /// Get token DEX pairs
    Pairs {
        /// Token contract address
        address: String,
    },

    /// Get top token holders
    Holders {
        /// Token contract address
        address: String,
    },

    /// Get token swaps
    Swaps {
        /// Token contract address
        address: String,
    },

    /// Get token stats
    Stats {
        /// Token contract address
        address: String,
    },

    /// Search tokens
    Search {
        /// Search query
        query: String,
    },

    /// Get trending tokens
    Trending,

    /// Get pair OHLCV
    PairOhlcv {
        /// Pair address
        address: String,
    },

    /// Get pair stats
    PairStats {
        /// Pair address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum NftCommands {
    /// Get NFTs for a wallet
    List {
        /// Wallet address
        address: String,
    },

    /// Get NFT metadata
    Metadata {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get NFT transfers for a wallet
    Transfers {
        /// Wallet address
        address: String,
    },

    /// Get NFT collection metadata
    Collection {
        /// Contract address
        address: String,
    },

    /// Get collection stats
    CollectionStats {
        /// Contract address
        address: String,
    },

    /// Get NFT owners
    Owners {
        /// Contract address
        address: String,
    },

    /// Get NFT trades
    Trades {
        /// Contract address
        address: String,
    },

    /// Get floor price
    FloorPrice {
        /// Contract address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum ResolveCommands {
    /// Resolve ENS/domain to address
    Domain {
        /// Domain name (e.g., vitalik.eth)
        domain: String,
    },

    /// Reverse lookup address to domain
    Address {
        /// Wallet address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum MarketCommands {
    /// Get top ERC20 tokens by market cap
    TopTokens,

    /// Get top movers (gainers/losers)
    TopMovers,

    /// Get top NFT collections
    TopNfts,
}

/// Handle Moralis commands
pub async fn handle(command: &MoralisCommands, quiet: bool) -> anyhow::Result<()> {
    let client = mrls::Client::from_env()
        .map_err(|_| anyhow::anyhow!("MORALIS_API_KEY environment variable not set"))?;

    match command {
        MoralisCommands::Wallet { action, args } => {
            handle_wallet(&client, action, args, quiet).await
        }
        MoralisCommands::Token { action, args } => handle_token(&client, action, args, quiet).await,
        MoralisCommands::Nft { action, args } => handle_nft(&client, action, args, quiet).await,
        MoralisCommands::Resolve { action, args } => {
            handle_resolve(&client, action, args, quiet).await
        }
        MoralisCommands::Market { action, args } => {
            handle_market(&client, action, args, quiet).await
        }
    }
}

async fn handle_wallet(
    client: &mrls::Client,
    action: &WalletCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        WalletCommands::Balance { address } => {
            if !quiet {
                eprintln!("Fetching native balance for {}...", address);
            }
            let response = client
                .wallet()
                .get_native_balance(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::Tokens { address } => {
            if !quiet {
                eprintln!("Fetching token balances for {}...", address);
            }
            let query = mrls::WalletQuery::new().chain(&args.chain);
            let response = client
                .wallet()
                .get_token_balances(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::Transactions { address } => {
            if !quiet {
                eprintln!("Fetching transactions for {}...", address);
            }
            let query = mrls::WalletQuery::new().chain(&args.chain);
            let response = client
                .wallet()
                .get_transactions(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::NetWorth { address } => {
            if !quiet {
                eprintln!("Fetching net worth for {}...", address);
            }
            let response = client.wallet().get_net_worth(address).await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::ActiveChains { address } => {
            if !quiet {
                eprintln!("Fetching active chains for {}...", address);
            }
            let response = client.wallet().get_active_chains(address).await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::Approvals { address } => {
            if !quiet {
                eprintln!("Fetching approvals for {}...", address);
            }
            let query = mrls::WalletQuery::new().chain(&args.chain);
            let response = client.wallet().get_approvals(address, Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::History { address } => {
            if !quiet {
                eprintln!("Fetching history for {}...", address);
            }
            let query = mrls::WalletQuery::new().chain(&args.chain);
            let response = client.wallet().get_history(address, Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::Stats { address } => {
            if !quiet {
                eprintln!("Fetching stats for {}...", address);
            }
            let response = client.wallet().get_stats(address).await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::Profitability { address } => {
            if !quiet {
                eprintln!("Fetching profitability for {}...", address);
            }
            let response = client.wallet().get_profitability_summary(address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_token(
    client: &mrls::Client,
    action: &TokenCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TokenCommands::Metadata { address } => {
            if !quiet {
                eprintln!("Fetching token metadata for {}...", address);
            }
            let response = client
                .token()
                .get_metadata(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Price { address } => {
            if !quiet {
                eprintln!("Fetching token price for {}...", address);
            }
            let response = client.token().get_price(address, Some(&args.chain)).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Transfers { address } => {
            if !quiet {
                eprintln!("Fetching token transfers for {}...", address);
            }
            let response = client
                .token()
                .get_transfers(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Pairs { address } => {
            if !quiet {
                eprintln!("Fetching pairs for {}...", address);
            }
            let response = client.token().get_pairs(address, Some(&args.chain)).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Holders { address } => {
            if !quiet {
                eprintln!("Fetching holders for {}...", address);
            }
            let response = client
                .token()
                .get_holders(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Swaps { address } => {
            if !quiet {
                eprintln!("Fetching swaps for {}...", address);
            }
            let response = client.token().get_swaps(address, Some(&args.chain)).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Stats { address } => {
            if !quiet {
                eprintln!("Fetching stats for {}...", address);
            }
            let response = client.token().get_stats(address, Some(&args.chain)).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Search { query } => {
            if !quiet {
                eprintln!("Searching for {}...", query);
            }
            let response = client.token().search(query, Some(&args.chain)).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Trending => {
            if !quiet {
                eprintln!("Fetching trending tokens...");
            }
            let response = client.token().get_trending(Some(&args.chain)).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::PairOhlcv { address } => {
            if !quiet {
                eprintln!("Fetching OHLCV for pair {}...", address);
            }
            let response = client
                .token()
                .get_pair_ohlcv(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::PairStats { address } => {
            if !quiet {
                eprintln!("Fetching stats for pair {}...", address);
            }
            let response = client
                .token()
                .get_pair_stats(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_nft(
    client: &mrls::Client,
    action: &NftCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        NftCommands::List { address } => {
            if !quiet {
                eprintln!("Fetching NFTs for {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client.nft().get_wallet_nfts(address, Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Metadata { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching NFT metadata for {}:{}...", contract, token_id);
            }
            let response = client
                .nft()
                .get_nft_metadata(contract, token_id, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Transfers { address } => {
            if !quiet {
                eprintln!("Fetching NFT transfers for {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_wallet_nft_transfers(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Collection { address } => {
            if !quiet {
                eprintln!("Fetching collection metadata for {}...", address);
            }
            let response = client
                .nft()
                .get_collection_metadata(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::CollectionStats { address } => {
            if !quiet {
                eprintln!("Fetching collection stats for {}...", address);
            }
            let response = client
                .nft()
                .get_collection_stats(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Owners { address } => {
            if !quiet {
                eprintln!("Fetching owners for {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_collection_owners(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Trades { address } => {
            if !quiet {
                eprintln!("Fetching trades for {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_collection_trades(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::FloorPrice { address } => {
            if !quiet {
                eprintln!("Fetching floor price for {}...", address);
            }
            let response = client
                .nft()
                .get_floor_price(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_resolve(
    client: &mrls::Client,
    action: &ResolveCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        ResolveCommands::Domain { domain } => {
            if !quiet {
                eprintln!("Resolving {}...", domain);
            }
            let response = client.resolve().resolve_domain(domain).await?;
            print_output(&response, args.format)?;
        }
        ResolveCommands::Address { address } => {
            if !quiet {
                eprintln!("Reverse lookup for {}...", address);
            }
            let response = client.resolve().reverse_resolve(address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_market(
    client: &mrls::Client,
    action: &MarketCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        MarketCommands::TopTokens => {
            if !quiet {
                eprintln!("Fetching top tokens...");
            }
            let response = client.market().get_top_tokens(None).await?;
            print_output(&response, args.format)?;
        }
        MarketCommands::TopMovers => {
            if !quiet {
                eprintln!("Fetching top movers...");
            }
            let response = client.market().get_top_movers(None).await?;
            print_output(&response, args.format)?;
        }
        MarketCommands::TopNfts => {
            if !quiet {
                eprintln!("Fetching top NFT collections...");
            }
            let response = client.market().get_top_nft_collections(None).await?;
            print_output(&response, args.format)?;
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
