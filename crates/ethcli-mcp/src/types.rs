//! Input types for MCP tools
//!
//! These structs define the JSON Schema for tool parameters.
//! All types derive JsonSchema for MCP protocol compatibility.

use rmcp::schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// =============================================================================
// INPUT TYPES (with JSON Schema for MCP)
// =============================================================================

fn default_chain() -> String {
    "ethereum".to_string()
}

fn default_moralis_chain() -> String {
    "eth".to_string()
}

fn default_network() -> String {
    "eth-mainnet".to_string()
}

fn default_unit() -> String {
    "ether".to_string()
}

// --- Logs ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LogsInput {
    /// Contract address to query logs from
    pub contract: String,
    /// Event signature (e.g., "Transfer(address,address,uint256)")
    pub event: Option<String>,
    /// Starting block number or tag (e.g., "latest", "0x1234")
    pub from_block: Option<String>,
    /// Ending block number or tag
    pub to_block: Option<String>,
    /// Additional topic filters
    pub topics: Option<Vec<String>>,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Transaction ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TxAnalyzeInput {
    /// Transaction hash (with or without 0x prefix).
    /// Partial hashes are supported when block is specified.
    pub hash: String,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number for partial hash resolution.
    /// When provided with a partial tx hash (e.g. from account_txs output),
    /// the transaction will be looked up by matching the prefix within this block.
    #[serde(default)]
    pub block: Option<u64>,
}

// --- Account ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountAddressInput {
    /// Ethereum address (hex format)
    pub address: String,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountBalanceInput {
    /// Ethereum address(es) - supports multiple addresses
    #[serde(default)]
    pub addresses: Vec<String>,
    /// Ethereum address (legacy singular form)
    pub address: Option<String>,
    /// Chain name (ethereum, polygon, arbitrum, gnosis, fantom, linea, zksync, etc.) or numeric chain ID
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountTxsInput {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
    /// Sort order: "asc" or "desc" (default: desc)
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountErc20Input {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Filter by token contract address
    pub token: Option<String>,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountErc721Input {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Filter by token contract address
    pub token: Option<String>,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountErc1155Input {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Filter by token contract address
    pub token: Option<String>,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AccountInternalTxsInput {
    /// Ethereum address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Page number (1-indexed)
    pub page: Option<u32>,
    /// Number of results per page (default: 50)
    pub limit: Option<u32>,
}

// --- Contract ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractAddressInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractAbiInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Disabled over MCP; omit to return the ABI in the tool response
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractSourceInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Disabled over MCP; omit to return source data in the tool response
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractSelectorsInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Lookup function signatures from 4byte.directory
    #[serde(default)]
    pub lookup: bool,
    /// If proxy detected, extract selectors from the implementation contract instead
    #[serde(default)]
    pub follow_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractDisassembleInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Maximum number of opcodes to show
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractOpcodesInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractAnalyzeInput {
    /// Contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Include full opcode disassembly in output
    #[serde(default)]
    pub include_disassembly: bool,
    /// Limit number of opcodes in disassembly (requires include_disassembly)
    pub limit: Option<u32>,
    /// Lookup function signatures from 4byte.directory
    #[serde(default)]
    pub lookup: bool,
    /// If proxy detected, analyze the implementation contract instead
    #[serde(default)]
    pub follow_proxy: bool,
    /// Include selector -> handler offset dispatcher mapping
    #[serde(default)]
    pub dispatcher: bool,
    /// Include handler guard/check heuristics for auth, calldata hash validation, storage checks, and external calls
    #[serde(default)]
    pub checks: bool,
}

// --- Token ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenInfoInput {
    /// Token contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenHoldersInput {
    /// Token contract address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Maximum number of holders to return
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TokenBalanceInput {
    /// Token contract address(es) or labels. Use "eth" for native ETH balance.
    #[serde(default)]
    pub tokens: Vec<String>,
    /// Token contract address or label (legacy singular form)
    pub token: Option<String>,
    /// Holder address(es) to check balance for
    #[serde(default)]
    pub holders: Vec<String>,
    /// Holder address (legacy singular form)
    pub address: Option<String>,
    /// Get balances for all addresses with this tag (can combine with holders)
    pub tag: Option<String>,
    /// Show zero balances (hidden by default when multiple holders)
    #[serde(default)]
    pub show_zero: bool,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Gas ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GasOracleInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GasEstimateInput {
    /// Gas price in gwei to estimate confirmation time for
    pub gwei: u64,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Signature Lookup ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SigLookupInput {
    /// 4-byte function selector (e.g., "0xa9059cbb") or event topic
    pub selector: String,
}

// --- Cast (Conversions) ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastToWeiInput {
    /// Amount to convert
    pub amount: String,
    /// Unit (ether, gwei, or wei)
    #[serde(default = "default_unit")]
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastFromWeiInput {
    /// Wei amount to convert
    pub wei: String,
    /// Target unit (ether or gwei)
    #[serde(default = "default_unit")]
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastValueInput {
    /// Value to convert
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastSigInput {
    /// Function signature (e.g., "transfer(address,uint256)")
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastAbiEncodeInput {
    /// Function signature
    pub sig: String,
    /// Arguments to encode
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastAbiDecodeInput {
    /// Type signature for raw ABI data, or function signature for calldata with selector
    /// (e.g. "uint256", "(address,uint256)", or "transfer(address,uint256)")
    pub sig: String,
    /// Encoded data to decode
    pub data: String,
}

// --- RPC ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcCallInput {
    /// Contract address
    pub to: String,
    /// Calldata (hex encoded)
    pub data: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number or tag
    pub block: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcBlockInput {
    /// Block number, hash, or tag (latest, pending, etc.)
    pub block: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Show full transactions (not just hashes)
    #[serde(default)]
    pub full: bool,
    /// Custom RPC URL (overrides default)
    pub rpc_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcStorageInput {
    /// Contract address
    pub address: String,
    /// Storage slot
    pub slot: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcAddressInput {
    /// Address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcHashInput {
    /// Transaction hash
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RpcChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- ENS ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsResolveInput {
    /// ENS name (e.g., "vitalik.eth")
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsLookupInput {
    /// Ethereum address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsNamehashInput {
    /// ENS name to hash
    pub name: String,
}

// --- Price ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PriceInput {
    /// Token symbol or address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Portfolio ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PortfolioInput {
    /// Wallet address(es) to query. Can specify multiple.
    /// If empty/omitted, uses "me" from address book.
    #[serde(default)]
    pub addresses: Vec<String>,
    /// Query all addresses with this tag from address book
    pub tag: Option<String>,
    /// Aggregate all wallets into a single combined view
    #[serde(default)]
    pub aggregate: bool,
    /// Chain(s) to query (default: ethereum)
    pub chain: Option<String>,
    /// Source to query: all, alchemy, moralis, dsim, uniswap, yearn.
    /// "all" excludes dsim (Dune Sim shuts down 2026-08-01, issue #64).
    pub source: Option<String>,
    /// Minimum USD value to show (filter small balances)
    pub min_value: Option<f64>,
    /// Show per-source breakdown
    #[serde(default)]
    pub show_sources: bool,
    /// Include blacklisted tokens (normally filtered out)
    #[serde(default)]
    pub show_blacklisted: bool,
}

// --- NFTs ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NftsInput {
    /// Wallet address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Yields ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct YieldsInput {
    /// Protocol name (optional)
    pub protocol: Option<String>,
    /// Chain name (optional)
    pub chain: Option<String>,
}

// --- Quote ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteBestInput {
    /// Source token address (or symbol like ETH, USDC)
    pub from_token: String,
    /// Destination token address (or symbol like ETH, USDC)
    pub to_token: String,
    /// Amount to swap. Use raw units or human format with decimals param.
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of decimals for input amount (converts human amount to raw).
    /// E.g., decimals=18 converts "1.5" to "1500000000000000000"
    pub decimals: Option<u8>,
    /// Sender address (required by some aggregators for accurate quotes)
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in output
    #[serde(default)]
    pub show_tx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteCompareInput {
    /// Source token address (or symbol like ETH, USDC)
    pub from_token: String,
    /// Destination token address (or symbol like ETH, USDC)
    pub to_token: String,
    /// Amount to swap. Use raw units or human format with decimals param.
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of decimals for input amount (converts human amount to raw)
    pub decimals: Option<u8>,
    /// Sender address (required by some aggregators for accurate quotes)
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in output
    #[serde(default)]
    pub show_tx: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QuoteFromInput {
    /// The aggregator to use (open-ocean, kyber-swap, 0x, 1inch, cow-swap, li-fi, velora, enso)
    pub source: String,
    /// Source token address (or symbol like ETH, USDC)
    pub from_token: String,
    /// Destination token address (or symbol like ETH, USDC)
    pub to_token: String,
    /// Amount to swap. Use raw units or human format with decimals param.
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of decimals for input amount (converts human amount to raw)
    pub decimals: Option<u8>,
    /// Sender address (required by some aggregators for accurate quotes)
    pub sender: Option<String>,
    /// Slippage tolerance in basis points (e.g., 50 = 0.5%)
    pub slippage: Option<u32>,
    /// Show transaction data in output
    #[serde(default)]
    pub show_tx: bool,
}

// --- Chainlink ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkPriceInput {
    /// Token symbol (e.g., "ETH", "BTC")
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number for historical price
    pub block: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkFeedInput {
    /// Token symbol
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkOraclesInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsFeedInput {
    /// Feed ID (hex string, e.g., "0x00036...")
    pub feed_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsReportInput {
    /// Feed ID (hex string)
    pub feed_id: String,
    /// Unix timestamp in seconds
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsBulkInput {
    /// Comma-separated feed IDs (hex strings)
    pub feed_ids: String,
    /// Unix timestamp in seconds
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlinkStreamsHistoryInput {
    /// Feed ID (hex string)
    pub feed_id: String,
    /// Start timestamp in seconds
    pub timestamp: u64,
    /// Number of reports to fetch
    #[serde(default = "default_streams_limit")]
    pub limit: u32,
}

fn default_streams_limit() -> u32 {
    10
}

// --- GoPlus Security ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoplusInput {
    /// Address to check
    pub address: String,
    /// Chain ID (1 for Ethereum, 137 for Polygon, etc.)
    pub chain_id: u64,
}

// --- Solodit ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SoloditSearchInput {
    /// Search query
    pub query: String,
    /// Filter by impact (critical, high, medium, low)
    pub impact: Option<String>,
    /// Maximum results
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SoloditGetInput {
    /// Finding slug/ID
    pub slug: String,
}

// --- Uniswap ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapPoolInput {
    /// Pool address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapEthPriceInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Uniswap version (v2, v3)
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapTopPoolsInput {
    /// Number of pools to return
    pub limit: Option<u32>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Uniswap version
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapPositionsInput {
    /// Wallet address
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Uniswap version
    pub version: Option<String>,
}

// --- Curve ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurvePoolsInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveRouteInput {
    /// Source token address
    pub from_token: String,
    /// Destination token address
    pub to_token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Alchemy ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioInput {
    /// Wallet address
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTransfersInput {
    /// Wallet address
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- CoinGecko ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinInput {
    /// CoinGecko coin ID
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoPriceInput {
    /// Comma-separated coin IDs
    pub ids: String,
    /// Target currencies (default: usd)
    pub vs_currencies: Option<String>,
}

// --- DefiLlama ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaTvlInput {
    /// Protocol name
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaYieldsInput {}

// --- DEX Aggregators ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchQuoteInput {
    /// Source token address
    pub src: String,
    /// Destination token address
    pub dst: String,
    /// Amount in smallest unit
    pub amount: String,
    /// Chain ID
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxQuoteInput {
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Sell amount
    pub sell_amount: String,
    /// Taker address
    pub taker: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenoceanQuoteInput {
    /// Input token address
    pub in_token: String,
    /// Output token address
    pub out_token: String,
    /// Amount
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Gas price in Gwei (default: 30)
    #[serde(default = "default_gas_price")]
    pub gas_price: String,
}

fn default_gas_price() -> String {
    "30".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapQuoteInput {
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Amount
    pub amount: String,
    /// From address
    pub from: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiQuoteInput {
    /// Source chain name
    pub from_chain: String,
    /// Source token address
    pub from_token: String,
    /// Destination chain name
    pub to_chain: String,
    /// Destination token address
    pub to_token: String,
    /// Amount
    pub amount: String,
    /// Sender address
    pub from_address: String,
}

// --- Pyth ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PythPriceInput {
    /// Symbol or price feed ID
    pub symbols: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PythSearchInput {
    /// Search query
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PythFeedsInput {
    /// Asset type filter (optional)
    pub asset_type: Option<String>,
}

// --- Address Book ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressAddInput {
    /// Name for the address
    pub name: String,
    /// Ethereum address
    pub address: String,
    /// Description or notes (optional)
    pub description: Option<String>,
    /// Tags for categorization (optional, can specify multiple)
    #[serde(default)]
    pub tags: Vec<String>,
    /// Mark address as chain-specific (e.g., "polygon", "arbitrum")
    pub for_chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressNameInput {
    /// Address name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressImportInput {
    /// Path to JSON file to import
    pub file: String,
    /// Overwrite existing entries with same label
    #[serde(default)]
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressExportInput {
    /// Disabled over MCP; omit to return JSON in the tool response
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddressSearchInput {
    /// Search query
    pub query: String,
}

// --- Blacklist ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistAddInput {
    /// Token contract address to blacklist
    pub address: String,
    /// Token symbol for display (optional)
    pub symbol: Option<String>,
    /// Reason for blacklisting (optional)
    pub reason: Option<String>,
    /// Chain where this token exists (optional)
    pub chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistAddressInput {
    /// Address to check
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistScanInput {
    /// Address to scan
    pub address: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlacklistPathInput {}

// --- Contract Call ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContractCallInput {
    /// Contract address
    pub address: String,
    /// Function name or full signature (e.g. "balanceOf" or "balanceOf(address)")
    pub sig: String,
    /// Arguments
    #[serde(default)]
    pub args: Vec<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Block number or "latest" (default: latest)
    pub block: Option<String>,
    /// Custom RPC URL (overrides config)
    pub rpc_url: Option<String>,
    /// Format output for human readability (commas in numbers, token decimals)
    #[serde(default)]
    pub human: bool,
}

// --- Cast Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastComputeAddressInput {
    /// Deployer address
    pub deployer: String,
    /// Nonce
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastCreate2Input {
    /// Deployer address
    pub deployer: String,
    /// Salt
    pub salt: String,
    /// Init code hash
    pub init_code_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CastConcatInput {
    /// Values to concatenate
    pub values: Vec<String>,
}

// --- Simulate Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateBundleInput {
    /// JSON string of transaction array (e.g., '[{"from":"0x...","to":"0x...","data":"0x..."}]')
    pub txs: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateIdInput {
    /// Simulation ID
    pub id: String,
}

// --- Tenderly ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TenderlySimulateInput {
    /// Contract address
    pub contract: String,
    /// Transaction data
    pub data: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Tenderly VNets ---

fn default_network_id() -> u64 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsCreateInput")]
pub struct TenderlyVnetsCreateInput {
    /// Unique slug identifier for the VNet
    pub slug: String,
    /// Display name for the VNet
    pub name: String,
    /// Network ID to fork from (1 = mainnet, 137 = polygon, etc.)
    #[serde(default = "default_network_id")]
    pub network_id: u64,
    /// Block number to fork from (latest if not specified)
    #[serde(default)]
    pub block_number: Option<u64>,
    /// Custom chain ID for the VNet
    #[serde(default)]
    pub chain_id: Option<u64>,
    /// Enable state sync with parent network
    #[serde(default)]
    pub sync_state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsIdInput")]
pub struct TenderlyVnetsIdInput {
    /// VNet ID (UUID)
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsDeleteInput")]
pub struct TenderlyVnetsDeleteInput {
    /// VNet ID(s) to delete (provide one or more IDs)
    #[serde(default)]
    pub ids: Vec<String>,
    /// Delete all Virtual TestNets
    #[serde(default)]
    pub all: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsUpdateInput")]
pub struct TenderlyVnetsUpdateInput {
    /// VNet ID (UUID)
    pub id: String,
    /// New display name
    #[serde(default)]
    pub name: Option<String>,
    /// New slug
    #[serde(default)]
    pub slug: Option<String>,
    /// Enable/disable state sync
    #[serde(default)]
    pub sync_state: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsForkInput")]
pub struct TenderlyVnetsForkInput {
    /// Source VNet ID to fork from
    pub source: String,
    /// New slug identifier for the fork
    pub slug: String,
    /// Display name for the fork
    pub name: String,
    /// Block number to fork at (latest if not specified)
    #[serde(default)]
    pub block_number: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsTransactionsInput")]
pub struct TenderlyVnetsTransactionsInput {
    /// VNet ID (UUID)
    pub id: String,
    /// Page number
    #[serde(default)]
    pub page: Option<u32>,
    /// Results per page
    #[serde(default)]
    pub per_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsSendInput")]
pub struct TenderlyVnetsSendInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Transaction data (hex encoded)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei (default: 0)
    #[serde(default)]
    pub value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsGetTransactionInput")]
pub struct TenderlyVnetsGetTransactionInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Transaction hash
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsSimulateInput")]
pub struct TenderlyVnetsSimulateInput {
    /// VNet ID to simulate against
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient/contract address (omit for contract deployment)
    #[serde(default)]
    pub to: Option<String>,
    /// Calldata (hex encoded)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei (default: 0)
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<u64>,
    /// Gas price (legacy transactions, in wei)
    #[serde(default)]
    pub gas_price: Option<String>,
    /// Max fee per gas (EIP-1559, in wei)
    #[serde(default)]
    pub max_fee_per_gas: Option<String>,
    /// Max priority fee per gas (EIP-1559, in wei)
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<String>,
}

// --- Tenderly VNets Admin ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminBalanceInput")]
pub struct TenderlyVnetsAdminBalanceInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Account address
    pub address: String,
    /// Amount (wei, or use suffix: 1eth, 100gwei)
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminErc20Input")]
pub struct TenderlyVnetsAdminErc20Input {
    /// VNet ID (UUID)
    pub vnet: String,
    /// ERC20 token contract address
    pub token: String,
    /// Wallet address
    pub wallet: String,
    /// Token amount in smallest unit (only for set-erc20-balance, omit for set-max)
    #[serde(default)]
    pub amount: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminTimeInput")]
pub struct TenderlyVnetsAdminTimeInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Seconds to advance (increase-time) or Unix timestamp (set-timestamp)
    pub value: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminBlocksInput")]
pub struct TenderlyVnetsAdminBlocksInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Number of blocks to skip
    pub blocks: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminVnetInput")]
pub struct TenderlyVnetsAdminVnetInput {
    /// VNet ID (UUID)
    pub vnet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminRevertInput")]
pub struct TenderlyVnetsAdminRevertInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Snapshot ID to revert to
    pub snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminStorageInput")]
pub struct TenderlyVnetsAdminStorageInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Contract address
    pub address: String,
    /// Storage slot (32-byte hex or decimal, auto-padded)
    pub slot: String,
    /// Value to set (32-byte hex or decimal, auto-padded)
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminCodeInput")]
pub struct TenderlyVnetsAdminCodeInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Address to set code at
    pub address: String,
    /// Bytecode (hex string)
    pub code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminSendTxInput")]
pub struct TenderlyVnetsAdminSendTxInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address (optional for contract creation)
    #[serde(default)]
    pub to: Option<String>,
    /// Transaction data (hex)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
}

// --- Tenderly VNet Admin RPC Simulation ---

fn default_block_tag() -> String {
    "latest".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminSimulateTxInput")]
pub struct TenderlyVnetsAdminSimulateTxInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address (optional for contract creation)
    #[serde(default)]
    pub to: Option<String>,
    /// Transaction data (hex calldata)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei (hex or decimal)
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit (hex or decimal)
    #[serde(default)]
    pub gas: Option<String>,
    /// Block tag or hex number (default: "latest")
    #[serde(default = "default_block_tag")]
    pub block: String,
    /// State overrides as JSON string: {"0xaddr": {"balance": "0x...", "stateDiff": {"0x0": "0x1"}}}
    #[serde(default)]
    pub state_overrides: Option<String>,
    /// Block overrides as JSON string: {"time": "0x...", "baseFee": "0x..."}
    #[serde(default)]
    pub block_overrides: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminSimulateBundleInput")]
pub struct TenderlyVnetsAdminSimulateBundleInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Array of transactions as JSON string: [{"from":"0x...","to":"0x...","data":"0x..."}]
    pub txs: String,
    /// State overrides as JSON string
    #[serde(default)]
    pub state_overrides: Option<String>,
    /// Block overrides as JSON string
    #[serde(default)]
    pub block_overrides: Option<String>,
}

// --- Tenderly Actions Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyActionsIdsInput")]
pub struct TenderlyActionsIdsInput {
    /// Action ID(s)
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyActionsGetCallInput")]
pub struct TenderlyActionsGetCallInput {
    /// Action ID
    pub id: String,
    /// Execution ID
    pub execution_id: String,
}

// --- Tenderly Alerts Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyAlertsUpdateInput")]
pub struct TenderlyAlertsUpdateInput {
    /// Alert ID
    pub id: String,
    /// Alert name (required)
    pub name: String,
    /// Alert type (required, e.g., "successful_transaction", "failed_transaction", "event_emitted")
    pub alert_type: String,
    /// Network ID (e.g., "1" for mainnet)
    #[serde(default)]
    pub network: Option<String>,
    /// Target contract addresses (repeatable)
    #[serde(default)]
    pub addresses: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyAlertsAddDestinationInput")]
pub struct TenderlyAlertsAddDestinationInput {
    /// Alert ID
    pub id: String,
    /// Destination type (e.g., "webhook", "email", "slack", "telegram")
    pub destination_type: String,
    /// Destination ID
    pub destination_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyAlertsRemoveDestinationInput")]
pub struct TenderlyAlertsRemoveDestinationInput {
    /// Alert ID
    pub id: String,
    /// Destination ID to remove
    pub destination_id: String,
}

// --- Tenderly Contracts Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyContractsUpdateInput")]
pub struct TenderlyContractsUpdateInput {
    /// Contract address
    pub address: String,
    /// Network name
    #[serde(default)]
    pub network: Option<String>,
    /// New display name
    #[serde(default)]
    pub name: Option<String>,
    /// Tags to set on the contract
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyContractsRemoveTagInput")]
pub struct TenderlyContractsRemoveTagInput {
    /// Contract address
    pub address: String,
    /// Network name
    #[serde(default)]
    pub network: Option<String>,
    /// Tag to remove
    pub tag: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyContractsBulkTagInput")]
pub struct TenderlyContractsBulkTagInput {
    /// Tag to apply
    pub tag: String,
    /// Contract IDs to tag
    pub contract_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyContractsEncodeStateInput")]
pub struct TenderlyContractsEncodeStateInput {
    /// Network name
    #[serde(default)]
    pub network: Option<String>,
    /// State overrides as JSON string
    pub state_json: String,
}

// --- Tenderly VNets Admin Batch ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminBatchBalanceInput")]
pub struct TenderlyVnetsAdminBatchBalanceInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Addresses to set/add balances for
    pub addresses: Vec<String>,
    /// Amount (wei, or use suffix: 10eth, 100gwei)
    pub amount: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(title = "TenderlyVnetsAdminCreateAccessListInput")]
pub struct TenderlyVnetsAdminCreateAccessListInput {
    /// VNet ID (UUID)
    pub vnet: String,
    /// Sender address
    pub from: String,
    /// Recipient address
    pub to: String,
    /// Transaction data (hex calldata)
    #[serde(default)]
    pub data: Option<String>,
    /// Value in wei
    #[serde(default)]
    pub value: Option<String>,
    /// Gas limit
    #[serde(default)]
    pub gas: Option<String>,
    /// Block tag or hex number (default: "latest")
    #[serde(default)]
    pub block: Option<String>,
}

// --- Alchemy Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesInput {
    /// Comma-separated token addresses
    pub tokens: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugInput {
    /// Transaction hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftMetadataInput {
    /// NFT contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftContractInput {
    /// NFT contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftIsHolderInput {
    /// Wallet address to check
    pub address: String,
    /// NFT contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTokenMetadataInput {
    /// Token contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTokenAllowancesInput {
    /// Token contract address
    pub contract: String,
    /// Token owner address
    pub owner: String,
    /// Spender address
    pub spender: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTransfersToInput {
    /// Destination address
    pub address: String,
    /// Starting block number or tag
    #[serde(default)]
    pub from_block: Option<String>,
    /// Ending block number or tag
    #[serde(default)]
    pub to_block: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesByAddressInput {
    /// Comma-separated token contract addresses
    pub addresses: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy NFT (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftContractOwnerInput {
    /// NFT contract address
    pub contract: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftAddressInput {
    /// Wallet address
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftForContractInput {
    /// Contract address
    pub contract: String,
    /// Starting token for pagination
    pub start_token: Option<String>,
    /// Maximum number of NFTs to return
    pub limit: Option<u32>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftSlugInput {
    /// OpenSea collection slug
    pub slug: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftSearchInput {
    /// Search query
    pub query: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftSalesInput {
    /// Contract address
    pub contract: String,
    /// Optional token ID
    pub token_id: Option<String>,
    /// Optional from block number
    pub from_block: Option<u64>,
    /// Optional to block number
    pub to_block: Option<u64>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNftForCollectionInput {
    /// Collection slug
    pub slug: String,
    /// Starting token for pagination
    pub start_token: Option<String>,
    /// Maximum number of NFTs to return
    pub limit: Option<u32>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyChainOnlyInput {
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Token (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTokenBalancesForTokensInput {
    /// Wallet address to query
    pub address: String,
    /// Comma-separated token contract addresses
    pub tokens: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Transfers (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTransfersAllInput {
    /// Address to query (both sent and received)
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Portfolio (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioTokenInfoInput {
    /// Comma-separated tokens in network:address format (e.g., eth-mainnet:0x...)
    pub tokens: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioNftsInput {
    /// Address to query
    pub address: String,
    /// Include NFT metadata (default: true)
    #[serde(default = "default_true")]
    pub with_metadata: bool,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPortfolioNftContractsInput {
    /// Address to query
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Prices (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesBySymbolInput {
    /// Comma-separated token symbols
    pub symbols: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesHistoricalBySymbolInput {
    /// Token symbol (e.g., "ETH")
    pub symbol: String,
    /// Start time (ISO 8601 or Unix timestamp)
    pub start_time: String,
    /// End time (ISO 8601 or Unix timestamp)
    pub end_time: String,
    /// Time interval: 5m, 1h, 1d (default: 1h)
    pub interval: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyPricesHistoricalByAddressInput {
    /// Token contract address
    pub address: String,
    /// Start time (ISO 8601 or Unix timestamp)
    pub start_time: String,
    /// End time (ISO 8601 or Unix timestamp)
    pub end_time: String,
    /// Time interval: 5m, 1h, 1d (default: 1h)
    pub interval: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Debug (additional) ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugTraceCallInput {
    /// Recipient address
    pub to: String,
    /// Block number or tag (default: "latest")
    pub block: Option<String>,
    /// Sender address
    pub from: Option<String>,
    /// Call data (hex)
    pub data: Option<String>,
    /// Value to send (hex)
    pub value: Option<String>,
    /// Gas limit (hex)
    pub gas: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugBlockHashInput {
    /// Block hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyDebugBlockInput {
    /// Block number (hex or decimal) or tag
    pub block: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Trace ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceBlockInput {
    /// Block number (hex or tag)
    pub block: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceCallInput {
    /// Recipient address
    pub to: String,
    /// Block number or tag (default: "latest")
    pub block: Option<String>,
    /// Sender address
    pub from: Option<String>,
    /// Call data (hex)
    pub data: Option<String>,
    /// Value to send (hex)
    pub value: Option<String>,
    /// Gas limit (hex)
    pub gas: Option<String>,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceGetInput {
    /// Transaction hash
    pub hash: String,
    /// Trace indices (comma-separated, e.g., "0,1")
    pub indices: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceRawTxInput {
    /// Raw signed transaction (hex)
    pub raw_tx: String,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceReplayBlockInput {
    /// Block number (hex or tag)
    pub block: String,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceReplayTxInput {
    /// Transaction hash
    pub hash: String,
    /// Trace types (comma-separated: trace,stateDiff,vmTrace; default: trace)
    pub trace_types: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceTxInput {
    /// Transaction hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyTraceFilterInput {
    /// From block (hex or tag)
    pub from_block: Option<String>,
    /// To block (hex or tag)
    pub to_block: Option<String>,
    /// Filter by sender addresses (comma-separated)
    pub from_address: Option<String>,
    /// Filter by recipient addresses (comma-separated)
    pub to_address: Option<String>,
    /// Skip first N traces
    pub after: Option<u32>,
    /// Maximum traces to return
    pub count: Option<u32>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Simulation ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySimAssetChangesInput {
    /// Recipient address
    pub to: String,
    /// Sender address
    pub from: Option<String>,
    /// Call data (hex)
    pub data: Option<String>,
    /// Value to send (hex)
    pub value: Option<String>,
    /// Gas limit (hex)
    pub gas: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySimExecutionInput {
    /// Recipient address
    pub to: String,
    /// Sender address
    pub from: Option<String>,
    /// Call data (hex)
    pub data: Option<String>,
    /// Value to send (hex)
    pub value: Option<String>,
    /// Gas limit (hex)
    pub gas: Option<String>,
    /// Block tag (e.g., "latest")
    pub block: Option<String>,
    /// Output format: nested or flat (default: nested)
    pub trace_format: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Bundler ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBundlerEstimateGasInput {
    /// UserOperation as JSON string
    pub user_op_json: String,
    /// Entry point address (default: 0x5FF137D4b0FDCD49DcA30c7CF57E578a026d2789)
    pub entry_point: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBundlerHashInput {
    /// UserOperation hash
    pub hash: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Gas Manager ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyGasManagerPolicyInput {
    /// Policy ID
    pub policy_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Notify ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyNotifyWebhookInput {
    /// Webhook ID
    pub webhook_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Beacon ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconBlockIdInput {
    /// Block ID (slot number, "head", "finalized", "justified", or block root hash)
    pub block_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconStateIdInput {
    /// State ID (slot number, "head", "finalized", "justified", or state root hash)
    pub state_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconValidatorInput {
    /// State ID
    pub state_id: String,
    /// Validator ID (index or pubkey)
    pub validator_id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconDutiesInput {
    /// Epoch number
    pub epoch: String,
    /// Comma-separated validator indices
    pub validators: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemyBeaconEpochInput {
    /// Epoch number
    pub epoch: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Alchemy Solana ---

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaAssetInput {
    /// Asset ID (mint address)
    pub id: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaAssetsInput {
    /// Comma-separated asset IDs
    pub ids: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaOwnerInput {
    /// Owner address
    pub owner: String,
    /// Page number (default: 1)
    pub page: Option<u32>,
    /// Results per page (default: 100)
    pub limit: Option<u32>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaAddressInput {
    /// Address (creator, authority, etc.)
    pub address: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaGroupInput {
    /// Group key (e.g., "collection")
    pub group_key: String,
    /// Group value (e.g., collection address)
    pub group_value: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaTokenAccountsInput {
    /// Owner address
    pub owner: Option<String>,
    /// Mint address
    pub mint: Option<String>,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AlchemySolanaMintInput {
    /// Mint address
    pub mint: String,
    /// Alchemy network name (e.g., eth-mainnet, polygon-mainnet)
    #[serde(default = "default_network")]
    pub network: String,
}

// --- Gecko Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoNftInput {
    /// NFT collection ID
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainInput {
    /// Network name (e.g., "ethereum")
    pub network: String,
    /// Token address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoTokenPriceInput {
    /// Platform ID (e.g., "ethereum", "polygon-pos")
    pub platform: String,
    /// Comma-separated contract addresses
    pub addresses: String,
    /// Target currencies (comma-separated, default: usd)
    #[serde(default)]
    pub vs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsListInput {
    /// Include platform contract addresses
    #[serde(default)]
    pub with_platforms: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsMarketsInput {
    /// Target currency (e.g., "usd")
    #[serde(default)]
    pub vs_currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsChartInput {
    /// CoinGecko coin ID (e.g., "bitcoin")
    pub id: String,
    /// Target currency (default: usd)
    #[serde(default)]
    pub vs: Option<String>,
    /// Days of data (1, 7, 14, 30, 90, 180, 365, max)
    #[serde(default)]
    pub days: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsOhlcInput {
    /// CoinGecko coin ID (e.g., "bitcoin")
    pub id: String,
    /// Target currency (default: usd)
    #[serde(default)]
    pub vs: Option<String>,
    /// Days of data (1, 7, 14, 30, 90, 180, 365)
    #[serde(default)]
    pub days: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsHistoryInput {
    /// CoinGecko coin ID (e.g., "bitcoin")
    pub id: String,
    /// Date in dd-mm-yyyy format
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoCoinsTopMoversInput {
    /// Target currency (default: usd)
    #[serde(default)]
    pub vs: Option<String>,
    /// Duration: 1h, 24h, 7d, 14d, 30d, 60d, 1y
    #[serde(default)]
    pub duration: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoContractInput {
    /// Platform ID (e.g., "ethereum")
    pub platform: String,
    /// Contract address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoSearchInput {
    /// Search query
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainNetworkInput {
    /// Network ID (e.g., "eth", "polygon_pos")
    pub network: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainOptionalNetworkInput {
    /// Network ID (optional, all networks if omitted)
    #[serde(default)]
    pub network: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainTokenPriceInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Comma-separated token addresses
    pub addresses: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GeckoOnchainPoolOhlcvInput {
    /// Network ID (e.g., "eth")
    pub network: String,
    /// Pool address
    pub address: String,
    /// Timeframe: minute, hour, day (default: hour)
    #[serde(default)]
    pub timeframe: Option<String>,
}

// --- GoPlus Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoplusBatchInput {
    /// Comma-separated addresses
    pub addresses: String,
    /// Chain ID
    pub chain_id: u64,
}

// --- Solodit Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[allow(dead_code)] // Reserved for future use
pub struct SoloditSlugInput {
    /// Finding slug/ID
    pub slug: String,
}

// --- Llama Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaCoinsInput {
    /// Addresses in format "chain:address" (comma-separated)
    pub addresses: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LlamaProtocolInput {
    /// Protocol slug (e.g., "uniswap", "aave")
    pub protocol: String,
}

// --- Moralis ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisChainOnlyInput {
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAddressInput {
    /// Wallet or contract address
    pub address: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDomainInput {
    /// Domain to resolve (e.g., ENS, Unstoppable)
    pub domain: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisSearchInput {
    /// Search query string
    pub query: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisNftMetadataInput {
    /// NFT contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAddressesInput {
    /// Comma-separated addresses
    pub addresses: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisExchangeInput {
    /// Exchange name (e.g., uniswapv2, uniswapv3, pancakeswap)
    pub exchange: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisSymbolsInput {
    /// Comma-separated token symbols (e.g., USDC,WETH,DAI)
    pub symbols: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisNftContractInput {
    /// NFT contract address
    pub contract: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisNftTokenInput {
    /// NFT contract address
    pub contract: String,
    /// Token ID
    pub token_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisMultipleNftsInput {
    /// Comma-separated token_address:token_id pairs
    pub tokens: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisTxHashInput {
    /// Transaction hash
    pub tx_hash: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisTxVerboseInput {
    /// Transaction hash
    pub tx_hash: String,
    /// Include internal transactions
    #[serde(default)]
    pub include_internal: bool,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisWalletVerboseInput {
    /// Wallet address
    pub address: String,
    /// Include internal transactions
    #[serde(default)]
    pub include_internal: bool,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisBlockInput {
    /// Block number or hash
    pub block_number_or_hash: String,
    /// Include transactions in the response
    #[serde(default)]
    pub include_transactions: bool,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDateInput {
    /// Date in ISO 8601 format (e.g. 2023-01-01T00:00:00Z)
    pub date: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDefiPairPriceInput {
    /// Token 0 address
    pub token0: String,
    /// Token 1 address
    pub token1: String,
    /// DEX exchange name (e.g., uniswapv2, sushiswap)
    pub exchange: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDefiPairAddressInput {
    /// Token 0 address
    pub token0: String,
    /// Token 1 address
    pub token1: String,
    /// DEX exchange name
    pub exchange: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisProtocolPositionsInput {
    /// Wallet address
    pub address: String,
    /// Protocol name (e.g., aave, uniswap-v3)
    pub protocol: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisDiscoveryFilterInput {
    /// Minimum market cap
    pub min_market_cap: Option<f64>,
    /// Maximum market cap
    pub max_market_cap: Option<f64>,
    /// Minimum liquidity
    pub min_liquidity: Option<f64>,
    /// Maximum liquidity
    pub max_liquidity: Option<f64>,
    /// Minimum 24h volume
    pub min_volume_24h: Option<f64>,
    /// Maximum 24h volume
    pub max_volume_24h: Option<f64>,
    /// Minimum holders
    pub min_holders: Option<i64>,
    /// Minimum security score
    pub min_security_score: Option<i32>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisAnalyticsTimeseriesInput {
    /// Comma-separated token addresses
    pub addresses: String,
    /// Timeframe (e.g., 1h, 4h, 1d)
    pub timeframe: Option<String>,
    /// From date (ISO 8601)
    pub from_date: Option<String>,
    /// To date (ISO 8601)
    pub to_date: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisEntityIdInput {
    /// Entity ID
    pub entity_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisCategoryIdInput {
    /// Category ID
    pub category_id: String,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisVolumeTimeseriesInput {
    /// Timeframe (e.g., 1h, 4h, 1d)
    pub timeframe: Option<String>,
    /// From date (ISO 8601)
    pub from_date: Option<String>,
    /// To date (ISO 8601)
    pub to_date: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MoralisVolumeCategoryTimeseriesInput {
    /// Category ID
    pub category_id: String,
    /// Timeframe (e.g., 1h, 4h, 1d)
    pub timeframe: Option<String>,
    /// From date (ISO 8601)
    pub from_date: Option<String>,
    /// To date (ISO 8601)
    pub to_date: Option<String>,
    /// Blockchain chain (e.g., eth, polygon, bsc, arbitrum, base, optimism, avalanche). Defaults to eth.
    #[serde(default = "default_moralis_chain")]
    pub chain: String,
}

// --- Dsim ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DsimAddressInput {
    /// Address to query
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DsimTokenInfoInput {
    /// Token/contract address to query
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DsimTokenInput {
    /// Token address
    pub token: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

// --- Dune ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueryInput {
    /// Query ID
    pub query_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneSqlInput {
    /// SQL query
    pub sql: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneExecutionInput {
    /// Execution ID
    pub execution_id: String,
}

// --- Dune Queries CRUD ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesCreateInput {
    /// Query name
    pub name: String,
    /// SQL query body
    pub sql: String,
    /// Query description
    pub description: Option<String>,
    /// Make query private
    #[serde(default)]
    pub private: bool,
    /// Comma-separated tags
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesGetInput {
    /// Query ID
    pub query_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesUpdateInput {
    /// Query ID to update
    pub query_id: String,
    /// New query name
    pub name: Option<String>,
    /// New SQL query body
    pub sql: Option<String>,
    /// New description
    pub description: Option<String>,
    /// New comma-separated tags
    pub tags: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesListInput {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneQueriesIdInput {
    /// Query ID
    pub query_id: String,
}

// --- Dune Tables CRUD ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesCreateInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
    /// JSON schema definition for the table columns
    pub schema_json: String,
    /// Table description
    pub description: Option<String>,
    /// Make table private
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesUploadCsvInput {
    /// Table name
    pub table_name: String,
    /// CSV data content (or file path if --file is used)
    pub data: String,
    /// Table description
    pub description: Option<String>,
    /// Make table private
    #[serde(default)]
    pub private: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesListInput {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesGetInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesInsertInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
    /// JSON data to insert (array of row objects)
    pub data_json: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneTablesNamespaceTableInput {
    /// Namespace (e.g., your Dune username)
    pub namespace: String,
    /// Table name
    pub table_name: String,
}

// --- Dune Materialized Views ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsUpsertInput {
    /// Materialized view name
    pub name: String,
    /// Source query ID
    pub query_id: String,
    /// Cron schedule expression (e.g., "0 */6 * * *")
    pub cron: Option<String>,
    /// Expiration timestamp (ISO 8601)
    pub expires_at: Option<String>,
    /// Make materialized view private
    #[serde(default)]
    pub private: bool,
    /// Performance tier (medium, large)
    pub performance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsNameInput {
    /// Materialized view name
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsListInput {
    /// Maximum number of results to return
    pub limit: Option<u32>,
    /// Offset for pagination
    pub offset: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DuneMatviewsRefreshInput {
    /// Materialized view name
    pub name: String,
    /// Performance tier (medium, large)
    pub performance: Option<String>,
}

// --- Dune Pipelines ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DunePipelinesExecuteInput {
    /// Pipeline definition as JSON
    pub pipeline_json: String,
    /// Parameters as JSON string
    pub params: Option<String>,
    /// Performance tier (medium, large)
    pub performance: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DunePipelinesStatusInput {
    /// Pipeline execution ID
    pub execution_id: String,
}

// --- Curve Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveRouterEncodeInput {
    /// Input token address
    pub from: String,
    /// Output token address
    pub to: String,
    /// Input amount (raw, no decimals)
    pub amount: String,
    /// Minimum output amount (raw, no decimals)
    pub min_out: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveChainRegistryInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Registry ID (e.g., "main", "factory", "factory-crypto")
    pub registry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveOptionalChainInput {
    /// Chain name (optional, all chains if omitted)
    pub chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveChainAddressInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Token or contract address
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveChainAddressTimeRangeInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Token or contract address
    pub address: String,
    /// Start timestamp (unix seconds)
    pub start: Option<u64>,
    /// End timestamp (unix seconds)
    pub end: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CurveDaoLockersInput {
    /// Number of top lockers to return
    pub top: Option<u32>,
}

// --- CCXT Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtExchangeInput {
    /// Exchange name
    pub exchange: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtOhlcvInput {
    /// Exchange name
    pub exchange: String,
    /// Trading symbol
    pub symbol: String,
    /// Timeframe (e.g., "1m", "1h", "1d")
    pub timeframe: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtTradesInput {
    /// Exchange name
    pub exchange: String,
    /// Trading symbol
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtCompareInput {
    /// Trading symbol (e.g., BTC/USDT)
    pub symbol: String,
}

// --- Uniswap Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapPoolAddressInput {
    /// Pool address
    pub pool: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UniswapBalanceInput {
    /// Wallet address
    pub address: String,
    /// Token address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Kong ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongChainInput {
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongTvlInput {
    /// Vault or strategy address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongReportsInput {
    /// Report type: "vault" or "strategy"
    pub report_type: String,
    /// Vault or strategy address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KongPricesInput {
    /// Token contract address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

// --- 1inch Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchSwapInput {
    /// Source token address
    pub src: String,
    /// Destination token address
    pub dst: String,
    /// Amount
    pub amount: String,
    /// Sender address
    pub from: String,
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchChainInput {
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchAllowanceInput {
    /// Token address
    pub token: String,
    /// Owner address
    pub owner: String,
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OneinchApproveInput {
    /// Token address
    pub token: String,
    /// Amount (optional, max if omitted)
    pub amount: Option<String>,
    /// Chain ID (optional)
    pub chain_id: Option<u64>,
}

// --- OpenOcean Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenoceanSwapInput {
    /// Input token address
    pub in_token: String,
    /// Output token address
    pub out_token: String,
    /// Amount
    pub amount: String,
    /// Account address
    pub account: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct OpenoceanChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- KyberSwap ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KyberRoutesInput {
    /// Input token address
    pub token_in: String,
    /// Output token address
    pub token_out: String,
    /// Input amount
    pub amount_in: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KyberBuildInput {
    /// Source token address
    pub token_in: String,
    /// Destination token address
    pub token_out: String,
    /// Amount in smallest units (wei)
    pub amount_in: String,
    /// Sender address
    pub sender: String,
    /// Recipient address
    pub recipient: String,
    /// Route summary JSON from routes command
    pub route_summary: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- 0x Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxPriceInput {
    /// Sell token address
    pub sell_token: String,
    /// Buy token address
    pub buy_token: String,
    /// Sell amount
    pub sell_amount: String,
    /// Taker address (wallet executing the swap)
    pub taker: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ZeroxChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- CoW Swap Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapOrderInput {
    /// Order UID
    pub order_uid: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapOwnerInput {
    /// Owner address
    pub owner: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapAuctionInput {
    /// Auction ID
    pub auction_id: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapTokenInput {
    /// Token address
    pub token: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- CowSwap Order Management ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapCreateOrderInput {
    /// Source token address
    pub sell_token: String,
    /// Destination token address
    pub buy_token: String,
    /// Sell amount in smallest units (wei)
    pub sell_amount: String,
    /// Buy amount in smallest units (wei)
    pub buy_amount: String,
    /// Order expiration (Unix timestamp)
    pub valid_to: u64,
    /// Order creator address
    pub from: String,
    /// Receiver address
    pub receiver: String,
    /// Signed order data (hex)
    pub signature: String,
    /// Order kind: "sell" or "buy"
    #[serde(default = "default_order_kind")]
    pub kind: String,
    /// Signing scheme: "eip712", "eip1271", or "presign"
    #[serde(default = "default_signing_scheme")]
    pub signing_scheme: String,
    /// Fee amount in sell token (smallest units)
    #[serde(default)]
    pub fee_amount: Option<String>,
    /// App data hash (32 bytes hex)
    #[serde(default)]
    pub app_data: Option<String>,
    /// Allow partial fills
    #[serde(default)]
    pub partially_fillable: bool,
    /// Quote ID reference
    pub quote_id: Option<i64>,
    /// Chain name (ethereum, gnosis, arbitrum)
    #[serde(default = "default_chain")]
    pub chain: String,
}

fn default_order_kind() -> String {
    "sell".to_string()
}

fn default_signing_scheme() -> String {
    "eip712".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CowswapCancelOrderInput {
    /// Order UID to cancel
    pub uid: String,
    /// EIP-712 signature proving ownership
    pub signature: String,
    /// Chain name (ethereum, gnosis, arbitrum)
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- LiFi Extra ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiStatusInput {
    /// Transaction hash
    pub tx_hash: String,
    /// Bridge name (optional)
    pub bridge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiChainInput {
    /// Chain name
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiGasInput {
    /// Chain ID (e.g., "1" for Ethereum, "137" for Polygon)
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiTokensInput {
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiConnectionsInput {
    /// Source chain ID (optional)
    pub from_chain: Option<String>,
    /// Destination chain ID (optional)
    pub to_chain: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LifiStepTransactionInput {
    /// Step JSON from a route response
    pub step_json: String,
}

// --- Velora ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraPriceInput {
    /// Source token address
    pub src_token: String,
    /// Destination token address
    pub dest_token: String,
    /// Amount in smallest units (wei)
    pub amount: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraTxInput {
    /// User address (wallet executing the swap)
    pub user_address: String,
    /// Price route JSON from price command
    pub price_route: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Slippage in basis points (e.g., 100 = 1%)
    pub slippage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VeloraChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Enso ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoRouteInput {
    /// From token address
    pub from_token: String,
    /// To token address
    pub to_token: String,
    /// Amount
    pub amount: String,
    /// Sender address (required by Enso API)
    pub from_address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoPriceInput {
    /// Token address
    pub token: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoBalancesInput {
    /// Wallet address
    pub address: String,
    /// Chain ID (e.g. 1 for Ethereum, 137 for Polygon)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EnsoBundleInput {
    /// Sender address
    pub from_address: String,
    /// Actions JSON array: [{"protocol":"...","action":"...","args":{...}}]
    pub actions_json: String,
    /// Chain ID (e.g., "1" for Ethereum)
    #[serde(default = "default_chain_id")]
    pub chain_id: String,
    /// Routing strategy: router, delegate, ensowallet
    pub routing_strategy: Option<String>,
}

fn default_chain_id() -> String {
    "1".to_string()
}

// --- CCXT ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtTickerInput {
    /// Exchange name (binance, coinbase, etc.)
    pub exchange: String,
    /// Trading symbol (BTC/USDT, ETH/USD, etc.)
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CcxtOrderbookInput {
    /// Exchange name
    pub exchange: String,
    /// Trading symbol
    pub symbol: String,
    /// Depth limit
    pub limit: Option<u32>,
}

// --- Simulate ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateCallInput {
    /// Target contract address
    pub contract: String,
    /// Function signature (e.g., "transfer(address,uint256)")
    pub sig: Option<String>,
    /// Raw calldata (hex encoded, alternative to sig)
    pub data: Option<String>,
    /// Function arguments (used with sig).
    /// For array types (e.g., address[], uint256[]), pass the array in brackets WITHOUT quotes:
    ///   - address[]: `[0x1234...,0x5678...]` (no quotes, no spaces)
    ///   - uint256[]: `[1,2,3]`
    ///
    /// Example for swapExactETHForTokens(uint256,address[],address,uint256):
    /// `args: ["0", "[0xC02aaA39...,0xA0b86991...]", "0xRecipient", "999999999"]`
    #[serde(default)]
    pub args: Vec<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,

    // === Transaction parameters ===
    /// Sender address (default: zero address)
    pub from: Option<String>,
    /// Value to send in wei (default: 0)
    pub value: Option<String>,
    /// Block number or tag (latest, pending, etc.)
    pub block: Option<String>,
    /// Gas limit
    pub gas: Option<String>,
    /// Gas price in wei
    pub gas_price: Option<String>,

    // === State overrides (format: "address=value" or "address:slot=value") ===
    /// Set balance overrides (format: address=wei, can specify multiple)
    #[serde(default)]
    pub balance_overrides: Vec<String>,
    /// Set storage overrides (format: address:slot=value, can specify multiple)
    #[serde(default)]
    pub storage_overrides: Vec<String>,
    /// Set code overrides (format: address=bytecode, can specify multiple)
    #[serde(default)]
    pub code_overrides: Vec<String>,
    /// Set nonce overrides (format: address=nonce, can specify multiple)
    #[serde(default)]
    pub nonce_overrides: Vec<String>,

    // === Block overrides ===
    /// Override block timestamp (unix seconds)
    pub block_timestamp: Option<u64>,
    /// Override block number
    pub block_number_override: Option<u64>,
    /// Override block gas limit
    pub block_gas_limit: Option<String>,
    /// Override block coinbase/miner address
    pub block_coinbase: Option<String>,
    /// Override block difficulty
    pub block_difficulty: Option<String>,
    /// Override block base fee per gas (wei)
    pub block_base_fee: Option<String>,
    /// Transaction index within the block
    pub transaction_index: Option<u32>,

    // === L2 options (Optimism, etc.) ===
    /// L1 block number (for L2 simulations)
    pub l1_block_number: Option<u64>,
    /// L1 timestamp (for L2 simulations)
    pub l1_timestamp: Option<u64>,
    /// L1 message sender (for L2 cross-chain simulations)
    pub l1_message_sender: Option<String>,
    /// Mark as deposit transaction (Optimism Bedrock)
    #[serde(default)]
    pub deposit_tx: bool,
    /// Mark as system transaction (Optimism Bedrock)
    #[serde(default)]
    pub system_tx: bool,

    // === Simulation options ===
    /// Simulation backend (cast, anvil, tenderly, debug, trace, alchemy)
    pub via: Option<String>,
    /// RPC URL (for debug backend)
    pub rpc_url: Option<String>,
    /// Show execution trace
    #[serde(default)]
    pub trace: bool,
    /// Decode internal functions in Foundry traces
    #[serde(default)]
    pub decode_internal: bool,
    /// Disable address labels in Foundry traces
    #[serde(default)]
    pub disable_labels: bool,
    /// Label addresses in Foundry traces (format: address:label)
    #[serde(default)]
    pub labels: Vec<String>,
    /// EVM version for Foundry trace execution
    pub evm_version: Option<String>,
    /// Use local Foundry project artifacts for trace decoding
    #[serde(default)]
    pub with_local_artifacts: bool,
    /// RPC timeout in seconds for Foundry calls
    pub rpc_timeout: Option<u64>,
    /// Disabled over MCP because headers can carry credentials; configure RPC auth outside MCP
    #[serde(default)]
    pub rpc_headers: Vec<String>,
    /// Disable automatic proxy detection in Foundry RPC clients
    #[serde(default)]
    pub no_proxy: bool,
    /// Additional Anvil fork RPC URLs
    #[serde(default)]
    pub fork_urls: Vec<String>,
    /// Anvil fork block number or negative offset from latest
    pub fork_block_number: Option<String>,
    /// Anvil fork state after a specific transaction hash
    pub fork_transaction_hash: Option<String>,
    /// Anvil fork chain ID for offline-start mode
    pub fork_chain_id: Option<u64>,
    /// Disabled over MCP because headers can carry credentials; configure fork RPC auth outside MCP
    #[serde(default)]
    pub fork_headers: Vec<String>,
    /// Anvil hardfork to use (e.g., prague, cancun)
    pub hardfork: Option<String>,
    /// Anvil network family to enable (ethereum, optimism, tempo)
    pub network: Option<String>,
    /// Disable Anvil fork RPC rate limiting
    #[serde(default)]
    pub no_rate_limit: bool,
    /// Disable Anvil fork storage caching
    #[serde(default)]
    pub no_storage_caching: bool,
    /// Anvil upstream fork RPC timeout in milliseconds
    pub fork_timeout_ms: Option<u64>,
    /// Anvil upstream fork retry count
    pub fork_retries: Option<u32>,
    /// Disable Anvil block gas limit checks
    #[serde(default)]
    pub disable_block_gas_limit: bool,
    /// Enable Anvil transaction gas limit checks
    #[serde(default)]
    pub enable_tx_gas_limit: bool,
    /// Enable precise gas estimation (Tenderly)
    #[serde(default)]
    pub estimate_gas: bool,
    /// Generate EIP-2930 access list in response (Tenderly)
    #[serde(default)]
    pub generate_access_list: bool,
    /// Provide access list (JSON format)
    pub access_list: Option<String>,
    /// Simulation type: full, quick, or abi
    pub simulation_type: Option<String>,
    /// Network ID to simulate on
    pub network_id: Option<u64>,
    /// Save simulation to Tenderly (returns simulation ID)
    #[serde(default)]
    pub save: bool,

    // === Tenderly options ===
    /// Disabled over MCP; use TENDERLY_ACCESS_KEY env or ethcli config instead
    pub tenderly_key: Option<String>,
    /// Tenderly account slug (or use TENDERLY_ACCOUNT env)
    pub tenderly_account: Option<String>,
    /// Tenderly project slug (or use TENDERLY_PROJECT env)
    pub tenderly_project: Option<String>,

    // === Alchemy options ===
    /// Disabled over MCP; use ALCHEMY_API_KEY env or ethcli config instead
    pub alchemy_key: Option<String>,
    /// Alchemy network (e.g., eth-mainnet, polygon-mainnet, arb-mainnet)
    pub alchemy_network: Option<String>,

    // === Debug options ===
    /// Dry run - output request without executing (json, curl, fetch, powershell, url, python, httpie, wget, go, rust, axios)
    pub dry_run: Option<String>,
    /// Disabled over MCP; dry-run output is always masked
    #[serde(default)]
    pub show_secrets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SimulateTxInput {
    /// Transaction hash to simulate
    pub hash: String,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Simulation backend (cast, tenderly, debug, trace, alchemy)
    pub via: Option<String>,
    /// RPC URL
    pub rpc_url: Option<String>,
    /// Show full opcode trace
    #[serde(default)]
    pub trace: bool,
    /// Open interactive Foundry debugger
    #[serde(default)]
    pub debug: bool,
    /// Only execute with previous-block state for faster Foundry replay
    #[serde(default)]
    pub quick: bool,
    /// Decode internal functions in Foundry traces
    #[serde(default)]
    pub decode_internal: bool,
    /// Limit Foundry trace depth
    pub trace_depth: Option<u32>,
    /// Replay system transactions before the target transaction
    #[serde(default)]
    pub replay_system_txs: bool,
    /// Disable address labels in Foundry traces
    #[serde(default)]
    pub disable_labels: bool,
    /// Label addresses in Foundry traces (format: address:label)
    #[serde(default)]
    pub labels: Vec<String>,
    /// EVM version for Foundry replay
    pub evm_version: Option<String>,
    /// Use local Foundry project artifacts for trace decoding
    #[serde(default)]
    pub with_local_artifacts: bool,
    /// Disable automatic proxy detection in Foundry RPC clients
    #[serde(default)]
    pub no_proxy: bool,
    /// RPC timeout in seconds for Foundry replay
    pub rpc_timeout: Option<u64>,
    /// Disabled over MCP because headers can carry credentials; configure RPC auth outside MCP
    #[serde(default)]
    pub rpc_headers: Vec<String>,
    /// Enable transaction gas limit checks in Foundry replay
    #[serde(default)]
    pub enable_tx_gas_limit: bool,
    /// Disable block gas limit checks in Foundry replay
    #[serde(default)]
    pub disable_block_gas_limit: bool,
    /// Disabled over MCP; use ETHERSCAN_API_KEY env or ethcli config instead
    pub etherscan_api_key: Option<String>,
    /// Print raw callTracer JSON instead of the decoded call tree (debug backend)
    #[serde(default)]
    pub raw: bool,
}

// --- Config ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigKeyInput {
    /// API key value
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigTenderlyInput {
    /// Tenderly account name
    pub account: String,
    /// Tenderly project name
    pub project: String,
    /// Tenderly API key
    pub key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigChainlinkInput {
    /// Chainlink API key (client ID)
    pub key: String,
    /// Chainlink user secret (client secret)
    pub secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigDebugRpcInput {
    /// RPC URL for debug mode
    pub url: String,
}

// --- Endpoints ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsChainInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsHealthInput {
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
    /// Number of probe requests per endpoint
    #[serde(default = "default_endpoint_health_probes")]
    pub probes: u32,
}

fn default_endpoint_health_probes() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsUrlInput {
    /// RPC URL
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsAddInput {
    /// RPC URL to add
    pub url: String,
    /// Chain this endpoint serves (auto-detected if not specified)
    pub chain: Option<String>,
    /// Skip optimization (just add with defaults)
    #[serde(default)]
    pub no_optimize: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EndpointsOptimizeInput {
    /// Optional specific RPC URL to optimize (if omitted, optimizes all)
    pub url: Option<String>,
    /// Chain name
    #[serde(default = "default_chain")]
    pub chain: String,
}

// --- Chainlist ---
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistSearchInput {
    /// Search query (chain name, short name, or chain ID)
    pub query: String,
    /// Include testnet chains in results
    #[serde(default)]
    pub testnets: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistChainInput {
    /// Chain name or numeric chain ID (e.g., "gnosis", "fantom", "100")
    pub chain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistAddInput {
    /// Chain name or numeric chain ID (e.g., "gnosis", "fantom", "100")
    pub chain: String,
    /// Maximum number of endpoints to add (default 5)
    #[serde(default = "default_chainlist_max")]
    pub max: usize,
}

fn default_chainlist_max() -> usize {
    5
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ChainlistListInput {
    /// Include testnet chains in results
    #[serde(default)]
    pub testnets: bool,
}
