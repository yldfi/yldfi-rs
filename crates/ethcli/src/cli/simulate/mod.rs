pub mod alchemy;
pub mod anvil;
pub mod cast;
pub mod decode;
pub mod rpc;
pub mod tenderly;
pub mod types;
pub mod utils;

pub use alchemy::*;
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
#[command(after_help = r#"Examples:
  # Simulate a balanceOf call using cast
  ethcli simulate call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --sig "balanceOf(address)" 0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045

  # Simulate with trace using debug-capable node
  ethcli simulate call 0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48 --sig "transfer(address,uint256)" 0x123... 1000000 --trace

  # Simulate via Tenderly with state overrides
  ethcli simulate call 0x7a250d5630B4cF539739dF2C5dAcb4c659F2488D --sig "swapExactETHForTokens(uint256,address[],address,uint256)" 0 '[...]' 0x... 9999999999 --via tenderly --balance-override 0x123=1000000000000000000

  # Trace an existing transaction
  ethcli simulate tx 0x123abc... --via tenderly

  # Simulate using Anvil fork
  ethcli simulate call 0x... --sig "foo()" --via anvil"#)]
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

        /// Decode internal functions in Foundry traces
        #[arg(long)]
        decode_internal: bool,

        /// Disable address labels in Foundry traces
        #[arg(long)]
        disable_labels: bool,

        /// Label addresses in Foundry traces (format: address:label, can repeat)
        #[arg(long = "label", alias = "labels", action = clap::ArgAction::Append)]
        labels: Vec<String>,

        /// EVM version for Foundry trace execution
        #[arg(long)]
        evm_version: Option<String>,

        /// Use local Foundry project artifacts for trace decoding
        #[arg(long, alias = "la")]
        with_local_artifacts: bool,

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

        /// RPC timeout in seconds for Foundry calls
        #[arg(long)]
        rpc_timeout: Option<u64>,

        /// RPC header for Foundry calls (format: "Name: value", can repeat)
        #[arg(long = "rpc-header", alias = "rpc-headers", action = clap::ArgAction::Append)]
        rpc_headers: Vec<String>,

        /// Disable automatic proxy detection in Foundry RPC clients
        #[arg(long)]
        no_proxy: bool,

        /// Additional Anvil fork RPC URL (can repeat)
        #[arg(long = "fork-url", action = clap::ArgAction::Append)]
        fork_urls: Vec<String>,

        /// Anvil fork block number or negative offset from latest
        #[arg(long)]
        fork_block_number: Option<String>,

        /// Anvil fork state after a specific transaction hash
        #[arg(long)]
        fork_transaction_hash: Option<String>,

        /// Anvil fork chain ID for offline-start mode
        #[arg(long)]
        fork_chain_id: Option<u64>,

        /// Header for Anvil upstream fork RPC requests (can repeat)
        #[arg(long = "fork-header", alias = "fork-headers", action = clap::ArgAction::Append)]
        fork_headers: Vec<String>,

        /// Anvil hardfork to use (e.g. prague, cancun)
        #[arg(long)]
        hardfork: Option<String>,

        /// Anvil network family to enable (ethereum, optimism, tempo)
        #[arg(long)]
        network: Option<String>,

        /// Disable Anvil fork RPC rate limiting
        #[arg(long)]
        no_rate_limit: bool,

        /// Disable Anvil fork storage caching
        #[arg(long)]
        no_storage_caching: bool,

        /// Anvil upstream fork RPC timeout in milliseconds
        #[arg(long = "fork-timeout-ms", alias = "fork-timeout")]
        fork_timeout_ms: Option<u64>,

        /// Anvil upstream fork retry count
        #[arg(long)]
        fork_retries: Option<u32>,

        /// Disable Anvil block gas limit checks
        #[arg(long)]
        disable_block_gas_limit: bool,

        /// Enable Anvil transaction gas limit checks
        #[arg(long)]
        enable_tx_gas_limit: bool,

        /// Show execution trace (requires debug-capable node for cast)
        #[arg(long, short)]
        trace: bool,

        /// Tenderly credentials
        #[command(flatten)]
        tenderly: TenderlyArgs,

        /// Alchemy credentials
        #[command(flatten)]
        alchemy: AlchemyArgs,

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

        /// Alchemy credentials
        #[command(flatten)]
        alchemy: AlchemyArgs,

        /// Show full opcode trace
        #[arg(long, short)]
        trace: bool,

        /// Open interactive debugger (cast only)
        #[arg(long, short)]
        debug: bool,

        /// Only execute with previous-block state for faster Foundry replay
        #[arg(long)]
        quick: bool,

        /// Decode internal functions in Foundry traces
        #[arg(long)]
        decode_internal: bool,

        /// Limit Foundry trace depth
        #[arg(long)]
        trace_depth: Option<u32>,

        /// Replay system transactions before the target transaction
        #[arg(long)]
        replay_system_txs: bool,

        /// Disable address labels in Foundry traces
        #[arg(long)]
        disable_labels: bool,

        /// Label addresses in Foundry traces (format: address:label, can repeat)
        #[arg(long = "label", alias = "labels", action = clap::ArgAction::Append)]
        labels: Vec<String>,

        /// EVM version for Foundry replay
        #[arg(long)]
        evm_version: Option<String>,

        /// Use local Foundry project artifacts for trace decoding
        #[arg(long, alias = "la")]
        with_local_artifacts: bool,

        /// Disable automatic proxy detection in Foundry RPC clients
        #[arg(long)]
        no_proxy: bool,

        /// RPC timeout in seconds for Foundry replay
        #[arg(long)]
        rpc_timeout: Option<u64>,

        /// RPC header for Foundry replay (format: "Name: value", can repeat)
        #[arg(long = "rpc-header", alias = "rpc-headers", action = clap::ArgAction::Append)]
        rpc_headers: Vec<String>,

        /// Enable transaction gas limit checks in Foundry replay
        #[arg(long)]
        enable_tx_gas_limit: bool,

        /// Disable block gas limit checks in Foundry replay
        #[arg(long)]
        disable_block_gas_limit: bool,

        /// Etherscan API key override for Foundry trace decoding
        #[arg(long = "etherscan-api-key")]
        etherscan_api_key: Option<String>,

        /// Print raw callTracer JSON instead of the decoded call tree (debug backend)
        #[arg(long)]
        raw: bool,
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

pub async fn handle(
    action: &SimulateCommands,
    chain: Chain,
    etherscan_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
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
            nonce_overrides,
            block_timestamp,
            block_number_override,
            via,
            rpc_url,
            rpc_timeout,
            rpc_headers,
            no_proxy,
            fork_urls,
            fork_block_number,
            fork_transaction_hash,
            fork_chain_id,
            fork_headers,
            hardfork,
            network,
            no_rate_limit,
            no_storage_caching,
            fork_timeout_ms,
            fork_retries,
            disable_block_gas_limit,
            enable_tx_gas_limit,
            trace,
            decode_internal,
            disable_labels,
            labels,
            evm_version,
            with_local_artifacts,
            tenderly,
            alchemy,
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
                    if block_coinbase.is_some() || block_difficulty.is_some() {
                        not_supported.push("block header overrides");
                    }
                    if block_base_fee.is_some() && !matches!(via, SimulateVia::Anvil) {
                        not_supported.push("--block-base-fee");
                    }
                    if block_gas_limit.is_some() && !matches!(via, SimulateVia::Anvil) {
                        not_supported.push("--block-gas-limit");
                    }
                    if matches!(via, SimulateVia::Cast)
                        && (!fork_urls.is_empty()
                            || fork_block_number.is_some()
                            || fork_transaction_hash.is_some()
                            || fork_chain_id.is_some()
                            || !fork_headers.is_empty()
                            || hardfork.is_some()
                            || network.is_some()
                            || *no_rate_limit
                            || *no_storage_caching
                            || fork_timeout_ms.is_some()
                            || fork_retries.is_some()
                            || *disable_block_gas_limit
                            || *enable_tx_gas_limit)
                    {
                        not_supported.push("anvil fork options");
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

            let needs_cast_trace = *trace
                || *decode_internal
                || *disable_labels
                || !labels.is_empty()
                || evm_version.is_some()
                || *with_local_artifacts;

            let cast_options = CastCallOptions {
                chain,
                trace: needs_cast_trace,
                gas: *gas,
                gas_price: *gas_price,
                access_list: access_list.clone(),
                balance_overrides: balance_overrides.clone(),
                storage_overrides: storage_overrides.clone(),
                code_overrides: code_overrides.clone(),
                nonce_overrides: nonce_overrides.clone(),
                block_timestamp: *block_timestamp,
                block_number_override: *block_number_override,
                decode_internal: *decode_internal,
                disable_labels: *disable_labels,
                labels: labels.clone(),
                evm_version: evm_version.clone(),
                with_local_artifacts: *with_local_artifacts,
                no_proxy: *no_proxy,
                rpc_timeout: *rpc_timeout,
                rpc_headers: rpc_headers.clone(),
            };

            let anvil_options = AnvilOptions {
                chain,
                fork_urls: fork_urls.clone(),
                fork_block_number: fork_block_number.clone(),
                fork_transaction_hash: fork_transaction_hash.clone(),
                fork_chain_id: *fork_chain_id,
                fork_headers: fork_headers.clone(),
                hardfork: hardfork.clone(),
                network: network.clone(),
                no_rate_limit: *no_rate_limit,
                no_storage_caching: *no_storage_caching,
                timeout_ms: *fork_timeout_ms,
                retries: *fork_retries,
                block_gas_limit: *block_gas_limit,
                block_base_fee: *block_base_fee,
                disable_block_gas_limit: *disable_block_gas_limit,
                enable_tx_gas_limit: *enable_tx_gas_limit,
            };

            let cast_request = CastCallRequest {
                to,
                sig,
                data,
                args,
                from,
                value,
                block,
            };

            match via {
                SimulateVia::Cast => {
                    if dry_run.is_some() {
                        return Err(anyhow::anyhow!("--dry-run not supported for cast backend. Use --via tenderly, debug, or trace"));
                    }
                    simulate_via_cast(cast_request, rpc_url, &cast_options, quiet).await
                }
                SimulateVia::Anvil => {
                    if dry_run.is_some() {
                        return Err(anyhow::anyhow!("--dry-run not supported for anvil backend. Use --via tenderly, debug, or trace"));
                    }
                    simulate_via_anvil(cast_request, rpc_url, &anvil_options, &cast_options, quiet)
                        .await
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
                SimulateVia::Alchemy => {
                    if dry_run.is_some() {
                        return Err(anyhow::anyhow!(
                            "--dry-run not supported for alchemy backend"
                        ));
                    }
                    simulate_via_alchemy(
                        to, sig, data, args, from, value, *gas, *gas_price, alchemy, quiet,
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
            alchemy,
            trace,
            debug,
            quick,
            decode_internal,
            trace_depth,
            replay_system_txs,
            disable_labels,
            labels,
            evm_version,
            with_local_artifacts,
            no_proxy,
            rpc_timeout,
            rpc_headers,
            enable_tx_gas_limit,
            disable_block_gas_limit,
            etherscan_api_key,
            raw,
        } => match via {
            SimulateVia::Cast | SimulateVia::Anvil => {
                let cast_options = CastTxOptions {
                    chain,
                    trace: *trace,
                    debug: *debug,
                    quick: *quick,
                    decode_internal: *decode_internal,
                    trace_depth: *trace_depth,
                    replay_system_txs: *replay_system_txs,
                    disable_labels: *disable_labels,
                    labels: labels.clone(),
                    evm_version: evm_version.clone(),
                    with_local_artifacts: *with_local_artifacts,
                    no_proxy: *no_proxy,
                    rpc_timeout: *rpc_timeout,
                    rpc_headers: rpc_headers.clone(),
                    enable_tx_gas_limit: *enable_tx_gas_limit,
                    disable_block_gas_limit: *disable_block_gas_limit,
                    etherscan_api_key: etherscan_api_key.clone().or_else(|| etherscan_key.clone()),
                };
                trace_tx_via_cast(hash, rpc_url, &cast_options, quiet).await
            }
            SimulateVia::Tenderly => trace_tx_via_tenderly(hash, tenderly, quiet).await,
            SimulateVia::Debug => {
                let key = etherscan_api_key.clone().or_else(|| etherscan_key.clone());
                trace_tx_via_debug_rpc(hash, rpc_url, chain, key, *raw, quiet).await
            }
            SimulateVia::Trace => trace_tx_via_trace_rpc(hash, rpc_url, chain, quiet).await,
            SimulateVia::Alchemy => trace_tx_via_alchemy(hash, alchemy, quiet).await,
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
