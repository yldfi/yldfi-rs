pub mod anvil;
pub mod cast;
pub mod rpc;
pub mod tenderly;
pub mod types;
pub mod utils;

pub use anvil::*;
pub use cast::*;
pub use rpc::*;
pub use tenderly::*;
pub use types::*;
pub use utils::*;

use crate::config::Chain;
use clap::Subcommand;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum SimulateCommands {
    /// Simulate a transaction call (without sending)
    Call {
        /// Target contract address
        to: String,

        /// Function signature (e.g., "transfer(address,uint256)")
        #[arg(long, short)]
        sig: Option<String>,

        /// Raw calldata (hex encoded, alternative to --sig)
        #[arg(long, short)]
        data: Option<String>,

        /// Function arguments (used with --sig)
        args: Vec<String>,

        /// Sender address (default: zero address)
        #[arg(long)]
        from: Option<String>,

        /// Value to send in wei
        #[arg(long, default_value = "0")]
        value: String,

        /// Block number or tag (latest, pending, etc.)
        #[arg(long, short, default_value = "latest")]
        block: String,

        /// Gas limit
        #[arg(long)]
        gas: Option<u64>,

        /// Gas price in wei
        #[arg(long)]
        gas_price: Option<u64>,

        /// State override: set balance (format: address=wei, can repeat)
        #[arg(long = "balance-override", action = clap::ArgAction::Append)]
        balance_overrides: Vec<String>,

        /// State override: set storage slot (format: address:slot=value, can repeat)
        #[arg(long = "storage-override", action = clap::ArgAction::Append)]
        storage_overrides: Vec<String>,

        /// State override: set code (format: address=bytecode, can repeat)
        #[arg(long = "code-override", action = clap::ArgAction::Append)]
        code_overrides: Vec<String>,

        /// Override block timestamp (unix seconds)
        #[arg(long)]
        block_timestamp: Option<u64>,

        /// Override block number (for Tenderly)
        #[arg(long)]
        block_number_override: Option<u64>,

        /// Override block gas limit
        #[arg(long)]
        block_gas_limit: Option<u64>,

        /// Override block coinbase/miner address
        #[arg(long)]
        block_coinbase: Option<String>,

        /// Override block difficulty
        #[arg(long)]
        block_difficulty: Option<u64>,

        /// Override block base fee per gas (wei)
        #[arg(long)]
        block_base_fee: Option<u64>,

        /// Transaction index within the block
        #[arg(long)]
        transaction_index: Option<u64>,

        /// State override: set nonce (format: address=nonce, can repeat)
        #[arg(long = "nonce-override", action = clap::ArgAction::Append)]
        nonce_overrides: Vec<String>,

        /// Enable precise gas estimation (Tenderly)
        #[arg(long)]
        estimate_gas: bool,

        /// Generate EIP-2930 access list in response (Tenderly)
        #[arg(long)]
        generate_access_list: bool,

        /// Provide access list (JSON format or @file.json)
        #[arg(long)]
        access_list: Option<String>,

        /// Simulation type: full (default), quick (faster, less data), or abi (decode only)
        #[arg(long, value_enum, default_value = "full")]
        simulation_type: SimulationType,

        /// Network ID to simulate on (default: 1 for Ethereum mainnet)
        #[arg(long)]
        network_id: Option<String>,

        /// L1 block number (for L2 simulations like Optimism)
        #[arg(long)]
        l1_block_number: Option<u64>,

        /// L1 timestamp (for L2 simulations)
        #[arg(long)]
        l1_timestamp: Option<u64>,

        /// L1 message sender (for L2 cross-chain simulations)
        #[arg(long)]
        l1_message_sender: Option<String>,

        /// Mark as deposit transaction (Optimism Bedrock)
        #[arg(long)]
        deposit_tx: bool,

        /// Mark as system transaction (Optimism Bedrock)
        #[arg(long)]
        system_tx: bool,

        /// Simulation backend
        #[arg(long, value_enum, default_value = "cast")]
        via: SimulateVia,

        /// RPC URL
        #[arg(long)]
        rpc_url: Option<String>,

        /// Show execution trace (requires debug-capable node for cast)
        #[arg(long, short)]
        trace: bool,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,

        /// Save simulation to Tenderly (returns simulation ID)
        #[arg(long)]
        save: bool,

        /// Dry run - output request without executing (json, curl, fetch, powershell, url)
        #[arg(long, value_enum)]
        dry_run: Option<DryRunFormat>,

        /// Show API keys in dry-run output (default: masked with env var placeholders)
        #[arg(long)]
        show_secrets: bool,
    },

    /// Trace an existing transaction
    Tx {
        /// Transaction hash
        hash: String,

        /// Simulation backend
        #[arg(long, value_enum, default_value = "cast")]
        via: SimulateVia,

        /// RPC URL (for debug backend)
        #[arg(long)]
        rpc_url: Option<String>,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,

        /// Show full opcode trace
        #[arg(long, short)]
        trace: bool,

        /// Open interactive debugger (cast only)
        #[arg(long, short)]
        debug: bool,
    },

    /// Simulate a bundle of transactions in sequence (Tenderly only)
    Bundle {
        /// JSON file with transaction array, or inline JSON
        /// Format: [{"from": "0x...", "to": "0x...", "data": "0x...", "value": "0x0"}, ...]
        #[arg(long, short)]
        txs: String,

        /// Block number or tag (latest, pending, etc.)
        #[arg(long, short, default_value = "latest")]
        block: String,

        /// State override: set balance (format: address=wei, can repeat)
        #[arg(long = "balance-override", action = clap::ArgAction::Append)]
        balance_overrides: Vec<String>,

        /// State override: set storage slot (format: address:slot=value, can repeat)
        #[arg(long = "storage-override", action = clap::ArgAction::Append)]
        storage_overrides: Vec<String>,

        /// State override: set code (format: address=bytecode, can repeat)
        #[arg(long = "code-override", action = clap::ArgAction::Append)]
        code_overrides: Vec<String>,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,

        /// Save simulation bundle to Tenderly
        #[arg(long)]
        save: bool,
    },

    /// List saved simulations (Tenderly only)
    List {
        /// Number of simulations to list
        #[arg(long, short, default_value = "20")]
        limit: u32,

        /// Page number (0-indexed)
        #[arg(long, short, default_value = "0")]
        page: u32,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Get a saved simulation by ID (Tenderly only)
    Get {
        /// Simulation ID
        id: String,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Get simulation info/metadata by ID (Tenderly only)
    Info {
        /// Simulation ID
        id: String,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Share a simulation publicly (Tenderly only)
    /// Creates a public URL to view the simulation in the dashboard
    Share {
        /// Simulation ID
        id: String,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Unshare a simulation (make private) (Tenderly only)
    Unshare {
        /// Simulation ID
        id: String,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,
    },
}

pub async fn handle(action: &SimulateCommands, chain: Chain, quiet: bool) -> anyhow::Result<()> {
    match action {
        SimulateCommands::Call {
            to,
            sig,
            data,
            args,
            from,
            value,
            block,
            gas,
            gas_price,
            balance_overrides,
            storage_overrides,
            code_overrides,
            block_timestamp,
            via,
            rpc_url,
            trace,
            tenderly,
            save,
            dry_run,
            show_secrets,
            simulation_type,
            network_id,
            transaction_index,
            estimate_gas,
            generate_access_list,
            access_list,
            l1_block_number,
            l1_timestamp,
            l1_message_sender,
            deposit_tx,
            system_tx,
            block_gas_limit,
            block_coinbase,
            block_difficulty,
            block_base_fee,
            ..
        } => {
            // Warn if Tenderly-exclusive flags are used with non-Tenderly backends
            if !matches!(via, SimulateVia::Tenderly) {
                let mut tenderly_only = Vec::new();

                if *save {
                    tenderly_only.push("--save");
                }
                if *estimate_gas {
                    tenderly_only.push("--estimate-gas");
                }
                if *generate_access_list {
                    tenderly_only.push("--generate-access-list");
                }
                if !matches!(simulation_type, SimulationType::Full) {
                    tenderly_only.push("--simulation-type");
                }

                if l1_block_number.is_some() {
                    tenderly_only.push("--l1-block-number");
                }
                if l1_timestamp.is_some() {
                    tenderly_only.push("--l1-timestamp");
                }
                if l1_message_sender.is_some() {
                    tenderly_only.push("--l1-message-sender");
                }
                if *deposit_tx {
                    tenderly_only.push("--deposit-tx");
                }
                if *system_tx {
                    tenderly_only.push("--system-tx");
                }

                if !tenderly_only.is_empty() {
                    eprintln!("Warning: The following flags only work with --via tenderly and will be ignored:");
                    eprintln!("  {}", tenderly_only.join(", "));
                    eprintln!();
                }

                if matches!(via, SimulateVia::Cast | SimulateVia::Anvil) {
                    let mut not_supported = Vec::new();
                    if !balance_overrides.is_empty()
                        || !storage_overrides.is_empty()
                        || !code_overrides.is_empty()
                    {
                        not_supported.push("state overrides");
                    }
                    if block_timestamp.is_some()
                        || block_gas_limit.is_some()
                        || block_coinbase.is_some()
                        || block_difficulty.is_some()
                        || block_base_fee.is_some()
                    {
                        not_supported.push("block header overrides");
                    }
                    if access_list.is_some() {
                        not_supported.push("--access-list");
                    }
                    if transaction_index.is_some() {
                        not_supported.push("--transaction-index");
                    }
                    if network_id.is_some() {
                        not_supported.push("--network-id");
                    }

                    if !not_supported.is_empty() {
                        eprintln!("Warning: {} not supported for --via {:?}, use --via tenderly or --via debug/trace",
                            not_supported.join(", "), via);
                        eprintln!();
                    }
                }
            }

            match via {
                SimulateVia::Cast => {
                    if dry_run.is_some() {
                        return Err(anyhow::anyhow!("--dry-run not supported for cast backend. Use --via tenderly, debug, or trace"));
                    }
                    simulate_via_cast(
                        to, sig, data, args, from, value, block, rpc_url, *trace, quiet,
                    )
                    .await
                }
                SimulateVia::Anvil => {
                    if dry_run.is_some() {
                        return Err(anyhow::anyhow!("--dry-run not supported for anvil backend. Use --via tenderly, debug, or trace"));
                    }
                    simulate_via_anvil(to, sig, data, args, from, value, rpc_url, quiet).await
                }
                SimulateVia::Tenderly => {
                    simulate_via_tenderly(
                        to,
                        sig,
                        data,
                        args,
                        from,
                        value,
                        block,
                        *gas,
                        *gas_price,
                        balance_overrides,
                        storage_overrides,
                        code_overrides,
                        *block_timestamp,
                        *simulation_type,
                        *save,
                        tenderly,
                        *dry_run,
                        *show_secrets,
                        quiet,
                        network_id,
                        *transaction_index,
                        *estimate_gas,
                        *generate_access_list,
                        access_list,
                        *l1_block_number,
                        *l1_timestamp,
                        l1_message_sender,
                        *deposit_tx,
                        *system_tx,
                        *block_gas_limit,
                        block_coinbase,
                        *block_difficulty,
                        *block_base_fee,
                    )
                    .await
                }
                SimulateVia::Debug => {
                    simulate_via_debug_rpc(
                        to,
                        sig,
                        data,
                        args,
                        from,
                        value,
                        block,
                        rpc_url,
                        chain,
                        balance_overrides,
                        storage_overrides,
                        code_overrides,
                        *dry_run,
                        *show_secrets,
                        quiet,
                    )
                    .await
                }
                SimulateVia::Trace => {
                    simulate_via_trace_rpc(
                        to,
                        sig,
                        data,
                        args,
                        from,
                        value,
                        block,
                        rpc_url,
                        chain,
                        balance_overrides,
                        storage_overrides,
                        code_overrides,
                        *dry_run,
                        *show_secrets,
                        quiet,
                    )
                    .await
                }
            }
        }

        SimulateCommands::Tx {
            hash,
            via,
            rpc_url,
            tenderly,
            trace,
            debug,
        } => match via {
            SimulateVia::Cast | SimulateVia::Anvil => {
                trace_tx_via_cast(hash, *trace, *debug, rpc_url, quiet).await
            }
            SimulateVia::Tenderly => trace_tx_via_tenderly(hash, tenderly, quiet).await,
            SimulateVia::Debug => trace_tx_via_debug_rpc(hash, rpc_url, chain, quiet).await,
            SimulateVia::Trace => trace_tx_via_trace_rpc(hash, rpc_url, chain, quiet).await,
        },

        SimulateCommands::Bundle {
            txs,
            block,
            balance_overrides,
            storage_overrides,
            code_overrides,
            tenderly,
            save,
        } => {
            simulate_bundle_tenderly(
                txs,
                block,
                balance_overrides,
                storage_overrides,
                code_overrides,
                *save,
                tenderly,
                quiet,
            )
            .await
        }

        SimulateCommands::List {
            limit,
            page,
            tenderly,
        } => list_simulations_tenderly(*limit, *page, tenderly, quiet).await,

        SimulateCommands::Get { id, tenderly } => {
            get_simulation_tenderly(id, tenderly, quiet).await
        }

        SimulateCommands::Info { id, tenderly } => {
            get_simulation_info_tenderly(id, tenderly, quiet).await
        }

        SimulateCommands::Share { id, tenderly } => {
            share_simulation_tenderly(id, tenderly, quiet).await
        }

        SimulateCommands::Unshare { id, tenderly } => {
            unshare_simulation_tenderly(id, tenderly, quiet).await
        }
    }
}
