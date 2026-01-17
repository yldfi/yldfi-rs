//! Direct Alchemy API commands
//!
//! Provides 1:1 access to Alchemy API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand, ValueEnum};

/// Network selection for Alchemy API
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum AlchemyNetwork {
    #[default]
    EthMainnet,
    EthSepolia,
    PolygonMainnet,
    ArbitrumMainnet,
    OptMainnet,
    BaseMainnet,
    ZksyncMainnet,
    LineaMainnet,
    ScrollMainnet,
    BlastMainnet,
    Bnb,
    Avalanche,
}

impl AlchemyNetwork {
    /// Get the network name as used in Alchemy API
    pub fn as_str(&self) -> &'static str {
        match self {
            AlchemyNetwork::EthMainnet => "eth-mainnet",
            AlchemyNetwork::EthSepolia => "eth-sepolia",
            AlchemyNetwork::PolygonMainnet => "polygon-mainnet",
            AlchemyNetwork::ArbitrumMainnet => "arb-mainnet",
            AlchemyNetwork::OptMainnet => "opt-mainnet",
            AlchemyNetwork::BaseMainnet => "base-mainnet",
            AlchemyNetwork::ZksyncMainnet => "zksync-mainnet",
            AlchemyNetwork::LineaMainnet => "linea-mainnet",
            AlchemyNetwork::ScrollMainnet => "scroll-mainnet",
            AlchemyNetwork::BlastMainnet => "blast-mainnet",
            AlchemyNetwork::Bnb => "bnb-mainnet",
            AlchemyNetwork::Avalanche => "avax-mainnet",
        }
    }
}

impl From<AlchemyNetwork> for alcmy::Network {
    fn from(n: AlchemyNetwork) -> alcmy::Network {
        match n {
            AlchemyNetwork::EthMainnet => alcmy::Network::EthMainnet,
            AlchemyNetwork::EthSepolia => alcmy::Network::EthSepolia,
            AlchemyNetwork::PolygonMainnet => alcmy::Network::PolygonMainnet,
            AlchemyNetwork::ArbitrumMainnet => alcmy::Network::ArbitrumMainnet,
            AlchemyNetwork::OptMainnet => alcmy::Network::OptMainnet,
            AlchemyNetwork::BaseMainnet => alcmy::Network::BaseMainnet,
            AlchemyNetwork::ZksyncMainnet => alcmy::Network::ZksyncMainnet,
            AlchemyNetwork::LineaMainnet => alcmy::Network::LineaMainnet,
            AlchemyNetwork::ScrollMainnet => alcmy::Network::ScrollMainnet,
            AlchemyNetwork::BlastMainnet => alcmy::Network::BlastMainnet,
            AlchemyNetwork::Bnb => alcmy::Network::Bnb,
            AlchemyNetwork::Avalanche => alcmy::Network::Avalanche,
        }
    }
}

#[derive(Args)]
pub struct AlchemyArgs {
    /// Alchemy network
    #[arg(long, short, default_value = "eth-mainnet")]
    pub network: AlchemyNetwork,

    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum AlchemyCommands {
    /// NFT operations
    Nft {
        #[command(subcommand)]
        action: NftCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Token operations
    Token {
        #[command(subcommand)]
        action: TokenCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Transfer history
    Transfers {
        #[command(subcommand)]
        action: TransferCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Portfolio/balances
    Portfolio {
        #[command(subcommand)]
        action: PortfolioCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Token prices
    Prices {
        #[command(subcommand)]
        action: PriceCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Debug/trace operations
    Debug {
        #[command(subcommand)]
        action: DebugCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },
}

#[derive(Subcommand)]
pub enum NftCommands {
    /// Get NFTs owned by an address
    GetNfts {
        /// Owner address
        address: String,
    },

    /// Get NFT metadata
    Metadata {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get floor price for a collection
    FloorPrice {
        /// Contract address
        contract: String,
    },

    /// Get owners of an NFT
    Owners {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Check if address owns contract NFT
    IsHolder {
        /// Address to check
        address: String,
        /// Contract address
        contract: String,
    },
}

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Get token balances for an address
    Balances {
        /// Address to query
        address: String,
    },

    /// Get token metadata
    Metadata {
        /// Token contract address
        contract: String,
    },

    /// Get token allowances
    Allowances {
        /// Owner address
        owner: String,
        /// Spender address
        spender: String,
    },
}

#[derive(Subcommand)]
pub enum TransferCommands {
    /// Get transfers from an address
    From {
        /// Source address
        address: String,
        /// Optional block range start
        #[arg(long)]
        from_block: Option<String>,
        /// Optional block range end
        #[arg(long)]
        to_block: Option<String>,
    },

    /// Get transfers to an address
    To {
        /// Destination address
        address: String,
        /// Optional block range start
        #[arg(long)]
        from_block: Option<String>,
        /// Optional block range end
        #[arg(long)]
        to_block: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum PortfolioCommands {
    /// Get token balances with prices
    Tokens {
        /// Address to query
        address: String,
    },
}

#[derive(Subcommand)]
pub enum PriceCommands {
    /// Get token prices by symbol
    BySymbol {
        /// Token symbols (comma-separated)
        symbols: String,
    },

    /// Get token prices by address
    ByAddress {
        /// Token addresses (comma-separated)
        addresses: String,
    },
}

#[derive(Subcommand)]
pub enum DebugCommands {
    /// Trace a transaction
    TraceTx {
        /// Transaction hash
        hash: String,
    },
}

/// Handle Alchemy commands
pub async fn handle(command: &AlchemyCommands, quiet: bool) -> anyhow::Result<()> {
    // Try config first, then fall back to env var
    let api_key = if let Ok(Some(config)) = ConfigFile::load_default() {
        if let Some(ref alchemy_config) = config.alchemy {
            alchemy_config.api_key.clone()
        } else {
            std::env::var("ALCHEMY_API_KEY")
                .map_err(|_| anyhow::anyhow!("ALCHEMY_API_KEY not set in config or environment"))?
        }
    } else {
        std::env::var("ALCHEMY_API_KEY")
            .map_err(|_| anyhow::anyhow!("ALCHEMY_API_KEY not set in config or environment"))?
    };

    match command {
        AlchemyCommands::Nft { action, args } => handle_nft(action, args, &api_key, quiet).await,
        AlchemyCommands::Token { action, args } => {
            handle_token(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Transfers { action, args } => {
            handle_transfers(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Portfolio { action, args } => {
            handle_portfolio(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Prices { action, args } => {
            handle_prices(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Debug { action, args } => {
            handle_debug(action, args, &api_key, quiet).await
        }
    }
}

async fn handle_nft(
    action: &NftCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        NftCommands::GetNfts { address } => {
            if !quiet {
                eprintln!("Fetching NFTs for {}...", address);
            }
            let response = client.nft().get_nfts_for_owner(address).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Metadata { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching NFT metadata for {}:{}...", contract, token_id);
            }
            let response = client.nft().get_nft_metadata(contract, token_id).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::FloorPrice { contract } => {
            if !quiet {
                eprintln!("Fetching floor price for {}...", contract);
            }
            let response = client.nft().get_floor_price(contract).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Owners { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching owners for {}:{}...", contract, token_id);
            }
            let response = client.nft().get_owners_for_nft(contract, token_id).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::IsHolder { address, contract } => {
            if !quiet {
                eprintln!("Checking if {} holds {}...", address, contract);
            }
            let response = client
                .nft()
                .is_holder_of_contract(address, contract)
                .await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

async fn handle_token(
    action: &TokenCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        TokenCommands::Balances { address } => {
            if !quiet {
                eprintln!("Fetching token balances for {}...", address);
            }
            let response = client.token().get_token_balances(address).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Metadata { contract } => {
            if !quiet {
                eprintln!("Fetching token metadata for {}...", contract);
            }
            let response = client.token().get_token_metadata(contract).await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Allowances { owner, spender } => {
            if !quiet {
                eprintln!("Fetching allowances from {} to {}...", owner, spender);
            }
            let response = client
                .token()
                .get_token_allowance(owner, spender, owner)
                .await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

async fn handle_transfers(
    action: &TransferCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        TransferCommands::From {
            address,
            from_block,
            to_block,
        } => {
            if !quiet {
                eprintln!("Fetching transfers from {}...", address);
            }
            let mut opts = alcmy::transfers::AssetTransfersOptions::from_address(address);
            if let Some(ref from) = from_block {
                opts.from_block = Some(from.clone());
            }
            if let Some(ref to) = to_block {
                opts.to_block = Some(to.clone());
            }
            let response = client.transfers().get_asset_transfers(&opts).await?;
            print_output(&response, args.format)?;
        }
        TransferCommands::To {
            address,
            from_block,
            to_block,
        } => {
            if !quiet {
                eprintln!("Fetching transfers to {}...", address);
            }
            let mut opts = alcmy::transfers::AssetTransfersOptions::to_address(address);
            if let Some(ref from) = from_block {
                opts.from_block = Some(from.clone());
            }
            if let Some(ref to) = to_block {
                opts.to_block = Some(to.clone());
            }
            let response = client.transfers().get_asset_transfers(&opts).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

async fn handle_portfolio(
    action: &PortfolioCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        PortfolioCommands::Tokens { address } => {
            if !quiet {
                eprintln!("Fetching portfolio for {}...", address);
            }
            // API expects (address, networks) tuples
            let networks: &[&str] = &[args.network.as_str()];
            let addresses: &[(&str, &[&str])] = &[(address.as_str(), networks)];
            let response = client.portfolio().get_token_balances(addresses).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

async fn handle_prices(
    action: &PriceCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        PriceCommands::BySymbol { symbols } => {
            if !quiet {
                eprintln!("Fetching prices for symbols: {}...", symbols);
            }
            let symbol_list: Vec<&str> = symbols.split(',').map(|s| s.trim()).collect();
            let response = client.prices().get_prices_by_symbol(&symbol_list).await?;
            print_output(&response, args.format)?;
        }
        PriceCommands::ByAddress { addresses } => {
            if !quiet {
                eprintln!("Fetching prices for addresses: {}...", addresses);
            }
            // API expects (network, address) tuples
            let addr_strs: Vec<&str> = addresses.split(',').map(|s| s.trim()).collect();
            let network = args.network.as_str();
            let addr_list: Vec<(&str, &str)> = addr_strs.iter().map(|a| (network, *a)).collect();
            let response = client.prices().get_prices_by_address(&addr_list).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

async fn handle_debug(
    action: &DebugCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        DebugCommands::TraceTx { hash } => {
            if !quiet {
                eprintln!("Tracing transaction {}...", hash);
            }
            let response = client.debug().trace_transaction(hash).await?;
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
            // For table format, just use JSON since these are raw API responses
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}
