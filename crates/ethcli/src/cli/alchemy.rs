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
    #[arg(long, short = 'o', visible_alias = "output", default_value = "json")]
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

    /// Debug/trace operations (debug_* RPC methods)
    Debug {
        #[command(subcommand)]
        action: DebugCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Parity-style trace operations (trace_* RPC methods)
    Trace {
        #[command(subcommand)]
        action: TraceCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Transaction simulation
    Simulation {
        #[command(subcommand)]
        action: SimulationCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// ERC-4337 Bundler operations
    Bundler {
        #[command(subcommand)]
        action: BundlerCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Gas Manager policy management (read-only)
    GasManager {
        #[command(subcommand)]
        action: GasManagerCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Webhook/Notify operations (read-only)
    Notify {
        #[command(subcommand)]
        action: NotifyCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Ethereum Beacon Chain (consensus layer) operations
    Beacon {
        #[command(subcommand)]
        action: BeaconCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },

    /// Solana DAS (Digital Asset Standard) operations
    Solana {
        #[command(subcommand)]
        action: SolanaCommands,

        #[command(flatten)]
        args: AlchemyArgs,
    },
}

// =============================================================================
// NFT Commands
// =============================================================================

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

    /// Get all owners of an NFT contract
    OwnersForContract {
        /// Contract address
        contract: String,
    },

    /// Get NFT contracts owned by an address
    ContractsForOwner {
        /// Owner address
        address: String,
    },

    /// Get all NFTs for a contract
    NftsForContract {
        /// Contract address
        contract: String,
        /// Starting token for pagination
        #[arg(long)]
        start_token: Option<String>,
        /// Maximum number of NFTs to return
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Get contract metadata
    ContractMetadata {
        /// Contract address
        contract: String,
    },

    /// Get collection metadata by OpenSea slug
    CollectionMetadata {
        /// OpenSea collection slug
        slug: String,
    },

    /// Search contract metadata by keyword
    SearchContractMetadata {
        /// Search query
        query: String,
    },

    /// Compute rarity for an NFT
    ComputeRarity {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Summarize NFT attributes for a contract
    SummarizeAttributes {
        /// Contract address
        contract: String,
    },

    /// Refresh metadata for an NFT
    RefreshMetadata {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get NFT sales for a contract
    Sales {
        /// Contract address
        contract: String,
        /// Optional token ID
        #[arg(long)]
        token_id: Option<String>,
        /// Optional from block number
        #[arg(long)]
        from_block: Option<u64>,
        /// Optional to block number
        #[arg(long)]
        to_block: Option<u64>,
    },

    /// Get list of spam contracts
    SpamContracts,

    /// Check if a contract is spam
    IsSpam {
        /// Contract address
        contract: String,
    },

    /// Check if an NFT is an airdrop
    IsAirdrop {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Report a contract as spam
    ReportSpam {
        /// Contract address
        contract: String,
    },

    /// Get all NFTs for a collection by slug
    NftsForCollection {
        /// Collection slug
        slug: String,
        /// Starting token for pagination
        #[arg(long)]
        start_token: Option<String>,
        /// Maximum number of NFTs to return
        #[arg(long)]
        limit: Option<u32>,
    },

    /// Get collections owned by an address
    CollectionsForOwner {
        /// Owner address
        address: String,
    },

    /// Invalidate cached metadata for a contract
    InvalidateContract {
        /// Contract address
        contract: String,
    },
}

// =============================================================================
// Token Commands
// =============================================================================

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
        /// Token contract address
        contract: String,
        /// Owner address
        owner: String,
        /// Spender address
        spender: String,
    },

    /// Get token balances for specific token addresses
    BalancesForTokens {
        /// Wallet address to query
        address: String,
        /// Comma-separated token contract addresses
        tokens: String,
    },
}

// =============================================================================
// Transfer Commands
// =============================================================================

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

    /// Get all transfers for an address (both sent and received)
    All {
        /// Address to query
        address: String,
    },
}

// =============================================================================
// Portfolio Commands
// =============================================================================

#[derive(Subcommand)]
pub enum PortfolioCommands {
    /// Get token balances with prices
    Tokens {
        /// Address to query
        address: String,
    },

    /// Get token info for multiple tokens
    TokenInfo {
        /// Comma-separated token addresses (network:address format, e.g., eth-mainnet:0x...)
        tokens: String,
    },

    /// Get NFTs owned by an address
    Nfts {
        /// Address to query
        address: String,
        /// Include NFT metadata
        #[arg(long, default_value = "true")]
        with_metadata: bool,
    },

    /// Get NFT contracts owned by an address
    NftContracts {
        /// Address to query
        address: String,
    },
}

// =============================================================================
// Price Commands
// =============================================================================

/// Historical price interval
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum HistoricalIntervalCli {
    /// 5-minute intervals
    #[value(name = "5m")]
    FiveMinutes,
    /// 1-hour intervals
    #[value(name = "1h")]
    #[default]
    OneHour,
    /// 1-day intervals
    #[value(name = "1d")]
    OneDay,
}

impl From<HistoricalIntervalCli> for alcmy::prices::HistoricalInterval {
    fn from(v: HistoricalIntervalCli) -> Self {
        match v {
            HistoricalIntervalCli::FiveMinutes => alcmy::prices::HistoricalInterval::FiveMinutes,
            HistoricalIntervalCli::OneHour => alcmy::prices::HistoricalInterval::OneHour,
            HistoricalIntervalCli::OneDay => alcmy::prices::HistoricalInterval::OneDay,
        }
    }
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

    /// Get historical prices for a token by symbol
    HistoricalBySymbol {
        /// Token symbol (e.g., "ETH")
        symbol: String,
        /// Start time (ISO 8601 or Unix timestamp)
        start_time: String,
        /// End time (ISO 8601 or Unix timestamp)
        end_time: String,
        /// Time interval between data points
        #[arg(long, default_value = "1h")]
        interval: HistoricalIntervalCli,
    },

    /// Get historical prices for a token by address
    HistoricalByAddress {
        /// Token contract address
        address: String,
        /// Start time (ISO 8601 or Unix timestamp)
        start_time: String,
        /// End time (ISO 8601 or Unix timestamp)
        end_time: String,
        /// Time interval between data points
        #[arg(long, default_value = "1h")]
        interval: HistoricalIntervalCli,
    },
}

// =============================================================================
// Debug Commands
// =============================================================================

#[derive(Subcommand)]
pub enum DebugCommands {
    /// Trace a transaction
    TraceTx {
        /// Transaction hash
        hash: String,
    },

    /// Trace a call without executing it on-chain
    TraceCall {
        /// Recipient address
        to: String,
        /// Block number or tag (e.g., "latest")
        #[arg(long, default_value = "latest")]
        block: String,
        /// Sender address
        #[arg(long)]
        from: Option<String>,
        /// Call data (hex)
        #[arg(long)]
        data: Option<String>,
        /// Value to send (hex)
        #[arg(long)]
        value: Option<String>,
        /// Gas limit (hex)
        #[arg(long)]
        gas: Option<String>,
    },

    /// Trace all transactions in a block by hash
    TraceBlockByHash {
        /// Block hash
        hash: String,
    },

    /// Trace all transactions in a block by number
    TraceBlockByNumber {
        /// Block number (hex or decimal)
        block: String,
    },

    /// Get RLP-encoded block
    GetRawBlock {
        /// Block number or tag
        block: String,
    },

    /// Get RLP-encoded block header
    GetRawHeader {
        /// Block number or tag
        block: String,
    },

    /// Get EIP-2718 binary-encoded receipts
    GetRawReceipts {
        /// Block number or tag
        block: String,
    },
}

// =============================================================================
// Trace Commands (Parity-style)
// =============================================================================

#[derive(Subcommand)]
pub enum TraceCommands {
    /// Get traces for a block
    Block {
        /// Block number (hex or tag)
        block: String,
    },

    /// Execute a call and return traces
    Call {
        /// Recipient address
        to: String,
        /// Block number or tag
        #[arg(long, default_value = "latest")]
        block: String,
        /// Sender address
        #[arg(long)]
        from: Option<String>,
        /// Call data (hex)
        #[arg(long)]
        data: Option<String>,
        /// Value to send (hex)
        #[arg(long)]
        value: Option<String>,
        /// Gas limit (hex)
        #[arg(long)]
        gas: Option<String>,
        /// Trace types (comma-separated: trace,stateDiff,vmTrace)
        #[arg(long, default_value = "trace")]
        trace_types: String,
    },

    /// Get a specific trace by position in a transaction
    Get {
        /// Transaction hash
        hash: String,
        /// Trace indices (comma-separated, e.g., "0,1")
        indices: String,
    },

    /// Trace a raw transaction without executing
    RawTransaction {
        /// Raw signed transaction (hex)
        raw_tx: String,
        /// Trace types (comma-separated: trace,stateDiff,vmTrace)
        #[arg(long, default_value = "trace")]
        trace_types: String,
    },

    /// Replay all transactions in a block with traces
    ReplayBlockTransactions {
        /// Block number (hex or tag)
        block: String,
        /// Trace types (comma-separated: trace,stateDiff,vmTrace)
        #[arg(long, default_value = "trace")]
        trace_types: String,
    },

    /// Replay a transaction with traces
    ReplayTransaction {
        /// Transaction hash
        hash: String,
        /// Trace types (comma-separated: trace,stateDiff,vmTrace)
        #[arg(long, default_value = "trace")]
        trace_types: String,
    },

    /// Get all traces for a transaction
    Transaction {
        /// Transaction hash
        hash: String,
    },

    /// Filter traces by criteria
    Filter {
        /// From block (hex or tag)
        #[arg(long)]
        from_block: Option<String>,
        /// To block (hex or tag)
        #[arg(long)]
        to_block: Option<String>,
        /// Filter by sender addresses (comma-separated)
        #[arg(long)]
        from_address: Option<String>,
        /// Filter by recipient addresses (comma-separated)
        #[arg(long)]
        to_address: Option<String>,
        /// Skip first N traces
        #[arg(long)]
        after: Option<u32>,
        /// Maximum traces to return
        #[arg(long)]
        count: Option<u32>,
    },
}

// =============================================================================
// Simulation Commands
// =============================================================================

#[derive(Subcommand)]
pub enum SimulationCommands {
    /// Simulate a transaction and return asset changes
    AssetChanges {
        /// Recipient address
        to: String,
        /// Sender address
        #[arg(long)]
        from: Option<String>,
        /// Call data (hex)
        #[arg(long)]
        data: Option<String>,
        /// Value to send (hex)
        #[arg(long)]
        value: Option<String>,
        /// Gas limit (hex)
        #[arg(long)]
        gas: Option<String>,
    },

    /// Simulate execution with decoded traces and logs
    Execution {
        /// Recipient address
        to: String,
        /// Sender address
        #[arg(long)]
        from: Option<String>,
        /// Call data (hex)
        #[arg(long)]
        data: Option<String>,
        /// Value to send (hex)
        #[arg(long)]
        value: Option<String>,
        /// Gas limit (hex)
        #[arg(long)]
        gas: Option<String>,
        /// Block tag (e.g., "latest")
        #[arg(long, default_value = "latest")]
        block: String,
        /// Output format: nested or flat
        #[arg(long, default_value = "nested")]
        trace_format: String,
    },
}

// =============================================================================
// Bundler Commands (ERC-4337)
// =============================================================================

#[derive(Subcommand)]
pub enum BundlerCommands {
    /// Get supported entry points
    SupportedEntryPoints,

    /// Estimate gas for a UserOperation (pass JSON on stdin or as argument)
    EstimateGas {
        /// UserOperation as JSON string
        user_op_json: String,
        /// Entry point address
        #[arg(long, default_value = "0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789")]
        entry_point: String,
    },

    /// Get UserOperation by hash
    GetByHash {
        /// UserOperation hash
        hash: String,
    },

    /// Get UserOperation receipt
    GetReceipt {
        /// UserOperation hash
        hash: String,
    },

    /// Get recommended max priority fee per gas
    MaxPriorityFee,
}

// =============================================================================
// Gas Manager Commands (read-only)
// =============================================================================

#[derive(Subcommand)]
pub enum GasManagerCommands {
    /// List all gas manager policies
    ListPolicies,

    /// Get a policy by ID
    GetPolicy {
        /// Policy ID
        policy_id: String,
    },

    /// Get policy statistics
    PolicyStats {
        /// Policy ID
        policy_id: String,
    },

    /// List sponsorships for a policy
    ListSponsorships {
        /// Policy ID
        policy_id: String,
    },
}

// =============================================================================
// Notify Commands (read-only)
// =============================================================================

#[derive(Subcommand)]
pub enum NotifyCommands {
    /// List all webhooks
    ListWebhooks,

    /// List addresses tracked by a webhook
    ListAddresses {
        /// Webhook ID
        webhook_id: String,
    },

    /// List NFT filters for a webhook
    ListNftFilters {
        /// Webhook ID
        webhook_id: String,
    },
}

// =============================================================================
// Beacon Commands
// =============================================================================

#[derive(Subcommand)]
pub enum BeaconCommands {
    /// Get genesis info
    Genesis,

    /// Get fork schedule
    ForkSchedule,

    /// Get deposit contract info
    DepositContract,

    /// Get spec/config values
    Spec,

    /// Get block headers
    Headers,

    /// Get block header by ID
    Header {
        /// Block ID (slot number, "head", "finalized", "justified", or block root hash)
        block_id: String,
    },

    /// Get block by ID
    Block {
        /// Block ID (slot number, "head", "finalized", "justified", or block root hash)
        block_id: String,
    },

    /// Get block root
    BlockRoot {
        /// Block ID
        block_id: String,
    },

    /// Get block attestations
    BlockAttestations {
        /// Block ID
        block_id: String,
    },

    /// Get blob sidecars for a block
    BlobSidecars {
        /// Block ID
        block_id: String,
    },

    /// Get state root
    StateRoot {
        /// State ID (slot number, "head", "finalized", "justified", or state root hash)
        state_id: String,
    },

    /// Get fork info for state
    StateFork {
        /// State ID
        state_id: String,
    },

    /// Get finality checkpoints
    FinalityCheckpoints {
        /// State ID
        state_id: String,
    },

    /// Get validators
    Validators {
        /// State ID
        state_id: String,
    },

    /// Get a specific validator
    Validator {
        /// State ID
        state_id: String,
        /// Validator ID (index or pubkey)
        validator_id: String,
    },

    /// Get validator balances
    ValidatorBalances {
        /// State ID
        state_id: String,
    },

    /// Get sync committees
    SyncCommittees {
        /// State ID
        state_id: String,
    },

    /// Get RANDAO value
    Randao {
        /// State ID
        state_id: String,
    },

    /// Get pool attestations
    PoolAttestations,

    /// Get voluntary exits in the pool
    VoluntaryExits,

    /// Get block rewards
    BlockRewards {
        /// Block ID
        block_id: String,
    },

    /// Get node sync status
    Syncing,

    /// Get node version
    Version,

    /// Get peers
    Peers,

    /// Get peer count
    PeerCount,

    /// Get attester duties for an epoch
    AttesterDuties {
        /// Epoch number
        epoch: String,
        /// Comma-separated validator indices
        validators: String,
    },

    /// Get proposer duties for an epoch
    ProposerDuties {
        /// Epoch number
        epoch: String,
    },

    /// Get sync duties for an epoch
    SyncDuties {
        /// Epoch number
        epoch: String,
        /// Comma-separated validator indices
        validators: String,
    },
}

// =============================================================================
// Solana Commands
// =============================================================================

#[derive(Subcommand)]
pub enum SolanaCommands {
    /// Get a single asset by ID
    GetAsset {
        /// Asset ID (mint address)
        id: String,
    },

    /// Get multiple assets by IDs
    GetAssets {
        /// Comma-separated asset IDs
        ids: String,
    },

    /// Get proof for a compressed asset
    GetAssetProof {
        /// Asset ID
        id: String,
    },

    /// Get proofs for multiple compressed assets
    GetAssetProofs {
        /// Comma-separated asset IDs
        ids: String,
    },

    /// Get assets owned by an address
    GetAssetsByOwner {
        /// Owner address
        owner: String,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Results per page
        #[arg(long, default_value = "100")]
        limit: u32,
    },

    /// Get assets by creator address
    GetAssetsByCreator {
        /// Creator address
        creator: String,
    },

    /// Get assets by authority address
    GetAssetsByAuthority {
        /// Authority address
        authority: String,
    },

    /// Get assets by group (e.g., collection)
    GetAssetsByGroup {
        /// Group key (e.g., "collection")
        group_key: String,
        /// Group value (e.g., collection address)
        group_value: String,
    },

    /// Get token accounts
    GetTokenAccounts {
        /// Owner address
        #[arg(long)]
        owner: Option<String>,
        /// Mint address
        #[arg(long)]
        mint: Option<String>,
    },

    /// Get NFT editions
    GetNftEditions {
        /// Mint address
        mint: String,
    },

    /// Get asset signatures (transactions)
    GetAssetSignatures {
        /// Asset ID
        id: String,
    },
}

// =============================================================================
// Main Handler
// =============================================================================

/// Handle Alchemy commands
pub async fn handle(command: &AlchemyCommands, quiet: bool) -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    // Try config first, then fall back to env var
    let api_key = if let Ok(Some(config)) = ConfigFile::load_default() {
        if let Some(ref alchemy_config) = config.alchemy {
            alchemy_config.api_key.expose_secret().to_string()
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
        AlchemyCommands::Trace { action, args } => {
            handle_trace(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Simulation { action, args } => {
            handle_simulation(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Bundler { action, args } => {
            handle_bundler(action, args, &api_key, quiet).await
        }
        AlchemyCommands::GasManager { action, args } => {
            handle_gas_manager(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Notify { action, args } => {
            handle_notify(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Beacon { action, args } => {
            handle_beacon(action, args, &api_key, quiet).await
        }
        AlchemyCommands::Solana { action, args } => {
            handle_solana(action, args, &api_key, quiet).await
        }
    }
}

// =============================================================================
// NFT Handler
// =============================================================================

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
        NftCommands::OwnersForContract { contract } => {
            if !quiet {
                eprintln!("Fetching all owners for contract {}...", contract);
            }
            let response = client.nft().get_owners_for_contract(contract).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ContractsForOwner { address } => {
            if !quiet {
                eprintln!("Fetching NFT contracts for owner {}...", address);
            }
            let response = client.nft().get_contracts_for_owner(address).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::NftsForContract {
            contract,
            start_token,
            limit,
        } => {
            if !quiet {
                eprintln!("Fetching NFTs for contract {}...", contract);
            }
            let response = client
                .nft()
                .get_nfts_for_contract_with_options(contract, start_token.as_deref(), *limit)
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ContractMetadata { contract } => {
            if !quiet {
                eprintln!("Fetching contract metadata for {}...", contract);
            }
            let response = client.nft().get_contract_metadata(contract).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::CollectionMetadata { slug } => {
            if !quiet {
                eprintln!("Fetching collection metadata for slug {}...", slug);
            }
            let response = client.nft().get_collection_metadata(slug).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::SearchContractMetadata { query } => {
            if !quiet {
                eprintln!("Searching contract metadata for '{}'...", query);
            }
            let response = client.nft().search_contract_metadata(query).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ComputeRarity { contract, token_id } => {
            if !quiet {
                eprintln!("Computing rarity for {}:{}...", contract, token_id);
            }
            let response = client.nft().compute_rarity(contract, token_id).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::SummarizeAttributes { contract } => {
            if !quiet {
                eprintln!("Summarizing NFT attributes for {}...", contract);
            }
            let response = client.nft().summarize_nft_attributes(contract).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::RefreshMetadata { contract, token_id } => {
            if !quiet {
                eprintln!("Refreshing metadata for {}:{}...", contract, token_id);
            }
            let response = client
                .nft()
                .refresh_nft_metadata(contract, token_id)
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::Sales {
            contract,
            token_id,
            from_block,
            to_block,
        } => {
            if !quiet {
                eprintln!("Fetching NFT sales for {}...", contract);
            }
            let response = client
                .nft()
                .get_nft_sales_with_options(contract, token_id.as_deref(), *from_block, *to_block)
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::SpamContracts => {
            if !quiet {
                eprintln!("Fetching spam contracts...");
            }
            let response = client.nft().get_spam_contracts().await?;
            print_output(&response, args.format)?;
        }
        NftCommands::IsSpam { contract } => {
            if !quiet {
                eprintln!("Checking if {} is spam...", contract);
            }
            let response = client.nft().is_spam_contract(contract).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::IsAirdrop { contract, token_id } => {
            if !quiet {
                eprintln!("Checking if {}:{} is an airdrop...", contract, token_id);
            }
            let response = client.nft().is_airdrop_nft(contract, token_id).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ReportSpam { contract } => {
            if !quiet {
                eprintln!("Reporting {} as spam...", contract);
            }
            client.nft().report_spam(contract).await?;
            println!("Reported {} as spam", contract);
        }
        NftCommands::NftsForCollection {
            slug,
            start_token,
            limit,
        } => {
            if !quiet {
                eprintln!("Fetching NFTs for collection {}...", slug);
            }
            let response = client
                .nft()
                .get_nfts_for_collection_with_options(slug, start_token.as_deref(), *limit)
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::CollectionsForOwner { address } => {
            if !quiet {
                eprintln!("Fetching collections for owner {}...", address);
            }
            let response = client.nft().get_collections_for_owner(address).await?;
            print_output(&response, args.format)?;
        }
        NftCommands::InvalidateContract { contract } => {
            if !quiet {
                eprintln!("Invalidating contract cache for {}...", contract);
            }
            let response = client.nft().invalidate_contract(contract).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Token Handler
// =============================================================================

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
        TokenCommands::Allowances {
            contract,
            owner,
            spender,
        } => {
            if !quiet {
                eprintln!(
                    "Fetching allowances from {} to {} for contract {}...",
                    owner, spender, contract
                );
            }
            let response = client
                .token()
                .get_token_allowance(contract, owner, spender)
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::BalancesForTokens { address, tokens } => {
            if !quiet {
                eprintln!(
                    "Fetching token balances for {} (specific tokens)...",
                    address
                );
            }
            let token_list: Vec<&str> = tokens.split(',').map(|s| s.trim()).collect();
            let response = client
                .token()
                .get_token_balances_for_tokens(address, &token_list)
                .await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Transfers Handler
// =============================================================================

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
        TransferCommands::All { address } => {
            if !quiet {
                eprintln!("Fetching all transfers for {}...", address);
            }
            let (from, to) = client.transfers().get_all_transfers(address).await?;
            let combined = serde_json::json!({
                "from": from,
                "to": to
            });
            print_output(&combined, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Portfolio Handler
// =============================================================================

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
        PortfolioCommands::TokenInfo { tokens } => {
            if !quiet {
                eprintln!("Fetching token info...");
            }
            let token_pairs: Vec<(&str, &str)> = tokens
                .split(',')
                .filter_map(|s| {
                    let s = s.trim();
                    s.split_once(':')
                })
                .collect();
            if token_pairs.is_empty() {
                anyhow::bail!("Tokens must be in network:address format (e.g., eth-mainnet:0x...)");
            }
            let response = client.portfolio().get_token_info(&token_pairs).await?;
            print_output(&response, args.format)?;
        }
        PortfolioCommands::Nfts {
            address,
            with_metadata,
        } => {
            if !quiet {
                eprintln!("Fetching NFTs for {}...", address);
            }
            let networks: &[&str] = &[args.network.as_str()];
            let addresses: &[(&str, &[&str])] = &[(address.as_str(), networks)];
            let response = client
                .portfolio()
                .get_nfts_by_address(addresses, *with_metadata)
                .await?;
            print_output(&response, args.format)?;
        }
        PortfolioCommands::NftContracts { address } => {
            if !quiet {
                eprintln!("Fetching NFT contracts for {}...", address);
            }
            let networks: &[&str] = &[args.network.as_str()];
            let addresses: &[(&str, &[&str])] = &[(address.as_str(), networks)];
            let response = client
                .portfolio()
                .get_nft_contracts_by_address(addresses)
                .await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Prices Handler
// =============================================================================

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
        PriceCommands::HistoricalBySymbol {
            symbol,
            start_time,
            end_time,
            interval,
        } => {
            if !quiet {
                eprintln!("Fetching historical prices for {}...", symbol);
            }
            let response = client
                .prices()
                .get_historical_by_symbol(symbol, start_time, end_time, (*interval).into())
                .await?;
            print_output(&response, args.format)?;
        }
        PriceCommands::HistoricalByAddress {
            address,
            start_time,
            end_time,
            interval,
        } => {
            if !quiet {
                eprintln!("Fetching historical prices for {}...", address);
            }
            let network = args.network.as_str();
            let response = client
                .prices()
                .get_historical_by_address(
                    network,
                    address,
                    start_time,
                    end_time,
                    (*interval).into(),
                )
                .await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Debug Handler
// =============================================================================

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
        DebugCommands::TraceCall {
            to,
            block,
            from,
            data,
            value,
            gas,
        } => {
            if !quiet {
                eprintln!("Tracing call to {}...", to);
            }
            let call = alcmy::debug::TraceCallObject {
                to: to.clone(),
                from: from.clone(),
                data: data.clone(),
                value: value.clone(),
                gas: gas.clone(),
                ..Default::default()
            };
            let response = client.debug().trace_call(&call, block).await?;
            print_output(&response, args.format)?;
        }
        DebugCommands::TraceBlockByHash { hash } => {
            if !quiet {
                eprintln!("Tracing block {}...", hash);
            }
            let response = client.debug().trace_block_by_hash(hash).await?;
            print_output(&response, args.format)?;
        }
        DebugCommands::TraceBlockByNumber { block } => {
            if !quiet {
                eprintln!("Tracing block {}...", block);
            }
            let response = client.debug().trace_block_by_number(block).await?;
            print_output(&response, args.format)?;
        }
        DebugCommands::GetRawBlock { block } => {
            if !quiet {
                eprintln!("Fetching raw block {}...", block);
            }
            let response = client.debug().get_raw_block(block).await?;
            print_output(&response, args.format)?;
        }
        DebugCommands::GetRawHeader { block } => {
            if !quiet {
                eprintln!("Fetching raw header {}...", block);
            }
            let response = client.debug().get_raw_header(block).await?;
            print_output(&response, args.format)?;
        }
        DebugCommands::GetRawReceipts { block } => {
            if !quiet {
                eprintln!("Fetching raw receipts for block {}...", block);
            }
            let response = client.debug().get_raw_receipts(block).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Trace Handler (Parity-style)
// =============================================================================

/// Parse comma-separated trace types
fn parse_trace_types(s: &str) -> Vec<alcmy::trace::TraceType> {
    s.split(',')
        .filter_map(|t| match t.trim().to_lowercase().as_str() {
            "trace" => Some(alcmy::trace::TraceType::Trace),
            "statediff" | "state_diff" => Some(alcmy::trace::TraceType::StateDiff),
            "vmtrace" | "vm_trace" => Some(alcmy::trace::TraceType::VmTrace),
            _ => None,
        })
        .collect()
}

async fn handle_trace(
    action: &TraceCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        TraceCommands::Block { block } => {
            if !quiet {
                eprintln!("Fetching traces for block {}...", block);
            }
            let response = client.trace().block(block).await?;
            print_output(&response, args.format)?;
        }
        TraceCommands::Call {
            to,
            block,
            from,
            data,
            value,
            gas,
            trace_types,
        } => {
            if !quiet {
                eprintln!("Tracing call to {}...", to);
            }
            let request = alcmy::trace::TraceCallRequest {
                to: to.clone(),
                from: from.clone(),
                data: data.clone(),
                value: value.clone(),
                gas: gas.clone(),
                ..Default::default()
            };
            let types = parse_trace_types(trace_types);
            let response = client
                .trace()
                .call(&request, &types, Some(block.as_str()))
                .await?;
            print_output(&response, args.format)?;
        }
        TraceCommands::Get { hash, indices } => {
            if !quiet {
                eprintln!("Fetching trace at {}...", hash);
            }
            let idx: Vec<u32> = indices
                .split(',')
                .filter_map(|s| s.trim().parse().ok())
                .collect();
            let response = client.trace().get(hash, &idx).await?;
            print_output(&response, args.format)?;
        }
        TraceCommands::RawTransaction {
            raw_tx,
            trace_types,
        } => {
            if !quiet {
                eprintln!("Tracing raw transaction...");
            }
            let types = parse_trace_types(trace_types);
            let response = client.trace().raw_transaction(raw_tx, &types).await?;
            print_output(&response, args.format)?;
        }
        TraceCommands::ReplayBlockTransactions { block, trace_types } => {
            if !quiet {
                eprintln!("Replaying block {} transactions...", block);
            }
            let types = parse_trace_types(trace_types);
            let response = client
                .trace()
                .replay_block_transactions(block, &types)
                .await?;
            print_output(&response, args.format)?;
        }
        TraceCommands::ReplayTransaction { hash, trace_types } => {
            if !quiet {
                eprintln!("Replaying transaction {}...", hash);
            }
            let types = parse_trace_types(trace_types);
            let response = client.trace().replay_transaction(hash, &types).await?;
            print_output(&response, args.format)?;
        }
        TraceCommands::Transaction { hash } => {
            if !quiet {
                eprintln!("Fetching traces for transaction {}...", hash);
            }
            let response = client.trace().transaction(hash).await?;
            print_output(&response, args.format)?;
        }
        TraceCommands::Filter {
            from_block,
            to_block,
            from_address,
            to_address,
            after,
            count,
        } => {
            if !quiet {
                eprintln!("Filtering traces...");
            }
            let filter = alcmy::trace::TraceFilter {
                from_block: from_block.clone(),
                to_block: to_block.clone(),
                from_address: from_address
                    .as_ref()
                    .map(|s| s.split(',').map(|a| a.trim().to_string()).collect()),
                to_address: to_address
                    .as_ref()
                    .map(|s| s.split(',').map(|a| a.trim().to_string()).collect()),
                after: *after,
                count: *count,
            };
            let response = client.trace().filter(&filter).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Simulation Handler
// =============================================================================

async fn handle_simulation(
    action: &SimulationCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        SimulationCommands::AssetChanges {
            to,
            from,
            data,
            value,
            gas,
        } => {
            if !quiet {
                eprintln!("Simulating asset changes for call to {}...", to);
            }
            let tx = alcmy::simulation::SimulationTransaction {
                to: to.clone(),
                from: from.clone(),
                data: data.clone(),
                value: value.clone(),
                gas: gas.clone(),
                ..Default::default()
            };
            let response = client.simulation().simulate_asset_changes(&tx).await?;
            print_output(&response, args.format)?;
        }
        SimulationCommands::Execution {
            to,
            from,
            data,
            value,
            gas,
            block,
            trace_format,
        } => {
            if !quiet {
                eprintln!("Simulating execution for call to {}...", to);
            }
            let tx = alcmy::simulation::SimulationTransaction {
                to: to.clone(),
                from: from.clone(),
                data: data.clone(),
                value: value.clone(),
                gas: gas.clone(),
                ..Default::default()
            };
            let format = match trace_format.to_lowercase().as_str() {
                "flat" => alcmy::simulation::ExecutionFormat::Flat,
                _ => alcmy::simulation::ExecutionFormat::Nested,
            };
            let response = client
                .simulation()
                .simulate_execution(&tx, format, block)
                .await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Bundler Handler (ERC-4337)
// =============================================================================

async fn handle_bundler(
    action: &BundlerCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        BundlerCommands::SupportedEntryPoints => {
            if !quiet {
                eprintln!("Fetching supported entry points...");
            }
            let response = client.bundler().supported_entry_points().await?;
            print_output(&response, args.format)?;
        }
        BundlerCommands::EstimateGas {
            user_op_json,
            entry_point,
        } => {
            if !quiet {
                eprintln!("Estimating gas for UserOperation...");
            }
            let user_op: alcmy::bundler::UserOperation = serde_json::from_str(user_op_json)?;
            let response = client
                .bundler()
                .estimate_user_operation_gas(&user_op, entry_point)
                .await?;
            print_output(&response, args.format)?;
        }
        BundlerCommands::GetByHash { hash } => {
            if !quiet {
                eprintln!("Fetching UserOperation {}...", hash);
            }
            let response = client.bundler().get_user_operation_by_hash(hash).await?;
            print_output(&response, args.format)?;
        }
        BundlerCommands::GetReceipt { hash } => {
            if !quiet {
                eprintln!("Fetching UserOperation receipt {}...", hash);
            }
            let response = client.bundler().get_user_operation_receipt(hash).await?;
            print_output(&response, args.format)?;
        }
        BundlerCommands::MaxPriorityFee => {
            if !quiet {
                eprintln!("Fetching max priority fee per gas...");
            }
            let response = client.bundler().max_priority_fee_per_gas().await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Gas Manager Handler (read-only)
// =============================================================================

async fn handle_gas_manager(
    action: &GasManagerCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        GasManagerCommands::ListPolicies => {
            if !quiet {
                eprintln!("Listing gas manager policies...");
            }
            let response = client.gas_manager().list_policies().await?;
            print_output(&response, args.format)?;
        }
        GasManagerCommands::GetPolicy { policy_id } => {
            if !quiet {
                eprintln!("Fetching policy {}...", policy_id);
            }
            let response = client.gas_manager().get_policy(policy_id).await?;
            print_output(&response, args.format)?;
        }
        GasManagerCommands::PolicyStats { policy_id } => {
            if !quiet {
                eprintln!("Fetching policy stats for {}...", policy_id);
            }
            let response = client.gas_manager().get_policy_stats(policy_id).await?;
            print_output(&response, args.format)?;
        }
        GasManagerCommands::ListSponsorships { policy_id } => {
            if !quiet {
                eprintln!("Listing sponsorships for policy {}...", policy_id);
            }
            let response = client.gas_manager().list_sponsorships(policy_id).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Notify Handler (read-only)
// =============================================================================

async fn handle_notify(
    action: &NotifyCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        NotifyCommands::ListWebhooks => {
            if !quiet {
                eprintln!("Listing webhooks...");
            }
            let response = client.notify().list_webhooks().await?;
            print_output(&response, args.format)?;
        }
        NotifyCommands::ListAddresses { webhook_id } => {
            if !quiet {
                eprintln!("Listing addresses for webhook {}...", webhook_id);
            }
            let response = client.notify().list_webhook_addresses(webhook_id).await?;
            print_output(&response, args.format)?;
        }
        NotifyCommands::ListNftFilters { webhook_id } => {
            if !quiet {
                eprintln!("Listing NFT filters for webhook {}...", webhook_id);
            }
            let response = client.notify().list_nft_filters(webhook_id).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Beacon Handler
// =============================================================================

async fn handle_beacon(
    action: &BeaconCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        BeaconCommands::Genesis => {
            if !quiet {
                eprintln!("Fetching genesis info...");
            }
            let response = client.beacon().get_genesis().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::ForkSchedule => {
            if !quiet {
                eprintln!("Fetching fork schedule...");
            }
            let response = client.beacon().get_fork_schedule().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::DepositContract => {
            if !quiet {
                eprintln!("Fetching deposit contract info...");
            }
            let response = client.beacon().get_deposit_contract().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Spec => {
            if !quiet {
                eprintln!("Fetching spec/config...");
            }
            let response = client.beacon().get_spec().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Headers => {
            if !quiet {
                eprintln!("Fetching block headers...");
            }
            let response = client.beacon().get_headers().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Header { block_id } => {
            if !quiet {
                eprintln!("Fetching header for {}...", block_id);
            }
            let response = client.beacon().get_header(block_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Block { block_id } => {
            if !quiet {
                eprintln!("Fetching block {}...", block_id);
            }
            let response = client.beacon().get_block(block_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::BlockRoot { block_id } => {
            if !quiet {
                eprintln!("Fetching block root for {}...", block_id);
            }
            let response = client.beacon().get_block_root(block_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::BlockAttestations { block_id } => {
            if !quiet {
                eprintln!("Fetching attestations for block {}...", block_id);
            }
            let response = client.beacon().get_block_attestations(block_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::BlobSidecars { block_id } => {
            if !quiet {
                eprintln!("Fetching blob sidecars for block {}...", block_id);
            }
            let response = client.beacon().get_blob_sidecars(block_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::StateRoot { state_id } => {
            if !quiet {
                eprintln!("Fetching state root for {}...", state_id);
            }
            let response = client.beacon().get_state_root(state_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::StateFork { state_id } => {
            if !quiet {
                eprintln!("Fetching fork info for state {}...", state_id);
            }
            let response = client.beacon().get_state_fork(state_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::FinalityCheckpoints { state_id } => {
            if !quiet {
                eprintln!("Fetching finality checkpoints for {}...", state_id);
            }
            let response = client.beacon().get_finality_checkpoints(state_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Validators { state_id } => {
            if !quiet {
                eprintln!("Fetching validators for state {}...", state_id);
            }
            let response = client.beacon().get_validators(state_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Validator {
            state_id,
            validator_id,
        } => {
            if !quiet {
                eprintln!(
                    "Fetching validator {} at state {}...",
                    validator_id, state_id
                );
            }
            let response = client
                .beacon()
                .get_validator(state_id, validator_id)
                .await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::ValidatorBalances { state_id } => {
            if !quiet {
                eprintln!("Fetching validator balances for state {}...", state_id);
            }
            let response = client.beacon().get_validator_balances(state_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::SyncCommittees { state_id } => {
            if !quiet {
                eprintln!("Fetching sync committees for state {}...", state_id);
            }
            let response = client.beacon().get_sync_committees(state_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Randao { state_id } => {
            if !quiet {
                eprintln!("Fetching RANDAO for state {}...", state_id);
            }
            let response = client.beacon().get_randao(state_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::PoolAttestations => {
            if !quiet {
                eprintln!("Fetching pool attestations...");
            }
            let response = client.beacon().get_pool_attestations().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::VoluntaryExits => {
            if !quiet {
                eprintln!("Fetching voluntary exits...");
            }
            let response = client.beacon().get_voluntary_exits().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::BlockRewards { block_id } => {
            if !quiet {
                eprintln!("Fetching block rewards for {}...", block_id);
            }
            let response = client.beacon().get_block_rewards(block_id).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Syncing => {
            if !quiet {
                eprintln!("Fetching sync status...");
            }
            let response = client.beacon().get_syncing().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Version => {
            if !quiet {
                eprintln!("Fetching node version...");
            }
            let response = client.beacon().get_version().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::Peers => {
            if !quiet {
                eprintln!("Fetching peers...");
            }
            let response = client.beacon().get_peers().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::PeerCount => {
            if !quiet {
                eprintln!("Fetching peer count...");
            }
            let response = client.beacon().get_peer_count().await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::AttesterDuties { epoch, validators } => {
            if !quiet {
                eprintln!("Fetching attester duties for epoch {}...", epoch);
            }
            let indices: Vec<&str> = validators.split(',').map(|s| s.trim()).collect();
            let response = client.beacon().get_attester_duties(epoch, &indices).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::ProposerDuties { epoch } => {
            if !quiet {
                eprintln!("Fetching proposer duties for epoch {}...", epoch);
            }
            let response = client.beacon().get_proposer_duties(epoch).await?;
            print_output(&response, args.format)?;
        }
        BeaconCommands::SyncDuties { epoch, validators } => {
            if !quiet {
                eprintln!("Fetching sync duties for epoch {}...", epoch);
            }
            let indices: Vec<&str> = validators.split(',').map(|s| s.trim()).collect();
            let response = client.beacon().get_sync_duties(epoch, &indices).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Solana Handler
// =============================================================================

async fn handle_solana(
    action: &SolanaCommands,
    args: &AlchemyArgs,
    api_key: &str,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = alcmy::Client::new(api_key, args.network.into())?;

    match action {
        SolanaCommands::GetAsset { id } => {
            if !quiet {
                eprintln!("Fetching asset {}...", id);
            }
            let response = client.solana().get_asset(id).await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssets { ids } => {
            if !quiet {
                eprintln!("Fetching assets...");
            }
            let id_list: Vec<&str> = ids.split(',').map(|s| s.trim()).collect();
            let response = client.solana().get_assets(&id_list).await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssetProof { id } => {
            if !quiet {
                eprintln!("Fetching asset proof for {}...", id);
            }
            let response = client.solana().get_asset_proof(id).await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssetProofs { ids } => {
            if !quiet {
                eprintln!("Fetching asset proofs...");
            }
            let id_list: Vec<&str> = ids.split(',').map(|s| s.trim()).collect();
            let response = client.solana().get_asset_proofs(&id_list).await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssetsByOwner { owner, page, limit } => {
            if !quiet {
                eprintln!("Fetching assets for owner {}...", owner);
            }
            let pagination = alcmy::solana::PaginationOptions {
                page: Some(*page),
                limit: Some(*limit),
                cursor: None,
                sort_by: None,
            };
            let display = alcmy::solana::DisplayOptions::default();
            let response = client
                .solana()
                .get_assets_by_owner_with_options(owner, &pagination, &display)
                .await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssetsByCreator { creator } => {
            if !quiet {
                eprintln!("Fetching assets by creator {}...", creator);
            }
            let response = client.solana().get_assets_by_creator(creator).await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssetsByAuthority { authority } => {
            if !quiet {
                eprintln!("Fetching assets by authority {}...", authority);
            }
            let response = client.solana().get_assets_by_authority(authority).await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssetsByGroup {
            group_key,
            group_value,
        } => {
            if !quiet {
                eprintln!("Fetching assets by group {}:{}...", group_key, group_value);
            }
            let response = client
                .solana()
                .get_assets_by_group(group_key, group_value)
                .await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetTokenAccounts { owner, mint } => {
            if !quiet {
                eprintln!("Fetching token accounts...");
            }
            let response = client
                .solana()
                .get_token_accounts(owner.as_deref(), mint.as_deref())
                .await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetNftEditions { mint } => {
            if !quiet {
                eprintln!("Fetching NFT editions for {}...", mint);
            }
            let response = client.solana().get_nft_editions(mint).await?;
            print_output(&response, args.format)?;
        }
        SolanaCommands::GetAssetSignatures { id } => {
            if !quiet {
                eprintln!("Fetching asset signatures for {}...", id);
            }
            let response = client.solana().get_asset_signatures(id).await?;
            print_output(&response, args.format)?;
        }
    }

    Ok(())
}

// =============================================================================
// Output Helper
// =============================================================================

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
