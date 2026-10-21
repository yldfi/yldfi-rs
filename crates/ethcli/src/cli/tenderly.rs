//! Tenderly API commands
//!
//! Full Tenderly API access: vnets, wallets, contracts, alerts, actions, networks

use crate::cli::simulate::TenderlyArgs;
use anyhow::Context;
use clap::Subcommand;

/// Validate that a string is a valid Ethereum address (40 hex chars, with optional 0x prefix)
fn validate_address(address: &str) -> anyhow::Result<()> {
    let addr = address.strip_prefix("0x").unwrap_or(address);
    if addr.len() != 40 {
        anyhow::bail!(
            "Invalid address '{}': expected 40 hex characters (with optional 0x prefix)",
            address
        );
    }
    if !addr.chars().all(|c| c.is_ascii_hexdigit()) {
        anyhow::bail!(
            "Invalid address '{}': contains non-hexadecimal characters",
            address
        );
    }
    Ok(())
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum TenderlyCommands {
    /// Transaction simulation (alias for 'ethcli simulate' with --via tenderly)
    #[command(visible_alias = "sim")]
    Simulate {
        #[command(subcommand)]
        action: Box<crate::cli::simulate::SimulateCommands>,
    },

    /// Virtual TestNets management
    Vnets {
        #[command(subcommand)]
        action: VnetsCommands,

        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Wallet monitoring
    Wallets {
        #[command(subcommand)]
        action: WalletsCommands,

        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Contract management
    Contracts {
        #[command(subcommand)]
        action: ContractsCommands,

        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Alert management
    Alerts {
        #[command(subcommand)]
        action: AlertsCommands,

        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Web3 Actions
    Actions {
        #[command(subcommand)]
        action: ActionsCommands,

        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Network information
    Networks {
        #[command(subcommand)]
        action: NetworksCommands,

        #[command(flatten)]
        tenderly: TenderlyArgs,
    },

    /// Delivery channels (Slack, Discord, Email, etc.)
    Channels {
        #[command(subcommand)]
        action: ChannelsCommands,

        #[command(flatten)]
        tenderly: TenderlyArgs,
    },
}

// ============================================================================
// VNets Commands
// ============================================================================

#[derive(Subcommand)]
pub enum VnetsCommands {
    /// Create a new Virtual TestNet
    Create {
        /// Unique slug identifier
        #[arg(long)]
        slug: String,

        /// Display name
        #[arg(long)]
        name: String,

        /// Network ID to fork from (1 = mainnet, 137 = polygon, etc.)
        #[arg(long, default_value = "1")]
        network_id: u64,

        /// Block number to fork from (latest if not specified)
        #[arg(long)]
        block_number: Option<u64>,

        /// Custom chain ID for the VNet
        #[arg(long)]
        chain_id: Option<u64>,

        /// Enable state sync
        #[arg(long)]
        sync_state: bool,
    },

    /// List Virtual TestNets
    List {
        /// Filter by slug prefix
        #[arg(long)]
        slug: Option<String>,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,

        /// Results per page
        #[arg(long, default_value = "20")]
        per_page: u32,
    },

    /// Get Virtual TestNet details
    Get {
        /// VNet ID
        id: String,
    },

    /// Delete Virtual TestNet(s)
    Delete {
        /// VNet ID(s) to delete (can specify multiple)
        #[arg(required_unless_present = "all")]
        ids: Vec<String>,

        /// Delete all Virtual TestNets
        #[arg(long, conflicts_with = "ids")]
        all: bool,
    },

    /// Update a Virtual TestNet
    Update {
        /// VNet ID
        id: String,

        /// New display name
        #[arg(long)]
        name: Option<String>,

        /// New slug
        #[arg(long)]
        slug: Option<String>,

        /// Enable/disable state sync
        #[arg(long)]
        sync_state: Option<bool>,
    },

    /// Get a specific transaction from a Virtual TestNet
    GetTransaction {
        /// VNet ID
        #[arg(long)]
        vnet: String,

        /// Transaction hash
        hash: String,
    },

    /// Fork an existing Virtual TestNet
    Fork {
        /// Source VNet ID
        #[arg(long)]
        source: String,

        /// New slug identifier
        #[arg(long)]
        slug: String,

        /// Display name
        #[arg(long)]
        name: String,

        /// Block number to fork at
        #[arg(long)]
        block_number: Option<u64>,
    },

    /// Get RPC URLs for a Virtual TestNet
    Rpc {
        /// VNet ID
        id: String,
    },

    /// List transactions on a Virtual TestNet
    Transactions {
        /// VNet ID
        id: String,

        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,

        /// Results per page
        #[arg(long, default_value = "20")]
        per_page: u32,
    },

    /// Send a transaction to a Virtual TestNet
    Send {
        /// VNet ID
        #[arg(long)]
        vnet: String,

        /// From address
        #[arg(long)]
        from: String,

        /// To address
        #[arg(long)]
        to: String,

        /// Transaction data (hex)
        #[arg(long)]
        data: Option<String>,

        /// Value in wei
        #[arg(long, default_value = "0")]
        value: String,
    },

    /// Simulate a transaction against a VNet's state (without persisting)
    #[command(visible_alias = "sim")]
    Simulate {
        /// VNet ID
        #[arg(long)]
        vnet: String,

        /// From address (sender)
        #[arg(long)]
        from: String,

        /// To address (recipient/contract, omit for contract deployment)
        #[arg(long)]
        to: Option<String>,

        /// Transaction data (hex)
        #[arg(long)]
        data: Option<String>,

        /// Value in wei
        #[arg(long, default_value = "0")]
        value: String,

        /// Gas limit
        #[arg(long)]
        gas: Option<u64>,

        /// Gas price (legacy)
        #[arg(long)]
        gas_price: Option<String>,

        /// Max fee per gas (EIP-1559)
        #[arg(long)]
        max_fee_per_gas: Option<String>,

        /// Max priority fee per gas (EIP-1559)
        #[arg(long)]
        max_priority_fee_per_gas: Option<String>,
    },

    /// Admin RPC commands (balance, time, storage, snapshots)
    Admin {
        #[command(subcommand)]
        action: AdminCommands,

        /// VNet ID
        #[arg(long)]
        vnet: String,
    },
}

// ============================================================================
// Admin RPC Commands
// ============================================================================

#[derive(Subcommand)]
pub enum AdminCommands {
    /// Set ETH balance for an address
    SetBalance {
        /// Account address
        address: String,

        /// Balance amount (wei, or use suffix: 1eth, 100gwei)
        amount: String,
    },

    /// Add ETH to an address balance
    AddBalance {
        /// Account address
        address: String,

        /// Amount to add (wei, or use suffix: 1eth, 100gwei)
        amount: String,
    },

    /// Set ERC20 token balance for a wallet
    SetErc20Balance {
        /// ERC20 token contract address
        #[arg(long)]
        token: String,

        /// Wallet address
        #[arg(long)]
        wallet: String,

        /// Token amount (in smallest unit, e.g., wei for 18 decimals)
        amount: String,
    },

    /// Set maximum possible ERC20 token balance
    SetMaxErc20Balance {
        /// ERC20 token contract address
        #[arg(long)]
        token: String,

        /// Wallet address
        #[arg(long)]
        wallet: String,
    },

    /// Advance blockchain time by seconds
    IncreaseTime {
        /// Seconds to advance
        seconds: u64,
    },

    /// Set timestamp for the next block (creates empty block)
    SetTimestamp {
        /// Unix epoch timestamp in seconds
        timestamp: u64,
    },

    /// Set timestamp for next block (no empty block)
    SetTimestampNoMine {
        /// Unix epoch timestamp in seconds
        timestamp: u64,
    },

    /// Skip a number of blocks
    IncreaseBlocks {
        /// Number of blocks to skip
        blocks: u64,
    },

    /// Create a state snapshot
    Snapshot,

    /// Revert to a previous snapshot
    Revert {
        /// Snapshot ID
        snapshot_id: String,
    },

    /// Set storage at a specific slot
    SetStorage {
        /// Contract address
        #[arg(long)]
        address: String,

        /// Storage slot (32-byte hex)
        #[arg(long)]
        slot: String,

        /// Value to set (32-byte hex)
        #[arg(long)]
        value: String,
    },

    /// Set bytecode at an address
    SetCode {
        /// Address to set code at
        #[arg(long)]
        address: String,

        /// Bytecode (hex string)
        #[arg(long)]
        code: String,
    },

    /// Send an unsigned transaction
    SendTx {
        /// From address
        #[arg(long)]
        from: String,

        /// To address
        #[arg(long)]
        to: Option<String>,

        /// Transaction data (hex)
        #[arg(long)]
        data: Option<String>,

        /// Value in wei
        #[arg(long)]
        value: Option<String>,

        /// Gas limit
        #[arg(long)]
        gas: Option<String>,
    },

    /// Get the latest transaction ID
    GetLatest,

    /// Simulate a transaction via RPC (tenderly_simulateTransaction)
    ///
    /// Uses the JSON-RPC method which supports state and block overrides.
    /// Returns rich decoded results including logs, traces, and asset changes.
    #[command(visible_alias = "sim-tx")]
    SimulateTx {
        /// From address (sender)
        #[arg(long)]
        from: String,

        /// To address (recipient/contract)
        #[arg(long)]
        to: Option<String>,

        /// Transaction data (hex calldata)
        #[arg(long)]
        data: Option<String>,

        /// Value in wei (hex or decimal)
        #[arg(long)]
        value: Option<String>,

        /// Gas limit (hex or decimal)
        #[arg(long)]
        gas: Option<String>,

        /// Block tag or number (default: "latest")
        #[arg(long, default_value = "latest")]
        block: String,

        /// State overrides as JSON: {"0xaddr": {"balance": "0x...", "stateDiff": {"0x0": "0x1"}}}
        #[arg(long)]
        state_overrides: Option<String>,

        /// Block overrides as JSON: {"time": "0x...", "baseFee": "0x..."}
        #[arg(long)]
        block_overrides: Option<String>,
    },

    /// Simulate a bundle of transactions via RPC (tenderly_simulateBundle)
    ///
    /// Executes multiple transactions sequentially, each seeing state changes
    /// from previous transactions. Supports state and block overrides.
    #[command(visible_alias = "sim-bundle")]
    SimulateBundle {
        /// Transactions as JSON array: [{"from":"0x...","to":"0x...","data":"0x..."}]
        txs: String,

        /// State overrides as JSON
        #[arg(long)]
        state_overrides: Option<String>,

        /// Block overrides as JSON
        #[arg(long)]
        block_overrides: Option<String>,
    },

    /// Set the same balance for multiple addresses at once
    SetBalances {
        /// Addresses (space-separated)
        #[arg(required = true)]
        addresses: Vec<String>,

        /// Amount (e.g., "10eth", "100gwei", or raw hex)
        #[arg(long)]
        amount: String,
    },

    /// Add the same amount to multiple addresses at once
    AddBalances {
        /// Addresses (space-separated)
        #[arg(required = true)]
        addresses: Vec<String>,

        /// Amount to add (e.g., "1eth", "100gwei", or raw hex)
        #[arg(long)]
        amount: String,
    },

    /// Create an EIP-2930 access list for a transaction
    CreateAccessList {
        /// From address
        #[arg(long)]
        from: String,

        /// To address
        #[arg(long)]
        to: Option<String>,

        /// Transaction data (hex)
        #[arg(long)]
        data: Option<String>,

        /// Value in wei
        #[arg(long)]
        value: Option<String>,

        /// Gas limit
        #[arg(long)]
        gas: Option<String>,

        /// Block tag or number (default: "latest")
        #[arg(long, default_value = "latest")]
        block: String,
    },
}

// ============================================================================
// Wallets Commands
// ============================================================================

#[derive(Subcommand)]
pub enum WalletsCommands {
    /// Add a wallet for monitoring
    Add {
        /// Wallet address
        address: String,

        /// Display name
        #[arg(long)]
        name: Option<String>,

        /// Network IDs to monitor on (can be repeated)
        #[arg(long = "network", action = clap::ArgAction::Append)]
        networks: Vec<String>,
    },

    /// List monitored wallets
    List,

    /// Get wallet details on a network
    Get {
        /// Wallet address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,
    },
}

// ============================================================================
// Contracts Commands
// ============================================================================

#[derive(Subcommand)]
pub enum ContractsCommands {
    /// Add a contract for monitoring
    Add {
        /// Contract address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,

        /// Display name
        #[arg(long)]
        name: Option<String>,
    },

    /// List monitored contracts
    List,

    /// Get contract details
    Get {
        /// Contract address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,
    },

    /// Delete a contract from monitoring
    Delete {
        /// Contract address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,
    },

    /// Add a tag to a contract
    Tag {
        /// Contract address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,

        /// Tag to add
        #[arg(long)]
        tag: String,
    },

    /// Rename a contract
    Rename {
        /// Contract address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,

        /// New display name
        #[arg(long)]
        name: String,
    },

    /// Remove a tag from a contract
    RemoveTag {
        /// Contract address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,

        /// Tag to remove
        tag: String,
    },

    /// Delete a tag from a contract (alias for remove-tag)
    DeleteTag {
        /// Contract address
        address: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,

        /// Tag to delete
        tag: String,
    },

    /// Add a tag to multiple contracts at once
    BulkTag {
        /// Tag to apply
        tag: String,

        /// Contract IDs in format "eth:1:0xAddress" (can be repeated)
        #[arg(required = true)]
        contract_ids: Vec<String>,
    },

    /// Encode named-variable state overrides into raw storage slots
    /// (POST /contracts/encode-states)
    EncodeState {
        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,

        /// Block number to encode against (default: latest)
        #[arg(long)]
        block_number: Option<String>,

        /// State overrides as JSON: {"0xAddr": {"value": {"balances[0xHolder]": "1000"}}}
        state_json: String,
    },
}

// ============================================================================
// Alerts Commands
// ============================================================================

#[derive(Subcommand)]
pub enum AlertsCommands {
    /// Create a new alert
    Create {
        /// Alert name
        #[arg(long)]
        name: String,

        /// Alert type (simple_type): successful_tx, failed_tx, allowlisted_callers,
        /// blocklisted_callers, log_emitted, erc20_transfer_log_emitted,
        /// emitted_log_parameter, function_called, called_function_parameter,
        /// state_change, eth_balance, tx_value, view_function
        #[arg(long)]
        alert_type: String,

        /// Alert target type: address, network, project, tag
        #[arg(long, default_value = "address")]
        target_type: String,

        /// Target addresses (can be repeated)
        #[arg(long = "address", action = clap::ArgAction::Append)]
        addresses: Vec<String>,

        /// Contract tag (for --target-type tag)
        #[arg(long)]
        tag: Option<String>,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,

        /// Extra expression as JSON, e.g. '{"type":"method_call","expression":{...}}'
        /// (can be repeated)
        #[arg(long = "expression", action = clap::ArgAction::Append)]
        expressions: Vec<String>,

        /// Delivery channel ID to notify (can be repeated; see `tenderly channels`)
        #[arg(long = "channel", action = clap::ArgAction::Append)]
        channels: Vec<String>,
    },

    /// List alerts
    List,

    /// Get alert details
    Get {
        /// Alert ID
        id: String,
    },

    /// Delete an alert
    Delete {
        /// Alert ID
        id: String,
    },

    /// Enable an alert
    Enable {
        /// Alert ID
        id: String,
    },

    /// Disable an alert
    Disable {
        /// Alert ID
        id: String,
    },

    /// View alert history
    History {
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,

        /// Results per page
        #[arg(long, default_value = "20")]
        per_page: u32,
    },

    /// Test an alert
    Test {
        /// Alert ID
        id: String,

        /// Transaction hash to test with
        #[arg(long)]
        tx_hash: String,

        /// Network ID
        #[arg(long, default_value = "1")]
        network: String,
    },

    /// Update an alert
    ///
    /// Starts from the existing alert, so its enabled state, description,
    /// color, target and channels are kept unless overridden here.
    Update {
        /// Alert ID
        id: String,

        /// New alert name (default: keep)
        #[arg(long)]
        name: Option<String>,

        /// Alert type (simple_type, see `alerts create --help`); required
        /// because the API does not return it
        #[arg(long)]
        alert_type: String,

        /// Network ID (default: keep)
        #[arg(long)]
        network: Option<String>,

        /// Target addresses, replacing the current address/tag target (can be repeated)
        #[arg(long = "address", action = clap::ArgAction::Append)]
        addresses: Vec<String>,

        /// Extra expression as JSON, replacing the current non-target
        /// conditions (can be repeated)
        #[arg(long = "expression", action = clap::ArgAction::Append)]
        expressions: Vec<String>,

        /// Delivery channel ID to notify, replacing the current channels (can be repeated)
        #[arg(long = "channel", action = clap::ArgAction::Append)]
        channels: Vec<String>,
    },

    /// Manage webhooks
    Webhooks {
        #[command(subcommand)]
        action: WebhookCommands,
    },
}

#[derive(Subcommand)]
pub enum WebhookCommands {
    /// Create a webhook
    Create {
        /// Webhook name
        #[arg(long)]
        name: String,

        /// Webhook URL
        #[arg(long)]
        url: String,
    },

    /// List webhooks
    List,

    /// Get webhook details
    Get {
        /// Webhook ID
        id: String,
    },

    /// Delete a webhook
    Delete {
        /// Webhook ID
        id: String,
    },
}

// ============================================================================
// Actions Commands
// ============================================================================

#[derive(Subcommand)]
pub enum ActionsCommands {
    /// Create a new Web3 Action
    Create {
        /// Action name
        #[arg(long)]
        name: String,

        /// Trigger type: alert, webhook, periodic, block, transaction, transactionsimple
        #[arg(long)]
        trigger: String,

        /// Source code file path
        #[arg(long)]
        source: String,

        /// Trigger configuration as JSON, e.g.
        /// '{"type":"periodic","periodic":{"cron":"0 * * * *"}}' (default: {"type":"<trigger>"})
        #[arg(long)]
        trigger_config: Option<String>,

        /// Exported function to invoke
        #[arg(long, default_value = tndrly::actions::DEFAULT_FUNCTION)]
        function: String,
    },

    /// List Web3 Actions
    List,

    /// Get Action details
    Get {
        /// Action ID
        id: String,
    },

    /// Delete an Action
    Delete {
        /// Action ID
        id: String,
    },

    /// Stop an Action
    Stop {
        /// Action ID
        id: String,
    },

    /// Resume a stopped Action
    Resume {
        /// Action ID
        id: String,
    },

    /// Stop multiple Actions (pass IDs, or none to stop all)
    StopMany {
        /// Action IDs (if empty, stops all actions in project)
        ids: Vec<String>,
    },

    /// Resume multiple Actions (pass IDs, or none to resume all)
    ResumeMany {
        /// Action IDs (if empty, resumes all actions in project)
        ids: Vec<String>,
    },

    /// Get execution details for an Action call
    GetCall {
        /// Action ID
        id: String,
        /// Execution/call ID
        execution_id: String,
    },
}

// ============================================================================
// Networks Commands
// ============================================================================

#[derive(Subcommand)]
pub enum NetworksCommands {
    /// List all supported networks
    List,

    /// Get network by ID
    Get {
        /// Network ID (e.g., 1, 137)
        id: u64,
    },

    /// List mainnet networks only
    Mainnets,

    /// List testnet networks only
    Testnets,

    /// List networks with simulation support
    Simulation,

    /// List networks with VNet support
    Vnet,
}

// ============================================================================
// Channels Commands
// ============================================================================

#[derive(Subcommand)]
pub enum ChannelsCommands {
    /// List all delivery channels (account + project)
    List,

    /// List account-level delivery channels
    Account,

    /// List project-level delivery channels
    Project,
}

// ============================================================================
// Handler
// ============================================================================

pub async fn handle(
    cmd: &TenderlyCommands,
    chain: crate::config::Chain,
    etherscan_key: Option<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    match cmd {
        TenderlyCommands::Simulate { action } => {
            // Delegate to the simulate handler, forcing the Tenderly backend
            // (this command is documented as `ethcli simulate --via tenderly`)
            crate::cli::simulate::handle_with_via(
                action,
                Some(crate::cli::simulate::SimulateVia::Tenderly),
                chain,
                etherscan_key,
                quiet,
            )
            .await
        }
        TenderlyCommands::Vnets { action, tenderly } => handle_vnets(action, tenderly, quiet).await,
        TenderlyCommands::Wallets { action, tenderly } => {
            handle_wallets(action, tenderly, quiet).await
        }
        TenderlyCommands::Contracts { action, tenderly } => {
            handle_contracts(action, tenderly, quiet).await
        }
        TenderlyCommands::Alerts { action, tenderly } => {
            handle_alerts(action, tenderly, quiet).await
        }
        TenderlyCommands::Actions { action, tenderly } => {
            handle_actions(action, tenderly, quiet).await
        }
        TenderlyCommands::Networks { action, tenderly } => {
            handle_networks(action, tenderly, quiet).await
        }
        TenderlyCommands::Channels { action, tenderly } => {
            handle_channels(action, tenderly, quiet).await
        }
    }
}

// ============================================================================
// VNets Handler
// ============================================================================

async fn handle_vnets(
    cmd: &VnetsCommands,
    tenderly: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = tenderly.create_client()?;

    match cmd {
        VnetsCommands::Create {
            slug,
            name,
            network_id,
            block_number,
            chain_id,
            sync_state,
        } => {
            if !quiet {
                eprintln!("Creating Virtual TestNet '{}'...", slug);
            }

            let mut request = tndrly::vnets::CreateVNetRequest::new(slug, name, *network_id);

            if let Some(bn) = block_number {
                request = request.block_number(*bn);
            }
            if let Some(cid) = chain_id {
                request = request.chain_id(*cid);
            }
            if *sync_state {
                request = request.sync_state(true);
            }

            let vnet = client.vnets().create(&request).await?;
            println!("{}", serde_json::to_string_pretty(&vnet)?);
        }

        VnetsCommands::List {
            slug,
            page,
            per_page,
        } => {
            if !quiet {
                eprintln!("Listing Virtual TestNets...");
            }

            let mut query = tndrly::vnets::ListVNetsQuery::new()
                .page(*page)
                .per_page(*per_page);

            if let Some(s) = slug {
                query = query.slug(s);
            }

            let vnets = client.vnets().list(Some(query)).await?;
            println!("{}", serde_json::to_string_pretty(&vnets)?);
        }

        VnetsCommands::Get { id } => {
            if !quiet {
                eprintln!("Getting Virtual TestNet {}...", id);
            }
            let vnet = client.vnets().get(id).await?;
            println!("{}", serde_json::to_string_pretty(&vnet)?);
        }

        VnetsCommands::Delete { ids, all } => {
            if *all {
                if !quiet {
                    eprintln!("Fetching all Virtual TestNets...");
                }
                let vnets = client.vnets().list(None).await?;
                if vnets.is_empty() {
                    println!("No Virtual TestNets to delete.");
                    return Ok(());
                }
                let vnet_ids: Vec<String> = vnets.into_iter().map(|v| v.id).collect();
                let count = vnet_ids.len();
                if !quiet {
                    eprintln!("Deleting {} Virtual TestNets...", count);
                }
                client.vnets().delete_many(vnet_ids).await?;
                println!("{} Virtual TestNets deleted.", count);
            } else if ids.len() == 1 {
                let id = &ids[0];
                if !quiet {
                    eprintln!("Deleting Virtual TestNet {}...", id);
                }
                client.vnets().delete(id).await?;
                println!("Virtual TestNet {} deleted.", id);
            } else {
                if !quiet {
                    eprintln!("Deleting {} Virtual TestNets...", ids.len());
                }
                client.vnets().delete_many(ids.clone()).await?;
                println!("{} Virtual TestNets deleted.", ids.len());
            }
        }

        VnetsCommands::Update {
            id,
            name,
            slug,
            sync_state,
        } => {
            if !quiet {
                eprintln!("Updating Virtual TestNet {}...", id);
            }

            let mut request = tndrly::vnets::UpdateVNetRequest::new();

            if let Some(n) = name {
                request = request.display_name(n);
            }
            if let Some(s) = slug {
                request = request.slug(s);
            }
            if let Some(ss) = sync_state {
                request = request.sync_state(*ss);
            }

            let vnet = client.vnets().update(id, &request).await?;
            println!("{}", serde_json::to_string_pretty(&vnet)?);
        }

        VnetsCommands::GetTransaction { vnet, hash } => {
            if !quiet {
                eprintln!("Getting transaction {} from VNet {}...", hash, vnet);
            }
            let tx = client.vnets().get_transaction(vnet, hash).await?;
            println!("{}", serde_json::to_string_pretty(&tx)?);
        }

        VnetsCommands::Fork {
            source,
            slug,
            name,
            block_number,
        } => {
            if !quiet {
                eprintln!("Forking Virtual TestNet {} as '{}'...", source, slug);
            }

            let mut request = tndrly::vnets::ForkVNetRequest::new(source, slug, name);

            if let Some(bn) = block_number {
                request = request.block_number(*bn);
            }

            let vnet = client.vnets().fork(&request).await?;
            println!("{}", serde_json::to_string_pretty(&vnet)?);
        }

        VnetsCommands::Rpc { id } => {
            if !quiet {
                eprintln!("Getting RPC URLs for VNet {}...", id);
            }
            let rpcs = client.vnets().rpc_urls(id).await?;
            println!("{}", serde_json::to_string_pretty(&rpcs)?);
        }

        VnetsCommands::Transactions { id, page, per_page } => {
            if !quiet {
                eprintln!("Listing transactions on VNet {}...", id);
            }

            let query = tndrly::vnets::ListVNetTransactionsQuery::new()
                .page(*page)
                .per_page(*per_page);

            let txs = client.vnets().transactions(id, Some(query)).await?;
            println!("{}", serde_json::to_string_pretty(&txs)?);
        }

        VnetsCommands::Send {
            vnet,
            from,
            to,
            data,
            value,
        } => {
            if !quiet {
                eprintln!("Sending transaction on VNet {}...", vnet);
            }

            let mut request = tndrly::vnets::SendVNetTransactionRequest::new(
                from,
                to,
                data.as_deref().unwrap_or("0x"),
            );

            if value != "0" {
                request = request.value(value);
            }

            let tx = client.vnets().send_transaction(vnet, &request).await?;
            println!("{}", serde_json::to_string_pretty(&tx)?);
        }

        VnetsCommands::Simulate {
            vnet,
            from,
            to,
            data,
            value,
            gas,
            gas_price,
            max_fee_per_gas,
            max_priority_fee_per_gas,
        } => {
            validate_address(from)?;
            if let Some(to_addr) = to {
                validate_address(to_addr)?;
            }

            if !quiet {
                eprintln!("Simulating transaction on VNet {}...", vnet);
            }

            let mut request =
                tndrly::vnets::VNetSimulationRequest::new(from, data.as_deref().unwrap_or("0x"));
            if let Some(t) = to {
                request = request.to(t);
            }

            if value != "0" {
                request = request.value(value);
            }
            if let Some(g) = gas {
                request = request.gas(*g);
            }
            if let Some(gp) = gas_price {
                request.gas_price = Some(gp.clone());
            }
            if let Some(mfpg) = max_fee_per_gas {
                request = request.max_fee_per_gas(mfpg);
            }
            if let Some(mpfpg) = max_priority_fee_per_gas {
                request = request.max_priority_fee_per_gas(mpfpg);
            }

            let result = client.vnets().simulate(vnet, &request).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        VnetsCommands::Admin { action, vnet } => {
            handle_admin(action, vnet, &client, quiet).await?;
        }
    }

    Ok(())
}

// ============================================================================
// Admin RPC Handler
// ============================================================================

async fn handle_admin(
    cmd: &AdminCommands,
    vnet_id: &str,
    client: &tndrly::Client,
    quiet: bool,
) -> anyhow::Result<()> {
    if !quiet {
        eprintln!("Connecting to Admin RPC for VNet {}...", vnet_id);
    }

    let admin = client.vnets().admin_rpc(vnet_id).await?;

    match cmd {
        AdminCommands::SetBalance { address, amount } => {
            validate_address(address)?;
            let amount_wei = parse_eth_amount(amount)?;
            if !quiet {
                eprintln!("Setting balance for {} to {} wei...", address, amount_wei);
            }
            let hash = admin.set_balance(address, &amount_wei).await?;
            println!("{}", hash);
        }

        AdminCommands::AddBalance { address, amount } => {
            validate_address(address)?;
            let amount_wei = parse_eth_amount(amount)?;
            if !quiet {
                eprintln!("Adding {} wei to {}...", amount_wei, address);
            }
            let hash = admin.add_balance(address, &amount_wei).await?;
            println!("{}", hash);
        }

        AdminCommands::SetErc20Balance {
            token,
            wallet,
            amount,
        } => {
            validate_address(token)?;
            validate_address(wallet)?;
            if !quiet {
                eprintln!("Setting ERC20 balance for {} on {}...", wallet, token);
            }
            let hash = admin.set_erc20_balance(token, wallet, amount).await?;
            println!("{}", hash);
        }

        AdminCommands::SetMaxErc20Balance { token, wallet } => {
            validate_address(token)?;
            validate_address(wallet)?;
            if !quiet {
                eprintln!("Setting max ERC20 balance for {} on {}...", wallet, token);
            }
            let hash = admin.set_max_erc20_balance(token, wallet).await?;
            println!("{}", hash);
        }

        AdminCommands::IncreaseTime { seconds } => {
            if !quiet {
                eprintln!("Advancing time by {} seconds...", seconds);
            }
            let hash = admin.increase_time(*seconds).await?;
            println!("{}", hash);
        }

        AdminCommands::SetTimestamp { timestamp } => {
            if !quiet {
                eprintln!("Setting next block timestamp to {}...", timestamp);
            }
            let result = admin.set_next_block_timestamp(*timestamp).await?;
            println!("{}", result);
        }

        AdminCommands::SetTimestampNoMine { timestamp } => {
            if !quiet {
                eprintln!("Setting next block timestamp to {} (no mine)...", timestamp);
            }
            let result = admin.set_next_block_timestamp_no_mine(*timestamp).await?;
            println!("{}", result);
        }

        AdminCommands::IncreaseBlocks { blocks } => {
            if !quiet {
                eprintln!("Skipping {} blocks...", blocks);
            }
            let hash = admin.increase_blocks(*blocks).await?;
            println!("{}", hash);
        }

        AdminCommands::Snapshot => {
            if !quiet {
                eprintln!("Creating snapshot...");
            }
            let snapshot_id = admin.snapshot().await?;
            println!("{}", snapshot_id);
        }

        AdminCommands::Revert { snapshot_id } => {
            if !quiet {
                eprintln!("Reverting to snapshot {}...", snapshot_id);
            }
            let success = admin.revert(snapshot_id).await?;
            if success {
                println!("Reverted successfully");
            } else {
                anyhow::bail!("Revert failed");
            }
        }

        AdminCommands::SetStorage {
            address,
            slot,
            value,
        } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Setting storage at {} slot {}...", address, slot);
            }
            let hash = admin.set_storage_at(address, slot, value).await?;
            println!("{}", hash);
        }

        AdminCommands::SetCode { address, code } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Setting code at {}...", address);
            }
            let hash = admin.set_code(address, code).await?;
            println!("{}", hash);
        }

        AdminCommands::SendTx {
            from,
            to,
            data,
            value,
            gas,
        } => {
            validate_address(from)?;
            if let Some(to_addr) = to {
                validate_address(to_addr)?;
            }
            if !quiet {
                eprintln!("Sending transaction from {}...", from);
            }

            let mut tx = tndrly::vnets::admin_rpc::SendTransactionParams::new(from);
            if let Some(to_addr) = to {
                tx = tx.to(to_addr);
            }
            if let Some(d) = data {
                tx = tx.data(d);
            }
            if let Some(v) = value {
                tx = tx.value(v);
            }
            if let Some(g) = gas {
                tx = tx.gas(g);
            }

            let hash = admin.send_transaction(&tx).await?;
            println!("{}", hash);
        }

        AdminCommands::GetLatest => {
            if !quiet {
                eprintln!("Getting latest block info...");
            }
            let block = admin.get_latest().await?;
            // Build JSON manually since LatestBlock doesn't derive Serialize
            let mut json = serde_json::Map::new();
            if let Some(num) = &block.block_number {
                json.insert("blockNumber".to_string(), serde_json::json!(num));
            }
            if let Some(hash) = &block.block_hash {
                json.insert("blockHash".to_string(), serde_json::json!(hash));
            }
            if let Some(tx) = &block.transaction_hash {
                json.insert("transactionHash".to_string(), serde_json::json!(tx));
            }
            for (k, v) in &block.extra {
                json.insert(k.clone(), v.clone());
            }
            println!("{}", serde_json::to_string_pretty(&json)?);
        }

        AdminCommands::SimulateTx {
            from,
            to,
            data,
            value,
            gas,
            block,
            state_overrides,
            block_overrides,
        } => {
            validate_address(from)?;
            if let Some(to_addr) = to {
                validate_address(to_addr)?;
            }
            if !quiet {
                eprintln!("Simulating transaction via RPC (tenderly_simulateTransaction)...");
            }

            let mut tx = tndrly::vnets::SimulateTransactionParams::new(from);
            if let Some(t) = to {
                tx = tx.to(t);
            }
            if let Some(d) = data {
                tx = tx.data(d);
            }
            if let Some(v) = value {
                tx = tx.value(v);
            }
            if let Some(g) = gas {
                tx = tx.gas(g);
            }

            let so: Option<std::collections::HashMap<String, tndrly::vnets::StateOverride>> =
                state_overrides
                    .as_ref()
                    .map(|s| serde_json::from_str(s))
                    .transpose()
                    .context("Invalid --state-overrides JSON")?;
            let bo: Option<tndrly::vnets::BlockOverride> = block_overrides
                .as_ref()
                .map(|s| serde_json::from_str(s))
                .transpose()
                .context("Invalid --block-overrides JSON")?;

            let result = admin
                .simulate_transaction(&tx, block, so.as_ref(), bo.as_ref())
                .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        AdminCommands::SimulateBundle {
            txs,
            state_overrides,
            block_overrides,
        } => {
            if !quiet {
                eprintln!("Simulating transaction bundle via RPC (tenderly_simulateBundle)...");
            }

            let txs: Vec<tndrly::vnets::SimulateTransactionParams> =
                serde_json::from_str(txs).context("Invalid --txs JSON array")?;

            let so: Option<std::collections::HashMap<String, tndrly::vnets::StateOverride>> =
                state_overrides
                    .as_ref()
                    .map(|s| serde_json::from_str(s))
                    .transpose()
                    .context("Invalid --state-overrides JSON")?;
            let bo: Option<tndrly::vnets::BlockOverride> = block_overrides
                .as_ref()
                .map(|s| serde_json::from_str(s))
                .transpose()
                .context("Invalid --block-overrides JSON")?;

            let result = admin
                .simulate_bundle(&txs, so.as_ref(), bo.as_ref())
                .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        AdminCommands::SetBalances { addresses, amount } => {
            if !quiet {
                eprintln!("Setting balance for {} addresses...", addresses.len());
            }
            let parsed = parse_eth_amount(amount)?;
            let addr_refs: Vec<&str> = addresses.iter().map(|s| s.as_str()).collect();
            let result = admin.set_balances(&addr_refs, &parsed).await?;
            println!("{}", result);
        }

        AdminCommands::AddBalances { addresses, amount } => {
            if !quiet {
                eprintln!("Adding balance to {} addresses...", addresses.len());
            }
            let parsed = parse_eth_amount(amount)?;
            let addr_refs: Vec<&str> = addresses.iter().map(|s| s.as_str()).collect();
            let result = admin.add_balances(&addr_refs, &parsed).await?;
            println!("{}", result);
        }

        AdminCommands::CreateAccessList {
            from,
            to,
            data,
            value,
            gas,
            block,
        } => {
            if !quiet {
                eprintln!("Creating access list...");
            }
            let mut tx = tndrly::vnets::SendTransactionParams::new(from);
            if let Some(t) = to {
                tx.to = Some(t.clone());
            }
            if let Some(d) = data {
                tx.data = Some(d.clone());
            }
            if let Some(v) = value {
                tx.value = Some(v.clone());
            }
            if let Some(g) = gas {
                tx.gas = Some(g.clone());
            }
            let result = admin.create_access_list(&tx, block).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

/// Parse ETH amount with optional suffix (e.g., "1eth", "100gwei", "1000000000000000000")
fn parse_eth_amount(amount: &str) -> anyhow::Result<String> {
    let amount_lower = amount.to_lowercase();

    if amount_lower.ends_with("eth") || amount_lower.ends_with("ether") {
        let num_str = amount_lower
            .trim_end_matches("ether")
            .trim_end_matches("eth");
        let num: f64 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid ETH amount: {}", amount))?;
        let wei = (num * 1e18) as u128;
        Ok(wei.to_string())
    } else if amount_lower.ends_with("gwei") {
        let num_str = amount_lower.trim_end_matches("gwei");
        let num: f64 = num_str
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid gwei amount: {}", amount))?;
        let wei = (num * 1e9) as u128;
        Ok(wei.to_string())
    } else if amount_lower.ends_with("wei") {
        let num_str = amount_lower.trim_end_matches("wei");
        Ok(num_str.to_string())
    } else {
        // Assume wei if no suffix
        Ok(amount.to_string())
    }
}

// ============================================================================
// Wallets Handler
// ============================================================================

async fn handle_wallets(
    cmd: &WalletsCommands,
    tenderly: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = tenderly.create_client()?;

    match cmd {
        WalletsCommands::Add {
            address,
            name,
            networks,
        } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Adding wallet {}...", address);
            }

            let mut request = tndrly::wallets::AddWalletRequest::new(address);

            if let Some(n) = name {
                request = request.display_name(n);
            }

            for network in networks {
                request = request.network(network);
            }

            let result = client.wallets().add(&request).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        WalletsCommands::List => {
            if !quiet {
                eprintln!("Listing wallets...");
            }
            let wallets = client.wallets().list().await?;
            println!("{}", serde_json::to_string_pretty(&wallets)?);
        }

        WalletsCommands::Get { address, network } => {
            if !quiet {
                eprintln!("Getting wallet {} on network {}...", address, network);
            }
            let wallet = client.wallets().get(address, network).await?;
            println!("{}", serde_json::to_string_pretty(&wallet)?);
        }
    }

    Ok(())
}

// ============================================================================
// Contracts Handler
// ============================================================================

async fn handle_contracts(
    cmd: &ContractsCommands,
    tenderly: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = tenderly.create_client()?;

    match cmd {
        ContractsCommands::Add {
            address,
            network,
            name,
        } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Adding contract {} on network {}...", address, network);
            }

            let mut request = tndrly::contracts::AddContractRequest::new(network, address);

            if let Some(n) = name {
                request = request.display_name(n);
            }

            let result = client.contracts().add(&request).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        ContractsCommands::List => {
            if !quiet {
                eprintln!("Listing contracts...");
            }
            let query = tndrly::contracts::ListContractsQuery::new(
                tndrly::contracts::AccountType::Contract,
            );
            let contracts = client.contracts().list(&query).await?;
            println!("{}", serde_json::to_string_pretty(&contracts)?);
        }

        ContractsCommands::Get { address, network } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Getting contract {} on network {}...", address, network);
            }
            let contract = client.contracts().get(network, address).await?;
            println!("{}", serde_json::to_string_pretty(&contract)?);
        }

        ContractsCommands::Delete { address, network } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Deleting contract {} on network {}...", address, network);
            }
            client.contracts().delete(network, address).await?;
            println!("Contract {} deleted.", address);
        }

        ContractsCommands::Tag {
            address,
            network,
            tag,
        } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Adding tag '{}' to contract {}...", tag, address);
            }
            client.contracts().add_tag(network, address, tag).await?;
            println!("Tag '{}' added to contract {}.", tag, address);
        }

        ContractsCommands::Rename {
            address,
            network,
            name,
        } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Renaming contract {} to '{}'...", address, name);
            }
            client.contracts().rename(network, address, name).await?;
            println!("Contract renamed successfully");
        }

        ContractsCommands::RemoveTag {
            address,
            network,
            tag,
        } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Removing tag '{}' from contract {}...", tag, address);
            }
            client.contracts().remove_tag(network, address, tag).await?;
            println!("Tag '{}' removed from contract {}.", tag, address);
        }

        ContractsCommands::DeleteTag {
            address,
            network,
            tag,
        } => {
            validate_address(address)?;
            if !quiet {
                eprintln!("Deleting tag '{}' from contract {}...", tag, address);
            }
            client.contracts().delete_tag(network, address, tag).await?;
            println!("Tag '{}' deleted from contract {}.", tag, address);
        }

        ContractsCommands::BulkTag { tag, contract_ids } => {
            if !quiet {
                eprintln!(
                    "Applying tag '{}' to {} contracts...",
                    tag,
                    contract_ids.len()
                );
            }
            let result = client
                .contracts()
                .bulk_tag(tag, contract_ids.clone())
                .await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        ContractsCommands::EncodeState {
            network,
            block_number,
            state_json,
        } => {
            if !quiet {
                eprintln!("Encoding state overrides...");
            }
            let state_overrides: std::collections::HashMap<
                String,
                tndrly::contracts::StateOverrideInput,
            > = serde_json::from_str(state_json).context("Invalid state overrides JSON")?;
            let mut request =
                tndrly::contracts::EncodeStateRequest::new(network.clone(), state_overrides);
            if let Some(block) = block_number {
                request = request.block_number(block.clone());
            }
            let result = client.contracts().encode_state(&request).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
    }

    Ok(())
}

// ============================================================================
// Alerts Handler
// ============================================================================

async fn handle_alerts(
    cmd: &AlertsCommands,
    tenderly: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = tenderly.create_client()?;

    match cmd {
        AlertsCommands::Create {
            name,
            alert_type,
            target_type,
            addresses,
            tag,
            network,
            expressions,
            channels,
        } => {
            if !quiet {
                eprintln!("Creating alert '{}'...", name);
            }

            let request = build_alert_request(
                name,
                alert_type,
                target_type,
                addresses,
                tag.as_deref(),
                network,
                expressions,
                channels,
            )?;
            let alert = client.alerts().create(&request).await?;
            println!("{}", serde_json::to_string_pretty(&alert)?);
        }

        AlertsCommands::List => {
            if !quiet {
                eprintln!("Listing alerts...");
            }
            let alerts = client.alerts().list().await?;
            println!("{}", serde_json::to_string_pretty(&alerts)?);
        }

        AlertsCommands::Get { id } => {
            if !quiet {
                eprintln!("Getting alert {}...", id);
            }
            let alert = client.alerts().get(id).await?;
            println!("{}", serde_json::to_string_pretty(&alert)?);
        }

        AlertsCommands::Delete { id } => {
            if !quiet {
                eprintln!("Deleting alert {}...", id);
            }
            client.alerts().delete(id).await?;
            println!("Alert {} deleted.", id);
        }

        AlertsCommands::Enable { id } => {
            if !quiet {
                eprintln!("Enabling alert {}...", id);
            }
            client.alerts().enable(id).await?;
            println!("Alert {} enabled.", id);
        }

        AlertsCommands::Disable { id } => {
            if !quiet {
                eprintln!("Disabling alert {}...", id);
            }
            client.alerts().disable(id).await?;
            println!("Alert {} disabled.", id);
        }

        AlertsCommands::History { page, per_page } => {
            if !quiet {
                eprintln!("Getting alert history...");
            }
            let query = tndrly::alerts::AlertHistoryQuery::new()
                .page(*page)
                .per_page(*per_page);
            let history = client.alerts().history(Some(query)).await?;
            println!("{}", serde_json::to_string_pretty(&history)?);
        }

        AlertsCommands::Test {
            id,
            tx_hash,
            network,
        } => {
            if !quiet {
                eprintln!("Testing alert {}...", id);
            }
            let request = tndrly::alerts::TestAlertRequest {
                alert_id: id.clone(),
                network_id: network.clone(),
                tx_hash: tx_hash.clone(),
            };
            client.alerts().test_alert(&request).await?;
            println!("Alert test triggered for {}.", id);
        }

        AlertsCommands::Update {
            id,
            name,
            alert_type,
            network,
            addresses,
            expressions,
            channels,
        } => {
            if !quiet {
                eprintln!("Updating alert {}...", id);
            }
            // PATCH replaces the whole alert, so merge onto the current one
            let existing = client
                .alerts()
                .list()
                .await?
                .alerts
                .into_iter()
                .find(|a| a.id == *id)
                .ok_or_else(|| anyhow::anyhow!("Alert {} not found", id))?;
            let request = merge_alert_update(
                &existing,
                name.as_deref(),
                alert_type,
                network.as_deref(),
                addresses,
                expressions,
                channels,
            )?;
            let result = client.alerts().update(id, &request).await?;
            println!("{}", serde_json::to_string_pretty(&result)?);
        }

        AlertsCommands::Webhooks { action } => {
            handle_webhooks(action, &client, quiet).await?;
        }
    }

    Ok(())
}

/// Build an alert request (`simple_type` + expressions + delivery channels) from CLI args
#[allow(clippy::too_many_arguments)]
fn build_alert_request(
    name: &str,
    alert_type: &str,
    target_type: &str,
    addresses: &[String],
    tag: Option<&str>,
    network: &str,
    expressions: &[String],
    channels: &[String],
) -> anyhow::Result<tndrly::alerts::CreateAlertRequest> {
    use tndrly::alerts::{AlertExpression, AlertType, CreateAlertRequest};

    let simple_type: AlertType = alert_type
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid alert type '{}': {}", alert_type, e))?;
    let mut request = CreateAlertRequest::new(name, simple_type);

    match target_type.to_lowercase().as_str() {
        "address" | "addr" => {
            for address in addresses {
                validate_address(address)?;
                request = request.address(address);
            }
        }
        "tag" => {
            let tag = tag.ok_or_else(|| anyhow::anyhow!("--target-type tag requires --tag"))?;
            request = request.tag(tag);
        }
        "network" | "net" | "project" | "proj" => {}
        other => anyhow::bail!(
            "Invalid target type '{}': expected address, network, project or tag",
            other
        ),
    }
    request = request.network(network);

    match simple_type {
        AlertType::SuccessfulTx => request = request.expression(AlertExpression::tx_status(true)),
        AlertType::FailedTx => request = request.expression(AlertExpression::tx_status(false)),
        _ => {}
    }

    for json in expressions {
        let expression: AlertExpression = serde_json::from_str(json)
            .with_context(|| format!("Invalid --expression JSON: {}", json))?;
        request = request.expression(expression);
    }
    for channel in channels {
        request = request.delivery_channel(channel);
    }

    Ok(request)
}

/// Build an `alerts update` body: the existing alert with the given CLI
/// overrides applied (`PATCH /alert/{id}` is a full replace)
fn merge_alert_update(
    existing: &tndrly::alerts::Alert,
    name: Option<&str>,
    alert_type: &str,
    network: Option<&str>,
    addresses: &[String],
    expressions: &[String],
    channels: &[String],
) -> anyhow::Result<tndrly::alerts::CreateAlertRequest> {
    use tndrly::alerts::{
        AlertDeliveryChannelInput, AlertExpression, AlertType, CreateAlertRequest, ExpressionType,
    };

    let simple_type: AlertType = alert_type
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid alert type '{}': {}", alert_type, e))?;
    let new_expressions = expressions
        .iter()
        .map(|json| {
            serde_json::from_str::<AlertExpression>(json)
                .with_context(|| format!("Invalid --expression JSON: {}", json))
        })
        .collect::<anyhow::Result<Vec<_>>>()?;
    let status_expression = match simple_type {
        AlertType::SuccessfulTx => Some(AlertExpression::tx_status(true)),
        AlertType::FailedTx => Some(AlertExpression::tx_status(false)),
        _ => None,
    };

    let mut request = CreateAlertRequest::from_alert(existing, simple_type);
    if let Some(name) = name {
        request.name = name.to_string();
    }

    // Keep each existing expression unless the matching CLI option replaces it
    request.expressions.retain(|e| match e.expression_type {
        ExpressionType::ContractAddress | ExpressionType::Tag => addresses.is_empty(),
        ExpressionType::Network => network.is_none(),
        ExpressionType::TxStatus => status_expression.is_none() && new_expressions.is_empty(),
        _ => new_expressions.is_empty(),
    });
    for address in addresses {
        validate_address(address)?;
        request = request.address(address);
    }
    if let Some(network) = network {
        request = request.network(network);
    }
    if let Some(status) = status_expression {
        request = request.expression(status);
    }
    request.expressions.extend(new_expressions);

    if !channels.is_empty() {
        request.delivery_channels = channels
            .iter()
            .map(AlertDeliveryChannelInput::new)
            .collect();
    }

    Ok(request)
}

async fn handle_webhooks(
    cmd: &WebhookCommands,
    client: &tndrly::Client,
    quiet: bool,
) -> anyhow::Result<()> {
    match cmd {
        WebhookCommands::Create { name, url } => {
            if !quiet {
                eprintln!("Creating webhook '{}'...", name);
            }
            let request = tndrly::alerts::CreateWebhookRequest::new(name, url);
            let webhook = client.alerts().create_webhook(&request).await?;
            println!("{}", serde_json::to_string_pretty(&webhook)?);
        }

        WebhookCommands::List => {
            if !quiet {
                eprintln!("Listing webhooks...");
            }
            let webhooks = client.alerts().list_webhooks().await?;
            println!("{}", serde_json::to_string_pretty(&webhooks)?);
        }

        WebhookCommands::Get { id } => {
            if !quiet {
                eprintln!("Getting webhook {}...", id);
            }
            let webhook = client.alerts().get_webhook(id).await?;
            println!("{}", serde_json::to_string_pretty(&webhook)?);
        }

        WebhookCommands::Delete { id } => {
            if !quiet {
                eprintln!("Deleting webhook {}...", id);
            }
            client.alerts().delete_webhook(id).await?;
            println!("Webhook {} deleted.", id);
        }
    }

    Ok(())
}

// ============================================================================
// Actions Handler
// ============================================================================

async fn handle_actions(
    cmd: &ActionsCommands,
    tenderly: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = tenderly.create_client()?;

    match cmd {
        ActionsCommands::Create {
            name,
            trigger,
            source,
            trigger_config,
            function,
        } => {
            if !quiet {
                eprintln!("Creating Web3 Action '{}'...", name);
            }

            let trigger_type: tndrly::actions::ActionTrigger = trigger
                .parse()
                .map_err(|e| anyhow::anyhow!("Invalid trigger type '{}': {}", trigger, e))?;

            let source_code = std::fs::read_to_string(source)
                .with_context(|| format!("Failed to read action source file: {}", source))?;
            let mut request =
                tndrly::actions::CreateActionRequest::new(name, trigger_type, &source_code)
                    .function(function);
            if let Some(json) = trigger_config {
                let value: serde_json::Value =
                    serde_json::from_str(json).context("Invalid --trigger-config JSON")?;
                request = request.trigger_config(tndrly::actions::TriggerConfig::from_value(value));
            }
            let action = client.actions().create(&request).await?;
            println!("{}", serde_json::to_string_pretty(&action)?);
        }

        ActionsCommands::List => {
            if !quiet {
                eprintln!("Listing Web3 Actions...");
            }
            let actions = client.actions().list().await?;
            println!("{}", serde_json::to_string_pretty(&actions)?);
        }

        ActionsCommands::Get { id } => {
            if !quiet {
                eprintln!("Getting Action {}...", id);
            }
            let action = client.actions().get(id).await?;
            println!("{}", serde_json::to_string_pretty(&action)?);
        }

        ActionsCommands::Delete { id } => {
            if !quiet {
                eprintln!("Deleting Action {}...", id);
            }
            client.actions().delete(id).await?;
            println!("Action {} deleted.", id);
        }

        ActionsCommands::Stop { id } => {
            if !quiet {
                eprintln!("Stopping Action {}...", id);
            }
            client.actions().stop(id).await?;
            println!("Action {} stopped.", id);
        }

        ActionsCommands::Resume { id } => {
            if !quiet {
                eprintln!("Resuming Action {}...", id);
            }
            client.actions().resume(id).await?;
            println!("Action {} resumed.", id);
        }

        ActionsCommands::StopMany { ids } => {
            if !quiet {
                if ids.is_empty() {
                    eprintln!("Stopping all Actions...");
                } else {
                    eprintln!("Stopping {} Actions...", ids.len());
                }
            }
            client.actions().stop_many(ids.clone()).await?;
            println!("Actions stopped.");
        }

        ActionsCommands::ResumeMany { ids } => {
            if !quiet {
                if ids.is_empty() {
                    eprintln!("Resuming all Actions...");
                } else {
                    eprintln!("Resuming {} Actions...", ids.len());
                }
            }
            client.actions().resume_many(ids.clone()).await?;
            println!("Actions resumed.");
        }

        ActionsCommands::GetCall { id, execution_id } => {
            if !quiet {
                eprintln!("Getting call {} for Action {}...", execution_id, id);
            }
            let call = client.actions().get_call(id, execution_id).await?;
            println!("{}", serde_json::to_string_pretty(&call)?);
        }
    }

    Ok(())
}

// ============================================================================
// Networks Handler
// ============================================================================

async fn handle_networks(
    cmd: &NetworksCommands,
    tenderly: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = tenderly.create_client()?;

    match cmd {
        NetworksCommands::List => {
            if !quiet {
                eprintln!("Listing supported networks...");
            }
            let networks = client.networks().supported().await?;
            println!("{}", serde_json::to_string_pretty(&networks)?);
        }

        NetworksCommands::Get { id } => {
            if !quiet {
                eprintln!("Getting network {}...", id);
            }
            let network = client.networks().get_by_chain_id(*id).await?;
            println!("{}", serde_json::to_string_pretty(&network)?);
        }

        NetworksCommands::Mainnets => {
            if !quiet {
                eprintln!("Listing mainnet networks...");
            }
            let networks = client.networks().mainnets().await?;
            println!("{}", serde_json::to_string_pretty(&networks)?);
        }

        NetworksCommands::Testnets => {
            if !quiet {
                eprintln!("Listing testnet networks...");
            }
            let networks = client.networks().testnets().await?;
            println!("{}", serde_json::to_string_pretty(&networks)?);
        }

        NetworksCommands::Simulation => {
            if !quiet {
                eprintln!("Listing networks with simulation support...");
            }
            let networks = client.networks().with_simulation_support().await?;
            println!("{}", serde_json::to_string_pretty(&networks)?);
        }

        NetworksCommands::Vnet => {
            if !quiet {
                eprintln!("Listing networks with VNet support...");
            }
            let networks = client.networks().with_vnet_support().await?;
            println!("{}", serde_json::to_string_pretty(&networks)?);
        }
    }

    Ok(())
}

// ============================================================================
// Channels Handler
// ============================================================================

async fn handle_channels(
    cmd: &ChannelsCommands,
    tenderly: &TenderlyArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = tenderly.create_client()?;

    match cmd {
        ChannelsCommands::List => {
            if !quiet {
                eprintln!("Listing all delivery channels...");
            }
            let channels = client.delivery_channels().list_all().await?;
            println!("{}", serde_json::to_string_pretty(&channels)?);
        }

        ChannelsCommands::Account => {
            if !quiet {
                eprintln!("Listing account-level delivery channels...");
            }
            let response = client.delivery_channels().list_account().await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response.delivery_channels)?
            );
        }

        ChannelsCommands::Project => {
            if !quiet {
                eprintln!("Listing project-level delivery channels...");
            }
            let response = client.delivery_channels().list_project().await?;
            println!(
                "{}",
                serde_json::to_string_pretty(&response.delivery_channels)?
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn existing_alert() -> tndrly::alerts::Alert {
        serde_json::from_value(serde_json::json!({
            "id": "00000000-0000-0000-0000-000000000010",
            "name": "Old name",
            "enabled": false,
            "delivery_channels": [
                { "delivery_channel_id": "00000000-0000-0000-0000-000000000001", "enabled": true }
            ],
            "expressions": [
                { "type": "tag", "expression": { "tag": "core" } },
                { "type": "network", "expression": { "network_id": "10" } },
                { "type": "tx_status", "expression": { "transaction_success": false } }
            ]
        }))
        .unwrap()
    }

    #[test]
    fn test_merge_alert_update_keeps_target_state_and_channels() {
        let request = merge_alert_update(
            &existing_alert(),
            Some("New name"),
            "failed_tx",
            None,
            &[],
            &[],
            &[],
        )
        .unwrap();
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "New name");
        assert_eq!(
            json["enabled"], false,
            "must not re-enable a disabled alert"
        );
        let types: Vec<_> = json["expressions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|e| e["type"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(types, ["tag", "network", "tx_status"]);
        assert_eq!(json["expressions"][1]["expression"]["network_id"], "10");
        assert_eq!(
            json["delivery_channels"][0]["id"],
            "00000000-0000-0000-0000-000000000001"
        );
    }

    #[test]
    fn test_merge_alert_update_overrides() {
        let request = merge_alert_update(
            &existing_alert(),
            None,
            "successful_tx",
            Some("1"),
            &["0x0000000000000000000000000000000000000002".to_string()],
            &[],
            &["00000000-0000-0000-0000-000000000003".to_string()],
        )
        .unwrap();
        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["name"], "Old name");
        assert_eq!(json["expressions"].as_array().unwrap().len(), 3);
        assert_eq!(json["expressions"][0]["type"], "contract_address");
        assert_eq!(json["expressions"][1]["expression"]["network_id"], "1");
        assert_eq!(
            json["expressions"][2]["expression"]["transaction_success"],
            true
        );
        assert_eq!(
            json["delivery_channels"],
            serde_json::json!([{ "id": "00000000-0000-0000-0000-000000000003", "enabled": true }])
        );
    }
}
