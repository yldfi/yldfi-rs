//! Direct Yearn Kong API commands
//!
//! Provides 1:1 access to Yearn's Kong GraphQL API for vaults, strategies, prices, TVL, and reports.

use crate::cli::OutputFormat;
use anyhow::bail;
use clap::{Args, Subcommand, ValueEnum};

/// Validate that a string is a valid Ethereum address (0x + 40 hex chars)
fn validate_address(address: &str) -> anyhow::Result<()> {
    if !address.starts_with("0x") {
        bail!("Invalid address: must start with '0x'");
    }
    if address.len() != 42 {
        bail!(
            "Invalid address: expected 42 characters, got {}",
            address.len()
        );
    }
    if !address[2..].chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("Invalid address: contains non-hexadecimal characters");
    }
    Ok(())
}

/// Validate chain ID is a known supported network
fn validate_chain_id(chain_id: u64) -> anyhow::Result<()> {
    const SUPPORTED_CHAINS: &[(u64, &str)] = &[
        (1, "Ethereum"),
        (137, "Polygon"),
        (42161, "Arbitrum"),
        (10, "Optimism"),
        (8453, "Base"),
        (250, "Fantom"),
        (100, "Gnosis"),
    ];

    if !SUPPORTED_CHAINS.iter().any(|(id, _)| *id == chain_id) {
        let supported: Vec<String> = SUPPORTED_CHAINS
            .iter()
            .map(|(id, name)| format!("{}={}", id, name))
            .collect();
        bail!(
            "Unsupported chain ID: {}. Supported chains: {}",
            chain_id,
            supported.join(", ")
        );
    }
    Ok(())
}

#[derive(Args)]
pub struct KongArgs {
    /// Output format
    #[arg(long, short = 'o', default_value = "json")]
    pub format: OutputFormat,
}

#[derive(Subcommand)]
#[non_exhaustive]
pub enum KongCommands {
    /// Yearn vault data
    Vaults {
        #[command(subcommand)]
        action: VaultCommands,

        #[command(flatten)]
        args: KongArgs,
    },

    /// Yearn strategy data
    Strategies {
        #[command(subcommand)]
        action: StrategyCommands,

        #[command(flatten)]
        args: KongArgs,
    },

    /// Token prices
    Prices {
        #[command(subcommand)]
        action: PriceCommands,

        #[command(flatten)]
        args: KongArgs,
    },

    /// TVL (Total Value Locked) data
    Tvl {
        #[command(subcommand)]
        action: TvlCommands,

        #[command(flatten)]
        args: KongArgs,
    },

    /// Vault and strategy reports (harvests)
    Reports {
        #[command(subcommand)]
        action: ReportCommands,

        #[command(flatten)]
        args: KongArgs,
    },
}

#[derive(Subcommand)]
#[non_exhaustive]
pub enum VaultCommands {
    /// List all vaults (optionally filtered)
    List {
        /// Filter by chain ID (1=Ethereum, 137=Polygon, 42161=Arbitrum, etc.)
        #[arg(long, short = 'c')]
        chain_id: Option<u64>,

        /// Filter v3 vaults only
        #[arg(long)]
        v3: bool,

        /// Filter official Yearn vaults only
        #[arg(long)]
        yearn: bool,

        /// Filter ERC4626 compliant vaults
        #[arg(long)]
        erc4626: bool,
    },

    /// Get a specific vault by address
    Get {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Vault address
        address: String,
    },

    /// Get user positions in vaults
    Accounts {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// User wallet address
        address: String,
    },
}

#[derive(Subcommand)]
#[non_exhaustive]
pub enum StrategyCommands {
    /// List all strategies (optionally filtered)
    List {
        /// Filter by chain ID
        #[arg(long, short = 'c')]
        chain_id: Option<u64>,

        /// Filter by vault address
        #[arg(long)]
        vault: Option<String>,

        /// Filter v3 strategies only
        #[arg(long)]
        v3: bool,
    },

    /// Get a specific strategy by address
    Get {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Strategy address
        address: String,
    },
}

#[derive(Subcommand)]
#[non_exhaustive]
pub enum PriceCommands {
    /// Get current token price
    Current {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Token contract address
        address: String,
    },

    /// Get historical price at timestamp
    Historical {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Token contract address
        address: String,

        /// Unix timestamp
        timestamp: u64,
    },
}

/// TVL period for historical queries
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum TvlPeriodArg {
    /// Daily data points
    #[default]
    Day,
    /// Weekly data points
    Week,
    /// Monthly data points
    Month,
}

impl From<TvlPeriodArg> for ykong::TvlPeriod {
    fn from(arg: TvlPeriodArg) -> Self {
        match arg {
            TvlPeriodArg::Day => ykong::TvlPeriod::Day,
            TvlPeriodArg::Week => ykong::TvlPeriod::Week,
            TvlPeriodArg::Month => ykong::TvlPeriod::Month,
        }
    }
}

#[derive(Subcommand)]
#[non_exhaustive]
pub enum TvlCommands {
    /// Get current TVL for a vault/strategy
    Current {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Vault or strategy address
        address: String,
    },

    /// Get TVL history
    History {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Vault or strategy address
        address: String,

        /// Time period granularity
        #[arg(long, short, default_value = "day")]
        period: TvlPeriodArg,

        /// Number of data points to return
        #[arg(long, short, default_value = "30")]
        limit: u32,
    },
}

#[derive(Subcommand)]
#[non_exhaustive]
pub enum ReportCommands {
    /// Get vault reports (harvest events)
    Vault {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Vault address
        address: String,
    },

    /// Get strategy reports (harvest events)
    Strategy {
        /// Chain ID
        #[arg(long, short = 'c', default_value = "1")]
        chain_id: u64,

        /// Strategy address
        address: String,
    },
}

/// Handle Kong commands
pub async fn handle(command: &KongCommands, quiet: bool) -> anyhow::Result<()> {
    let client = ykong::Client::new()?;

    match command {
        KongCommands::Vaults { action, args } => handle_vaults(&client, action, args, quiet).await,
        KongCommands::Strategies { action, args } => {
            handle_strategies(&client, action, args, quiet).await
        }
        KongCommands::Prices { action, args } => handle_prices(&client, action, args, quiet).await,
        KongCommands::Tvl { action, args } => handle_tvl(&client, action, args, quiet).await,
        KongCommands::Reports { action, args } => {
            handle_reports(&client, action, args, quiet).await
        }
    }
}

async fn handle_vaults(
    client: &ykong::Client,
    action: &VaultCommands,
    args: &KongArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        VaultCommands::List {
            chain_id,
            v3,
            yearn,
            erc4626,
        } => {
            if !quiet {
                eprintln!("Fetching vaults...");
            }

            let mut filter = ykong::VaultFilter::new();
            if let Some(cid) = chain_id {
                validate_chain_id(*cid)?;
                filter = filter.chain_id(*cid);
            }
            if *v3 {
                filter = filter.v3(true);
            }
            if *yearn {
                filter = filter.yearn(true);
            }
            if *erc4626 {
                filter = filter.erc4626(true);
            }

            let vaults = client.vaults().list(Some(filter)).await?;
            if !quiet {
                eprintln!("Found {} vaults", vaults.len());
            }
            print_output(&vaults, args.format)?;
        }
        VaultCommands::Get { chain_id, address } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!("Fetching vault {} on chain {}...", address, chain_id);
            }
            let vault = client.vaults().get(*chain_id, address).await?;
            print_output(&vault, args.format)?;
        }
        VaultCommands::Accounts { chain_id, address } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!(
                    "Fetching vault accounts for {} on chain {}...",
                    address, chain_id
                );
            }
            let accounts = client.vaults().accounts(*chain_id, address).await?;
            if !quiet {
                eprintln!("Found {} positions", accounts.len());
            }
            print_output(&accounts, args.format)?;
        }
    }
    Ok(())
}

async fn handle_strategies(
    client: &ykong::Client,
    action: &StrategyCommands,
    args: &KongArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        StrategyCommands::List { chain_id, vault, v3 } => {
            if !quiet {
                eprintln!("Fetching strategies...");
            }

            let mut filter = ykong::StrategyFilter::new();
            if let Some(cid) = chain_id {
                validate_chain_id(*cid)?;
                filter = filter.chain_id(*cid);
            }
            if let Some(vault_addr) = vault {
                validate_address(vault_addr)?;
                filter = filter.vault(vault_addr.clone());
            }
            if *v3 {
                filter = filter.v3(true);
            }

            let strategies = client.strategies().list(Some(filter)).await?;
            if !quiet {
                eprintln!("Found {} strategies", strategies.len());
            }
            print_output(&strategies, args.format)?;
        }
        StrategyCommands::Get { chain_id, address } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!("Fetching strategy {} on chain {}...", address, chain_id);
            }
            let strategy = client.strategies().get(*chain_id, address).await?;
            print_output(&strategy, args.format)?;
        }
    }
    Ok(())
}

async fn handle_prices(
    client: &ykong::Client,
    action: &PriceCommands,
    args: &KongArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        PriceCommands::Current { chain_id, address } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!("Fetching price for {} on chain {}...", address, chain_id);
            }
            let price = client.prices().current(*chain_id, address).await?;
            print_output(&price, args.format)?;
        }
        PriceCommands::Historical {
            chain_id,
            address,
            timestamp,
        } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!(
                    "Fetching price for {} on chain {} at {}...",
                    address, chain_id, timestamp
                );
            }
            let prices = client
                .prices()
                .at_timestamp(*chain_id, address, *timestamp)
                .await?;
            print_output(&prices, args.format)?;
        }
    }
    Ok(())
}

async fn handle_tvl(
    client: &ykong::Client,
    action: &TvlCommands,
    args: &KongArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TvlCommands::Current { chain_id, address } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!(
                    "Fetching current TVL for {} on chain {}...",
                    address, chain_id
                );
            }
            let tvl = client.tvls().current(*chain_id, address).await?;
            print_output(&tvl, args.format)?;
        }
        TvlCommands::History {
            chain_id,
            address,
            period,
            limit,
        } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!(
                    "Fetching TVL history for {} on chain {} ({:?}, {} points)...",
                    address, chain_id, period, limit
                );
            }
            let tvls = client
                .tvls()
                .history(*chain_id, address, (*period).into(), *limit)
                .await?;
            if !quiet {
                eprintln!("Found {} data points", tvls.len());
            }
            print_output(&tvls, args.format)?;
        }
    }
    Ok(())
}

async fn handle_reports(
    client: &ykong::Client,
    action: &ReportCommands,
    args: &KongArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        ReportCommands::Vault { chain_id, address } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!(
                    "Fetching vault reports for {} on chain {}...",
                    address, chain_id
                );
            }
            let reports = client.reports().vault_reports(*chain_id, address).await?;
            if !quiet {
                eprintln!("Found {} reports", reports.len());
            }
            print_output(&reports, args.format)?;
        }
        ReportCommands::Strategy { chain_id, address } => {
            validate_address(address)?;
            validate_chain_id(*chain_id)?;
            if !quiet {
                eprintln!(
                    "Fetching strategy reports for {} on chain {}...",
                    address, chain_id
                );
            }
            let reports = client
                .reports()
                .strategy_reports(*chain_id, address)
                .await?;
            if !quiet {
                eprintln!("Found {} reports", reports.len());
            }
            print_output(&reports, args.format)?;
        }
    }
    Ok(())
}

/// Print output in the specified format.
///
/// Note: Table format falls back to pretty JSON for Kong API responses because:
/// 1. Kong returns deeply nested GraphQL data (vaults contain strategies, tokens, etc.)
/// 2. The data structure varies significantly between endpoints
/// 3. Implementing generic table formatting for arbitrary nested JSON is complex
///
/// For simpler tabular output, consider using `--format json` and piping to `jq`.
fn print_output<T: serde::Serialize>(data: &T, format: OutputFormat) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
        OutputFormat::Ndjson => {
            println!("{}", serde_json::to_string(data)?);
        }
        OutputFormat::Table => {
            // Table format uses pretty JSON for Kong's deeply nested GraphQL responses.
            // See function doc comment for rationale.
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    }
    Ok(())
}
