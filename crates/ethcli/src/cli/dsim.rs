//! Direct Dune SIM API commands
//!
//! Provides 1:1 access to Dune SIM API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct DsimArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
pub enum DsimCommands {
    /// Supported chains
    Chains {
        #[command(flatten)]
        args: DsimArgs,
    },

    /// Token balances
    Balances {
        #[command(subcommand)]
        action: BalancesCommands,

        #[command(flatten)]
        args: DsimArgs,
    },

    /// Collectibles (NFTs)
    Collectibles {
        #[command(subcommand)]
        action: CollectiblesCommands,

        #[command(flatten)]
        args: DsimArgs,
    },

    /// Wallet activity
    Activity {
        #[command(subcommand)]
        action: ActivityCommands,

        #[command(flatten)]
        args: DsimArgs,
    },

    /// Token info
    Token {
        #[command(subcommand)]
        action: TokenCommands,

        #[command(flatten)]
        args: DsimArgs,
    },

    /// Token holders
    Holders {
        #[command(subcommand)]
        action: HoldersCommands,

        #[command(flatten)]
        args: DsimArgs,
    },

    /// DeFi positions (Beta)
    Defi {
        #[command(subcommand)]
        action: DefiCommands,

        #[command(flatten)]
        args: DsimArgs,
    },
}

#[derive(Subcommand)]
pub enum BalancesCommands {
    /// Get all token balances for a wallet
    Get {
        /// Wallet address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum CollectiblesCommands {
    /// Get NFTs for a wallet
    Get {
        /// Wallet address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum ActivityCommands {
    /// Get wallet activity
    Get {
        /// Wallet address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum TokenCommands {
    /// Get token info
    Info {
        /// Token contract address or "native"
        address: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: i64,
    },
}

#[derive(Subcommand)]
pub enum HoldersCommands {
    /// Get token holders
    Get {
        /// Token contract address
        address: String,
        /// Chain ID
        #[arg(long, default_value = "1")]
        chain_id: i64,
    },
}

#[derive(Subcommand)]
pub enum DefiCommands {
    /// Get DeFi positions for a wallet
    Positions {
        /// Wallet address
        address: String,
    },
}

/// Handle Dune SIM commands
pub async fn handle(command: &DsimCommands, quiet: bool) -> anyhow::Result<()> {
    // Try config first, then fall back to env var
    let api_key = if let Ok(Some(config)) = ConfigFile::load_default() {
        if let Some(ref dune_sim_config) = config.dune_sim {
            dune_sim_config.api_key.clone()
        } else if let Some(ref dune_config) = config.dune {
            // Fall back to Dune Analytics key if no SIM-specific key
            dune_config.api_key.clone()
        } else {
            std::env::var("DUNE_SIM_API_KEY")
                .or_else(|_| std::env::var("DUNE_API_KEY"))
                .map_err(|_| {
                    anyhow::anyhow!("DUNE_SIM_API_KEY not set in config or environment")
                })?
        }
    } else {
        std::env::var("DUNE_SIM_API_KEY")
            .or_else(|_| std::env::var("DUNE_API_KEY"))
            .map_err(|_| anyhow::anyhow!("DUNE_SIM_API_KEY not set in config or environment"))?
    };

    let client = dnsim::Client::new(&api_key)?;

    match command {
        DsimCommands::Chains { args } => {
            if !quiet {
                eprintln!("Fetching supported chains...");
            }
            let response = client.chains().list().await?;
            print_output(&response, args.format)?;
        }
        DsimCommands::Balances { action, args } => {
            handle_balances(&client, action, args, quiet).await?
        }
        DsimCommands::Collectibles { action, args } => {
            handle_collectibles(&client, action, args, quiet).await?
        }
        DsimCommands::Activity { action, args } => {
            handle_activity(&client, action, args, quiet).await?
        }
        DsimCommands::Token { action, args } => handle_token(&client, action, args, quiet).await?,
        DsimCommands::Holders { action, args } => {
            handle_holders(&client, action, args, quiet).await?
        }
        DsimCommands::Defi { action, args } => handle_defi(&client, action, args, quiet).await?,
    }

    Ok(())
}

async fn handle_balances(
    client: &dnsim::Client,
    action: &BalancesCommands,
    args: &DsimArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        BalancesCommands::Get { address } => {
            if !quiet {
                eprintln!("Fetching balances for {}...", address);
            }
            let response = client.balances().get(address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_collectibles(
    client: &dnsim::Client,
    action: &CollectiblesCommands,
    args: &DsimArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        CollectiblesCommands::Get { address } => {
            if !quiet {
                eprintln!("Fetching collectibles for {}...", address);
            }
            let response = client.collectibles().get(address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_activity(
    client: &dnsim::Client,
    action: &ActivityCommands,
    args: &DsimArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        ActivityCommands::Get { address } => {
            if !quiet {
                eprintln!("Fetching activity for {}...", address);
            }
            let response = client.activity().get(address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_token(
    client: &dnsim::Client,
    action: &TokenCommands,
    args: &DsimArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TokenCommands::Info { address, chain_id } => {
            if !quiet {
                eprintln!("Fetching token info for {}...", address);
            }
            let chain_id_str = chain_id.to_string();
            let options = dnsim::tokens::TokenInfoOptions::new(&chain_id_str);
            let response = client.tokens().get(address, &options).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_holders(
    client: &dnsim::Client,
    action: &HoldersCommands,
    args: &DsimArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        HoldersCommands::Get { address, chain_id } => {
            if !quiet {
                eprintln!("Fetching holders for {}...", address);
            }
            let response = client.holders().get(*chain_id, address).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_defi(
    client: &dnsim::Client,
    action: &DefiCommands,
    args: &DsimArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        DefiCommands::Positions { address } => {
            if !quiet {
                eprintln!("Fetching DeFi positions for {}...", address);
            }
            let response = client.defi().positions(address).await?;
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
