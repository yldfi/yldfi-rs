//! Direct Moralis API commands
//!
//! Provides 1:1 access to Moralis Web3 API endpoints.

use crate::cli::OutputFormat;
use crate::config::ConfigFile;
use clap::{Args, Subcommand};

const MORALIS_FANTOM_REMOVAL_DATE: &str = "2026-05-29";
const MORALIS_LEGACY_REMOVAL_DATE: &str = "2026-06-04";
const MORALIS_HOLDERS_HISTORICAL_REMOVAL_DATE: &str = "2026-07-31";

fn ensure_moralis_chain_supported(chain: &str) -> anyhow::Result<()> {
    match chain.trim().to_ascii_lowercase().as_str() {
        "fantom" | "ftm" | "0xfa" | "250" => anyhow::bail!(
            "Moralis is removing Fantom support on {MORALIS_FANTOM_REMOVAL_DATE}; \
             ethcli no longer routes Moralis requests to Fantom. Use another provider \
             or migrate Fantom-specific workflows before relying on this command."
        ),
        _ => Ok(()),
    }
}

fn deprecated_moralis_endpoint(
    command: &str,
    endpoints: &[&str],
    replacement: Option<&str>,
) -> anyhow::Result<()> {
    deprecated_moralis_endpoint_with_date(
        command,
        endpoints,
        MORALIS_LEGACY_REMOVAL_DATE,
        replacement,
    )
}

fn deprecated_moralis_endpoint_with_date(
    command: &str,
    endpoints: &[&str],
    removal_date: &str,
    replacement: Option<&str>,
) -> anyhow::Result<()> {
    let endpoint_list = endpoints.join(", ");
    let mut message = format!(
        "`ethcli moralis {command}` calls Moralis endpoint(s) scheduled for removal on \
         {removal_date}: {endpoint_list}."
    );

    if let Some(replacement) = replacement {
        message.push_str(" Suggested migration: ");
        message.push_str(replacement);
        message.push('.');
    }

    anyhow::bail!("{message}")
}

fn moralis_command_args(command: &MoralisCommands) -> &MoralisArgs {
    match command {
        MoralisCommands::Wallet { args, .. }
        | MoralisCommands::Token { args, .. }
        | MoralisCommands::Nft { args, .. }
        | MoralisCommands::Resolve { args, .. }
        | MoralisCommands::Market { args, .. }
        | MoralisCommands::Transaction { args, .. }
        | MoralisCommands::Block { args, .. }
        | MoralisCommands::Defi { args, .. }
        | MoralisCommands::Discovery { args, .. }
        | MoralisCommands::Analytics { args, .. }
        | MoralisCommands::Entities { args, .. }
        | MoralisCommands::Volume { args, .. } => args,
    }
}

fn reject_deprecated_moralis_command(command: &MoralisCommands) -> anyhow::Result<()> {
    match command {
        MoralisCommands::Token { action, .. } => match action {
            TokenCommands::Stats { .. } => deprecated_moralis_endpoint(
                "token stats",
                &["GET /erc20/{address}/stats"],
                Some("use Moralis Token API analytics where available"),
            ),
            TokenCommands::ExchangeNewTokens { .. } => deprecated_moralis_endpoint(
                "token exchange-new-tokens",
                &["GET /erc20/exchange/{exchangeName}/new"],
                None,
            ),
            TokenCommands::ExchangeBondingTokens { .. } => deprecated_moralis_endpoint(
                "token exchange-bonding-tokens",
                &["GET /erc20/exchange/{exchangeName}/bonding"],
                None,
            ),
            TokenCommands::ExchangeGraduatedTokens { .. } => deprecated_moralis_endpoint(
                "token exchange-graduated-tokens",
                &["GET /erc20/exchange/{exchangeName}/graduated"],
                None,
            ),
            TokenCommands::BySymbols { .. } => deprecated_moralis_endpoint(
                "token by-symbols",
                &["GET /erc20/metadata/symbols"],
                Some("use `ethcli moralis token search <symbol>` for one symbol at a time"),
            ),
            TokenCommands::HoldersHistorical { .. } => deprecated_moralis_endpoint_with_date(
                "token holders-historical",
                &["GET /erc20/{address}/holders/historical"],
                MORALIS_HOLDERS_HISTORICAL_REMOVAL_DATE,
                Some(
                    "use `ethcli moralis token holders` or `ethcli moralis token \
                     holders-summary` for current holder data (Moralis documents no \
                     historical replacement)",
                ),
            ),
            TokenCommands::PairsStats { .. } => deprecated_moralis_endpoint(
                "token pairs-stats",
                &["GET /erc20/{token_address}/pairs/stats"],
                Some("migrate to `GET /tokens/{tokenAddress}/analytics`"),
            ),
            TokenCommands::PairSnipers { .. } => deprecated_moralis_endpoint(
                "token pair-snipers",
                &["GET /pairs/{address}/snipers"],
                None,
            ),
            TokenCommands::BondingStatus { .. } => deprecated_moralis_endpoint(
                "token bonding-status",
                &["GET /erc20/{tokenAddress}/bondingStatus"],
                None,
            ),
            _ => Ok(()),
        },
        MoralisCommands::Market { action, .. } => match action {
            MarketCommands::TopTokens => deprecated_moralis_endpoint(
                "market top-tokens",
                &["GET /market-data/erc20s/top-tokens"],
                None,
            ),
            MarketCommands::TopMovers => deprecated_moralis_endpoint(
                "market top-movers",
                &["GET /market-data/erc20s/top-movers"],
                None,
            ),
            MarketCommands::TopNfts => deprecated_moralis_endpoint(
                "market top-nfts",
                &["GET /market-data/nfts/top-collections"],
                None,
            ),
            MarketCommands::HottestNfts => deprecated_moralis_endpoint(
                "market hottest-nfts",
                &["GET /market-data/nfts/hottest-collections"],
                None,
            ),
            MarketCommands::GlobalMarketCap => deprecated_moralis_endpoint(
                "market global-market-cap",
                &["GET /market-data/global/market-cap"],
                None,
            ),
            MarketCommands::GlobalVolume => deprecated_moralis_endpoint(
                "market global-volume",
                &["GET /market-data/global/volume"],
                None,
            ),
        },
        MoralisCommands::Discovery { action, .. } => match action {
            DiscoveryCommands::RisingLiquidity => deprecated_moralis_endpoint(
                "discovery rising-liquidity",
                &["GET /discovery/tokens/rising-liquidity"],
                None,
            ),
            DiscoveryCommands::BuyingPressure => deprecated_moralis_endpoint(
                "discovery buying-pressure",
                &["GET /discovery/tokens/buying-pressure"],
                None,
            ),
            DiscoveryCommands::SolidPerformers => deprecated_moralis_endpoint(
                "discovery solid-performers",
                &["GET /discovery/tokens/solid-performers"],
                None,
            ),
            DiscoveryCommands::ExperiencedBuyers => deprecated_moralis_endpoint(
                "discovery experienced-buyers",
                &["GET /discovery/tokens/experienced-buyers"],
                None,
            ),
            DiscoveryCommands::RiskyBets => deprecated_moralis_endpoint(
                "discovery risky-bets",
                &["GET /discovery/tokens/risky-bets"],
                None,
            ),
            DiscoveryCommands::BlueChip => deprecated_moralis_endpoint(
                "discovery blue-chip",
                &["GET /discovery/tokens/blue-chip"],
                None,
            ),
            DiscoveryCommands::TopGainers => deprecated_moralis_endpoint(
                "discovery top-gainers",
                &["GET /discovery/tokens/top-gainers"],
                None,
            ),
            DiscoveryCommands::TopLosers => deprecated_moralis_endpoint(
                "discovery top-losers",
                &["GET /discovery/tokens/top-losers"],
                None,
            ),
            DiscoveryCommands::Trending => deprecated_moralis_endpoint(
                "discovery trending",
                &["GET /discovery/tokens/trending"],
                Some("use `ethcli moralis token trending`"),
            ),
            DiscoveryCommands::Filter { .. } => {
                deprecated_moralis_endpoint("discovery filter", &["POST /discovery/tokens"], None)
            }
            DiscoveryCommands::Token { .. } => {
                deprecated_moralis_endpoint("discovery token", &["GET /discovery/token"], None)
            }
            DiscoveryCommands::TokenAnalytics { .. } | DiscoveryCommands::TokenScore { .. } => {
                Ok(())
            }
        },
        MoralisCommands::Volume { action, .. } => match action {
            VolumeCommands::Chains => {
                deprecated_moralis_endpoint("volume chains", &["GET /volume/chains"], None)
            }
            VolumeCommands::Categories => {
                deprecated_moralis_endpoint("volume categories", &["GET /volume/categories"], None)
            }
            VolumeCommands::Timeseries { .. } => {
                deprecated_moralis_endpoint("volume timeseries", &["GET /volume/timeseries"], None)
            }
            VolumeCommands::CategoryTimeseries { .. } => deprecated_moralis_endpoint(
                "volume category-timeseries",
                &["GET /volume/timeseries/{category_id}"],
                None,
            ),
        },
        _ => Ok(()),
    }
}

#[derive(Args)]
pub struct MoralisArgs {
    /// Chain (e.g., eth, polygon, bsc, arbitrum)
    #[arg(long, short, default_value = "eth")]
    pub chain: String,

    /// Output format
    #[arg(long, short = 'o', visible_alias = "output", default_value = "json")]
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

    /// Transaction operations
    Transaction {
        #[command(subcommand)]
        action: TransactionCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Block operations
    Block {
        #[command(subcommand)]
        action: BlockCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// DeFi operations
    Defi {
        #[command(subcommand)]
        action: DefiCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Token discovery
    Discovery {
        #[command(subcommand)]
        action: DiscoveryCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Token analytics
    Analytics {
        #[command(subcommand)]
        action: AnalyticsCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Entity operations (wallets, protocols, exchanges)
    Entities {
        #[command(subcommand)]
        action: EntitiesCommands,

        #[command(flatten)]
        args: MoralisArgs,
    },

    /// Volume analytics
    Volume {
        #[command(subcommand)]
        action: VolumeCommands,

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

    /// Get profitability by token
    ProfitabilityTokens {
        /// Wallet address
        address: String,
    },

    /// Get balances for multiple wallets (batch)
    MultipleBalances {
        /// Comma-separated wallet addresses
        addresses: String,
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

    /// Get token transfers for a wallet address
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

        /// Timeframe (e.g., 1h, 4h, 1d, 1w)
        #[arg(long, default_value = "1d")]
        timeframe: String,

        /// From date (ISO 8601, e.g. 2024-01-01T00:00:00Z)
        #[arg(long)]
        from_date: Option<String>,

        /// To date (ISO 8601)
        #[arg(long)]
        to_date: Option<String>,

        /// Limit number of results
        #[arg(long)]
        limit: Option<i32>,
    },

    /// Get pair stats
    PairStats {
        /// Pair address
        address: String,
    },

    /// Get wallet swaps
    WalletSwaps {
        /// Wallet address
        address: String,
    },

    /// Get pair swaps
    PairSwaps {
        /// Pair address
        address: String,
    },

    /// Get token categories
    Categories,

    /// Get new tokens on an exchange (e.g., uniswap, pancakeswap)
    ExchangeNewTokens {
        /// Exchange name (e.g., uniswapv2, uniswapv3, pancakeswap)
        exchange: String,
    },

    /// Get bonding tokens on an exchange (e.g., pump.fun)
    ExchangeBondingTokens {
        /// Exchange name
        exchange: String,
    },

    /// Get graduated tokens on an exchange
    ExchangeGraduatedTokens {
        /// Exchange name
        exchange: String,
    },

    /// Get multiple token prices (batch, comma-separated addresses)
    MultiplePrices {
        /// Comma-separated token addresses
        addresses: String,
    },

    /// Get tokens by symbols (comma-separated)
    BySymbols {
        /// Comma-separated token symbols (e.g., USDC,WETH,DAI)
        symbols: String,
    },

    /// Get contract transfers for a token (not wallet transfers)
    ContractTransfers {
        /// Token contract address
        address: String,
    },

    /// Get holders summary for a token
    HoldersSummary {
        /// Token contract address
        address: String,
    },

    /// Get historical holders data for a token
    HoldersHistorical {
        /// Token contract address
        address: String,
    },

    /// Get aggregated pair stats for a token
    PairsStats {
        /// Token contract address
        address: String,
    },

    /// Get top gainers/traders for a token
    TopGainers {
        /// Token contract address
        address: String,
    },

    /// Get pair snipers
    PairSnipers {
        /// Pair address
        address: String,
    },

    /// Get token bonding status (pump.fun, etc.)
    BondingStatus {
        /// Token contract address
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

    /// Get NFT collections owned by a wallet
    WalletCollections {
        /// Wallet address
        address: String,
    },

    /// Get NFT transfers by contract
    ContractTransfers {
        /// Contract address
        contract: String,
    },

    /// Get NFT transfers for a specific token
    TokenTransfers {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get owners of a specific NFT token
    TokenOwners {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get floor price for a specific token
    TokenFloorPrice {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get trades for a specific token
    TokenTrades {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get wallet NFT trades
    WalletTrades {
        /// Wallet address
        address: String,
    },

    /// Get collection traits
    CollectionTraits {
        /// Contract address
        address: String,
    },

    /// Get collection traits (paginated)
    CollectionTraitsPaginated {
        /// Contract address
        address: String,
    },

    /// Get unique owners count
    UniqueOwners {
        /// Contract address
        address: String,
    },

    /// Resync NFT metadata
    ResyncMetadata {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
    },

    /// Get multiple NFTs by token addresses and IDs (batch, format: addr:id,addr:id)
    MultipleNfts {
        /// Comma-separated token_address:token_id pairs
        tokens: String,
    },

    /// Get historical floor price for a collection
    FloorPriceHistorical {
        /// Contract address
        address: String,
    },

    /// Sync NFT collection metadata
    SyncCollection {
        /// Contract address
        address: String,
    },

    /// Get NFTs by contract address
    ContractNfts {
        /// Contract address
        address: String,
    },

    /// Get metadata for multiple NFT contracts (batch, comma-separated)
    MultipleCollections {
        /// Comma-separated contract addresses
        addresses: String,
    },

    /// Resync NFT collection traits
    ResyncTraits {
        /// Contract address
        address: String,
    },

    /// Get NFT sale prices by collection
    CollectionPrices {
        /// Contract address
        address: String,
    },

    /// Get NFT sale prices by token
    TokenPrices {
        /// Contract address
        contract: String,
        /// Token ID
        token_id: String,
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

    /// Get all domains for an address
    AddressDomains {
        /// Wallet address
        address: String,
    },

    /// Get ENS domain details
    EnsDomain {
        /// ENS domain (e.g., vitalik.eth)
        domain: String,
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

    /// Get hottest NFT collections
    HottestNfts,

    /// Get global market cap
    GlobalMarketCap,

    /// Get global volume
    GlobalVolume,
}

#[derive(Subcommand)]
pub enum TransactionCommands {
    /// Get transaction by hash
    Get {
        /// Transaction hash
        tx_hash: String,
    },

    /// Get verbose transaction with decoded data and internal transactions
    Verbose {
        /// Transaction hash
        tx_hash: String,

        /// Include internal transactions
        #[arg(long)]
        include_internal: bool,
    },

    /// Get transactions for a wallet address
    Wallet {
        /// Wallet address
        address: String,
    },

    /// Get verbose transactions for a wallet address
    WalletVerbose {
        /// Wallet address
        address: String,

        /// Include internal transactions
        #[arg(long)]
        include_internal: bool,
    },
}

#[derive(Subcommand)]
pub enum BlockCommands {
    /// Get block by number or hash
    Get {
        /// Block number or hash
        block_number_or_hash: String,

        /// Include transactions
        #[arg(long)]
        include_transactions: bool,
    },

    /// Get latest block number for a chain
    Latest,

    /// Get block number by date (ISO 8601 format, e.g. 2023-01-01T00:00:00Z)
    DateToBlock {
        /// Date in ISO 8601 format (e.g. 2023-01-01T00:00:00Z)
        date: String,
    },
}

#[derive(Subcommand)]
pub enum DefiCommands {
    /// Get price between two tokens in a pair
    PairPrice {
        /// Token 0 address
        token0: String,
        /// Token 1 address
        token1: String,

        /// DEX exchange (e.g., uniswapv2, sushiswap)
        #[arg(long)]
        exchange: Option<String>,
    },

    /// Get reserves for a pair
    PairReserves {
        /// Pair address
        pair_address: String,
    },

    /// Get pair address for two tokens
    PairAddress {
        /// Token 0 address
        token0: String,
        /// Token 1 address
        token1: String,

        /// DEX exchange
        #[arg(long)]
        exchange: Option<String>,
    },

    /// Get DeFi summary for a wallet
    WalletSummary {
        /// Wallet address
        address: String,
    },

    /// Get all DeFi positions for a wallet
    WalletPositions {
        /// Wallet address
        address: String,
    },

    /// Get DeFi positions for a specific protocol
    ProtocolPositions {
        /// Wallet address
        address: String,
        /// Protocol name (e.g., aave, uniswap-v3)
        protocol: String,
    },
}

#[derive(Subcommand)]
pub enum DiscoveryCommands {
    /// Get tokens with rising liquidity
    RisingLiquidity,

    /// Get tokens with buying pressure
    BuyingPressure,

    /// Get solid performers
    SolidPerformers,

    /// Get tokens with experienced buyers
    ExperiencedBuyers,

    /// Get risky bet tokens
    RiskyBets,

    /// Get blue chip tokens
    BlueChip,

    /// Get top gainers
    TopGainers,

    /// Get top losers
    TopLosers,

    /// Get trending tokens (discovery)
    Trending,

    /// Get token analytics (buyers/sellers/volume)
    TokenAnalytics {
        /// Token contract address
        address: String,
    },

    /// Get token score (security, verified, spam)
    TokenScore {
        /// Token contract address
        address: String,
    },

    /// Filter tokens with custom criteria
    Filter {
        /// Minimum market cap
        #[arg(long)]
        min_market_cap: Option<f64>,

        /// Maximum market cap
        #[arg(long)]
        max_market_cap: Option<f64>,

        /// Minimum liquidity
        #[arg(long)]
        min_liquidity: Option<f64>,

        /// Maximum liquidity
        #[arg(long)]
        max_liquidity: Option<f64>,

        /// Minimum 24h volume
        #[arg(long)]
        min_volume_24h: Option<f64>,

        /// Maximum 24h volume
        #[arg(long)]
        max_volume_24h: Option<f64>,

        /// Minimum holders
        #[arg(long)]
        min_holders: Option<i64>,

        /// Minimum security score
        #[arg(long)]
        min_security_score: Option<i32>,
    },

    /// Get single token details from discovery
    Token {
        /// Token contract address
        address: String,
    },
}

#[derive(Subcommand)]
pub enum AnalyticsCommands {
    /// Get token analytics timeseries (batch, comma-separated addresses)
    Timeseries {
        /// Comma-separated token addresses
        addresses: String,

        /// Timeframe (e.g., 1h, 4h, 1d)
        #[arg(long)]
        timeframe: Option<String>,

        /// From date (ISO 8601)
        #[arg(long)]
        from_date: Option<String>,

        /// To date (ISO 8601)
        #[arg(long)]
        to_date: Option<String>,
    },

    /// Get batch token analytics (comma-separated addresses)
    Batch {
        /// Comma-separated token addresses
        addresses: String,
    },
}

#[derive(Subcommand)]
pub enum EntitiesCommands {
    /// Search for entities
    Search {
        /// Search query
        query: String,
    },

    /// Get entity by ID
    Get {
        /// Entity ID
        entity_id: String,
    },

    /// Get all entity categories
    Categories,

    /// Get entities in a category
    CategoryEntities {
        /// Category ID
        category_id: String,
    },
}

#[derive(Subcommand)]
pub enum VolumeCommands {
    /// Get volume by chain
    Chains,

    /// Get volume by category
    Categories,

    /// Get overall volume timeseries
    Timeseries {
        /// Timeframe (e.g., 1h, 4h, 1d)
        #[arg(long)]
        timeframe: Option<String>,

        /// From date (ISO 8601)
        #[arg(long)]
        from_date: Option<String>,

        /// To date (ISO 8601)
        #[arg(long)]
        to_date: Option<String>,
    },

    /// Get volume timeseries for a category
    CategoryTimeseries {
        /// Category ID
        category_id: String,

        /// Timeframe (e.g., 1h, 4h, 1d)
        #[arg(long)]
        timeframe: Option<String>,

        /// From date (ISO 8601)
        #[arg(long)]
        from_date: Option<String>,

        /// To date (ISO 8601)
        #[arg(long)]
        to_date: Option<String>,
    },
}

/// Handle Moralis commands
pub async fn handle(command: &MoralisCommands, quiet: bool) -> anyhow::Result<()> {
    use secrecy::ExposeSecret;

    let args = moralis_command_args(command);
    ensure_moralis_chain_supported(&args.chain)?;
    reject_deprecated_moralis_command(command)?;

    // Try config first, then fall back to env var
    let client = if let Ok(Some(config)) = ConfigFile::load_default() {
        if let Some(ref moralis_config) = config.moralis {
            mrls::Client::new(moralis_config.api_key.expose_secret())?
        } else {
            mrls::Client::from_env()
                .map_err(|_| anyhow::anyhow!("MORALIS_API_KEY not set in config or environment"))?
        }
    } else {
        mrls::Client::from_env()
            .map_err(|_| anyhow::anyhow!("MORALIS_API_KEY not set in config or environment"))?
    };

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
        MoralisCommands::Transaction { action, args } => {
            handle_transaction(&client, action, args, quiet).await
        }
        MoralisCommands::Block { action, args } => handle_block(&client, action, args, quiet).await,
        MoralisCommands::Defi { action, args } => handle_defi(&client, action, args, quiet).await,
        MoralisCommands::Discovery { action, args } => {
            handle_discovery(&client, action, args, quiet).await
        }
        MoralisCommands::Analytics { action, args } => {
            handle_analytics(&client, action, args, quiet).await
        }
        MoralisCommands::Entities { action, args } => {
            handle_entities(&client, action, args, quiet).await
        }
        MoralisCommands::Volume { action, args } => {
            handle_volume(&client, action, args, quiet).await
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
        WalletCommands::ProfitabilityTokens { address } => {
            if !quiet {
                eprintln!("Fetching profitability by token for {}...", address);
            }
            let query = mrls::WalletQuery::new().chain(&args.chain);
            let response = client
                .wallet()
                .get_profitability(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        WalletCommands::MultipleBalances { addresses } => {
            let addrs: Vec<&str> = addresses.split(',').map(str::trim).collect();
            if !quiet {
                eprintln!("Fetching balances for {} wallets...", addrs.len());
            }
            let response = client
                .wallet()
                .get_multiple_balances(&addrs, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

#[allow(deprecated)]
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
        TokenCommands::PairOhlcv {
            address,
            timeframe,
            from_date,
            to_date,
            limit,
        } => {
            if !quiet {
                eprintln!("Fetching OHLCV for pair {}...", address);
            }
            let mut query = mrls::token::PairOhlcvQuery::new()
                .chain(&args.chain)
                .timeframe(timeframe);
            if let Some(fd) = from_date {
                query = query.from_date(fd);
            }
            if let Some(td) = to_date {
                query = query.to_date(td);
            }
            if let Some(l) = limit {
                query = query.limit(*l);
            }
            let response = client.token().get_pair_ohlcv(address, Some(&query)).await?;
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
        TokenCommands::WalletSwaps { address } => {
            if !quiet {
                eprintln!("Fetching wallet swaps for {}...", address);
            }
            let response = client
                .token()
                .get_wallet_swaps(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::PairSwaps { address } => {
            if !quiet {
                eprintln!("Fetching pair swaps for {}...", address);
            }
            let response = client
                .token()
                .get_pair_swaps(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::Categories => {
            if !quiet {
                eprintln!("Fetching token categories...");
            }
            let response = client.token().get_categories().await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::ExchangeNewTokens { exchange } => {
            if !quiet {
                eprintln!("Fetching new tokens on {}...", exchange);
            }
            let response = client
                .token()
                .get_exchange_new_tokens(exchange, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::ExchangeBondingTokens { exchange } => {
            if !quiet {
                eprintln!("Fetching bonding tokens on {}...", exchange);
            }
            let response = client
                .token()
                .get_exchange_bonding_tokens(exchange, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::ExchangeGraduatedTokens { exchange } => {
            if !quiet {
                eprintln!("Fetching graduated tokens on {}...", exchange);
            }
            let response = client
                .token()
                .get_exchange_graduated_tokens(exchange, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::MultiplePrices { addresses } => {
            let addrs: Vec<&str> = addresses.split(',').map(str::trim).collect();
            if !quiet {
                eprintln!("Fetching prices for {} tokens...", addrs.len());
            }
            let request = mrls::token::GetMultiplePricesRequest {
                tokens: addrs
                    .iter()
                    .map(|a| mrls::token::TokenAddressInput {
                        token_address: a.to_string(),
                        exchange: None,
                    })
                    .collect(),
            };
            let response = client
                .token()
                .get_multiple_prices(&request, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::BySymbols { symbols } => {
            let syms: Vec<&str> = symbols.split(',').map(str::trim).collect();
            if !quiet {
                eprintln!("Fetching tokens by symbols: {}...", symbols);
            }
            let response = client
                .token()
                .get_by_symbols(&syms, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::ContractTransfers { address } => {
            if !quiet {
                eprintln!("Fetching contract transfers for {}...", address);
            }
            let response = client
                .token()
                .get_contract_transfers(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::HoldersSummary { address } => {
            if !quiet {
                eprintln!("Fetching holders summary for {}...", address);
            }
            let response = client
                .token()
                .get_holders_summary(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::HoldersHistorical { address } => {
            if !quiet {
                eprintln!("Fetching historical holders for {}...", address);
            }
            let response = client
                .token()
                .get_holders_historical(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::PairsStats { address } => {
            if !quiet {
                eprintln!("Fetching aggregated pairs stats for {}...", address);
            }
            let response = client
                .token()
                .get_pairs_stats(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::TopGainers { address } => {
            if !quiet {
                eprintln!("Fetching top gainers for {}...", address);
            }
            let response = client
                .token()
                .get_top_gainers(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::PairSnipers { address } => {
            if !quiet {
                eprintln!("Fetching snipers for pair {}...", address);
            }
            let response = client
                .token()
                .get_pair_snipers(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TokenCommands::BondingStatus { address } => {
            if !quiet {
                eprintln!("Fetching bonding status for {}...", address);
            }
            let response = client
                .token()
                .get_bonding_status(address, Some(&args.chain))
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
        NftCommands::WalletCollections { address } => {
            if !quiet {
                eprintln!("Fetching NFT collections for wallet {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_wallet_collections(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ContractTransfers { contract } => {
            if !quiet {
                eprintln!("Fetching NFT transfers for contract {}...", contract);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_contract_transfers(contract, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::TokenTransfers { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching transfers for NFT {}:{}...", contract, token_id);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_token_transfers(contract, token_id, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::TokenOwners { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching owners for NFT {}:{}...", contract, token_id);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_token_owners(contract, token_id, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::TokenFloorPrice { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching floor price for NFT {}:{}...", contract, token_id);
            }
            let response = client
                .nft()
                .get_token_floor_price(contract, token_id, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::TokenTrades { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching trades for NFT {}:{}...", contract, token_id);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_token_trades(contract, token_id, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::WalletTrades { address } => {
            if !quiet {
                eprintln!("Fetching NFT trades for wallet {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_wallet_trades(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::CollectionTraits { address } => {
            if !quiet {
                eprintln!("Fetching traits for collection {}...", address);
            }
            let response = client
                .nft()
                .get_collection_traits(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::CollectionTraitsPaginated { address } => {
            if !quiet {
                eprintln!("Fetching paginated traits for collection {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_collection_traits_paginated(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::UniqueOwners { address } => {
            if !quiet {
                eprintln!("Fetching unique owners for {}...", address);
            }
            let response = client
                .nft()
                .get_unique_owners(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ResyncMetadata { contract, token_id } => {
            if !quiet {
                eprintln!("Resyncing metadata for NFT {}:{}...", contract, token_id);
            }
            let response = client
                .nft()
                .resync_metadata(contract, token_id, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::MultipleNfts { tokens } => {
            // Format: "addr1:id1,addr2:id2"
            let token_inputs: Vec<mrls::nft::NftTokenInput> = tokens
                .split(',')
                .map(str::trim)
                .filter_map(|t| {
                    let parts: Vec<&str> = t.splitn(2, ':').collect();
                    if parts.len() == 2 {
                        Some(mrls::nft::NftTokenInput {
                            token_address: parts[0].to_string(),
                            token_id: parts[1].to_string(),
                        })
                    } else {
                        None
                    }
                })
                .collect();
            if !quiet {
                eprintln!("Fetching {} NFTs...", token_inputs.len());
            }
            let request = mrls::nft::GetMultipleNftsRequest {
                tokens: token_inputs,
                normalise_metadata: None,
                media_items: None,
            };
            let response = client
                .nft()
                .get_multiple_nfts(&request, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::FloorPriceHistorical { address } => {
            if !quiet {
                eprintln!("Fetching historical floor price for {}...", address);
            }
            let response = client
                .nft()
                .get_floor_price_historical(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::SyncCollection { address } => {
            if !quiet {
                eprintln!("Syncing collection metadata for {}...", address);
            }
            let response = client
                .nft()
                .sync_collection(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ContractNfts { address } => {
            if !quiet {
                eprintln!("Fetching NFTs for contract {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_contract_nfts(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::MultipleCollections { addresses } => {
            let addrs: Vec<String> = addresses.split(',').map(|s| s.trim().to_string()).collect();
            if !quiet {
                eprintln!("Fetching metadata for {} collections...", addrs.len());
            }
            let request = mrls::nft::GetMultipleCollectionsRequest { addresses: addrs };
            let response = client
                .nft()
                .get_multiple_collections(&request, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::ResyncTraits { address } => {
            if !quiet {
                eprintln!("Resyncing traits for collection {}...", address);
            }
            let response = client
                .nft()
                .resync_traits(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::CollectionPrices { address } => {
            if !quiet {
                eprintln!("Fetching sale prices for collection {}...", address);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_collection_prices(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        NftCommands::TokenPrices { contract, token_id } => {
            if !quiet {
                eprintln!("Fetching sale prices for NFT {}:{}...", contract, token_id);
            }
            let query = mrls::NftQuery::new().chain(&args.chain);
            let response = client
                .nft()
                .get_token_prices(contract, token_id, Some(&query))
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
        ResolveCommands::AddressDomains { address } => {
            if !quiet {
                eprintln!("Fetching domains for {}...", address);
            }
            let response = client.resolve().get_address_domains(address).await?;
            print_output(&response, args.format)?;
        }
        ResolveCommands::EnsDomain { domain } => {
            if !quiet {
                eprintln!("Fetching ENS domain details for {}...", domain);
            }
            let response = client.resolve().get_ens_domain(domain).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

#[allow(deprecated)]
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
        MarketCommands::HottestNfts => {
            if !quiet {
                eprintln!("Fetching hottest NFT collections...");
            }
            let response = client.market().get_hottest_nft_collections(None).await?;
            print_output(&response, args.format)?;
        }
        MarketCommands::GlobalMarketCap => {
            if !quiet {
                eprintln!("Fetching global market cap...");
            }
            let response = client.market().get_global_market_cap().await?;
            print_output(&response, args.format)?;
        }
        MarketCommands::GlobalVolume => {
            if !quiet {
                eprintln!("Fetching global volume...");
            }
            let response = client.market().get_global_volume().await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_transaction(
    client: &mrls::Client,
    action: &TransactionCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        TransactionCommands::Get { tx_hash } => {
            if !quiet {
                eprintln!("Fetching transaction {}...", tx_hash);
            }
            let response = client
                .transaction()
                .get_transaction(tx_hash, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TransactionCommands::Verbose {
            tx_hash,
            include_internal,
        } => {
            if !quiet {
                eprintln!("Fetching verbose transaction {}...", tx_hash);
            }
            let query = if *include_internal {
                Some(
                    mrls::TransactionQuery::new()
                        .chain(&args.chain)
                        .include_internal_transactions(true),
                )
            } else {
                Some(mrls::TransactionQuery::new().chain(&args.chain))
            };
            let response = client
                .transaction()
                .get_transaction_verbose(tx_hash, query.as_ref())
                .await?;
            print_output(&response, args.format)?;
        }
        TransactionCommands::Wallet { address } => {
            if !quiet {
                eprintln!("Fetching transactions for {}...", address);
            }
            let response = client
                .transaction()
                .get_wallet_transactions(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        TransactionCommands::WalletVerbose {
            address,
            include_internal,
        } => {
            if !quiet {
                eprintln!("Fetching verbose transactions for {}...", address);
            }
            let query = if *include_internal {
                Some(
                    mrls::TransactionQuery::new()
                        .chain(&args.chain)
                        .include_internal_transactions(true),
                )
            } else {
                Some(mrls::TransactionQuery::new().chain(&args.chain))
            };
            let response = client
                .transaction()
                .get_wallet_transactions_verbose(address, query.as_ref())
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_block(
    client: &mrls::Client,
    action: &BlockCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        BlockCommands::Get {
            block_number_or_hash,
            include_transactions,
        } => {
            if !quiet {
                eprintln!("Fetching block {}...", block_number_or_hash);
            }
            let mut query = mrls::BlockQuery::new().chain(&args.chain);
            if *include_transactions {
                query = query.include_transactions(true);
            }
            let response = client
                .block()
                .get_block(block_number_or_hash, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        BlockCommands::Latest => {
            if !quiet {
                eprintln!("Fetching latest block number for {}...", args.chain);
            }
            let response = client.block().get_latest_block_number(&args.chain).await?;
            print_output(&response, args.format)?;
        }
        BlockCommands::DateToBlock { date } => {
            if !quiet {
                eprintln!("Converting date {} to block...", date);
            }
            let response = client
                .block()
                .date_to_block(date, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_defi(
    client: &mrls::Client,
    action: &DefiCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        DefiCommands::PairPrice {
            token0,
            token1,
            exchange,
        } => {
            if !quiet {
                eprintln!("Fetching pair price for {} / {}...", token0, token1);
            }
            let mut query = mrls::DefiQuery::new().chain(&args.chain);
            if let Some(ex) = exchange {
                query = query.exchange(ex);
            }
            let response = client
                .defi()
                .get_pair_price(token0, token1, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        DefiCommands::PairReserves { pair_address } => {
            if !quiet {
                eprintln!("Fetching reserves for pair {}...", pair_address);
            }
            let response = client
                .defi()
                .get_pair_reserves(pair_address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        DefiCommands::PairAddress {
            token0,
            token1,
            exchange,
        } => {
            if !quiet {
                eprintln!("Fetching pair address for {} / {}...", token0, token1);
            }
            let mut query = mrls::DefiQuery::new().chain(&args.chain);
            if let Some(ex) = exchange {
                query = query.exchange(ex);
            }
            let response = client
                .defi()
                .get_pair_address(token0, token1, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        DefiCommands::WalletSummary { address } => {
            if !quiet {
                eprintln!("Fetching DeFi summary for {}...", address);
            }
            let response = client.defi().get_wallet_defi_summary(address).await?;
            print_output(&response, args.format)?;
        }
        DefiCommands::WalletPositions { address } => {
            if !quiet {
                eprintln!("Fetching DeFi positions for {}...", address);
            }
            let query = mrls::DefiQuery::new().chain(&args.chain);
            let response = client
                .defi()
                .get_wallet_defi_positions(address, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        DefiCommands::ProtocolPositions { address, protocol } => {
            if !quiet {
                eprintln!("Fetching {} positions for {}...", protocol, address);
            }
            let query = mrls::DefiQuery::new().chain(&args.chain);
            let response = client
                .defi()
                .get_wallet_protocol_positions(address, protocol, Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

#[allow(deprecated)]
async fn handle_discovery(
    client: &mrls::Client,
    action: &DiscoveryCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    let query = mrls::DiscoveryQuery::new().chain(&args.chain);

    match action {
        DiscoveryCommands::RisingLiquidity => {
            if !quiet {
                eprintln!("Fetching tokens with rising liquidity...");
            }
            let response = client
                .discovery()
                .get_rising_liquidity(Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::BuyingPressure => {
            if !quiet {
                eprintln!("Fetching tokens with buying pressure...");
            }
            let response = client.discovery().get_buying_pressure(Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::SolidPerformers => {
            if !quiet {
                eprintln!("Fetching solid performers...");
            }
            let response = client
                .discovery()
                .get_solid_performers(Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::ExperiencedBuyers => {
            if !quiet {
                eprintln!("Fetching tokens with experienced buyers...");
            }
            let response = client
                .discovery()
                .get_experienced_buyers(Some(&query))
                .await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::RiskyBets => {
            if !quiet {
                eprintln!("Fetching risky bet tokens...");
            }
            let response = client.discovery().get_risky_bets(Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::BlueChip => {
            if !quiet {
                eprintln!("Fetching blue chip tokens...");
            }
            let response = client.discovery().get_blue_chip(Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::TopGainers => {
            if !quiet {
                eprintln!("Fetching top gainers...");
            }
            let response = client.discovery().get_top_gainers(Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::TopLosers => {
            if !quiet {
                eprintln!("Fetching top losers...");
            }
            let response = client.discovery().get_top_losers(Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::Trending => {
            if !quiet {
                eprintln!("Fetching trending tokens (discovery)...");
            }
            let response = client.discovery().get_trending(Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::TokenAnalytics { address } => {
            if !quiet {
                eprintln!("Fetching analytics for {}...", address);
            }
            let response = client
                .discovery()
                .get_token_analytics(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::TokenScore { address } => {
            if !quiet {
                eprintln!("Fetching score for {}...", address);
            }
            let response = client
                .discovery()
                .get_token_score(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::Filter {
            min_market_cap,
            max_market_cap,
            min_liquidity,
            max_liquidity,
            min_volume_24h,
            max_volume_24h,
            min_holders,
            min_security_score,
        } => {
            if !quiet {
                eprintln!("Filtering tokens with custom criteria...");
            }
            let filter = mrls::discovery::DiscoveryFilter {
                min_market_cap: *min_market_cap,
                max_market_cap: *max_market_cap,
                min_liquidity: *min_liquidity,
                max_liquidity: *max_liquidity,
                min_volume_24h: *min_volume_24h,
                max_volume_24h: *max_volume_24h,
                min_holders: *min_holders,
                min_security_score: *min_security_score,
                chains: Some(vec![args.chain.clone()]),
            };
            let response = client.discovery().filter_tokens(&filter).await?;
            print_output(&response, args.format)?;
        }
        DiscoveryCommands::Token { address } => {
            if !quiet {
                eprintln!("Fetching discovery details for {}...", address);
            }
            let response = client
                .discovery()
                .get_token(address, Some(&args.chain))
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_analytics(
    client: &mrls::Client,
    action: &AnalyticsCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        AnalyticsCommands::Timeseries {
            addresses,
            timeframe,
            from_date,
            to_date,
        } => {
            let addrs: Vec<&str> = addresses.split(',').map(str::trim).collect();
            if !quiet {
                eprintln!(
                    "Fetching analytics timeseries for {} tokens...",
                    addrs.len()
                );
            }
            let request = mrls::analytics::AnalyticsTimeseriesRequest {
                tokens: addrs
                    .iter()
                    .map(|a| mrls::analytics::TokenAnalyticsInput {
                        token_address: a.to_string(),
                        chain: Some(args.chain.clone()),
                    })
                    .collect(),
                timeframe: timeframe.clone(),
                from_date: from_date.clone(),
                to_date: to_date.clone(),
            };
            let response = client.analytics().get_timeseries(&request).await?;
            print_output(&response, args.format)?;
        }
        AnalyticsCommands::Batch { addresses } => {
            let addrs: Vec<&str> = addresses.split(',').map(str::trim).collect();
            if !quiet {
                eprintln!("Fetching batch analytics for {} tokens...", addrs.len());
            }
            let request = mrls::analytics::BatchAnalyticsRequest {
                tokens: addrs
                    .iter()
                    .map(|a| mrls::analytics::TokenAnalyticsInput {
                        token_address: a.to_string(),
                        chain: Some(args.chain.clone()),
                    })
                    .collect(),
            };
            let response = client.analytics().get_batch(&request).await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

async fn handle_entities(
    client: &mrls::Client,
    action: &EntitiesCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        EntitiesCommands::Search { query } => {
            if !quiet {
                eprintln!("Searching entities for {}...", query);
            }
            let eq = mrls::EntityQuery::new().search(query);
            let response = client.entities().search(&eq).await?;
            print_output(&response, args.format)?;
        }
        EntitiesCommands::Get { entity_id } => {
            if !quiet {
                eprintln!("Fetching entity {}...", entity_id);
            }
            let response = client.entities().get_entity(entity_id).await?;
            print_output(&response, args.format)?;
        }
        EntitiesCommands::Categories => {
            if !quiet {
                eprintln!("Fetching entity categories...");
            }
            let response = client.entities().get_categories().await?;
            print_output(&response, args.format)?;
        }
        EntitiesCommands::CategoryEntities { category_id } => {
            if !quiet {
                eprintln!("Fetching entities in category {}...", category_id);
            }
            let response = client
                .entities()
                .get_category_entities(category_id, None)
                .await?;
            print_output(&response, args.format)?;
        }
    }
    Ok(())
}

#[allow(deprecated)]
async fn handle_volume(
    client: &mrls::Client,
    action: &VolumeCommands,
    args: &MoralisArgs,
    quiet: bool,
) -> anyhow::Result<()> {
    match action {
        VolumeCommands::Chains => {
            if !quiet {
                eprintln!("Fetching volume by chain...");
            }
            let response = client.volume().get_chains_volume().await?;
            print_output(&response, args.format)?;
        }
        VolumeCommands::Categories => {
            if !quiet {
                eprintln!("Fetching volume by category...");
            }
            let response = client.volume().get_categories_volume().await?;
            print_output(&response, args.format)?;
        }
        VolumeCommands::Timeseries {
            timeframe,
            from_date,
            to_date,
        } => {
            if !quiet {
                eprintln!("Fetching volume timeseries...");
            }
            let mut query = mrls::VolumeQuery::new().chain(&args.chain);
            if let Some(tf) = timeframe {
                query = query.timeframe(tf);
            }
            if let Some(fd) = from_date {
                query = query.from_date(fd);
            }
            if let Some(td) = to_date {
                query = query.to_date(td);
            }
            let response = client.volume().get_timeseries(Some(&query)).await?;
            print_output(&response, args.format)?;
        }
        VolumeCommands::CategoryTimeseries {
            category_id,
            timeframe,
            from_date,
            to_date,
        } => {
            if !quiet {
                eprintln!("Fetching volume timeseries for category {}...", category_id);
            }
            let mut query = mrls::VolumeQuery::new().chain(&args.chain);
            if let Some(tf) = timeframe {
                query = query.timeframe(tf);
            }
            if let Some(fd) = from_date {
                query = query.from_date(fd);
            }
            if let Some(td) = to_date {
                query = query.to_date(td);
            }
            let response = client
                .volume()
                .get_category_timeseries(category_id, Some(&query))
                .await?;
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
        deprecated_moralis_endpoint, reject_deprecated_moralis_command, DiscoveryCommands,
        MoralisArgs, MoralisCommands, OutputFormat,
    };
    use super::{ensure_moralis_chain_supported, TokenCommands};

    #[test]
    fn moralis_fantom_aliases_are_blocked() {
        for chain in ["fantom", "Fantom", "ftm", "250", "0xfa"] {
            assert!(ensure_moralis_chain_supported(chain).is_err());
        }
    }

    #[test]
    fn supported_moralis_chains_still_pass() {
        for chain in ["eth", "polygon", "bsc", "arbitrum", "base", "optimism"] {
            assert!(ensure_moralis_chain_supported(chain).is_ok());
        }
    }

    #[test]
    fn deprecated_endpoint_message_names_command_and_endpoint() {
        let error =
            deprecated_moralis_endpoint("market top-tokens", &["GET /market-data/test"], None)
                .unwrap_err()
                .to_string();

        assert!(error.contains("market top-tokens"));
        assert!(error.contains("GET /market-data/test"));
        assert!(error.contains("2026-06-04"));
    }

    #[test]
    fn deprecated_moralis_commands_are_rejected_before_client_setup() {
        let args = MoralisArgs {
            chain: "eth".to_string(),
            format: OutputFormat::Json,
        };
        let command = MoralisCommands::Token {
            action: TokenCommands::PairSnipers {
                address: "0xpair".to_string(),
            },
            args,
        };

        let error = reject_deprecated_moralis_command(&command)
            .unwrap_err()
            .to_string();

        assert!(error.contains("pair-snipers"));
        assert!(error.contains("/pairs/{address}/snipers"));
    }

    #[test]
    fn holders_historical_is_rejected_with_its_own_removal_date() {
        let args = MoralisArgs {
            chain: "eth".to_string(),
            format: OutputFormat::Json,
        };
        let command = MoralisCommands::Token {
            action: TokenCommands::HoldersHistorical {
                address: "0xtoken".to_string(),
            },
            args,
        };

        let error = reject_deprecated_moralis_command(&command)
            .unwrap_err()
            .to_string();

        assert!(error.contains("holders-historical"));
        assert!(error.contains("/erc20/{address}/holders/historical"));
        assert!(error.contains("2026-07-31"));
    }

    #[test]
    fn non_deprecated_moralis_commands_are_not_rejected() {
        let args = MoralisArgs {
            chain: "eth".to_string(),
            format: OutputFormat::Json,
        };
        let command = MoralisCommands::Discovery {
            action: DiscoveryCommands::TokenScore {
                address: "0xtoken".to_string(),
            },
            args,
        };

        assert!(reject_deprecated_moralis_command(&command).is_ok());
    }
}
