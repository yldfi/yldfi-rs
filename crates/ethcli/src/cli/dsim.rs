//! Direct Dune SIM API commands
//!
//! Provides 1:1 access to Dune SIM API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};

const DUNE_SIM_SUNSET_WARNING: &str = "WARNING: Dune Sim shuts down 2026-08-01 (issue #64)";
const DUNE_SIM_DEFI_REMOVED: &str = "Dune Sim DeFi Positions was deprecated 2026-06-01 and the Sim platform shuts down 2026-08-01. See https://github.com/yldfi/yldfi-rs/issues/64";

/// Reject subcommands whose Dune Sim endpoints have already been removed
fn reject_removed_dsim_command(command: &DsimCommands) -> anyhow::Result<()> {
    match command {
        DsimCommands::Defi { .. } => anyhow::bail!(DUNE_SIM_DEFI_REMOVED),
        _ => Ok(()),
    }
}

#[derive(Args)]
pub struct DsimArgs {
    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "json")]
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
    use secrecy::ExposeSecret;

    reject_removed_dsim_command(command)?;

    if !quiet {
        eprintln!("{DUNE_SIM_SUNSET_WARNING}");
    }

    // Try config first, then fall back to env var. Dune Analytics keys are not
    // valid Sim credentials, so there is intentionally no DUNE_API_KEY fallback.
    let api_key = if let Ok(Some(config)) = ConfigFile::load_default() {
        if let Some(ref dune_sim_config) = config.dune_sim {
            dune_sim_config.api_key.expose_secret().to_string()
        } else {
            std::env::var("DUNE_SIM_API_KEY")
                .map_err(|_| anyhow::anyhow!("DUNE_SIM_API_KEY not set in config or environment"))?
        }
    } else {
        std::env::var("DUNE_SIM_API_KEY")
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

// Unreachable behind `reject_removed_dsim_command`; kept compiled until the
// 2026-08-01 Sim sunset removes the command tree entirely.
#[allow(deprecated)]
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

#[cfg(test)]
mod tests {
    use super::{
        reject_removed_dsim_command, BalancesCommands, DefiCommands, DsimArgs, DsimCommands,
        OutputFormat, DUNE_SIM_DEFI_REMOVED,
    };

    fn dsim_args() -> DsimArgs {
        DsimArgs {
            format: OutputFormat::Json,
        }
    }

    #[test]
    fn dsim_defi_positions_is_rejected() {
        let command = DsimCommands::Defi {
            action: DefiCommands::Positions {
                address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
            },
            args: dsim_args(),
        };

        let error = reject_removed_dsim_command(&command)
            .expect_err("dsim defi should be rejected")
            .to_string();
        assert_eq!(error, DUNE_SIM_DEFI_REMOVED);
    }

    #[test]
    fn other_dsim_commands_still_pass_the_guard() {
        let commands = [
            DsimCommands::Chains { args: dsim_args() },
            DsimCommands::Balances {
                action: BalancesCommands::Get {
                    address: "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045".to_string(),
                },
                args: dsim_args(),
            },
        ];

        for command in &commands {
            assert!(reject_removed_dsim_command(command).is_ok());
        }
    }
}
